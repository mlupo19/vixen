#version 140
in vec3 v_normal;
in vec3 v_position;
in vec2 v_tex_coords;
out vec4 color;
uniform vec3 u_light;
uniform sampler2D diffuse_tex;
uniform sampler2D normal_tex;

void main() {
    color = vec4(0.2, 0.8, 0.4, 1.0);
}