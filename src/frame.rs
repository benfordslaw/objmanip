use glium::{uniform, Display, Frame, Program, Surface};
use glutin::surface::{self, WindowSurface};

use crate::{camera, load::ObjVertex, shader};

pub struct Application {
    index_buffer: glium::IndexBuffer<u16>,
    diffuse_texture: glium::texture::SrgbTexture2d,
}

impl Application {
    pub fn new(
        i_buffer: glium::IndexBuffer<u16>,
        d_texture: glium::texture::SrgbTexture2d,
    ) -> Self {
        Self {
            index_buffer: i_buffer,
            diffuse_texture: d_texture,
        }
    }

    pub fn draw_frame(
        &self,
        camera: &mut camera::CameraState,
        target: &mut Frame,
        program: &Program,
        vertex_buffer: &glium::VertexBuffer<ObjVertex>,
    ) {
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };

        let light = [1.4, 0.4, -0.7f32];

        let uniforms = uniform! {
            persp_matrix: camera.get_perspective(),
            view_matrix: camera.get_view(),
            u_light: light,
            diffuse_tex: &self.diffuse_texture,
        };

        target
            .draw(
                vertex_buffer,
                &self.index_buffer,
                program,
                &uniforms,
                &params,
            )
            .unwrap();
    }
}
