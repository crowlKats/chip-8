#version 450

layout(location=0)in vec2 quad_pos;
layout(location=0)out vec4 outColor;

const vec3 colors[5]={
vec3(0, 0, 0),
vec3(1, 1, 1),
vec3(1, 0, 0),
vec3(0, 1, 0),
vec3(0, 0, 1),
};

const ivec2 raster_size=ivec2(64, 32);

void main(){
  vec2 raster_size_float=raster_size;
  vec2 scaled_pos=quad_pos*raster_size_float;
  vec2 floored_pos=floor(scaled_pos);
  uvec2 integer_pos=uvec2(floored_pos);

  // This now contains x values in the range 0..63 and y values in the range 0..31
  uvec2 pos=uvec2(
  clamp(integer_pos.x, 0, raster_size.x-1),
  clamp(integer_pos.y, 0, raster_size.y-1)
  );

  uint color_idx=(pos.x*64+pos.y)%5;

  outColor=vec4(colors[color_idx], 1.);
}
