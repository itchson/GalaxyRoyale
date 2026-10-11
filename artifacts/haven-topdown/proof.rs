//! Actual public Game fixed-step/extraction proof, not a browser/GPU test.
use engine_core::Vec2;
use game_core::{Game, InputFrame, Profile};
use serde_json::{Value, json};

const GROUPS: [(&str, [f32; 2], f32); 5] = [
    ("dock", [0.0, 0.0], 1.5),
    ("forge", [-120.0, -80.0], 0.55),
    ("hangar", [120.0, -80.0], 0.55),
    ("journal", [-120.0, 95.0], 0.55),
    ("arenas", [120.0, 95.0], 0.55),
];
fn extract(game: &Game) -> Vec<Value> {
    let frame = game.render(1.0);
    GROUPS.iter().map(|(name,center,scale)| {
        let mut cells = Vec::new();
        let mut transform_count = 0;
        for batch in &frame.batches {
            for (index,transform) in batch.transforms.iter().enumerate() {
                if transform.a != [center[0],center[1],*scale,0.0]
                    || transform.b[..2] != [0.0,*scale] { continue; }
                let selected:Vec<_> = batch.pixels.iter().filter(|p|p.transform as usize == index).collect();
                if selected.is_empty() {continue;}
                transform_count += 1;
                for pixel in selected {
                    let local = Vec2::from_array(pixel.local);
                    let corners = [Vec2::splat(-0.5),Vec2::new(0.5,-0.5),Vec2::splat(0.5),Vec2::new(-0.5,0.5)]
                        .map(|offset| {
                            let p=local+offset;
                            [transform.a[0]+transform.a[2]*p.x+transform.b[0]*p.y,
                             transform.a[1]+transform.a[3]*p.x+transform.b[1]*p.y]
                        });
                    assert!(corners.iter().flatten().all(|x|x.is_finite()));
                    cells.push(json!({"corners":corners,"color":pixel.color,"material":[transform.b[2],transform.b[3]]}));
                }
            }
        }
        assert!(!cells.is_empty(),"{name} must be present in actual Game extraction");
        json!({"name":name,"center":center,"scale":scale,"transforms":transform_count,"occupied_cells":cells.len(),"cells":cells})
    }).collect()
}
fn main() {
    let mut game = Game::new(Profile::default());
    let before = serde_json::to_value(game.profile()).unwrap();
    let initial = extract(&game);
    for _ in 0..60 {
        game.tick(&InputFrame::default(), 1.0 / 60.0);
    }
    let later = extract(&game);
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
    for (a, b) in initial.iter().zip(&later) {
        assert_eq!(a["occupied_cells"], b["occupied_cells"]);
        let cells_a = a["cells"].as_array().unwrap();
        let cells_b = b["cells"].as_array().unwrap();
        assert!(
            cells_a
                .iter()
                .zip(cells_b)
                .all(|(x, y)| x["corners"] == y["corners"])
        );
        assert!(
            cells_a
                .iter()
                .zip(cells_b)
                .any(|(x, y)| x["color"] != y["color"]),
            "{} needs real relative signal animation",
            a["name"]
        );
    }
    game.set_reduced_motion(true);
    let reduced = extract(&game);
    for _ in 0..60 {
        game.tick(&InputFrame::default(), 1.0 / 60.0);
    }
    assert_eq!(extract(&game), reduced);
    assert_eq!(serde_json::to_value(game.profile()).unwrap(), before);
    let output = json!({"scope":"Actual public Game fixed-step plus PixelBody/FrameBuilder extraction; native flat software review only","frames":[{"stage":"start","groups":initial},{"stage":"after60fixedticks","groups":later}],"reduced_motion_packets_equal_after60more_ticks":true,"profile_unchanged":true});
    let path = std::env::args().nth(1).expect("output JSON path");
    std::fs::write(path, serde_json::to_string_pretty(&output).unwrap() + "\n").unwrap();
    println!(
        "Native Haven extraction:5 objects,2 frames; live signals/frozen Reduced Motion/profile checks passed"
    );
}
