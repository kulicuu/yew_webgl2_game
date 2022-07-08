#version 300 es

layout(std140) uniform;

layout(location=0) in vec3 aPosition;
layout(location=1) in vec3 aVelocity;
layout(location=2) in vec3 aColor;

uniform Mass {
    float mass1Factor;
    float mass2Factor;
    float mass3Factor;
    vec4 mass1Position;
    vec4 mass2Position;
    vec4 mass3Position;
};

out vec3 vPosition;
out vec3 vVelocity;
out vec3 vColor;
void main() {
    vec3 position = aPosition;
    vec3 velocity = aVelocity;

    vec3 massVec = mass1Position.xyz - position;
    float massDist2 = max(0.01, dot(massVec, massVec));
    vec3 acceleration = mass1Factor * normalize(massVec) / massDist2;

    massVec = mass2Position.xyz - position;
    massDist2 = max(0.01, dot(massVec, massVec));
    acceleration += mass2Factor * normalize(massVec) / massDist2;

    massVec = mass3Position.xyz - position;
    massDist2 = max(0.01, dot(massVec, massVec));
    acceleration += mass3Factor * normalize(massVec) / massDist2;

    velocity += acceleration;
    velocity *= 0.9999;

    vPosition = position + velocity;
    vVelocity = velocity;

    vColor = aColor;
    gl_PointSize = 2.0;
    gl_Position = vec4(position, 1.0);
}