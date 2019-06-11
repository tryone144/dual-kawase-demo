#version 330 core

uniform sampler2D tex_src;

in VS_OUT {
    vec2 texcoord;
} IN;

out vec4 Color;

void main() {
    Color = texture2D(tex_src, IN.texcoord);
}
