#version 450

layout (location = 0) in vec2 triangle_pos;
layout (location = 0) out vec4 outColor;

void main(){
  outColor = vec4(triangle_pos, 0, 1.);
}
