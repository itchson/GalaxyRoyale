// Actual source: apps/game/src/music-clock.ts; SHA256 12fff1dfe804ed4aaaa274418b74efa8604b0afd656ef718cb4e43930a475aa9
const EPSILON = 1e-9;
const seconds = (segment, beats) => {
    const ramp = Math.min(Math.max(0, beats), segment.beats);
    const slope = segment.beats ? (segment.to - segment.from) / segment.beats : 0;
    const during = Math.abs(slope) < EPSILON
        ? (60 * ramp) / segment.from
        : (60 / slope) * Math.log((segment.from + slope * ramp) / segment.from);
    return during + (60 * Math.max(0, beats - ramp)) / segment.to;
};
export class MusicClock {
    segments;
    minimumBeat = 0;
    minimumTime;
    constructor(time, bpm) {
        if (!Number.isFinite(time) || time < 0 || !Number.isFinite(bpm) || bpm <= 0)
            throw new RangeError("Music clock needs finite time and positive tempo");
        this.segments = [{ beat: 0, time, from: bpm, to: bpm, beats: 0 }];
        this.minimumTime = time;
    }
    atBeat(beat) {
        for (let i = this.segments.length - 1; i >= 0; i--)
            if (this.segments[i].beat <= beat + EPSILON)
                return this.segments[i];
        return this.segments[0];
    }
    timeAtBeat(beat) {
        const segment = this.atBeat(beat);
        return segment.time + seconds(segment, beat - segment.beat);
    }
    beatAtTime(time) {
        if (time <= this.minimumTime)
            return this.minimumBeat;
        let segment = this.segments[0];
        for (let i = this.segments.length - 1; i >= 0; i--)
            if (this.segments[i].time <= time + EPSILON) {
                segment = this.segments[i];
                break;
            }
        const elapsed = Math.max(0, time - segment.time);
        const rampTime = seconds(segment, segment.beats);
        if (elapsed >= rampTime)
            return (segment.beat + segment.beats + ((elapsed - rampTime) * segment.to) / 60);
        const slope = (segment.to - segment.from) / segment.beats;
        return (segment.beat +
            (Math.abs(slope) < EPSILON
                ? (elapsed * segment.from) / 60
                : (segment.from * Math.expm1((elapsed * slope) / 60)) / slope));
    }
    bpmAtBeat(beat) {
        const segment = this.atBeat(beat);
        const progress = segment.beats
            ? Math.min(1, Math.max(0, (beat - segment.beat) / segment.beats))
            : 1;
        return segment.from + (segment.to - segment.from) * progress;
    }
    /** Replace future ramps, retaining the boundary's beat, time and tempo. */
    retarget(time, bpm, beats, hold = 0) {
        if (!Number.isFinite(time) ||
            !Number.isFinite(bpm) ||
            bpm <= 0 ||
            !Number.isFinite(beats) ||
            beats < 0 ||
            !Number.isFinite(hold) ||
            hold < 0)
            throw new RangeError("Music ramp needs finite time, positive tempo and nonnegative beats");
        const beat = Math.ceil(this.beatAtTime(time) - EPSILON) + hold;
        const boundary = this.timeAtBeat(beat);
        const from = this.bpmAtBeat(beat);
        this.segments = this.segments.filter((segment) => segment.beat < beat - EPSILON);
        this.segments.push({ beat, time: boundary, from, to: bpm, beats });
        if (this.segments.length > 16)
            this.segments = this.segments.slice(-16);
        return { beat, time: boundary };
    }
    /** Retain phrase phase across AudioContext/browser suspension. */
    reanchor(time, beat) {
        const shift = time - this.timeAtBeat(beat);
        for (const segment of this.segments)
            segment.time += shift;
        this.minimumBeat = beat;
        this.minimumTime = time;
    }
    diagnostics(time) {
        const beat = this.beatAtTime(time), last = this.segments.at(-1);
        return {
            beat,
            bpm: this.bpmAtBeat(beat),
            targetBpm: last.to,
            rampStartBeat: last.beat,
            rampEndBeat: last.beat + last.beats,
            segments: this.segments.length,
        };
    }
}
