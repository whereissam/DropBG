import Foundation
import Vision
import CoreImage
import AppKit

/// Remove background using Apple Vision framework.
/// Takes a file path to an image, returns PNG data with alpha channel as a byte array.
/// Uses VNGenerateForegroundInstanceMaskRequest (macOS 14+) with fallback to
/// VNGeneratePersonSegmentationRequest (macOS 12+).
@_cdecl("apple_vision_remove_background")
func appleVisionRemoveBackground(pathPtr: UnsafePointer<CChar>, outLen: UnsafeMutablePointer<Int>) -> UnsafeMutablePointer<UInt8>? {
    let path = String(cString: pathPtr)

    guard let nsImage = NSImage(contentsOfFile: path) else {
        outLen.pointee = 0
        return nil
    }

    guard let cgImage = nsImage.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
        outLen.pointee = 0
        return nil
    }

    let width = cgImage.width
    let height = cgImage.height

    // Try foreground instance mask (macOS 14+), fall back to person segmentation (macOS 12+)
    var maskImage: CGImage?

    if #available(macOS 14.0, *) {
        maskImage = generateForegroundMask(cgImage: cgImage)
    }

    if maskImage == nil {
        if #available(macOS 12.0, *) {
            maskImage = generatePersonMask(cgImage: cgImage, width: width, height: height)
        }
    }

    guard let mask = maskImage else {
        outLen.pointee = 0
        return nil
    }

    // Apply mask as alpha channel to original image
    guard let resultData = applyMaskAsAlpha(original: cgImage, mask: mask, width: width, height: height) else {
        outLen.pointee = 0
        return nil
    }

    // Convert to PNG
    let ciImage = CIImage(cgImage: resultData)
    let context = CIContext()
    let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!

    guard let pngData = context.pngRepresentation(of: ciImage, format: .RGBA8, colorSpace: colorSpace) else {
        outLen.pointee = 0
        return nil
    }

    // Copy to a buffer that Rust can free
    let count = pngData.count
    let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: count)
    pngData.copyBytes(to: buffer, count: count)
    outLen.pointee = count
    return buffer
}

/// Free the buffer allocated by apple_vision_remove_background
@_cdecl("apple_vision_free_buffer")
func appleVisionFreeBuffer(ptr: UnsafeMutablePointer<UInt8>?, len: Int) {
    if let ptr = ptr {
        ptr.deallocate()
    }
}

/// Check if Apple Vision foreground segmentation is available
@_cdecl("apple_vision_available")
func appleVisionAvailable() -> Bool {
    if #available(macOS 14.0, *) {
        return true
    }
    if #available(macOS 12.0, *) {
        return true // Person segmentation at least
    }
    return false
}

// MARK: - Private helpers

@available(macOS 14.0, *)
private func generateForegroundMask(cgImage: CGImage) -> CGImage? {
    let request = VNGenerateForegroundInstanceMaskRequest()
    let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])

    do {
        try handler.perform([request])
    } catch {
        return nil
    }

    guard let result = request.results?.first else {
        return nil
    }

    do {
        let maskPixelBuffer = try result.generateScaledMaskForImage(
            forInstances: result.allInstances,
            from: handler
        )
        let ciMask = CIImage(cvPixelBuffer: maskPixelBuffer)
        let context = CIContext()
        return context.createCGImage(ciMask, from: ciMask.extent)
    } catch {
        return nil
    }
}

@available(macOS 12.0, *)
private func generatePersonMask(cgImage: CGImage, width: Int, height: Int) -> CGImage? {
    let request = VNGeneratePersonSegmentationRequest()
    request.qualityLevel = .accurate
    request.outputPixelFormat = kCVPixelFormatType_OneComponent8

    let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])

    do {
        try handler.perform([request])
    } catch {
        return nil
    }

    guard let result = request.results?.first else {
        return nil
    }

    let maskBuffer = result.pixelBuffer
    let ciMask = CIImage(cvPixelBuffer: maskBuffer)

    // Scale mask to match original image dimensions
    let scaleX = CGFloat(width) / ciMask.extent.width
    let scaleY = CGFloat(height) / ciMask.extent.height
    let scaledMask = ciMask.transformed(by: CGAffineTransform(scaleX: scaleX, y: scaleY))

    let context = CIContext()
    return context.createCGImage(scaledMask, from: CGRect(x: 0, y: 0, width: width, height: height))
}

private func applyMaskAsAlpha(original: CGImage, mask: CGImage, width: Int, height: Int) -> CGImage? {
    let colorSpace = CGColorSpace(name: CGColorSpace.sRGB)!
    let bytesPerPixel = 4
    let bytesPerRow = width * bytesPerPixel

    guard let context = CGContext(
        data: nil,
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: bytesPerRow,
        space: colorSpace,
        bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
    ) else {
        return nil
    }

    let rect = CGRect(x: 0, y: 0, width: width, height: height)

    // Draw original image
    context.draw(original, in: rect)

    guard let pixelData = context.data else { return nil }
    let pixels = pixelData.bindMemory(to: UInt8.self, capacity: width * height * 4)

    // Get mask pixel data
    guard let maskContext = CGContext(
        data: nil,
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: width,
        space: CGColorSpaceCreateDeviceGray(),
        bitmapInfo: CGImageAlphaInfo.none.rawValue
    ) else {
        return nil
    }

    maskContext.draw(mask, in: rect)
    guard let maskData = maskContext.data else { return nil }
    let maskPixels = maskData.bindMemory(to: UInt8.self, capacity: width * height)

    // Apply mask as alpha channel
    for i in 0..<(width * height) {
        let alpha = maskPixels[i]
        let pixelIdx = i * 4
        // Un-premultiply, set alpha, re-premultiply
        let oldAlpha = pixels[pixelIdx + 3]
        if oldAlpha > 0 {
            let r = UInt16(pixels[pixelIdx]) * 255 / UInt16(oldAlpha)
            let g = UInt16(pixels[pixelIdx + 1]) * 255 / UInt16(oldAlpha)
            let b = UInt16(pixels[pixelIdx + 2]) * 255 / UInt16(oldAlpha)
            pixels[pixelIdx] = UInt8(min(r * UInt16(alpha) / 255, 255))
            pixels[pixelIdx + 1] = UInt8(min(g * UInt16(alpha) / 255, 255))
            pixels[pixelIdx + 2] = UInt8(min(b * UInt16(alpha) / 255, 255))
        }
        pixels[pixelIdx + 3] = alpha
    }

    return context.makeImage()
}
