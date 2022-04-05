#version 140

in vec3 v_normal;
in vec3 v_position;
in vec2 v_tex_coords;

out vec4 color;

uniform vec3 u_light;
uniform sampler2D diffuse_tex;
uniform sampler2D normal_tex;

const vec3 skybox_color = vec3(0.2, 0.6, 0.9);
const float light_strength = 0.95;

void main() {
    vec3 diffuse_color = texture(diffuse_tex, v_tex_coords).rgb;
    vec3 ambient_color = diffuse_color * 0.15;
    float diffuse = max(light_strength*dot(v_normal, -normalize(u_light)), 0.0);
    color = vec4(ambient_color + diffuse * diffuse_color, 1.0);
}