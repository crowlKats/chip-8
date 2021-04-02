const vertices: array<vec2<f32>, 6u> = array<vec2<f32>, 6u>(
  vec2<f32>(0.0, 0.0),
  vec2<f32>(1.0, 0.0),
  vec2<f32>(0.0, 1.0),
  vec2<f32>(1.0, 0.0),
  vec2<f32>(0.0, 1.0),
  vec2<f32>(1.0, 1.0)
);

struct VertexOutput {
  [[builtin(position)]] pos_out: vec4<f32>;
  [[location(0)]] triangle_pos: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput {
  var position: vec2<f32> = vertices[vertex_index];
  var out: VertexOutput;
  out.triangle_pos = vec2<f32>(position.x, 1.0 - position.y);
  out.pos_out = vec4<f32>((position * 2.0) - vec2<f32>(1.0, 1.0), 0.0, 1.0);
  return out;
}

[[block]] struct SBuffer {
  b_vertices: array<u32, 32>;
};

[[group(0), binding(0)]]
var<storage> pbuf: [[access(read)]] SBuffer;

const colors: array<vec3<u32>, 2> = array<vec3<u32>, 2>(
  vec3<u32>(0, 0, 0),
  vec3<u32>(1, 1, 1)
);
const raster_size: vec2<u32> = vec2<u32>(64, 32);

[[stage(fragment)]]
fn fs_main([[location(0)]] quad_pos: vec2<f32>) -> [[location(0)]] vec4<f32> {
  var scaled_pos: vec2<f32> = quad_pos * vec2<f32>(raster_size);
  var integer_pos: vec2<u32> = vec2<u32>(floor(scaled_pos));
  var pos: vec2<u32> = vec2<u32>(
    clamp(integer_pos.x, 0u, raster_size.x - 1u),
    clamp(integer_pos.y, 0u, raster_size.y - 1u)
  );
  var a: u32 = pbuf.b_vertices[pos.y * 2u + (1u - pos.x / 32u)];
  var b: u32 = (a >> (31u - pos.x % 32u)) & 1u;
  return vec4<f32>(vec3<f32>(1.0, 1.0, 1.0) * f32(b), 1.0);
}
