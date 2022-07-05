#version 300 es

precision mediump float;
uniform vec2 pos_deltas;
uniform float vifo_theta;

in vec2 b_position;


mat2 r2d(float a) {
	float c = cos(a), s = sin(a);
	// https://en.wikipedia.org/wiki/Rotation_matrix
    // https://www.khronos.org/opengl/wiki/Data_Type_(GLSL)#Matrix_constructors
    return mat2(
        c, s, // column 1
        -s, c // column 2
    );
}

void main() {
    mat2 t_2 = r2d(vifo_theta);

    vec2 p_3 = t_2 * b_position;


    gl_Position = vec4(p_3[0] + pos_deltas[0], p_3[1] + pos_deltas[1], 0.0, 1.0);

    // gl_Position = vec4(b_position, 0.0, 1.0);
}