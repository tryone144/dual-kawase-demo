#version 330 core

uniform int iteration;
uniform vec2 halfpixel;

layout(location = 0) in vec2 coord;
layout(location = 1) in vec2 texcoord;

out VS_OUT {
    vec2 texcoord;
    vec2 clamp_low;
    vec2 clamp_high;
} OUT;

void main() {
    gl_Position = vec4(coord / float(1 << iteration), 0.0, 1.0);
    float i_fac = float(1 << (iteration + 1));
    float i_off = 0.5 - 4.0 * max(halfpixel.x, halfpixel.y);

    OUT.clamp_low = vec2(0.5 - i_off / i_fac);
    OUT.clamp_high = vec2(0.5 + i_off / i_fac);
    OUT.texcoord = texcoord / i_fac + vec2(0.5 - 0.5 / i_fac);
}
