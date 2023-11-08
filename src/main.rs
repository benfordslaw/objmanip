#![warn(clippy::pedantic)]
use std::io::Cursor;

use buffer::DisplayVertexBuffer;
use conversion::CartesianCoords;
use frame::ShaderBuffer;
use glium::{IndexBuffer, Surface, VertexBuffer};

use load::ObjVertex;
use markov::{self, Chain};

use crate::conversion::PolarCoords;

mod buffer;
mod camera;
mod conversion;
mod frame;
mod graph;
mod load;
mod shader;

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    // load assets
    let data = load::get_objdata(include_bytes!("../assets/spot_triangulated.obj")).unwrap();
    // TODO: move texture loading somewhere else
    let texture = image::load(
        Cursor::new(&include_bytes!("../assets/Epona_grp.png")),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let image_dimensions = texture.dimensions();
    let texture =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&texture.into_raw(), image_dimensions);
    let diffuse_texture = glium::texture::SrgbTexture2d::new(&display, texture).unwrap();

    let vertex_graph = graph::VertexDag::from(&data);

    let mut chain: Chain<String> = Chain::new();
    for polar_off in vertex_graph.connected_subgraph_polar_offs() {
        chain.feed(polar_off);
    }

    let vertex_buffer = vertex_graph.to_buffer(&display).unwrap();

    // stores the faces of `vertex_buffer`
    let indices = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &load::get_indices(&data),
    )
    .unwrap();

    let mut app = frame::Application::new(diffuse_texture);
    let red_shader = shader::red(&display);
    let default_shader = shader::full(&display);

    // rendering loop
    event_loop
        .run(move |event, window_target| {
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => window_target.exit(),

                    // render everything
                    winit::event::WindowEvent::RedrawRequested => {
                        let mut target = display.draw();
                        target.clear_color_and_depth((0.2, 0.2, 1.0, 1.0), 1.0);

                        // generate a new path and create a buffer
                        let gen_polar: Vec<PolarCoords> =
                            chain.generate().iter().map(PolarCoords::from).collect();
                        let mut run_pos = CartesianCoords::default();
                        let mut new_vertices = Vec::<ObjVertex>::new();
                        for gen_coord in &gen_polar {
                            run_pos.sum_with(&CartesianCoords::from(gen_coord));
                            new_vertices.push(run_pos.clone().into());
                        }
                        let generated_buffer = VertexBuffer::new(&display, &new_vertices).unwrap();
                        let generated_indices =
                            IndexBuffer::try_from(DisplayVertexBuffer(&generated_buffer, &display))
                                .unwrap();

                        let shader_buffers = &[
                            &ShaderBuffer::new(&generated_buffer, &generated_indices, &red_shader),
                            &ShaderBuffer::new(&vertex_buffer, &indices, &default_shader),
                        ];

                        app.draw_frame(&mut target, shader_buffers, &display);
                        target.finish().unwrap();
                    }
                    // resize the display when the window's size has changed
                    winit::event::WindowEvent::Resized(window_size) => {
                        display.resize(window_size.into());
                    }
                    // all keyboard inputs are associated to camera movements at this point, so
                    // just passes the keyboard input to camera
                    winit::event::WindowEvent::KeyboardInput { event, .. } => {
                        app.camera.process_input(&event);
                    }
                    _ => (),
                },
                // ensures continuous rendering
                winit::event::Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => (),
            };
        })
        .unwrap();
}
