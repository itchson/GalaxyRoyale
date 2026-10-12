/** Work-only timing prototype. Payloads contain no synthesized or inferred notes. */
import type { MusicClock } from "./inputs/apps/game/src/music-clock.ts";

export type FractionalEvent<T> = Readonly<{
  onsetBeat: number;
  /** End in the first loop's coordinates; it may extend beyond loopBeats. */
  endBeat: number;
  payload: T;
}>;
export type ScheduledEvent<T> = Readonly<{
  /** Original input index plus loop identifies one occurrence, even at a chord. */
  eventIndex: number;
  loop: number;
  windowStep: number;
  onsetBeat: number;
  endBeat: number;
  onsetTime: number;
  endTime: number;
  payload: T;
}>;
export type DrainBounds = Readonly<{
  /** Eligibility floor; caller may add its explicit AudioContext scheduling lead. */
  fromTime: number;
  /** Strict exclusive onset horizon, independent of the note's end time. */
  untilTime: number;
  maxWindows: number;
  maxEvents: number;
}>;
export type DrainResult<T> = Readonly<{
  events: readonly ScheduledEvent<T>[];
  reason: "horizon" | "event-cap" | "window-cap";
  windowsUsed: number;
  /** Explicit stale-onset skips only; capacity never advances past an event. */
  skippedPast: number;
  nextWindowStep: number;
}>;

type Clock = Pick<MusicClock, "timeAtBeat" | "beatAtTime">;
const MAX_SCORE_EVENTS = 4096;
const MAX_WINDOWS = 16;
const MAX_EVENTS = 128;
const MAX_LOOKAHEAD_SECONDS = 0.5;
const finite = (value: number): boolean => Number.isFinite(value);
const safeCount = (value: number): boolean =>
  Number.isSafeInteger(value) && value >= 0;

/** Reference-time conversion once at authoring time, never a runtime tempo scale. */
export function referenceBeat(seconds: number, referenceBpm: number): number {
  const beat = (seconds * referenceBpm) / 60;
  if (
    !finite(seconds) ||
    seconds < 0 ||
    !finite(referenceBpm) ||
    referenceBpm <= 0 ||
    !finite(beat)
  )
    throw new RangeError(
      "Reference timing requires finite seconds and positive BPM",
    );
  return beat;
}

export class FractionalEventScheduler<T> {
  private readonly events: readonly (FractionalEvent<T> & {
    eventIndex: number;
  })[];
  private ordinal = 0;
  private windowStep = 0;

  constructor(
    readonly loopBeats: number,
    events: readonly FractionalEvent<T>[],
  ) {
    if (
      !finite(loopBeats) ||
      loopBeats <= 0 ||
      events.length > MAX_SCORE_EVENTS
    )
      throw new RangeError(
        "Loop requires positive finite beats and at most 4096 events",
      );
    this.events = Object.freeze(
      events
        .map((event, eventIndex) => {
          if (
            !finite(event.onsetBeat) ||
            event.onsetBeat < 0 ||
            event.onsetBeat >= loopBeats ||
            !finite(event.endBeat) ||
            event.endBeat <= event.onsetBeat
          )
            throw new RangeError(
              "Event requires an in-loop onset and a later finite end beat",
            );
          return Object.freeze({ ...event, eventIndex });
        })
        .sort(
          (a, b) => a.onsetBeat - b.onsetBeat || a.eventIndex - b.eventIndex,
        ),
    );
  }

  private onset(ordinal: number): number {
    const loop = Math.floor(ordinal / this.events.length);
    return (
      loop * this.loopBeats +
      this.events[ordinal % this.events.length]!.onsetBeat
    );
  }

  private first(beat: number, atOrAfter: (onset: number) => boolean): number {
    if (!this.events.length) return 0;
    let loop = Math.max(0, Math.floor(beat / this.loopBeats));
    // An inverse-clock result may straddle a loop boundary by a floating ULP.
    // Membership uses the actual mapped onset, not that approximate inverse.
    if (
      loop > 0 &&
      atOrAfter((loop - 1) * this.loopBeats + this.events.at(-1)!.onsetBeat)
    )
      loop--;
    if (!safeCount(loop * this.events.length))
      throw new RangeError("Occurrence index exceeds safe integer range");
    let low = 0,
      high = this.events.length;
    while (low < high) {
      const middle = Math.floor((low + high) / 2);
      if (atOrAfter(loop * this.loopBeats + this.events[middle]!.onsetBeat))
        high = middle;
      else low = middle + 1;
    }
    const ordinal = loop * this.events.length + low;
    if (!safeCount(ordinal))
      throw new RangeError("Occurrence index exceeds safe integer range");
    return ordinal;
  }

  /** Caller must first cancel every previously scheduled occurrence at/after beat. */
  resetAtBeat(beat: number): void {
    const step = Math.floor(beat * 4);
    if (!finite(beat) || beat < 0 || !safeCount(step))
      throw new RangeError(
        "Reset requires nonnegative finite beat in the safe step range",
      );
    const ordinal = this.first(beat, (onset) => onset >= beat);
    this.ordinal = ordinal;
    this.windowStep = step;
  }

  drain(clock: Clock, bounds: DrainBounds): DrainResult<T> {
    const { fromTime, untilTime, maxWindows, maxEvents } = bounds;
    if (
      !finite(fromTime) ||
      fromTime < 0 ||
      !finite(untilTime) ||
      untilTime <= fromTime ||
      untilTime - fromTime > MAX_LOOKAHEAD_SECONDS ||
      !Number.isInteger(maxWindows) ||
      maxWindows < 1 ||
      maxWindows > MAX_WINDOWS ||
      !Number.isInteger(maxEvents) ||
      maxEvents < 1 ||
      maxEvents > MAX_EVENTS
    )
      throw new RangeError(
        "Drain requires a <=0.5s horizon, 1..16 windows and 1..128 events",
      );
    const floorBeat = clock.beatAtTime(fromTime);
    let floorStep = Math.floor(floorBeat * 4);
    if (!finite(floorBeat) || floorBeat < 0 || !safeCount(floorStep))
      throw new RangeError("Clock returned an invalid beat");
    if (floorStep > 0 && clock.timeAtBeat(floorStep / 4) > fromTime)
      floorStep--;
    // Reanchor can place the transport's beat origin later than fromTime.
    const first = this.first(
      floorBeat,
      (onset) => clock.timeAtBeat(onset) >= fromTime,
    );
    let ordinal = Math.max(this.ordinal, first);
    let step = Math.max(this.windowStep, floorStep);
    let skippedPast = ordinal - this.ordinal;
    let windowsUsed = 0;
    const events: ScheduledEvent<T>[] = [];
    const finish = (reason: DrainResult<T>["reason"]): DrainResult<T> => {
      this.ordinal = ordinal;
      this.windowStep = step;
      return { events, reason, windowsUsed, skippedPast, nextWindowStep: step };
    };

    while (windowsUsed < maxWindows) {
      if (clock.timeAtBeat(step / 4) >= untilTime) return finish("horizon");
      windowsUsed++;
      const endBeat = (step + 1) / 4;
      while (this.events.length && this.onset(ordinal) < endBeat) {
        const onsetBeat = this.onset(ordinal);
        const onsetTime = clock.timeAtBeat(onsetBeat);
        if (!finite(onsetTime))
          throw new RangeError("Clock returned a nonfinite onset time");
        if (onsetTime >= untilTime) return finish("horizon");
        if (onsetTime < fromTime) {
          ordinal++;
          skippedPast++;
          continue;
        }
        if (events.length === maxEvents) return finish("event-cap");
        const event = this.events[ordinal % this.events.length]!;
        const noteEndBeat =
          Math.floor(ordinal / this.events.length) * this.loopBeats +
          event.endBeat;
        const endTime = clock.timeAtBeat(noteEndBeat);
        if (!finite(endTime) || endTime <= onsetTime || !safeCount(ordinal + 1))
          throw new RangeError(
            "Clock returned an invalid event end or occurrence index",
          );
        events.push({
          eventIndex: event.eventIndex,
          loop: Math.floor(ordinal / this.events.length),
          windowStep: step,
          onsetBeat,
          endBeat: noteEndBeat,
          onsetTime,
          endTime,
          payload: event.payload,
        });
        ordinal++;
      }
      if (clock.timeAtBeat(endBeat) > untilTime) return finish("horizon");
      step++;
      if (!safeCount(step))
        throw new RangeError("Window step exceeds safe integer range");
    }
    return finish(
      clock.timeAtBeat(step / 4) >= untilTime ? "horizon" : "window-cap",
    );
  }
}
