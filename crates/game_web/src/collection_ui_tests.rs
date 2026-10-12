use super::*;
use crate::ui_scene::{UiGroup, UiNode, UiScene};
use engine_core::RenderFrame;

fn input(rect: [f32; 4]) -> Input {
    Input {
        parent: AffineTransform::default(),
        rect,
        clip: None,
        active: false,
        selected: false,
        disabled: false,
        time: 0.0,
        reduced: false,
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
fn symbol(kind: SymbolKind, index: usize, input: &Input) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    let (mut pixels, mut work) = (60_000, 120_000);
    let count = append_symbol(&mut frame, input, kind, index, &mut pixels, &mut work);
    assert!(count > 0);
    assert_eq!(
        (frame.frame().pixel_count, pixels, work),
        (count, 60_000 - count, 120_000 - count)
    );
    frame.finish()
}
fn cell(body: &PixelBody, x: usize, y: usize) -> Option<u32> {
    body.cells_indexed()
        .find(|(i, _, _)| *i == y * SIZE + x)
        .map(|(_, _, c)| c)
}

#[test]
fn six_authored_symbols_have_distinct_motifs_explicit_metal_landmarks_and_bounded_sources() {
    let mut masks = vec![];
    for (i, art) in SYMBOLS.iter().enumerate() {
        assert_eq!((art.body.width, art.body.height), (32, 32));
        assert!(art.body.occupied_count() <= 600);
        assert_eq!(cell(&art.body, 0, 0), None);
        assert_eq!(cell(&art.body, 6, 8), Some(GOLD));
        assert_eq!(cell(&art.body, 7, 8), Some(RECESS));
        assert_eq!(cell(&art.body, 14, 27), Some(EMITTER));
        assert!(art.body.cells().any(|(_, c)| c == art.accent));
        assert!(art.body.cells().any(|(_, c)| c == WHITE));
        assert!(art.body.cells().any(|(_, c)| c == PLATINUM));
        let mask: Vec<_> = art.body.cells_indexed().map(|(i, _, _)| i).collect();
        assert!(
            !masks.contains(&mask),
            "each role has an independently authored silhouette"
        );
        masks.push(mask);
        println!(
            "COLLECTION symbol{i} cells={} emitter={}",
            art.body.occupied_count(),
            art.body.cells().filter(|(_, c)| *c == EMITTER).count()
        );
    }
    // Manufacturer order PAN/RENKO/HUGO differs from category order. This
    // proves the renderer does not equate construction currency with maker.
    assert_eq!(SYMBOLS[3].accent, CATEGORY_COLORS[2]);
    assert_eq!(SYMBOLS[4].accent, CATEGORY_COLORS[0]);
    assert_eq!(SYMBOLS[5].accent, CATEGORY_COLORS[1]);
}

#[test]
fn fractional_fit_retains_every_source_cell_and_exact_parent_composition() {
    for kind in [SymbolKind::Category, SymbolKind::Corporation] {
        for i in 0..3 {
            let mut original = None;
            for r in [
                [0.0, 0.0, 32.0, 32.0],
                [4.125, 7.25, 14.5, 17.125],
                [0.0, 0.0, 2.0, 2.0],
            ] {
                let mut p = input(r);
                p.parent = AffineTransform {
                    position: Vec2::new(83.0, -12.0),
                    basis_x: Vec2::new(-1.2, 0.3),
                    basis_y: Vec2::new(0.25, 0.9),
                };
                let frame = symbol(kind, i, &p);
                assert!(signature(&frame).iter().all(|p| p.1 >> 24 == 255));
                if let Some(count) = original {
                    assert_eq!(frame.pixel_count, count);
                }
                original = Some(frame.pixel_count);
                let scale = r[2].min(r[3]) / 32.0;
                let art = &SYMBOLS[i + if kind == SymbolKind::Corporation {
                    3
                } else {
                    0
                }];
                let packet = signature(&frame);
                for ((_, local, _), actual) in art.body.cells_indexed().zip(packet) {
                    let expected = p
                        .parent
                        .point(Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5) + local * scale);
                    assert!(expected.distance(Vec2::from_array(actual.0)) < 0.00003);
                }
            }
        }
    }
}

#[test]
fn animation_changes_only_four_emitter_cells_and_reduced_or_disabled_freezes_exactly() {
    for kind in [SymbolKind::Category, SymbolKind::Corporation] {
        for i in 0..3 {
            let mut p = input([0.0, 0.0, 18.0, 18.0]);
            p.active = true;
            let first = signature(&symbol(kind, i, &p));
            p.time = 1.0;
            let second = signature(&symbol(kind, i, &p));
            let art = &SYMBOLS[i + if kind == SymbolKind::Corporation {
                3
            } else {
                0
            }];
            let mut changed = 0;
            for (((_, _, source), a), b) in art.body.cells_indexed().zip(&first).zip(&second) {
                assert_eq!((a.0, a.2), (b.0, b.2));
                if source != EMITTER {
                    assert_eq!(a.1, b.1);
                } else if a.1 != b.1 {
                    changed += 1;
                }
            }
            assert_eq!(changed, 4);
            for (reduced, disabled, selected) in [
                (true, false, false),
                (false, true, false),
                (false, false, true),
            ] {
                p.reduced = reduced;
                p.disabled = disabled;
                p.selected = selected;
                p.time = 0.0;
                let before = signature(&symbol(kind, i, &p));
                p.time = 123.0;
                assert_eq!(before, signature(&symbol(kind, i, &p)));
            }
        }
    }
}

#[test]
fn symbol_partial_clip_is_whole_cell_and_charges_alpha_zero_cells() {
    let mut p = input([10.25, 11.5, 18.25, 18.25]);
    let full = symbol(SymbolKind::Category, 0, &p);
    let clip = [14.0, 13.0, 11.0, 12.0];
    p.clip = Some(clip);
    let partial = symbol(SymbolKind::Category, 0, &p);
    assert_eq!(partial.pixel_count, full.pixel_count);
    assert!(signature(&partial).iter().any(|p| p.1 >> 24 == 0));
    assert!(signature(&partial).iter().any(|p| p.1 >> 24 == 255));
    for b in &partial.batches {
        for cell in &b.pixels {
            if cell.color >> 24 == 0 {
                continue;
            }
            let t = b.transforms[cell.transform_slot()];
            for corner in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ] {
                let point = t.point(Vec2::from_array(cell.local) + corner);
                assert!(
                    point.x >= clip[0]
                        && point.y >= clip[1]
                        && point.x <= clip[0] + clip[2]
                        && point.y <= clip[1] + clip[3]
                );
            }
        }
    }
}

#[test]
fn invalid_tokens_affines_and_both_quota_boundaries_reject_without_any_mutation() {
    let p = input([0.0, 0.0, 16.0, 16.0]);
    let count = SYMBOLS[0].body.occupied_count();
    for (pixels, work, index) in [
        (count - 1, 120000, 0),
        (60000, count - 1, 0),
        (60000, 120000, 3),
    ] {
        let mut f = FrameBuilder::new();
        f.push_pixel(Vec2::ZERO, 1.0, WHITE);
        let before = signature(f.frame());
        let (mut px, mut wk) = (pixels, work);
        assert_eq!(
            append_symbol(&mut f, &p, SymbolKind::Category, index, &mut px, &mut wk),
            0
        );
        assert_eq!(signature(f.frame()), before);
        assert_eq!((px, wk), (pixels, work));
    }
    for parent in [
        AffineTransform {
            basis_x: Vec2::ZERO,
            ..Default::default()
        },
        AffineTransform {
            basis_x: Vec2::new(1.0, 2.0),
            basis_y: Vec2::new(2.0, 4.0),
            ..Default::default()
        },
        AffineTransform {
            position: Vec2::splat(f32::NAN),
            ..Default::default()
        },
    ] {
        let mut bad = input(p.rect);
        bad.parent = parent;
        let mut f = FrameBuilder::new();
        let (mut px, mut wk) = (60000, 120000);
        assert_eq!(
            append_symbol(&mut f, &bad, SymbolKind::Category, 0, &mut px, &mut wk),
            0
        );
        assert_eq!(append_entry(&mut f, &bad, None, &mut px, &mut wk), 0);
        assert_eq!((f.frame().pixel_count, px, wk), (0, 60000, 120000));
    }
    let mut f = FrameBuilder::new();
    let (mut px, mut wk) = (8, 120000);
    assert_eq!(append_entry(&mut f, &p, None, &mut px, &mut wk), 0);
    assert_eq!((f.frame().pixel_count, px, wk), (0, 8, 120000));
    let (mut px, mut wk) = (9, 9);
    assert_eq!(append_entry(&mut f, &p, Some(2), &mut px, &mut wk), 9);
    assert_eq!((f.frame().pixel_count, px, wk), (9, 0, 0));
}

#[test]
fn entry_clip_and_focus_preserve_geometry_while_admitting_all_nine_cells() {
    let mut p = input([10.25, 12.0, 120.5, 68.0]);
    p.clip = Some([20.0, 18.0, 82.0, 46.0]);
    let extract = |p: &Input| {
        let mut f = FrameBuilder::new();
        let (mut px, mut wk) = (60_000, 120_000);
        assert_eq!(append_entry(&mut f, p, Some(1), &mut px, &mut wk), 9);
        assert_eq!((px, wk), (59_991, 119_991));
        f.finish()
    };
    p.active = true;
    let first = extract(&p);
    p.time = 1.0;
    let second = extract(&p);
    let a = signature(&first);
    let b = signature(&second);
    assert_eq!(a.len(), 9);
    assert!(a.iter().zip(&b).all(|(a, b)| (a.0, a.2) == (b.0, b.2)));
    let clip = p.clip.unwrap();
    for batch in &second.batches {
        for cell in &batch.pixels {
            if cell.color >> 24 == 0 {
                continue;
            }
            for corner in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ] {
                let q = batch.transforms[cell.transform_slot()]
                    .point(Vec2::from_array(cell.local) + corner);
                assert!(
                    q.x >= clip[0]
                        && q.y >= clip[1]
                        && q.x <= clip[0] + clip[2]
                        && q.y <= clip[1] + clip[3]
                );
            }
        }
    }
    p.reduced = true;
    let stable = signature(&extract(&p));
    p.time = 173.0;
    assert_eq!(stable, signature(&extract(&p)));
    let mut f = FrameBuilder::new();
    let (mut px, mut wk) = (60_000, 8);
    assert_eq!(append_entry(&mut f, &p, None, &mut px, &mut wk), 0);
    assert_eq!((f.frame().pixel_count, px, wk), (0, 60_000, 8));
}

fn node(kind: &str, text: &str, rect: [f32; 4], value: f32) -> UiNode {
    UiNode {
        kind: kind.into(),
        text: text.into(),
        rect,
        value,
        font: 1.0,
        ..Default::default()
    }
}
fn draw(nodes: Vec<UiNode>, time: f32, reduced: bool) -> RenderFrame {
    let mut f = FrameBuilder::new();
    crate::ui_scene::draw(
        &mut f,
        &UiScene {
            groups: vec![UiGroup {
                position: [0.0; 2],
                nodes,
            }],
        },
        [0.0; 2],
        [960.0, 540.0],
        time,
        reduced,
    );
    f.finish()
}

#[test]
fn actual_semantic_hierarchy_keeps_identical_copy_geometry_and_never_infers_cost_or_ownership() {
    for (kind, value, expected) in [
        ("collection-name", 0.0, crate::ui_scene::WHITE),
        ("collection-meta", 0.0, crate::ui_scene::DIM),
        ("collection-cost", 0.0, crate::ui_scene::WHITE),
        ("collection-reason", 0.0, GOLD),
        ("collection-state", 0.0, crate::ui_scene::DIM),
        ("collection-state", 1.0, crate::ui_scene::GOLD),
        ("collection-state", 2.0, crate::ui_scene::WHITE),
        ("collection-state", 3.0, crate::ui_scene::CYAN),
    ] {
        let original = draw(
            vec![node(
                "text",
                "OWNED RANK 5 / ACTIVE 1",
                [10.0, 10.0, 160.0, 12.0],
                0.0,
            )],
            0.0,
            true,
        );
        let styled = draw(
            vec![node(
                kind,
                "OWNED RANK 5 / ACTIVE 1",
                [10.0, 10.0, 160.0, 12.0],
                value,
            )],
            0.0,
            true,
        );
        let original = signature(&original);
        let styled = signature(&styled);
        assert_eq!(original.len(), styled.len());
        assert!(!styled.is_empty());
        for (a, b) in original.iter().zip(&styled) {
            assert_eq!((a.0, a.2), (b.0, b.2));
            if b.1 >> 24 != 0 {
                assert_eq!(b.1, expected);
            }
        }
    }
    for value in [-1.0, 0.5, 3.0, f32::NAN, f32::INFINITY] {
        for kind in ["category-symbol", "corporation-symbol"] {
            assert_eq!(
                draw(
                    vec![node(
                        kind,
                        "UNUSED CATALOG NAME",
                        [0.0, 0.0, 16.0, 16.0],
                        value
                    )],
                    0.0,
                    true
                )
                .pixel_count,
                0
            );
        }
    }
    for index in 0..3 {
        for (kind, native_kind) in [
            ("category-symbol", SymbolKind::Category),
            ("corporation-symbol", SymbolKind::Corporation),
        ] {
            let r = [10.25, 11.0, 16.0, 16.0];
            let actual = draw(vec![node(kind, "", r, index as f32)], 0.0, true);
            let mut p = input(r);
            p.parent.position = Vec2::new(-480.0, -270.0);
            p.reduced = true;
            let expected = symbol(native_kind, index, &p);
            assert_eq!(
                signature(&actual),
                signature(&expected),
                "semantic corp/category token routes exactly"
            );
        }
    }
}

fn collection_nodes() -> Vec<UiNode> {
    let mut nodes = vec![node("panel", "", [8.0, 8.0, 944.0, 524.0], 0.0)];
    for category in 0..3 {
        let x = 24.0 + category as f32 * 194.0;
        let selected = category == 0;
        nodes.push(UiNode {
            selected,
            ..node(
                "collection-entry",
                "",
                [x, 78.0, 184.0, 245.0],
                category as f32,
            )
        });
        nodes.push(UiNode {
            selected,
            ..node(
                "category-symbol",
                "",
                [x + 8.0, 86.0, 16.0, 16.0],
                category as f32,
            )
        });
        nodes.push(node(
            "collection-name",
            ["PULSE LENS", "GUARD COIL", "FLUX DRIVE"][category],
            [x + 28.0, 89.0, 145.0, 12.0],
            0.0,
        ));
        nodes.push(node(
            "collection-state",
            ["EQUIPPED", "OWNED", "BLUEPRINT"][category],
            [x + 8.0, 112.0, 166.0, 12.0],
            [3.0, 2.0, 1.0][category],
        ));
        nodes.push(node(
            "collection-meta",
            "OWNED RANK 5 / ACTIVE 1",
            [x + 8.0, 136.0, 168.0, 24.0],
            0.0,
        ));
        nodes.push(node(
            "collection-cost",
            "T3 2 / POINTS 10",
            [x + 8.0, 175.0, 168.0, 18.0],
            0.0,
        ));
        nodes.push(node(
            "collection-reason",
            "REQUIRES RAICHO LV 5",
            [x + 8.0, 202.0, 168.0, 22.0],
            0.0,
        ));
        nodes.push(node(
            ["offense-button", "defense-button", "passive-button"][category],
            "UPGRADE",
            [x + 8.0, 267.0, 168.0, 36.0],
            0.0,
        ));
    }
    for (i, kind) in [
        "status-icon",
        "map-icon",
        "settings-icon",
        "save-icon",
        "close-icon",
    ]
    .iter()
    .enumerate()
    {
        nodes.push(node(
            kind,
            "",
            [24.0 + i as f32 * 36.0, 25.0, 24.0, 24.0],
            0.0,
        ));
    }
    nodes.push(UiNode {
        portrait_kind: Some(0),
        portrait_main: Some(0),
        portrait_sub: Some(1),
        ..node("portrait", "", [640.0, 78.0, 180.0, 240.0], 9.0)
    });
    nodes
}

#[test]
fn actual_collection_route_retains_all_supplied_components_and_largest_portrait_under_shared_cap() {
    let actual = draw(collection_nodes(), 0.0, true);
    let expected: Vec<_> = collection_nodes()
        .into_iter()
        .flat_map(|node| signature(&draw(vec![node], 0.0, true)))
        .collect();
    assert_eq!(
        signature(&actual),
        expected,
        "supplied cards, reasons and later portrait must not be omitted"
    );
    assert!(actual.pixel_count < crate::ui_scene::MAX_UI_PIXELS);
    assert!(actual.batches.iter().all(|b| {
        b.transforms.len() <= 128
            && b.pixels
                .iter()
                .all(|p| (p.transform_slot()) < b.transforms.len())
    }));
    println!(
        "COLLECTION actual joined cells={} transforms={}",
        actual.pixel_count,
        actual
            .batches
            .iter()
            .map(|b| b.transforms.len())
            .sum::<usize>()
    );
}

#[test]
fn optional_collection_native_packet_export() {
    let frame = draw(collection_nodes(), 0.0, true);
    if let Some(path) = std::env::var_os("GALAXY_COLLECTION_REVIEW_OUTPUT") {
        let path = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&path).unwrap();
        let packet = serde_json::json!({"scope":"actual native UiScene packet; software only, not current DOM/browser", "viewport":[960,540],"cells":frame.pixel_count,
            "batches":frame.batches.iter().map(|b|serde_json::json!({"transforms":b.transforms.iter().map(|t|serde_json::json!({"a":t.a,"b":t.b})).collect::<Vec<_>>(),"pixels":b.pixels.iter().map(|p|serde_json::json!({"local":p.local,"color":p.color,"transform":p.transform})).collect::<Vec<_>>()})).collect::<Vec<_>>()});
        std::fs::write(
            path.join("collection.json"),
            serde_json::to_vec(&packet).unwrap(),
        )
        .unwrap();
    }
}
