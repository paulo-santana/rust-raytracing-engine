use glutin::{event_loop::EventLoop, GlRequest};
use imgui::ConfigFlags;
use imgui_winit_support::WinitPlatform;

pub type Window = glutin::WindowedContext<glutin::PossiblyCurrent>;

pub fn create_window(title: &str, gl_request: GlRequest) -> (EventLoop<()>, Window) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new()
        .with_title(title)
        .with_maximized(true);
    // .with_inner_size(glutin::dpi::LogicalSize::new(1024, 768));
    let window = glutin::ContextBuilder::new()
        .with_gl(gl_request)
        // .with_vsync(true)
        .build_windowed(window, &event_loop)
        .expect("could not create window");
    let window = unsafe {
        window
            .make_current()
            .expect("could not make window context current")
    };
    (event_loop, window)
}

pub fn glow_context(window: &Window) -> glow::Context {
    unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s).cast()) }
}

pub fn imgui_init(window: &Window) -> (WinitPlatform, imgui::Context) {
    let mut imgui_context = imgui::Context::create();
    // imgui_context.set_ini_filename(None);

    let mut winit_platform = WinitPlatform::init(&mut imgui_context);
    winit_platform.attach_window(
        imgui_context.io_mut(),
        window.window(),
        imgui_winit_support::HiDpiMode::Rounded,
    );

    imgui_context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;
    imgui_context
        .io_mut()
        .config_flags
        .insert(ConfigFlags::DOCKING_ENABLE);
    imgui_context
        .io_mut()
        .config_flags
        .insert(ConfigFlags::VIEWPORTS_ENABLE);

    (winit_platform, imgui_context)
}