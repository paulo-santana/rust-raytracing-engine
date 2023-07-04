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
    pub fn on_resize(&mut self, viewport_width: u32, viewport_height: u32) {
        if self.canvas.width == viewport_width && self.canvas.height == viewport_height {
            return;
        }
        self.canvas.resize(viewport_width, viewport_height);
    }

    pub fn render(&mut self, state: &State, camera: &Camera) -> time::Duration {
        let start = Instant::now();

        let render_row = |(y, row): (usize, &mut [u32])| {
            let mut ray = Ray::new(camera.position, Vector3::default());
            let offset = y as u32 * self.canvas.width;
            for x in 0..self.canvas.width {
                ray.direction = camera.get_ray_directions()[(offset + x) as usize];
                let color = trace_ray(
                    &ray,
                    &nalgebra::convert(Vector4::from_row_slice(&state.sphere_color)),
                );
                let color = glm::clamp(&color, 0.0, 1.0);
                row[x as usize] = color_to_u32(&color);
            }
        };

        let data = &mut self.canvas.data;
        let width = self.canvas.width as usize;

        match state.use_threads {
            true => data.par_chunks_mut(width).enumerate().for_each(render_row),
            false => data.chunks_mut(width).enumerate().for_each(render_row),
        };

        start.elapsed()
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

    pub fn resize(&mut self, width: u32, height: u32) {
        self.data.resize((width * height) as usize, 0);
        self.width = width;
        self.height = height;
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

#[inline(never)]
fn trace_ray(ray: &Ray, sphere_color: &Vector4<f64>) -> Vector4<f64> {
    let sphere_origin = Vector3::new(0.0, 0.0, 0.0);
    let radius = 0.5;

    let oc = ray.origin - sphere_origin;

    let a = ray.direction.dot(&ray.direction);
    let b = 2.0 * oc.dot(&ray.direction);
    let c = oc.dot(&oc) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return Vector4::new(0.0, 0.0, 0.0, 1.0);
    }

    // (-b +- sqrt(discriminant)) / 2a
    let closest_t = (-b - discriminant.sqrt()) / 2.0 * a;

    let hit_point = ray.origin + ray.direction * closest_t;
    let normal = hit_point.normalize();

    let light_direction = glm::vec3(-1.0, -1.0, -1.0).normalize();

    let d = glm::max2_scalar(normal.dot(&-light_direction), 0.0);

    sphere_color * d
}
