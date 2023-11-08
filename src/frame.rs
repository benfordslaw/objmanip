use glium::{uniform, DrawParameters, Frame, Program, Surface, VertexBuffer};

use crate::{
    camera::{self, CameraState},
    load::ObjVertex,
};

pub struct Application {
    index_buffer: glium::IndexBuffer<u16>,
    params: DrawParameters<'static>,
    light: [f32; 3],
    diffuse_texture: glium::texture::SrgbTexture2d,
    pub camera: CameraState,
}

impl Application {
    pub fn new(
        i_buffer: glium::IndexBuffer<u16>,
        d_texture: glium::texture::SrgbTexture2d,
    ) -> Self {
        Self {
            index_buffer: i_buffer,
            params: glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            },
            light: [1.4, 0.4, -0.7f32],
            diffuse_texture: d_texture,
            camera: CameraState::new(),
        }
    }

    pub fn draw_frame(&mut self, target: &mut Frame, shader_buffers: &[&ShaderBuffer]) {
        self.camera.update();
        let uniforms = uniform! {
            persp_matrix: self.camera.get_perspective(),
            view_matrix: self.camera.get_view(),
            u_light: self.light,
            diffuse_tex: &self.diffuse_texture,
        };

        for shader_buffer in shader_buffers {
            target
                .draw(
                    &shader_buffer.buffer,
                    &self.index_buffer,
                    &shader_buffer.shader,
                    &uniforms,
                    &self.params,
                )
                .unwrap();
        }
    }
}

/// Simple struct to link buffers to shaders in order to easily pass pairs into `draw_frame`
pub struct ShaderBuffer {
    buffer: VertexBuffer<ObjVertex>,
    shader: Program,
}

impl ShaderBuffer {
    pub fn new(b: VertexBuffer<ObjVertex>, p: Program) -> Self {
        Self {
            buffer: b,
            shader: p,
        }
    }
}
