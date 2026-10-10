@group(0) @binding(0) var scene:texture_2d<f32>;
@group(0) @binding(1) var nearest_sampler:sampler;
const MANUAL_SRGB:bool=MANUAL_SRGB_VALUE;
struct VertexOut { @builtin(position) position:vec4<f32>, @location(0) uv:vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) index:u32)->VertexOut {
  let p=array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));
  var out:VertexOut;out.position=vec4<f32>(p[index],0.0,1.0);out.uv=vec2<f32>((p[index].x+1.0)*0.5,(1.0-p[index].y)*0.5);return out;
}
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32>{
  let color=textureSample(scene,nearest_sampler,input.uv);
  if MANUAL_SRGB {return vec4<f32>(pow(max(color.rgb,vec3<f32>(0.0)),vec3<f32>(1.0/2.2)),1.0);}
  return vec4<f32>(color.rgb,1.0);
}
