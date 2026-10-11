// QA observation only: consumes real native events/publications; never simulates shots.
export const fifthReasons = new Set([
  "fresh_save_required",
  "scenario_unavailable",
  "starter_mismatch",
  "fifth_sequence",
  "fifth_ambiguous_events",
  "fifth_queue_full",
  "fifth_native_context",
  "fifth_shot_not_retained",
  "fifth_wrong_pattern",
  "fifth_mismatch",
  "fifth_timeout",
  "fifth_not_ready",
]);
const close = (a, b) => Number.isFinite(a) && Math.abs(a - b) <= 0.0001;
const exact = (v, keys) =>
  v &&
  typeof v === "object" &&
  !Array.isArray(v) &&
  Object.keys(v).sort().join() === [...keys].sort().join();
export function starterSelection(scenario, role, loaded, owned) {
  if (
    !["default", "renko-fifth"].includes(scenario) ||
    !["host", "guest"].includes(role)
  )
    throw new Error("scenario_unavailable");
  if (scenario === "renko-fifth") {
    if (!owned) throw new Error("scenario_unavailable");
    if (loaded !== null) throw new Error("fresh_save_required");
  }
  return role === "host"
    ? ["dora", scenario === "renko-fifth" ? "renko" : "pan", "renko"]
    : ["doro", "hugo", "pan"];
}
export function validateFifthReport(value, role) {
  if (
    !exact(value, ["state", "reason", "primaries", "ordinals"]) ||
    ![
      "idle",
      "departing",
      "firing",
      "holding",
      "complete",
      "inconclusive",
    ].includes(value.state) ||
    (value.reason !== null && !fifthReasons.has(value.reason)) ||
    !Number.isInteger(value.primaries) ||
    value.primaries < 0 ||
    value.primaries > 10 ||
    (role === "guest" && value.primaries !== 0) ||
    !Array.isArray(value.ordinals) ||
    value.ordinals.length > 10
  )
    throw new Error("Invalid fifth-shot observations.");
  for (const [i, row] of value.ordinals.entries()) {
    const bonus = (i + 1) % 5 === 0;
    if (
      !exact(row, [
        "ordinal",
        "projectiles",
        "owner_matched",
        "epoch_matched",
        "damage_ratio",
        "speed_delta",
      ]) ||
      row.ordinal !== i + 1 ||
      row.projectiles !== (bonus ? 2 : 1) ||
      row.owner_matched !== true ||
      row.epoch_matched !== true ||
      (bonus
        ? !close(row.damage_ratio, 1.5) || !close(row.speed_delta, 70)
        : row.damage_ratio !== null || row.speed_delta !== null)
    )
      throw new Error("Invalid fifth-shot ordinal.");
  }
  if (
    (value.state === "inconclusive") !== (value.reason !== null) ||
    (role === "host" && value.ordinals.length > value.primaries) ||
    (value.state === "complete" &&
      (value.ordinals.length !== 10 ||
        (role === "host" && value.primaries !== 10)))
  )
    throw new Error("Invalid fifth-shot completion.");
  return value;
}
export class FifthShotObservation {
  #session;
  #owner;
  #epoch;
  #seen = new Set();
  #damage;
  constructor(role) {
    this.role = role;
    this.state = "idle";
    this.reason = null;
    this.primaries = 0;
    this.ordinals = [];
  }
  fail(reason) {
    this.reason = fifthReasons.has(reason) ? reason : "fifth_native_context";
    this.state = "inconclusive";
    throw new Error(this.reason);
  }
  start(snapshot) {
    const host = snapshot?.participants?.find((p) => p.slot === 0);
    if (
      !host ||
      host.identity?.corporation !== "renko" ||
      host.selected_ship !== 0 ||
      snapshot.progression !== false ||
      !/^[a-f0-9]{32}$/.test(host.id) ||
      !Number.isInteger(host.epoch)
    )
      this.fail("starter_mismatch");
    if (snapshot.projectiles.some((p) => p.owner === host.id))
      this.fail("fifth_not_ready");
    this.#session = snapshot.session;
    this.#owner = host.id;
    this.#epoch = host.epoch;
    this.#seen.clear();
    this.#damage = undefined;
    this.primaries = 0;
    this.ordinals = [];
    this.reason = null;
    this.state = "holding";
  }
  fire(ordinal) {
    if (
      this.role !== "host" ||
      this.state !== "holding" ||
      ordinal !== this.ordinals.length + 1 ||
      ordinal > 10
    )
      this.fail("fifth_sequence");
    this.state = "firing";
  }
  events(events) {
    if (this.state !== "firing") return false;
    if (!Array.isArray(events) || events.length >= 128)
      this.fail("fifth_queue_full");
    if (events.some((e) => e.kind === "death"))
      this.fail("fifth_native_context");
    const count = events.filter((e) => e.kind === "shot").length;
    if (count > 1) this.fail("fifth_ambiguous_events");
    if (!count) return false;
    this.primaries++;
    this.state = "holding";
    return true;
  }
  accept(snapshot, ordinal) {
    const host = snapshot?.participants?.find((p) => p.slot === 0);
    if (
      snapshot?.session !== this.#session ||
      host?.id !== this.#owner ||
      host?.epoch !== this.#epoch ||
      snapshot.progression !== false ||
      host.hull <= 0
    )
      this.fail("fifth_native_context");
    if (
      ordinal !== this.ordinals.length + 1 ||
      ordinal > 10 ||
      (this.role === "host" && this.primaries !== ordinal)
    )
      this.fail("fifth_sequence");
    const owned = snapshot.projectiles.filter((p) => p.owner === this.#owner);
    if (owned.some((p) => p.owner_epoch !== this.#epoch))
      this.fail("fifth_native_context");
    const shots = owned.filter((p) => !this.#seen.has(p.id));
    const bonus = ordinal % 5 === 0;
    if (shots.length < (bonus ? 2 : 1)) this.fail("fifth_shot_not_retained");
    if (shots.length !== (bonus ? 2 : 1)) this.fail("fifth_wrong_pattern");
    const sorted = [...shots].sort((a, b) => a.damage - b.damage),
      primary = sorted[0];
    this.#damage ??= primary.damage;
    if (
      !(primary.damage > 0) ||
      !close(primary.damage, this.#damage) ||
      shots.some((p) => p.remaining_hits !== 1)
    )
      this.fail("fifth_mismatch");
    let ratio = null,
      delta = null;
    if (bonus) {
      const second = sorted[1];
      ratio = second.damage / primary.damage;
      const dx = second.velocity[0] - primary.velocity[0],
        dy = second.velocity[1] - primary.velocity[1];
      delta = Math.hypot(dx, dy);
      const direction = [Math.cos(host.heading), Math.sin(host.heading)];
      // Core forward() uses heading directly. Common inherited velocity cancels.
      if (
        !close(ratio, 1.5) ||
        !close(delta, 70) ||
        !close(dx, direction[0] * 70) ||
        !close(dy, direction[1] * 70)
      )
        this.fail("fifth_mismatch");
    }
    for (const shot of owned) this.#seen.add(shot.id);
    this.ordinals.push({
      ordinal,
      projectiles: shots.length,
      owner_matched: true,
      epoch_matched: true,
      damage_ratio: ratio,
      speed_delta: delta,
    });
    this.state = ordinal === 10 ? "complete" : "holding";
    return this.report();
  }
  report() {
    return validateFifthReport(
      {
        state: this.state,
        reason: this.reason,
        primaries: this.primaries,
        ordinals: this.ordinals.map((row) => ({ ...row })),
      },
      this.role,
    );
  }
}
