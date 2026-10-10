//! Native CPU extraction proof, not a browser/device FPS benchmark.
//! cargo run --release -p engine_core --example boss_rig_proof
use engine_core::{
    FrameBuilder, PixelBody, PixelRig, PixelRigPart, PixelRigState, Transform, Vec2,
};
use serde::Deserialize;
use std::{collections::BTreeMap, time::Instant};

#[derive(Deserialize)]
struct Atlas {
    palette: Vec<u32>,
    sprites: BTreeMap<String, Sprite>,
    rigs: BTreeMap<String, RigData>,
}
#[derive(Deserialize)]
struct Sprite {
    width: usize,
    height: usize,
    indices: Vec<u8>,
}
#[derive(Deserialize)]
struct RigData {
    roles: Vec<u8>,
    parts: Vec<PixelRigPart>,
}
fn main() {
    let atlas: Atlas = serde_json::from_str(include_str!("../../../assets/pixels/boss-atlas.json"))
        .expect("authored boss atlas");
    let mut actors = BTreeMap::new();
    for (name, sprite) in &atlas.sprites {
        let rows: Vec<String> = sprite
            .indices
            .chunks(sprite.width)
            .map(|row| {
                row.iter()
                    .map(|&i| char::from_u32(0xe000 + i as u32).unwrap())
                    .collect()
            })
            .collect();
        let palette: Vec<_> = atlas
            .palette
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, &c)| (char::from_u32(0xe000 + i as u32).unwrap(), c))
            .collect();
        let body = PixelBody::from_ascii(
            &rows.iter().map(String::as_str).collect::<Vec<_>>(),
            &palette,
        );
        let data = &atlas.rigs[name];
        let rig = PixelRig::new(
            sprite.width,
            sprite.height,
            data.roles.clone(),
            data.parts.clone(),
        )
        .unwrap();
        actors.insert(name.clone(), (body, rig));
    }
    let mut scenes: Vec<_> = actors
        .keys()
        .filter(|name| !name.ends_with("_segment"))
        .map(|name| ((*name).clone(), vec![(name.as_str(), 1usize)]))
        .collect();
    scenes.push((
        "coil_5_full_19_segments".into(),
        vec![("boss_coil_5", 1), ("boss_coil_5_segment", 19)],
    ));
    let mut reports = Vec::new();
    for (name, groups) in scenes {
        let mut frame = FrameBuilder::new();
        let mut timings = Vec::new();
        for sample in 0..600 {
            let started = Instant::now();
            frame.clear();
            for &(body_name, count) in &groups {
                let (body, rig) = &actors[body_name];
                for index in 0..count {
                    rig.draw(
                        &mut frame,
                        body,
                        Transform {
                            position: Vec2::new(index as f32 * 27.0, 0.0),
                            rotation: sample as f32 * 0.002,
                            ..Transform::default()
                        },
                        PixelRigState {
                            time: sample as f32 / 60.0,
                            charge: 0.7,
                            core_open: sample % 3 == 2,
                            hit_flash: 0.0,
                        },
                    );
                }
            }
            std::hint::black_box(frame.frame());
            if sample >= 100 {
                timings.push(started.elapsed().as_secs_f64() * 1000.0);
            }
        }
        timings.sort_by(f64::total_cmp);
        reports.push(serde_json::json!({
            "scene": name, "pixels": frame.frame().pixel_count,
            "transforms": frame.frame().batches.iter().map(|b| b.transforms.len()).sum::<usize>(),
            "batches": frame.frame().batches.len(), "samples":timings.len(),
            "cpu_extract_ms_p50": timings[timings.len()/2],
            "cpu_extract_ms_p95": timings[timings.len()*95/100],
            "cpu_extract_ms_max": timings[timings.len()-1],
        }));
    }
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "scope":"native release CPU extraction only; excludes simulation, GPU, browser and physical devices",
        "command":"cargo run --release -p engine_core --example boss_rig_proof",
        "scenes":reports,
    })).unwrap());
}
