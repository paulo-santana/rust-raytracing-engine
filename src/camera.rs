use nalgebra::Vector3;

use crate::rt::ray::Ray;

pub struct Camera {
    position: Vector3<f64>,
    canvas_width: u32,
    canvas_height: u32,
    horizontal: Vector3<f64>,
    vertical: Vector3<f64>,
    lower_left_corner: Vector3<f64>,
}

impl Camera {
    pub fn new(position: Vector3<f64>, canvas_width: u32, canvas_height: u32) -> Camera {
        let viewport_height = 2.0;
        let aspect_ratio = canvas_width as f64 / canvas_height as f64;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.0;

        let horizontal = Vector3::new(viewport_width, 0.0, 0.0);
        let vertical = Vector3::new(0.0, viewport_height, 0.0);
        let lower_left_corner =
            position - horizontal / 2.0 - vertical / 2.0 - Vector3::new(0.0, 0.0, focal_length);

        Camera {
            position,
            canvas_width,
            canvas_height,
            horizontal,
            vertical,
            lower_left_corner,
        }
    }

    pub fn ray_to_coordinate(&self, x: u32, y: u32) -> Ray {
        let u = ratio(x, self.canvas_width);
        let v = ratio(y, self.canvas_height);

        Ray::new(
            self.position,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.position,
        )
    }
}

#[inline]
fn ratio(a: u32, b: u32) -> f64 {
    a as f64 / (b as f64 - 1.0)
}
