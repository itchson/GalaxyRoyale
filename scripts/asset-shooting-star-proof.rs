use engine_core::{CameraView, FrameBuilder, Vec2};
use serde_json::{Value, json};
use shooting_star_review::{self as art, SceneInput};
fn mix(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^ (x >> 16)
}
fn offset(h: u32) -> f32 {
    ((mix(h ^ 303) >> 8) as f32 / 16_777_216.0) * 80.0
}
fn main() {
    let viewport = Vec2::new(533.0, 300.0);
    let birth_hash = 0;
    let active_time = (9.0 - offset(birth_hash)).rem_euclid(80.0);
    let frozen_hash = (0..1000)
        .find(|&h| offset(h) > 1.0 && offset(h) < 17.0)
        .unwrap();
    let mut scenes = Vec::new();
    for (name, zoom, reduced, reduced_motion, time, h, camera) in [
        (
            "full_1x",
            1.0,
            false,
            false,
            active_time,
            birth_hash,
            Vec2::ZERO,
        ),
        (
            "reduced_1x",
            1.0,
            true,
            false,
            active_time,
            birth_hash,
            Vec2::ZERO,
        ),
        (
            "later_1x",
            1.0,
            false,
            false,
            active_time + 0.3,
            birth_hash,
            Vec2::ZERO,
        ),
        (
            "full_min_zoom",
            0.6,
            false,
            false,
            active_time,
            birth_hash,
            Vec2::ZERO,
        ),
        (
            "full_max_zoom",
            1.6,
            false,
            false,
            active_time,
            birth_hash,
            Vec2::ZERO,
        ),
        (
            "frozen_time0",
            1.0,
            false,
            true,
            0.0,
            frozen_hash,
            Vec2::ZERO,
        ),
        (
            "frozen_time543",
            1.0,
            false,
            true,
            543.0,
            frozen_hash,
            Vec2::ZERO,
        ),
        (
            "far_world_min_zoom",
            0.6,
            false,
            false,
            active_time,
            birth_hash,
            Vec2::new(999_000.0, -999_000.0),
        ),
    ] {
        let view = CameraView::new(camera, viewport, zoom).unwrap();
        let input = SceneInput {
            origin: camera * art::PARALLAX,
            camera,
            viewport: view.world_extent(),
            time,
            birth_hash: h,
            tint: [u32::from_le_bytes([95, 115, 137, 255]); 4],
            reduced,
            reduced_motion,
        };
        let mut frame = FrameBuilder::new();
        let mut budget = 6000;
        let stats = art::extract(&mut frame, &input, &mut budget);
        assert_eq!(stats.quads, frame.frame().pixel_count);
        assert!(stats.quads > 800);
        assert_eq!(6000 - budget, stats.quads);
        let packets:Vec<Value>=frame.frame().batches.iter().flat_map(|batch|batch.pixels.iter().map(|pixel|{
            let transform=batch.transforms[pixel.transform as usize];
            let local=Vec2::from_array(pixel.local);
            let relative=view.relative_transform(transform);
            let corners=[Vec2::splat(-0.5),Vec2::new(0.5,-0.5),Vec2::new(-0.5,0.5),Vec2::splat(0.5)].map(|c|
                (relative.point(local+c)/view.world_extent()*viewport+viewport*0.5).to_array());
            assert!(corners.iter().flatten().all(|v|v.is_finite()));
            json!({"corners":corners,"color":pixel.color,"light_response":transform.b[2],"emission":transform.b[3]})
        })).collect();
        scenes.push(json!({"name":name,"width":533,"height":300,"zoom":zoom,"reduced":reduced,"reduced_motion":reduced_motion,
            "time":time,"birth_hash":h,"camera":camera.to_array(),"quads":stats.quads,"transforms":stats.transforms,"tail_cells":stats.tail_cells,"cells":packets}));
    }
    let frozen_a = scenes.iter().find(|s| s["name"] == "frozen_time0").unwrap();
    let frozen_b = scenes
        .iter()
        .find(|s| s["name"] == "frozen_time543")
        .unwrap();
    assert_eq!(frozen_a["cells"], frozen_b["cells"]);
    std::fs::write(std::env::args().nth(1).expect("Output JSON path"),serde_json::to_string(&json!({
        "scope":"Actual native FrameBuilder cells and CameraView projection, posed software/material approximation only",
        "scenes":scenes})).unwrap()).unwrap();
    println!(
        "Eight actual native posed scenes emitted; full884/reduced852 cells, four transforms, frozen geometry/color equality"
    );
}
