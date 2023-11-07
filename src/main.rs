use std::{fs::File, io::Cursor};

use gif::{Encoder, Repeat};
use glium::{Surface, VertexBuffer};
use glutin::display::GetGlDisplay;
use load::ObjVertex;
use petgraph::{algo, graph::NodeIndex};

mod camera;
mod frame;
mod graph;
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
    let continuous_buffer = vertex_graph.continuous_path_from(1u32, &display).unwrap();
    let vertex_buffer = vertex_graph.to_buffer(&display).unwrap();

    // stores the faces of `vertex_buffer`
    let indices = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &load::get_indices(&data),
    )
    .unwrap();

    let mut camera = camera::CameraState::new();
    // TODO: this struct makes little sense
    let app = frame::Application::new(indices, diffuse_texture);

    // rendering loop
    event_loop
        .run(move |event, window_target| {
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => window_target.exit(),

                    // render everything
                    winit::event::WindowEvent::RedrawRequested => {
                        let mut target = display.draw();
                        camera.update();
                        target.clear_color_and_depth((0.2, 0.2, 1.0, 1.0), 1.0);
                        // TODO: maybe the vertex_buffer input should be a vec to allow
                        // rendering of multiple vertex buffers on the same frame
                        app.draw_frame(
                            &mut camera,
                            &mut target,
                            &shader::red_shader(&display),
                            &continuous_buffer,
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

                        // TODO: move to camera
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
