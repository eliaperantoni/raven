#version 330 core

in vec3 frag_pos;
in vec3 normal;
in vec2 uv;

out vec4 color;

void main() {
    color = vec4(0.0, 0.0, 1.0, 1.0);
}
