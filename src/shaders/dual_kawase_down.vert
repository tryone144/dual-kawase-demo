#version 330 core

uniform int iteration;

layout(location = 0) in vec2 coord;
layout(location = 1) in vec2 texcoord;

out VS_OUT {
    vec2 texcoord;
} OUT;

void main() {
    gl_Position = vec4(coord / (1 << iteration), 0.0, 1.0);
    OUT.texcoord = texcoord;
}
