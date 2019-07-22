#version 330 core

uniform sampler2D glyph_tex;

in VS_OUT {
    vec2 texcoord;
    vec4 color;
} IN;

layout(location = 0) out vec4 Color;

void main() {
    float alpha = texture2D(glyph_tex, IN.texcoord).r;
    if (alpha <= 0.0) {
        discard;
    }

    Color = IN.color * vec4(1.0, 1.0, 1.0, alpha);
}
