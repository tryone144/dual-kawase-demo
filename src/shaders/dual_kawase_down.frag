#version 330 core

uniform sampler2D tex_src;
uniform vec2 halfpixel;
uniform float offset;

in VS_OUT {
    vec2 texcoord;
} IN;

out vec4 Color;

void main() {
    vec2 uv = IN.texcoord;

    vec4 sum = texture2D(tex_src, uv) * 4.0;
    sum += texture2D(tex_src, uv - halfpixel.xy * offset);
    sum += texture2D(tex_src, uv + halfpixel.xy * offset);
    sum += texture2D(tex_src, uv + vec2(halfpixel.x, -halfpixel.y) * offset);
    sum += texture2D(tex_src, uv - vec2(halfpixel.x, -halfpixel.y) * offset);

    Color = sum / 8.0;
}
