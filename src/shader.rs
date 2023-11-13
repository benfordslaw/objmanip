use glium::{program, Display, Program};
use glutin::surface::WindowSurface;

pub fn full(display: &Display<WindowSurface>) -> Program {
    program! (display,
        140 => { vertex: "
                    #version 140

                    in vec3 position;
                    in vec3 normal;
                    in vec2 texture;

                    void main() {
                        gl_Position = vec4(vec3(position), 1.0);
                    }
                ",

        fragment: "
                    #version 140

                    out vec4 f_color;

                    void main() {
                        f_color = vec4(vec3(1.0-gl_FragCoord.z), 1.0);
                    }
                ",
    })
    .unwrap()
}
