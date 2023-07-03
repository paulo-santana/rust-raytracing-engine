use rayon::prelude::*;
extern crate nalgebra_glm as glm;
use core::time;
use std::time::Instant;

use nalgebra::{Vector3, Vector4};
use serde::{Deserialize, Serialize};

use crate::{
    camera::Camera,
    rt::{color::color_to_u32, ray::Ray},
};

pub struct RaytracingRenderer {
    pub canvas: Canvas,
}

impl RaytracingRenderer {
    pub fn render(&mut self, state: &mut State) {
        let start = Instant::now();
        if self.canvas.data.len() != (self.canvas.width * self.canvas.height) as usize {
            self.canvas = Canvas::new(self.canvas.width, self.canvas.height);
        }

        let camera = Camera::new(
            Vector3::new(0.0, 0.0, 0.0),
            self.canvas.width,
            self.canvas.height,
        );

        match state.use_threads {
            true => {
                self.canvas
                    .data
                    .par_chunks_mut(self.canvas.width as usize)
                    .enumerate()
                    .for_each(|(y, row)| {
                        for x in 0..self.canvas.width {
                            // let y = self.canvas.height - row_number as u32 - 1;
                            let ray = camera.ray_to_coordinate(x, y as u32);
                            let color = ray_color(ray);
                            row[x as usize] = color_to_u32(&color);
                            // assign_color(&mut self.canvas, x, y, &color);
                        }
                    });
            }
            false => {
                let half_width = self.canvas.width as f64 / 2.0;
                let half_heigth = self.canvas.height as f64 / 2.0;
                for y in 0..self.canvas.height {
                    let cy = y as f64 / half_heigth - 1.0;
                    let offset = y * self.canvas.width;
                    for x in 0..self.canvas.width {
                        let cx = x as f64 / half_width - 1.0;
                        let color = per_pixel(
                            cx,
                            cy,
                            &nalgebra::convert(Vector4::from_row_slice(&state.sphere_color)),
                        );
                        let color = glm::clamp(&color, 0.0, 1.0);
                        self.canvas.data[(offset + x) as usize] = color_to_u32(&color);
                    }
                }
            }
        }

        state.last_render_time = start.elapsed();
    }
}

pub struct Canvas {
    pub data: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        Canvas {
            data: vec![0; (width * height) as usize],
            width,
            height,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub use_linear_filter: bool,
    pub use_threads: bool,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub sphere_color: [f32; 4],
    #[serde(skip_serializing, skip_deserializing)]
    pub last_render_time: time::Duration,
    #[serde(skip_serializing, skip_deserializing)]
    pub error_msg: String,
}

impl Default for State {
    fn default() -> Self {
        State {
            use_linear_filter: false,
            use_threads: false,
            canvas_width: 400,
            canvas_height: 270,
            sphere_color: [1.0; 4],
            last_render_time: time::Duration::ZERO,
            error_msg: String::default(),
        }
    }
}

#[inline]
fn hit_sphere(center: &Vector3<f64>, radius: f64, ray: &Ray) -> f64 {
    let oc = ray.origin - *center;
    let a = ray.direction.dot(&ray.direction);
    let half_b = oc.dot(&ray.direction);
    let c = oc.dot(&oc) - radius * radius;

    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        -1.0
    } else {
        (-half_b - discriminant.sqrt()) / a
    }
}

#[inline]
fn ray_color(ray: Ray) -> Vector4<f64> {
    let t = hit_sphere(&Vector3::new(0.0, 0.0, -1.0), 0.5, &ray);
    if t >= 0.0 {
        let mut color = Vector4::new(1.0, 0.0, 1.0, 1.0);
        let normal = Vector3::normalize(&(ray.at(t) - Vector3::new(0.0, 0.0, -1.0)));
        let light_direction = Vector3::normalize(&Vector3::new(-1.0, -1.0, -1.0));
        let d = f64::max(normal.dot(&-light_direction), 0.0);

        color *= d;
        return color;
    }
    // return Color::black();
    let unit_direction = Vector3::normalize(&ray.direction);
    let t = 0.5 * (unit_direction.y + 1.0);

    (1.0 - t) * Vector4::new(1.0, 1.0, 1.0, 1.0) + t * Vector4::new(0.5, 0.7, 1.0, 1.0)
}

#[inline(never)]
fn per_pixel(x: f64, y: f64, sphere_color: &Vector4<f64>) -> Vector4<f64> {
    let sphere_origin = Vector3::new(0.0, 0.0, 0.0);
    let radius = 0.5;

    let ray_origin = Vector3::new(0.0, 0.0, 1.0);
    let ray_direction = Vector3::normalize(&Vector3::new(x, y, -1.0));

    let oc = ray_origin - sphere_origin;

    let a = ray_direction.dot(&ray_direction);
    let b = 2.0 * oc.dot(&ray_direction);
    let c = oc.dot(&oc) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return Vector4::new(0.0, 0.0, 0.0, 1.0);
    }

    // (-b +- sqrt(discriminant)) / 2a
    let closest_t = (-b - discriminant.sqrt()) / 2.0 * a;

    let hit_point = ray_origin + ray_direction * closest_t;
    let normal = hit_point.normalize();

    let light_direction = glm::vec3(-1.0, -1.0, -1.0).normalize();

    let d = glm::max2_scalar(normal.dot(&-light_direction), 0.0);

    sphere_color * d
}
