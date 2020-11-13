#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 v_tex_coords;
layout(location=2) in vec4 a_color;
layout(location=0) out vec4 v_color;
layout(location=1) out vec2 out_tex_coords;


layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    v_color = a_color;
    out_tex_coords = v_tex_coords;
    gl_Position = u_view_proj * vec4(a_position, 1.0);
}
