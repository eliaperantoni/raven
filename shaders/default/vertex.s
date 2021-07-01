#version 330 core
layout (location = 0) in vec3 pos_in;
layout (location = 1) in vec3 normal_in;
layout (location = 2) in vec2 uv_in;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 frag_pos;
out vec3 normal;
out vec2 uv;

void main() {
    gl_Position = projection * view * model * vec4(pos_in, 1.0);

    frag_pos = vec3(model * vec4(pos_in, 1.0));
    normal = normal_in;
    uv = uv_in;
}
