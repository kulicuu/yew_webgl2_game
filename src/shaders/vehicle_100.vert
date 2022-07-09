#version 300 es

precision mediump float;

in vec2 a_position; 
uniform vec2 pos_deltas;
uniform float vifo_theta;

mat2 r2d(float a) {
	float c = cos(a), s = sin(a);
    return mat2(
        c, s, // column 1
        -s, c // column 2
    );
}

mat3 rotate_translate(float theta, float dx, float dy) {
    float c = cos(theta), s = sin(theta);
    return mat3(
        c, -s, dx,
        s, c, dy,
        0, 0, 1
    );
}

void main() {
    mat2 t_2 = r2d(vifo_theta);
    mat3 transform = rotate_translate(vifo_theta, pos_deltas[0], pos_deltas[1]);
    vec2 p_3 = t_2 * a_position;
    gl_Position = vec4(p_3[0] + pos_deltas[0], p_3[1] + pos_deltas[1], 0.0, 1.0);

}
