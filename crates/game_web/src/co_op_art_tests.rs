use super::*;
use engine_core::RenderFrame;

fn input(kind: Kind, rect: [f32; 4]) -> Input {
    Input {
        kind,
        rect,
        clip: None,
        parent: AffineTransform::default(),
        style: Style::default(),
    }
}
fn draw(input: &Input) -> RenderFrame {
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
fn signature(frame: &RenderFrame) -> Vec<([f32; 2], u32)> {
    frame
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels.iter().map(|p| {
                (
                    b.transforms[p.transform_slot()]
                        .point(Vec2::from_array(p.local))
                        .to_array(),
                    p.color,
                )
            })
        })
        .collect()
}

#[test]
fn authored_invite_and_join_have_distinct_landmarks_negative_space_and_materials() {
    let invite = &art(Kind::Invite).unwrap().body;
    let join = &art(Kind::Join).unwrap().body;
    assert_eq!(invite.color_at(30, 25), Some(CYAN), "outward beacon stem");
    assert_eq!(invite.color_at(19, 49), Some(GOLD), "plinth fastener");
    assert_eq!(
        invite.color_at(32, 41),
        None,
        "arrow and plinth stay separated"
    );
    assert_eq!(
        invite.color_at(32, 10),
        Some(PLATINUM),
        "broad athletic arrow head"
    );
    assert_eq!(
        invite.color_at(39, 19),
        Some(ALLOY),
        "separate right head facet"
    );
    assert_eq!(join.color_at(31, 30), Some(CYAN), "central receiver signal");
    assert_eq!(join.color_at(31, 41), Some(GOLD), "receiver latch");
    assert_eq!(join.color_at(10, 17), Some(CYAN), "left docking arm");
    assert_eq!(join.color_at(7, 17), Some(SPECULAR), "arm bevel");
    assert_eq!(
        join.color_at(10, 19),
        Some(CYAN_DARK),
        "segmented instrument recess"
    );
    assert_eq!(
        join.color_at(31, 10),
        None,
        "open top differentiates inward receiver"
    );
    for body in [invite, join] {
        assert_eq!([body.width, body.height], [64, 64]);
        assert!((600..=1800).contains(&body.occupied_count()));
        assert_eq!(body.color_at(0, 0), None);
        for color in [GRAPHITE, ALLOY, PLATINUM, SPECULAR, CYAN, GOLD] {
            assert!(body.cells().any(|(_, c)| c == color));
        }
        assert!(body.cells().all(|(_, c)| c >> 24 == 255));
    }
    assert_ne!(
        invite
            .cells_indexed()
            .map(|p| (p.0, p.2))
            .collect::<Vec<_>>(),
        join.cells_indexed().map(|p| (p.0, p.2)).collect::<Vec<_>>()
    );
    println!(
        "COOP cells Invite={} Join={} Roster={}",
        invite.occupied_count(),
        join.occupied_count(),
        art(Kind::Roster(8)).unwrap().body.occupied_count()
    );
}

#[test]
fn eight_seats_only_illuminate_admitted_count_without_changing_occupancy() {
    let empty = art(Kind::Roster(0)).unwrap();
    let indices: Vec<_> = empty.body.cells_indexed().map(|p| p.0).collect();
    for count in 0..=8 {
        let a = art(Kind::Roster(count)).unwrap();
        assert_eq!([a.body.width, a.body.height], [128, 24]);
        assert_eq!(
            a.body.cells_indexed().map(|p| p.0).collect::<Vec<_>>(),
            indices
        );
        assert!(a.body.occupied_count() <= 1800);
        for slot in 0..8 {
            assert_eq!(
                a.body.color_at(9 + slot * 15, 9),
                Some(if slot < count as usize {
                    CYAN
                } else {
                    GRAPHITE
                })
            );
            assert_eq!(a.body.color_at(9 + slot * 15, 3), Some(PLATINUM));
            assert_eq!(
                a.body.color_at(16 + slot * 15, 12),
                None,
                "seats keep a real separating gap"
            );
        }
    }
    assert!(art(Kind::Roster(9)).is_none());
    assert!(art(Kind::Roster(u8::MAX)).is_none());
    assert!(std::ptr::eq(empty, art(Kind::Roster(0)).unwrap()));
}

#[test]
fn only_explicit_signals_move_and_reduced_motion_idle_and_disabled_freeze() {
    for kind in [Kind::Invite, Kind::Join, Kind::Roster(4), Kind::Roster(8)] {
        let a = art(kind).unwrap();
        let first = Style {
            hovered: true,
            time: 0.2,
            ..Default::default()
        };
        let later = Style { time: 1.4, ..first };
        let mut changes = 0;
        for (i, p, c) in a.body.cells_indexed() {
            let x = a.color(p, c, first);
            let y = a.color(p, c, later);
            if !a.signals[i] {
                assert_eq!((x, y), (c, c));
            }
            changes += usize::from(x != y);
            for static_style in [
                Style {
                    reduced_motion: true,
                    ..first
                },
                Style {
                    disabled: true,
                    ..first
                },
                Style::default(),
            ] {
                assert_eq!(
                    a.color(p, c, static_style),
                    a.color(
                        p,
                        c,
                        Style {
                            time: 22.0,
                            ..static_style
                        }
                    )
                );
            }
            for time in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                assert_eq!(
                    a.color(p, c, Style { time, ..first }),
                    a.color(p, c, Style { time: 0.0, ..first })
                );
            }
            assert_eq!(x >> 24, 255);
        }
        assert!(changes > 0 && changes < a.body.occupied_count() / 4);
    }
    let a = art(Kind::Invite).unwrap();
    assert_eq!(a.color(Vec2::splat(f32::NAN), GOLD, Style::default()), GOLD);
}

#[test]
fn continuous_fit_preserves_all_source_cells_and_full_fractional_bases() {
    for kind in [Kind::Invite, Kind::Join, Kind::Roster(8)] {
        let a = art(kind).unwrap();
        for width in [64.0, 16.0, 7.125, 0.8] {
            let height = width * a.body.height as f32 / a.body.width as f32;
            let rect = [5.25, 8.5, width, height];
            let frame = draw(&input(kind, rect));
            assert_eq!(frame.pixel_count, a.body.occupied_count());
            assert_eq!(
                frame
                    .batches
                    .iter()
                    .map(|b| b.transforms.len())
                    .sum::<usize>(),
                1
            );
            for batch in &frame.batches {
                for p in &batch.pixels {
                    let t = batch.transforms[p.transform_slot()];
                    assert_eq!(t.a[2], width / a.body.width as f32);
                    assert_eq!(t.b[1], t.a[2]);
                    assert_eq!(&t.b[2..], &[0.0, 0.0]);
                    for corner in [
                        Vec2::splat(-0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(-0.5, 0.5),
                        Vec2::splat(0.5),
                    ] {
                        let q = t.point(Vec2::from_array(p.local) + corner);
                        assert!(
                            q.x >= rect[0] - 0.00001
                                && q.y >= rect[1] - 0.00001
                                && q.x <= rect[0] + rect[2] + 0.00001
                                && q.y <= rect[1] + rect[3] + 0.00001
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn local_whole_cell_clipping_precedes_exact_reflected_sheared_parent() {
    let mut i = input(Kind::Join, [10.0, 20.0, 16.0, 16.0]);
    i.clip = Some([17.1, 22.0, 6.3, 10.2]);
    i.parent = AffineTransform {
        position: Vec2::new(30.0, -4.0),
        basis_x: Vec2::new(-1.3, 0.2),
        basis_y: Vec2::new(0.4, 0.7),
    };
    let f = draw(&i);
    let local = Transform {
        position: Vec2::new(18.0, 28.0),
        scale: Vec2::splat(0.25),
        ..Default::default()
    }
    .affine();
    let b = &f.batches[0];
    let mut visible = 0;
    let mut hidden = 0;
    for p in &b.pixels {
        let cell = Vec2::from_array(p.local);
        let source_point = local.point(cell);
        let [x, y, w, h] = i.clip.unwrap();
        let within = source_point.x - 0.125 >= x
            && source_point.y - 0.125 >= y
            && source_point.x + 0.125 <= x + w
            && source_point.y + 0.125 <= y + h;
        assert_eq!(p.color != 0, within);
        if within {
            visible += 1;
        } else {
            hidden += 1;
        }
        let expected = i.parent.point(source_point);
        let actual = b.transforms[p.transform_slot()].point(cell);
        assert!((actual - expected).length() < 0.00001);
    }
    assert!(visible > 0 && hidden > 0);
    assert_eq!(
        f.pixel_count,
        art(Kind::Join).unwrap().body.occupied_count()
    );
}

#[test]
fn whole_object_admission_rejects_either_quota_without_partial_frame_or_debit() {
    for kind in [Kind::Invite, Kind::Join, Kind::Roster(8)] {
        let count = art(kind).unwrap().body.occupied_count();
        for (p, w) in [(count - 1, count), (count, count - 1)] {
            let mut frame = FrameBuilder::new();
            frame.push_pixel(Vec2::ZERO, 1.0, GOLD);
            let before = signature(frame.frame());
            let (mut pixels, mut work) = (p, w);
            assert_eq!(
                append(
                    &mut frame,
                    &input(kind, [0.0, 0.0, 16.0, 16.0]),
                    &mut pixels,
                    &mut work
                ),
                0
            );
            assert_eq!((pixels, work), (p, w));
            assert_eq!(signature(frame.frame()), before);
        }
        let (mut p, mut w) = (count, count);
        let mut f = FrameBuilder::new();
        assert_eq!(
            append(&mut f, &input(kind, [0.0, 0.0, 16.0, 16.0]), &mut p, &mut w),
            count
        );
        assert_eq!((p, w), (0, 0));
    }
}

#[test]
fn invalid_tokens_rectangles_affines_and_overflow_fail_before_append() {
    let base = input(Kind::Invite, [0.0, 0.0, 16.0, 16.0]);
    let mut cases = vec![Input {
        kind: Kind::Roster(9),
        ..base
    }];
    for rect in [
        [0.0, 0.0, 0.0, 16.0],
        [0.0, 0.0, -1.0, 16.0],
        [f32::NAN, 0.0, 16.0, 16.0],
        [0.0, 0.0, 9000.0, 16.0],
    ] {
        cases.push(input(Kind::Invite, rect));
    }
    for parent in [
        AffineTransform {
            basis_x: Vec2::ZERO,
            ..Default::default()
        },
        AffineTransform {
            basis_y: Vec2::X,
            ..Default::default()
        },
        AffineTransform {
            position: Vec2::splat(f32::INFINITY),
            ..Default::default()
        },
        AffineTransform {
            basis_x: Vec2::splat(f32::MAX),
            basis_y: Vec2::Y,
            ..Default::default()
        },
    ] {
        let mut i = input(Kind::Invite, [0.0, 0.0, 16.0, 16.0]);
        i.parent = parent;
        cases.push(i);
    }
    let mut clipped = input(Kind::Invite, [0.0, 0.0, 16.0, 16.0]);
    clipped.clip = Some([0.0, 0.0, f32::INFINITY, 16.0]);
    cases.push(clipped);
    for i in cases {
        let mut f = FrameBuilder::new();
        let (mut p, mut w) = (60000, 120000);
        assert_eq!(append(&mut f, &i, &mut p, &mut w), 0);
        assert_eq!((p, w, f.frame().pixel_count), (60000, 120000, 0));
    }
}

#[test]
fn invite_join_and_full_roster_are_bounded_as_three_native_objects() {
    let mut f = FrameBuilder::new();
    let (mut p, mut w) = (60000, 120000);
    for i in [
        input(Kind::Invite, [0.0, 0.0, 16.0, 16.0]),
        input(Kind::Join, [24.0, 0.0, 16.0, 16.0]),
        input(Kind::Roster(8), [0.0, 24.0, 64.0, 12.0]),
    ] {
        assert!(append(&mut f, &i, &mut p, &mut w) > 0);
    }
    assert!(f.frame().pixel_count <= 5400);
    assert_eq!(f.frame().batches[0].transforms.len(), 3);
}

#[test]
#[ignore = "explicit native packet export for the author software contact sheet"]
fn export_co_op_native_art_packet() {
    let folder = std::path::PathBuf::from(
        std::env::var_os("GALAXY_CO_OP_ART_OUTPUT").expect("owned evidence output directory"),
    );
    std::fs::create_dir_all(&folder).unwrap();
    let mut f = FrameBuilder::new();
    let (mut p, mut w) = (60000, 120000);
    for i in [
        input(Kind::Invite, [12.0, 12.0, 64.0, 64.0]),
        input(Kind::Join, [96.0, 12.0, 64.0, 64.0]),
        input(Kind::Invite, [200.0, 25.0, 16.0, 16.0]),
        input(Kind::Join, [236.0, 25.0, 16.0, 16.0]),
        input(Kind::Roster(0), [12.0, 100.0, 128.0, 24.0]),
        input(Kind::Roster(1), [12.0, 140.0, 128.0, 24.0]),
        input(Kind::Roster(4), [12.0, 180.0, 128.0, 24.0]),
        input(Kind::Roster(8), [12.0, 220.0, 128.0, 24.0]),
        input(Kind::Roster(4), [188.0, 100.0, 64.0, 12.0]),
        input(Kind::Roster(8), [188.0, 140.0, 64.0, 12.0]),
    ] {
        assert!(append(&mut f, &i, &mut p, &mut w) > 0);
    }
    let frame = f.finish();
    let packet = serde_json::json!({"scope":"actual native FrameBuilder output; display sheet is flat RGBA software, not GPU","viewport":[280,260],"cells":frame.pixel_count,"batches":frame.batches.iter().map(|b|serde_json::json!({"transforms":b.transforms.iter().map(|t|serde_json::json!({"a":t.a,"b":t.b})).collect::<Vec<_>>(),"pixels":b.pixels.iter().map(|p|serde_json::json!({"local":p.local,"color":p.color,"transform":p.transform})).collect::<Vec<_>>()})).collect::<Vec<_>>()});
    std::fs::write(
        folder.join("native-packet.json"),
        serde_json::to_vec(&packet).unwrap(),
    )
    .unwrap();
    println!(
        "COOP native review cells={} transforms={}",
        frame.pixel_count,
        frame
            .batches
            .iter()
            .map(|b| b.transforms.len())
            .sum::<usize>()
    );
}
