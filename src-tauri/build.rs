fn main() {
    // Compile Swift sources for Apple Vision integration
    swift_rs::SwiftLinker::new("10.15")
        .with_package("swift-lib", "./swift-lib/")
        .link();

    tauri_build::build()
}
