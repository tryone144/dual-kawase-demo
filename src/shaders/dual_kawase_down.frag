#version 330 core

uniform sampler2D tex_src;
uniform vec2 halfpixel;
uniform float offset;

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

    vec4 sum = clamp_tex(uv) * 4.0;
    sum += clamp_tex(uv - halfpixel.xy * offset);
    sum += clamp_tex(uv + halfpixel.xy * offset);
    sum += clamp_tex(uv + vec2(halfpixel.x, -halfpixel.y) * offset);
    sum += clamp_tex(uv - vec2(halfpixel.x, -halfpixel.y) * offset);

    Color = sum / 8.0;
}
