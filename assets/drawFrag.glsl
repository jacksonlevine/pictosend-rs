#version 330 core
out vec4 FragColor;
in vec2 TexCoord;
uniform sampler2D ourTexture;
void main() {
    vec4 color = texture(ourTexture, TexCoord);
    FragColor = vec4(1.0-color.r, 1.0-color.r, 1.0-color.r, 1.0);
    if(FragColor.r > 0.4 && FragColor.r < 0.6 &&
    FragColor.g > 0.4 && FragColor.g < 0.6 &&
    FragColor.b > 0.4 && FragColor.b < 0.6) {
        discard;
    }
}