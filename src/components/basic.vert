precision mediump float;

attribute vec2 a_position;



uniform vec4 f1_displacement;




void main() {

    // gl_Position = vec4(a_position[0] + f1_displacement[(int(gl_InstanceID) * 2) + 0], a_position[1] + f1_displacement[(int(gl_InstanceID) * 2) + 1], 0.0, 1.0);

    // gl_Position = vec4(a_position[0] + f1_displacement[(int(gl_InstanceID) * 2) + 0], a_position[1] + f1_displacement[(int(gl_InstanceID) * 2) + 1], 0.0, 1.0);



    gl_Position = vec4(a_position[0] + f1_displacement[0 + 2], a_position[1] + f1_displacement[1 + 2], 0.0, 1.0);
}
