use nalgebra::{Vector3, Vector4};
use rayon::prelude::*;
use raytracing::renderer::Canvas;
use std::error::Error;
use std::io::Read;
use std::{fs::File, io::Write, time::Instant};

use __core::{mem, time};
use glow::{HasContext, NativeTexture};

use imgui_glow_renderer::Renderer;

mod utils;

use imgui::*;

use raytracing::rt::{
    color::{color_to_u32, write_color},
    ray::Ray,
};
use serde::{Deserialize, Serialize};

struct Camera {
    position: Vector3<f64>,
    canvas_width: u32,
    canvas_height: u32,
    horizontal: Vector3<f64>,
    vertical: Vector3<f64>,
    lower_left_corner: Vector3<f64>,
}

impl Camera {
    fn new(position: Vector3<f64>, canvas_width: u32, canvas_height: u32) -> Camera {
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

    fn ray_to_coordinate(&self, x: u32, y: u32) -> Ray {
        let u = ratio(x, self.canvas_width);
        let v = ratio(y, self.canvas_height);

        Ray::new(
            self.position,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.position,
        )
    }
}

#[derive(Serialize, Deserialize)]
struct State {
    use_linear_filter: bool,
    use_threads: bool,
    canvas_width: u32,
    canvas_height: u32,
    #[serde(skip_serializing, skip_deserializing)]
    last_render_time: time::Duration,
    #[serde(skip_serializing, skip_deserializing)]
    error_msg: String,
}

impl Default for State {
    fn default() -> Self {
        State {
            use_linear_filter: false,
            use_threads: false,
            canvas_width: 400,
            canvas_height: 270,
            last_render_time: time::Duration::ZERO,
            error_msg: String::default(),
        }
    }
}

fn save_ppm<T: Write>(out: &mut T, canvas: &Canvas) {
    out.write_all(format!("P3\n{} {}\n255\n", canvas.width, canvas.height).as_bytes())
        .expect("Failed writing the ppm header");
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            write_color(out, canvas.data[(y * canvas.width + x) as usize]);
        }
    }
}

#[inline]
fn ratio(a: u32, b: u32) -> f64 {
    a as f64 / (b as f64 - 1.0)
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
fn per_pixel(x: f64, y: f64) -> u32 {
    let sphere_origin = Vector3::new(1.0, 0.0, 0.0);
    let radius = 0.5;

    let ray_origin = Vector3::new(0.0, 0.0, 2.0);
    let ray_direction = Vector3::normalize(&Vector3::new(x, y, -1.0));

    let oc = ray_origin - sphere_origin;

    let a = ray_direction.dot(&ray_direction);
    let b = 2.0 * oc.dot(&ray_direction);
    let c = oc.dot(&oc) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;
    if discriminant >= 0.0 {
        return 0xffff00ff;
    }
    0xff000000
}

fn save_state(state: &State) -> Result<(), Box<dyn Error>> {
    let state = serde_yaml::to_string(state)?;
    let mut save_file = File::create("state.yaml")?;
    save_file.write_all(state.as_bytes())?;
    Ok(())
}

fn load_state() -> Result<State, Box<dyn Error>> {
    let mut state_file = File::open("state.yaml")?;
    let mut content = String::default();
    state_file.read_to_string(&mut content)?;
    let state: State = serde_yaml::from_str(&content)?;
    Ok(state)
}

fn main() {
    let mut state = match load_state() {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Failed to read state file: {}", err);
            State {
                error_msg: format!("Failed to read state file: {}\nusing default settings", err),
                ..State::default()
            }
        }
    };
    let (event_loop, window) = utils::create_window("Custom textures", glutin::GlRequest::Latest);
    let (mut winit_platform, mut imgui_context) = utils::imgui_init(&window);
    let gl = utils::glow_context(&window);
    // This time, we tell OpenGL this is an sRGB framebuffer and OpenGL will
    // do the conversion to sSGB space for us after the fragment shader.
    unsafe { gl.enable(glow::FRAMEBUFFER_SRGB) };

    let mut textures = imgui::Textures::<glow::Texture>::default();
    // Note that `output_srgb` is `false`. This is because we set
    // `glow::FRAMEBUFFER_SRGB` so we don't have to manually do the conversion
    // in the shader.
    let mut ig_renderer = Renderer::initialize(&gl, &mut imgui_context, &mut textures, false)
        .expect("failed to create renderer");
    let mut textures_ui = Program::new();

    let mut last_frame = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Note we can potentially make the loop more efficient by
        // changing the `Poll` (default) value to `ControlFlow::Wait`
        // but be careful to test on all target platforms!
        *control_flow = glutin::event_loop::ControlFlow::Poll;

        match event {
            glutin::event::Event::NewEvents(_) => {
                let now = Instant::now();
                imgui_context
                    .io_mut()
                    .update_delta_time(now.duration_since(last_frame));
                last_frame = now;
            }
            glutin::event::Event::MainEventsCleared => {
                winit_platform
                    .prepare_frame(imgui_context.io_mut(), window.window())
                    .unwrap();

                window.window().request_redraw();
            }
            glutin::event::Event::RedrawRequested(_) => {
                unsafe { gl.clear(glow::COLOR_BUFFER_BIT) };

                let ui = imgui_context.frame();
                textures_ui.show(ui, &mut state, &mut textures, &gl);

                winit_platform.prepare_render(ui, window.window());
                let draw_data = imgui_context.render();
                ig_renderer
                    .render(&gl, &textures, draw_data)
                    .expect("error rendering imgui");

                window.swap_buffers().unwrap();
            }
            glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = glutin::event_loop::ControlFlow::Exit;
            }
            glutin::event::Event::LoopDestroyed => {
                ig_renderer.destroy(&gl);
            }
            event => {
                winit_platform.handle_event(imgui_context.io_mut(), window.window(), &event);
            }
        }
    });
}

struct Program {
    generated_texture: Option<imgui::TextureId>,
    canvas: Canvas,
    viewport_width: u32,
    viewport_height: u32,
}

impl Program {
    fn new() -> Self {
        Self {
            generated_texture: None,
            viewport_width: 100,
            viewport_height: 100,
            canvas: Canvas::new(100, 100),
        }
    }

    fn render(&mut self, state: &mut State) {
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
                for y in 0..self.canvas.height {
                    let cy = y as f64 / self.canvas.height as f64 * 2.0 - 1.0;
                    let offset = y * self.canvas.width;
                    for x in 0..self.canvas.width {
                        let cx = x as f64 / self.canvas.width as f64 * 2.0 - 1.0;
                        self.canvas.data[(offset + x) as usize] = per_pixel(cx, cy);
                    }
                }
            }
        }

        state.last_render_time = start.elapsed();
    }

    fn prepare_texture(
        &mut self,
        texture: NativeTexture,
        textures: &mut imgui::Textures<glow::Texture>,
        gl: &glow::Context,
    ) {
        if let Some(generated_texture) = self.generated_texture {
            let old_texture = textures.get(generated_texture).unwrap();

            unsafe { gl.delete_texture(*old_texture) };
            textures.replace(generated_texture, texture);
        } else {
            let id = textures.insert(texture);
            self.generated_texture = Some(id);
        }
    }

    fn new_texture(canvas: &Canvas, state: &State, gl: &glow::Context) -> NativeTexture {
        let gl_texture = unsafe { gl.create_texture() }.expect("unable to create GL texture");
        let data = unsafe { mem::transmute::<&[u32], &[u8]>(&canvas.data) };
        let filter = match state.use_linear_filter {
            true => glow::LINEAR as _,
            false => glow::NEAREST as _,
        };
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(gl_texture));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, filter);

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, filter);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::SRGB8 as _, // We are working on sRGB color space somehow
                canvas.width as _,
                canvas.height as _,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(data),
            )
        }
        gl_texture
    }

    fn show(
        &mut self,
        ui: &imgui::Ui,
        state: &mut State,
        textures: &mut imgui::Textures<glow::Texture>,
        gl: &glow::Context,
    ) {
        ui.dockspace_over_main_viewport();

        ui.window("Settings")
            .size([400.0, 400.0], Condition::FirstUseEver)
            .build(|| {
                ui.text(format!(
                    "width: {}, height: {}",
                    self.viewport_width, self.viewport_height
                ));
                ui.text(format!("last render time: {:?}", state.last_render_time));
                ui.text(format!("FPS: {}", 1.0 / ui.io().delta_time));
                ui.checkbox("Use linear filter", &mut state.use_linear_filter);
                ui.checkbox("Use multithreaded rendering", &mut state.use_threads);
                Drag::new("Canvas width")
                    .range(20, self.viewport_width)
                    .speed(1.0)
                    .build(ui, &mut state.canvas_width);
                Drag::new("Canvas height")
                    .range(20, self.viewport_height)
                    .speed(1.0)
                    .build(ui, &mut state.canvas_height);

                self.canvas.width = state.canvas_width;
                self.canvas.height = state.canvas_height;

                if ui.button("Save to ppm") {
                    let mut file = File::create("canvas.ppm").expect("Failed to open 'canvas.ppm'");
                    save_ppm(&mut file, &self.canvas);
                }

                if ui.button("Save Settings") {
                    if let Err(err) = save_state(state) {
                        state.error_msg = format!("Failed saving state: {:?}", err);
                    }
                }

                if !state.error_msg.is_empty() {
                    ui.text_colored([1.0, 0.4, 0.1, 1.0], &state.error_msg);
                }

                self.render(state);
                let texture = Self::new_texture(&self.canvas, state, gl);
                self.prepare_texture(texture, textures, gl);
            });

        let token = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
        ui.window("Viewport")
            .size([400.0, 400.0], Condition::FirstUseEver)
            .build(|| {
                let [width, height] = ui.content_region_avail();
                self.viewport_width = width as u32;
                self.viewport_height = height as u32;
                if let Some(texture) = self.generated_texture {
                    imgui::Image::new(texture, [width, height])
                        .uv0([0.0, 1.0])
                        .uv1([1.0, 0.0])
                        .build(ui);
                }
            });
        token.pop();
    }
}
