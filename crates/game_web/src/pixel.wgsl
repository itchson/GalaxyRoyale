struct Affine { a:vec4<f32>, b:vec4<f32> }
struct TransformTable { values:array<Affine,128> }
struct Light { position_radius:vec4<f32>, color:vec4<f32> }
struct Globals { camera:vec4<f32>, lights:array<Light,8> }
@group(0) @binding(0) var<uniform> globals:Globals;
@group(0) @binding(1) var<uniform> transforms:TransformTable;
struct VertexOut {
  @builtin(position) position:vec4<f32>,
  @location(0) color:vec4<f32>,
  @location(1) @interpolate(flat) cell:vec2<f32>,
  @location(2) @interpolate(flat) material:vec2<f32>,
}
@vertex fn vs_main(@builtin(vertex_index) index:u32,@location(0) center:vec2<f32>,@location(1) rgba:u32,@location(2) slot:u32)->VertexOut {
  let corners=array<vec2<f32>,6>(vec2<f32>(-0.5,-0.5),vec2<f32>(0.5,-0.5),vec2<f32>(-0.5,0.5),vec2<f32>(-0.5,0.5),vec2<f32>(0.5,-0.5),vec2<f32>(0.5,0.5));
  let t=transforms.values[slot];let local=center+corners[index];let world=t.a.xy+t.a.zw*local.x+t.b.xy*local.y;
  let screen=(world-globals.camera.xy)/globals.camera.zw*2.0;
  var out:VertexOut;out.position=vec4<f32>(screen.x,-screen.y,0.0,1.0);
  let color=vec4<f32>(f32(rgba&255u),f32((rgba>>8u)&255u),f32((rgba>>16u)&255u),f32((rgba>>24u)&255u))/255.0;
  out.color=vec4<f32>(pow(color.rgb,vec3<f32>(2.2)),color.a);
  out.cell=t.a.xy+t.a.zw*center.x+t.b.xy*center.y;
  out.material=t.b.zw;
  return out;
}
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32> {
  // Each occupied cell receives one color. UI/backgrounds use an unlit material.
  let response=input.material.x;
  var color=input.color.rgb*(1.0-0.12*response+0.3*input.material.y);
  if(response>0.0) {
    for(var i=0u;i<8u;i=i+1u) {
      let light=globals.lights[i];
      let delta=input.cell-light.position_radius.xy;
      let radius=max(1.0,light.position_radius.z);
      let attenuation=max(0.0,1.0-dot(delta,delta)/(radius*radius));
      let energy=attenuation*attenuation*light.position_radius.w*response;
      color=color+light.color.rgb*(input.color.rgb*0.8+vec3<f32>(0.012))*energy;
    }
  }
  return vec4<f32>(min(color,vec3<f32>(1.0)),input.color.a);
}
