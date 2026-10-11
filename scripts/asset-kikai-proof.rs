use engine_core::{AffineTransform, FrameBuilder, Transform, Vec2};
use kikai_art_review::{self as art, CorporateAppearance, Corporation, DoraExpression, DoraInput};
use serde_json::{Value, json};

fn packets(frame: &FrameBuilder) -> Vec<Value> {
    frame
        .frame()
        .batches
        .iter()
        .flat_map(|batch| {
            batch.pixels.iter().map(|pixel| {
                let transform = batch.transforms[pixel.transform as usize];
                let local = Vec2::from_array(pixel.local);
                let corners = [
                    Vec2::splat(-0.5),
                    Vec2::new(0.5, -0.5),
                    Vec2::new(-0.5, 0.5),
                    Vec2::splat(0.5),
                ]
                .map(|corner| transform.point(local + corner).to_array());
                json!({"corners":corners,"color":pixel.color,"material":transform.b[2..]})
            })
        })
        .collect()
}
fn main() {
    let mut scenes = Vec::new();
    for (scene_index, (name, expression, speaking, time, reduced, main, sub)) in [
        (
            "OPR_neutral",
            DoraExpression::Neutral,
            false,
            0.0,
            true,
            Corporation::Pan,
            Corporation::Renko,
        ),
        (
            "OPR_focused",
            DoraExpression::Focused,
            false,
            0.0,
            true,
            Corporation::Pan,
            Corporation::Renko,
        ),
        (
            "OPR_concerned",
            DoraExpression::Concerned,
            false,
            0.0,
            true,
            Corporation::Pan,
            Corporation::Renko,
        ),
        (
            "OPR_pleased",
            DoraExpression::Pleased,
            false,
            0.0,
            true,
            Corporation::Pan,
            Corporation::Renko,
        ),
        (
            "OPR_blink",
            DoraExpression::Neutral,
            false,
            4.4,
            false,
            Corporation::Pan,
            Corporation::Renko,
        ),
        (
            "OPR_speech_open",
            DoraExpression::Neutral,
            true,
            0.34,
            false,
            Corporation::Pan,
            Corporation::Renko,
        ),
        (
            "LONG_HAIR_RENKO_PAN",
            DoraExpression::Neutral,
            false,
            0.0,
            true,
            Corporation::Renko,
            Corporation::Pan,
        ),
        (
            "LONG_HAIR_HUGO_RENKO",
            DoraExpression::Neutral,
            false,
            0.0,
            true,
            Corporation::Hugo,
            Corporation::Renko,
        ),
        (
            "LONG_HAIR_PAN_HUGO",
            DoraExpression::Neutral,
            false,
            0.0,
            true,
            Corporation::Pan,
            Corporation::Hugo,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let input = DoraInput {
            parent: AffineTransform::default(),
            rect: [0.0, 0.0, 96.0, 128.0],
            clip: None,
            time,
            reduced_motion: reduced,
            expression,
            speaking,
            disabled: false,
        };
        let mut frame = FrameBuilder::new();
        let (mut pixels, mut work) = (60_000, 120_000);
        let is_guide = scene_index < 6;
        let stats = if is_guide {
            art::append_dora(&mut frame, &input, &mut pixels, &mut work)
        } else {
            art::append_corporate_portrait(
                &mut frame,
                &input,
                CorporateAppearance { main, sub },
                &mut pixels,
                &mut work,
            )
        };
        assert_eq!(stats.quads, 8656);
        assert_eq!(stats.quads, stats.visible_cells);
        assert_eq!(stats.quads, frame.frame().pixel_count);
        scenes.push(json!({"name":name,"width":96,"height":128,"quads":stats.quads,"transforms":stats.transforms,
            "api":if is_guide {"append_dora"} else {"append_corporate_portrait"},
            "identity":if is_guide {"DORA-PAN-RENKO-1337"} else {"long-hair recolor only; not DORO/TALO"},
            "blink":stats.blink,"mouth_frame":stats.mouth_frame,"cells":packets(&frame)}));
    }
    let mut icons = Vec::new();
    let accent = u32::from_le_bytes([88, 222, 239, 255]);
    for kind in [
        "status-icon",
        "map-icon",
        "settings-icon",
        "save-icon",
        "close-icon",
        "resume-icon",
        "restart-icon",
        "title-icon",
        "quit-icon",
        "fullscreen-icon",
    ] {
        let body = art::icon(kind).unwrap();
        let mut frame = FrameBuilder::new();
        frame.push_body_affine_colored(
            body,
            Transform {
                position: Vec2::splat(16.0),
                ..Transform::default()
            }
            .affine(),
            |_, color| art::icon_color(color, accent, false, false, false),
        );
        icons.push(json!({"name":kind,"width":32,"height":32,"quads":body.occupied_count(),"cells":packets(&frame)}));
    }
    let output = json!({"scope":"Actual native OPR#1337 PAN-hair/RENKO-crystal cached PixelBody/FrameBuilder extraction; posed state, flat software review only",
        "portraits":scenes,"icons":icons});
    let path = std::env::args().nth(1).expect("OutputJSONpath");
    std::fs::write(path, serde_json::to_string(&output).unwrap()).unwrap();
    println!("Actual native portrait/icon packets emitted:9 posed portraits,10 controls");
}
