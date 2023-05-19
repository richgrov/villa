#version 100
attribute vec2 pos;
attribute vec2 uv;

varying lowp vec2 texCoord;

void main() {
  gl_Position = vec4(pos, 0, 1);
  texCoord = uv;
}

