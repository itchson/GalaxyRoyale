use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
pub(super) static FIELD_QUERIES: AtomicUsize = AtomicUsize::new(0);
fn digest(frame: &engine_core::RenderFrame) -> u64 {
    let mut result = 0xcbf29ce484222325_u64;
    let mut word = |value: u32| {
        for byte in value.to_le_bytes() { result = (result ^ u64::from(byte)).wrapping_mul(0x100000001b3); }
    };
    word(frame.pixel_count as u32);
    word(frame.batches.len() as u32);
    for batch in &frame.batches {
        word(batch.transforms.len() as u32);
        for t in &batch.transforms { for x in t.a.into_iter().chain(t.b) { word(x.to_bits()); } }
        word(batch.pixels.len() as u32);
        for p in &batch.pixels { word(p.local[0].to_bits());word(p.local[1].to_bits());word(p.color);word(p.transform); }
    }
    result
}
#[test]
fn actual_star_field_query_probe() {
    let empty = SceneInput { seed:17,camera:Vec2::ZERO,viewport:Vec2::splat(2.0),time:18.0,reduced:false,reduced_motion:false };
    FIELD_QUERIES.store(0,Ordering::Relaxed);
    let cells=star_cells(empty.seed,[0,0],0,0.09,0.7,&empty);
    assert!(cells.iter().all(Option::is_none));
    let empty_queries=FIELD_QUERIES.load(Ordering::Relaxed);
    let min=((empty.camera*0.09-empty.viewport*0.5-Vec2::splat(0.7))/STAR_TILE).floor().as_ivec2();
    let max=((empty.camera*0.09+empty.viewport*0.5+Vec2::splat(0.7))/STAR_TILE).floor().as_ivec2();
    assert!(min.x<=0 && max.x>=0 && min.y<=0 && max.y>=0, "empty fixture tile is visited by actual render bounds");
    let mut rows=Vec::new();
    for seed in [17,0x47524f59] {
        for camera in [Vec2::ZERO,Vec2::new(12_800.0,-9_700.0),Vec2::new(999_000.0,-999_000.0)] {
            for reduced in [false,true] { for reduced_motion in [false,true] { for zoom in [0.6,1.0,1.6] {
                let scene=SceneInput { seed,camera,viewport:Vec2::new(533.0,300.0)/zoom,time:18.0,reduced,reduced_motion };
                FIELD_QUERIES.store(0,Ordering::Relaxed);
                let mut frame=FrameBuilder::new();let stats=render(&mut frame,scene);
                rows.push(serde_json::json!({"seed":seed,"camera":camera.to_array(),"reduced":reduced,"reduced_motion":reduced_motion,"zoom":zoom,"pixel_count":frame.frame().pixel_count,"stars":stats.stars,"motifs":stats.motifs,"transforms":stats.transforms,"digest":format!("{:016x}",digest(frame.frame())),"field_queries":FIELD_QUERIES.load(Ordering::Relaxed)}));
            }}}
        }
    }
    let path=std::path::PathBuf::from(std::env::var_os("NEBULA_QUERY_PROBE_OUT").expect("owned absolute output"));
    assert!(path.is_absolute());
    let value=serde_json::json!({"scope":"Actual native cosmic packet extraction and instrumented regions::at query count; no GPU/browser/FPS claim","empty_tile":{"seed":17,"grid":[0,0],"viewport":[2,2],"field_queries":empty_queries,"candidate_count":32,"surviving":0},"fixtures":rows});
    std::fs::write(path,serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    eprintln!("Actual empty candidate tile: {empty_queries} biome queries; 72 actual cosmic frame packet digests recorded.");
}
