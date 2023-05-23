use std::time::Instant;

use minifb::{Key, Window, WindowOptions};
// use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};
// use vulkano::image::view::ImageView;
// use vulkano::image::{ImageUsage, SwapchainImage};
// use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
// use vulkano::swapchain::SwapchainCreateInfo;
// use vulkano::{
//     self,
//     device::{
//         physical::{PhysicalDevice, PhysicalDeviceType},
//         DeviceExtensions, QueueFlags,
//     },
//     instance::{Instance, InstanceCreateInfo},
//     swapchain::{Surface, Swapchain},
// };
// use vulkano_win::VkSurfaceBuild;
// use winit::{
//     event::WindowEvent,
//     event_loop::{ControlFlow, EventLoop},
//     window::{Window, WindowBuilder},
// };

use raytracing::{
    color::{color_to_u32, Color},
    ray::Ray,
    vec3::Point3,
    vec3::Vec3,
};

fn ratio(a: usize, b: usize) -> f64 {
    a as f64 / (b as f64 - 1.0)
}

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

fn ray_color(ray: Ray) -> Color {
    let t = hit_sphere(&Point3(0.0, 0.0, -1.0), 0.5, &ray);
    if t > 0.0 {
        let n = Vec3::unit_vector(&(ray.at(t) - Vec3(0.0, 0.0, -1.0)));
        return 0.5 * Color(n.0 + 1.0, n.1 + 1.0, n.2 + 1.0);
    }
    let unit_direction = Color::unit_vector(&ray.direction);
    let t = 0.5 * (unit_direction.y() + 1.0);

    return (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0);
}

// fn run() {
//     // image
//     const ASPECT_RATIO: f64 = 16.0 / 9.0;
//     const IMAGE_WIDTH: u32 = 400;
//     const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32;
//
//     // camera
//     let viewport_height = 2.0;
//     let viewport_width = ASPECT_RATIO * viewport_height;
//     let focal_length = 1.0;
//
//     let origin = Point3::new(0.0, 0.0, 0.0);
//     let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
//     let vertical = Vec3::new(0.0, viewport_height, 0.0);
//     let lower_left_corner =
//         &origin - &horizontal / 2.0 - &vertical / 2.0 - Vec3(0.0, 0.0, focal_length);
//
//     // render
//     println!("P3");
//     println!("{IMAGE_WIDTH} {IMAGE_HEIGHT}");
//     println!("255");
//
//     for j in (0..IMAGE_HEIGHT).rev() {
//         eprint!("\rScanlines remaininig: {j} ");
//         for i in 0..IMAGE_WIDTH {
//             let u = ratio(i, IMAGE_WIDTH);
//             let v = ratio(j, IMAGE_HEIGHT);
//             let ray = Ray::new(
//                 origin,
//                 &lower_left_corner + u * &horizontal + v * &vertical - &origin,
//             );
//             let color = ray_color(ray);
//             color::write_color(&mut io::stdout(), &color);
//         }
//     }
//
//     eprintln!("\nDone.");
// }

// fn select_physical_device(
//     instance: &Arc<Instance>,
//     surface: &Arc<Surface>,
//     device_extensions: &DeviceExtensions,
// ) -> (Arc<PhysicalDevice>, u32) {
//     instance
//         .enumerate_physical_devices()
//         .expect("could not enumerate devices")
//         .filter(|p| p.supported_extensions().contains(&device_extensions))
//         .filter_map(|p| {
//             p.queue_family_properties()
//                 .iter()
//                 .enumerate()
//                 .position(|(i, q)| {
//                     q.queue_flags.contains(QueueFlags::GRAPHICS)
//                         && p.surface_support(i as u32, &surface).unwrap_or(false)
//                 })
//                 .map(|q| (p, q as u32))
//         })
//         .min_by_key(|(p, _)| match p.properties().device_type {
//             PhysicalDeviceType::DiscreteGpu => 0,
//             PhysicalDeviceType::IntegratedGpu => 1,
//             PhysicalDeviceType::VirtualGpu => 2,
//             PhysicalDeviceType::Cpu => 3,
//             _ => 4,
//         })
//         .expect("no deveice available")
// }

// fn get_render_pass(device: Arc<Device>, swapchain: &Arc<Swapchain>) -> Arc<RenderPass> {
//     vulkano::single_pass_renderpass!(
//         device,
//         attachments: {
//             color: {
//                 load: Clear,
//                 store: Store,
//                 format: swapchain.image_format(),
//                 samples: 1,
//             },
//         },
//         pass: {
//             color: [color],
//             depth_stencil: {},
//         },
//     )
//     .unwrap()
// }

// fn get_framebuffers(
//     images: &[Arc<SwapchainImage>],
//     render_pass: &Arc<RenderPass>,
// ) -> Vec<Arc<Framebuffer>> {
//     images
//         .iter()
//         .map(|image| {
//             let view = ImageView::new_default(image.clone()).unwrap();
//             Framebuffer::new(
//                 render_pass.clone(),
//                 FramebufferCreateInfo {
//                     attachments: vec![view],
//                     ..Default::default()
//                 },
//             )
//             .unwrap()
//         })
//         .collect()
// }

// fn vulkan() {
//     let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL");
//     let required_extensions = vulkano_win::required_extensions(&library);
//     let instance = Instance::new(
//         library,
//         InstanceCreateInfo {
//             enabled_extensions: required_extensions,
//             ..Default::default()
//         },
//     )
//     .expect("failed to create instance");
//
//     let event_loop = EventLoop::new();
//     let surface = WindowBuilder::new()
//         .build_vk_surface(&event_loop, instance.clone())
//         .unwrap();
//
//     let device_extensions = DeviceExtensions {
//         khr_swapchain: true,
//         ..DeviceExtensions::empty()
//     };
//
//     let (physical_device, queue_family_index) =
//         select_physical_device(&instance, &surface, &device_extensions);
//
//     let window = surface
//         .object()
//         .unwrap()
//         .clone()
//         .downcast::<Window>()
//         .unwrap();
//
//     let caps = physical_device
//         .surface_capabilities(&surface, Default::default())
//         .expect("failed to get surface capabilities");
//
//     let dimensions = window.inner_size();
//
//     let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
//
//     let image_format = Some(
//         physical_device
//             .surface_formats(&surface, Default::default())
//             .unwrap()[0]
//             .0,
//     );
//
//     let (device, mut queues) = Device::new(
//         physical_device.clone(),
//         DeviceCreateInfo {
//             queue_create_infos: vec![QueueCreateInfo {
//                 queue_family_index,
//                 ..Default::default()
//             }],
//             enabled_extensions: device_extensions,
//             ..Default::default()
//         },
//     )
//     .expect("failed to create device");
//
//     let queue = queues.next().unwrap();
//
//     let (swapchain, images) = Swapchain::new(
//         device.clone(),
//         surface.clone(),
//         SwapchainCreateInfo {
//             min_image_count: caps.min_image_count + 1,
//             image_format,
//             image_extent: dimensions.into(),
//             image_usage: ImageUsage::COLOR_ATTACHMENT,
//             composite_alpha,
//             ..Default::default()
//         },
//     )
//     .unwrap();
//
//     let render_pass = get_render_pass(device.clone(), &swapchain);
//     let framebuffers = get_framebuffers(&images, &render_pass);
//
//     event_loop.run(|event, _, control_flow| match event {
//         winit::event::Event::WindowEvent {
//             event: WindowEvent::CloseRequested,
//             ..
//         } => {
//             *control_flow = ControlFlow::Exit;
//         }
//         _ => (),
//     });
// }

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const WIDTH: usize = 640;
const HEIGHT: usize = (WIDTH as f64 / ASPECT_RATIO) as usize;

fn get_window() -> Window {
    Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    })
}

fn run() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = get_window();

    // camera
    let viewport_height = 2.0;
    let viewport_width = ASPECT_RATIO * viewport_height;
    let focal_length = 1.0;

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner =
        origin - horizontal / 2.0 - vertical / 2.0 - Vec3(0.0, 0.0, focal_length);

    // Limit to max ~60 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    window.limit_update_rate(Some(std::time::Duration::from_millis(10)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let u = ratio(x, WIDTH);
                let v = ratio(y, HEIGHT);
                let ray = Ray::new(
                    origin,
                    lower_left_corner + u * horizontal + v * vertical - origin,
                );
                let color = ray_color(ray);
                buffer[(HEIGHT - y - 1) * WIDTH + x] = color_to_u32(&color)
            }
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        eprint!("\rfps: {} ", 1000 / now.elapsed().as_millis());
    }

    eprintln!("\nDone.");
}

fn main() {
    // vulkan();
    run();
}
