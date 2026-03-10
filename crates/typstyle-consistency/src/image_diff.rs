use tiny_skia::{Color, Pixmap};

/// Compute a diff image highlighting pixel differences.
/// Differing pixels are shown in red, identical pixels are transparent.
pub fn compute_diff_pixmap(orig_png: &Pixmap, fmt_png: &Pixmap) -> Pixmap {
    let width = orig_png.width();
    let height = orig_png.height();

    let mut diff_pixmap = Pixmap::new(width, height).unwrap();

    for y in 0..height {
        for x in 0..width {
            let orig_pixel = orig_png.pixel(x, y).unwrap();
            let fmt_pixel = fmt_png.pixel(x, y).unwrap();

            let color = if orig_pixel != fmt_pixel {
                Color::from_rgba8(255, 0, 0, 255)
            } else {
                Color::from_rgba8(0, 0, 0, 0)
            };

            diff_pixmap.pixels_mut()[(y * width + x) as usize] = color.premultiply().to_color_u8();
        }
    }

    diff_pixmap
}
