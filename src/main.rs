use std::{fs::File, io::Cursor};

use gif::{Encoder, Repeat};
use glium::{Surface, VertexBuffer};
use load::ObjVertex;
use petgraph::{algo, graph::NodeIndex};

mod camera;
mod frame;
mod load;
mod shader;

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("xx");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    // load assets
    let data = load::get_objdata(include_bytes!("../assets/teapot.obj")).unwrap();
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

    // output gif, used when recording in camera
    // TODO: move to camera
    let mut image = File::create("target/output.gif").unwrap();
    let mut encoder = Encoder::new(&mut image, 800, 480, &[]).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

    let vertex_graph = load::load_wavefront(&data);

    let mut new_vertex_graph = petgraph::Graph::<ObjVertex, f32>::new();
    for _ in 0..data.position.len() {
        new_vertex_graph.add_node(ObjVertex::default());
    }

    let mut start_idx = 1u32;
    let a = vertex_graph.node_indices().nth(start_idx as usize).unwrap();
    new_vertex_graph.node_weights_mut().for_each(|node| {
        *node = ObjVertex {
            position: [0.0; 3],
            normal: [0.0; 3],
            // -1 signals to red_shader to discard fragment
            texture: [-1.0, -1.0],
        }
    });

    // add the nodes of the longest continous path starting at `start_idx` to the initialized
    // vertex graph such that only this path's nodes will be rendered.
    let bellman_ford = algo::bellman_ford(&vertex_graph, a).unwrap();
    let mut prev = NodeIndex::from(start_idx);

    // parse the predecessors field to step along the path from `start_idx`
    // TODO: explain `idx` and `predecessor` relationship
    while let Some((idx, _)) = bellman_ford
        .predecessors
        .iter()
        .enumerate()
        .find(|(_, &predecessor)| predecessor == Some(prev))
    {
        *new_vertex_graph.node_weight_mut(prev).unwrap() = *vertex_graph.node_weight(prev).unwrap();
        prev = NodeIndex::from(idx as u32);
    }
    // the continous path's vertex_buffer
    let vertex_buffer_2 = VertexBuffer::new(
        &display,
        &new_vertex_graph
            .raw_nodes()
            .iter()
            .map(move |node| node.weight)
            .collect::<Vec<ObjVertex>>(),
    )
    .unwrap();

    // the actual object's `VertexBuffer`
    let vertex_buffer = VertexBuffer::new(
        &display,
        &vertex_graph
            .raw_nodes()
            .iter()
            .map(move |node| node.weight)
            .collect::<Vec<ObjVertex>>(),
    )
    .unwrap();

    // stores the faces of `vertex_buffer`
    let indices = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &load::get_indices(&data),
    )
    .unwrap();

    let mut camera = camera::CameraState::new();
    // TODO: this struct makes no sense
    let app = frame::Application::new(indices, diffuse_texture);

    // rendering loop
    event_loop
        .run(move |event, window_target| {
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => window_target.exit(),

                    // render everything
                    winit::event::WindowEvent::RedrawRequested => {
                        start_idx = (start_idx + 1) % vertex_graph.node_count() as u32;

                        let mut target = display.draw();
                        camera.update();
                        target.clear_color_and_depth((0.2, 0.2, 1.0, 1.0), 1.0);
                        app.draw_frame(
                            &mut camera,
                            &mut target,
                            &shader::red_shader(&display),
                            &vertex_buffer_2,
                        )
                        .unwrap();
                        app.draw_frame(
                            &mut camera,
                            &mut target,
                            &shader::default_program(&display),
                            &vertex_buffer,
                        )
                        .unwrap();
                        target.finish().unwrap();
                        if camera.is_recording() {
                            let mut image: glium::texture::RawImage2d<'_, u8> =
                                display.read_front_buffer().unwrap();
                            let frame = gif::Frame::from_rgba_speed(
                                image.width.try_into().unwrap(),
                                image.height.try_into().unwrap(),
                                image.data.to_mut(),
                                30,
                            );
                            encoder.write_frame(&frame).unwrap();
                        }
                    }
                    // resize the display when the window's size has changed
                    winit::event::WindowEvent::Resized(window_size) => {
                        display.resize(window_size.into())
                    }
                    // all keyboard inputs are associated to camera movements at this point, so
                    // just pass the keyboard input to camera
                    winit::event::WindowEvent::KeyboardInput { event, .. } => {
                        camera.process_input(&event)
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
