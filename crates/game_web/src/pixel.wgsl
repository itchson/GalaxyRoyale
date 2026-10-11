struct Affine { a:vec4<f32>, b:vec4<f32> }
struct TransformTable { values:array<Affine,128> }
struct Light { position_radius:vec4<f32>, color:vec4<f32> }
struct Globals { camera:vec4<f32>, ambient:vec4<f32>, lights:array<Light,8>, raster:vec4<f32> }
@group(0) @binding(0) var<uniform> globals:Globals;
@group(0) @binding(1) var<uniform> transforms:TransformTable;
struct VertexOut {
  @builtin(position) position:vec4<f32>,
  @location(0) color:vec4<f32>,
  @location(1) @interpolate(flat) cell:vec2<f32>,
  @location(2) @interpolate(flat) material:vec2<f32>,
  @location(3) particle_uv:vec2<f32>,
  @location(4) @interpolate(flat) coverage:f32,
}
@vertex fn vs_main(@builtin(vertex_index) index:u32,@location(0) center:vec2<f32>,@location(1) rgba:u32,@location(2) slot:u32)->VertexOut {
  let corners=array<vec2<f32>,6>(vec2<f32>(-0.5,-0.5),vec2<f32>(0.5,-0.5),vec2<f32>(-0.5,0.5),vec2<f32>(-0.5,0.5),vec2<f32>(0.5,-0.5),vec2<f32>(0.5,0.5));
  let t=transforms.values[slot];let local=center+corners[index];var world=t.a.xy+t.a.zw*local.x+t.b.xy*local.y;
  var coverage=1.0;
  if(t.b.w<0.0) {
    // Filter only soft footprints to two physical raster pixels; compensate area.
    // C=M*M^T retains the ellipse under rotation, reflection and hierarchy shear.
    let raster_scale=globals.raster.xy/globals.camera.zw;
    let x=t.a.zw*raster_scale;let y=t.b.xy*raster_scale;
    let cxx=x.x*x.x+y.x*y.x;let cyy=x.y*x.y+y.y*y.y;
    let cxy=x.x*x.y+y.x*y.y;
    let area=abs(x.x*y.y-x.y*y.x);
    let largest=0.5*(cxx+cyy+sqrt((cxx-cyy)*(cxx-cyy)+4.0*cxy*cxy));
    let smallest=area*area/max(largest,1e-20);
    if(smallest<4.0) {
      let variance=4.0-smallest;
      let xx=cxx+variance;let yy=cyy+variance;
      // det(C+vI)=det(C)+v*trace(C)+v*v avoids shear cancellation.
      let expanded_area=sqrt(max(area*area+variance*(cxx+cyy)+variance*variance,1e-20));
      let denominator=sqrt(max(xx+yy+2.0*expanded_area,1e-20));
      // Positive symmetric square root of the widened covariance.
      let bx=vec2<f32>(xx+expanded_area,cxy)/denominator;
      let by=vec2<f32>(cxy,yy+expanded_area)/denominator;
      let offset=bx*corners[index].x+by*corners[index].y;
      let cell=t.a.xy+t.a.zw*center.x+t.b.xy*center.y;
      world=cell+offset/raster_scale;
      coverage=clamp(area/expanded_area,0.0,1.0);
    }
  }
  let screen=(world-globals.camera.xy)/globals.camera.zw*2.0;
  var out:VertexOut;out.position=vec4<f32>(screen.x,-screen.y,0.0,1.0);
  let color=vec4<f32>(f32(rgba&255u),f32((rgba>>8u)&255u),f32((rgba>>16u)&255u),f32((rgba>>24u)&255u))/255.0;
  out.color=vec4<f32>(pow(color.rgb,vec3<f32>(2.2)),color.a);
  out.cell=t.a.xy+t.a.zw*center.x+t.b.xy*center.y;
  out.material=t.b.zw;
  out.particle_uv=corners[index]*2.0;
  out.coverage=coverage;
  return out;
}
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32> {
  // Each occupied cell receives one color. UI/backgrounds use an unlit material.
  let response=input.material.x;
  let emission=select(input.material.y,max(0.0,-input.material.y-1.0),input.material.y<0.0);
  var color=input.color.rgb*(1.0-0.12*response+0.3*emission);
  if(response>0.0) {
    color=color*mix(vec3<f32>(1.0),globals.ambient.xyz,clamp(globals.ambient.w*response,0.0,1.0));
    for(var i=0u;i<8u;i=i+1u) {
      let light=globals.lights[i];
      let delta=input.cell-light.position_radius.xy;
      let radius=max(1.0,light.position_radius.z);
      let attenuation=max(0.0,1.0-dot(delta,delta)/(radius*radius));
      let energy=attenuation*attenuation*light.position_radius.w*response;
      color=color+light.color.rgb*(input.color.rgb*0.8+vec3<f32>(0.012))*energy;
    }
  }
  var alpha=input.color.a;
  if(input.material.y<0.0) {
    let radial=max(0.0,1.0-dot(input.particle_uv,input.particle_uv));
    alpha=alpha*radial*radial*input.coverage;
  }
  return vec4<f32>(min(color,vec3<f32>(1.0)),alpha);
}
