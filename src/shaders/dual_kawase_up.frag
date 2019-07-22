#version 330 core

uniform sampler2D tex_src;
uniform vec2 halfpixel;
uniform float offset;
uniform float opacity;

in VS_OUT {
    vec2 texcoord;
} IN;

layout(location = 0) out vec4 Color;

void main() {
    vec2 uv = IN.texcoord;

    vec4 sum = texture2D(tex_src, uv + vec2(-halfpixel.x * 2.0, 0.0) * offset);
    sum += texture2D(tex_src, uv + vec2(-halfpixel.x, halfpixel.y) * offset) * 2.0;
    sum += texture2D(tex_src, uv + vec2(0.0, halfpixel.y * 2.0) * offset);
    sum += texture2D(tex_src, uv + vec2(halfpixel.x, halfpixel.y) * offset) * 2.0;
    sum += texture2D(tex_src, uv + vec2(halfpixel.x * 2.0, 0.0) * offset);
    sum += texture2D(tex_src, uv + vec2(halfpixel.x, -halfpixel.y) * offset) * 2.0;
    sum += texture2D(tex_src, uv + vec2(0.0, -halfpixel.y * 2.0) * offset);
    sum += texture2D(tex_src, uv + vec2(-halfpixel.x, -halfpixel.y) * offset) * 2.0;

    Color = sum / 12.0;
    Color.a = opacity;
}
