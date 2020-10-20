#version 450

layout (set = 0, binding = 0) restrict readonly buffer myStorageBuffer {
  uint vertices[32];
};

layout (location = 0) in vec2 quad_pos;
layout (location = 0) out vec4 outColor;

const vec3 colors[2] = {
vec3(0, 0, 0),
vec3(1, 1, 1),
};

const ivec2 raster_size = ivec2(64, 32);

void main(){
  vec2 raster_size_float = raster_size;
  vec2 scaled_pos = quad_pos * raster_size_float;
  vec2 floored_pos = floor(scaled_pos);
  uvec2 integer_pos = uvec2(floored_pos);

  uvec2 pos=uvec2(
  clamp(integer_pos.x, 0, raster_size.x - 1),
  clamp(integer_pos.y, 0, raster_size.y - 1)
  );

  uint a=vertices[pos.y * 2 + (1 - pos.x / 32)];
  uint b=(a >> (31 - pos.x % 32)) & 1;
  outColor=vec4(vec3(1, 1, 1) * b, 1.);
}
