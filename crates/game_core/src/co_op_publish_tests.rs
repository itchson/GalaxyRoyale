use super::*;
use co_op::{PROTOCOL, ParticipantSpec};

fn id(n: u32) -> ParticipantId {
    ParticipantId::parse(&format!("{n:032x}")).unwrap()
}
fn start() -> Game {
    let mut game = Game::new(Profile::default());
    game.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
    game.spawn_timer = 10000.0;
    game
}
fn acquire(game: &mut Game) -> (u32, u32, Counts) {
    let view = game.acquire_co_op_publication(id(100)).unwrap();
    (view.acquisition, view.snapshot.revision, view.counts)
}
fn encoded(value: &impl Serialize) -> String {
    serde_json::to_string(value).unwrap()
}

#[test]
fn coherent_body_and_metadata_survive_motion_damage_and_despawn() {
    let mut game = start();
    let target = game.spawn_enemy(2, false, Vec2::new(1800.0, 0.0), 1);
    let (token, revision, counts) = acquire(&mut game);
    let old = game.co_op_publication(id(100), token, revision).unwrap();
    let header = encoded(&old.capture());
    let rows = encoded(&old.entity_page(0).unwrap());
    let key = format!("{:016x}", target.to_bits().get());
    let body = encoded(&old.body_page(&key, 0).unwrap());
    assert!(counts.entities > 5); // actual enemy head + four attached segments
    game.world
        .get::<&mut PixelBody>(target)
        .unwrap()
        .damage(Vec2::ZERO, 6.0);
    game.world.despawn(target).unwrap();
    game.submit_host_co_op_input(
        id(100),
        1,
        InputFrame {
            fire: true,
            thrust: Some([1000.0, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    for _ in 0..8 {
        game.tick_co_op(id(100)).unwrap();
    }
    let retained = game.co_op_publication(id(100), token, revision).unwrap();
    assert_eq!(encoded(&retained.capture()), header);
    assert_eq!(encoded(&retained.entity_page(0).unwrap()), rows);
    assert_eq!(encoded(&retained.body_page(&key, 0).unwrap()), body);
    assert!(game.co_op_snapshot().unwrap().revision > revision);
    assert!(matches!(
        game.co_op_body_page(id(100), revision, &key, 0),
        Err(Error::StaleSnapshot)
    ));
}

#[test]
fn page_boundaries_exact_cells_and_borrowed_native_bodies_agree() {
    let mut game = start();
    for index in 0..70 {
        game.spawn_enemy(0, false, Vec2::new(1800.0 + index as f32, 0.0), 1);
    }
    let (token, revision, counts) = acquire(&mut game);
    let view = game.co_op_publication(id(100), token, revision).unwrap();
    let mut rows = Vec::new();
    let mut offset = 0;
    loop {
        let page = view.entity_page(offset).unwrap();
        assert!(page.entities.len() <= co_op::ENTITY_PAGE_SIZE);
        rows.extend(page.entities);
        let Some(next) = page.next_offset else { break };
        offset = next;
    }
    assert_eq!(encoded(&rows), encoded(&view.entities()));
    assert_eq!(rows.len(), counts.entities);
    let mut cells = 0;
    for row in &rows {
        let body = view.body(&row.id).unwrap();
        let expected: Vec<_> = body
            .cells_indexed()
            .map(|(i, _, c)| [i as u32, c])
            .collect();
        let mut actual = Vec::new();
        let mut offset = 0;
        loop {
            let page = view.body_page(&row.id, offset).unwrap();
            assert!(encoded(&page).len() < 65536);
            assert!(page.cells.len() <= co_op::CELL_PAGE_SIZE);
            actual.extend(page.cells);
            let Some(next) = page.next_offset else { break };
            offset = next;
        }
        assert_eq!(actual, expected);
        cells += actual.len();
        assert!(
            view.body_page(&row.id, row.occupied)
                .unwrap()
                .cells
                .is_empty()
        );
        assert!(matches!(
            view.body_page(&row.id, row.occupied + 1),
            Err(Error::InvalidOffset)
        ));
    }
    assert_eq!(cells, counts.occupied_cells);
    assert!(view.entity_page(rows.len()).unwrap().entities.is_empty());
    assert!(matches!(
        view.entity_page(rows.len() + 1),
        Err(Error::InvalidOffset)
    ));
    assert!(matches!(
        view.body_page("ffffffffffffffff", 0),
        Err(Error::InvalidEntity)
    ));
}

#[test]
fn membership_end_and_recapture_invalidate_but_invalid_attempts_do_not() {
    let mut game = start();
    let (first, rev, _) = acquire(&mut game);
    assert!(matches!(
        game.co_op_publication(id(101), first, rev),
        Err(Error::StaleSession)
    ));
    assert!(matches!(
        game.co_op_publication(id(100), first, rev + 1),
        Err(Error::StalePublication)
    ));
    let bad = ParticipantSpec {
        id: id(1),
        local_seed: 2,
        identity: None,
    };
    assert!(game.join_co_op(id(100), bad).is_err());
    assert!(game.co_op_publication(id(100), first, rev).is_ok());
    let a = game
        .join_co_op(
            id(100),
            ParticipantSpec {
                id: id(2),
                local_seed: 123,
                identity: None,
            },
        )
        .unwrap();
    assert!(matches!(
        game.co_op_publication(id(100), first, rev),
        Err(Error::NoPublication)
    ));
    let (second, rev2, _) = acquire(&mut game);
    let (third, rev3, _) = acquire(&mut game);
    assert!(third > second && second > first);
    assert!(matches!(
        game.co_op_publication(id(100), second, rev2),
        Err(Error::StalePublication)
    ));
    game.leave_co_op(id(100), a.id, a.epoch).unwrap();
    assert!(matches!(
        game.co_op_publication(id(100), third, rev3),
        Err(Error::NoPublication)
    ));
    game.end_co_op(id(100)).unwrap();
    assert!(matches!(
        game.co_op_publication(id(100), third, rev3),
        Err(Error::NoSession)
    ));
    game.begin_co_op(PROTOCOL, id(100), id(1)).unwrap();
    let (fourth, rev4, _) = acquire(&mut game);
    assert!(fourth > third);
    assert_eq!(rev4, rev);
    assert!(matches!(
        game.co_op_publication(id(100), first, rev),
        Err(Error::StalePublication)
    ));
}

#[test]
fn exact_storage_bound_is_admitted_and_overflow_preserves_previous_cache() {
    let mut game = start();
    let remove: Vec<_> = game
        .world
        .query::<(Entity, ())>()
        .iter()
        .map(|(e, ())| e)
        .filter(|&e| e != game.player)
        .collect();
    for e in remove {
        game.world.despawn(e).unwrap();
    }
    let line = "#".repeat(MAX_BODY_DIMENSION);
    let body = PixelBody::from_ascii(
        &vec![line.as_str(); MAX_BODY_DIMENSION],
        &[('#', rgba(12, 20, 30, 255))],
    );
    *game.world.get::<&mut PixelBody>(game.player).unwrap() = body.clone();
    for _ in 0..3 {
        game.world
            .spawn((Motion::new(Vec2::ZERO, Vec2::ZERO, 0.0), body.clone()));
    }
    let (token, revision, counts) = acquire(&mut game);
    assert_eq!(counts.storage_cells, MAX_STORAGE_CELLS);
    assert_eq!(counts.occupied_cells, MAX_STORAGE_CELLS);
    let extra = game.world.spawn((
        Motion::new(Vec2::ZERO, Vec2::ZERO, 0.0),
        PixelBody::from_ascii(&["#"], &[('#', rgba(1, 2, 3, 255))]),
    ));
    assert!(matches!(
        game.acquire_co_op_publication(id(100)),
        Err(Error::PublicationBound)
    ));
    assert_eq!(
        game.co_op_publication(id(100), token, revision)
            .unwrap()
            .counts()
            .storage_cells,
        MAX_STORAGE_CELLS
    );
    game.world.despawn(extra).unwrap();
    game.publication_generation = u32::MAX;
    assert!(matches!(
        game.acquire_co_op_publication(id(100)),
        Err(Error::Exhausted)
    ));
    assert!(game.co_op_publication(id(100), token, revision).is_ok());
}

#[test]
fn entity_and_dimension_bounds_fail_before_replacing_cache() {
    let mut game = start();
    let (token, revision, _) = acquire(&mut game);
    let huge = PixelBody::from_ascii(
        &[&"#".repeat(MAX_BODY_DIMENSION + 1)],
        &[('#', rgba(2, 3, 4, 255))],
    );
    let e = game
        .world
        .spawn((Motion::new(Vec2::ZERO, Vec2::ZERO, 0.0), huge));
    assert!(matches!(
        game.acquire_co_op_publication(id(100)),
        Err(Error::PublicationBound)
    ));
    game.world.despawn(e).unwrap();
    for _ in 0..MAX_ENTITIES {
        game.world.spawn((
            Motion::new(Vec2::ZERO, Vec2::ZERO, 0.0),
            PixelBody::from_ascii(&["#"], &[('#', rgba(2, 3, 4, 255))]),
        ));
    }
    assert!(matches!(
        game.acquire_co_op_publication(id(100)),
        Err(Error::PublicationBound)
    ));
    assert!(game.co_op_publication(id(100), token, revision).is_ok());
}

#[test]
fn real_eight_pilot_capture_reports_cost_and_retains_solo_profile() {
    let mut game = start();
    let solo = encoded(game.profile());
    for n in 2..=8 {
        game.join_co_op(
            id(100),
            ParticipantSpec {
                id: id(n),
                local_seed: n * 13,
                identity: None,
            },
        )
        .unwrap();
    }
    game.submit_host_co_op_input(
        id(100),
        0,
        InputFrame {
            fire: true,
            thrust: Some([1000.0, 0.0]),
            ..Default::default()
        },
    )
    .unwrap();
    for _ in 0..5 {
        game.tick_co_op(id(100)).unwrap();
    }
    let now = std::time::Instant::now();
    let c = game.acquire_co_op_publication(id(100)).unwrap();
    let elapsed = now.elapsed();
    assert_eq!(c.snapshot.participants.len(), 8);
    assert!(!c.snapshot.progression);
    let bytes = encoded(&c).len();
    assert!(bytes < 65536);
    println!(
        "actual8pilot acquisition_us={} header_bytes={} rows={} occupied={} storage={} entity_pages={} body_pages={}",
        elapsed.as_micros(),
        bytes,
        c.counts.entities,
        c.counts.occupied_cells,
        c.counts.storage_cells,
        c.counts.entity_pages,
        c.counts.body_pages
    );
    let mut samples = Vec::new();
    for _ in 0..32 {
        let time = std::time::Instant::now();
        game.acquire_co_op_publication(id(100)).unwrap();
        samples.push(time.elapsed().as_nanos());
    }
    samples.sort_unstable();
    let view = game.publication.as_ref().unwrap();
    let mut entity_bytes = 0;
    let mut body_bytes = 0;
    let mut maximum_body_bytes = 0;
    for offset in (0..view.entities().len()).step_by(co_op::ENTITY_PAGE_SIZE) {
        entity_bytes += encoded(&view.entity_page(offset).unwrap()).len();
    }
    for row in view.entities() {
        for offset in (0..row.occupied).step_by(co_op::CELL_PAGE_SIZE) {
            let bytes = encoded(&view.body_page(&row.id, offset).unwrap()).len();
            body_bytes += bytes;
            maximum_body_bytes = maximum_body_bytes.max(bytes);
        }
    }
    println!(
        "actual8pilot warm32_acquisition_p50_us={:.3} p95_us={:.3} entity_json_bytes={} initial_body_json_bytes={} max_body_page_bytes={} CPU_ONLY",
        samples[16] as f64 / 1000.0,
        samples[30] as f64 / 1000.0,
        entity_bytes,
        body_bytes,
        maximum_body_bytes
    );
    assert_eq!(encoded(game.profile()), solo);
    game.end_co_op(id(100)).unwrap();
    assert_eq!(encoded(game.profile()), solo);
}

#[test]
fn content_signature_covers_same_count_topology_color_and_dimensions() {
    let a = PixelBody::from_ascii(&["#.", ".#"], &[('#', 0x01020304)]);
    let different_holes = PixelBody::from_ascii(&[".#", "#."], &[('#', 0x01020304)]);
    let different_color = PixelBody::from_ascii(&["#.", ".#"], &[('#', 0x01020305)]);
    let different_size = PixelBody::from_ascii(&["#..#"], &[('#', 0x01020304)]);
    assert_eq!(a.occupied_count(), different_holes.occupied_count());
    assert_eq!(a.occupied_count(), different_color.occupied_count());
    assert_eq!(a.occupied_count(), different_size.occupied_count());
    let signature = body_signature(&a);
    // Independent Python struct.pack/FNV calculation, not a copied body oracle.
    assert_eq!(signature, "613c9e3ec04953ef");
    assert_eq!(signature, body_signature(&a.clone()));
    assert_ne!(signature, body_signature(&different_holes));
    assert_ne!(signature, body_signature(&different_color));
    assert_ne!(signature, body_signature(&different_size));
    println!("signature_golden_2x2_indices0_3_rgba01020304={signature}");
}
