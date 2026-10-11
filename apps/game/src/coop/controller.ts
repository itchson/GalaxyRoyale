import { CoopTransport } from "./transport.ts";
import type { TransportOptions } from "./transport.ts";
import {
  PROTOCOL,
  TransportError,
  fail,
  id,
  json,
  publication,
  record,
  uint,
  validatePayload,
} from "./wire.ts";
import type { Frame, FrameKind, Json, ParticipantSpec } from "./wire.ts";

export interface CoopState {
  phase:
    | "idle"
    | "creating"
    | "joining"
    | "hosting"
    | "admitting"
    | "loading"
    | "connected"
    | "ended";
  role: "host" | "guest" | null;
  members: number;
  transport_ready: number;
  admitted: number;
  reason: string | null;
  message: string;
  invite_available: boolean;
  progression: false;
}
export interface FlightInput {
  joystick?: [number, number] | null;
  destination?: [number, number] | null;
  aim?: [number, number] | null;
  thrust?: [number, number] | null;
  fire: boolean;
}
export interface Publication {
  acquisition: number;
  snapshot: {
    session: string;
    revision: number;
    tick: number;
    world_seed: number;
    participants: Json[];
  } & Record<string, Json>;
  counts: {
    entities: number;
    occupied_cells: number;
    storage_cells: number;
    entity_pages: number;
    body_pages: number;
  };
}
/** Concrete delegates must call the native transaction/render path, never JS gameplay. */
export interface ReplicaPort {
  begin(
    session: string,
    peer: string,
    epoch: number,
    worldSeed: number,
  ): boolean;
  stage(header: Publication): boolean;
  page(page: Json): boolean;
  commit(acquisition: number, revision: number): boolean;
  end(): void;
}
export interface ControllerOptions extends Omit<
  TransportOptions,
  "onChange" | "onFrame" | "onWritable"
> {
  localSpec: () => ParticipantSpec;
  native: { co_op_batch(request: string): string };
  replica?: ReplicaPort;
  availability?: { host: boolean; guest: boolean };
  onChange?: () => void;
  now?: () => number;
  /** Lifecycle tests can inject the transport boundary; production uses actual RTC. */
  transportFactory?: (options: TransportOptions) => CoopTransport;
}
type Row = Record<string, Json> & {
  id: string;
  body_signature: string;
  occupied: number;
  width: number;
  height: number;
};
const bodyKey = (r: Row): string =>
  `${r.body_signature}:${r.width}:${r.height}:${r.occupied}`;
type Piece = {
  kind: "publication" | "body" | "publication-end";
  payload: Json;
};
type Peer = {
  epoch: number;
  admission: Json | null;
  cache: Map<string, string>;
  index: number;
  ended: boolean;
  acknowledged: boolean;
  history: { at: number; bytes: number }[];
};
type Capture = {
  header: Publication;
  pieces: Piece[];
  rows: Row[];
  entityOffset: number;
  bodyIndex: number;
  bodyOffset: number;
  complete: boolean;
  started: number;
};
type Receiving = {
  header: Publication;
  rows: Row[];
  entityOffset: number;
  bodyIndex: number;
  bodyOffset: number;
  lastCell: number;
  entityPages: number;
  bodyPages: number;
};
const MAX_BYTES = 1_048_576,
  MAX_FRAMES = 80,
  TRANSFER_MS = 30_000;
const message = (phase: CoopState["phase"], reason: string | null): string => {
  if (reason === "host_ended") return "The host ended the session.";
  if (reason === "restore_failed")
    return "Connection ended. Solo restoration failed. Retry restore.";
  if (reason === "native_unavailable")
    return "Native co-op hosting is unavailable.";
  if (reason === "replica_unavailable")
    return "Native guest rendering is unavailable.";
  if (reason === "admission_rejected")
    return "The host could not admit this participant.";
  if (reason === "publication_stalled")
    return "Shared-world updates stopped. Session closed.";
  if (reason === "local_left" || reason === "disposed")
    return "Session closed.";
  return {
    idle: "Co-op is not active.",
    creating: "Creating an invitation...",
    joining: "Joining the signaling session...",
    hosting: "Hosting a temporary session. Progression is disabled.",
    admitting: "Channel ready. Waiting for host admission...",
    loading: "Loading the native shared-world view...",
    connected: "Native shared-world view active. Progression is disabled.",
    ended: "Connection ended.",
  }[phase];
};

/** Session adapter only. Galaxy.step remains the sole authority tick/render clock. */
export class CoopController {
  #options: ControllerOptions;
  #transport: CoopTransport;
  #phase: CoopState["phase"] = "idle";
  #role: CoopState["role"] = null;
  #reason: string | null = null;
  #generation = 0;
  #disposed = false;
  #restoreFailed = false;
  #session: string | null = null;
  #self: string | null = null;
  #host: string | null = null;
  #nativeActive = false;
  #replicaActive = false;
  #joined = false;
  #epoch = 0;
  #worldSeed = 0;
  #lastPublication = 0;
  #sequence = 0;
  #destination: [number, number] | null = null;
  #cancelCourse = false;
  #inputTime = 1;
  #publicationTime = 1;
  #members = 0;
  #peers = new Map<string, Peer>();
  #inputs = new Map<string, Json>();
  #capture: Capture | null = null;
  #receiving: Receiving | null = null;
  #cache = new Map<string, string>();
  #ack: Json | null = null;
  constructor(options: ControllerOptions) {
    this.#options = options;
    this.#transport = (
      options.transportFactory ?? ((o) => new CoopTransport(o))
    )({
      ...options,
      onChange: () => this.#sync(),
      onFrame: (peer, frame) => this.#receive(peer, frame),
    });
  }
  #now(): number {
    return (this.#options.now ?? (() => performance.now()))();
  }
  #emit(): void {
    try {
      this.#options.onChange?.();
    } catch {
      /* No observer authority. */
    }
  }
  state(): CoopState {
    const d = this.#transport.diagnostics(),
      live = this.#session !== null;
    return {
      phase: this.#phase,
      role: this.#role,
      members:
        live && !this.#restoreFailed
          ? this.#role === "host"
            ? 1 + this.#peers.size
            : this.#members
          : 0,
      transport_ready: live
        ? d.peers.filter((p) => p.transport === "ready").length
        : 0,
      admitted: live
        ? d.peers.filter((p) => p.admission === "admitted").length
        : 0,
      reason: this.#reason,
      message: message(this.#phase, this.#reason),
      invite_available:
        this.#phase === "hosting" && this.#transport.invite() !== null,
      progression: false,
    };
  }
  invite(): string | null {
    return this.#phase === "hosting" ? this.#transport.invite() : null;
  }
  /** Root must suppress solo ticks/saves while this transient ownership exists. */
  frameMode(): "solo" | "host" | "guest" | "blocked" {
    if (this.#restoreFailed) return "blocked";
    if (this.#nativeActive) return "host";
    if (this.#replicaActive) return "guest";
    return this.#session ||
      this.#phase === "creating" ||
      this.#phase === "joining"
      ? "blocked"
      : "solo";
  }
  blocksSolo(): boolean {
    return this.frameMode() !== "solo";
  }
  /** Existing zero-stick semantics clear the native pilot's persistent course. */
  cancelInput(): void {
    this.#destination = null;
    this.#cancelCourse = true;
    this.#inputTime = 1;
  }
  #results(operations: Json[]): ({ ok: true; value: Json } | { ok: false })[] {
    if (operations.length > 16) fail("native_batch_bound");
    const request = JSON.stringify({ v: 1, operations });
    if (new TextEncoder().encode(request).length > 65_536)
      fail("native_batch_bound");
    const text = this.#options.native.co_op_batch(request);
    if (
      typeof text !== "string" ||
      new TextEncoder().encode(text).length > 1_100_000
    )
      fail("native_response_bound");
    const response = record(JSON.parse(text), ["v", "results"]);
    if (
      response.v !== 1 ||
      !Array.isArray(response.results) ||
      response.results.length !== operations.length
    )
      fail("native_response");
    return response.results.map((item) => {
      const r = record(item, ["ok", "value", "error"]);
      if (
        r.ok === false &&
        typeof r.error === "string" &&
        /^[a-z0-9_]{1,64}$/.test(r.error)
      )
        return { ok: false };
      if (r.ok !== true || r.error !== undefined) fail("native_response");
      json(r.value);
      return { ok: true, value: r.value };
    });
  }
  #batch(operations: Json[]): Json[] {
    return this.#results(operations).map((r) => {
      if (!r.ok) fail("native_rejected");
      return r.value;
    });
  }
  #retire(peer: string): void {
    const p = this.#peers.get(peer);
    if (!p || !this.#session) return;
    this.#batch([
      { op: "leave", session: this.#session, peer, epoch: p.epoch },
    ]);
    this.#peers.delete(peer);
    this.#inputs.delete(peer);
    this.#capture = null;
    this.#transport.disconnect(peer);
  }
  #restore(): void {
    this.#restoreFailed = false;
    if (this.#nativeActive && this.#session) {
      try {
        this.#batch([{ op: "end", session: this.#session }]);
        this.#nativeActive = false;
      } catch {
        this.#reason = "restore_failed";
        this.#restoreFailed = true;
      }
    }
    if (this.#replicaActive) {
      try {
        this.#options.replica!.end();
        this.#replicaActive = false;
      } catch {
        this.#reason = "restore_failed";
        this.#restoreFailed = true;
      }
    }
    this.#capture = this.#receiving = null;
    this.#peers.clear();
    this.#inputs.clear();
    this.#cache.clear();
    this.#ack = null;
    if (!this.#restoreFailed) this.#session = this.#self = this.#host = null;
    this.#epoch = this.#sequence = 0;
    this.#worldSeed = 0;
    this.#lastPublication = 0;
    this.#destination = null;
    this.#cancelCourse = false;
    this.#joined = false;
  }
  async #start(role: "host" | "guest", invite?: string): Promise<void> {
    if (this.#disposed) fail("disposed");
    if (this.#restoreFailed) fail("restore_failed");
    if (this.#options.availability?.[role] === false)
      fail(role === "host" ? "native_unavailable" : "replica_unavailable");
    const generation = ++this.#generation;
    this.#restore();
    if (this.#restoreFailed) {
      this.#phase = "ended";
      this.#emit();
      fail("restore_failed");
    }
    this.#role = role;
    this.#reason = null;
    this.#phase = role === "host" ? "creating" : "joining";
    this.#emit();
    await this.#transport.leave("replaced");
    if (generation !== this.#generation) return;
    if (this.#restoreFailed) {
      this.#phase = "ended";
      this.#emit();
      fail("restore_failed");
    }
    this.#role = role;
    this.#reason = null;
    this.#phase = role === "host" ? "creating" : "joining";
    this.#emit();
    try {
      const local = this.#options.localSpec();
      validatePayload("join", local);
      if (role === "host") await this.#transport.create(local.local_seed);
      else await this.#transport.join(invite!);
      if (generation !== this.#generation) return;
      const d = this.#transport.diagnostics();
      if (!d.session || !d.participant || d.participant.role !== role)
        fail("signaling_unavailable");
      this.#session = d.session.id;
      this.#self = d.participant.id;
      this.#worldSeed = d.session.seed;
      this.#inputTime = this.#publicationTime = 1;
      if (role === "host") {
        // Retain ownership if a successful native side effect has a malformed reply.
        this.#nativeActive = true;
        const result = this.#results([
          {
            op: "begin",
            protocol: PROTOCOL,
            session: this.#session,
            host: this.#self,
          },
        ])[0];
        if (!result.ok) {
          this.#nativeActive = false;
          fail("native_rejected");
        }
        const a = record(result.value);
        if (
          a.id !== this.#self ||
          a.slot !== 0 ||
          !uint(a.epoch) ||
          a.epoch === 0
        )
          fail("native_admission");
        this.#nativeActive = true;
        this.#epoch = a.epoch;
        this.#phase = "hosting";
      } else this.#phase = "joining";
      this.#sync();
    } catch (error) {
      if (generation === this.#generation)
        await this.#stop(
          error instanceof TransportError &&
            ["native_unavailable", "replica_unavailable"].includes(error.code)
            ? error.code
            : "session_failed",
        );
    }
  }
  create(): Promise<void> {
    return this.#start("host");
  }
  join(invite: string): Promise<void> {
    return this.#start("guest", invite);
  }
  async #stop(reason: string): Promise<void> {
    ++this.#generation;
    this.#reason = reason;
    this.#phase = "ended";
    this.#restore();
    this.#emit();
    await this.#transport.leave(reason);
  }
  leave(): Promise<void> {
    return this.#stop("local_left");
  }
  async end(): Promise<void> {
    if (this.#role !== "host" || !this.#session) fail("host_only");
    for (const peer of this.#transport.diagnostics().peers)
      this.#transport.send(peer.id, "notice", {
        code: "host_ended",
        ended: true,
      });
    ++this.#generation;
    this.#reason = "host_ended";
    this.#phase = "ended";
    this.#restore();
    this.#emit();
    await this.#transport.end();
  }
  async dispose(): Promise<void> {
    this.#disposed = true;
    await this.#stop("disposed");
    await this.#transport.dispose();
  }
  #sync(): void {
    const d = this.#transport.diagnostics();
    if (this.#session && !this.#restoreFailed && d.phase === "ended") {
      this.#reason =
        d.reason === "host_ended" ? "host_ended" : "connection_ended";
      this.#phase = "ended";
      this.#restore();
    }
    if (this.#session && this.#role === "host") {
      for (const [peer, p] of this.#peers)
        if (!d.peers.some((x) => x.id === peer)) {
          try {
            this.#batch([
              { op: "leave", session: this.#session, peer, epoch: p.epoch },
            ]);
          } catch {
            void this.#stop("native_rejected");
            return;
          }
          this.#peers.delete(peer);
          this.#inputs.delete(peer);
          this.#capture = null;
        }
    }
    if (this.#session && this.#role === "guest" && !this.#joined) {
      const peer = d.peers.find((p) => p.transport === "ready");
      if (peer) {
        this.#phase = "admitting";
        this.#host = peer.id;
        this.#joined = this.#transport.send(
          peer.id,
          "join",
          this.#options.localSpec() as unknown as Json,
        );
      }
    }
    this.#emit();
  }
  #receive(peer: string, frame: Frame): void {
    if (
      !this.#session ||
      this.#phase === "ended" ||
      this.#restoreFailed ||
      frame.session !== this.#session
    )
      return;
    validatePayload(frame.kind, frame.payload);
    if (this.#role === "host") {
      if (frame.kind === "join") {
        if (this.#peers.has(peer)) fail("duplicate_admission");
        try {
          const a = record(
            this.#batch([
              {
                op: "join",
                session: this.#session,
                participant: { ...record(frame.payload), id: peer } as Json,
              },
            ])[0],
          );
          if (
            a.id !== peer ||
            !uint(a.epoch) ||
            a.epoch === 0 ||
            !uint(a.slot) ||
            a.slot < 1 ||
            a.slot > 7
          )
            fail("native_admission");
          this.#peers.set(peer, {
            epoch: a.epoch,
            admission: { accepted: true, slot: a.slot, epoch: a.epoch },
            cache: new Map(),
            index: 0,
            ended: false,
            acknowledged: false,
            history: [],
          });
          this.#capture = null;
        } catch {
          this.#transport.send(peer, "admission", {
            accepted: false,
            reason: "native_rejected",
          });
        }
      } else if (frame.kind === "input") {
        if (!this.#peers.has(peer)) fail("unknown_pilot");
        this.#inputs.set(peer, frame.payload);
      } else if (frame.kind === "publication-ack") {
        const c = this.#capture,
          a = record(frame.payload),
          p = this.#peers.get(peer);
        if (
          c &&
          p &&
          p.ended &&
          a.acquisition === c.header.acquisition &&
          a.revision === c.header.snapshot.revision
        ) {
          p.cache = new Map(c.rows.map((r) => [r.id, bodyKey(r)]));
          p.acknowledged = true;
          this.#transport.markSimulation(peer, true);
        }
      }
    } else if (peer === this.#host) {
      if (frame.kind === "admission") {
        const a = record(frame.payload);
        if (a.accepted !== true) {
          void this.#stop("admission_rejected");
          return;
        }
        this.#epoch = Number(a.epoch);
        if (!this.#options.replica) {
          void this.#stop("replica_unavailable");
          return;
        }
        this.#replicaActive = true;
        try {
          if (
            !this.#options.replica.begin(
              this.#session,
              this.#self!,
              this.#epoch,
              this.#worldSeed,
            )
          ) {
            this.#replicaActive = false;
            void this.#stop("replica_rejected");
            return;
          }
        } catch {
          void this.#stop("replica_failed");
          return;
        }
        this.#phase = "loading";
        this.#lastPublication = this.#now();
      } else if (frame.kind === "publication") {
        const h = frame.payload as unknown as Publication;
        if (
          !this.#replicaActive ||
          !h.snapshot.participants.some(
            (p) =>
              record(p).id === this.#self && record(p).epoch === this.#epoch,
          )
        )
          fail("publication_pilot");
        if (!this.#options.replica!.stage(h)) fail("replica_stage");
        this.#receiving = {
          header: h,
          rows: [],
          entityOffset: 0,
          bodyIndex: 0,
          bodyOffset: 0,
          lastCell: -1,
          entityPages: 0,
          bodyPages: 0,
        };
        this.#lastPublication = this.#now();
        this.#ack = null;
        this.#members = h.snapshot.participants.length;
      } else if (frame.kind === "body") this.#page(frame.payload);
      else if (frame.kind === "publication-end") this.#commit(frame.payload);
      else if (frame.kind === "notice" && record(frame.payload).ended === true)
        void this.#stop(
          record(frame.payload).code === "host_ended"
            ? "host_ended"
            : "connection_ended",
        );
    }
    this.#emit();
  }
  #nextBody(r: Receiving): Row | undefined {
    while (
      r.bodyIndex < r.rows.length &&
      (r.rows[r.bodyIndex].occupied === 0 ||
        this.#cache.get(r.rows[r.bodyIndex].id) ===
          bodyKey(r.rows[r.bodyIndex]))
    )
      r.bodyIndex++;
    return r.rows[r.bodyIndex];
  }
  #page(payload: Json): void {
    const r = this.#receiving;
    if (!r) fail("publication_missing");
    const p = record(payload),
      h = r.header;
    if (
      p.session !== h.snapshot.session ||
      p.tick !== h.snapshot.tick ||
      p.revision !== h.snapshot.revision
    )
      fail("publication_revision");
    if (Array.isArray(p.entities)) {
      if (
        r.bodyPages ||
        p.offset !== r.entityOffset ||
        ++r.entityPages > h.counts.entity_pages
      )
        fail("publication_order");
      for (const raw of p.entities) {
        const row = record(raw) as Row;
        if (
          typeof row.body_signature !== "string" ||
          !/^[0-9a-f]{16}$/.test(row.body_signature) ||
          (r.rows.length && row.id <= r.rows.at(-1)!.id)
        )
          fail("publication_signature");
        r.rows.push(row);
      }
      r.entityOffset += p.entities.length;
      if (
        p.next_offset !==
          (r.entityOffset < h.counts.entities ? r.entityOffset : null) ||
        r.entityOffset > h.counts.entities
      )
        fail("publication_offset");
      if (r.entityOffset === h.counts.entities) {
        if (
          r.rows.reduce((n, x) => n + x.occupied, 0) !==
            h.counts.occupied_cells ||
          r.rows.reduce((n, x) => n + x.width * x.height, 0) !==
            h.counts.storage_cells ||
          r.rows.reduce((n, x) => n + Math.ceil(x.occupied / 1024), 0) !==
            h.counts.body_pages
        )
          fail("publication_counts");
      }
    } else {
      const row = this.#nextBody(r);
      if (
        r.entityOffset !== h.counts.entities ||
        !row ||
        p.id !== row.id ||
        p.width !== row.width ||
        p.height !== row.height ||
        p.occupied !== row.occupied ||
        p.offset !== r.bodyOffset
      )
        fail("publication_body");
      for (const cell of p.cells as number[][]) {
        if (cell[0] <= r.lastCell) fail("publication_cells");
        r.lastCell = cell[0];
      }
      r.bodyOffset += (p.cells as Json[]).length;
      r.bodyPages++;
      if (
        p.next_offset !== (r.bodyOffset < row.occupied ? r.bodyOffset : null) ||
        r.bodyOffset > row.occupied
      )
        fail("publication_offset");
      if (p.next_offset === null) {
        r.bodyIndex++;
        r.bodyOffset = 0;
        r.lastCell = -1;
      }
    }
    if (!this.#options.replica!.page(payload)) fail("replica_page");
  }
  #commit(payload: Json): void {
    const r = this.#receiving,
      p = record(payload);
    if (
      !r ||
      p.acquisition !== r.header.acquisition ||
      p.revision !== r.header.snapshot.revision ||
      r.entityOffset !== r.header.counts.entities ||
      r.bodyOffset ||
      this.#nextBody(r)
    )
      fail("publication_incomplete");
    if (
      !this.#options.replica!.commit(Number(p.acquisition), Number(p.revision))
    )
      fail("replica_commit");
    this.#cache = new Map(r.rows.map((x) => [x.id, bodyKey(x)]));
    this.#receiving = null;
    this.#ack = payload;
    this.#phase = "connected";
    this.#lastPublication = this.#now();
    this.#transport.markSimulation(this.#host!, true);
  }
  /** Call before the existing Galaxy.step; no simulation tick is performed here. */
  beforeStep(elapsed: number, input: FlightInput): void {
    if (!Number.isFinite(elapsed) || elapsed < 0 || elapsed > 0.25)
      fail("invalid_elapsed");
    if (!this.#session || this.#restoreFailed) return;
    this.#sync();
    if (!this.#session || this.#restoreFailed || this.#phase === "ended")
      return;
    if (
      this.#role === "guest" &&
      this.#replicaActive &&
      this.#now() - this.#lastPublication > TRANSFER_MS
    ) {
      void this.#stop("publication_stalled");
      return;
    }
    validatePayload("input", {
      epoch: this.#epoch,
      seq: this.#sequence,
      input,
    });
    if (input.destination && !this.#cancelCourse)
      this.#destination = [...input.destination];
    const controls: FlightInput = this.#cancelCourse
      ? {
          joystick: [0, 0],
          destination: null,
          thrust: null,
          aim: null,
          fire: false,
        }
      : { ...input, destination: this.#destination };
    this.#inputTime += elapsed;
    if (this.#role === "host") {
      const operations: Json[] = [];
      for (const [peer, payload] of this.#inputs) {
        const p = record(payload);
        operations.push({
          op: "input",
          session: this.#session,
          peer,
          epoch: p.epoch as number,
          seq: p.seq as number,
          input: p.input as Json,
        });
      }
      this.#inputs.clear();
      if (this.#inputTime >= 1 / 30) {
        if (this.#sequence === 0xffff_ffff) {
          void this.#stop("input_exhausted");
          return;
        }
        validatePayload("input", {
          epoch: this.#epoch,
          seq: this.#sequence + 1,
          input: controls,
        });
        operations.push({
          op: "host_input",
          session: this.#session,
          seq: ++this.#sequence,
          input: controls as unknown as Json,
        });
        this.#inputTime = 0;
      }
      try {
        if (operations.length)
          this.#results(operations).forEach((r, index) => {
            const op = record(operations[index]);
            if (r.ok) {
              if (op.op === "host_input") {
                this.#destination = null;
                this.#cancelCourse = false;
              }
              return;
            }
            if (op.op === "input") this.#retire(String(op.peer));
            else fail("native_rejected");
          });
      } catch {
        void this.#stop("native_rejected");
      }
    } else if (this.#phase === "connected" && this.#inputTime >= 1 / 30) {
      if (this.#sequence === 0xffff_ffff) {
        void this.#stop("input_exhausted");
        return;
      }
      if (
        this.#transport.send(this.#host!, "input", {
          epoch: this.#epoch,
          seq: this.#sequence + 1,
          input: controls as unknown as Json,
        })
      ) {
        this.#sequence++;
        this.#inputTime = 0;
        this.#destination = null;
        this.#cancelCourse = false;
      }
    }
    if (
      this.#ack &&
      this.#transport.send(this.#host!, "publication-ack", this.#ack)
    )
      this.#ack = null;
  }
  /** Call after Galaxy.step. Page serialization work is bounded to four calls/frame. */
  afterStep(elapsed: number): void {
    if (!Number.isFinite(elapsed) || elapsed < 0 || elapsed > 0.25)
      fail("invalid_elapsed");
    if (!this.#session || this.#restoreFailed || this.#role !== "host") return;
    this.#publicationTime += elapsed;
    for (const [peer, p] of this.#peers)
      if (p.admission && this.#transport.send(peer, "admission", p.admission))
        p.admission = null;
    if (this.#capture && [...this.#peers.values()].every((p) => p.acknowledged))
      this.#capture = null;
    const ready = [...this.#peers.values()].some((p) => !p.admission);
    try {
      if (!this.#capture && ready && this.#publicationTime >= 0.05) {
        const header = this.#batch([
          { op: "acquire", session: this.#session },
        ])[0];
        publication(header);
        const h = header as unknown as Publication;
        if (
          h.snapshot.session !== this.#session ||
          h.snapshot.world_seed !== this.#transport.diagnostics().session!.seed
        )
          fail("publication_session");
        this.#capture = {
          header: h,
          pieces: [{ kind: "publication", payload: h as unknown as Json }],
          rows: [],
          entityOffset: 0,
          bodyIndex: 0,
          bodyOffset: 0,
          complete: false,
          started: this.#now(),
        };
        for (const p of this.#peers.values()) {
          p.index = 0;
          p.ended = p.acknowledged = false;
        }
        this.#publicationTime = 0;
      }
      const c = this.#capture;
      if (!c) return;
      if (this.#now() - c.started > TRANSFER_MS) {
        for (const [peer, p] of this.#peers)
          if (!p.acknowledged) this.#retire(peer);
        return;
      }
      for (let work = 0; work < 4 && !c.complete; work++) this.#append(c);
      for (const [peer, p] of this.#peers) {
        if (p.admission || p.acknowledged) continue;
        while (p.history.length && this.#now() - p.history[0].at >= 1_000)
          p.history.shift();
        for (let work = 0; work < 2 && p.index < c.pieces.length; work++) {
          const piece = c.pieces[p.index];
          if (
            piece.kind === "body" &&
            !Array.isArray(record(piece.payload).entities)
          ) {
            const row =
              c.rows[
                c.rows.findIndex((x) => x.id === record(piece.payload).id)
              ];
            if (row && p.cache.get(row.id) === bodyKey(row)) {
              p.index++;
              work--;
              continue;
            }
          }
          const bytes =
            new TextEncoder().encode(JSON.stringify(piece.payload)).length +
            256;
          if (
            p.history.reduce((n, x) => n + x.bytes, 0) + bytes > MAX_BYTES ||
            p.history.length >= MAX_FRAMES ||
            !this.#transport.send(peer, piece.kind, piece.payload)
          )
            break;
          p.history.push({ at: this.#now(), bytes });
          p.index++;
          if (piece.kind === "publication-end") p.ended = true;
        }
      }
    } catch {
      void this.#stop("publication_failed");
    }
  }
  #append(c: Capture): void {
    const h = c.header;
    if (c.entityOffset < h.counts.entities) {
      const page = this.#batch([
        {
          op: "entity_page",
          session: this.#session,
          acquisition: h.acquisition,
          revision: h.snapshot.revision,
          offset: c.entityOffset,
        },
      ])[0];
      validatePayload("body", page);
      const p = record(page);
      if (
        p.offset !== c.entityOffset ||
        p.session !== this.#session ||
        p.revision !== h.snapshot.revision ||
        !Array.isArray(p.entities) ||
        !p.entities.length
      )
        fail("native_page");
      for (const raw of p.entities) {
        const row = record(raw) as Row;
        if (!/^[0-9a-f]{16}$/.test(row.body_signature))
          fail("publication_signature");
        c.rows.push(row);
      }
      c.entityOffset += p.entities.length;
      c.pieces.push({ kind: "body", payload: page as Json });
      return;
    }
    while (
      c.bodyIndex < c.rows.length &&
      (c.rows[c.bodyIndex].occupied === 0 ||
        [...this.#peers.values()].every(
          (p) =>
            p.cache.get(c.rows[c.bodyIndex].id) ===
            bodyKey(c.rows[c.bodyIndex]),
        ))
    )
      c.bodyIndex++;
    const row = c.rows[c.bodyIndex];
    if (!row) {
      c.pieces.push({
        kind: "publication-end",
        payload: { acquisition: h.acquisition, revision: h.snapshot.revision },
      });
      c.complete = true;
      return;
    }
    const page = this.#batch([
      {
        op: "body_page",
        session: this.#session,
        acquisition: h.acquisition,
        revision: h.snapshot.revision,
        entity: row.id,
        offset: c.bodyOffset,
      },
    ])[0];
    validatePayload("body", page);
    const p = record(page);
    if (
      p.id !== row.id ||
      p.offset !== c.bodyOffset ||
      p.session !== this.#session ||
      p.revision !== h.snapshot.revision ||
      !Array.isArray(p.cells) ||
      !p.cells.length
    )
      fail("native_page");
    c.pieces.push({ kind: "body", payload: page as Json });
    c.bodyOffset += p.cells.length;
    if (p.next_offset === null) {
      c.bodyIndex++;
      c.bodyOffset = 0;
    }
  }
}
