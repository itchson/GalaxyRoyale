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
    let result = if json.len() > 65_536 {
        Err("request_bound")
    } else {
        serde_json::from_str::<Request>(json)
            .map_err(|_| "invalid_request")
            .and_then(|request| {
                if request.version() != 1 {
                    Err("invalid_version")
                } else {
                    execute(slot, game, request)
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
}
