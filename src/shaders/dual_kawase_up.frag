#version 330 core

uniform sampler2D tex_src;
uniform vec2 halfpixel;
uniform float offset;
uniform float opacity;

in VS_OUT {
    vec2 texcoord;
    vec2 clamp_low;
    vec2 clamp_high;
} IN;

out vec4 Color;

vec4 clamp_tex(vec2 uv) {
    return texture2D(tex_src, clamp(uv, IN.clamp_low, IN.clamp_high));
}

void main() {
    vec2 uv = IN.texcoord;

    vec4 sum = clamp_tex(uv + vec2(-halfpixel.x * 2.0, 0.0) * offset);
    sum += clamp_tex(uv + vec2(-halfpixel.x, halfpixel.y) * offset) * 2.0;
    sum += clamp_tex(uv + vec2(0.0, halfpixel.y * 2.0) * offset);
    sum += clamp_tex(uv + vec2(halfpixel.x, halfpixel.y) * offset) * 2.0;
    sum += clamp_tex(uv + vec2(halfpixel.x * 2.0, 0.0) * offset);
    sum += clamp_tex(uv + vec2(halfpixel.x, -halfpixel.y) * offset) * 2.0;
    sum += clamp_tex(uv + vec2(0.0, -halfpixel.y * 2.0) * offset);
    sum += clamp_tex(uv + vec2(-halfpixel.x, -halfpixel.y) * offset) * 2.0;

    Color = sum / 12.0;
    Color.a = opacity;
}
