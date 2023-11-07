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

    let data = load::get_objdata(include_bytes!("../assets/teapot.obj")).unwrap();
    let vertex_graph = load::load_wavefront(&data);

    let mut new_vertex_graph = petgraph::Graph::<ObjVertex, f32>::new();
    for _ in 0..data.position.len() {
        new_vertex_graph.add_node(ObjVertex::default());
    }

    let mut start_idx = 257u32;
    let a = vertex_graph.node_indices().nth(start_idx as usize).unwrap();
    new_vertex_graph.node_weights_mut().for_each(|node| {
        *node = ObjVertex {
            position: [0.0; 3],
            normal: [0.0; 3],
            // -1 signals to red_shader to discard fragment
            texture: [-1.0, -1.0],
        }
    });
    let b = algo::bellman_ford(&vertex_graph, a).unwrap();
    let mut prev = NodeIndex::from(start_idx);
    while let Some((x, _)) = b
        .predecessors
        .iter()
        .enumerate()
        .find(|(_, &val)| val == Some(NodeIndex::from(prev)))
    {
        *new_vertex_graph.node_weight_mut(prev).unwrap() = *vertex_graph.node_weight(prev).unwrap();
        prev = NodeIndex::from(x as u32);
    }
    println!("{:?}", prev);
    let vertex_buffer_2 = VertexBuffer::new(
        &display,
        &new_vertex_graph
            .raw_nodes()
            .iter()
            .map(move |node| node.weight)
            .collect::<Vec<ObjVertex>>(),
    )
    .unwrap();

    let vertex_buffer = VertexBuffer::new(
        &display,
        &vertex_graph
            .raw_nodes()
            .iter()
            .map(move |node| node.weight)
            .collect::<Vec<ObjVertex>>(),
    )
    .unwrap();

    let mut image = File::create("target/output.gif").unwrap();
    let mut encoder = Encoder::new(&mut image, 800, 480, &[]).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

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
    let indices = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &load::get_indices(&data),
    )
    .unwrap();

    let mut camera = camera::CameraState::new();

    let app = frame::Application::new(indices, diffuse_texture);

    event_loop
        .run(move |event, window_target| {
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        window_target.exit();
                    }

                    // We now need to render everything in response to a RedrawRequested event due to the animation
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
                        );
                        app.draw_frame(
                            &mut camera,
                            &mut target,
                            &shader::default_program(&display),
                            &vertex_buffer,
                        );
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
                    // Because glium doesn't know about windows we need to resize the display
                    // when the window's size has changed.
                    winit::event::WindowEvent::Resized(window_size) => {
                        display.resize(window_size.into());
                    }
                    // all keyboard inputs are associated to camera movements at this point
                    winit::event::WindowEvent::KeyboardInput { event, .. } => {
                        camera.process_input(&event);
                    }
                    _ => (),
                },
                // By requesting a redraw in response to a AboutToWait event we get continuous rendering.
                // For applications that only change due to user input you could remove this handler.
                winit::event::Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => (),
            };
        })
        .unwrap();
}
