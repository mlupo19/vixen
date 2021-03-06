#version 150
in uint position;
in vec2 tex_coords;

out vec3 v_normal;
out vec3 v_position;
out vec2 v_tex_coords;

uniform mat4 view_projection;
uniform vec3 chunk_coords;

void main() {
    vec3 vertexPos = vec3(float(position & 0x3Fu), float((position & 0xFC0u) >> 6u), float((position & 0x3F000u) >> 12u));

    vec3 normal = vec3(float((position & 0x40000u) >> 18u), float((position & 0x80000u) >> 19u), float((position & 0x100000u) >> 20u));	

    v_tex_coords = tex_coords;//vec2(float(tex_coords & 0xFFFFu) / 1000.0, float((tex_coords & 0xFFFF0000u) >> 16u) / 1000.0);
    v_normal = normal;
    gl_Position = view_projection * vec4(vertexPos + chunk_coords, 1.0);
    v_position = gl_Position.xyz;// / gl_Position.w;
}