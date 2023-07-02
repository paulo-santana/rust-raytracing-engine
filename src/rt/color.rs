use std::io::Write;

pub use crate::rt::vec3::Vec3 as Color;

impl Color {
    pub fn black() -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    pub fn white() -> Color {
        Color::new(1.0, 1.0, 1.0)
    }

    pub fn from_rgba(rgba: u32) -> Color {
        Color::new(
            (rgba & 0xff) as f64 / 255.999,
            (rgba >> 8 & 0xff) as f64 / 255.999,
            (rgba >> 16 & 0xff) as f64 / 255.999,
        )
    }
}

pub fn write_color<T: Write>(out: &mut T, pixel_color: &Color) {
    out.write_all(
        format!(
            "{} {} {}\n",
            (pixel_color.0[0] * 255.999) as u8,
            (pixel_color.0[1] * 255.999) as u8,
            (pixel_color.0[2] * 255.999) as u8,
        )
        .as_bytes(),
    )
    .expect("Error writing color to the output");
}

#[cfg(feature = "portable_simd")]
use std::simd::f64x4;

// static LIMIT: Simd<f64, 4> = Simd([255.999, 255.999, 255.999, 255.999]);
#[cfg(feature = "portable_simd")]
pub fn color_to_u32(pixel_color: &Color) -> u32 {
    let colors = pixel_color.as_simd();
    let limit = f64x4::splat(255.999);
    let result = (colors * limit).to_array();
    let r = (result[0]) as u32;
    let g = (result[1]) as u32;
    let b = (result[2]) as u32;

    0xff << 24 | b << 16 | g << 8 | r
}

#[cfg(not(feature = "portable_simd"))]
pub fn color_to_u32(pixel_color: &Color) -> u32 {
    let r = (pixel_color.0[0] * 255.999) as u32;
    let g = (pixel_color.0[1] * 255.999) as u32;
    let b = (pixel_color.0[2] * 255.999) as u32;

    0xff << 24 | b << 16 | g << 8 | r
}
