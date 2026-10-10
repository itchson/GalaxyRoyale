use game_web::boss_layout::*;

fn frame(screen: Screen) -> FrameInput<'static> {
    FrameInput {
        screen,
        projection: Projection {
            camera: Point::new(1800.0, -200.0),
            zoom: 1.0,
        },
        head: Point::new(1800.0, -410.0),
        core: None,
        player: None,
        reserved: &[],
        now_ms: 0,
        phase_epoch: 0,
    }
}

fn world_for(input: &FrameInput<'_>, point: Point) -> Point {
    Point::new(
        input.projection.camera.x + (point.x - input.screen.width * 0.5) / input.projection.zoom,
        input.projection.camera.y + (point.y - input.screen.height * 0.5) / input.projection.zoom,
    )
}

fn placed(outcome: Outcome) -> Placement {
    match outcome {
        Outcome::Placed(p) => p,
        other => panic!("expected placement, got {other:?}"),
    }
}

fn assert_near(a: f64, b: f64) {
    assert!((a - b).abs() < 1.0e-8, "{a} != {b}");
}

fn assert_clear(input: &FrameInput<'_>, panel: Rect) {
    assert!([220.0, 168.0, 140.0, 100.0].contains(&panel.width));
    assert_eq!(panel.height, 40.0);
    assert_eq!(panel.x.fract(), 0.0);
    assert_eq!(panel.y.fract(), 0.0);
    let [top, right, bottom, left] = input.screen.insets;
    assert!(panel.x >= left && panel.y >= top);
    assert!(panel.right() <= input.screen.width - right);
    assert!(panel.bottom() <= input.screen.height - bottom);
    for target in [input.core, input.player].into_iter().flatten() {
        let disc = target.project(input.screen, input.projection).unwrap();
        // Independent reference closest-point check for the emitted rectangle.
        let dx =
            (disc.center.x - panel.x).min(0.0).abs() + (disc.center.x - panel.right()).max(0.0);
        let dy =
            (disc.center.y - panel.y).min(0.0).abs() + (disc.center.y - panel.bottom()).max(0.0);
        assert!(dx.hypot(dy) > disc.radius, "panel covered {disc:?}");
    }
    for reserved in input.reserved {
        assert!(!panel.intersects(reserved.expanded(4.0)));
    }
}

fn widgets(screen: Screen) -> [Rect; 7] {
    // Known pixel-UI geometry plus synthetic input rectangles for a DOM pause
    // button. These are geometry fixtures, not recorded browser hit bounds.
    [
        Rect::new(12.0, 12.0, 160.0, 28.0),
        Rect::new(screen.width - 214.0, 12.0, 142.0, 24.0),
        Rect::new(screen.width - 42.0, 24.0, 28.0, 36.0),
        Rect::new(screen.width * 0.5 - 88.0, screen.height - 48.0, 176.0, 36.0),
        Rect::new(screen.width - 60.0, screen.height - 144.0, 48.0, 48.0),
        Rect::new(28.0, screen.height - 78.0, 56.0, 56.0),
        Rect::new(screen.width - 76.0, screen.height - 78.0, 56.0, 56.0),
    ]
}

#[test]
fn northern_diagnostic_projection_matches_actual_zoom_limits() {
    let mut input = frame(Screen::new(720.0, 360.0));
    input.projection.zoom = 0.6;
    assert_eq!(
        input.projection.project(input.screen, input.head),
        Some(Point::new(360.0, 54.0))
    );
    assert_near(input.screen.width / input.projection.zoom, 1200.0);
    input.projection.zoom = 1.6;
    assert_near(
        input
            .projection
            .project(input.screen, input.head)
            .unwrap()
            .y,
        -156.0,
    );
    assert_near(input.screen.width / input.projection.zoom, 450.0);
    input.screen = Screen::new(533.0, 300.0);
    assert_near(input.screen.height / input.projection.zoom, 187.5);
    assert_near(
        input
            .projection
            .project(input.screen, input.head)
            .unwrap()
            .y,
        -186.0,
    );
}

#[test]
fn current_largest_core_has_real_radius_plus_clearance() {
    let mut input = frame(Screen::new(720.0, 360.0));
    input.projection.zoom = 1.6;
    let core = WorldDisc::core(input.head, 8.5 * 1.34);
    assert_near(core.world_radius * input.projection.zoom, 18.224);
    assert_eq!(
        core.project(input.screen, input.projection).unwrap().radius,
        32.0
    );
    // A future larger core expands protection; the floor is not a cap.
    let larger = WorldDisc::core(input.head, 40.0);
    assert_eq!(
        larger
            .project(input.screen, input.projection)
            .unwrap()
            .radius,
        72.0
    );
}

#[test]
fn real_f32_zoom_endpoints_promote_without_invalidating_layout() {
    for actual_zoom in [0.6_f32, 1.0_f32, 1.6_f32] {
        let mut input = frame(Screen::new(720.0, 360.0));
        input.projection.zoom = f64::from(actual_zoom);
        input.core = Some(WorldDisc::core(input.head, 11.39));
        let result = placed(LayoutController::default().update(&input));
        assert_clear(&input, result.screen_rect);
    }
    let mut input = frame(Screen::new(720.0, 360.0));
    input.projection.zoom = f64::from(f32::from_bits(1.6_f32.to_bits() + 1));
    assert_eq!(
        LayoutController::default().update(&input),
        Outcome::InvalidInput
    );
}

#[test]
fn north_core_clears_center_hud_without_scaling_panel() {
    for zoom in [0.6, 1.0, 1.6] {
        let mut input = frame(Screen::new(720.0, 360.0));
        input.projection.zoom = zoom;
        input.core = Some(WorldDisc::core(input.head, 11.39));
        input.player = Some(WorldDisc::player(input.projection.camera, 16.0));
        let placement = placed(LayoutController::default().update(&input));
        assert_eq!(placement.anchor, Anchor::UpperLeft);
        assert_eq!(placement.screen_rect, Rect::new(12.0, 68.0, 220.0, 40.0));
        assert_clear(&input, placement.screen_rect);
    }
}

#[test]
fn partial_visibility_is_protected_and_fully_offscreen_core_is_ignored() {
    let mut input = frame(Screen::new(720.0, 360.0));
    let core = WorldDisc::core(world_for(&input, Point::new(-8.0, 88.0)), 0.0);
    assert!(
        core.project(input.screen, input.projection)
            .unwrap()
            .visible(input.screen)
    );
    input.core = Some(core);
    assert_eq!(
        placed(LayoutController::default().update(&input)).anchor,
        Anchor::UpperRight
    );
    input.core = Some(WorldDisc::core(
        world_for(&input, Point::new(-100.0, 88.0)),
        0.0,
    ));
    assert!(
        !input
            .core
            .unwrap()
            .project(input.screen, input.projection)
            .unwrap()
            .visible(input.screen)
    );
    assert_eq!(
        placed(LayoutController::default().update(&input)).anchor,
        Anchor::UpperLeft
    );
}

#[test]
fn exact_core_tangency_is_not_accepted_as_clearance() {
    let mut input = frame(Screen::new(720.0, 360.0));
    input.core = Some(WorldDisc::core(
        world_for(&input, Point::new(264.0, 88.0)),
        0.0,
    ));
    let result = placed(LayoutController::default().update(&input));
    assert_eq!(result.anchor, Anchor::UpperRight);
    assert_clear(&input, result.screen_rect);
}

#[test]
fn exact_corner_tangency_and_widget_clearance_are_excluded() {
    let mut input = frame(Screen::new(720.0, 360.0));
    // 24/18/30 triangle from the lower-right corner of the left panel.
    input.core = Some(WorldDisc {
        center: world_for(&input, Point::new(256.0, 126.0)),
        world_radius: 30.0,
        padding_pixels: 0.0,
        minimum_pixels: 0.0,
    });
    assert_eq!(
        placed(LayoutController::default().update(&input)).anchor,
        Anchor::UpperRight
    );
    input.core = None;
    let touching = [Rect::new(236.0, 68.0, 10.0, 10.0)];
    input.reserved = &touching;
    assert_eq!(
        placed(LayoutController::default().update(&input)).anchor,
        Anchor::UpperRight
    );
    let separated = [Rect::new(237.0, 68.0, 10.0, 10.0)];
    input.reserved = &separated;
    assert_eq!(
        placed(LayoutController::default().update(&input)).anchor,
        Anchor::UpperLeft
    );
}

#[test]
fn widget_gap_boundary_is_exactly_four_logical_pixels() {
    for (gap, expected) in [
        (3.999, Anchor::UpperRight),
        (4.0, Anchor::UpperRight),
        (4.001, Anchor::UpperLeft),
    ] {
        let mut input = frame(Screen::new(720.0, 360.0));
        let reserved = [Rect::new(232.0 + gap, 68.0, 10.0, 10.0)];
        input.reserved = &reserved;
        assert_eq!(
            placed(LayoutController::default().update(&input)).anchor,
            expected
        );
    }
}

#[test]
fn core_and_player_block_separate_top_anchors_then_lower_is_used() {
    let mut input = frame(Screen::new(720.0, 360.0));
    input.core = Some(WorldDisc::core(
        world_for(&input, Point::new(122.0, 88.0)),
        11.0,
    ));
    input.player = Some(WorldDisc::player(
        world_for(&input, Point::new(598.0, 88.0)),
        18.0,
    ));
    let result = placed(LayoutController::default().update(&input));
    assert_eq!(result.anchor, Anchor::LowerCenter);
    assert_eq!(result.screen_rect, Rect::new(250.0, 262.0, 220.0, 40.0));
    assert_clear(&input, result.screen_rect);
}

#[test]
fn known_hud_and_touch_geometry_leave_a_clear_constant_size_panel() {
    for screen in [
        Screen::new(533.0, 300.0),
        Screen::new(640.0, 360.0),
        Screen::new(720.0, 360.0),
        Screen::new(844.0, 390.0),
    ] {
        let reserved = widgets(screen);
        let mut input = frame(screen);
        input.reserved = &reserved;
        input.core = Some(WorldDisc::core(input.head, 11.39));
        input.player = Some(WorldDisc::player(input.projection.camera, 18.0));
        let result = placed(LayoutController::default().update(&input));
        assert_clear(&input, result.screen_rect);
    }
}

#[test]
fn widgets_are_hard_exclusions_and_all_blocked_fails_closed() {
    let mut input = frame(Screen::new(720.0, 360.0));
    let top_widgets = [
        Rect::new(12.0, 68.0, 220.0, 40.0),
        Rect::new(488.0, 68.0, 220.0, 40.0),
    ];
    input.reserved = &top_widgets;
    let mut controller = LayoutController::default();
    assert_eq!(
        placed(controller.update(&input)).anchor,
        Anchor::LowerCenter
    );
    input.core = Some(WorldDisc::core(
        world_for(&input, Point::new(360.0, 282.0)),
        0.0,
    ));
    let compact = placed(controller.update(&input));
    assert_clear(&input, compact.screen_rect);
    assert!(compact.screen_rect.width < 220.0);
    let full_widget = [Rect::new(0.0, 0.0, 720.0, 360.0)];
    input.reserved = &full_widget;
    assert_eq!(controller.update(&input), Outcome::NoClearAnchor);
    input.reserved = &top_widgets;
    input.core = None;
    input.phase_epoch = 1;
    assert_eq!(
        placed(controller.update(&input)).anchor,
        Anchor::LowerCenter
    );
}

#[test]
fn unsafe_retained_anchor_moves_immediately_without_waiting() {
    let mut input = frame(Screen::new(720.0, 360.0));
    let mut controller = LayoutController::default();
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
    input.now_ms = 1;
    input.core = Some(WorldDisc::core(
        world_for(&input, Point::new(122.0, 88.0)),
        0.0,
    ));
    let result = placed(controller.update(&input));
    assert_eq!(result.anchor, Anchor::UpperRight);
    assert_clear(&input, result.screen_rect);
}

#[test]
fn safe_challenger_requires_continuous_350_ms() {
    let mut input = frame(Screen::new(720.0, 360.0));
    input.head = world_for(&input, Point::new(650.0, 88.0));
    let mut controller = LayoutController::default();
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
    input.head = world_for(&input, Point::new(70.0, 88.0));
    for time in [100, 200, 449] {
        input.now_ms = time;
        assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
    }
    input.now_ms = 450;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperRight);
}

#[test]
fn phase_change_can_promote_a_meaningfully_better_safe_anchor() {
    let mut input = frame(Screen::new(720.0, 360.0));
    input.head = world_for(&input, Point::new(650.0, 88.0));
    let mut controller = LayoutController::default();
    controller.update(&input);
    input.head = world_for(&input, Point::new(70.0, 88.0));
    input.now_ms = 1;
    input.phase_epoch = 1;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperRight);
}

#[test]
fn small_center_jitter_never_promotes_another_safe_anchor() {
    let mut input = frame(Screen::new(720.0, 360.0));
    let mut controller = LayoutController::default();
    for time in 0..2000 {
        input.head = world_for(
            &input,
            Point::new(360.0 + if time % 2 == 0 { 1.0 } else { -1.0 }, 54.0),
        );
        input.now_ms = time;
        assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
    }
}

#[test]
fn improvement_threshold_is_24_pixels_and_interruption_restarts_hold() {
    let mut input = frame(Screen::new(720.0, 360.0));
    let mut controller = LayoutController::default();
    input.head = world_for(&input, Point::new(650.0, 88.0));
    controller.update(&input);
    // Between the panels, x348 gives right140 minus left116 =24 exactly.
    input.head = world_for(&input, Point::new(348.0, 88.0));
    input.now_ms = 100;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
    input.head = world_for(&input, Point::new(348.001, 88.0));
    input.now_ms = 449;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
    input.head = world_for(&input, Point::new(348.0, 88.0));
    input.now_ms = 450;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
    input.now_ms = 799;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
    input.now_ms = 800;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperRight);
}

#[test]
fn any_clear_candidate_is_reported_instead_of_silently_hiding_the_panel() {
    let screen = Screen::new(533.0, 300.0);
    let reserved = widgets(screen);
    let mut input = frame(screen);
    input.reserved = &reserved;
    input.projection.zoom = f64::from(1.6_f32);
    // Current largest player is the generated Haven Scout: radius≈19.90026
    // after its0.7 actor scale.20 is a conservative authored-content bound.
    input.player = Some(WorldDisc::player(input.projection.camera, 20.0));
    let mut controller = LayoutController::default();
    for x in 0..=30 {
        for y in 0..=20 {
            input.head = world_for(
                &input,
                Point::new(
                    -40.0 + 613.0 * x as f64 / 30.0,
                    -40.0 + 380.0 * y as f64 / 20.0,
                ),
            );
            input.core = Some(WorldDisc::core(input.head, 11.39));
            input.now_ms += 16;
            let result = placed(controller.update(&input));
            assert_clear(&input, result.screen_rect);
        }
    }
}

#[test]
fn tiny_viewport_and_expanded_quick_menu_need_explicit_no_fit_fallback() {
    let mut input = frame(Screen::new(320.0, 180.0));
    input.projection.zoom = f64::from(1.6_f32);
    input.player = Some(WorldDisc::player(input.projection.camera, 20.0));
    let compact = placed(LayoutController::default().update(&input));
    assert!(compact.screen_rect.width < 220.0);
    assert_clear(&input, compact.screen_rect);
    input = frame(Screen::new(533.0, 300.0));
    let overlays = [
        Rect::new(0.0, 50.0, 533.0, 90.0),    // expanded quick menu fixture
        Rect::new(145.0, 196.0, 240.0, 60.0), // wide toast fixture
    ];
    input.reserved = &overlays;
    let compact = placed(LayoutController::default().update(&input));
    assert_clear(&input, compact.screen_rect);
}

#[test]
fn compact_fallback_clears_actual_player_and_native_mobile_widget_fixtures() {
    for screen in [Screen::new(320.0, 180.0), Screen::new(400.0, 240.0)] {
        let reserved = widgets(screen);
        let mut input = frame(screen);
        input.projection.zoom = f64::from(1.6_f32);
        input.player = Some(WorldDisc::player(input.projection.camera, 20.0));
        input.core = Some(WorldDisc::core(input.head, 11.39));
        input.reserved = &reserved;
        let result = placed(LayoutController::default().update(&input));
        assert_clear(&input, result.screen_rect);
        assert!(result.screen_rect.width < 220.0);
        input.projection.zoom = f64::from(0.6_f32);
        let result = placed(LayoutController::default().update(&input));
        assert_clear(&input, result.screen_rect);
    }
}

#[test]
fn compact_rescue_returns_to_primary_only_after_safe_hold() {
    let mut input = frame(Screen::new(320.0, 180.0));
    let reserved = widgets(input.screen);
    input.projection.zoom = f64::from(1.6_f32);
    input.player = Some(WorldDisc::player(input.projection.camera, 20.0));
    input.reserved = &reserved;
    let mut controller = LayoutController::default();
    let compact = placed(controller.update(&input));
    assert!(compact.screen_rect.width < 220.0);
    // Expanding the surface makes the normal primary safe while preserving
    // the old rectangle's safety. The panel may not immediately jump wider.
    input.screen = Screen::new(533.0, 300.0);
    input.reserved = &[];
    input.now_ms = 10;
    let retained = placed(controller.update(&input));
    assert_eq!(retained.screen_rect.width, compact.screen_rect.width);
    input.now_ms = 359;
    assert!(placed(controller.update(&input)).screen_rect.width < 220.0);
    input.now_ms = 360;
    assert_eq!(placed(controller.update(&input)).screen_rect.width, 220.0);
}

#[test]
fn unsupported_screen_extent_is_invalid_instead_of_claiming_no_fit() {
    for screen in [Screen::new(4097.0, 2160.0), Screen::new(4096.0, 2161.0)] {
        let input = frame(screen);
        assert_eq!(
            LayoutController::default().update(&input),
            Outcome::InvalidInput
        );
    }
}

#[test]
fn invalid_frame_preserves_pending_state_so_caller_must_reset_on_gaps() {
    let mut input = frame(Screen::new(720.0, 360.0));
    input.head = world_for(&input, Point::new(650.0, 88.0));
    let mut controller = LayoutController::default();
    controller.update(&input);
    input.head = world_for(&input, Point::new(70.0, 88.0));
    input.now_ms = 100;
    controller.update(&input);
    let pending = controller.clone();
    input.projection.zoom = f64::NAN;
    input.now_ms = 449;
    assert_eq!(controller.update(&input), Outcome::InvalidInput);
    assert_eq!(controller, pending);
    input.projection.zoom = 1.0;
    input.now_ms = 450;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperRight);
    controller.reset();
    assert_eq!(controller, LayoutController::default());
}

#[test]
fn clock_rewind_and_interrupted_challenge_restart_the_hold() {
    let mut input = frame(Screen::new(720.0, 360.0));
    let mut controller = LayoutController::default();
    input.head = world_for(&input, Point::new(650.0, 88.0));
    controller.update(&input);
    input.head = world_for(&input, Point::new(70.0, 88.0));
    input.now_ms = 100;
    controller.update(&input);
    input.now_ms = 400;
    controller.update(&input);
    input.now_ms = 50;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
    input.now_ms = 399;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
    input.now_ms = 400;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperRight);
}

#[test]
fn lower_fallback_returns_to_an_upper_anchor_after_hold() {
    let top_widgets = [
        Rect::new(12.0, 68.0, 220.0, 40.0),
        Rect::new(488.0, 68.0, 220.0, 40.0),
    ];
    let mut input = frame(Screen::new(720.0, 360.0));
    input.reserved = &top_widgets;
    let mut controller = LayoutController::default();
    assert_eq!(
        placed(controller.update(&input)).anchor,
        Anchor::LowerCenter
    );
    input.reserved = &[];
    input.now_ms = 10;
    assert_eq!(
        placed(controller.update(&input)).anchor,
        Anchor::LowerCenter
    );
    input.now_ms = 360;
    assert_eq!(placed(controller.update(&input)).anchor, Anchor::UpperLeft);
}

#[test]
fn safe_area_fractional_insets_snap_inward_to_real_pixels() {
    let mut input = frame(Screen {
        width: 720.0,
        height: 360.0,
        insets: [11.2, 20.8, 9.3, 35.4],
    });
    input.player = Some(WorldDisc::player(input.projection.camera, 18.0));
    for (head_x, expected_x) in [(650.0, 48.0), (70.0, 467.0)] {
        input.head = world_for(&input, Point::new(head_x, 54.0));
        let result = placed(LayoutController::default().update(&input));
        assert_eq!(result.screen_rect.x, expected_x);
        assert_eq!(result.screen_rect.y, 80.0);
        assert_clear(&input, result.screen_rect);
    }
}

#[test]
fn large_world_translation_and_camera_shake_use_same_projection() {
    let mut base = frame(Screen::new(720.0, 360.0));
    base.projection.zoom = 1.6;
    base.core = Some(WorldDisc::core(
        world_for(&base, Point::new(122.0, 88.0)),
        11.39,
    ));
    let baseline = LayoutController::default().update(&base);
    let delta = Point::new(990_000.0, -980_000.0);
    base.projection.camera.x += delta.x;
    base.projection.camera.y += delta.y;
    base.head.x += delta.x;
    base.head.y += delta.y;
    let core = base.core.as_mut().unwrap();
    core.center.x += delta.x;
    core.center.y += delta.y;
    assert_eq!(LayoutController::default().update(&base), baseline);
    base.projection.camera.x += 5.0;
    base.projection.camera.y -= 5.0;
    let result = placed(LayoutController::default().update(&base));
    assert_clear(&base, result.screen_rect);
}

#[test]
fn oversize_core_and_tiny_viewport_have_no_forced_unsafe_placement() {
    let mut input = frame(Screen::new(720.0, 360.0));
    input.core = Some(WorldDisc::core(input.projection.camera, 2000.0));
    assert_eq!(
        LayoutController::default().update(&input),
        Outcome::NoClearAnchor
    );
    input.core = None;
    input.screen = Screen::new(96.0, 60.0);
    assert_eq!(
        LayoutController::default().update(&input),
        Outcome::NoClearAnchor
    );
}

#[test]
fn invalid_inputs_fail_closed_and_preserve_retained_state() {
    let input = frame(Screen::new(720.0, 360.0));
    let mut controller = LayoutController::default();
    controller.update(&input);
    let before = controller.clone();
    let mut bad = frame(input.screen);
    for zoom in [f64::NAN, f64::INFINITY, 0.0, 0.599, 1.601] {
        bad.projection.zoom = zoom;
        assert_eq!(controller.update(&bad), Outcome::InvalidInput);
        assert_eq!(controller, before);
    }
    bad = frame(input.screen);
    bad.core = Some(WorldDisc::core(Point::new(f64::NAN, 0.0), 11.0));
    assert_eq!(controller.update(&bad), Outcome::InvalidInput);
    bad = frame(Screen {
        width: 720.0,
        height: 360.0,
        insets: [0.0, 0.0, 0.0, 721.0],
    });
    assert_eq!(controller.update(&bad), Outcome::InvalidInput);
    let reserves = [Rect::new(0.0, 0.0, 1.0, 1.0); 65];
    bad = frame(input.screen);
    bad.reserved = &reserves;
    assert_eq!(controller.update(&bad), Outcome::InvalidInput);
    assert_eq!(controller, before);
}

#[test]
fn oversized_projection_overflow_is_invalid_not_false_safe() {
    let mut input = frame(Screen::new(720.0, 360.0));
    input.projection.camera.x = -f64::MAX;
    input.head.x = f64::MAX;
    assert_eq!(
        LayoutController::default().update(&input),
        Outcome::InvalidInput
    );
    input = frame(Screen::new(720.0, 360.0));
    input.projection.zoom = 1.6;
    input.core = Some(WorldDisc::core(input.head, f64::MAX));
    assert_eq!(
        LayoutController::default().update(&input),
        Outcome::InvalidInput
    );
}

#[test]
fn deterministic_replay_covers_moving_core_widgets_and_phase_changes() {
    let mut a = LayoutController::default();
    let mut b = LayoutController::default();
    let screen = Screen::new(533.0, 300.0);
    let reserved = widgets(screen);
    let mut input = frame(screen);
    input.reserved = &reserved;
    for index in 0..1600 {
        input.now_ms = index * 16;
        input.phase_epoch = index / 180;
        input.projection.zoom = if index % 500 < 250 { 0.6 } else { 1.6 };
        let angle = index as f64 * 0.012;
        input.head = world_for(
            &input,
            Point::new(266.0 + angle.sin() * 250.0, 150.0 + angle.cos() * 140.0),
        );
        input.core = Some(WorldDisc::core(input.head, 11.39));
        input.player = Some(WorldDisc::player(input.projection.camera, 18.0));
        let output = a.update(&input);
        assert_eq!(output, b.update(&input));
        assert_eq!(a, b);
        if let Outcome::Placed(p) = output {
            assert_clear(&input, p.screen_rect);
        }
    }
}

#[test]
fn viewport_zoom_and_core_position_sweep_never_covers_visible_targets() {
    let mut checked = 0;
    for (width, height) in [
        (320.0, 180.0),
        (533.0, 300.0),
        (640.0, 360.0),
        (720.0, 360.0),
        (844.0, 390.0),
    ] {
        for zoom in [0.6, 1.0, 1.6] {
            for insets in [[0.0; 4], [12.4, 16.7, 8.1, 22.3]] {
                let mut input = frame(Screen {
                    width,
                    height,
                    insets,
                });
                input.projection.zoom = zoom;
                let reserved = widgets(input.screen);
                input.reserved = &reserved;
                let mut controller = LayoutController::default();
                for x in 0..=20 {
                    for y in 0..=12 {
                        input.head = world_for(
                            &input,
                            Point::new(
                                -40.0 + (width + 80.0) * x as f64 / 20.0,
                                -40.0 + (height + 80.0) * y as f64 / 12.0,
                            ),
                        );
                        input.core = Some(WorldDisc::core(input.head, 11.39));
                        input.player = Some(WorldDisc::player(input.projection.camera, 18.0));
                        input.now_ms += 16;
                        match controller.update(&input) {
                            Outcome::Placed(p) => assert_clear(&input, p.screen_rect),
                            Outcome::NoClearAnchor => {}
                            Outcome::InvalidInput => panic!("valid geometry failed"),
                        }
                        checked += 1;
                    }
                }
            }
        }
    }
    assert_eq!(checked, 8190);
}
