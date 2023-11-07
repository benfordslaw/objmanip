use glium::{uniform, DrawParameters, Frame, Program, Surface};

use crate::{camera, load::ObjVertex};

pub struct Application {
    index_buffer: glium::IndexBuffer<u16>,
    params: DrawParameters<'static>,
    light: [f32; 3],
    diffuse_texture: glium::texture::SrgbTexture2d,
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
        }
    }

    pub fn draw_frame(
        &self,
        camera: &mut camera::CameraState,
        target: &mut Frame,
        program: &Program,
        vertex_buffer: &glium::VertexBuffer<ObjVertex>,
    ) -> Result<(), glium::DrawError> {
        // must be re-calculated each redraw due to camera movement
        let uniforms = uniform! {
            persp_matrix: camera.get_perspective(),
            view_matrix: camera.get_view(),
            u_light: self.light,
            diffuse_tex: &self.diffuse_texture,
        };

        target.draw(
            vertex_buffer,
            &self.index_buffer,
            program,
            &uniforms,
            &self.params,
        )
    }
}
