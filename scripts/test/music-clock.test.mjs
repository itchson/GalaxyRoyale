/** Pure timing checks against integrated beat durations; not browser DSP. */
import assert from "node:assert/strict";
import { test } from "node:test";
import { MusicClock } from "../../apps/game/src/music-clock.ts";

const close = (actual, expected, tolerance = 1e-8) =>
  assert(Math.abs(actual - expected) < tolerance, `${actual} vs ${expected}`);
test("120-to-166 ramp retains phase and quarter grid with finite monotonic note times", () => {
  const clock = new MusicClock(2, 120);
  const boundary = clock.retarget(2.37, 166, 4);
  close(boundary.beat, 1);
  close(boundary.time, 2.5);
  close(clock.timeAtBeat(0.5), 2.25);
  let previous = clock.timeAtBeat(1);
  for (let i = 1; i <= 16; i++) {
    const beat = 1 + i / 4,
      time = clock.timeAtBeat(beat);
    assert(time > previous);
    close(clock.beatAtTime(time), beat);
    previous = time;
  }
  // Independent midpoint integration of dt/db = 60 / BPM, not a copy of the
  // clock's logarithmic primitive. Four authored ramp beats stay four beats.
  let expected = 0;
  for (let i = 0; i < 20000; i++)
    expected += ((60 / (120 + (46 * (i + 0.5)) / 20000)) * 4) / 20000;
  close(clock.timeAtBeat(5) - boundary.time, expected, 1e-8);
  close(clock.bpmAtBeat(5), 166);
  close(clock.timeAtBeat(6) - clock.timeAtBeat(5), 60 / 166);
});
test("two-beat release hold and re-chase replace pending future ramp without a phase jump", () => {
  const clock = new MusicClock(0, 166);
  const release = clock.retarget(0.9, 120, 8, 2);
  close(release.beat, 5);
  const beatBefore = clock.beatAtTime(1.05),
    timeBefore = clock.timeAtBeat(3);
  const chase = clock.retarget(1.05, 166, 4);
  close(chase.beat, 3);
  close(clock.beatAtTime(1.05), beatBefore);
  close(clock.timeAtBeat(3), timeBefore);
  close(clock.bpmAtBeat(20), 166, 1e-9);
  assert.equal(clock.diagnostics(1.05).targetBpm, 166);
});
test("retarget during a slowing ramp starts from current boundary tempo and preserves heard timing", () => {
  const clock = new MusicClock(0, 166);
  clock.retarget(0, 120, 8);
  const heard = clock.timeAtBeat(2),
    current = clock.timeAtBeat(3.4);
  const from = clock.bpmAtBeat(4),
    expectedBoundary = clock.timeAtBeat(4);
  const change = clock.retarget(current, 166, 4);
  close(change.beat, 4);
  close(change.time, expectedBoundary);
  close(clock.timeAtBeat(2), heard);
  close(clock.bpmAtBeat(4), from);
  close(clock.bpmAtBeat(8), 166);
  close(clock.beatAtTime(current), 3.4);
});
test("pause/resume reanchors the existing beat and pending ramp instead of restarting the phrase", () => {
  const clock = new MusicClock(0, 169.275);
  clock.retarget(1, 120, 8);
  const beat = clock.beatAtTime(1.3),
    bpm = clock.bpmAtBeat(beat);
  const remaining = clock.timeAtBeat(12) - clock.timeAtBeat(beat);
  clock.reanchor(50, beat);
  close(clock.beatAtTime(49.955), beat, 1e-9);
  close(clock.beatAtTime(50), beat);
  close(clock.bpmAtBeat(beat), bpm);
  close(clock.timeAtBeat(12) - 50, remaining);
});
test("long runs bound history and reject nonfinite tempo inputs before they reach audio parameters", () => {
  const clock = new MusicClock(0, 120);
  let time = 0;
  for (let i = 0; i < 200; i++) {
    time += 5;
    clock.retarget(time, i % 2 ? 120 : 166, 4);
    const state = clock.diagnostics(time);
    assert(
      state.segments <= 16 &&
        Number.isFinite(state.beat) &&
        Number.isFinite(state.bpm),
    );
    const beat = state.beat;
    close(clock.beatAtTime(clock.timeAtBeat(beat + 1)), beat + 1);
  }
  for (const bpm of [0, -1, NaN, Infinity])
    assert.throws(() => clock.retarget(time, bpm, 4), RangeError);
  assert.throws(() => clock.retarget(time, 120, NaN), RangeError);
  assert.throws(() => new MusicClock(0, NaN), RangeError);
});
