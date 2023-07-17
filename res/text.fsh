#version 460
in vec2 texCoord;

uniform sampler2D tex;

out vec4 fragColor;

void main() {
  fragColor = texture2D(tex, texCoord);
}
