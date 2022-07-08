#version 300 es
precision highp float;

in vec3 vColor;

out vec4 fragColor;
void main() {
    float alpha = 0.3;
    fragColor = vec4(vColor * alpha, alpha);
}