//! Actual native chart extraction: geometry, material, clip and bounded work.
use super::*;
use engine_core::RenderFrame;

const VIEW: [f32; 2] = [640.0, 360.0];
const GROUP: [f32; 2] = [17.25, 11.5];
fn node(kind: &str, rect: [f32; 4], value: f32) -> UiNode {
    UiNode {
        kind: kind.into(),
        rect,
        value,
        ..Default::default()
    }
}
fn extract(nodes: Vec<UiNode>, camera: [f32; 2], time: f32, reduced: bool) -> RenderFrame {
    let mut frame = FrameBuilder::new();
    draw(
        &mut frame,
        &UiScene {
            groups: vec![UiGroup {
                position: GROUP,
                nodes,
            }],
        },
        camera,
        VIEW,
        time,
        reduced,
    );
    frame.finish()
}
fn points(frame: &RenderFrame, camera: [f32; 2]) -> Vec<(Vec2, u32)> {
    frame
        .batches
        .iter()
        .flat_map(|b| {
            b.pixels.iter().map(move |p| {
                (
                    b.transforms[p.transform as usize].point(Vec2::from_array(p.local))
                        - Vec2::from_array(camera)
                        + Vec2::from_array(VIEW) * 0.5
                        - Vec2::from_array(GROUP),
                    p.color,
                )
            })
        })
        .collect()
}
fn bounds(frame: &RenderFrame, camera: [f32; 2], rect: [f32; 4], tolerance: f32) {
    for b in &frame.batches {
        for p in &b.pixels {
            if p.color.to_le_bytes()[3] == 0 {
                continue;
            }
            for offset in [
                Vec2::splat(-0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::splat(0.5),
            ] {
                let q = b.transforms[p.transform as usize]
                    .point(Vec2::from_array(p.local) + offset)
                    - Vec2::from_array(camera)
                    + Vec2::from_array(VIEW) * 0.5
                    - Vec2::from_array(GROUP);
                assert!(
                    q.x >= rect[0] - tolerance && q.x <= rect[0] + rect[2] + tolerance,
                    "x {q:?} rect{rect:?}"
                );
                assert!(
                    q.y >= rect[1] - tolerance && q.y <= rect[1] + rect[3] + tolerance,
                    "y {q:?} rect{rect:?}"
                );
            }
        }
    }
}
fn same(a: &RenderFrame, b: &RenderFrame) {
    assert_eq!(a.pixel_count, b.pixel_count);
    assert_eq!(a.batches.len(), b.batches.len());
    for (a, b) in a.batches.iter().zip(&b.batches) {
        assert_eq!(a.pixels, b.pixels);
        assert_eq!(a.transforms, b.transforms);
    }
}

#[test]
fn map_icon_and_six_biome_objects_are_cached_distinct_unlit_masks() {
    let (body, _) = menu_icon("map-icon").unwrap();
    assert_eq!((body.width, body.height), (24, 24));
    assert!(std::ptr::eq(body, menu_icon("map-icon").unwrap().0));
    assert!(body.is_occupied(2, 8) && !body.is_occupied(0, 0));
    let mut masks = std::collections::HashSet::new();
    for (kind, body) in MAP_SITES.iter().enumerate() {
        assert!(
            masks.insert(
                (0..9)
                    .flat_map(|y| (0..9).map(move |x| body.is_occupied(x, y)))
                    .collect::<Vec<_>>()
            )
        );
        let frame = extract(
            vec![node("map-site", [0.0, 0.0, 44.0, 44.0], kind as f32)],
            [0.0; 2],
            0.0,
            false,
        );
        assert_eq!(frame.pixel_count, body.occupied_count());
        assert_eq!(frame.batches[0].transforms.len(), 1);
        assert_eq!(frame.batches[0].transforms[0].b[2..], [0.0, 0.0]);
        assert!(
            frame.batches[0]
                .pixels
                .iter()
                .any(|p| p.color == map_color(kind))
        );
        let extents = points(&frame, [0.0; 2]);
        assert!(
            extents
                .iter()
                .all(|(p, _)| (p - Vec2::splat(22.0)).abs().max_element() <= 4.5)
        );
    }
}

#[test]
fn map_icon_stays_centered_in_44_physical_hitbox_at_fractional_ui_scales() {
    for scale in [1.0, 1.375, 2.4, 4.0, 6.0] {
        let rect = [9.25, 8.5, 44.0 / scale, 44.0 / scale];
        let frame = extract(vec![node("map-icon", rect, 0.0)], [0.0; 2], 0.0, false);
        assert!(frame.pixel_count > 20);
        bounds(&frame, [0.0; 2], rect, 0.0001);
    }
}

#[test]
fn rings_have_unique_eightfold_symmetric_perimeters_and_one_transform() {
    for radius in [1, 2, 3, 8, 17, 64, 191, 4095] {
        let cells: Vec<_> = map_ring_cells(radius).collect();
        let unique: std::collections::HashSet<_> =
            cells.iter().map(|p| (p.x as i32, p.y as i32)).collect();
        assert_eq!(unique.len(), cells.len());
        for p in &cells {
            assert!((p.length() - radius as f32).abs() <= 0.75);
            assert!(unique.contains(&(-p.y as i32, p.x as i32)));
        }
        for axis in [(radius, 0), (0, radius), (-radius, 0), (0, -radius)] {
            assert!(unique.contains(&axis));
        }
        assert!(cells.len() <= 8 * (radius as usize + 1));
    }
    for kind in 0..3 {
        let frame = extract(
            vec![node("map-ring", [0.0, 0.0, 67.0, 81.0], kind as f32)],
            [0.0; 2],
            0.0,
            false,
        );
        assert_eq!(frame.batches[0].transforms.len(), 1);
        assert_eq!(frame.pixel_count, map_ring_cells(33).count());
        bounds(&frame, [0.0; 2], [0.0, 0.0, 67.0, 81.0], 0.0001);
    }
}

#[test]
fn field_rings_and_rotating_marker_are_whole_cell_clipped_to_chart() {
    let clip = [20.25, 18.5, 91.5, 68.25];
    for kind in [
        "map-field",
        "map-ring",
        "map-home",
        "map-player",
        "map-site",
    ] {
        let mut n = node(kind, [-12.0, -7.0, 180.0, 140.0], 0.7);
        n.clip = Some(clip);
        let frame = extract(vec![n], [0.0; 2], 0.0, false);
        bounds(&frame, [0.0; 2], clip, 0.0001);
    }
    for heading in [
        0.0,
        0.3,
        std::f32::consts::FRAC_PI_4,
        std::f32::consts::FRAC_PI_2,
        2.4,
        std::f32::consts::PI,
        -0.8,
    ] {
        let rect = [32.25, 15.5, 12.0, 12.0];
        let frame = extract(
            vec![node("map-player", rect, heading)],
            [0.0; 2],
            0.0,
            false,
        );
        assert!(
            frame.batches[0]
                .pixels
                .iter()
                .all(|p| p.color.to_le_bytes()[3] != 0)
        );
        bounds(&frame, [0.0; 2], rect, 0.0001);
    }
}

#[test]
fn player_heading_zero_is_right_and_quarter_turn_is_down() {
    for (heading, direction) in [(0.0, Vec2::X), (std::f32::consts::FRAC_PI_2, Vec2::Y)] {
        let frame = extract(
            vec![node("map-player", [0.0, 0.0, 12.0, 12.0], heading)],
            [0.0; 2],
            0.0,
            false,
        );
        let t = frame.batches[0].transforms[0];
        let a = t.point(Vec2::X) - t.point(Vec2::ZERO);
        assert!((a.normalize() - direction).length() < 0.0001);
    }
}

#[test]
fn selection_is_distinct_from_hover_and_all_chart_packets_freeze_with_reduced_motion() {
    let mut selected = node("map-site", [1.0, 1.0, 44.0, 44.0], 1.0);
    selected.selected = true;
    let frame = extract(vec![selected], [0.0; 2], 0.0, false);
    assert!(
        frame.batches[0]
            .pixels
            .iter()
            .any(|p| p.color == rgb(255, 215, 131))
    );
    let mut selected = node("map-site", [1.0, 1.0, 44.0, 44.0], 1.0);
    selected.selected = true;
    selected.active = true;
    same(&frame, &extract(vec![selected], [0.0; 2], 12.0, true));
    let mut hovered = node("map-site", [1.0, 1.0, 44.0, 44.0], 1.0);
    hovered.active = true;
    assert_ne!(
        frame.batches[0].pixels,
        extract(vec![hovered], [0.0; 2], 0.0, false).batches[0].pixels
    );
    for kind in [
        "map-field",
        "map-ring",
        "map-home",
        "map-player",
        "map-site",
        "map-icon",
    ] {
        same(
            &extract(
                vec![node(kind, [0.0, 0.0, 67.0, 67.0], 1.0)],
                [0.0; 2],
                0.0,
                false,
            ),
            &extract(
                vec![node(kind, [0.0, 0.0, 67.0, 67.0], 1.0)],
                [0.0; 2],
                1000.0,
                true,
            ),
        );
    }
}

#[test]
fn chart_uses_composed_parent_and_keeps_local_geometry_at_far_camera() {
    let rect = [4.25, 7.5, 24.0, 24.0];
    let near = extract(vec![node("map-site", rect, 5.0)], [0.0; 2], 0.0, false);
    let far = extract(
        vec![node("map-site", rect, 5.0)],
        [999000.0, -999000.0],
        0.0,
        false,
    );
    assert_eq!(near.pixel_count, far.pixel_count);
    for ((a, c), (b, d)) in points(&near, [0.0; 2])
        .iter()
        .zip(points(&far, [999000.0, -999000.0]))
    {
        assert_eq!(*c, d);
        assert!((*a - b).abs().max_element() <= 0.0625);
    }
    let parent = AffineTransform {
        position: Vec2::new(72.5, -14.25),
        basis_x: Vec2::new(-1.3, 0.2),
        basis_y: Vec2::new(0.4, 0.8),
    };
    let mut frame = FrameBuilder::new();
    let mut painter = Painter {
        frame: &mut frame,
        parent,
        remaining: 1000,
        viewport: Vec2::from_array(VIEW),
        clip: None,
        work_remaining: 2000,
    };
    painter.map_marker(&node("map-player", rect, 0.7));
    let local = Transform {
        position: Vec2::new(rect[0] + 12.0, rect[1] + 12.0),
        rotation: 0.7,
        scale: Vec2::ONE,
    };
    let expected = parent.compose(local.affine());
    let actual = frame.frame().batches[0].transforms[0];
    for (p, _) in MAP_PLAYER.cells() {
        assert!((actual.point(p) - expected.point(p)).length() < 0.0001);
    }
}

#[test]
fn sixty_four_markers_and_six_rings_stay_bounded_and_oversize_input_cannot_leak_budget() {
    let mut nodes = vec![node("map-field", [0.0, 0.0, 320.0, 180.0], 0.0)];
    for i in 0..64 {
        nodes.push(node(
            "map-site",
            [(i % 8) as f32 * 40.0, (i / 8) as f32 * 20.0, 44.0, 44.0],
            (i % 6) as f32,
        ));
    }
    for i in 0..6 {
        nodes.push(node(
            "map-ring",
            [20.0, 5.0, 160.0 + i as f32 * 4.0, 160.0],
            (i % 3) as f32,
        ));
    }
    let frame = extract(nodes, [0.0; 2], 0.0, false);
    println!(
        "64site+6ring+field native packet: {}quads / {}transforms / {}batches; biome cells {:?}; map icon {}cells",
        frame.pixel_count,
        frame
            .batches
            .iter()
            .map(|b| b.transforms.len())
            .sum::<usize>(),
        frame.batches.len(),
        MAP_SITES.each_ref().map(|b| b.occupied_count()),
        menu_icon("map-icon").unwrap().0.occupied_count()
    );
    assert!(frame.pixel_count < 10_000);
    assert!(frame.batches.iter().all(|b| {
        b.transforms.len() <= 128
            && b.pixels
                .iter()
                .all(|p| (p.transform as usize) < b.transforms.len())
    }));
    let mut frame = FrameBuilder::new();
    let mut painter = Painter {
        frame: &mut frame,
        parent: AffineTransform::default(),
        remaining: 71,
        viewport: Vec2::from_array(VIEW),
        clip: Some([0.0, 0.0, 8192.0, 8192.0]),
        work_remaining: 79,
    };
    painter.map_ring(&node("map-ring", [0.0, 0.0, 8192.0, 8192.0], 0.0));
    assert!(painter.remaining <= 71 && painter.work_remaining <= 79);
    assert!(painter.frame.frame().pixel_count <= 71);
    // A completely clipped ring consumes traversal budget but creates no transform.
    let mut frame = FrameBuilder::new();
    let mut painter = Painter {
        frame: &mut frame,
        parent: AffineTransform::default(),
        remaining: 71,
        viewport: Vec2::from_array(VIEW),
        clip: Some([4000.0, 4000.0, 3.0, 3.0]),
        work_remaining: 79,
    };
    painter.map_ring(&node("map-ring", [0.0, 0.0, 8192.0, 8192.0], 0.0));
    assert_eq!(painter.work_remaining, 0);
    assert_eq!(painter.remaining, 71);
    assert_eq!(painter.frame.frame().pixel_count, 0);
}
