import {
  CHANNEL,
  PROTOCOL,
  MAX_BUFFERED_BYTES,
  LOW_BUFFERED_BYTES,
  allowed,
  decode,
  encode,
  fail,
  id,
  record,
  spec,
  TransportError,
} from "./wire.ts";
import type { Frame, FrameKind, Json, ParticipantSpec, Role } from "./wire.ts";
import { grant, participant, session, Signaling } from "./signaling.ts";
import type { Participant, Session, SignalEvent } from "./signaling.ts";

type Timer = ReturnType<typeof setTimeout>;
export interface Clock {
  now(): number;
  setTimeout(callback: () => void, ms: number): Timer;
  clearTimeout(timer: Timer): void;
}
export interface TransportOptions {
  service: string;
  /** Explicit STUN/TURN configuration; no default relay or microphone/camera. */
  rtcConfiguration?: RTCConfiguration;
  fetch?: typeof fetch;
  createPeerConnection?: (configuration: RTCConfiguration) => RTCPeerConnection;
  clock?: Clock;
  onChange?: () => void;
  onFrame?: (peer: string, frame: Frame) => void;
  onWritable?: (peer: string) => void;
}
type Admission = "pending" | "admitted" | "rejected";
interface Peer {
  id: string;
  pc: RTCPeerConnection;
  channel: RTCDataChannel | null;
  state: "negotiating" | "channel-open" | "ready";
  hello: boolean;
  remoteDescription: boolean;
  descriptionSent: boolean;
  ice: (RTCIceCandidateInit | null)[];
  localIce: (RTCIceCandidateInit | null)[];
  iceCount: number;
  receivedIce: number;
  sendSeq: number;
  receiveSeq: number;
  outgoing: Promise<void>;
  outgoingCount: number;
  incoming: Promise<void>;
  admission: Admission;
  admissionEpoch: number | null;
  simulation: "waiting" | "running";
  deadline: Timer;
  disconnected: Timer | null;
  window: number;
  sent: number;
  received: number;
  receivedBytes: number;
}
/** Host-star transport only. It never imports Game, storage, Profile or save APIs. */
export class CoopTransport {
  #http: Signaling;
  #options: TransportOptions;
  #clock: Clock;
  #phase: "idle" | "connecting" | "signaling" | "ended" = "idle";
  #reason: string | null = null;
  #session: Session | null = null;
  #member: Participant | null = null;
  #token: string | null = null;
  #invite: string | null = null;
  #cursor = 0;
  #epoch = 0;
  #peers = new Map<string, Peer>();
  #retired = new Set<string>();
  #controllers = new Set<AbortController>();
  #timers = new Set<Timer>();
  constructor(options: TransportOptions) {
    this.#options = options;
    this.#http = new Signaling(options.service, options.fetch);
    this.#clock = options.clock ?? {
      now: () => performance.now(),
      setTimeout: (f, ms) => setTimeout(f, ms),
      clearTimeout: (t) => clearTimeout(t),
    };
  }
  #changed(): void {
    try {
      this.#options.onChange?.();
    } catch {
      /* Observer errors cannot grant membership. */
    }
  }
  #timer(fn: () => void, ms: number): Timer {
    const timer = this.#clock.setTimeout(() => {
      this.#timers.delete(timer);
      fn();
    }, ms);
    this.#timers.add(timer);
    return timer;
  }
  #clear(timer: Timer | null): void {
    if (timer !== null) {
      this.#clock.clearTimeout(timer);
      this.#timers.delete(timer);
    }
  }
  #live(epoch: number): boolean {
    return epoch === this.#epoch && this.#phase !== "ended";
  }
  async #request(
    path: string,
    body: unknown,
    token = this.#token,
    timeout = 10_000,
  ): Promise<unknown> {
    const controller = new AbortController();
    this.#controllers.add(controller);
    const timer = this.#timer(() => controller.abort(), timeout);
    try {
      return await this.#http.request(path, token, body, controller.signal);
    } finally {
      this.#clear(timer);
      this.#controllers.delete(controller);
    }
  }
  async #start(role: Role, body: unknown): Promise<void> {
    if (!["idle", "ended"].includes(this.#phase)) fail("already_in_session");
    const epoch = ++this.#epoch;
    this.#phase = "connecting";
    this.#reason = null;
    this.#changed();
    try {
      const g = grant(
        await this.#request(
          role === "host" ? "/v1/sessions" : "/v1/join",
          body,
          null,
        ),
        role,
      );
      if (!this.#live(epoch)) {
        // A delayed response must not reopen a disposed transport.
        void this.#request("/v1/leave", {}, g.participant.token, 1_500).catch(
          () => {},
        );
        return;
      }
      const { token, ...member } = g.participant;
      this.#token = token;
      this.#session = g.session;
      this.#member = member;
      this.#invite = g.invite ?? null;
      this.#cursor = g.cursor;
      this.#phase = "signaling";
      if (role === "guest") this.#peer(g.session.host_id);
      this.#changed();
      void this.#heartbeat(epoch, 0);
      void this.#poll(epoch, 0);
    } catch (error) {
      if (!this.#live(epoch)) return;
      await this.leave(this.#code(error));
      throw new TransportError(this.#code(error));
    }
  }
  create(seed: number): Promise<void> {
    if (!Number.isInteger(seed) || seed <= 0 || seed > 0xffff_ffff)
      fail("invalid_seed");
    return this.#start("host", { seed, protocol: PROTOCOL });
  }
  join(invite: string): Promise<void> {
    if (
      typeof invite !== "string" ||
      invite.length > 1_100 ||
      !/^#?[0-9a-f]{64,1024}$/i.test(invite.trim()) ||
      invite.trim().replace(/^#/, "").length % 2
    )
      fail("invalid_invite");
    return this.#start("guest", { invite });
  }
  /** Explicit capability getter for a user copy action; omitted from diagnostics. */
  invite(): string | null {
    return this.#invite;
  }
  diagnostics() {
    return {
      phase: this.#phase,
      reason: this.#reason,
      session: this.#session
        ? {
            id: this.#session.id,
            seed: this.#session.seed,
            protocol: this.#session.protocol,
          }
        : null,
      participant: this.#member ? { ...this.#member } : null,
      peers: [...this.#peers.values()].map((p) => ({
        id: p.id,
        transport: p.state,
        admission: p.admission,
        simulation: p.simulation,
        bufferedBytes: p.channel?.bufferedAmount ?? 0,
      })),
      pendingRequests: this.#controllers.size,
      timers: this.#timers.size,
    };
  }
  #code(error: unknown): string {
    return error instanceof TransportError ? error.code : "transport_failed";
  }
  async #heartbeat(epoch: number, failures: number): Promise<void> {
    if (!this.#live(epoch) || this.#phase !== "signaling") return;
    try {
      const value = record(
        await this.#request(
          "/v1/heartbeat",
          this.#member?.role === "guest" &&
            [...this.#peers.values()].some((p) => p.state === "ready")
            ? { ready: true }
            : {},
          this.#token,
          3_000,
        ),
      );
      if (!this.#live(epoch)) return;
      if (this.#sameSession(value.session).state !== "open") return;
      const p = participant(value.participant);
      if (p.id !== this.#member?.id || p.role !== this.#member.role)
        fail("changed_participant");
      this.#member = p;
      this.#changed();
      failures = 0;
    } catch (error) {
      if (!this.#live(epoch)) return;
      if (error instanceof TransportError || ++failures >= 3) {
        void this.leave(this.#code(error));
        return;
      }
    }
    this.#timer(() => void this.#heartbeat(epoch, failures), 5_000);
  }
  #sameSession(value: unknown): Session {
    const s = session(value);
    if (
      s.id !== this.#session?.id ||
      s.host_id !== this.#session.host_id ||
      s.seed !== this.#session.seed
    )
      fail("changed_session");
    if (s.state === "ended") this.#terminal(s.reason ?? "session_ended");
    return s;
  }
  async #poll(epoch: number, failures: number): Promise<void> {
    while (this.#live(epoch) && this.#phase === "signaling") {
      try {
        const result = record(
          await this.#request(
            `/v1/events?after=${this.#cursor}&wait_ms=20000`,
            undefined,
            this.#token,
            25_000,
          ),
        );
        if (!this.#live(epoch)) return;
        const s = this.#sameSession(result.session);
        if (s.state !== "open") return;
        const p = participant(result.participant);
        if (p.id !== this.#member?.id || p.role !== this.#member.role)
          fail("changed_participant");
        if (p.status === "left" || p.status === "ended") {
          this.#terminal("participant_left");
          return;
        }
        if (
          !Array.isArray(result.events) ||
          result.events.length > 64 ||
          !Number.isSafeInteger(result.cursor) ||
          Number(result.cursor) < this.#cursor
        )
          fail("invalid_events");
        let sequence = this.#cursor;
        for (const event of result.events) {
          const e = record(event) as SignalEvent;
          if (
            !Number.isSafeInteger(e.seq) ||
            e.seq <= sequence ||
            e.seq > Number(result.cursor)
          )
            fail("invalid_event_seq");
          sequence = e.seq;
          await this.#event(e, epoch);
          if (!this.#live(epoch)) return;
        }
        this.#cursor = Number(result.cursor);
        failures = 0;
      } catch (error) {
        if (!this.#live(epoch)) return;
        if (error instanceof TransportError || ++failures >= 3) {
          void this.leave(this.#code(error));
          return;
        }
        await new Promise<void>((resolve) => {
          const timer = this.#timer(resolve, 250 * 2 ** (failures - 1));
          // Teardown clears timers; the abandoned poll has no resource to keep alive.
          void timer;
        });
      }
    }
  }
  async #event(e: SignalEvent, epoch: number): Promise<void> {
    if (e.type === "session-ended" || e.type === "participant-left") {
      this.#terminal(
        typeof e.reason === "string" ? e.reason.slice(0, 128) : "session_ended",
      );
      return;
    }
    if (e.type === "peer-joined" || e.type === "peer-ready") {
      const p = participant(e.peer);
      if (
        this.#member?.role !== "host" ||
        p.role !== "guest" ||
        p.id === this.#member.id
      )
        fail("invalid_membership_event");
      if (
        e.type === "peer-joined" &&
        !this.#peers.has(p.id) &&
        !this.#retired.has(p.id)
      ) {
        const peer = this.#peer(p.id);
        try {
          const offer = await peer.pc.createOffer();
          if (!this.#current(peer, epoch)) return;
          await peer.pc.setLocalDescription(offer);
          if (!this.#current(peer, epoch)) return;
          await this.#description(
            peer,
            "offer",
            peer.pc.localDescription?.sdp ?? offer.sdp ?? "",
          );
        } catch {
          this.#drop(peer, "negotiation_failed");
        }
      }
    } else if (e.type === "peer-left") {
      if (!id(e.peer_id) || this.#member?.role !== "host")
        fail("invalid_membership_event");
      const peer = this.#peers.get(e.peer_id);
      if (peer) this.#drop(peer, "peer_left");
    } else if (e.type === "signal") {
      if (!id(e.from) || !["offer", "answer", "ice"].includes(String(e.kind)))
        fail("invalid_signal");
      const peer = this.#peers.get(e.from);
      // A previously accepted ICE event may still be in the HTTP queue when
      // its channel closes. It must not end unrelated host peers.
      if (!peer && this.#member?.role === "host" && this.#retired.has(e.from))
        return;
      if (
        !peer ||
        (this.#member?.role === "guest" && e.from !== this.#session?.host_id)
      )
        fail("unknown_signal_peer");
      const run = async () => {
        if (!this.#current(peer, epoch)) return;
        if (e.kind === "ice") {
          if (++peer.receivedIce > 16) fail("ice_bound");
          const candidate = this.#candidate(e.payload);
          if (!peer.remoteDescription) peer.ice.push(candidate);
          else await peer.pc.addIceCandidate(candidate);
        } else {
          const payload = record(e.payload, ["sdp"]);
          if (
            typeof payload.sdp !== "string" ||
            !payload.sdp ||
            new TextEncoder().encode(payload.sdp).length > 16_384 ||
            peer.remoteDescription ||
            (e.kind === "offer") !== (this.#member?.role === "guest")
          )
            fail("invalid_description");
          await peer.pc.setRemoteDescription({
            type: e.kind as "offer" | "answer",
            sdp: payload.sdp,
          });
          if (!this.#current(peer, epoch)) return;
          peer.remoteDescription = true;
          for (const candidate of peer.ice.splice(0))
            await peer.pc.addIceCandidate(candidate);
          if (e.kind === "offer") {
            const answer = await peer.pc.createAnswer();
            if (!this.#current(peer, epoch)) return;
            await peer.pc.setLocalDescription(answer);
            if (!this.#current(peer, epoch)) return;
            await this.#description(
              peer,
              "answer",
              peer.pc.localDescription?.sdp ?? answer.sdp ?? "",
            );
          }
        }
      };
      peer.incoming = peer.incoming
        .then(run)
        .catch(() => this.#drop(peer, "negotiation_failed"));
      await peer.incoming;
    } else fail("unknown_event");
  }
  #candidate(value: unknown): RTCIceCandidateInit | null {
    if (value === null) return null;
    const o = record(value, [
      "candidate",
      "sdpMid",
      "sdpMLineIndex",
      "usernameFragment",
    ]);
    if (
      typeof o.candidate !== "string" ||
      new TextEncoder().encode(o.candidate).length > 2_048 ||
      ["sdpMid", "usernameFragment"].some(
        (key) =>
          o[key] != null &&
          (typeof o[key] !== "string" || String(o[key]).length > 64),
      ) ||
      (o.sdpMLineIndex != null &&
        (!Number.isInteger(o.sdpMLineIndex) ||
          Number(o.sdpMLineIndex) < 0 ||
          Number(o.sdpMLineIndex) > 32))
    )
      fail("invalid_ice");
    return o as RTCIceCandidateInit;
  }
  async #description(
    peer: Peer,
    kind: "offer" | "answer",
    sdp: string,
  ): Promise<void> {
    if (!sdp || new TextEncoder().encode(sdp).length > 16_384)
      fail("sdp_bound");
    await this.#signal(peer, kind, { sdp });
    if (!this.#peers.has(peer.id)) return;
    peer.descriptionSent = true;
    for (const ice of peer.localIce.splice(0))
      void this.#signal(peer, "ice", ice);
  }
  #signal(peer: Peer, kind: string, payload: unknown): Promise<void> {
    if (peer.outgoingCount >= 20) {
      this.#drop(peer, "signal_queue_bound");
      return Promise.resolve();
    }
    peer.outgoingCount++;
    const epoch = this.#epoch;
    peer.outgoing = peer.outgoing
      .then(async () => {
        if (!this.#current(peer, epoch)) return;
        await this.#request("/v1/signals", { to: peer.id, kind, payload });
      })
      .catch((error) => this.#drop(peer, this.#code(error)))
      .finally(() => {
        peer.outgoingCount--;
      });
    return peer.outgoing;
  }
  #current(peer: Peer, epoch: number): boolean {
    return this.#live(epoch) && this.#peers.get(peer.id) === peer;
  }
  #peer(peerId: string): Peer {
    if (
      !id(peerId) ||
      peerId === this.#member?.id ||
      this.#peers.has(peerId) ||
      this.#peers.size >= (this.#member?.role === "host" ? 7 : 1)
    )
      fail("peer_bound");
    const epoch = this.#epoch;
    const factory =
      this.#options.createPeerConnection ??
      ((configuration: RTCConfiguration) =>
        new RTCPeerConnection(configuration));
    const pc = factory(this.#options.rtcConfiguration ?? { iceServers: [] });
    const p: Peer = {
      id: peerId,
      pc,
      channel: null,
      state: "negotiating",
      hello: false,
      remoteDescription: false,
      descriptionSent: false,
      ice: [],
      localIce: [],
      iceCount: 0,
      receivedIce: 0,
      sendSeq: 0,
      receiveSeq: 0,
      outgoing: Promise.resolve(),
      outgoingCount: 0,
      incoming: Promise.resolve(),
      admission: "pending",
      admissionEpoch: null,
      simulation: "waiting",
      deadline: this.#timer(() => this.#drop(p, "negotiation_timeout"), 15_000),
      disconnected: null,
      window: this.#clock.now(),
      sent: 0,
      received: 0,
      receivedBytes: 0,
    };
    this.#peers.set(peerId, p);
    pc.onicecandidate = (event) => {
      if (!this.#current(p, epoch)) return;
      try {
        if (++p.iceCount > 16) fail("ice_bound");
        const ice = this.#candidate(event.candidate?.toJSON() ?? null);
        if (!p.descriptionSent) p.localIce.push(ice);
        else void this.#signal(p, "ice", ice);
      } catch {
        this.#drop(p, "ice_bound");
      }
    };
    pc.onconnectionstatechange = () => {
      if (!this.#current(p, epoch)) return;
      if (pc.connectionState === "failed" || pc.connectionState === "closed")
        this.#drop(p, "connection_failed");
      else if (pc.connectionState === "disconnected" && p.disconnected === null)
        p.disconnected = this.#timer(
          () => this.#drop(p, "connection_lost"),
          5_000,
        );
      else if (pc.connectionState === "connected") {
        this.#clear(p.disconnected);
        p.disconnected = null;
      }
    };
    pc.ondatachannel = (event) => {
      if (!this.#current(p, epoch)) {
        event.channel.close();
        return;
      }
      if (this.#member?.role !== "guest" || p.channel) {
        event.channel.close();
        this.#drop(p, "unexpected_channel");
        return;
      }
      this.#channel(p, event.channel, epoch);
    };
    if (this.#member?.role === "host")
      this.#channel(p, pc.createDataChannel(CHANNEL, { ordered: true }), epoch);
    this.#changed();
    return p;
  }
  #window(peer: Peer): void {
    const now = this.#clock.now();
    if (now - peer.window >= 1_000) {
      peer.window = now;
      peer.sent = 0;
      peer.received = 0;
      peer.receivedBytes = 0;
    }
  }
  #channel(p: Peer, channel: RTCDataChannel, epoch: number): void {
    if (
      channel.label !== CHANNEL ||
      !channel.ordered ||
      channel.maxRetransmits !== null ||
      channel.maxPacketLifeTime !== null
    ) {
      channel.close();
      this.#drop(p, "invalid_channel");
      return;
    }
    p.channel = channel;
    channel.binaryType = "arraybuffer";
    channel.bufferedAmountLowThreshold = LOW_BUFFERED_BYTES;
    channel.onopen = () => {
      if (!this.#current(p, epoch)) return;
      p.state = "channel-open";
      if (
        !this.#send(p, "hello", {
          protocol: PROTOCOL,
          peer_id: this.#member!.id,
        })
      ) {
        this.#drop(p, "hello_failed");
        return;
      }
      this.#ready(p);
      this.#changed();
    };
    channel.onmessage = (event) => {
      if (!this.#current(p, epoch)) return;
      try {
        this.#window(p);
        if (
          ++p.received > 120 ||
          (p.receivedBytes +=
            typeof event.data === "string"
              ? new TextEncoder().encode(event.data).length
              : 65_537) > 2_097_152
        )
          fail("receive_rate");
        const frame = decode(event.data);
        const remoteRole = this.#member!.role === "host" ? "guest" : "host";
        if (
          frame.session !== this.#session!.id ||
          frame.seq !== p.receiveSeq + 1 ||
          !allowed(frame.kind, remoteRole)
        )
          fail("frame_authority");
        p.receiveSeq = frame.seq;
        if (frame.kind === "hello") {
          if (p.hello || record(frame.payload).peer_id !== p.id)
            fail("peer_identity");
          p.hello = true;
          this.#ready(p);
          return;
        }
        if (
          p.state !== "ready" ||
          (["input", "snapshot", "body", "body-query"].includes(frame.kind) &&
            p.admission !== "admitted")
        )
          fail("not_admitted");
        if (
          frame.kind === "input" &&
          record(frame.payload).epoch !== p.admissionEpoch
        )
          fail("stale_admission_epoch");
        if (
          frame.kind === "snapshot" &&
          (record(frame.payload).session !== this.#session!.id ||
            record(frame.payload).world_seed !== this.#session!.seed)
        )
          fail("snapshot_session");
        if (
          frame.kind === "body" &&
          record(frame.payload).session !== this.#session!.id
        )
          fail("body_session");
        if (frame.kind === "admission") {
          if (p.admission !== "pending") fail("duplicate_admission");
          p.admission = record(frame.payload).accepted
            ? "admitted"
            : "rejected";
          p.admissionEpoch = record(frame.payload).accepted
            ? Number(record(frame.payload).epoch)
            : null;
          this.#changed();
        }
        this.#options.onFrame?.(p.id, frame);
        if (frame.kind === "admission" && p.admission === "rejected")
          this.#drop(p, "admission_rejected");
      } catch {
        this.#drop(p, "invalid_frame");
      }
    };
    channel.onbufferedamountlow = () => {
      if (this.#current(p, epoch)) {
        try {
          this.#options.onWritable?.(p.id);
        } catch {
          /* Observer cannot change transport state. */
        }
      }
    };
    channel.onerror = () => this.#drop(p, "channel_error");
    channel.onclose = () => this.#drop(p, "channel_closed");
    if (channel.readyState === "open") channel.onopen(new Event("open"));
  }
  #ready(p: Peer): void {
    if (p.hello && p.channel?.readyState === "open" && p.state !== "ready") {
      p.state = "ready";
      this.#clear(p.deadline);
      this.#changed();
      if (this.#member?.role === "guest")
        void this.#request("/v1/heartbeat", { ready: true })
          .then((v) => {
            if (this.#peers.get(p.id) === p) {
              const o = record(v);
              if (
                this.#sameSession(o.session).state !== "open" ||
                this.#peers.get(p.id) !== p
              )
                return;
              const member = participant(o.participant);
              if (member.id !== this.#member?.id || member.role !== "guest")
                fail("changed_participant");
              this.#member = member;
              this.#changed();
            }
          })
          .catch((error) => this.#drop(p, this.#code(error)));
    }
  }
  #send(p: Peer, kind: FrameKind, payload: Json): boolean {
    const channel = p.channel;
    if (!channel || channel.readyState !== "open" || p.sendSeq >= 0xffff_ffff)
      return false;
    this.#window(p);
    if (p.sent >= 120) return false;
    const text = encode({
      v: 1,
      session: this.#session!.id,
      seq: p.sendSeq + 1,
      kind,
      payload,
    });
    const bytes = new TextEncoder().encode(text).length;
    if (
      channel.bufferedAmount + bytes > MAX_BUFFERED_BYTES ||
      (p.pc.sctp?.maxMessageSize && p.pc.sctp.maxMessageSize < bytes)
    )
      return false;
    try {
      channel.send(text);
    } catch {
      this.#drop(p, "send_failed");
      return false;
    }
    p.sendSeq++;
    p.sent++;
    return true;
  }
  send(
    peerId: string,
    kind: Exclude<FrameKind, "hello">,
    payload: Json,
  ): boolean {
    const p = this.#peers.get(peerId);
    if (
      !p ||
      this.#phase !== "signaling" ||
      p.state !== "ready" ||
      !allowed(kind, this.#member!.role)
    )
      return false;
    if (
      ["input", "snapshot", "body", "body-query"].includes(kind) &&
      p.admission !== "admitted"
    )
      return false;
    if (kind === "admission" && p.admission !== "pending") return false;
    if (kind === "join") spec(payload);
    if (kind === "input" && record(payload).epoch !== p.admissionEpoch)
      return false;
    if (
      kind === "snapshot" &&
      (record(payload).session !== this.#session!.id ||
        record(payload).world_seed !== this.#session!.seed)
    )
      return false;
    if (kind === "body" && record(payload).session !== this.#session!.id)
      return false;
    const sent = this.#send(p, kind, payload);
    if (sent && kind === "admission") {
      p.admission = record(payload).accepted ? "admitted" : "rejected";
      p.admissionEpoch = record(payload).accepted
        ? Number(record(payload).epoch)
        : null;
      this.#changed();
    }
    return sent;
  }
  /** Call only after the integration adapter has admitted/applied real Rust state. */
  markSimulation(peerId: string, running: boolean): boolean {
    const p = this.#peers.get(peerId);
    if (!p || p.state !== "ready" || p.admission !== "admitted") return false;
    p.simulation = running ? "running" : "waiting";
    this.#changed();
    return true;
  }
  #closePeer(p: Peer): void {
    this.#clear(p.deadline);
    this.#clear(p.disconnected);
    p.pc.onicecandidate = null;
    p.pc.onconnectionstatechange = null;
    p.pc.ondatachannel = null;
    if (p.channel) {
      p.channel.onopen = null;
      p.channel.onmessage = null;
      p.channel.onclose = null;
      p.channel.onerror = null;
      p.channel.onbufferedamountlow = null;
      try {
        p.channel.close();
      } catch {}
    }
    try {
      p.pc.close();
    } catch {}
    p.ice.length = 0;
    p.localIce.length = 0;
  }
  #drop(p: Peer, reason: string): void {
    if (this.#peers.get(p.id) !== p) return;
    this.#peers.delete(p.id);
    if (this.#retired.size >= 64)
      this.#retired.delete(this.#retired.values().next().value!);
    this.#retired.add(p.id);
    this.#closePeer(p);
    if (this.#member?.role === "guest") {
      void this.leave(reason);
      return;
    }
    this.#changed();
  }
  #terminal(reason: string): void {
    this.#phase = "ended";
    this.#reason = reason;
    ++this.#epoch;
    for (const c of this.#controllers) c.abort();
    this.#controllers.clear();
    for (const t of this.#timers) this.#clock.clearTimeout(t);
    this.#timers.clear();
    for (const p of this.#peers.values()) this.#closePeer(p);
    this.#peers.clear();
    this.#retired.clear();
    this.#token = null;
    this.#invite = null;
    this.#session = null;
    this.#member = null;
    this.#cursor = 0;
    this.#changed();
  }
  async #finish(path: string, reason: string): Promise<void> {
    const token = this.#token;
    this.#terminal(reason);
    if (token) {
      const controller = new AbortController();
      const timer = this.#clock.setTimeout(() => controller.abort(), 1_500);
      try {
        await this.#http.request(path, token, {}, controller.signal);
      } catch {
        /* Local teardown is unconditional; service timeout bounds a lost notice. */
      } finally {
        this.#clock.clearTimeout(timer);
      }
    }
  }
  leave(reason = "local_left"): Promise<void> {
    return this.#finish("/v1/leave", reason);
  }
  end(): Promise<void> {
    if (this.#member?.role !== "host") fail("host_only");
    return this.#finish("/v1/end", "host_end");
  }
  dispose(): Promise<void> {
    return this.leave("disposed");
  }
}
