#version 300 es

precision mediump float;

in vec2 b_position;

void main() {
    gl_Position = vec4(b_position, 0.0, 1.0);
}