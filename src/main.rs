extern crate nalgebra_glm as glm;
use nalgebra::{Vector2, Vector3};
use raytracing::camera::Camera;
use raytracing::renderer::{Canvas, RaytracingRenderer, State};
use raytracing::scene::{self, Scene, Sphere};
use std::error::Error;
use std::io::Read;
use std::{fs::File, io::Write, time::Instant};
use winit::dpi::PhysicalPosition;
use winit::event::ElementState::{Pressed, Released};
use winit::event::MouseButton::Right;
use winit::window::CursorGrabMode;

use __core::mem;
use glow::{HasContext, NativeTexture};

use imgui_glow_renderer::Renderer;

mod utils;

use imgui::*;

static DEFAULT_WIDTH: u32 = 400;
static DEFAULT_HEIGHT: u32 = 400;

use raytracing::rt::color::write_color;

fn save_ppm<T: Write>(out: &mut T, canvas: &Canvas) {
    out.write_all(format!("P3\n{} {}\n255\n", canvas.width, canvas.height).as_bytes())
        .expect("Failed writing the ppm header");
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            write_color(out, canvas.data[(y * canvas.width + x) as usize]);
        }
    }
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

    let mut camera = Camera::new(45.0, 0.1, 100.0);
    let mut scene = scene::Scene {
        spheres: vec![
            Sphere {
                position: glm::vec3(1.3, 0.0, 0.0),
                albedo: glm::vec3(1.0, 0.0, 0.5),
                ..Default::default()
            },
            Sphere {
                albedo: glm::vec3(0.3, 0.5, 0.8),
                ..Default::default()
            },
        ],
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
    let mut where_mouse_clicked = PhysicalPosition::new(200, 200);
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

                camera.on_update(
                    glm::convert(Vector2::from_row_slice(&ui.io().mouse_pos)),
                    ui.io().delta_time as f64,
                );

                if camera.state.is_active {
                    window
                        .window()
                        .set_cursor_position(where_mouse_clicked)
                        .expect("Failed to set cursor position");
                }

                textures_ui.show(ui, &mut state, &mut scene);

                textures_ui
                    .renderer
                    .on_resize(state.canvas_width, state.canvas_height);
                camera.on_resize(state.canvas_width, state.canvas_height);
                textures_ui.renderer.use_threads = state.use_threads;
                state.last_render_time = textures_ui.renderer.render(&scene, &camera);

                let texture = Program::new_texture(&textures_ui.renderer.canvas, &state, &gl);
                textures_ui.prepare_texture(texture, &mut textures, &gl);

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
            glutin::event::Event::WindowEvent {
                event:
                    glutin::event::WindowEvent::KeyboardInput {
                        input,
                        is_synthetic: false,
                        ..
                    },
                ..
            } => {
                camera.handle_input(input);
                winit_platform.handle_event(imgui_context.io_mut(), window.window(), &event);
            }

            glutin::event::Event::WindowEvent {
                event:
                    glutin::event::WindowEvent::MouseInput {
                        state,
                        button: Right,
                        ..
                    },
                ..
            } => {
                let (cursor_mode, is_active) = match state {
                    Pressed => {
                        where_mouse_clicked =
                            PhysicalPosition::from(imgui_context.io_mut().mouse_pos);
                        (CursorGrabMode::Confined, true)
                    }
                    Released => (CursorGrabMode::None, false),
                };
                camera.state.is_active = is_active;
                window.window().set_cursor_visible(!is_active);
                window
                    .window()
                    .set_cursor_grab(cursor_mode)
                    .expect("Failed to set the cursor mode to Locked");
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
    renderer: RaytracingRenderer,
    viewport_width: u32,
    viewport_height: u32,
}

impl Program {
    fn new() -> Self {
        let renderer = RaytracingRenderer {
            canvas: Canvas::new(DEFAULT_WIDTH, DEFAULT_HEIGHT),
            use_threads: false,
        };
        Self {
            generated_texture: None,
            viewport_width: 100,
            viewport_height: 100,
            renderer,
        }
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

    fn show(&mut self, ui: &imgui::Ui, state: &mut State, scene: &mut Scene) {
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

                if ui.button("Save to ppm") {
                    let mut file = File::create("canvas.ppm").expect("Failed to open 'canvas.ppm'");
                    save_ppm(&mut file, &self.renderer.canvas);
                }

                if ui.button("Save Settings") {
                    if let Err(err) = save_state(state) {
                        state.error_msg = format!("Failed saving state: {:?}", err);
                    }
                }

                if !state.error_msg.is_empty() {
                    ui.text_colored([1.0, 0.4, 0.1, 1.0], &state.error_msg);
                }
            });

        ui.window("Scene").build(|| {
            scene
                .spheres
                .iter_mut()
                .enumerate()
                .for_each(|(i, sphere)| {
                    // ui.input_scalar_n(format!("sphere {i}"), sphere.position.as_mut_slice())
                    //     .build();
                    Drag::new(format!("sphere {i}"))
                        .range(-100.0, 100.0)
                        .speed(0.1)
                        .build_array(ui, sphere.position.as_mut_slice());
                    Drag::new(format!("radius {i}"))
                        .range(0.1, 100.0)
                        .speed(0.1)
                        .build(ui, &mut sphere.radius);
                    let a = glm::convert::<Vector3<f64>, Vector3<f32>>(sphere.albedo);
                    let mut colors = [a.x, a.y, a.z];
                    ui.color_edit3(format!("albedo {i}"), &mut colors);
                    sphere.albedo =
                        glm::convert::<Vector3<f32>, Vector3<f64>>(Vector3::from(colors));
                    ui.separator();
                });
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
