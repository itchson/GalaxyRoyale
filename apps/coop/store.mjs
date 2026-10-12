import { createHash, randomBytes } from "node:crypto";
import { EventEmitter } from "node:events";

export const PROTOCOL = "galaxy-coop-1";
export const LIMITS = Object.freeze({
  players: 8,
  sessions: 64,
  retainedParticipants: 64,
  sessionMs: 3_600_000,
  inviteMs: 600_000,
  pendingMs: 20_000,
  codesPerSession: 64,
  codeAttempts: 32,
  adminPerMinute: 30,
  hostMs: 30_000,
  guestMs: 45_000,
  retentionMs: 60_000,
  bodyBytes: 24_576,
  sdpBytes: 16_384,
  iceBytes: 2_048,
  queueEvents: 64,
  queueBytes: 262_144,
  signalPerMinute: 60,
  sessionSignalsPerMinute: 240,
  createPerMinute: 12,
  joinPerMinute: 120,
  clientCreatePerMinute: 2,
  clientJoinPerMinute: 20,
  clientSessions: 4,
  rateClients: 256,
  pollMs: 20_000,
});
const hexId = (s) => typeof s === "string" && /^[0-9a-f]{32}$/.test(s);
const hash = (s) => createHash("sha256").update(s).digest("hex");
const random = (n) => randomBytes(n).toString("hex");

/** Short lookup identifier, not an administration capability. */
export function normalizeRoomCode(input) {
  if (typeof input !== "string" || input.length > 64) return null;
  const compact = input.replace(/[\x09-\x0d\x20-]/g, "").replace(/^#/, "");
  return /^[0-9a-fA-F]{8}$/.test(compact) ? compact.toUpperCase() : null;
}
const displayCode = (code) => `#${code.slice(0, 4)}-${code.slice(4)}`;

export class CoopError extends Error {
  constructor(status, code, message) {
    super(message);
    this.status = status;
    this.code = code;
  }
}
function fail(status, code, message) {
  throw new CoopError(status, code, message);
}
export function object(body, allowed) {
  if (
    !body ||
    typeof body !== "object" ||
    Array.isArray(body) ||
    Object.keys(body).some((k) => !allowed.includes(k))
  )
    fail(400, "invalid_body", "Unexpected request fields.");
  return body;
}
function metadata(body) {
  object(body, ["seed", "protocol"]);
  if (
    !Number.isInteger(body.seed) ||
    body.seed < 0 ||
    body.seed > 0xffff_ffff ||
    body.protocol !== PROTOCOL
  )
    fail(
      400,
      "invalid_metadata",
      "Use a uint32 seed and the supported protocol.",
    );
}
function rate(bucket, now, maximum) {
  if (now >= bucket.until) {
    bucket.count = 0;
    bucket.until = now + 60_000;
  }
  if (bucket.count >= maximum)
    fail(429, "rate_limit", "Request rate limit reached.");
  bucket.count++;
}

/** Ephemeral synchronous transactions. No disk, profiles, network or timers. */
export class SessionStore extends EventEmitter {
  #key = randomBytes(32);
  #sessions = new Map();
  #credentials = new Map();
  #codes = new Map();
  #clients = new Map();
  #globalRates = {
    create: { count: 0, until: 0 },
    join: { count: 0, until: 0 },
  };
  #clock;
  #codeBytes;
  #lastTime = 0;
  #closed = false;
  constructor({
    now = Date.now,
    limits = {},
    codeBytes = () => randomBytes(4),
  } = {}) {
    super();
    this.limits = Object.freeze({ ...LIMITS, ...limits });
    for (const [name, value] of Object.entries(this.limits)) {
      if (
        !Object.hasOwn(LIMITS, name) ||
        !Number.isSafeInteger(value) ||
        value <= 0 ||
        value > LIMITS[name]
      )
        throw new Error("Limits may only narrow the documented bounds.");
    }
    if (this.limits.players > 8) throw new Error("At most eight participants.");
    if (this.limits.queueBytes < 256)
      throw new Error("Reserve at least 256 bytes for a terminal notice.");
    this.#clock = now;
    if (typeof codeBytes !== "function")
      throw new Error("Invalid code generator.");
    this.#codeBytes = codeBytes;
    this.setMaxListeners(this.limits.sessions * this.limits.players + 1);
  }
  #now() {
    const value = this.#clock();
    if (!Number.isSafeInteger(value) || value < 0)
      throw new Error("Invalid service clock.");
    return (this.#lastTime = Math.max(this.#lastTime, value));
  }
  /** Salted remote-address digest, never embedded in an invite or exposed. */
  clientKey(address) {
    return hash(Buffer.concat([this.#key, Buffer.from(address)]));
  }
  #open() {
    if (this.#closed)
      fail(503, "service_closed", "Signaling service is shutting down.");
  }
  admitAnonymous(operation, client) {
    this.#open();
    const now = this.#now();
    this.sweep();
    if (
      !["create", "join"].includes(operation) ||
      !/^[0-9a-f]{64}$/.test(client)
    )
      throw new Error("Invalid admission caller.");
    rate(
      this.#globalRates[operation],
      now,
      this.limits[`${operation}PerMinute`],
    );
    let rates = this.#clients.get(client);
    if (!rates) {
      if (this.#clients.size >= this.limits.rateClients)
        fail(429, "rate_clients", "Admission is temporarily busy.");
      rates = { create: { count: 0, until: 0 }, join: { count: 0, until: 0 } };
      this.#clients.set(client, rates);
    }
    rate(
      rates[operation],
      now,
      this.limits[
        operation === "create" ? "clientCreatePerMinute" : "clientJoinPerMinute"
      ],
    );
  }
  admitReplacement(token) {
    const { session, member } = this.#auth(token);
    if (member.role !== "host")
      fail(403, "host_only", "Only the host can replace a session.");
    // Preserve the original owner's bucket across new sessions/credentials.
    this.admitAnonymous("create", session.owner);
  }
  admitAdministration(token) {
    const { session } = this.#host(token);
    rate(session.adminRate, this.#now(), this.limits.adminPerMinute);
  }
  #participant(session, role, now) {
    const token = random(32);
    const member = {
      id: random(16),
      role,
      tokenHash: hash(token),
      status: role === "host" ? "active" : "pending",
      approved: role === "host",
      approvedAt: null,
      lastSeen: now,
      joined: now,
      ended: null,
      ack: 0,
      queue: [],
      queueBytes: 0,
      rate: { count: 0, until: 0 },
      offer: false,
    };
    session.members.set(member.id, member);
    this.#credentials.set(member.tokenHash, { session, member });
    return { member, token };
  }
  #sessionView(s, host = false) {
    return {
      id: s.id,
      seed: s.seed,
      protocol: s.protocol,
      host_id: s.host,
      expires_at: s.expires,
      state: s.state,
      reason: s.reason,
      joining_locked: s.joiningLocked,
      room_revision: s.roomRevision,
      room_code: host ? displayCode(s.code) : null,
    };
  }
  #participantView(p) {
    return { id: p.id, role: p.role, status: p.status };
  }
  #newCode() {
    for (let attempt = 0; attempt < this.limits.codeAttempts; attempt++) {
      const bytes = this.#codeBytes();
      if (!Buffer.isBuffer(bytes) || bytes.length !== 4)
        throw new Error("Code generator must return four bytes.");
      const code = bytes.toString("hex").toUpperCase();
      if (!this.#codes.has(code)) return code;
    }
    fail(
      503,
      "code_capacity",
      "A room code could not be allocated. Try again.",
    );
  }
  #decode(invite) {
    const code = normalizeRoomCode(invite);
    const session = code && this.#codes.get(code);
    if (!session)
      fail(401, "invalid_invite", "Room code is invalid or unavailable.");
    if (session.state !== "open")
      fail(410, "session_ended", "Room is no longer open.");
    if (session.code !== code)
      fail(410, "stale_invite", "Room code was replaced.");
    if (this.#now() >= session.inviteExpires)
      fail(
        410,
        "invite_expired",
        "Room code has expired; ask the host to replace it.",
      );
    return session;
  }
  create(body, client, replacingToken = null) {
    this.#open();
    this.sweep();
    metadata(body);
    const previous = replacingToken ? this.#auth(replacingToken) : null;
    if (previous && previous.member.role !== "host")
      fail(403, "host_only", "Only the host can replace a session.");
    if (this.#sessions.size >= this.limits.sessions)
      fail(429, "session_quota", "Session capacity is full.");
    const owner = previous ? previous.session.owner : client;
    const owned = [...this.#sessions.values()].filter(
      (s) => s.owner === owner && s.state === "open",
    ).length;
    if (!previous && owned >= this.limits.clientSessions)
      fail(429, "client_sessions", "Host session limit reached.");
    const now = this.#now();
    const code = this.#newCode(); // Complete admission before allocating any credentials.
    const s = {
      id: random(16),
      code,
      codes: new Set([code]),
      joiningLocked: false,
      roomRevision: 0,
      owner,
      seed: body.seed,
      protocol: body.protocol,
      expires: now + this.limits.sessionMs,
      inviteExpires:
        now + Math.min(this.limits.inviteMs, this.limits.sessionMs),
      host: null,
      state: "open",
      reason: null,
      ended: null,
      seq: 0,
      members: new Map(),
      rate: { count: 0, until: 0 },
      adminRate: { count: 0, until: 0 },
    };
    const { member, token } = this.#participant(s, "host", now);
    s.host = member.id;
    if (previous) this.#end(previous.session, "replaced", now);
    this.#sessions.set(s.id, s);
    this.#codes.set(code, s);
    return {
      session: this.#sessionView(s, true),
      invite: displayCode(s.code),
      participant: { ...this.#participantView(member), token },
      cursor: 0,
    };
  }
  join(body) {
    this.#open();
    this.sweep();
    object(body, ["invite"]);
    const s = this.#decode(body.invite);
    if (s.joiningLocked)
      fail(409, "joining_locked", "The host has locked joining.");
    if (
      [...s.members.values()].filter((p) => p.ended === null).length >=
      this.limits.players
    )
      fail(
        409,
        "session_full",
        "Eight players including the host are already reserved.",
      );
    if (s.members.size >= this.limits.retainedParticipants)
      fail(429, "participant_quota", "Recent participant capacity is full.");
    const host = s.members.get(s.host);
    this.#room(host, {
      type: "join-request",
      peer: { id: "0".repeat(32), role: "guest", status: "pending" },
      expires_at: this.#now() + this.limits.pendingMs,
      room_revision: s.roomRevision,
    });
    const { member, token } = this.#participant(s, "guest", this.#now());
    this.#push(s, host, {
      type: "join-request",
      peer: this.#participantView(member),
      expires_at: member.joined + this.limits.pendingMs,
      room_revision: s.roomRevision,
    });
    return {
      session: this.#sessionView(s),
      participant: { ...this.#participantView(member), token },
      cursor: s.seq,
    };
  }
  #auth(token, reading = false) {
    this.sweep();
    if (!reading) this.#open();
    if (typeof token !== "string" || !/^[0-9a-f]{64}$/.test(token))
      fail(401, "unauthorized", "A participant bearer credential is required.");
    const found = this.#credentials.get(hash(token));
    if (!found) fail(401, "unauthorized", "Participant credential is invalid.");
    if (!reading && found.session.state !== "open")
      fail(410, "session_ended", "Session is ended.");
    if (!reading && found.member.ended !== null)
      fail(410, "participant_left", "Participant is no longer admitted.");
    return found;
  }
  #host(token) {
    const found = this.#auth(token);
    if (found.member.role !== "host")
      fail(403, "host_only", "Only the host can administer this room.");
    return found;
  }
  #pending(s) {
    return [...s.members.values()].filter(
      (p) => p.role === "guest" && !p.approved && p.ended === null,
    );
  }
  #cancelRoom(s, pending, reason) {
    const host = s.members.get(s.host);
    const events = pending.map((p) => ({
      type: "peer-left",
      peer_id: p.id,
      reason,
    }));
    const bytes = events.reduce(
      (sum, event) => sum + Buffer.byteLength(JSON.stringify(event)) + 32,
      0,
    );
    if (
      host.queue.length + events.length > this.limits.queueEvents ||
      host.queueBytes + bytes > this.limits.queueBytes
    )
      fail(
        429,
        "queue_full",
        "Host must acknowledge its pending events first.",
      );
  }
  #cancelPending(s, pending, reason, now) {
    for (const p of pending) {
      p.status = "left";
      p.ended = now;
      this.#push(s, p, { type: "participant-left", reason }, true, false);
      this.#push(
        s,
        s.members.get(s.host),
        { type: "peer-left", peer_id: p.id, reason },
        false,
        false,
      );
    }
  }
  rotateCode(token, body = {}) {
    object(body, []);
    const { session: s } = this.#host(token);
    if (s.codes.size >= this.limits.codesPerSession)
      fail(
        429,
        "code_quota",
        "This room has reached its code replacement limit.",
      );
    const pending = this.#pending(s);
    this.#cancelRoom(s, pending, "code_replaced");
    const code = this.#newCode();
    const now = this.#now();
    s.code = code;
    s.roomRevision++;
    s.codes.add(code);
    this.#codes.set(code, s);
    s.inviteExpires = Math.min(s.expires, now + this.limits.inviteMs);
    this.#cancelPending(s, pending, "code_replaced", now);
    this.emit("changed", s.id);
    return { session: this.#sessionView(s, true), invite: displayCode(code) };
  }
  joining(token, body) {
    object(body, ["locked"]);
    if (typeof body.locked !== "boolean")
      fail(400, "invalid_locked", "Locked must be a boolean.");
    const { session: s } = this.#host(token);
    const pending = body.locked ? this.#pending(s) : [];
    this.#cancelRoom(s, pending, "joining_locked");
    if (s.joiningLocked !== body.locked) s.roomRevision++;
    s.joiningLocked = body.locked;
    this.#cancelPending(s, pending, "joining_locked", this.#now());
    this.emit("changed", s.id);
    return { session: this.#sessionView(s, true) };
  }
  approval(token, body) {
    object(body, ["peer", "accept"]);
    if (!hexId(body.peer) || typeof body.accept !== "boolean")
      fail(
        400,
        "invalid_approval",
        "Use a peer ID and an explicit acceptance boolean.",
      );
    const { session: s } = this.#host(token);
    const p = s.members.get(body.peer);
    if (!p || p.role !== "guest" || p.approved || p.ended !== null)
      fail(
        409,
        "request_unavailable",
        "This request is no longer awaiting approval.",
      );
    const now = this.#now();
    if (!body.accept) {
      this.#cancelRoom(s, [p], "join_declined");
      this.#cancelPending(s, [p], "join_declined", now);
    } else {
      const event = { type: "peer-joined", peer: this.#participantView(p) };
      this.#room(s.members.get(s.host), event);
      this.#room(p, { type: "join-approved" });
      p.approved = true;
      p.approvedAt = now;
      this.#push(s, p, { type: "join-approved" }, false, false);
      this.#push(s, s.members.get(s.host), event, false, false);
    }
    // Prepare every queue before polling consumers can start negotiation.
    this.emit("changed", s.id);
    return {
      session: this.#sessionView(s, true),
      participant: this.#participantView(p),
    };
  }
  heartbeat(token, body) {
    object(body, ["ready"]);
    if (body.ready !== undefined && body.ready !== true)
      fail(400, "invalid_ready", "Ready, when supplied, must be true.");
    const { session: s, member: p } = this.#auth(token);
    if (body.ready && !p.approved)
      fail(
        403,
        "approval_required",
        "Await the host's approval before becoming ready.",
      );
    if (body.ready && p.status === "pending") {
      const event = {
        type: "peer-ready",
        peer: { ...this.#participantView(p), status: "active" },
      };
      this.#room(s.members.get(s.host), event);
      p.status = "active";
      this.#push(s, s.members.get(s.host), event);
    }
    p.lastSeen = this.#now();
    return {
      participant: this.#participantView(p),
      session: this.#sessionView(s, p.role === "host"),
    };
  }
  leave(token, body = {}) {
    object(body, []);
    const { session: s, member: p } = this.#auth(token);
    if (p.role === "host") this.#end(s, "host_left", this.#now());
    else this.#leave(s, p, "guest_left", this.#now());
    return {
      session: this.#sessionView(s, p.role === "host"),
      participant: this.#participantView(p),
    };
  }
  end(token, body = {}) {
    object(body, []);
    const { session: s, member: p } = this.#auth(token);
    if (p.role !== "host")
      fail(403, "host_only", "Only the host can end the session.");
    this.#end(s, "host_end", this.#now());
    return { session: this.#sessionView(s, true) };
  }
  #room(p, event) {
    const bytes = Buffer.byteLength(JSON.stringify(event)) + 32;
    if (
      p.queue.length >= this.limits.queueEvents ||
      p.queueBytes + bytes > this.limits.queueBytes
    )
      fail(
        429,
        "queue_full",
        "Recipient must read and acknowledge its pending events.",
      );
    return bytes;
  }
  #push(s, p, event, terminal = false, notify = true) {
    if (terminal) {
      p.queue = [];
      p.queueBytes = 0;
    }
    const bytes = terminal
      ? Buffer.byteLength(JSON.stringify(event)) + 32
      : this.#room(p, event);
    const item = { seq: ++s.seq, ...event };
    p.queue.push({ item, bytes });
    p.queueBytes += bytes;
    if (notify) this.emit("changed", s.id);
  }
  #leave(s, p, reason, now) {
    p.status = "left";
    p.ended = now;
    this.#push(s, p, { type: "participant-left", reason }, true);
    // Membership notices cannot be starved by a full signaling queue.
    const host = s.members.get(s.host);
    if (
      host.queue.length >= this.limits.queueEvents ||
      host.queueBytes + 256 > this.limits.queueBytes
    ) {
      this.#end(s, "signaling_overflow", now);
      return;
    }
    this.#push(s, host, { type: "peer-left", peer_id: p.id, reason });
  }
  #end(s, reason, now) {
    if (s.state !== "open") return;
    s.state = "ended";
    s.reason = reason;
    s.ended = now;
    for (const p of s.members.values()) {
      p.status = "ended";
      p.ended = now;
      this.#push(s, p, { type: "session-ended", reason }, true, false);
    }
    // Publish only after every terminal queue is ready: a polling guest must
    // never observe an ended session before its own final notice exists.
    this.emit("changed", s.id);
  }
  signal(token, body) {
    object(body, ["to", "kind", "payload"]);
    const { session: s, member: p } = this.#auth(token);
    const target = s.members.get(body.to);
    if (
      !hexId(body.to) ||
      !target ||
      target.ended !== null ||
      p.id === target.id ||
      p.role === target.role
    )
      fail(
        403,
        "invalid_peer",
        "Signals may only address this session's host or one of its guests.",
      );
    if (!p.approved || !target.approved)
      fail(
        403,
        "approval_required",
        "Both participants must be approved before signaling.",
      );
    if (!["offer", "answer", "ice"].includes(body.kind))
      fail(400, "invalid_signal", "Unsupported signal kind.");
    if (
      (body.kind === "offer" && p.role !== "host") ||
      (body.kind === "answer" && p.role !== "guest")
    )
      fail(403, "signal_role", "Host offers and guest answers are required.");
    let payload;
    if (body.kind === "ice") {
      payload = body.payload;
      if (payload !== null) {
        object(payload, [
          "candidate",
          "sdpMid",
          "sdpMLineIndex",
          "usernameFragment",
        ]);
        if (
          typeof payload.candidate !== "string" ||
          Buffer.byteLength(payload.candidate) > this.limits.iceBytes ||
          (payload.sdpMid != null &&
            (typeof payload.sdpMid !== "string" ||
              payload.sdpMid.length > 64)) ||
          (payload.sdpMLineIndex != null &&
            (!Number.isInteger(payload.sdpMLineIndex) ||
              payload.sdpMLineIndex < 0 ||
              payload.sdpMLineIndex > 32)) ||
          (payload.usernameFragment != null &&
            (typeof payload.usernameFragment !== "string" ||
              payload.usernameFragment.length > 64))
        )
          fail(
            400,
            "invalid_ice",
            "ICE candidate fields exceed the documented bounds.",
          );
      }
    } else {
      payload = object(body.payload, ["sdp"]);
      if (
        typeof payload.sdp !== "string" ||
        !payload.sdp.length ||
        Buffer.byteLength(payload.sdp) > this.limits.sdpBytes
      )
        fail(
          400,
          "invalid_sdp",
          "SDP must be nonempty and within the documented limit.",
        );
    }
    if (body.kind === "answer" && !p.offer)
      fail(409, "no_offer", "No host offer is awaiting an answer.");
    this.#room(target, {
      type: "signal",
      from: p.id,
      kind: body.kind,
      payload,
    });
    const now = this.#now();
    rate(p.rate, now, this.limits.signalPerMinute);
    rate(s.rate, now, this.limits.sessionSignalsPerMinute);
    if (body.kind === "offer") target.offer = true;
    if (body.kind === "answer") p.offer = false;
    // Clone transport data so callers cannot mutate a queued message afterward.
    this.#push(s, target, {
      type: "signal",
      from: p.id,
      kind: body.kind,
      payload: structuredClone(payload),
    });
    return { accepted: true };
  }
  events(token, after) {
    const { session: s, member: p } = this.#auth(token, true);
    if (!Number.isSafeInteger(after) || after < 0 || after > s.seq)
      fail(
        400,
        "invalid_cursor",
        "Cursor must be within this session's event sequence.",
      );
    if (p.ended === null && after < p.ack)
      fail(
        409,
        "cursor_expired",
        "Cursor has already been acknowledged; reconnect with the last cursor.",
      );
    if (p.ended === null) {
      p.ack = after;
      while (p.queue[0]?.item.seq <= after)
        p.queueBytes -= p.queue.shift().bytes;
    }
    return {
      session: this.#sessionView(s, p.role === "host"),
      participant: this.#participantView(p),
      events: p.queue
        .filter((e) => e.item.seq > after)
        .map((e) => structuredClone(e.item)),
      cursor: s.seq,
    };
  }
  sweep() {
    const now = this.#now();
    for (const [id, s] of this.#sessions) {
      if (s.state === "open") {
        if (now >= s.expires) this.#end(s, "expired", now);
        else if (now - s.members.get(s.host).lastSeen >= this.limits.hostMs)
          this.#end(s, "host_timeout", now);
        else
          for (const p of s.members.values()) {
            if (
              p.role === "guest" &&
              p.ended === null &&
              ((p.status === "pending" &&
                now - (p.approvedAt ?? p.joined) >= this.limits.pendingMs) ||
                (p.status === "active" &&
                  now - p.lastSeen >= this.limits.guestMs))
            )
              this.#leave(
                s,
                p,
                p.status === "pending"
                  ? p.approved
                    ? "reservation_timeout"
                    : "approval_timeout"
                  : "guest_timeout",
                now,
              );
          }
      }
      if (s.state === "ended" && now - s.ended >= this.limits.retentionMs) {
        for (const p of s.members.values())
          this.#credentials.delete(p.tokenHash);
        this.#sessions.delete(id);
        for (const code of s.codes) this.#codes.delete(code);
      } else if (s.state === "open")
        for (const [memberId, p] of s.members) {
          if (p.ended !== null && now - p.ended >= this.limits.retentionMs) {
            this.#credentials.delete(p.tokenHash);
            s.members.delete(memberId);
          }
        }
    }
    for (const [key, rates] of this.#clients) {
      if (now >= rates.create.until && now >= rates.join.until)
        this.#clients.delete(key);
    }
  }
  shutdown() {
    this.#closed = true;
    for (const s of this.#sessions.values())
      this.#end(s, "service_shutdown", this.#now());
  }
  diagnostics() {
    this.sweep();
    return {
      sessions: this.#sessions.size,
      credentials: this.#credentials.size,
      rate_clients: this.#clients.size,
      reserved_codes: this.#codes.size,
    };
  }
}
