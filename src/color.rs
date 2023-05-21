use std::io::Write;

pub use crate::vec3::Vec3 as Color;

impl Color {
    pub fn black() -> Color {
        Color(0.0, 0.0, 0.0)
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
