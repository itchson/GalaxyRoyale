use naga::{back::glsl, valid::{Validator, ValidationFlags, Capabilities}};
fn main() {
 let input=std::env::args().nth(1).expect("WGSL path");
 let output=std::env::args().nth(2).expect("output folder");
 let source=std::fs::read_to_string(input).unwrap();
 let module=naga::front::wgsl::parse_str(&source).expect("shared WGSL parses");
 let info=Validator::new(ValidationFlags::all(),Capabilities::empty()).validate(&module).expect("portable WGSL validation");
 println!("WGSL validation: all flags, empty optional capabilities");
 for (_,ty) in module.types.iter() { if let naga::TypeInner::Struct{span,..}=ty.inner {if let Some(name)=&ty.name { println!("layout {name}: {span}B");}}}
 for (stage,entry) in [(naga::ShaderStage::Vertex,"vs_main"),(naga::ShaderStage::Fragment,"fs_main")] {
 let mut text=String::new();
 let options=glsl::Options {version:glsl::Version::Embedded{version:300,is_webgl:true},..Default::default()};
 let pipeline=glsl::PipelineOptions {shader_stage:stage,entry_point:entry.into(),multiview:None};
 let mut writer=glsl::Writer::new(&mut text,&module,&info,&options,&pipeline,naga::proc::BoundsCheckPolicies::default()).unwrap();
 let reflection=writer.write().expect("WebGL2 ES300 GLSL emission");
 assert!(text.starts_with("#version 300 es"));
 println!("GLSL ES300 {entry}: {}B; {} reflected uniform blocks",text.len(),reflection.uniforms.len());
 std::fs::write(format!("{output}/{entry}.glsl"),text).unwrap();
 }
}
