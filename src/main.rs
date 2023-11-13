#![warn(clippy::pedantic)]

mod camera;
mod frame;
mod geometry;
mod load;
mod shader;

fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    let mut app = frame::Application::new(include_bytes!("../assets/r6.obj"), &display);

    // rendering loop
    event_loop
        .run(move |event, window_target| {
            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => window_target.exit(),

                    // render everything
                    winit::event::WindowEvent::RedrawRequested => {
                        app.draw_frame();
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
