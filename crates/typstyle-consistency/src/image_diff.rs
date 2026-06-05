use tiny_skia::{Color, Pixmap, PremultipliedColorU8};

/// Compute a diff image highlighting pixel differences.
/// Removed pixels are orange, added pixels are blue, color changes are magenta,
/// and identical pixels are transparent.
pub fn compute_diff_pixmap(orig_png: &Pixmap, fmt_png: &Pixmap) -> Pixmap {
    let width = orig_png.width().max(fmt_png.width());
    let height = orig_png.height().max(fmt_png.height());

    let mut diff_pixmap = Pixmap::new(width, height).unwrap();

    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) as usize;
            let orig_pixel = orig_png
                .pixel(x, y)
                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
            let fmt_pixel = fmt_png
                .pixel(x, y)
                .unwrap_or(PremultipliedColorU8::TRANSPARENT);

            let color = if orig_pixel != fmt_pixel {
                match orig_pixel.alpha().cmp(&fmt_pixel.alpha()) {
                    std::cmp::Ordering::Greater => Color::from_rgba8(230, 159, 0, 255),
                    std::cmp::Ordering::Less => Color::from_rgba8(0, 114, 178, 255),
                    std::cmp::Ordering::Equal => Color::from_rgba8(204, 121, 167, 255),
                }
            } else {
                Color::from_rgba8(0, 0, 0, 0)
            };

            diff_pixmap.pixels_mut()[i] = color.premultiply().to_color_u8();
        }
    }

    diff_pixmap
}
