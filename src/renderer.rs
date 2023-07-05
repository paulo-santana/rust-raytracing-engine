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

pub struct Canvas {
    pub data: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
struct HitPayload {
    hit_distance: f64,
    world_position: Vector3<f64>,
    world_normal: Vector3<f64>,
    object_index: usize,
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
            for x in 0..self.canvas.width {
                let color = Self::per_pixel(x, y as u32, self.canvas.width, camera, scene);
                let color = glm::clamp(&color, 0.0, 1.0);
                row[x as usize] = color_to_u32(&color);
            }
        };

        // let data = &mut self.canvas.data;
        let width = self.canvas.width as usize;

        match self.use_threads {
            true => self
                .canvas
                .data
                .par_chunks_mut(width)
                .enumerate()
                .for_each(render_row),
            false => self
                .canvas
                .data
                .chunks_mut(width)
                .enumerate()
                .for_each(render_row),
        };

        start.elapsed()
    }

    // RayGen
    pub fn per_pixel(x: u32, y: u32, width: u32, camera: &Camera, scene: &Scene) -> Vector4<f64> {
        let ray = Ray {
            origin: camera.position,
            direction: camera.get_ray_directions()[(x + y * width) as usize],
        };

        let payload = Self::trace_ray(&ray, scene);

        if payload.hit_distance == f64::MAX {
            return glm::vec4(0.0, 0.0, 0.0, 1.0);
        }

        let light_direction = glm::vec3(-1.0, -1.0, -1.0).normalize();

        let light_intensity = glm::max2_scalar(payload.world_normal.dot(&-light_direction), 0.0);

        let closest_sphere = &scene.spheres[payload.object_index];

        let sphere_color = closest_sphere.albedo;

        glm::vec3_to_vec4(&(sphere_color * light_intensity))
    }

    fn trace_ray(ray: &Ray, scene: &Scene) -> HitPayload {
        let mut closest_sphere_index = usize::MAX;
        let mut hit_distance = f64::MAX;
        for (i, sphere) in scene.spheres.iter().enumerate() {
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
                closest_sphere_index = i;
            }
        }

        if closest_sphere_index == usize::MAX {
            return Self::miss(ray);
        }

        Self::closest_hit(ray, scene, closest_sphere_index, hit_distance)
    }

    fn closest_hit(ray: &Ray, scene: &Scene, object_index: usize, hit_distance: f64) -> HitPayload {
        let closest_sphere = &scene.spheres[object_index];
        let origin = ray.origin - closest_sphere.position;
        let hit_point = origin + ray.direction * hit_distance;
        let normal = hit_point.normalize();

        HitPayload {
            hit_distance,
            world_position: hit_point + closest_sphere.position,
            world_normal: normal,
            object_index,
        }
    }

    fn miss(ray: &Ray) -> HitPayload {
        HitPayload {
            hit_distance: f64::MAX,
            ..Default::default()
        }
    }
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
