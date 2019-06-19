#version 330 core

uniform int iteration;

layout(location = 0) in vec2 coord;
layout(location = 1) in vec2 texcoord;

out VS_OUT {
    vec2 texcoord;
} OUT;

void main() {
    float i_fac = float(1 << iteration);
    vec2 i_off = vec2(1.0) - vec2(1.0) / i_fac;

    gl_Position = vec4(coord / i_fac - i_off, 0.0, 1.0);
    OUT.texcoord = texcoord;
}
