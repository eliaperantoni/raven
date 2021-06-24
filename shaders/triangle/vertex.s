#version 330 core
layout (location = 0) vec2 pos_in;
layout (location = 1) vec3 color_in;

out vec3 color;

void main() {
    gl_Position = pos_in;
    color = color_in;
}
