use glium::{uniform, Display, DrawParameters, Frame, Program, Surface, VertexBuffer};
use glutin::surface::WindowSurface;

use crate::{camera::State, load::ObjVertex};

// TODO: docs
pub struct Application {
    params: DrawParameters<'static>,
    light: [f32; 3],
    diffuse_texture: glium::texture::SrgbTexture2d,
    pub camera: State,
}

impl Application {
    pub fn new(d_texture: glium::texture::SrgbTexture2d) -> Self {
        Self {
            params: glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                blend: glium::Blend::alpha_blending(),
                polygon_mode: glium::PolygonMode::Fill,
                ..Default::default()
            },
            light: [1.4, 0.4, -0.7f32],
            diffuse_texture: d_texture,
            camera: State::new(),
        }
    }

    pub fn draw_frame(
        &mut self,
        target: &mut Frame,
        shader_buffers: &[&ShaderBuffer],
        display: &Display<WindowSurface>,
    ) {
        self.camera.update(display);
        let uniforms = uniform! {
            persp_matrix: self.camera.get_perspective(),
            view_matrix: self.camera.get_view(),
            u_light: self.light,
            diffuse_tex: &self.diffuse_texture,
        };

        for shader_buffer in shader_buffers {
            target
                .draw(
                    shader_buffer.vertex_buffer,
                    shader_buffer.index_buffer,
                    shader_buffer.shader,
                    &uniforms,
                    &self.params,
                )
                .unwrap();
        }
    }
}

/// Simple struct to link buffers to shaders in order to easily pass into `draw_frame`
pub struct ShaderBuffer<'a> {
    vertex_buffer: &'a VertexBuffer<ObjVertex>,
    index_buffer: &'a glium::IndexBuffer<u32>,
    shader: &'a Program,
}

impl<'a> ShaderBuffer<'_> {
    pub fn new(
        vertex_buffer: &'a VertexBuffer<ObjVertex>,
        index_buffer: &'a glium::IndexBuffer<u32>,
        shader: &'a Program,
    ) -> ShaderBuffer<'a> {
        ShaderBuffer {
            vertex_buffer,
            index_buffer,
            shader,
        }
    }
}
