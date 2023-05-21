use std::io;

use raytracing::color::{self, Color};

pub fn ratio(a: u32, b: u32) -> f64 {
    a as f64 / (b as f64 - 1.0)
}
fn main() {
    // image
    const IMAGE_WIDTH: u32 = 256;
    const IMAGE_HEIGHT: u32 = 256;

    // render
    println!("P3");
    println!("{IMAGE_WIDTH} {IMAGE_HEIGHT}");
    println!("255");

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaininig: {j} ");
        for i in 0..IMAGE_WIDTH {
            let color = Color::new(ratio(i, IMAGE_WIDTH), ratio(j, IMAGE_HEIGHT), 0.25);
            color::write_color(&mut io::stdout(), &color);
        }
    }

    eprintln!("\nDone.");
}
