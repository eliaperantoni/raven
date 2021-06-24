#version 330 core
layout (location = 0) in vec2 pos_in;
layout (location = 1) in vec3 color_in;

out vec3 color;

void main() {
    gl_Position = vec4(pos_in, 0.0, 1.0);
    color = color_in;
}
