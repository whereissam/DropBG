use image::RgbaImage;

#[allow(dead_code)]
pub fn autocrop(img: &RgbaImage) -> (u32, u32, u32, u32) {
    let (w, h) = img.dimensions();
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;

    for y in 0..h {
        for x in 0..w {
            let alpha = img.get_pixel(x, y)[3];
            if alpha > 10 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    if max_x < min_x || max_y < min_y {
        return (0, 0, w, h);
    }

    (min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid(w: u32, h: u32, alpha: u8) -> RgbaImage {
        let mut img = RgbaImage::new(w, h);
        for px in img.pixels_mut() {
            *px = Rgba([255, 255, 255, alpha]);
        }
        img
    }

    #[test]
    fn fully_opaque_returns_full_bounds() {
        let img = solid(10, 8, 255);
        assert_eq!(autocrop(&img), (0, 0, 10, 8));
    }

    #[test]
    fn fully_transparent_falls_back_to_full_bounds() {
        let img = solid(10, 8, 0);
        assert_eq!(autocrop(&img), (0, 0, 10, 8));
    }

    #[test]
    fn alpha_below_threshold_treated_as_transparent() {
        let img = solid(10, 8, 10);
        assert_eq!(autocrop(&img), (0, 0, 10, 8));
    }

    #[test]
    fn single_opaque_pixel_localized() {
        let mut img = solid(20, 20, 0);
        img.put_pixel(7, 5, Rgba([255, 0, 0, 255]));
        assert_eq!(autocrop(&img), (7, 5, 1, 1));
    }

    #[test]
    fn rectangular_opaque_region_bounded_tightly() {
        let mut img = solid(20, 20, 0);
        for y in 4..=9 {
            for x in 3..=12 {
                img.put_pixel(x, y, Rgba([0, 255, 0, 255]));
            }
        }
        // x: 3..=12 → 10 wide, y: 4..=9 → 6 tall
        assert_eq!(autocrop(&img), (3, 4, 10, 6));
    }
}
