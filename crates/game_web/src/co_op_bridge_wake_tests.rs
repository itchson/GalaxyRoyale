//! Real host JSON operation and native guest page boundaries.
use super::*;
use game_core::co_op::PROTOCOL;
use serde_json::json;

fn id(n: u32) -> String {
    format!("{n:032x}")
}
fn batch(game: &mut Game, operations: Value) -> Value {
    serde_json::from_str(&dispatch(
        game,
        &json!({"v":1,"operations":operations}).to_string(),
    ))
    .unwrap()
}
#[test]
fn actual_host_wake_operation_is_frozen_non_draining_and_has_strict_shape() {
    let mut p =
        game_core::identity::create_profile("talo", "hugo", "pan", "0123456789abcdef").unwrap();
    p.campaign.abilities[5] = game_core::campaign::AbilityState {
        blueprint: true,
        rank: 1,
    };
    p.campaign.loadouts[0].slots[2] = Some(5);
    p.boss_generations[2] = [16, 8, 4, 2, 1];
    let mut game = Game::new(p);
    let before = serde_json::to_string(game.profile()).unwrap();
    assert_eq!(
        batch(
            &mut game,
            json!([{"op":"begin","protocol":PROTOCOL,"session":id(100),"host":id(1)}])
        )["results"][0]["ok"],
        true
    );
    let session = ParticipantId::parse(&id(100)).unwrap();
    for seq in 1..=360 {
        game.submit_host_co_op_input(
            session,
            seq,
            InputFrame {
                joystick: Some([1.0, 0.0]),
                ..Default::default()
            },
        )
        .unwrap();
        game.tick_co_op(session).unwrap();
    }
    let capture =
        batch(&mut game, json!([{"op":"acquire","session":id(100)}]))["results"][0]["value"]
            .clone();
    let op = json!({"op":"wake_page","session":id(100),"acquisition":capture["acquisition"],"revision":capture["snapshot"]["revision"]});
    let result = batch(&mut game, json!([op.clone(), op.clone()]));
    assert_eq!(result["results"][0]["ok"], true);
    let page = result["results"][0]["value"].clone();
    assert_eq!(page, result["results"][1]["value"]);
    assert!(!page["points"].as_array().unwrap().is_empty());
    assert_eq!(
        page["points"].as_array().unwrap().len(),
        capture["snapshot"]["wake_fx"]["points"].as_u64().unwrap() as usize
    );
    assert!(page.get("offset").is_none() && page.get("next_offset").is_none());
    game.tick_co_op(session).unwrap();
    assert_eq!(
        batch(&mut game, json!([op.clone()]))["results"][0]["value"],
        page
    );
    let mut extra = op.clone();
    extra["offset"] = json!(0);
    assert_eq!(
        batch(&mut game, json!([extra]))["results"][0]["error"],
        "invalid_request"
    );
    let mut stale = op;
    stale["acquisition"] = json!(0);
    assert_eq!(
        batch(&mut game, json!([stale]))["results"][0]["error"],
        "stale_publication"
    );
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), before);
    println!(
        "actual_host_bridge_wake_points={} actual_value_bytes={}",
        page["points"].as_array().unwrap().len(),
        page.to_string().len()
    );
    if let Ok(directory) = std::env::var("GALAXY_WAKE_BRIDGE_FIXTURES") {
        std::fs::create_dir_all(&directory).unwrap();
        for (name, value) in [
            ("actual-host-header", capture),
            ("actual-host-page", page),
            ("actual-host-batch-response", result),
        ] {
            std::fs::write(
                std::path::Path::new(&directory).join(format!("{name}.json")),
                value.to_string() + "\n",
            )
            .unwrap();
        }
    }
}
