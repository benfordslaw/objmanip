use glium::{uniforms::EmptyUniforms, Display, DrawParameters, IndexBuffer, Surface, VertexBuffer};
use glutin::surface::WindowSurface;
use obj::ObjData;
use rustc_hash::FxHashSet;

use crate::{
    camera::State,
    geometry::winding,
    load::{self, obj_triangles, ObjVertex},
    shader,
};
use glam::Vec3;
use rayon::prelude::*;

pub struct Application {
    params: DrawParameters<'static>,
    display: Display<WindowSurface>,
    obj_data: ObjData,
    pub camera: State,
}

impl Application {
    pub fn new(data: &[u8], display: &Display<WindowSurface>) -> Self {
        let loaded_data = load::get_objdata(data).unwrap();
        Self {
            params: glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                polygon_mode: glium::PolygonMode::Fill,
                ..Default::default()
            },
            display: display.clone(),
            obj_data: loaded_data,
            camera: State::new(),
        }
    }

    pub fn draw_frame(&mut self) {
        let mut target = self.display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        self.camera.update(&self.display);

        let matrix = self.camera.get_perspective() * self.camera.get_view();

        // perspective-transform position data from obj file
        let pos_matrix: Vec<Vec3> = self
            .obj_data
            .position
            .par_iter()
            .map(|&position| matrix.transform_vector3(Vec3::from_array(position) * 20.0))
            .collect();

        // perform backface culling by testing clockwiseness
        let indices: Vec<u32> = obj_triangles(&self.obj_data)
            .par_iter()
            .filter(|triangle| {
                winding(
                    &triangle
                        .iter()
                        .map(|&vertex| *pos_matrix.get(vertex).unwrap())
                        .collect::<Vec<Vec3>>(),
                ) < 0.0
            })
            .flat_map(|triangle| triangle.par_iter().map(|&idx| u32::try_from(idx).unwrap()))
            .collect();

        // visible vertices must occur at least once in a non-culled triangle
        let valid_vertices = FxHashSet::<usize>::from_par_iter(
            indices.par_iter().map(|&x| usize::try_from(x).unwrap()),
        );

        // create vertex buffer of only non-culled vertices
        let pos_matrix = pos_matrix
            .iter()
            .enumerate()
            .map(|(idx, position)| {
                if valid_vertices.contains(&idx) {
                    ObjVertex {
                        position: position.to_array(),
                        normal: [0.0; 3],
                        texture: [0.0; 2],
                    }
                } else {
                    ObjVertex {
                        position: [0.0; 3],
                        normal: [0.0; 3],
                        texture: [-1.0; 2],
                    }
                }
            })
            .collect::<Vec<ObjVertex>>();
        let vertex_buffer = VertexBuffer::new(&self.display, &pos_matrix).unwrap();

        // create index buffer of only non-culled triangles
        let index_buffer = IndexBuffer::new(
            &self.display,
            glium::index::PrimitiveType::TrianglesList,
            &indices,
        )
        .unwrap();

        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &shader::full(&self.display),
                &EmptyUniforms,
                &self.params,
            )
            .unwrap();
        target.finish().unwrap();
    }
}
