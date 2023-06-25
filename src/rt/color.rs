use std::io::Write;

pub use crate::rt::vec3::Vec3 as Color;

impl Color {
    pub fn black() -> Color {
        Color(0.0, 0.0, 0.0)
    }

    pub fn white() -> Color {
        Color(1.0, 1.0, 1.0)
    }

    pub fn from_rgba(rgba: u32) -> Color {
        Color(
            (rgba & 0xff) as f64 / 255.999,
            (rgba >> 8 & 0xff) as f64 / 255.999,
            (rgba >> 16 & 0xff) as f64 / 255.999,
        )
    }
}

pub fn write_color<T: Write>(out: &mut T, pixel_color: &Color) {
    out.write(
        format!(
            "{} {} {}\n",
            (pixel_color.0 * 255.999) as u8,
            (pixel_color.1 * 255.999) as u8,
            (pixel_color.2 * 255.999) as u8,
        )
        .as_bytes(),
    )
    .expect("Error writing color to the output");
}

pub fn color_to_u32(pixel_color: &Color) -> u32 {
    let r = (pixel_color.0 * 255.999) as u32;
    let g = (pixel_color.1 * 255.999) as u32;
    let b = (pixel_color.2 * 255.999) as u32;

    return 0xff << 24 | b << 16 | g << 8 | r;
}
