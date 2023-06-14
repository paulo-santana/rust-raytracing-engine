#![allow(unused)]

mod common;
use common::*;
use simple_logger::SimpleLogger;

use std::{
    error::Error,
    time::{Duration, Instant},
};

use imgui::*;
use imgui_rs_vulkan_renderer::*;

use raytracing::{
    color::{color_to_u32, Color},
    ray::Ray,
    vec3::Point3,
    vec3::Vec3,
};

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const HEIGHT: usize = 540;
const WIDTH: usize = (HEIGHT as f64 * ASPECT_RATIO) as usize;

#[inline]
fn ratio(a: usize, b: usize) -> f64 {
    a as f64 / (b as f64 - 1.0)
}

#[inline]
fn hit_sphere(center: &Point3, radius: f64, ray: &Ray) -> f64 {
    let oc = ray.origin - *center;
    let a = ray.direction.lenght_squared();
    let half_b = oc.dot(&ray.direction);
    let c = oc.lenght_squared() - radius * radius;

    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        return -1.0;
    } else {
        return (-half_b - discriminant.sqrt()) / a;
    }
}

#[inline]
fn ray_color(ray: Ray) -> Color {
    let t = hit_sphere(&Point3(0.0, 0.0, -1.0), 0.5, &ray);
    if t > 0.0 {
        let n = Vec3::unit_vector(&(ray.at(t) - Vec3(0.0, 0.0, -1.0)));
        return 0.5 * Color(n.0 + 1.0, n.1 + 1.0, n.2 + 1.0);
    }
    return Color::black();
    let unit_direction = Color::unit_vector(&ray.direction);
    let t = 0.5 * unit_direction.y() + 0.5;

    return (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);
}

#[inline]
fn copy_pixel_data(buffer: &[u32]) -> Vec<u8> {
    buffer.iter().map(|x| x.to_be_bytes()).flatten().collect()
}

// fn copy_pixel_data<'a>(buffer: &[u32], dest: &'a mut [u8]) -> &'a [u8] {
//     unsafe {
//         let mut ptr = dest.as_mut_ptr() as *mut u32;
//         for i in 0..(buffer.len() as isize) {
//             *(ptr).offset(i) = buffer[i as usize].to_be();
//         }
//     }
//     return dest;
// }

fn assign_color(buffer: &mut [u32], x: usize, y: usize, color: &Color) {
    buffer[(HEIGHT - 1 - y) * WIDTH + x] = color_to_u32(color);
}

fn run() {
    let mut buffer = [0; WIDTH * HEIGHT];
    let mut pixel_data: Vec<u8> = vec![0; WIDTH * 4 * HEIGHT];

    // camera
    let viewport_height = 2.0;
    let viewport_width = ASPECT_RATIO * viewport_height;
    let focal_length = 1.0;

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner =
        origin - horizontal / 2.0 - vertical / 2.0 - Vec3(0.0, 0.0, focal_length);

    let mut milis = 0;
    let mut frames_per_half_second = 0;
    'running: loop {
        let now = Instant::now();

        // if handle_events(&mut event_pump) == true {
        //     break 'running;
        // }

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let u = ratio(x, WIDTH);
                let v = ratio(y, HEIGHT);
                let ray = Ray::new(
                    origin,
                    lower_left_corner + u * horizontal + v * vertical - origin,
                );
                let color = ray_color(ray);
                assign_color(&mut buffer, x, y, &color);
            }
        }

        let data = copy_pixel_data(&buffer);
        // let data = copy_pixel_data(&buffer, &mut pixel_data);

        milis += now.elapsed().as_millis();
        frames_per_half_second += 1;
        if (milis >= 500) {
            eprint!("\rfps: {} ", frames_per_half_second * 2);

            frames_per_half_second = 0;
            milis = 0;
        }
    }

    eprintln!("\nDone.");
}

fn open_window() -> Result<(), Box<dyn Error>> {
    // SimpleLogger::new().init()?;
    System::new("Iala kkk")?.run((), |run, ui, _| {
        ui.dockspace_over_main_viewport();
        ui.show_demo_window(&mut true);
        ui.window("Teste")
            .size([400.0, 400.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Oba");
            });

        ui.end_frame_early();
    })?;

    return Ok(());
}

fn main() {
    // vulkan();
    open_window().expect("Failed to open window");
    // run();
}
