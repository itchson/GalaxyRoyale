import assert from "node:assert/strict";
import { test } from "node:test";
import { MusicClock } from "./inputs/apps/game/src/music-clock.ts";
import {
  FractionalEventScheduler,
  referenceBeat,
} from "./fractional-event-scheduler.ts";

const close = (actual, expected, tolerance = 1e-9) =>
  assert(Math.abs(actual - expected) <= tolerance, `${actual} vs ${expected}`);
const limits = (fromTime, untilTime, overrides = {}) => ({
  fromTime,
  untilTime,
  maxWindows: 16,
  maxEvents: 128,
  ...overrides,
});
const event = (onsetBeat, durationBeats = 0.13, payload = `${onsetBeat}`) => ({
  onsetBeat,
  endBeat: onsetBeat + durationBeats,
  payload,
});
const key = (occurrence) => `${occurrence.loop}:${occurrence.eventIndex}`;
// An independent midpoint quadrature, not the MusicClock logarithmic primitive.
const integral = (from, to, bpmAtBeat, count = 40000) => {
  let time = 0;
  for (let i = 0; i < count; i++)
    time +=
      ((to - from) * 60) /
      bpmAtBeat(from + ((i + 0.5) * (to - from)) / count) /
      count;
  return time;
};

test("42.68 source seconds retain their fractional period at 169.275 BPM", () => {
  const period = referenceBeat(42.68, 169.275);
  close(period, 120.41095);
  assert.notEqual(period, Math.round(period * 4) / 4);
  const clock = new MusicClock(2, 169.275);
  close(clock.timeAtBeat(period), 44.68);
  close(clock.timeAtBeat(referenceBeat(0.071, 169.275)), 2.071);
  for (const [seconds, bpm] of [
    [-1, 169.275],
    [Infinity, 169.275],
    [1, 0],
    [1, NaN],
  ])
    assert.throws(() => referenceBeat(seconds, bpm), RangeError);
});

test("source onset and end seconds map independently without rounding or truncating a long gate", () => {
  const clock = new MusicClock(2, 169.275);
  const onsetBeat = referenceBeat(0.071, 169.275);
  const endBeat = referenceBeat(0.219, 169.275);
  const scheduler = new FractionalEventScheduler(
    referenceBeat(42.68, 169.275),
    [{ onsetBeat, endBeat, payload: "timing-only" }],
  );
  const result = scheduler.drain(clock, limits(2, 2.18));
  assert.equal(result.events.length, 1);
  assert.equal(result.events[0].onsetBeat, onsetBeat);
  assert.equal(result.events[0].endBeat, endBeat);
  close(result.events[0].onsetTime, 2.071);
  close(result.events[0].endTime, 2.219);
  assert(result.events[0].endTime > 2.18);
});

test("half-open quarter windows preserve fractional onsets, sorted ties and input indices", () => {
  const clock = new MusicClock(0, 120);
  const input = [
    event(0.25, 0.17, "boundary-b"),
    event(0.249999, 0.11, "inside"),
    event(0, 0.04, "origin"),
    event(0.25, 0.19, "boundary-a"),
  ];
  const before = JSON.stringify(input);
  const scheduler = new FractionalEventScheduler(3.37, input);
  const first = scheduler.drain(clock, limits(0, clock.timeAtBeat(0.25)));
  assert.deepEqual(
    first.events.map((note) => note.payload),
    ["origin", "inside"],
  );
  assert.deepEqual(
    first.events.map((note) => note.eventIndex),
    [2, 1],
  );
  close(first.events[1].onsetTime, 0.1249995);
  assert.deepEqual(
    first.events.map((note) => note.windowStep),
    [0, 0],
  );
  const second = scheduler.drain(clock, limits(0.125, 0.3));
  assert.deepEqual(
    second.events.map((note) => note.payload),
    ["boundary-b", "boundary-a"],
  );
  assert.deepEqual(
    second.events.map((note) => note.windowStep),
    [1, 1],
  );
  assert.equal(scheduler.drain(clock, limits(0.125, 0.3)).events.length, 0);
  assert.equal(JSON.stringify(input), before);
});

test("source loop crossing is neither a bar reset nor a fixed quarter-grid alias", () => {
  const clock = new MusicClock(0, 169.275);
  const period = referenceBeat(42.68, 169.275);
  const scheduler = new FractionalEventScheduler(period, [
    event(period - 0.031, 0.24, "tail"),
    event(0, 0.08, "head"),
    event(0.07, 0.11, "reply"),
  ]);
  const result = scheduler.drain(
    clock,
    limits(clock.timeAtBeat(period - 0.06), clock.timeAtBeat(period + 0.15)),
  );
  assert.deepEqual(
    result.events.map((note) => [note.payload, note.loop]),
    [
      ["tail", 0],
      ["head", 1],
      ["reply", 1],
    ],
  );
  close(result.events[1].onsetBeat, period);
  close(result.events[1].onsetTime, 42.68);
  assert.equal(result.events[0].windowStep, 481);
  assert.equal(result.events[1].windowStep, 481);
  close(result.events[0].endBeat, period + 0.209);
  assert(result.events[0].endTime > result.events[1].onsetTime);
});

test("ramp onset and end are each mapped through the real clock and independent integration", () => {
  const clock = new MusicClock(0, 120);
  clock.retarget(0, 166, 4);
  const scheduler = new FractionalEventScheduler(120.41095, [
    event(1.137, 2.628),
  ]);
  const result = scheduler.drain(
    clock,
    limits(clock.timeAtBeat(1), clock.timeAtBeat(1.6)),
  );
  const note = result.events[0];
  assert.equal(result.events.length, 1);
  close(note.onsetBeat, 1.137);
  close(note.endBeat, 3.765);
  close(
    note.onsetTime,
    integral(0, 1.137, (beat) => 120 + (46 * beat) / 4),
  );
  close(
    note.endTime,
    integral(0, 3.765, (beat) => 120 + (46 * beat) / 4),
  );
  assert(
    Math.abs(
      note.endTime - note.onsetTime - (2.628 * 60) / clock.bpmAtBeat(1.137),
    ) > 0.05,
  );
});

test("a note ending after a release hold and ramp retains its authored end beat", () => {
  const clock = new MusicClock(0, 166);
  const change = clock.retarget(0.9, 120, 8, 2);
  assert.equal(change.beat, 5);
  const scheduler = new FractionalEventScheduler(120.41095, [
    event(5.37, 11.11),
  ]);
  const result = scheduler.drain(
    clock,
    limits(clock.timeAtBeat(5.1), clock.timeAtBeat(5.6)),
  );
  const note = result.events[0];
  close(note.endBeat, 16.48);
  const before = (5 * 60) / 166;
  close(
    note.onsetTime,
    before + integral(5, 5.37, (beat) => 166 - (46 * (beat - 5)) / 8),
  );
  close(
    note.endTime,
    before +
      integral(5, 13, (beat) => 166 - (46 * (beat - 5)) / 8) +
      (3.48 * 60) / 120,
  );
});

test("a fractional loop crosses a tempo change with shared beat phase intact", () => {
  const period = referenceBeat(42.68, 169.275);
  const clock = new MusicClock(0, 169.275);
  const change = clock.retarget(clock.timeAtBeat(118.2), 120, 8);
  assert.equal(change.beat, 119);
  const scheduler = new FractionalEventScheduler(period, [event(0, 0.41)]);
  const result = scheduler.drain(
    clock,
    limits(clock.timeAtBeat(period - 0.12), clock.timeAtBeat(period + 0.12)),
  );
  assert.equal(result.events.length, 1);
  assert.equal(result.events[0].loop, 1);
  close(result.events[0].onsetBeat, period);
  close(result.events[0].onsetTime, clock.timeAtBeat(period));
  assert(result.events[0].onsetTime > 42.68);
});

test("overlapping lookahead drains match an independent corpus with no duplicate occurrences", () => {
  const period = 1.37;
  const input = [
    event(0.071),
    event(1.319, 0.4),
    event(0),
    event(0.071, 0.27),
    event(0.249999),
  ];
  const clock = new MusicClock(0, 120);
  const scheduler = new FractionalEventScheduler(period, input);
  const observed = [];
  let lastHorizon = 0;
  for (let tick = 0; tick < 101; tick++) {
    const now = tick * 0.025;
    lastHorizon = now + 0.18;
    const result = scheduler.drain(clock, limits(now, lastHorizon));
    assert.equal(result.skippedPast, 0);
    assert.equal(result.reason, "horizon");
    assert(
      result.events.every(
        (note) => note.onsetTime >= now && note.onsetTime < lastHorizon,
      ),
    );
    observed.push(...result.events);
  }
  const expected = [];
  for (let loop = 0; loop < 6; loop++)
    input.forEach((note, eventIndex) => {
      const onsetBeat = loop * period + note.onsetBeat;
      if (clock.timeAtBeat(onsetBeat) < lastHorizon)
        expected.push({ loop, eventIndex, onsetBeat });
    });
  expected.sort(
    (a, b) => a.onsetBeat - b.onsetBeat || a.eventIndex - b.eventIndex,
  );
  assert.deepEqual(observed.map(key), expected.map(key));
  assert.equal(new Set(observed.map(key)).size, observed.length);
});

test("stall skips stale onsets but keeps the future fraction of the current quarter window", () => {
  const clock = new MusicClock(0, 120);
  const scheduler = new FractionalEventScheduler(32, [
    event(0),
    event(7.01, 0.1, "stale"),
    event(7.126, 0.1, "current-fraction"),
  ]);
  scheduler.drain(clock, limits(0, 0.18));
  const result = scheduler.drain(
    clock,
    limits(clock.timeAtBeat(7.123), clock.timeAtBeat(7.22)),
  );
  assert.equal(result.skippedPast, 1);
  assert.deepEqual(
    result.events.map((note) => note.payload),
    ["current-fraction"],
  );
  assert.equal(result.events[0].windowStep, 28);
  assert(result.events[0].onsetTime >= clock.timeAtBeat(7.123));
});

test("a partial quarter remains resumable without scheduling beyond the exact time horizon", () => {
  const clock = new MusicClock(0, 120);
  const scheduler = new FractionalEventScheduler(3.37, [
    event(0.07),
    event(0.11),
    event(0.18),
  ]);
  const first = scheduler.drain(clock, limits(0, 0.04));
  assert.deepEqual(
    first.events.map((note) => note.onsetBeat),
    [0.07],
  );
  assert.equal(first.nextWindowStep, 0);
  const second = scheduler.drain(clock, limits(0, 0.065));
  assert.deepEqual(
    second.events.map((note) => note.onsetBeat),
    [0.11],
  );
  assert.equal(scheduler.drain(clock, limits(0, 0.065)).events.length, 0);
  assert.deepEqual(
    scheduler.drain(clock, limits(0, 0.1)).events.map((note) => note.onsetBeat),
    [0.18],
  );
});

test("actual mapped onset is exclusive at the horizon and inclusive at the floor down to one ULP", () => {
  const adjacent = (value, upwards) => {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setFloat64(0, value);
    view.setBigUint64(0, view.getBigUint64(0) + (upwards ? 1n : -1n));
    return view.getFloat64(0);
  };
  const period = referenceBeat(42.68, 169.275);
  for (const [loop, offset] of [
    [0, 0],
    [0, 0.25],
    [1, 0],
    [11, 0.123456789],
  ]) {
    const clock = new MusicClock(0.045, 169.275);
    clock.retarget(clock.timeAtBeat(1.17), 120, 8);
    const scheduler = new FractionalEventScheduler(period, [
      event(offset, 0.17),
    ]);
    const beat = loop * period + offset;
    const time = clock.timeAtBeat(beat);
    scheduler.resetAtBeat(Math.max(0, beat - 0.01));
    const before = scheduler.drain(clock, limits(adjacent(time, false), time));
    assert.equal(before.events.length, 0);
    const on = scheduler.drain(clock, limits(time, adjacent(time, true)));
    assert.equal(on.events.length, 1, `loop ${loop}, offset ${offset}`);
    assert.equal(on.events[0].loop, loop);
    assert.equal(on.events[0].onsetTime, time);
    assert.equal(
      scheduler.drain(clock, limits(time, adjacent(time, true))).events.length,
      0,
    );
  }
});

test("event cap retains the rest of a simultaneous chord for the caller's capacity policy", () => {
  const clock = new MusicClock(0, 120);
  const scheduler = new FractionalEventScheduler(4, [
    event(0.05),
    event(0.05),
    event(0.05),
    event(0.05),
  ]);
  const first = scheduler.drain(clock, limits(0, 0.18, { maxEvents: 2 }));
  assert.equal(first.reason, "event-cap");
  assert.equal(first.nextWindowStep, 0);
  assert.deepEqual(
    first.events.map((note) => note.eventIndex),
    [0, 1],
  );
  const second = scheduler.drain(clock, limits(0, 0.18, { maxEvents: 2 }));
  assert.deepEqual(
    second.events.map((note) => note.eventIndex),
    [2, 3],
  );
  assert.equal(second.reason, "horizon");
  assert.equal(second.skippedPast, 0);
  assert.equal(
    scheduler.drain(clock, limits(0, 0.18, { maxEvents: 2 })).events.length,
    0,
  );
});

test("window work and returned event count remain bounded at an extreme actual clock tempo", () => {
  const clock = new MusicClock(0, 12000);
  const scheduler = new FractionalEventScheduler(
    1,
    Array.from({ length: 8 }, (_, i) => event(i / 8)),
  );
  const result = scheduler.drain(clock, limits(0, 0.18));
  assert.equal(result.reason, "window-cap");
  assert.equal(result.windowsUsed, 16);
  assert.equal(result.nextWindowStep, 16);
  assert.equal(result.events.length, 32);
  const second = scheduler.drain(clock, limits(0, 0.18));
  assert.equal(second.events.length, 32);
  assert.equal(second.nextWindowStep, 32);
  assert.equal(second.skippedPast, 0);
  assert.equal(new Set([...result.events, ...second.events].map(key)).size, 64);
});

test("even many loop crossings inside one quarter window stop at the event cap", () => {
  const clock = new MusicClock(0, 120);
  const scheduler = new FractionalEventScheduler(0.001, [event(0, 0.0005)]);
  const result = scheduler.drain(clock, limits(0, 0.18));
  assert.equal(result.reason, "event-cap");
  assert.equal(result.events.length, 128);
  assert.equal(result.windowsUsed, 1);
  assert.equal(result.events.at(-1).loop, 127);
  const resumed = scheduler.drain(clock, limits(0, 0.18));
  assert.equal(resumed.events[0].loop, 128);
  assert.equal(resumed.skippedPast, 0);
});

test("stall after capacity reports explicit stale-note loss rather than hiding it in the cap", () => {
  const clock = new MusicClock(0, 120);
  const scheduler = new FractionalEventScheduler(8, [
    ...Array.from({ length: 32 }, () => event(0)),
    event(3.13, 0.1, "future"),
  ]);
  const first = scheduler.drain(clock, limits(0, 0.18, { maxEvents: 2 }));
  assert.equal(first.reason, "event-cap");
  assert.equal(first.skippedPast, 0);
  const afterStall = scheduler.drain(clock, limits(1.56, 1.74));
  assert.equal(afterStall.skippedPast, 30);
  assert.deepEqual(
    afterStall.events.map((note) => note.payload),
    ["future"],
  );
});

test("explicit cancellation and reset remap future gates across a retarget without changing occurrences", () => {
  const clock = new MusicClock(0, 169.275);
  const scheduler = new FractionalEventScheduler(120.41095, [
    event(2.9, 0.13, "heard-side"),
    event(3.04, 2.17, "future"),
    event(3.21, 0.19, "future-b"),
  ]);
  const before = scheduler.drain(clock, limits(1, 1.18));
  const change = clock.retarget(1, 120, 8);
  assert.equal(change.beat, 3);
  const cancellation = before.events.filter(
    (note) => note.onsetBeat >= change.beat,
  );
  assert.deepEqual(
    cancellation.map((note) => note.payload),
    ["future", "future-b"],
  );
  scheduler.resetAtBeat(change.beat); // Caller cancellation is modeled above, not performed by the primitive.
  const after = scheduler.drain(clock, limits(1, 1.18));
  assert.deepEqual(after.events.map(key), cancellation.map(key));
  assert(after.events[0].endTime > cancellation[0].endTime);
  assert(after.events[0].onsetTime > cancellation[0].onsetTime);
  assert.equal(
    after.events.some((note) => note.payload === "heard-side"),
    false,
  );
});

test("title to Haven 120 to combat 166 cancels and reschedules the same fractional occurrences", () => {
  const clock = new MusicClock(0, 169.275);
  const scheduler = new FractionalEventScheduler(
    referenceBeat(42.68, 169.275),
    [
      event(0.31),
      event(2.37),
      event(3.07, 1.91),
      event(3.14, 0.27),
      event(5.63),
      event(6.01, 1.83),
      event(6.13, 0.27),
      event(8.33),
    ],
  );
  const first = scheduler.drain(clock, limits(0, 0.18));
  const heard = first.events[0];
  const leaveTime = clock.timeAtBeat(2.77);
  const beforeHaven = scheduler.drain(
    clock,
    limits(leaveTime, leaveTime + 0.18),
  );
  const phaseAtLeave = clock.beatAtTime(leaveTime);
  const haven = clock.retarget(leaveTime, 120, 8);
  assert.equal(haven.beat, 3);
  close(clock.beatAtTime(leaveTime), phaseAtLeave);
  assert.equal(clock.diagnostics(leaveTime).targetBpm, 120);
  const havenCancelled = beforeHaven.events.filter(
    (note) => note.onsetBeat >= haven.beat,
  );
  assert.deepEqual(
    havenCancelled.map((note) => note.onsetBeat),
    [3.07, 3.14],
  );
  scheduler.resetAtBeat(haven.beat);
  const inHaven = scheduler.drain(clock, limits(leaveTime, leaveTime + 0.18));
  assert.deepEqual(inHaven.events.map(key), havenCancelled.map(key));
  assert(inHaven.events[0].endTime > havenCancelled[0].endTime);
  close(clock.timeAtBeat(heard.onsetBeat), heard.onsetTime);

  const chaseTime = clock.timeAtBeat(5.82);
  const beforeCombat = scheduler.drain(
    clock,
    limits(chaseTime, chaseTime + 0.18),
  );
  const phaseAtChase = clock.beatAtTime(chaseTime);
  const combat = clock.retarget(chaseTime, 166, 4);
  assert.equal(combat.beat, 6);
  close(clock.beatAtTime(chaseTime), phaseAtChase);
  assert.equal(clock.diagnostics(chaseTime).targetBpm, 166);
  const combatCancelled = beforeCombat.events.filter(
    (note) => note.onsetBeat >= combat.beat,
  );
  assert.deepEqual(
    combatCancelled.map((note) => note.onsetBeat),
    [6.01, 6.13],
  );
  scheduler.resetAtBeat(combat.beat);
  const inCombat = scheduler.drain(clock, limits(chaseTime, chaseTime + 0.18));
  assert.deepEqual(inCombat.events.map(key), combatCancelled.map(key));
  assert(inCombat.events[0].endTime < combatCancelled[0].endTime);
  close(clock.timeAtBeat(heard.onsetBeat), heard.onsetTime);
  close(inCombat.events[0].onsetBeat, 6.01);
  close(inCombat.events[0].endBeat, 7.84);
});

test("a stationary audible long-note caller must retime the stored end beat after each ramp", () => {
  const clock = new MusicClock(0, 169.275);
  const onsetBeat = referenceBeat(0.071, 169.275);
  const endBeat = referenceBeat(12.37, 169.275);
  const scheduler = new FractionalEventScheduler(
    referenceBeat(42.68, 169.275),
    [{ onsetBeat, endBeat, payload: "long-gate-contract" }],
  );
  const [heard] = scheduler.drain(clock, limits(0, 0.18)).events;
  close(heard.onsetTime, 0.071);
  close(heard.endTime, 12.37);
  clock.retarget(0.8, 120, 8);
  const havenEnd = clock.timeAtBeat(heard.endBeat);
  assert(havenEnd > heard.endTime + 3);
  // Results are scheduling snapshots. Recomputing this end is caller policy;
  // the pure test creates no source, envelope, gameplay motion or AudioNode.
  assert.equal(heard.endBeat, endBeat);
  close(heard.endTime, 12.37);
  clock.retarget(clock.timeAtBeat(8.33), 166, 4);
  const combatEnd = clock.timeAtBeat(heard.endBeat);
  assert(combatEnd < havenEnd - 2);
  assert(combatEnd > clock.timeAtBeat(13));
  close(clock.beatAtTime(combatEnd), endBeat);
  close(clock.timeAtBeat(heard.onsetBeat), heard.onsetTime);
  assert.equal(heard.endBeat, endBeat);
});

test("reanchor keeps fractional loop phase after suspension when caller clears its future voices", () => {
  const clock = new MusicClock(0, 120);
  const scheduler = new FractionalEventScheduler(1.37, [
    event(0),
    event(0.071),
    event(0.41),
  ]);
  scheduler.drain(clock, limits(0, 0.18));
  const frozenBeat = 1.375;
  clock.reanchor(50, frozenBeat);
  scheduler.resetAtBeat(frozenBeat);
  const result = scheduler.drain(clock, limits(50, 50.18));
  assert.deepEqual(
    result.events.map((note) => [note.loop, note.eventIndex]),
    [[1, 1]],
  );
  close(result.events[0].onsetBeat, 1.441);
  close(result.events[0].onsetTime, 50.033);
});

test("lookahead before the real clock's initial anchor does not manufacture a negative onset", () => {
  const clock = new MusicClock(0.045, 169.275);
  const scheduler = new FractionalEventScheduler(120.41095, [event(0, 0.1)]);
  const result = scheduler.drain(clock, limits(0, 0.18));
  assert.equal(result.events.length, 1);
  close(result.events[0].onsetTime, 0.045);
  assert.equal(result.skippedPast, 0);
});

test("invalid timing and caller bounds fail before consuming the valid pending event", () => {
  for (const [period, notes] of [
    [0, []],
    [NaN, []],
    [1, [event(1)]],
    [1, [event(-0.1)]],
    [1, [event(0, 0)]],
    [1, [event(0, Infinity)]],
    [1, Array.from({ length: 4097 }, () => event(0))],
  ])
    assert.throws(
      () => new FractionalEventScheduler(period, notes),
      RangeError,
    );
  const clock = new MusicClock(0, 120);
  const scheduler = new FractionalEventScheduler(3.37, [event(0.07)]);
  for (const invalid of [
    limits(0, 0.51),
    limits(-1, 0),
    limits(0, Infinity),
    limits(0, 0.18, { maxEvents: 129 }),
    limits(0, 0.18, { maxWindows: 17 }),
    limits(0, 0.18, { maxWindows: 0 }),
    limits(0, 0.18, { maxEvents: 1.5 }),
  ])
    assert.throws(() => scheduler.drain(clock, invalid), RangeError);
  assert.throws(() => scheduler.resetAtBeat(NaN), RangeError);
  assert.throws(() => scheduler.resetAtBeat(-1), RangeError);
  assert.throws(
    () => scheduler.resetAtBeat(Number.MAX_SAFE_INTEGER),
    RangeError,
  );
  assert.deepEqual(
    scheduler
      .drain(clock, limits(0, 0.18))
      .events.map((note) => note.onsetBeat),
    [0.07],
  );
});

test("an empty score still has bounded window work and no fabricated notes", () => {
  const clock = new MusicClock(0, 12000);
  const scheduler = new FractionalEventScheduler(120.41095, []);
  const result = scheduler.drain(clock, limits(0, 0.18));
  assert.equal(result.events.length, 0);
  assert.equal(result.windowsUsed, 16);
  assert.equal(result.reason, "window-cap");
  assert.equal(result.skippedPast, 0);
});
