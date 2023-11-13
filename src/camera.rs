use std::{f32::consts::PI, fs::File};

use gif::Encoder;
use glam::Mat4;
use glium::Display;
use glutin::surface::WindowSurface;

pub struct State {
    view: Mat4,
    perspective: Mat4,
    camera_t: f32,
    recording_since: f32,
    encoder: Encoder<File>,

    // vertical, forward, horiz. rotation
    motion: (i32, i32, i32),
}

impl State {
    pub fn new() -> Self {
        Self {
            view: Mat4::default(),
            perspective: Mat4::default(),
            camera_t: PI / 2.0,
            recording_since: -1.0,
            encoder: Encoder::new(File::create("target/output.gif").unwrap(), 800, 400, &[])
                .unwrap(),

            motion: (0, 0, 0),
        }
    }

    /// checks with some margin of error
    pub fn is_recording(&self) -> bool {
        (self.recording_since + 1.0).abs() > 0.001
    }

    pub fn get_perspective(&self) -> Mat4 {
        self.perspective
    }

    pub fn get_view(&self) -> Mat4 {
        self.view
    }

    // FIXME: such a mess
    pub fn update(&mut self, display: &Display<WindowSurface>) {
        if self.motion.2 > 0 {
            self.view *= Mat4::from_rotation_y(0.01);
        }
        if self.motion.2 < 0 {
            self.view *= Mat4::from_rotation_y(-0.01);
        }

        // stop recording on full rotation
        if self.is_recording() && self.camera_t >= self.recording_since + (2.0 * PI) {
            self.recording_since = -1.0;
        }

        if self.motion.0 > 0 {
            self.perspective *= Mat4::from_rotation_x(0.01);
        }

        if self.motion.0 < 0 {
            self.perspective *= Mat4::from_rotation_x(-0.01);
        }

        // FIXME: strangeness going on with recording being lighter after first frame
        if self.is_recording() {
            let mut image: glium::texture::RawImage2d<'_, u8> =
                display.read_front_buffer().unwrap();
            let frame = gif::Frame::from_rgba_speed(
                image.width.try_into().unwrap(),
                image.height.try_into().unwrap(),
                image.data.to_mut(),
                30,
            );
            self.encoder.write_frame(&frame).unwrap();
        }
    }

    pub fn process_input(&mut self, event: &winit::event::KeyEvent) {
        use winit::keyboard::{KeyCode, PhysicalKey};
        let pressed: i32 = (event.state == winit::event::ElementState::Pressed).into();
        match &event.physical_key {
            PhysicalKey::Code(KeyCode::ArrowUp) => self.motion.0 = pressed,
            PhysicalKey::Code(KeyCode::ArrowDown) => self.motion.0 = -pressed,
            PhysicalKey::Code(KeyCode::KeyW) => self.motion.1 = pressed,
            PhysicalKey::Code(KeyCode::KeyS) => self.motion.1 = -pressed,
            PhysicalKey::Code(KeyCode::KeyA) => self.motion.2 = pressed,
            PhysicalKey::Code(KeyCode::KeyD) => self.motion.2 = -pressed,
            PhysicalKey::Code(KeyCode::KeyM) => self.recording_since = self.camera_t,
            _ => (),
        };
    }
}
