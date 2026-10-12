use super::*;
use crate::ui_scene::{UiGroup, UiNode, UiScene};
use engine_core::RenderFrame;

fn input(kind: Kind, rect: [f32; 4]) -> Input {
    Input {
        parent: AffineTransform::default(),
        rect,
        clip: None,
        kind,
        accent: ENAMEL,
        active: false,
        selected: false,
        disabled: false,
    }
}
fn signature(frame: &RenderFrame) -> Vec<([f32; 2], u32, [f32; 4])> {
    frame
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels.iter().map(|p| {
                let t = b.transforms[p.transform_slot()];
                (
                    t.point(Vec2::from_array(p.local)).to_array(),
                    p.color,
                    [t.a[2], t.a[3], t.b[0], t.b[1]],
                )
            })
        })
        .collect()
}
fn extract(input: &Input) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    let (mut pixels, mut work) = (60_000, 120_000);
    let count = append(&mut frame, input, &mut pixels, &mut work);
    assert!(count > 0);
    assert_eq!(
        (frame.frame().pixel_count, pixels, work),
        (count, 60_000 - count, 120_000 - count)
    );
    frame.finish()
}
fn color_at(body: &PixelBody, x: usize, y: usize) -> Option<u32> {
    body.cells_indexed()
        .find(|(i, _, _)| *i == y * body.width + x)
        .map(|(_, _, c)| c)
}

#[test]
fn authored_cut_metal_corners_have_distinct_facets_recesses_enamel_and_locator() {
    for body in [&*PANEL_CORNER, &*BUTTON_CORNER] {
        let size = body.width;
        let rail = size / 3;
        let cut = size * 3 / 8;
        assert_eq!(color_at(body, 0, 0), None, "real chamfer negative space");
        assert_eq!(color_at(body, 0, cut), Some(SPECULAR));
        assert_eq!(color_at(body, rail + 2, rail + 2), Some(GOLD));
        assert_eq!(color_at(body, rail + 1, rail + 2), Some(GRAPHITE));
        assert_eq!(color_at(body, rail - 1, size - 1), Some(ENAMEL));
        assert_eq!(
            color_at(body, size - 1, size - 1),
            None,
            "open inner surface"
        );
        for color in [SPECULAR, PLATINUM, ALLOY, GRAPHITE, ENAMEL, GOLD] {
            assert!(body.cells().any(|(_, c)| c == color));
        }
    }
    println!(
        "SPORTS source corners panel={} button={} chevron={}",
        PANEL_CORNER.occupied_count(),
        BUTTON_CORNER.occupied_count(),
        CHEVRON.occupied_count()
    );
}

#[test]
fn header_competition_wing_has_authored_slope_facets_award_and_recess() {
    assert_eq!((HEADER_WING.width, HEADER_WING.height), (48, 12));
    assert_eq!(color_at(&HEADER_WING, 0, 0), Some(SPECULAR));
    assert_eq!(color_at(&HEADER_WING, 9, 3), Some(ENAMEL));
    assert_eq!(color_at(&HEADER_WING, 9, 6), Some(GOLD));
    assert_eq!(color_at(&HEADER_WING, 9, 2), Some(GRAPHITE));
    assert_eq!(
        color_at(&HEADER_WING, 40, 9),
        None,
        "real cut-back shoulder"
    );
    assert_eq!(color_at(&HEADER_WING, 0, 5), None, "sloped leading edge");
    for color in [SPECULAR, PLATINUM, ALLOY, GRAPHITE, ENAMEL, GOLD] {
        assert!(HEADER_WING.cells().any(|(_, c)| c == color));
    }
    println!(
        "SPORTS refined shoulder source cells={}",
        HEADER_WING.occupied_count()
    );
}

#[test]
fn panel_inherited_focus_and_corporation_cannot_tint_the_opaque_neutral_backing() {
    let mut panel = input(Kind::Panel, [0.0, 0.0, 300.0, 180.0]);
    let baseline = signature(&extract(&panel));
    assert!(baseline.iter().filter(|p| p.1 == RECESS).count() >= 2);
    for color in [rgb(255, 147, 119), rgb(92, 191, 255), rgb(159, 225, 138)] {
        panel.accent = color;
        panel.active = true;
        panel.selected = true;
        assert_eq!(
            signature(&extract(&panel)),
            baseline,
            "focus stays on actual controls"
        );
    }
    let [r, g, b, a] = RECESS.to_le_bytes();
    assert_eq!((r, g, b, a), (18, 18, 18, 255));
}

#[test]
fn neutral_surface_survives_linear_eight_bit_ui_quantization_without_cyan_shift() {
    // Independent model of pixel.wgsl's source conversion into Rgba8Unorm.
    // It reproduces the observed old dark focus-fill failure; not a GPU test.
    let quantized = |color: u32| {
        let [r, g, b, _] = color.to_le_bytes();
        [r, g, b].map(|v| ((v as f32 / 255.0).powf(2.2) * 255.0).round() as u8)
    };
    assert_eq!(quantized(rgb(12, 18, 24)), [0, 1, 1]);
    assert_eq!(quantized(RECESS), [1, 1, 1]);
    for source in [RECESS, subdued(RECESS), rgb(20, 20, 20)] {
        let [r, g, b] = quantized(source);
        assert_eq!((r, g), (g, b));
    }
}

#[test]
fn fractional_tiny_chrome_preserves_full_source_cells_and_local_geometry() {
    for kind in [Kind::Panel, Kind::Button] {
        let mut counts = vec![];
        for r in [
            [0.0, 0.0, 240.0, 120.0],
            [5.125, 3.25, 39.875, 18.125],
            [0.0, 0.0, 2.0, 1.25],
        ] {
            let frame = extract(&input(kind, r));
            counts.push(frame.pixel_count);
            for b in &frame.batches {
                for p in &b.pixels {
                    assert_eq!(p.color >> 24, 255);
                    let t = b.transforms[p.transform_slot()];
                    for c in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ] {
                        let point = t.point(Vec2::from_array(p.local) + c);
                        assert!(
                            point.x >= r[0] - 0.00001
                                && point.y >= r[1] - 0.00001
                                && point.x <= r[0] + r[2] + 0.00001
                                && point.y <= r[1] + r[3] + 0.00001
                        );
                    }
                }
            }
        }
        assert!(
            counts.iter().all(|c| *c == counts[0]),
            "no resized-detail LOD"
        );
        assert!(counts[0] < 1600);
        println!("SPORTS {kind:?} complete cells={}", counts[0]);
    }
}

#[test]
fn partial_clip_retains_and_charges_hidden_cells_with_reflected_sheared_parent() {
    let mut settings = input(Kind::Panel, [11.25, 8.5, 92.25, 55.75]);
    settings.parent = AffineTransform {
        position: Vec2::new(81.0, -43.0),
        basis_x: Vec2::new(-1.25, 0.25),
        basis_y: Vec2::new(0.4, 0.8),
    };
    let complete = extract(&settings);
    let clip = [20.0, 9.0, 60.0, 50.0];
    settings.clip = Some(clip);
    let clipped = extract(&settings);
    assert_eq!(clipped.pixel_count, complete.pixel_count);
    let mut visible = 0;
    for b in &clipped.batches {
        for p in &b.pixels {
            if p.color >> 24 == 0 {
                continue;
            }
            visible += 1;
            let t = b.transforms[p.transform_slot()];
            for c in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ] {
                let point = settings
                    .parent
                    .inverse_point(t.point(Vec2::from_array(p.local) + c));
                assert!(
                    point.x >= clip[0] - 0.00002
                        && point.y >= clip[1] - 0.00002
                        && point.x <= clip[0] + clip[2] + 0.00002
                        && point.y <= clip[1] + clip[3] + 0.00002
                );
            }
        }
    }
    assert!(visible > 0 && visible < clipped.pixel_count);
}

#[test]
fn whole_chrome_quota_and_invalid_transform_rejection_is_atomic() {
    let settings = input(Kind::Panel, [0.0, 0.0, 240.0, 120.0]);
    let count = extract(&settings).pixel_count;
    for (p, w) in [(count - 1, 120000), (60000, count - 1)] {
        let mut frame = FrameBuilder::new();
        frame.push_pixel(Vec2::new(4.0, 6.0), 1.0, 0xffffffff);
        let before = signature(frame.frame());
        let (mut pixels, mut work) = (p, w);
        assert_eq!(append(&mut frame, &settings, &mut pixels, &mut work), 0);
        assert_eq!((pixels, work), (p, w));
        assert_eq!(signature(frame.frame()), before);
    }
    for mut bad in [
        input(Kind::Panel, [0.0, 0.0, f32::NAN, 12.0]),
        input(Kind::Panel, [0.0, 0.0, 0.0, 12.0]),
        input(Kind::Panel, [0.0, 0.0, 240.0, 120.0]),
    ] {
        if bad.rect[2] == 240.0 {
            bad.parent.basis_x = Vec2::splat(f32::MAX);
        }
        let mut frame = FrameBuilder::new();
        let (mut p, mut w) = (60000, 120000);
        assert_eq!(append(&mut frame, &bad, &mut p, &mut w), 0);
        assert_eq!((frame.frame().pixel_count, p, w), (0, 60000, 120000));
    }
    for (x, y) in [
        (Vec2::ZERO, Vec2::Y),
        (Vec2::new(1.0, 2.0), Vec2::new(2.0, 4.0)),
    ] {
        let mut bad = input(Kind::Panel, [0.0, 0.0, 240.0, 120.0]);
        bad.parent.basis_x = x;
        bad.parent.basis_y = y;
        let mut frame = FrameBuilder::new();
        let (mut p, mut w) = (60000, 120000);
        assert_eq!(append(&mut frame, &bad, &mut p, &mut w), 0);
        assert_eq!((frame.frame().pixel_count, p, w), (0, 60000, 120000));
    }
}

#[test]
fn hover_selection_and_corporate_accents_preserve_neutral_metal_geometry() {
    let normal = extract(&input(Kind::Button, [0.0, 0.0, 100.0, 24.0]));
    let original = signature(&normal);
    for accent in [rgb(255, 147, 119), rgb(92, 191, 255), rgb(159, 225, 138)] {
        let mut state = input(Kind::Button, [0.0, 0.0, 100.0, 24.0]);
        state.accent = accent;
        state.active = true;
        let hovered = signature(&extract(&state));
        state.active = false;
        state.selected = true;
        assert_eq!(hovered, signature(&extract(&state)));
        assert!(hovered.iter().any(|p| p.1 == accent));
        for (a, b) in original.iter().zip(&hovered) {
            assert_eq!((a.0, a.2), (b.0, b.2));
            if [PLATINUM, SPECULAR, ALLOY, GRAPHITE, GOLD].contains(&a.1) {
                assert_eq!(a.1, b.1);
            }
        }
        state.disabled = true;
        let disabled = extract(&state);
        assert!(
            signature(&disabled)
                .iter()
                .all(|p| p.1 >> 24 == 255 && p.1 != accent)
        );
        state.active = true;
        assert_eq!(signature(&disabled), signature(&extract(&state)));
    }
}

fn node(kind: &str, rect: [f32; 4], text: &str) -> UiNode {
    UiNode {
        kind: kind.into(),
        rect,
        text: text.into(),
        font: 1.0,
        ..Default::default()
    }
}
fn menu_nodes() -> Vec<UiNode> {
    let mut nodes = vec![
        node("panel", [12.0, 12.0, 302.0, 332.0], ""),
        node("text", [24.0, 22.0, 270.0, 10.0], "RAICHO / FLIGHT SYSTEMS"),
        node("text", [24.0, 39.0, 270.0, 9.0], "FORGE / OWNED EQUIPMENT"),
    ];
    for (i, (kind, text)) in [
        ("primary-button", "CONTINUE"),
        ("button", "DETAILS"),
        ("offense-button", "RENKO / CRAFT"),
        ("defense-button", "HUGO / EQUIP"),
        ("passive-button", "PAN / UPGRADE"),
        ("exit-button", "RETURN TO TITLE"),
    ]
    .into_iter()
    .enumerate()
    {
        nodes.push(node(
            kind,
            [28.0, 65.0 + i as f32 * 33.0, 270.0, 24.0],
            text,
        ));
    }
    let mut slider = node("slider", [342.0, 35.0, 270.0, 30.0], "MUSIC");
    slider.label_rect = Some([350.0, 41.0, 82.0, 12.0]);
    slider.control_rect = Some([450.0, 41.0, 150.0, 12.0]);
    slider.value = 0.65;
    slider.active = true;
    nodes.push(slider);
    for (i, kind) in [
        "status-icon",
        "map-icon",
        "settings-icon",
        "save-icon",
        "close-icon",
    ]
    .into_iter()
    .enumerate()
    {
        nodes.push(node(kind, [342.0 + i as f32 * 44.0, 88.0, 28.0, 28.0], ""));
    }
    nodes.push(UiNode {
        value: 9.0,
        portrait_kind: Some(0),
        portrait_main: Some(0),
        portrait_sub: Some(1),
        ..node("portrait", [365.0, 130.0, 120.0, 160.0], "")
    });
    nodes
}
fn draw_scene(nodes: Vec<UiNode>) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    crate::ui_scene::draw(
        &mut frame,
        &UiScene {
            groups: vec![UiGroup {
                position: [0.0; 2],
                nodes,
            }],
        },
        [0.0; 2],
        [640.0, 360.0],
        0.5,
        true,
    );
    frame.finish()
}

#[test]
fn realistic_five_icon_largest_portrait_menu_keeps_every_isolated_component() {
    let nodes = menu_nodes();
    let joined = draw_scene(menu_nodes());
    let expected: Vec<_> = nodes
        .into_iter()
        .flat_map(|n| signature(&draw_scene(vec![n])))
        .collect();
    assert_eq!(
        signature(&joined),
        expected,
        "late UI must not be omitted by the60k cap"
    );
    assert!(joined.pixel_count < 60000);
    assert!(joined.batches.iter().all(|b| b.transforms.len() <= 128));
    println!(
        "SPORTS realistic joined cells={} transforms={}",
        joined.pixel_count,
        joined
            .batches
            .iter()
            .map(|b| b.transforms.len())
            .sum::<usize>()
    );
}

#[test]
fn optional_native_sports_packet_export_uses_actual_scene_only() {
    let frame = draw_scene(menu_nodes());
    if let Some(folder) = std::env::var_os("GALAXY_SPORTS_UI_REVIEW_OUTPUT") {
        let folder = std::path::PathBuf::from(folder);
        std::fs::create_dir_all(&folder).unwrap();
        let packet = serde_json::json!({"scope":"actual native FrameBuilder UiScene, software approximation only; no GPU/DOM observation",
            "viewport":[640,360],"cells":frame.pixel_count,"batches":frame.batches.iter().map(|b|
                serde_json::json!({"transforms":b.transforms.iter().map(|t|serde_json::json!({"a":t.a,"b":t.b})).collect::<Vec<_>>(),
                    "pixels":b.pixels.iter().map(|p|serde_json::json!({"local":p.local,"color":p.color,"transform":p.transform})).collect::<Vec<_>>()})
                ).collect::<Vec<_>>()});
        std::fs::write(
            folder.join("menu.json"),
            serde_json::to_vec(&packet).unwrap(),
        )
        .unwrap();
    }
}
