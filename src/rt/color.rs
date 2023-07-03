use nalgebra::Vector4;
use std::io::Write;

pub fn write_color<T: Write>(out: &mut T, pixel_color: u32) {
    out.write_all(
        format!(
            "{} {} {}\n",
            (pixel_color & 0xff) as u8,
            (pixel_color >> 8 & 0xff) as u8,
            (pixel_color >> 16 & 0xff) as u8,
        )
        .as_bytes(),
    )
    .expect("Error writing color to the output");
}

#[cfg(not(feature = "portable_simd"))]
pub fn color_to_u32(pixel_color: &Vector4<f64>) -> u32 {
    let r = (pixel_color[0] * 255.0) as u32;
    let g = (pixel_color[1] * 255.0) as u32;
    let b = (pixel_color[2] * 255.0) as u32;
    let a = (pixel_color[3] * 255.0) as u32;

    a << 24 | b << 16 | g << 8 | r
}
