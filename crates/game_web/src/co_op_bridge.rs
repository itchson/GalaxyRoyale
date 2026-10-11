//! Synchronous bounded JSON batch for the real native host authority.
//! Authorization maps authenticated channels outside this module. No Profile DTO.
use game_core::{
    Game, InputFrame,
    co_op::{Error, OwnedLoadout, ParticipantId, ParticipantSpec},
    identity::{Corporation, KikaiIdentity, KikaiKind},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_REQUEST_BYTES: usize = 65_536;
pub const MAX_OPERATIONS: usize = 16;
pub const MAX_VALUE_BYTES: usize = 65_536;
pub const MAX_RESPONSE_BYTES: usize = 1_100_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    v: u8,
    operations: Vec<Operation>,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Input {
    joystick: Option<[f32; 2]>,
    destination: Option<[f32; 2]>,
    aim: Option<[f32; 2]>,
    thrust: Option<[f32; 2]>,
    fire: bool,
}
impl From<Input> for InputFrame {
    fn from(i: Input) -> Self {
        Self {
            joystick: i.joystick,
            destination: i.destination,
            aim: i.aim,
            thrust: i.thrust,
            fire: i.fire,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    kind: KikaiKind,
    corporation: Corporation,
    sub_corporation: Corporation,
    hex_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Participant {
    id: ParticipantId,
    local_seed: u32,
    identity: Option<Identity>,
    #[serde(default)]
    owned_loadout: Option<OwnedLoadout>,
}
impl From<Participant> for ParticipantSpec {
    fn from(p: Participant) -> Self {
        Self {
            id: p.id,
            local_seed: p.local_seed,
            identity: p.identity.map(|i| KikaiIdentity {
                kind: i.kind,
                corporation: i.corporation,
                sub_corporation: i.sub_corporation,
                hex_id: i.hex_id,
            }),
        }
    }
}
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Operation {
    ExportLoadout,
    Begin {
        protocol: String,
        session: ParticipantId,
        host: ParticipantId,
    },
    Join {
        session: ParticipantId,
        participant: Participant,
    },
    HostInput {
        session: ParticipantId,
        seq: u32,
        input: Input,
    },
    Input {
        session: ParticipantId,
        peer: ParticipantId,
        epoch: u32,
        seq: u32,
        input: Input,
    },
    Leave {
        session: ParticipantId,
        peer: ParticipantId,
        epoch: u32,
    },
    End {
        session: ParticipantId,
    },
    Acquire {
        session: ParticipantId,
    },
    EntityPage {
        session: ParticipantId,
        acquisition: u32,
        revision: u32,
        offset: usize,
    },
    BodyPage {
        session: ParticipantId,
        acquisition: u32,
        revision: u32,
        entity: String,
        offset: usize,
    },
    Notices {
        session: ParticipantId,
    },
}
#[derive(Serialize)]
#[serde(untagged)]
enum OperationResult {
    Success { ok: bool, value: Value },
    Failure { ok: bool, error: String },
}
fn failure(code: &str) -> OperationResult {
    OperationResult::Failure {
        ok: false,
        error: code.into(),
    }
}
fn value(value: impl Serialize) -> Result<Value, Error> {
    // The native structures contain only bounded finite world data.
    serde_json::to_value(value).map_err(|_| Error::PublicationBound)
}
fn execute(game: &mut Game, op: Operation) -> Result<Value, Error> {
    match op {
        Operation::ExportLoadout => {
            if game.co_op_session().is_some() {
                return Err(Error::ActiveSession);
            }
            value(game.co_op_owned_loadout()?)
        }
        Operation::Begin {
            protocol,
            session,
            host,
        } => value(game.begin_co_op(&protocol, session, host)?),
        Operation::Join {
            session,
            mut participant,
        } => {
            let owned = participant.owned_loadout.take();
            let spec = participant.into();
            value(match owned {
                Some(loadout) => game.join_co_op_owned(session, spec, loadout)?,
                None => game.join_co_op(session, spec)?,
            })
        }
        Operation::HostInput {
            session,
            seq,
            input,
        } => value(game.submit_host_co_op_input(session, seq, input.into())?),
        Operation::Input {
            session,
            peer,
            epoch,
            seq,
            input,
        } => value(game.submit_guest_co_op_input(session, peer, epoch, seq, input.into())?),
        Operation::Leave {
            session,
            peer,
            epoch,
        } => value(game.leave_co_op(session, peer, epoch)?),
        Operation::End { session } => value(game.end_co_op(session)?),
        Operation::Acquire { session } => value(game.acquire_co_op_publication(session)?),
        Operation::EntityPage {
            session,
            acquisition,
            revision,
            offset,
        } => value(game.published_co_op_entity_page(session, acquisition, revision, offset)?),
        Operation::BodyPage {
            session,
            acquisition,
            revision,
            entity,
            offset,
        } => value(game.published_co_op_body_page(
            session,
            acquisition,
            revision,
            &entity,
            offset,
        )?),
        Operation::Notices { session } => value(game.take_co_op_destructions(session)?),
    }
}
fn response(results: &[OperationResult]) -> String {
    serde_json::to_string(&serde_json::json!({ "v": 1, "results": results })).unwrap()
}
/// All JSON/shape checks happen before executing; semantic failures are per-op.
/// There is one native fixed-step clock in Galaxy.step, not a batch tick command.
pub fn dispatch(game: &mut Game, json: &str) -> String {
    if json.len() > MAX_REQUEST_BYTES {
        return response(&[failure("request_bound")]);
    }
    let Ok(request) = serde_json::from_str::<Request>(json) else {
        return response(&[failure("invalid_request")]);
    };
    if request.v != 1 || request.operations.len() > MAX_OPERATIONS {
        return response(&[failure("request_bound")]);
    }
    let results: Vec<_> = request
        .operations
        .into_iter()
        .map(|op| match execute(game, op) {
            Ok(value)
                if serde_json::to_vec(&value).is_ok_and(|bytes| bytes.len() <= MAX_VALUE_BYTES) =>
            {
                OperationResult::Success { ok: true, value }
            }
            Ok(_) => failure("output_bound"),
            Err(error) => {
                let code = serde_json::to_value(error).unwrap();
                failure(code.as_str().unwrap())
            }
        })
        .collect();
    let output = response(&results);
    // 16 admitted values of64KiB plus small finite wrappers cannot exceed this.
    if output.len() > MAX_RESPONSE_BYTES {
        response(&[failure("output_bound")])
    } else {
        output
    }
}

#[cfg(test)]
#[path = "co_op_bridge_tests.rs"]
mod tests;
