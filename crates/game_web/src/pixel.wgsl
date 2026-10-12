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
  @location(5) @interpolate(flat) surface:vec4<f32>,
  @location(6) @interpolate(flat) mapped:f32,
}
fn sign_not_zero(value:vec2<f32>)->vec2<f32> {
  return select(vec2<f32>(-1.0),vec2<f32>(1.0),value>=vec2<f32>(0.0));
}
fn decode_normal(bits:u32)->vec3<f32> {
  let xy=vec2<f32>(f32((bits>>7u)&255u),f32((bits>>15u)&255u))/255.0*2.0-1.0;
  var normal=vec3<f32>(xy,1.0-abs(xy.x)-abs(xy.y));
  if(normal.z<0.0) { normal=vec3<f32>((vec2<f32>(1.0)-abs(xy.yx))*sign_not_zero(xy),normal.z); }
  return normalize(normal);
}
fn surface_normal(local:vec3<f32>,t:Affine)->vec3<f32> {
  // diag(A,sqrt(abs(det A))) embeds the composed 2D affine in 3D.
  // Inverse transpose preserves slopes under uniform scale and handles shear
  // and reflections. Normalize the basis first: tiny quads have no size floor.
  let magnitude=max(max(abs(t.a.z),abs(t.a.w)),max(abs(t.b.x),abs(t.b.y)));
  if(magnitude==0.0) { return vec3<f32>(0.0,0.0,1.0); }
  let x=t.a.zw/magnitude;let y=t.b.xy/magnitude;
  let determinant=x.x*y.y-x.y*y.x;
  if(abs(determinant)<1e-12) { return vec3<f32>(0.0,0.0,1.0); }
  let factor=select(-1.0,1.0,determinant>=0.0)/sqrt(abs(determinant));
  let xy=vec2<f32>(y.y*local.x-x.y*local.y,-y.x*local.x+x.x*local.y)*factor;
  return normalize(vec3<f32>(xy,local.z));
}
@vertex fn vs_main(@builtin(vertex_index) index:u32,@location(0) center:vec2<f32>,@location(1) rgba:u32,@location(2) slot:u32)->VertexOut {
  let corners=array<vec2<f32>,6>(vec2<f32>(-0.5,-0.5),vec2<f32>(0.5,-0.5),vec2<f32>(-0.5,0.5),vec2<f32>(-0.5,0.5),vec2<f32>(0.5,-0.5),vec2<f32>(0.5,0.5));
  // Seven slot bits plus optional oct8 normal/linear luminance data retain the
  // existing 16-byte pixel instance and 32-byte affine table records.
  let t=transforms.values[slot&127u];let local=center+corners[index];var world=t.a.xy+t.a.zw*local.x+t.b.xy*local.y;
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
  out.surface=vec4<f32>(0.0,0.0,1.0,1.0);out.mapped=0.0;
  if((slot&0x80000000u)!=0u) {
    out.surface=vec4<f32>(surface_normal(decode_normal(slot),t),f32((slot>>23u)&255u)/255.0);
    out.mapped=1.0;
  }
  return out;
}
@fragment fn fs_main(input:VertexOut)->@location(0) vec4<f32> {
  // Each occupied cell receives one color. UI/backgrounds use an unlit material.
  let response=input.material.x;
  let object_emission=select(input.material.y,max(0.0,-input.material.y-1.0),input.material.y<0.0);
  let emission=object_emission*input.surface.w;
  var color=input.color.rgb*(1.0-0.12*response+0.3*emission);
  if(response>0.0) {
    color=color*mix(vec3<f32>(1.0),globals.ambient.xyz,clamp(globals.ambient.w*response,0.0,1.0));
    for(var i=0u;i<8u;i=i+1u) {
      let light=globals.lights[i];
      let delta=input.cell-light.position_radius.xy;
      let radius=max(1.0,light.position_radius.z);
      let attenuation=max(0.0,1.0-dot(delta,delta)/(radius*radius));
      var directional=1.0;
      if(input.mapped>0.0 && light.color.w>0.0) {
        let toward_light=normalize(vec3<f32>(light.position_radius.xy-input.cell,light.color.w));
        directional=max(0.0,dot(input.surface.xyz,toward_light));
      }
      let energy=attenuation*attenuation*light.position_radius.w*response*directional;
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
