#version 300 es

precision mediump float;

in vec2 a_position;



// uniform vec4 f1_displacement;

uniform vec2 f3_d;

// uniform vec2 rotations;
uniform float r1;


mat2 r2d(float a) {
	float c = cos(a), s = sin(a);
    return mat2(
        c, s,
        -s, c,
    );
}



void main() {

    mat2 rmat = r2d(r1);

    // vec2 pos_4 = rmat * a_position;

    // vec2 pos_5 = vec2(pos_4[0] + f1_displacement[(int(gl_InstanceID) * 2) + 0], pos_4[1] + f1_displacement[(int(gl_InstanceID) * 2) + 1]); 

    // vec2 pos_2 = vec2(a_position[0] + f1_displacement[(int(gl_InstanceID) * 2) + 0], a_position[1] + f1_displacement[(int(gl_InstanceID) * 2) + 1]);

    vec2 pos_2 = rmat * a_position;

    vec2 pos_3 = vec2(pos_2[0] + f3_d[0], pos_2[1] + f3_d[1]);

    gl_Position = vec4(pos_3, 0.0, 1.0);


    // gl_Position = vec4(a_position[0] + f1_displacement[(int(gl_InstanceID) * 2) + 0], a_position[1] + f1_displacement[(int(gl_InstanceID) * 2) + 1], 0.0, 1.0);

}
