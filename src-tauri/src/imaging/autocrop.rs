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
