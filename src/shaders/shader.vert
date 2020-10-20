#version 450

const vec2 quadVertices[6] = {
vec2(0., 0.),
vec2(1., 0.),
vec2(0., 1.),
vec2(1., 0.),
vec2(0., 1.),
vec2(1., 1.)
};

out gl_PerVertex {
  vec4 gl_Position;
};

layout (location = 0) out vec2 triangle_pos;

void main() {
  vec2 position = quadVertices[gl_VertexIndex];
  triangle_pos = vec2(position.x, 1-position.y);
  gl_Position = vec4(position * 2 - 1, 0., 1.);
}
