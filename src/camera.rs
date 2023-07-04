extern crate nalgebra_glm as glm;
use nalgebra::{Matrix4, Vector2, Vector3};
use winit::event::{
    ElementState::{Pressed, Released},
    KeyboardInput, VirtualKeyCode,
};

use crate::rt::ray::Ray;

pub struct CameraState {
    up_speed: f64,
    down_speed: f64,
    left_speed: f64,
    right_speed: f64,
    forward_speed: f64,
    backward_speed: f64,
    pub is_active: bool,
}

pub struct Camera {
    pub position: Vector3<f64>,
    pub forward_direction: Vector3<f64>,
    vertical_fov: f64,
    near_clip: f64,
    far_clip: f64,
    view: Matrix4<f64>,
    projection: Matrix4<f64>,
    inverse_view: Matrix4<f64>,
    inverse_projection: Matrix4<f64>,
    last_mouse_pos: Vector2<f64>,
    viewport_width: u32,
    viewport_height: u32,
    horizontal: Vector3<f64>,
    vertical: Vector3<f64>,
    lower_left_corner: Vector3<f64>,
    pub state: CameraState,
    ray_directions: Vec<Vector3<f64>>,
}

impl Camera {
    pub fn new(vertical_fov: f64, near_clip: f64, far_clip: f64) -> Camera {
        let direction = glm::vec3(0.0, 0.0, -1.0);
        let position = glm::vec3(0.0, 0.0, 3.0);

        let viewport_height = 2.0;
        let aspect_ratio = 1.0;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.0;

        let horizontal = Vector3::new(viewport_width, 0.0, 0.0);
        let vertical = Vector3::new(0.0, viewport_height, 0.0);
        let lower_left_corner =
            position - horizontal / 2.0 - vertical / 2.0 - Vector3::new(0.0, 0.0, focal_length);

        Camera {
            position,
            forward_direction: direction,
            vertical_fov,
            near_clip,
            far_clip,
            last_mouse_pos: glm::vec2(0.0, 0.0),
            view: Matrix4::from_row_slice(&[1.0; 16]),
            projection: Matrix4::from_row_slice(&[1.0; 16]),
            inverse_view: Matrix4::from_row_slice(&[1.0; 16]),
            inverse_projection: Matrix4::from_row_slice(&[1.0; 16]),
            viewport_width: 400,
            viewport_height: 400,
            horizontal,
            vertical,
            lower_left_corner,
            ray_directions: Vec::with_capacity(400 * 400),
            state: CameraState {
                up_speed: 0.0,
                down_speed: 0.0,
                left_speed: 0.0,
                right_speed: 0.0,
                forward_speed: 0.0,
                backward_speed: 0.0,
                is_active: false,
            },
        }
    }

    pub fn ray_to_coordinate(&self, x: u32, y: u32) -> Ray {
        let u = ratio(x, self.viewport_width);
        let v = ratio(y, self.viewport_height);

        Ray::new(
            self.position,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.position,
        )
    }

    pub fn on_update(&mut self, mouse_pos: Vector2<f64>, ts: f64) {
        let curr_position = self.position;

        let delta = (mouse_pos - self.last_mouse_pos) * 0.002;
        if !self.state.is_active {
            self.last_mouse_pos = mouse_pos;
            return;
        }

        let mut moved = false;

        let up_direction = glm::vec3(0.0, 1.0, 0.0);
        let right_direction = glm::cross(&self.forward_direction, &up_direction);

        self.position +=
            self.forward_direction * (self.state.forward_speed - self.state.backward_speed) * ts;
        self.position += right_direction * (self.state.right_speed - self.state.left_speed) * ts;
        self.position += up_direction * (self.state.up_speed - self.state.down_speed) * ts;

        if curr_position != self.position {
            moved = true;
        }

        if delta.x != 0.0 || delta.y != 0.0 {
            let pitch_delta = delta.y * self.get_rotation_speed();
            let way_delta = delta.x * self.get_rotation_speed();
            let pitch_angle = glm::quat_angle_axis(-pitch_delta, &right_direction);
            let yaw_angle = glm::quat_angle_axis(-way_delta, &up_direction);
            let cross = glm::quat_cross(&pitch_angle, &yaw_angle);
            let q = cross.normalize();

            self.forward_direction = glm::quat_rotate_vec3(&q, &self.forward_direction);
            moved = true;
        }

        if moved {
            self.recalculate_view();
            self.recalculate_ray_directions();
        }
    }

    pub fn on_resize(&mut self, width: u32, height: u32) {
        if width == self.viewport_width && height == self.viewport_height {
            return;
        }

        self.viewport_width = width;
        self.viewport_height = height;

        self.recalculate_projection();
        self.recalculate_ray_directions();
    }

    pub fn handle_input(&mut self, input: KeyboardInput) {
        let speed = 5.0;
        match input.state {
            Pressed => match input.virtual_keycode {
                Some(VirtualKeyCode::W) => self.state.forward_speed = speed,
                Some(VirtualKeyCode::S) => self.state.backward_speed = speed,
                Some(VirtualKeyCode::A) => self.state.left_speed = speed,
                Some(VirtualKeyCode::D) => self.state.right_speed = speed,
                Some(VirtualKeyCode::Q) => self.state.down_speed = speed,
                Some(VirtualKeyCode::E) => self.state.up_speed = speed,
                _ => (),
            },

            Released => match input.virtual_keycode {
                Some(VirtualKeyCode::W) => self.state.forward_speed = 0.0,
                Some(VirtualKeyCode::S) => self.state.backward_speed = 0.0,
                Some(VirtualKeyCode::A) => self.state.left_speed = 0.0,
                Some(VirtualKeyCode::D) => self.state.right_speed = 0.0,
                Some(VirtualKeyCode::Q) => self.state.down_speed = 0.0,
                Some(VirtualKeyCode::E) => self.state.up_speed = 0.0,
                _ => (),
            },
        }
    }

    pub fn get_ray_directions(&self) -> &Vec<Vector3<f64>> {
        &self.ray_directions
    }

    pub fn get_rotation_speed(&self) -> f64 {
        0.3
    }

    fn recalculate_projection(&mut self) {
        self.projection = glm::perspective_fov(
            glm::radians(&glm::vec1(self.vertical_fov)).x,
            self.viewport_width as f64,
            self.viewport_height as f64,
            self.near_clip,
            self.far_clip,
        );
        self.inverse_projection = glm::inverse(&self.projection);
    }

    fn recalculate_view(&mut self) {
        self.view = glm::look_at(
            &self.position,
            &(self.position + self.forward_direction),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        self.inverse_view = glm::inverse(&self.view);
    }

    fn recalculate_ray_directions(&mut self) {
        self.ray_directions.resize(
            (self.viewport_width * self.viewport_height) as usize,
            Vector3::default(),
        );

        for y in 0..self.viewport_height {
            for x in 0..self.viewport_width {
                let mut coord = glm::vec2(
                    x as f64 / self.viewport_width as f64,
                    y as f64 / self.viewport_height as f64,
                );
                coord = coord * 2.0 - glm::vec2(1.0, 1.0); // expand to (-1, 1)

                let target = self.inverse_projection * glm::vec4(coord.x, coord.y, 1.0, 1.0);

                let mut a = glm::vec4_to_vec3(&target) / target.w;
                a.normalize_mut();
                let a = a.insert_row(3, 0.0);

                let a = self.inverse_view * a;

                let ray_direction = glm::vec4_to_vec3(&a);

                self.ray_directions[(x + y * self.viewport_width) as usize] = ray_direction;
            }
        }
    }
}

#[inline]
fn ratio(a: u32, b: u32) -> f64 {
    a as f64 / (b as f64 - 1.0)
}
