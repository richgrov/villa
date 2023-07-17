#version 460
in vec2 pos;
in vec2 uv;

out vec2 texCoord;

uniform mat4 u_mvp;

void main() {
  gl_Position = u_mvp * vec4(pos, 0, 1);
  texCoord = uv;
}

