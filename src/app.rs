pub mod app {

    use ash::{
        extensions::{ext::DebugUtils, khr::Surface},
        vk::{self, CommandPool, DebugUtilsMessengerEXT, PhysicalDevice, Queue},
        Device, Instance,
    };
    use imgui::{ConfigFlags, FontConfig, FontGlyphRanges, FontSource, TextureId};
    use imgui_rs_vulkan_renderer::Renderer;
    use imgui_winit_support::{HiDpiMode, WinitPlatform};
    use winit::{
        event_loop::EventLoop,
        platform::unix::WindowBuilderExtUnix,
        window::{Window, WindowBuilder},
    };

    use crate::common::{App, System};

    pub struct RaytracingApp {}

    pub struct Canvas {
        pub data: Vec<u32>,
        pub width: usize,
        pub height: usize,
    }

    impl RaytracingApp {
        pub fn new() -> Self {
            return RaytracingApp {};
        }
    }

    impl App for RaytracingApp {
        fn destroy(&mut self, context: &crate::common::VulkanContext) {
            todo!()
        }
    }

    impl Canvas {
        pub fn new(width: usize, height: usize) -> Self {
            let data = vec![0; width * height];
            return Canvas {
                data,
                width,
                height,
            };
        }
    }
}
