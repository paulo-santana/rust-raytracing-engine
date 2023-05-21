use std::io;

use raytracing::{
    color::{self, Color},
    ray::Ray,
    vec3::Point3,
    vec3::Vec3,
};

fn ratio(a: u32, b: u32) -> f64 {
    a as f64 / (b as f64 - 1.0)
}

fn ray_color(ray: Ray) -> Color {
    let unit_direction = Color::unit_vector(&ray.direction);
    let t = 0.5 * (unit_direction.y() + 1.0);

    return (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);
}

fn main() {
    // image
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: u32 = 400;
    const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32;

    // camera
    let viewport_height = 2.0;
    let viewport_width = ASPECT_RATIO * viewport_height;
    let focal_length = 1.0;

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner =
        &origin - &horizontal / 2.0 - &vertical / 2.0 - Vec3(0.0, 0.0, focal_length);

    // render
    println!("P3");
    println!("{IMAGE_WIDTH} {IMAGE_HEIGHT}");
    println!("255");

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaininig: {j} ");
        for i in 0..IMAGE_WIDTH {
            let u = ratio(i, IMAGE_WIDTH);
            let v = ratio(j, IMAGE_HEIGHT);
            let ray = Ray::new(
                origin,
                &lower_left_corner + u * &horizontal + v * &vertical - &origin,
            );
            let color = ray_color(ray);
            color::write_color(&mut io::stdout(), &color);
        }
    }

    eprintln!("\nDone.");
}
