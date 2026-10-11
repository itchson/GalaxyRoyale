@group(0) @binding(0) var scene:texture_2d<f32>;
@group(0) @binding(1) var nearest_sampler:sampler;
@group(0) @binding(2) var ui_scene:texture_2d<f32>;
// Full backing resolution and the old integer-nearest UI presentation extent.
@group(0) @binding(3) var<uniform> presentation:vec4<f32>;
const MANUAL_SRGB:bool=MANUAL_SRGB_VALUE;
struct VertexOut { @builtin(position) position:vec4<f32>, @location(0) uv:vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) index:u32)->VertexOut {
  let p=array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));
  var out:VertexOut;out.position=vec4<f32>(p[index],0.0,1.0);out.uv=vec2<f32>((p[index].x+1.0)*0.5,(1.0-p[index].y)*0.5);return out;
}
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32>{
  let world=textureSample(scene,nearest_sampler,input.uv);
  let ui_offset=floor((presentation.xy-presentation.zw)*0.5);
  let ui_uv=(input.position.xy-ui_offset)/presentation.zw;
  // Always sample before the bounds branch: implicit texture derivatives stay
  // uniform on the WebGL2 path, including the one-pixel letterbox boundary.
  let ui=textureSample(ui_scene,nearest_sampler,ui_uv);
  let in_ui=all(ui_uv>=vec2<f32>(0.0)) && all(ui_uv<vec2<f32>(1.0));
  // UI ALPHA_BLENDING into transparent black produces premultiplied RGB.
  let color=select(world.rgb,ui.rgb+world.rgb*(1.0-ui.a),in_ui);
  if MANUAL_SRGB {return vec4<f32>(pow(max(color,vec3<f32>(0.0)),vec3<f32>(1.0/2.2)),1.0);}
  return vec4<f32>(color,1.0);
}
