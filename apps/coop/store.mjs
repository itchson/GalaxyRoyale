import {
  createCipheriv,
  createDecipheriv,
  createHash,
  randomBytes,
} from "node:crypto";
import { EventEmitter } from "node:events";

export const PROTOCOL = "galaxy-coop-1";
export const LIMITS = Object.freeze({
  players: 8,
  sessions: 64,
  retainedParticipants: 64,
  sessionMs: 3_600_000,
  inviteMs: 600_000,
  pendingMs: 20_000,
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
const AAD = Buffer.from("galaxy-royale-coop/invite/v1");
const hexId = (s) => typeof s === "string" && /^[0-9a-f]{32}$/.test(s);
const hash = (s) => createHash("sha256").update(s).digest("hex");
const random = (n) => randomBytes(n).toString("hex");

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
  #clients = new Map();
  #globalRates = {
    create: { count: 0, until: 0 },
    join: { count: 0, until: 0 },
  };
  #clock;
  #lastTime = 0;
  #closed = false;
  constructor({ now = Date.now, limits = {} } = {}) {
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
  #participant(session, role, now) {
    const token = random(32);
    const member = {
      id: random(16),
      role,
      tokenHash: hash(token),
      status: role === "host" ? "active" : "pending",
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
  #sessionView(s) {
    return {
      id: s.id,
      seed: s.seed,
      protocol: s.protocol,
      host_id: s.host,
      expires_at: s.expires,
      state: s.state,
      reason: s.reason,
    };
  }
  #participantView(p) {
    return { id: p.id, role: p.role, status: p.status };
  }
  #invite(s) {
    const iv = randomBytes(12),
      cipher = createCipheriv("aes-256-gcm", this.#key, iv);
    cipher.setAAD(AAD);
    const content = Buffer.from(
      JSON.stringify({
        v: 1,
        session_id: s.id,
        invite_id: s.inviteId,
        expires_at: s.inviteExpires,
      }),
    );
    return (
      "#" +
      Buffer.concat([
        Buffer.from([1]),
        iv,
        cipher.update(content),
        cipher.final(),
        cipher.getAuthTag(),
      ])
        .toString("hex")
        .toUpperCase()
    );
  }
  #decode(invite) {
    const encoded =
      typeof invite === "string" && invite.length <= 1100
        ? invite.trim().replace(/^#/, "").toLowerCase()
        : "";
    if (!/^[0-9a-f]{64,1024}$/.test(encoded) || encoded.length % 2)
      fail(
        401,
        "invalid_invite",
        "Invite is invalid or belongs to a previous service run.",
      );
    let body;
    try {
      const bytes = Buffer.from(encoded, "hex");
      if (bytes[0] !== 1) throw new Error();
      const decipher = createDecipheriv(
        "aes-256-gcm",
        this.#key,
        bytes.subarray(1, 13),
      );
      decipher.setAAD(AAD);
      decipher.setAuthTag(bytes.subarray(-16));
      body = JSON.parse(
        Buffer.concat([
          decipher.update(bytes.subarray(13, -16)),
          decipher.final(),
        ]).toString(),
      );
      object(body, ["v", "session_id", "invite_id", "expires_at"]);
      if (
        body.v !== 1 ||
        !hexId(body.session_id) ||
        !hexId(body.invite_id) ||
        !Number.isSafeInteger(body.expires_at)
      )
        throw new Error();
    } catch {
      fail(
        401,
        "invalid_invite",
        "Invite is invalid or belongs to a previous service run.",
      );
    }
    if (this.#now() >= body.expires_at)
      fail(410, "invite_expired", "Invite has expired.");
    return body;
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
    const s = {
      id: random(16),
      inviteId: random(16),
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
    };
    const { member, token } = this.#participant(s, "host", now);
    s.host = member.id;
    if (previous) this.#end(previous.session, "replaced", now);
    this.#sessions.set(s.id, s);
    return {
      session: this.#sessionView(s),
      invite: this.#invite(s),
      participant: { ...this.#participantView(member), token },
      cursor: 0,
    };
  }
  join(body) {
    this.#open();
    this.sweep();
    object(body, ["invite"]);
    const invite = this.#decode(body.invite),
      s = this.#sessions.get(invite.session_id);
    if (!s || s.state !== "open" || s.inviteId !== invite.invite_id)
      fail(410, "session_ended", "Session is no longer open.");
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
      type: "peer-joined",
      peer: { id: "0".repeat(32), role: "guest", status: "pending" },
    });
    const { member, token } = this.#participant(s, "guest", this.#now());
    this.#push(s, host, {
      type: "peer-joined",
      peer: this.#participantView(member),
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
  heartbeat(token, body) {
    object(body, ["ready"]);
    if (body.ready !== undefined && body.ready !== true)
      fail(400, "invalid_ready", "Ready, when supplied, must be true.");
    const { session: s, member: p } = this.#auth(token);
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
      session: this.#sessionView(s),
    };
  }
  leave(token, body = {}) {
    object(body, []);
    const { session: s, member: p } = this.#auth(token);
    if (p.role === "host") this.#end(s, "host_left", this.#now());
    else this.#leave(s, p, "guest_left", this.#now());
    return {
      session: this.#sessionView(s),
      participant: this.#participantView(p),
    };
  }
  end(token, body = {}) {
    object(body, []);
    const { session: s, member: p } = this.#auth(token);
    if (p.role !== "host")
      fail(403, "host_only", "Only the host can end the session.");
    this.#end(s, "host_end", this.#now());
    return { session: this.#sessionView(s) };
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
      session: this.#sessionView(s),
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
                now - p.joined >= this.limits.pendingMs) ||
                (p.status === "active" &&
                  now - p.lastSeen >= this.limits.guestMs))
            )
              this.#leave(
                s,
                p,
                p.status === "pending"
                  ? "reservation_timeout"
                  : "guest_timeout",
                now,
              );
          }
      }
      if (s.state === "ended" && now - s.ended >= this.limits.retentionMs) {
        for (const p of s.members.values())
          this.#credentials.delete(p.tokenHash);
        this.#sessions.delete(id);
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
    };
  }
}
