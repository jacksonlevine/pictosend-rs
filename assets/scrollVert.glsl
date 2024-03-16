#version 330 core
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 texcoord;

out vec2 TexCoord;

uniform float scroll;

void main()
{
    gl_Position = vec4(vec2(pos.x, pos.y + scroll), 0.0, 1.0);
    TexCoord = texcoord;
}