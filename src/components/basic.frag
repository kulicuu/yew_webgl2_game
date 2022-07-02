#version 300 es

precision mediump float;

uniform float u_time;
// in vec4 Position;
// out vec4 gl_FragColor;

out vec4 FragColor;

void main() {
    float r = sin(u_time * 0.0003);
    float g = sin(u_time * 0.0005);
    float b = sin(u_time * 0.0007);

    FragColor = vec4(r, g, b, 1.0);
}
