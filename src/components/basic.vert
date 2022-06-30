precision mediump float;

attribute vec2 a_position;

// attribute vec2 u_displacement;


uniform vec2 f1_displacement;
uniform vec2 f2_displacement;
uniform float f1_rotation;
uniform float f2_rotation;




void main() {
    gl_Position = vec4(a_position[0] + f1_displacement[0], a_position[1] + f1_displacement[1], 0.0, 1.0);
}
