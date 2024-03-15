#version 330 core
out vec4 FragColor;
in vec2 TexCoord;
uniform sampler2D ourTexture;
void main() {
    vec4 color = texture(ourTexture, TexCoord);
    FragColor = vec4(1.0-color.r, 1.0-color.r, 1.0-color.r, 1.0);
}