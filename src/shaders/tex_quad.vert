#version 330 core

uniform mat4 transform;

layout(location = 0) in vec2 coord;
layout(location = 1) in vec2 texcoord;

out VS_OUT {
    vec2 texcoord;
} OUT;

void main() {
    gl_Position = transform * vec4(coord, 0.0, 1.0);
    OUT.texcoord = texcoord;
}
