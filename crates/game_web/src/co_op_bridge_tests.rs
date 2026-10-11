use super::*;
use game_core::{Profile, co_op::PROTOCOL};
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
fn begin() -> Value {
    json!({"op":"begin","protocol":PROTOCOL,"session":id(100),"host":id(1)})
}
fn guest() -> Value {
    json!({"op":"join","session":id(100),"participant":{"id":id(2),"local_seed":123,"identity":null}})
}

#[test]
fn real_admission_input_capture_and_pages_use_authority_without_profiles() {
    let mut game = Game::new(Profile::default());
    let solo = serde_json::to_string(game.profile()).unwrap();
    let response = batch(
        &mut game,
        json!([begin(), guest(), {"op":"host_input","session":id(100),"seq":1,"input":{"fire":true}}, {"op":"input","session":id(100),"peer":id(2),"epoch":2,"seq":1,"input":{"thrust":[1000.0,0.0]}}]),
    );
    assert!(
        response["results"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["ok"] == true)
    );
    assert_eq!(response["results"][1]["value"]["epoch"], 2);
    for _ in 0..4 {
        game.tick_co_op(ParticipantId::parse(&id(100)).unwrap())
            .unwrap();
    }
    let capture = batch(&mut game, json!([{"op":"acquire","session":id(100)}]));
    let c = &capture["results"][0]["value"];
    assert_eq!(c["snapshot"]["participants"].as_array().unwrap().len(), 2);
    assert_eq!(c["snapshot"]["tick"], 4);
    assert_eq!(c["snapshot"]["progression"], false);
    assert_ne!(
        c["snapshot"]["participants"][1]["local_seed"],
        c["snapshot"]["world_seed"]
    );
    assert!(capture.to_string().len() < MAX_VALUE_BYTES);
    game.tick_co_op(ParticipantId::parse(&id(100)).unwrap())
        .unwrap();
    let page = batch(
        &mut game,
        json!([{"op":"entity_page","session":id(100),"acquisition":c["acquisition"],"revision":c["snapshot"]["revision"],"offset":0}]),
    );
    assert_eq!(page["results"][0]["ok"], true);
    assert_eq!(page["results"][0]["value"]["tick"], 4);
    let row = &page["results"][0]["value"]["entities"][0];
    assert_eq!(row["body_signature"].as_str().unwrap().len(), 16);
    let body = batch(
        &mut game,
        json!([{"op":"body_page","session":id(100),"acquisition":c["acquisition"],"revision":c["snapshot"]["revision"],"entity":row["id"],"offset":0}]),
    );
    assert_eq!(body["results"][0]["ok"], true);
    assert!(
        !body["results"][0]["value"]["cells"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), solo);
    for forbidden in [
        "schema_version",
        "currency",
        "campaign",
        "password",
        "token",
    ] {
        assert!(!capture.to_string().contains(forbidden));
    }
}

#[test]
fn malformed_batch_and_bounds_reject_before_any_mutation() {
    let mut game = Game::new(Profile::default());
    for operations in [
        json!([begin(), {"op":"host_input","session":id(100),"seq":1,"input":{"profile":{"currency":[999,999,999]}}}]),
        json!([begin(), {"op":"join","session":id(100),"participant":{"id":id(2),"local_seed":12,"identity":null,"profile":{}}}]),
        json!([begin(), {"op":"join","session":id(100),"participant":{"id":id(2),"local_seed":12,"identity":{"kind":"talo","corporation":"pan","sub_corporation":"renko","hex_id":"0123456789abcdef","currency":[999,999,999]}}}]),
        json!([begin(), {"op":"end","session":"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"}]),
        json!([begin(), {"op":"host_input","session":id(100),"seq":1,"input":{"aim":[1,2,3]}}]),
        json!([begin(), {"op":"advance","session":id(100),"steps":60}]),
    ] {
        let r = batch(&mut game, operations);
        assert_eq!(r["results"][0]["error"], "invalid_request");
        assert_eq!(game.co_op_session(), None);
    }
    let too_many = vec![begin(); MAX_OPERATIONS + 1];
    assert_eq!(
        batch(&mut game, json!(too_many))["results"][0]["error"],
        "request_bound"
    );
    assert_eq!(game.co_op_session(), None);
    let oversized = " ".repeat(MAX_REQUEST_BYTES + 1);
    let r: Value = serde_json::from_str(&dispatch(&mut game, &oversized)).unwrap();
    assert_eq!(r["results"][0]["error"], "request_bound");
}

#[test]
fn semantic_failures_are_finite_per_op_and_never_alias_host_or_stale_membership() {
    let mut game = Game::new(Profile::default());
    let r = batch(
        &mut game,
        json!([begin(), guest(),
            {"op":"input","session":id(100),"peer":id(1),"epoch":1,"seq":100,"input":{"fire":true}},
            {"op":"input","session":id(100),"peer":id(2),"epoch":99,"seq":100,"input":{"fire":true}},
            {"op":"input","session":id(101),"peer":id(2),"epoch":2,"seq":100,"input":{"fire":true}},
            {"op":"host_input","session":id(100),"seq":1,"input":{"destination":[1000001.0,0.0]}},
            {"op":"host_input","session":id(100),"seq":1,"input":{}},
            {"op":"host_input","session":id(100),"seq":1,"input":{}}
        ]),
    );
    let results = r["results"].as_array().unwrap();
    for (index, code) in [
        (2, "host_only"),
        (3, "stale_membership"),
        (4, "stale_session"),
        (5, "invalid_input"),
        (7, "stale_sequence"),
    ] {
        assert_eq!(results[index]["error"], code);
    }
    assert_eq!(results[6]["ok"], true);
}

#[test]
fn membership_clear_restoration_and_new_capture_tokens_are_visible_in_bridge() {
    let mut game = Game::new(Profile::default());
    let before = serde_json::to_string(game.profile()).unwrap();
    batch(&mut game, json!([begin(), guest()]));
    let old = batch(&mut game, json!([{"op":"acquire","session":id(100)}]));
    let old = &old["results"][0]["value"];
    let operations = json!([
        {"op":"leave","session":id(100),"peer":id(2),"epoch":2},
        {"op":"entity_page","session":id(100),"acquisition":old["acquisition"],"revision":old["snapshot"]["revision"],"offset":0},
        {"op":"end","session":id(100)}, begin(), {"op":"acquire","session":id(100)},
        {"op":"entity_page","session":id(100),"acquisition":old["acquisition"],"revision":0,"offset":0}
    ]);
    let r = batch(&mut game, operations);
    assert_eq!(r["results"][1]["error"], "no_publication");
    assert!(r["results"][4]["value"]["acquisition"].as_u64() > old["acquisition"].as_u64());
    assert_eq!(r["results"][5]["error"], "stale_publication");
    batch(&mut game, json!([{"op":"end","session":id(100)}]));
    assert_eq!(serde_json::to_string(game.profile()).unwrap(), before);
}

#[test]
#[ignore = "explicit actual native publication JSON export"]
fn export_actual_publication_bridge_fixtures() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("work/co-op-publication/serialization");
    std::fs::create_dir_all(&directory).unwrap();
    let mut game = Game::new(Profile::default());
    let admissions = batch(&mut game, json!([begin(), guest()]));
    let inputs = batch(
        &mut game,
        json!([
            {"op":"host_input","session":id(100),"seq":1,"input":{"fire":true,"thrust":[1000.0,0.0]}},
            {"op":"input","session":id(100),"peer":id(2),"epoch":2,"seq":1,"input":{"thrust":[0.0,1000.0]}}
        ]),
    );
    for _ in 0..4 {
        game.tick_co_op(ParticipantId::parse(&id(100)).unwrap())
            .unwrap();
    }
    let capture = batch(&mut game, json!([{"op":"acquire","session":id(100)}]));
    let c = &capture["results"][0]["value"];
    let page = batch(
        &mut game,
        json!([{"op":"entity_page","session":id(100),"acquisition":c["acquisition"],"revision":c["snapshot"]["revision"],"offset":0}]),
    );
    let row = &page["results"][0]["value"]["entities"][0];
    let body = batch(
        &mut game,
        json!([{"op":"body_page","session":id(100),"acquisition":c["acquisition"],"revision":c["snapshot"]["revision"],"entity":row["id"],"offset":0}]),
    );
    for (name, result) in [
        ("admissions", admissions),
        ("inputs", inputs),
        ("capture", capture),
        ("entities", page),
        ("body", body),
    ] {
        assert!(
            result["results"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| r["ok"] == true)
        );
        let encoded = serde_json::to_string(&result).unwrap() + "\n";
        println!("{name}.json actual_native_bytes={}", encoded.len());
        std::fs::write(directory.join(format!("{name}.json")), encoded).unwrap();
    }
}
