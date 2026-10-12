import assert from "node:assert/strict";
import test from "node:test";
import { normalizeRoomCode } from "../../apps/game/src/coop/room-code.ts";
import {
  grant,
  session,
  Signaling,
} from "../../apps/game/src/coop/signaling.ts";
import { PROTOCOL } from "../../apps/game/src/coop/wire.ts";

const code = "#A7C2-9F34",
  host = "a".repeat(32),
  guest = "b".repeat(32);
const view = (patch = {}) => ({
  id: "c".repeat(32),
  seed: 123,
  protocol: PROTOCOL,
  host_id: host,
  expires_at: 20000,
  state: "open",
  reason: null,
  joining_locked: false,
  room_revision: 0,
  room_code: code,
  ...patch,
});

test("short room code accepts exactly eight hex digits, ASCII separators and raw64 bound", () => {
  for (const value of [
    code,
    "a7c29f34",
    "#a7c2-9F34",
    " \t#A7-C2\r\n9F-34\v\f ",
  ])
    assert.equal(normalizeRoomCode(value), code);
  assert.equal(normalizeRoomCode(code + " ".repeat(54)), code);
  for (const value of [
    undefined,
    null,
    123,
    "",
    "1234567",
    "123456789",
    "A7C29F3G",
    "A7C2#9F34",
    "##A7C29F34",
    "＃A7C29F34",
    "A7C2–9F34",
    "\u00a0" + code,
    "\ufeff" + code,
    code + "\u200b",
    code + " ".repeat(55),
    "a".repeat(64),
  ])
    assert.equal(normalizeRoomCode(value), null);
});

test("session policy requires canonical host code or explicit guest null and finite revision", () => {
  assert.deepEqual(session(view()), view());
  assert.equal(session(view({ room_code: null })).room_code, null);
  for (const patch of [
    { room_code: "a7c29f34" },
    { room_code: undefined },
    { room_revision: -1 },
    { room_revision: 0.5 },
    { room_revision: Infinity },
    { room_revision: Number.MAX_SAFE_INTEGER + 1 },
    { joining_locked: 1 },
  ])
    assert.throws(() => session(view(patch)), { code: "invalid_session" });
  assert.throws(() => session(view({ token: "x" })), {
    code: "invalid_object",
  });
});

test("create/join grants enforce role-specific policy projection and keep bearer separate", () => {
  const make = (role) => ({
    session: view({ room_code: role === "host" ? code : null }),
    participant: {
      id: role === "host" ? host : guest,
      role,
      status: role === "host" ? "active" : "pending",
      token: "d".repeat(64),
    },
    cursor: 0,
    ...(role === "host" ? { invite: code } : {}),
  });
  assert.equal(grant(make("host"), "host").invite, code);
  assert.equal(grant(make("guest"), "guest").invite, undefined);
  const badHost = make("host");
  badHost.invite = "#AAAA-AAAA";
  assert.throws(() => grant(badHost, "host"), { code: "invalid_grant" });
  const badGuest = make("guest");
  badGuest.session.room_code = code;
  assert.throws(() => grant(badGuest, "guest"), { code: "invalid_grant" });
  const leakedGuest = make("guest");
  leakedGuest.invite = code;
  assert.throws(() => grant(leakedGuest, "guest"), { code: "invalid_grant" });
});

test("HTTP room outcomes are status-matched allowlisted codes, never remote messages", async () => {
  const request = async (status, error) => {
    const http = new Signaling(
      "https://rooms.example",
      async () => new Response(JSON.stringify({ error }), { status }),
    );
    try {
      await http.request("/v1/join", null, {}, new AbortController().signal);
    } catch (failure) {
      return failure;
    }
    assert.fail("expected rejection");
  };
  for (const [status, remote, expected] of [
    [401, "invalid_invite", "invalid_invite"],
    [409, "joining_locked", "joining_locked"],
    [409, "session_full", "session_full"],
    [409, "request_unavailable", "request_unavailable"],
    [410, "stale_invite", "stale_invite"],
    [403, "approval_required", "approval_required"],
    [404, "invite_not_found", "invite_not_found"],
    [409, "stale_invite", "signaling_request"],
    [500, "joining_locked", "signaling_request"],
    [410, "PRIVATE-SDP-OR-TOKEN", "session_ended"],
  ]) {
    const failure = await request(status, {
      code: remote,
      message: "PRIVATE-SDP-OR-TOKEN",
    });
    assert.equal(failure.code, expected);
    assert.equal(String(failure).includes("PRIVATE-SDP-OR-TOKEN"), false);
  }
  assert.equal(
    (await request(409, "PRIVATE-SDP-OR-TOKEN")).code,
    "signaling_request",
  );
});
