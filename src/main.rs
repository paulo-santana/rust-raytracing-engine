#![allow(unused)]

mod app;
mod common;
use app::app::{Canvas, RaytracingApp};
use ash::{vk, Device, Entry, Instance};
use common::System;

use simple_logger::SimpleLogger;

use std::{
    error::Error,
    time::{Duration, Instant},
};

use imgui::*;
use imgui_rs_vulkan_renderer::vulkan::{
    create_vulkan_descriptor_pool, create_vulkan_descriptor_set,
    create_vulkan_descriptor_set_layout,
};

use raytracing::rt::{
    color::{color_to_u32, Color},
    ray::Ray,
    vec3::Point3,
    vec3::Vec3,
};

use crate::common::Texture;

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

fn render(canvas: &mut Canvas) {
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

    for y in 0..canvas.height {
        for x in 0..canvas.width {
            let u = ratio(x, WIDTH);
            let v = ratio(y, HEIGHT);
            let ray = Ray::new(
                origin,
                lower_left_corner + u * horizontal + v * vertical - origin,
            );
            let color = ray_color(ray);
            assign_color(&mut canvas.data, x, y, &color);
        }
    }
}

fn get_texture(
    canvas: &Canvas,
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    textures: &mut Textures<vk::DescriptorSet>,
) -> TextureId {
    println!("Creating new texture... ");

    let memory_properties =
        unsafe { instance.get_physical_device_memory_properties(physical_device) };
    let data = unsafe { std::mem::transmute::<&[u32], &[u8]>(&canvas.data) };

    let my_texture = Texture::from_rgba8(
        device,
        queue,
        command_pool,
        memory_properties,
        canvas.width as u32,
        canvas.height as u32,
        data,
    )
    .expect("Failed to create texture from rgba8");

    let descriptor_set_layout = create_vulkan_descriptor_set_layout(device).unwrap();

    let descriptor_pool = create_vulkan_descriptor_pool(device, 1).unwrap();

    let descriptor_set = create_vulkan_descriptor_set(
        device,
        descriptor_set_layout,
        descriptor_pool,
        my_texture.image_view,
        my_texture.sampler,
    )
    .unwrap();

    let texture_id = textures.insert(descriptor_set);

    println!("Done: {}", texture_id.id());

    return texture_id;
}

fn replace_texture(
    id: TextureId,
    canvas: &Canvas,
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    textures: &mut Textures<vk::DescriptorSet>,
) {
    let memory_properties =
        unsafe { instance.get_physical_device_memory_properties(physical_device) };
    let data = unsafe { std::mem::transmute::<&[u32], &[u8]>(canvas.data.as_ref()) };

    let my_texture = Texture::from_rgba8(
        device,
        queue,
        command_pool,
        memory_properties,
        canvas.width as u32,
        canvas.height as u32,
        data,
    )
    .expect("Failed to create texture from rgba8");

    let descriptor_set_layout = create_vulkan_descriptor_set_layout(device).unwrap();

    let descriptor_pool = create_vulkan_descriptor_pool(device, 1).unwrap();

    let descriptor_set = create_vulkan_descriptor_set(
        device,
        descriptor_set_layout,
        descriptor_pool,
        my_texture.image_view,
        my_texture.sampler,
    )
    .unwrap();

    textures.replace(id, descriptor_set);
}

fn open_window() -> Result<(), Box<dyn Error>> {
    // SimpleLogger::new().init()?;
    let mut system = System::new("Iala kkk")?;

    let my_app = RaytracingApp::new();

    let mut show = false;
    let mut canvas = Canvas::new(400, 400);
    let mut texture_id = TextureId::new(0);
    system.run(my_app, move |run, ui, app, vulkan_context, textures| {
        ui.dockspace_over_main_viewport();
        let region = ui.window("Output").build(|| {
            if texture_id.id() == 0 {
                texture_id = get_texture(
                    &canvas,
                    &vulkan_context.instance,
                    &vulkan_context.device,
                    vulkan_context.physical_device,
                    vulkan_context.graphics_queue,
                    vulkan_context.command_pool,
                    textures,
                );
            }

            Image::new(texture_id, [canvas.width as f32, canvas.height as f32]).build(ui);

            return ui.content_region_avail();
        });
        ui.window("Settings")
            .size([400.0, 400.0], Condition::FirstUseEver)
            .build(|| {
                if ui.button("Render") {
                    let [width, height] = region.unwrap();
                    if width as usize != canvas.width || height as usize != canvas.height {
                        canvas = Canvas::new(width as usize, height as usize);
                    }
                    render(&mut canvas);
                    show = !show;
                }
            });
    })?;

    return Ok(());
}

fn main() {
    // vulkan();
    open_window().expect("Failed to open window");
    // run();
}
