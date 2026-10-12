//! Bounded native guest-view boundary. It never receives a Profile or ticks Game.
use engine_core::Vec2;
use game_core::{
    Game,
    co_op::ParticipantId,
    co_op_replica::{BodyPage, EntityPage, Header, Replica},
};
use serde::Deserialize;

pub struct GuestView {
    pub replica: Replica,
    pub peer: ParticipantId,
    pub camera: Vec2,
    pub age: f32,
}

#[derive(Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
enum Request {
    #[serde(rename = "replica_begin")]
    Begin {
        v: u8,
        session: ParticipantId,
        peer: ParticipantId,
        epoch: u32,
        world_seed: u32,
    },
    #[serde(rename = "replica_stage")]
    Stage { v: u8, header: Header },
    #[serde(rename = "replica_entities")]
    Entities {
        v: u8,
        acquisition: u32,
        page: EntityPage,
    },
    #[serde(rename = "replica_body")]
    Body {
        v: u8,
        acquisition: u32,
        page: BodyPage,
    },
    #[serde(rename = "replica_wake")]
    Wake {
        v: u8,
        acquisition: u32,
        page: game_core::co_op_wake_fx::WakePage,
    },
    #[serde(rename = "replica_commit")]
    Commit {
        v: u8,
        acquisition: u32,
        revision: u32,
    },
    #[serde(rename = "replica_end")]
    End { v: u8, session: ParticipantId },
}
impl Request {
    fn version(&self) -> u8 {
        match self {
            Self::Begin { v, .. }
            | Self::Stage { v, .. }
            | Self::Entities { v, .. }
            | Self::Body { v, .. }
            | Self::Wake { v, .. }
            | Self::Commit { v, .. }
            | Self::End { v, .. } => *v,
        }
    }
}
fn execute(
    slot: &mut Option<GuestView>,
    game: &Game,
    request: Request,
) -> Result<(), &'static str> {
    match request {
        Request::Begin {
            session,
            peer,
            epoch,
            world_seed,
            ..
        } => {
            if slot.is_some() || game.co_op_session().is_some() {
                return Err("active_session");
            }
            if !game.hud().dock {
                return Err("not_at_dock");
            }
            let replica = Replica::new(session, peer, epoch, world_seed)?;
            *slot = Some(GuestView {
                replica,
                peer,
                camera: Vec2::ZERO,
                age: 0.0,
            });
            Ok(())
        }
        Request::Stage { header, .. } => slot.as_mut().ok_or("no_replica")?.replica.stage(header),
        Request::Entities {
            acquisition, page, ..
        } => slot
            .as_mut()
            .ok_or("no_replica")?
            .replica
            .entities(acquisition, page),
        Request::Body {
            acquisition, page, ..
        } => slot
            .as_mut()
            .ok_or("no_replica")?
            .replica
            .body(acquisition, page),
        Request::Wake {
            acquisition, page, ..
        } => slot
            .as_mut()
            .ok_or("no_replica")?
            .replica
            .wake(acquisition, page),
        Request::Commit {
            acquisition,
            revision,
            ..
        } => {
            let view = slot.as_mut().ok_or("no_replica")?;
            view.replica.commit(acquisition, revision)?;
            view.age = 0.0;
            Ok(())
        }
        Request::End { session, .. } => {
            slot.as_mut().ok_or("no_replica")?.replica.end(session)?;
            *slot = None;
            Ok(())
        }
    }
}
pub fn dispatch(slot: &mut Option<GuestView>, game: &Game, json: &str) -> String {
    dispatch_at(slot, game, json, 0.0)
}
/// Length of the actual top-level page JSON, including its internal whitespace.
/// Called only after strict serde request validation. Parsing raw spans avoids
/// admitting a >32 KiB numeric spelling that normalizes to a short f64.
fn raw_page_bytes(json: &str) -> Option<usize> {
    let bytes = json.as_bytes();
    let mut i = 0;
    while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
        i += 1;
    }
    i += 1; // validated root object
    loop {
        while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        if bytes.get(i) == Some(&b'}') {
            return None;
        }
        let key_start = i;
        i += 1;
        while let Some(&b) = bytes.get(i) {
            i += 1;
            if b == b'\\' {
                i += 1;
            } else if b == b'"' {
                break;
            }
        }
        let key: String = serde_json::from_str(json.get(key_start..i)?).ok()?;
        while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        i += 1; // colon
        while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        let start = i;
        let mut depth = 0usize;
        let mut string = false;
        while let Some(&b) = bytes.get(i) {
            if string {
                i += 1;
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    string = false;
                }
                continue;
            }
            match b {
                b'"' => string = true,
                b'{' | b'[' => depth += 1,
                b'}' | b']' if depth > 0 => depth -= 1,
                b',' | b'}' if depth == 0 => break,
                _ => {}
            }
            i += 1;
        }
        if key == "page" {
            return Some(json.get(start..i)?.trim_end().len());
        }
        if bytes.get(i) != Some(&b',') {
            return None;
        }
        i += 1;
    }
}
/// Browser monotonic receipt time, including paused/hidden body assembly. The
/// bridge does not use the clamped frame delta or reset interpolation age.
pub fn dispatch_at(slot: &mut Option<GuestView>, game: &Game, json: &str, seconds: f64) -> String {
    let result = if !seconds.is_finite() || seconds < 0.0 {
        Err("invalid_presentation_clock")
    } else if json.len() > 65_536 {
        Err("request_bound")
    } else {
        serde_json::from_str::<Request>(json)
            .map_err(|_| "invalid_request")
            .and_then(|request| {
                if request.version() != 1 {
                    Err("invalid_version")
                } else if matches!(request, Request::Wake { .. })
                    && raw_page_bytes(json)
                        .is_none_or(|n| n > game_core::co_op_wake_fx::MAX_PAGE_BYTES)
                {
                    Err("wake_page_bound")
                } else {
                    if let Some(view) = slot {
                        view.replica.advance_presentation(seconds)?;
                    }
                    execute(slot, game, request)?;
                    if let Some(view) = slot {
                        view.replica.advance_presentation(seconds)?;
                    }
                    Ok(())
                }
            })
    };
    match result {
        Ok(()) => "{\"v\":1,\"ok\":true,\"value\":null}".into(),
        Err(error) => serde_json::json!({"v":1,"ok":false,"error":error}).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::{Profile, co_op::PROTOCOL};
    use serde_json::{Value, json};
    fn id(n: u32) -> String {
        format!("{n:032x}")
    }
    fn request(slot: &mut Option<GuestView>, game: &Game, value: Value) -> Value {
        serde_json::from_str(&dispatch(slot, game, &value.to_string())).unwrap()
    }
    fn begin() -> Value {
        json!({"v":1,"op":"replica_begin","session":id(1),"peer":id(2),"epoch":2,"world_seed":123})
    }
    #[test]
    fn actual_raw_wake_page_bound_precedes_clock_or_pending_mutation() {
        let game = Game::new(Profile::default());
        let mut slot = None;
        let page = json!({"session":id(1),"tick":0,"revision":0,"points":[]});
        let compact = json!({"v":1,"op":"replica_wake","acquisition":1,"page":page}).to_string();
        assert_eq!(raw_page_bytes(&compact), Some(page.to_string().len()));
        let padded = compact.replace(
            "\"points\":[]",
            &format!("\"points\":[{}]", " ".repeat(32_768)),
        );
        assert!(padded.len() < 65_536);
        assert_eq!(
            serde_json::from_str::<Value>(&dispatch_at(&mut slot, &game, &padded, 2.0)).unwrap()["error"],
            "wake_page_bound"
        );
        assert!(slot.is_none());
        // Escaped property spelling is valid JSON and has the same raw gate.
        let escaped = padded.replace("\"page\":", "\"pa\\u0067e\":");
        assert_eq!(
            serde_json::from_str::<Value>(&dispatch_at(&mut slot, &game, &escaped, 2.0)).unwrap()["error"],
            "wake_page_bound"
        );
        let inner = " {\"nested\":[{\"text\":\"} , \\\" page\"}],\"number\":0.00000001} ";
        let raw = format!("{{\"first\":\"page,}}\",\"page\":{inner},\"last\":5}}");
        assert_eq!(raw_page_bytes(&raw), Some(inner.trim().len()));
    }
    #[test]
    #[ignore = "explicit actual exported core-Value contracts; requires GALAXY_WAKE_FIXTURES"]
    fn actual_maximum_value_wrappers_pass_native_stage_and_wake_boundaries() {
        let directory = std::path::PathBuf::from(
            std::env::var("GALAXY_WAKE_FIXTURES").expect("explicit fixture directory"),
        );
        let read =
            |name: &str| std::fs::read_to_string(directory.join(format!("{name}.json"))).unwrap();
        let header_text = read("maximum-native-stage");
        let wake_text = read("maximum-native-wake");
        assert!(header_text.trim().len() < 65_536 && wake_text.trim().len() < 65_536);
        assert!(raw_page_bytes(&wake_text).unwrap() <= game_core::co_op_wake_fx::MAX_PAGE_BYTES);
        let game = Game::new(Profile::default());
        let before = serde_json::to_string(game.profile()).unwrap();
        let mut slot = None;
        let start = json!({"v":1,"op":"replica_begin","session":id(100),"peer":id(2),"epoch":2,"world_seed":812345});
        assert_eq!(request(&mut slot, &game, start)["ok"], true);
        let response = |slot: &mut Option<GuestView>, text: &str| {
            serde_json::from_str::<Value>(&dispatch_at(slot, &game, text, 5.0)).unwrap()
        };
        assert_eq!(response(&mut slot, &header_text)["ok"], true);
        assert_eq!(response(&mut slot, &wake_text)["ok"], true);
        assert_eq!(
            response(&mut slot, &wake_text)["error"],
            "invalid_wake_page"
        );
        assert!(slot.as_ref().unwrap().replica.scene().is_none());
        let input: Value = serde_json::from_str(&wake_text).unwrap();
        let header: Value = serde_json::from_str(&header_text).unwrap();
        assert_eq!(response(&mut slot, &json!({"v":1,"op":"replica_commit","acquisition":input["acquisition"],"revision":header["header"]["snapshot"]["revision"]}).to_string())["error"], "incomplete_entities");
        assert_eq!(serde_json::to_string(game.profile()).unwrap(), before);
        println!(
            "actual_native_boundary_stage_bytes={} wake_bytes={} wake_raw_payload_bytes={} stage_and_wake_admitted=true duplicate_rejected=true scene_before_complete_transfer=none",
            header_text.trim().len(),
            wake_text.trim().len(),
            raw_page_bytes(&wake_text).unwrap()
        );
    }
    #[test]
    fn malformed_bound_version_and_unknown_fields_do_not_create_or_replace_a_view() {
        let game = Game::new(Profile::default());
        let mut slot = None;
        assert!(dispatch(&mut slot, &game, &"x".repeat(65_537)).contains("request_bound"));
        let mut input = begin();
        input["v"] = json!(2);
        assert_eq!(request(&mut slot, &game, input)["error"], "invalid_version");
        let mut input = begin();
        input["profile"] = json!({"currency":[999,999,999]});
        assert_eq!(request(&mut slot, &game, input)["error"], "invalid_request");
        assert!(slot.is_none());
        assert_eq!(request(&mut slot, &game, begin())["ok"], true);
        assert_eq!(
            request(&mut slot, &game, begin())["error"],
            "active_session"
        );
        assert!(slot.is_some());
    }
    #[test]
    fn wrong_session_end_retains_ownership_and_valid_end_leaves_solo_profile_exact() {
        let game = Game::new(Profile::default());
        let mut slot = None;
        let before = serde_json::to_string(game.profile()).unwrap();
        assert_eq!(request(&mut slot, &game, begin())["ok"], true);
        assert_eq!(
            request(
                &mut slot,
                &game,
                json!({"v":1,"op":"replica_end","session":id(9)})
            )["ok"],
            false
        );
        assert!(slot.is_some());
        assert_eq!(
            request(
                &mut slot,
                &game,
                json!({"v":1,"op":"replica_end","session":id(1)})
            )["ok"],
            true
        );
        assert!(slot.is_none());
        assert_eq!(before, serde_json::to_string(game.profile()).unwrap());
    }
    #[test]
    fn a_host_cannot_simultaneously_replace_its_authority_with_a_guest_view() {
        let mut game = Game::new(Profile::default());
        let mut slot = None;
        game.begin_co_op(
            PROTOCOL,
            ParticipantId::parse(&id(3)).unwrap(),
            ParticipantId::parse(&id(4)).unwrap(),
        )
        .unwrap();
        assert_eq!(
            request(&mut slot, &game, begin())["error"],
            "active_session"
        );
        assert!(slot.is_none());
    }
    #[test]
    fn receipt_clock_rejects_invalid_or_regressing_end_before_disposing_a_view() {
        let game = Game::new(Profile::default());
        let mut slot = None;
        assert!(dispatch_at(&mut slot, &game, &begin().to_string(), 100.0).contains("\"ok\":true"));
        let end = json!({"v":1,"op":"replica_end","session":id(1)}).to_string();
        for time in [99.0, f64::NAN, f64::INFINITY, -1.0] {
            assert!(
                dispatch_at(&mut slot, &game, &end, time).contains("invalid_presentation_clock")
            );
            assert!(slot.is_some());
        }
        assert!(dispatch_at(&mut slot, &game, &end, 101.0).contains("\"ok\":true"));
        assert!(slot.is_none());
    }
}
