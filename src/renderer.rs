use rayon::prelude::*;
extern crate nalgebra_glm as glm;
use core::time;
use std::time::Instant;

use nalgebra::{Vector3, Vector4};
use serde::{Deserialize, Serialize};

use crate::{
    camera::Camera,
    rt::{color::color_to_u32, ray::Ray},
    scene::Scene,
};

pub struct RaytracingRenderer {
    pub canvas: Canvas,
    pub use_threads: bool,
}

impl RaytracingRenderer {
    pub fn on_resize(&mut self, viewport_width: u32, viewport_height: u32) {
        if self.canvas.width == viewport_width && self.canvas.height == viewport_height {
            return;
        }
        self.canvas.resize(viewport_width, viewport_height);
    }

    pub fn render(&mut self, scene: &Scene, camera: &Camera) -> time::Duration {
        let start = Instant::now();

        let render_row = |(y, row): (usize, &mut [u32])| {
            let mut ray = Ray::new(camera.position, Vector3::default());
            let offset = y as u32 * self.canvas.width;
            for x in 0..self.canvas.width {
                ray.direction = camera.get_ray_directions()[(offset + x) as usize];
                let color = trace_ray(&ray, scene);
                let color = glm::clamp(&color, 0.0, 1.0);
                row[x as usize] = color_to_u32(&color);
            }
        };

        let data = &mut self.canvas.data;
        let width = self.canvas.width as usize;

        match self.use_threads {
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

fn trace_ray(ray: &Ray, scene: &Scene) -> Vector4<f64> {
    if scene.spheres.is_empty() {
        return glm::vec4(0.0, 0.0, 0.0, 1.0);
    }

    let mut closest_sphere = None;
    let mut hit_distance = f64::MAX;
    for sphere in &scene.spheres {
        let sphere_origin = sphere.position;
        let radius = sphere.radius;

        let oc = ray.origin - sphere_origin;

        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * oc.dot(&ray.direction);
        let c = oc.dot(&oc) - radius * radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            continue;
        }

        // (-b +- sqrt(discriminant)) / 2a
        let closest_t = (-b - discriminant.sqrt()) / 2.0 * a;
        if closest_t < hit_distance {
            hit_distance = closest_t;
            closest_sphere = Some(sphere);
        }
    }

    if closest_sphere.is_none() {
        return glm::vec4(0.0, 0.0, 0.0, 1.0);
    }

    let closest_sphere = closest_sphere.unwrap();

    let origin = ray.origin - closest_sphere.position;
    let hit_point = origin + ray.direction * hit_distance;
    let normal = hit_point.normalize();

    let light_direction = glm::vec3(-1.0, -1.0, -1.0).normalize();

    let light_intensity = glm::max2_scalar(normal.dot(&-light_direction), 0.0);

    let sphere_color = closest_sphere.albedo;

    glm::vec3_to_vec4(&(sphere_color * light_intensity))
}
