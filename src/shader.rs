use glium::{program, Display, Program};
use glutin::surface::WindowSurface;

pub fn red_shader(display: &Display<WindowSurface>) -> Program {
    program! (display,
        140 => { vertex: "
                    #version 140

                    uniform mat4 persp_matrix;
                    uniform mat4 view_matrix;

                    in vec3 position;
                    in vec3 normal;
                    in vec2 texture;

                    out vec3 v_position;
                    out vec3 v_normal;
                    out vec2 v_tex_coords;

                    void main() {
                        v_tex_coords = texture;
                        v_position = position;
                        v_normal = normal;
                        gl_Position = persp_matrix * view_matrix * vec4(v_position * 0.005, 1.0);
                    }
                ",

        fragment: "
                    #version 140

                    in vec3 v_normal;
                    in vec3 v_position;
                    in vec2 v_tex_coords;
                    out vec4 f_color;

                    uniform vec3 u_light;
                    uniform sampler2D diffuse_tex;

                    const vec3 specular_color = vec3(1.0, 1.0, 1.0);
                    vec3 diffuse_color = vec3(1.0, 0.0, 0.0);
                    vec3 ambient_color = diffuse_color * 0.1;

                    void main() {
                        float lum = max(dot(normalize(v_normal), normalize(u_light)), 0.0);
                        vec3 camera_dir = normalize(-v_position);
                        vec3 half_direction = normalize(normalize(u_light) + camera_dir);
                        float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);
                        if(v_tex_coords[0] < 0.0){
                            discard;
                        }

                        f_color = vec4(ambient_color + diffuse_color + specular * specular_color, 1.0);
                    }
                ",
    }).unwrap()
}

pub fn default_program(display: &Display<WindowSurface>) -> Program {
    program! (display,
        140 => { vertex: "
                    #version 140

                    uniform mat4 persp_matrix;
                    uniform mat4 view_matrix;

                    in vec3 position;
                    in vec3 normal;
                    in vec2 texture;

                    out vec3 v_position;
                    out vec3 v_normal;
                    out vec2 v_tex_coords;

                    void main() {
                        v_tex_coords = texture;
                        v_position = position;
                        v_normal = normal;
                        gl_Position = persp_matrix * view_matrix * vec4(v_position * 0.005, 1.0);
                    }
                ",

        fragment: "
                    #version 140

                    in vec3 v_normal;
                    in vec3 v_position;
                    in vec2 v_tex_coords;
                    out vec4 f_color;

                    uniform vec3 u_light;
                    uniform sampler2D diffuse_tex;

                    const vec3 specular_color = vec3(1.0, 1.0, 1.0);
                    vec3 diffuse_color = texture(diffuse_tex, v_tex_coords).rgb;
                    vec3 ambient_color = diffuse_color * 0.1;

                    void main() {
                        float lum = max(dot(normalize(v_normal), normalize(u_light)), 0.0);
                        vec3 camera_dir = normalize(-v_position);
                        vec3 half_direction = normalize(normalize(u_light) + camera_dir);
                        float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);

                        f_color = vec4(ambient_color + diffuse_color + specular * specular_color, 0.5);
                    }
                ",
    }).unwrap()
}
