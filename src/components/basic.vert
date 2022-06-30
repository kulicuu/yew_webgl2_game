precision mediump float;

attribute vec2 a_position;

attribute vec2 u_displacement;


uniform Deltas {
    vec2 fighter1Displacement;
    vec2 fighter2Displacement;
    float fighter1Rotation;
    float fighter2Rotation;
};

void main() {
    gl_Position = vec4(a_position, 0.0, 1.0);
}
