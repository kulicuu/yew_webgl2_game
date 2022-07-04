#version 300 es

precision mediump float;

in vec2 a_position; // model positioned at origin.
uniform vec2 pos_deltas;
uniform float vifo_theta;


mat2 r2d(float a) {
	float c = cos(a), s = sin(a);
    return mat2(
        c, s,
        -s, c
    );
}

mat3 rotate_translate(float theta, float dx, float dy) {
    float c = con(theta), s = sin(theta);
    return mat3(
        c, -s, dx,
        s, c, dy,
        0, 0, 1
    );
}

void main() {

    mat3 transform = rotate_translate(vifo_theta, pos_deltas[0], pos_deltas[1]);

    vec3 v3 = transform * vec3(a_position, 1.0);
    gl_Position = vec4(v3, 1.0);

}
