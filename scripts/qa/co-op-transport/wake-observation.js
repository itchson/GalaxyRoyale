// Observation/control setup only. No JS damage, trail generation or visual expiry.
export const WAKE_SCENARIO = "configured-ion-wake-r1-e1";
export const wakeReasons = new Set([
  "wake_fresh_origin_required",
  "wake_storage_unavailable",
  "wake_fixture_mismatch",
  "wake_normal_wasm_required",
  "wake_context",
  "wake_page_mismatch",
  "wake_sequence",
  "wake_timeout",
  "wake_native_frame",
  "wake_lifetime_expired",
]);
const uint = (v) => Number.isInteger(v) && v >= 0 && v <= 0xffff_ffff;
const exact = (v, keys) =>
  v &&
  typeof v === "object" &&
  !Array.isArray(v) &&
  Object.keys(v).sort().join() === [...keys].sort().join();
export const canonical = (v) =>
  Array.isArray(v)
    ? v.map(canonical)
    : v && typeof v === "object"
      ? Object.fromEntries(
          Object.keys(v)
            .sort()
            .map((k) => [k, canonical(v[k])]),
        )
      : v;
export const sameValue = (a, b) =>
  JSON.stringify(canonical(a)) === JSON.stringify(canonical(b));
export async function freshWakeOrigin(database) {
  if (!database || typeof database.databases !== "function")
    throw new Error("wake_storage_unavailable");
  let entries;
  try {
    entries = await database.databases();
  } catch {
    throw new Error("wake_storage_unavailable");
  }
  if (
    !Array.isArray(entries) ||
    entries.some((v) => !v || typeof v.name !== "string")
  )
    throw new Error("wake_storage_unavailable");
  if (entries.some((v) => v.name === "galaxy-royale"))
    throw new Error("wake_fresh_origin_required");
  return true;
}
export function requireWakeStage(config, role) {
  const meta = config?.wake_fixture;
  if (
    !config ||
    config.mode !== "wasm" ||
    config.profile_mode !== "owned" ||
    !config.scenarios?.includes(WAKE_SCENARIO) ||
    !["host", "guest"].includes(role) ||
    !/^[a-f0-9]{40}$/.test(config.source_commit ?? "") ||
    config.build_stamp?.head !== config.source_commit ||
    meta?.scenario !== WAKE_SCENARIO ||
    meta?.provenance !== "configured native fixture; NOT earned progression" ||
    meta?.fixture_source_commit !== "87d92f2cbf10c24dda585b47f05de2fad6fe178f"
  )
    throw new Error("wake_normal_wasm_required");
  for (const backend of ["webgpu", "webgl2"])
    if (
      !new RegExp(`^/wasm/${backend}-[a-f0-9]{64}/galaxy\\.js$`).test(
        config.builds?.[backend] ?? "",
      )
    )
      throw new Error("wake_normal_wasm_required");
  const entry = meta.profiles?.[role];
  if (
    entry?.path !== `fixtures/configured-wake/${role}-profile.json` ||
    !/^[a-f0-9]{64}$/.test(entry.sha256 ?? "") ||
    !uint(entry.seed) ||
    entry.seed === 0 ||
    !Number.isInteger(entry.bytes) ||
    entry.bytes < 1 ||
    entry.bytes > 65_536
  )
    throw new Error("wake_fixture_mismatch");
  return entry;
}
export function requireWakeProfile(role, value, normalize, entry) {
  const profile = normalize(value);
  const identity =
    role === "host" ? ["dora", "pan", "renko"] : ["doro", "hugo", "pan"];
  if (
    !profile ||
    !sameValue(profile, value) ||
    profile.schema_version !== 7 ||
    profile.seed !== entry.seed ||
    !sameValue(
      [
        profile.identity?.kind,
        profile.identity?.corporation,
        profile.identity?.sub_corporation,
      ],
      identity,
    ) ||
    !sameValue(profile.campaign.abilities[5], { blueprint: true, rank: 1 }) ||
    profile.campaign.selected_ship !== 0 ||
    profile.campaign.loadouts[0].slots[2] !== 5
  )
    throw new Error("wake_fixture_mismatch");
  return profile;
}
export function requireWakeGalaxy(galaxy) {
  if (
    !galaxy ||
    [
      "co_op_batch",
      "co_op_replica",
      "profile",
      "hud",
      "step",
      "set_paused",
      "cancel_course",
      "resize",
      "set_ui_mode",
      "set_shake",
      "set_reduced_motion",
      "free",
    ].some((name) => typeof galaxy[name] !== "function")
  )
    throw new Error("wake_normal_wasm_required");
}

export class WakeObservation {
  // Captured counts describe the last native full set (guest: after commit).
  // Receiver-clock visual ageing stays native and is not inferred here.
  constructor(role) {
    if (!["host", "guest"].includes(role)) throw new Error("wake_context");
    this.role = role;
    this.current = this.pending = null;
    this.session = null;
    this.sessionOwners = [false, false];
    this.leaving = null;
    this.value = {
      state: "idle",
      reason: null,
      configured_not_earned: true,
      fresh_origin: null,
      fixture_valid: null,
      native_export_match: null,
      native_pages: 0,
      complete_commits: 0,
      native_render_frames: 0,
      last_native_pixels: 0,
      positive_host: false,
      positive_guest: false,
      captured_points: 0,
      captured_host: false,
      captured_guest: false,
      peak_points: 0,
      peak_per_owner: 0,
      expired: false,
      retreated: false,
      stop_tick: null,
      last_birth_tick: null,
      last_complete_tick: null,
      removed_owner: null,
      local_view_ended: false,
      reduced_motion: false,
      visual_review: "pending",
    };
  }
  fail(reason) {
    this.value.state = "inconclusive";
    this.value.reason = wakeReasons.has(reason) ? reason : "wake_context";
    throw new Error(this.value.reason);
  }
  stage(header) {
    const s = header?.snapshot;
    if (
      !uint(header?.acquisition) ||
      !s ||
      !uint(s.tick) ||
      !uint(s.revision) ||
      s.progression !== false ||
      !Array.isArray(s.participants) ||
      s.participants.length > 2 ||
      !/^[a-f0-9]{32}$/.test(s.session ?? "") ||
      !exact(s.wake_fx, ["points"]) ||
      !uint(s.wake_fx.points) ||
      s.wake_fx.points > 24 ||
      new Set(s.participants.map((m) => m.slot)).size !==
        s.participants.length ||
      new Set(s.participants.map((m) => m.id)).size !== s.participants.length ||
      s.participants.some(
        (m) =>
          !uint(m.slot) ||
          m.slot > 1 ||
          !uint(m.epoch) ||
          m.epoch === 0 ||
          !/^[a-f0-9]{32}$/.test(m.id ?? "") ||
          !Array.isArray(m.position) ||
          m.position.length !== 2 ||
          m.position.some((n) => !Number.isFinite(n)),
      )
    )
      this.fail("wake_context");
    if (this.session !== s.session) {
      this.value.last_birth_tick =
        this.value.stop_tick =
        this.value.last_complete_tick =
          null;
      this.current = null;
      this.leaving = null;
      this.session = s.session;
      this.sessionOwners = [false, false];
      if (this.value.state !== "inconclusive") this.value.state = "idle";
    }
    this.pending = {
      header: structuredClone(header),
      points: s.wake_fx.points === 0 ? [] : null,
    };
    if (this.role === "host" && s.wake_fx.points === 0) this.complete();
  }
  page(acquisition, page) {
    const p = this.pending,
      s = p?.header.snapshot;
    if (
      !p ||
      p.header.acquisition !== acquisition ||
      p.points !== null ||
      !exact(page, ["session", "tick", "revision", "points"]) ||
      page.session !== s.session ||
      page.tick !== s.tick ||
      page.revision !== s.revision ||
      !Array.isArray(page.points) ||
      page.points.length !== s.wake_fx.points
    )
      this.fail("wake_page_mismatch");
    const members = new Map(s.participants.map((m) => [m.slot, m]));
    const counts = new Map();
    let previous = [-1, -1];
    for (const point of page.points) {
      const member = members.get(point.slot);
      const count = (counts.get(point.slot) ?? 0) + 1;
      if (
        !exact(point, ["slot", "epoch", "born_tick", "position", "life"]) ||
        !member ||
        point.epoch !== member.epoch ||
        !uint(point.born_tick) ||
        point.born_tick > s.tick ||
        count > 12 ||
        point.slot < previous[0] ||
        (point.slot === previous[0] && point.born_tick <= previous[1]) ||
        !Array.isArray(point.position) ||
        point.position.length !== 2 ||
        point.position.some(
          (v) =>
            !Number.isFinite(v) ||
            Math.abs(v) > 1_000_014 ||
            Math.abs(Math.fround(v)) > 1_000_014,
        ) ||
        !Number.isFinite(point.life) ||
        point.life <= 0 ||
        point.life > 2 ||
        Math.fround(point.life) <= 0 ||
        Math.fround(point.life) > 2
      )
        this.fail("wake_page_mismatch");
      counts.set(point.slot, count);
      previous = [point.slot, point.born_tick];
    }
    p.points = structuredClone(page.points);
    this.value.native_pages++;
    if (this.role === "host") this.complete();
  }
  commit(acquisition, revision) {
    if (
      this.role !== "guest" ||
      this.pending?.header.acquisition !== acquisition ||
      this.pending.header.snapshot.revision !== revision ||
      this.pending.points === null
    )
      this.fail("wake_sequence");
    this.value.complete_commits++;
    this.complete();
  }
  complete() {
    const p = this.pending,
      s = p.header.snapshot,
      v = this.value;
    const counts = new Map();
    for (const point of p.points) {
      counts.set(point.slot, (counts.get(point.slot) ?? 0) + 1);
      v.last_birth_tick = Math.max(v.last_birth_tick ?? 0, point.born_tick);
    }
    v.positive_host ||= counts.has(0);
    v.positive_guest ||= counts.has(1);
    this.sessionOwners[0] ||= counts.has(0);
    this.sessionOwners[1] ||= counts.has(1);
    v.captured_points = p.points.length;
    v.captured_host = counts.has(0);
    v.captured_guest = counts.has(1);
    v.peak_points = Math.max(v.peak_points, p.points.length);
    v.peak_per_owner = Math.max(v.peak_per_owner, 0, ...counts.values());
    v.last_complete_tick = s.tick;
    if (p.points.length && v.state === "idle") v.state = "live";
    if (
      v.stop_tick !== null &&
      !p.points.length &&
      s.tick - Math.max(v.stop_tick, v.last_birth_tick ?? 0) >= 121
    ) {
      if (v.state === "stopping") {
        v.expired = true;
        v.state = "expired";
      }
      if (
        v.state === "retreating" &&
        s.participants.length === 2 &&
        s.participants.every((m) => Math.hypot(...m.position) <= 128)
      ) {
        v.retreated = true;
        v.state = "retreated";
      }
    }
    if (
      this.leaving &&
      !s.participants.some(
        (m) => m.id === this.leaving.id && m.epoch === this.leaving.epoch,
      )
    )
      v.removed_owner = true;
    this.current = p;
    this.pending = null;
  }
  stopping(retreat = false) {
    if (
      !this.current ||
      this.value.state === "inconclusive" ||
      this.value.last_birth_tick === null ||
      !this.sessionOwners.every(Boolean)
    )
      this.fail("wake_sequence");
    this.value.stop_tick = this.value.last_complete_tick;
    this.value.state = retreat ? "retreating" : "stopping";
  }
  driving() {
    this.value.stop_tick = null;
    this.value.local_view_ended = false;
    this.value.state = "idle";
  }
  leavingOwner(id, epoch) {
    this.leaving = { id, epoch };
    this.value.removed_owner = false;
  }
  end() {
    this.current = this.pending = null;
    this.value.captured_points = 0;
    this.value.captured_host = this.value.captured_guest = false;
    this.value.local_view_ended = true;
    if (this.value.state !== "inconclusive") this.value.state = "ended";
  }
  frame(frame) {
    if (
      !frame ||
      !Array.isArray(frame.events) ||
      frame.events.length >= 128 ||
      frame.events.some((e) => e?.kind === "death") ||
      !uint(frame.pixels) ||
      frame.pixels === 0 ||
      frame.pixels > 2_000_000 ||
      ![frame.render_ms, frame.simulation_ms].every(
        (v) => Number.isFinite(v) && v >= 0 && v <= 60_000,
      )
    )
      this.fail("wake_native_frame");
    this.value.native_render_frames++;
    this.value.last_native_pixels = frame.pixels;
  }
  report() {
    return { ...this.value };
  }
}
export function validateWakeReport(value, role) {
  const fields = Object.keys(new WakeObservation(role).value);
  const flag = (v) => v === null || typeof v === "boolean";
  if (
    !exact(value, fields) ||
    value.configured_not_earned !== true ||
    value.visual_review !== "pending" ||
    ![
      "idle",
      "live",
      "stopping",
      "expired",
      "retreating",
      "retreated",
      "ended",
      "inconclusive",
    ].includes(value.state) ||
    (value.reason !== null && !wakeReasons.has(value.reason)) ||
    ![
      "fresh_origin",
      "fixture_valid",
      "native_export_match",
      "removed_owner",
    ].every((k) => flag(value[k])) ||
    ![
      "positive_host",
      "positive_guest",
      "captured_host",
      "captured_guest",
      "expired",
      "retreated",
      "local_view_ended",
      "reduced_motion",
    ].every((k) => typeof value[k] === "boolean") ||
    !["native_pages", "complete_commits", "native_render_frames"].every(
      (k) => uint(value[k]) && value[k] <= 1_000_000,
    ) ||
    !uint(value.last_native_pixels) ||
    value.last_native_pixels > 2_000_000 ||
    value.native_render_frames > 0 !== value.last_native_pixels > 0 ||
    !uint(value.captured_points) ||
    value.captured_points > 24 ||
    value.captured_points > 0 !==
      (value.captured_host || value.captured_guest) ||
    !uint(value.peak_points) ||
    value.peak_points > 24 ||
    !uint(value.peak_per_owner) ||
    value.peak_per_owner > 12 ||
    value.captured_points > value.peak_points ||
    value.peak_per_owner > value.peak_points ||
    ((value.positive_host || value.positive_guest) &&
      (value.native_pages === 0 || value.peak_points === 0)) ||
    !["stop_tick", "last_birth_tick", "last_complete_tick"].every(
      (k) => value[k] === null || uint(value[k]),
    ) ||
    (value.state === "expired" && !value.expired) ||
    (value.state === "retreated" && !value.retreated) ||
    (value.reason === null) !== (value.state !== "inconclusive") ||
    (role === "host" && value.complete_commits !== 0) ||
    (role === "guest" &&
      (value.positive_host || value.positive_guest) &&
      value.complete_commits === 0)
  )
    throw new Error("Invalid configured native Wake observations.");
  return value;
}
