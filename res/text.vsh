#version 100
attribute vec2 pos;
attribute vec2 uv;

varying lowp vec2 texCoord;

uniform mat4 u_mvp;

void main() {
  gl_Position = u_mvp * vec4(pos, 0, 1);
  texCoord = uv;
}

