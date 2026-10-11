/** Authored procedural score and effects: no samples, generated tracks, or network assets. */
import {
  advanceMix,
  audibleThemes,
  cleanState,
  newMix,
  QUIET_STATE,
  THEMES,
  type AudioState,
} from "./audio-state.ts";
export type { AudioState } from "./audio-state.ts";
type SoundEvent = { kind: string; x: number; y: number; intensity: number };
type Lane = "music" | "sfx";
type Voice = {
  lane: Lane;
  gain: GainNode;
  nodes: AudioNode[];
  sources: AudioScheduledSourceNode[];
  pending: number;
  end: number;
  released: boolean;
  persistent: boolean;
  cue: string | null;
  theme: number | null;
};
const FLOOR = 0.0001;
// Leave room for the score, warning and engine layers before the final compressor.
// The compressor catches stacked events; this trim is not a brick-wall limiter.
const CUE_LEVEL = 0.85;
const ENGINE_LEVEL = 0.24;
const ENGINE_NOTES = [45, 52] as const; // Quiet A–E fifth, shared score harmony.
// Original palm-muted synth string coloration, not a guitar recording/model.
const STRING_DRIVE = Float32Array.from(
  { length: 257 },
  (_, i) => Math.tanh((i / 128 - 1) * 2.1) / Math.tanh(2.1),
);
const clamp = (value: number): number =>
  Number.isFinite(value) ? Math.min(1, Math.max(0, value)) : 0;
const frequency = (note: number): number => 440 * 2 ** ((note - 69) / 12);
const SOUNDS = new Set([
  "mote-charge",
  "mote-pulse",
  "shot",
  "hit",
  "player-hit",
  "explosion",
  "death",
  "pickup",
  "boss",
  "boss-charge",
  "shield",
  "upgrade",
  "dock",
  "discovery",
  "champion",
  "royale-round",
  "credits",
  "ability-echo",
  "ability-nova",
  "ability-wake",
  "ability-royal",
  "ability-repair",
  ...[6, 7, 8, 9, 12, 13].map((i) => `ability-signature-${i}`),
]);

// Eight authored bars in A minor. Dock and combat share harmony and a 112 BPM pulse.
const CHORDS = [
  [45, 48, 52, 55],
  [41, 45, 48, 50],
  [48, 52, 55, 59],
  [43, 47, 50, 53],
  [45, 48, 52, 55],
  [41, 45, 48, 50],
  [50, 53, 57, 60],
  [40, 47, 52, 56],
] as const;
const BASS = [
  0,
  null,
  null,
  0,
  0,
  null,
  7,
  null,
  0,
  null,
  null,
  12,
  7,
  null,
  null,
  7,
] as const;

export class AudioEngine {
  private context: AudioContext | null = null;
  private master: GainNode | null = null;
  private music: GainNode | null = null;
  private sfx: GainNode | null = null;
  private musicWet: GainNode | null = null;
  private noise: AudioBuffer | null = null;
  private graph: AudioNode[] = [];
  private voices = new Set<Voice>();
  private timer: number | null = null;
  private nextNote = 0;
  private step = 0;
  private state: AudioState = { ...QUIET_STATE };
  private mix = newMix();
  private mixTime = 0;
  private engineVoice: Voice | null = null;
  private engineOscillators: OscillatorNode[] = [];
  private engineFilter: BiquadFilterNode | null = null;
  private engineDuckGain: GainNode | null = null;
  private engineDuckTarget = 1;
  private engineDuckDepth = 1;
  private engineDuckUntil = 0;
  private alarmVoice: Voice | null = null;
  private lifecycle = 0;
  private lastWake = -Infinity;
  private tailTimer: number | null = null;
  private tailVoice: Voice | null = null;
  private levels = [0.75, 0.4, 0.75];
  private paused = true;
  private disposed = false;

  /** Invoked by the launch gesture; the sole AudioContext creation path. */
  async unlock(): Promise<void> {
    if (this.disposed) return;
    if (!this.context) {
      const browser = window as Window & {
        webkitAudioContext?: typeof AudioContext;
      };
      const Constructor = window.AudioContext ?? browser.webkitAudioContext;
      if (!Constructor) return;
      try {
        this.context = new Constructor({ latencyHint: "interactive" });
        this.buildGraph(this.context);
        this.context.onstatechange = () => {
          if (this.context?.state === "running") {
            if (!this.paused) this.startScheduler();
            else {
              this.stopScheduler();
              // A late mobile resume must obey the newest stop intent. The
              // sole exception is the explicitly bounded existing death tail.
              if (this.tailTimer === null)
                void this.context.suspend().catch(() => undefined);
            }
          } else {
            this.stopScheduler();
            for (const voice of [...this.voices]) this.release(voice);
            this.mix.engine =
              this.mix.alarm =
              this.mix.combat =
              this.mix.boss =
                0;
            this.resetEngineDuck();
          }
        };
      } catch {
        this.dispose();
        return;
      }
    }
    await this.resume();
  }

  setLevels(master: number, music: number, sfx: number): void {
    this.levels = [clamp(master), clamp(music), clamp(sfx)];
    const context = this.context;
    if (!context || context.state === "closed") return;
    [this.master, this.music, this.sfx].forEach((node, index) => {
      node?.gain.setTargetAtTime(
        index === 1 && this.tailTimer !== null ? 0 : this.levels[index]!,
        context.currentTime,
        0.025,
      );
    });
  }

  /** Called from the authoritative post-step HUD; never creates a context. */
  update(state: AudioState): void {
    this.state = cleanState(state);
    this.advanceSound();
  }

  /** Bounded observational snapshot for the existing one-Hz performance UI. */
  diagnostics() {
    const scheduledThemes = new Set<number>();
    let music = 0,
      sfx = 0,
      continuous = 0;
    for (const voice of this.voices) {
      if (voice.lane === "music") music++;
      else sfx++;
      if (voice.persistent) continuous++;
      if (voice.theme !== null) scheduledThemes.add(voice.theme);
    }
    return {
      context: this.context?.state ?? "locked",
      enabled:
        !this.disposed && !this.paused && this.context?.state === "running",
      disposed: this.disposed,
      deathTail: this.tailTimer !== null,
      area: this.state.area,
      biome: this.state.biome,
      champion: this.state.champion,
      hostiles: this.state.hostiles,
      boss: this.state.boss,
      combat: this.mix.combat,
      targetCombat: this.state.combat,
      bossLayer: this.mix.boss,
      corporation: this.state.corporation ?? null,
      lowHull: this.mix.alarm,
      hull: this.state.hull,
      engine: this.mix.engine,
      engineDuck: this.engineDuckTarget,
      throttle: this.state.throttle,
      speed: this.state.speed,
      engines: this.state.engines,
      voices: { music, sfx, continuous },
      scheduledThemes: [...scheduledThemes].sort((a, b) => a - b),
      // Selection for future notes, not a measurement of currently audible DSP.
      themes: audibleThemes(this.mix).map(([index, weight]) => ({
        index,
        name: THEMES[index]!.name,
        weight,
      })),
    };
  }

  /** Older callers may quiet a refuge; distance alone never manufactures combat. */
  setDanger(_danger: number, safe: boolean): void {
    if (safe) this.update({ ...this.state, combat: 0 });
  }

  play(event: SoundEvent): void {
    const context = this.context;
    if (!context || context.state !== "running" || this.paused || this.disposed)
      return;
    if (!SOUNDS.has(event.kind)) return;
    const now = context.currentTime;
    if (event.kind === "ability-wake") {
      if (now - this.lastWake < 0.6) return;
      this.lastWake = now;
    }
    const voice = this.createVoice(
      "sfx",
      now,
      0,
      (0.75 + clamp(event.intensity) * 0.25) * CUE_LEVEL,
    );
    if (!voice) return;
    voice.cue = event.kind;
    const tone = (
      start: number,
      end: number,
      duration: number,
      type: OscillatorType = "square",
      gain = 0.14,
      delay = 0,
    ): void => {
      this.tone(voice, now + delay, start, end, duration, type, gain);
    };
    const noise = (
      duration: number,
      cutoff: number,
      gain: number,
      type: BiquadFilterType = "lowpass",
    ): void => {
      this.noiseBurst(voice, now, duration, cutoff, gain, type);
    };
    // Runtime retains its established hit contract: .9 is a player impact,
    // .25 an occupied target-cell impact. No guessed radial danger is involved.
    const kind =
      event.kind === "hit" && event.intensity >= 0.8
        ? "player-hit"
        : event.kind;
    switch (kind) {
      case "mote-charge":
        // Original short rising chirp; cancellation leaves no sustained tone.
        tone(330, 660, 0.09, "square", 0.045);
        tone(990, 1320, 0.075, "triangle", 0.055, 0.025);
        break;
      case "mote-pulse":
        // One dry discharge for the actual actor pulse, through the shared bus.
        tone(720, 180, 0.07, "square", 0.06);
        tone(92, 52, 0.095, "triangle", 0.07);
        noise(0.055, 1750, 0.035, "bandpass");
        break;
      case "shot":
        tone(960, 220, 0.075, "square", 0.065);
        tone(190, 100, 0.045, "triangle", 0.08);
        break;
      case "hit":
        noise(0.07, 2700, 0.075, "bandpass");
        tone(260, 95, 0.07, "triangle", 0.07);
        break;
      case "player-hit":
        noise(0.14, 2500, 0.22, "bandpass");
        tone(150, 65, 0.13, "triangle", 0.19);
        break;
      case "explosion":
        noise(0.46, 1100, 0.34);
        tone(110, 30, 0.45, "sawtooth", 0.14);
        break;
      case "death":
        noise(0.65, 1500, 0.3);
        tone(260, 35, 0.7, "sawtooth", 0.18);
        tone(130, 28, 0.7, "triangle", 0.22);
        break;
      case "pickup": {
        const note = [76, 79, 81][
          Math.min(2, Math.round(clamp(event.intensity) * 2))
        ]!;
        tone(frequency(note), frequency(note), 0.1, "triangle", 0.18);
        tone(
          frequency(note + 12),
          frequency(note + 12),
          0.15,
          "sine",
          0.12,
          0.07,
        );
        break;
      }
      case "boss":
        [45, 52, 57].forEach((note, index) =>
          tone(
            frequency(note),
            frequency(note),
            0.42,
            "sawtooth",
            0.09,
            index * 0.11,
          ),
        );
        noise(0.65, 420, 0.2);
        break;
      case "boss-charge":
        tone(180, 360, 0.11, "triangle", 0.09);
        tone(540, 720, 0.09, "square", 0.035, 0.03);
        break;
      case "shield":
        tone(650, 150, 0.23, "sine", 0.2);
        noise(0.16, 4200, 0.08, "highpass");
        break;
      case "upgrade":
        [69, 72, 76, 81].forEach((note, index) =>
          tone(
            frequency(note),
            frequency(note),
            0.25,
            "triangle",
            0.16,
            index * 0.075,
          ),
        );
        break;
      case "ability-echo":
        [0, 0.025, 0.05].forEach((delay, i) =>
          tone(620 + i * 160, 360 + i * 120, 0.07, "triangle", 0.055, delay),
        );
        break;
      case "ability-nova":
        tone(95, 290, 0.18, "sine", 0.14);
        tone(1200, 260, 0.13, "triangle", 0.045);
        noise(0.09, 1250, 0.06, "bandpass");
        break;
      case "ability-wake":
        tone(170, 245, 0.085, "triangle", 0.05);
        noise(0.065, 950, 0.025, "bandpass");
        break;
      case "ability-signature-6": // Dawnneedle: paired needle whistle.
        tone(1480, 680, 0.055, "triangle", 0.055);
        tone(1720, 860, 0.05, "square", 0.022, 0.015);
        break;
      case "ability-signature-7": // Bulwark: heavy broadside.
        tone(135, 48, 0.12, "square", 0.1);
        noise(0.08, 760, 0.09);
        break;
      case "ability-signature-8": // Guided missile lock and exhaust.
        tone(880, 1320, 0.08, "sine", 0.055);
        noise(0.11, 1400, 0.07, "bandpass");
        break;
      case "ability-signature-9": // Prism fork.
        tone(784, 392, 0.09, "triangle", 0.055);
        tone(988, 494, 0.09, "sine", 0.05);
        break;
      case "ability-signature-12": // Voidcoil seeker.
        tone(410, 110, 0.16, "sawtooth", 0.04);
        tone(820, 240, 0.12, "sine", 0.045, 0.02);
        break;
      case "ability-signature-13": // Sunspear long rail report.
        tone(1850, 360, 0.075, "square", 0.05);
        noise(0.045, 4500, 0.055, "highpass");
        break;
      case "ability-royal":
        [69, 76, 81].forEach((note, i) =>
          tone(
            frequency(note),
            frequency(note + 12),
            0.13,
            "triangle",
            0.045,
            i * 0.02,
          ),
        );
        break;
      case "ability-repair":
        tone(523, 659, 0.16, "sine", 0.05);
        tone(784, 988, 0.12, "triangle", 0.035, 0.04);
        break;
      case "discovery":
        tone(659, 784, 0.16, "sine", 0.1);
        tone(988, 1175, 0.2, "triangle", 0.04, 0.08);
        break;
      case "champion":
      case "royale-round":
      case "credits":
        [69, 72, 76, 81].forEach((note, i) =>
          tone(
            frequency(note),
            frequency(note),
            0.3,
            "triangle",
            0.1,
            i * 0.07,
          ),
        );
        break;
      case "dock":
        [57, 64, 69].forEach((note, index) =>
          tone(
            frequency(note),
            frequency(note),
            0.48,
            "sine",
            0.1,
            index * 0.06,
          ),
        );
        break;
      default:
        this.release(voice);
        return;
    }
    // Only an admitted real cue ducks the engine. Primary fire and ordinary
    // cell hits do not repeatedly pump the propulsion bed or the music bus.
    const important =
      kind === "player-hit" ||
      kind === "shield" ||
      kind === "explosion" ||
      kind === "death";
    const warning =
      kind === "boss" ||
      kind === "boss-charge" ||
      kind === "mote-charge" ||
      kind === "mote-pulse" ||
      kind === "champion" ||
      kind === "royale-round" ||
      (kind.startsWith("ability-") && kind !== "ability-wake");
    if (important || warning) {
      this.engineDuckDepth = Math.min(
        now < this.engineDuckUntil ? this.engineDuckDepth : 1,
        important ? 0.35 : 0.55,
      );
      this.engineDuckUntil = Math.max(
        this.engineDuckUntil,
        now + Math.min(0.4, Math.max(0.16, voice.end - now)),
      );
      this.updateEngineDuck(now);
    }
  }

  /** Death UI keeps only its already-created cue; it never creates audio. */
  finishDeath(): void {
    if (this.tailTimer !== null) return;
    const context = this.context;
    const death = [...this.voices]
      .filter((voice) => voice.cue === "death")
      .at(-1);
    if (!context || context.state !== "running" || !death || this.disposed) {
      this.suspend();
      return;
    }
    const epoch = ++this.lifecycle;
    this.paused = true;
    this.stopScheduler();
    // Echo output bypasses the dry music bus. Gate it too, so an earlier
    // delayed pad cannot continue under the sole allowed death effect.
    this.music?.gain.setTargetAtTime(0, context.currentTime, 0.008);
    this.musicWet?.gain.setTargetAtTime(0, context.currentTime, 0.008);
    this.mix.engine = this.mix.alarm = this.mix.combat = this.mix.boss = 0;
    this.resetEngineDuck();
    this.lastWake = -Infinity;
    for (const voice of [...this.voices])
      if (voice !== death) this.release(voice);
    this.tailVoice = death;
    const milliseconds = Math.min(
      800,
      Math.max(0, (death.end - context.currentTime) * 1000),
    );
    this.tailTimer = window.setTimeout(() => {
      if (epoch !== this.lifecycle || this.context !== context || !this.paused)
        return;
      this.tailTimer = null;
      if (this.tailVoice) this.release(this.tailVoice);
      this.tailVoice = null;
      if (context.state !== "closed")
        void context.suspend().catch(() => undefined);
    }, milliseconds);
  }

  private clearTail(): void {
    if (this.tailTimer !== null) window.clearTimeout(this.tailTimer);
    this.tailTimer = null;
    if (this.tailVoice) this.release(this.tailVoice);
    this.tailVoice = null;
  }

  suspend(): void {
    this.lifecycle++;
    this.clearTail();
    this.paused = true;
    this.stopScheduler();
    for (const voice of [...this.voices]) this.release(voice);
    this.mix.engine = this.mix.alarm = this.mix.combat = this.mix.boss = 0;
    this.resetEngineDuck();
    this.lastWake = -Infinity;
    if (this.context && this.context.state !== "closed")
      void this.context.suspend().catch(() => undefined);
  }

  async resume(): Promise<void> {
    if (this.disposed || !this.context || this.context.state === "closed")
      return;
    const epoch = ++this.lifecycle;
    this.clearTail();
    this.paused = false;
    try {
      await this.context.resume();
    } catch {
      if (epoch === this.lifecycle) this.paused = true;
      return;
    }
    if (
      epoch === this.lifecycle &&
      !this.disposed &&
      !this.paused &&
      this.context.state === "running"
    ) {
      this.setLevels(this.levels[0]!, this.levels[1]!, this.levels[2]!);
      this.musicWet?.gain.setTargetAtTime(
        0.12,
        this.context.currentTime,
        0.025,
      );
      this.startScheduler();
    }
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.lifecycle++;
    this.clearTail();
    this.paused = true;
    this.stopScheduler();
    for (const voice of [...this.voices]) this.release(voice);
    for (const node of this.graph) node.disconnect();
    this.graph = [];
    if (this.context) {
      this.context.onstatechange = null;
      if (this.context.state !== "closed")
        void this.context.close().catch(() => undefined);
    }
    this.context = null;
    this.master = null;
    this.music = null;
    this.sfx = null;
    this.musicWet = null;
    this.noise = null;
    this.resetEngineDuck();
  }

  private buildGraph(context: AudioContext): void {
    this.master = context.createGain();
    this.music = context.createGain();
    this.sfx = context.createGain();
    const limiter = context.createDynamicsCompressor();
    limiter.threshold.value = -12;
    limiter.knee.value = 6;
    limiter.ratio.value = 12;
    limiter.attack.value = 0.003;
    limiter.release.value = 0.18;
    // The compressor has attack overshoot. Bound the actual final samples,
    // leaving normal-level score/effects linear instead of lowering user buses.
    const ceiling = context.createWaveShaper();
    const curve = new Float32Array(4097);
    for (let i = 0; i < curve.length; i++) {
      const value = (i / (curve.length - 1)) * 2 - 1;
      const absolute = Math.abs(value);
      curve[i] =
        absolute <= 0.78
          ? value
          : Math.sign(value) *
            (0.78 + 0.15 * Math.tanh((absolute - 0.78) / 0.15));
    }
    ceiling.curve = curve;
    ceiling.oversample = "none";
    const warmth = context.createBiquadFilter();
    warmth.type = "lowpass";
    warmth.frequency.value = 5600;
    warmth.Q.value = 0.45;
    const echo = context.createDelay(1);
    echo.delayTime.value = (60 / 112) * 0.75;
    const feedback = context.createGain();
    feedback.gain.value = 0.16;
    const wet = context.createGain();
    this.musicWet = wet;
    wet.gain.value = 0.12;
    this.music.connect(warmth);
    warmth.connect(this.master);
    warmth.connect(echo);
    echo.connect(feedback);
    feedback.connect(echo);
    echo.connect(wet);
    wet.connect(this.master);
    this.sfx.connect(this.master);
    this.master.connect(limiter);
    limiter.connect(ceiling);
    ceiling.connect(context.destination);
    this.graph = [
      this.master,
      this.music,
      this.sfx,
      limiter,
      ceiling,
      warmth,
      echo,
      feedback,
      wet,
    ];
    this.noise = context.createBuffer(
      1,
      context.sampleRate,
      context.sampleRate,
    );
    const data = this.noise.getChannelData(0);
    let seed = 0x47524f59;
    for (let i = 0; i < data.length; i++) {
      seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
      data[i] = (seed / 0x80000000 - 1) * 0.7;
    }
    this.setLevels(this.levels[0]!, this.levels[1]!, this.levels[2]!);
  }

  private startScheduler(): void {
    if (this.timer !== null || !this.context || this.paused) return;
    this.nextNote = this.context.currentTime + 0.045;
    this.mixTime = this.context.currentTime;
    this.timer = window.setInterval(() => this.schedule(), 25);
    this.schedule();
  }
  private stopScheduler(): void {
    if (this.timer !== null) window.clearInterval(this.timer);
    this.timer = null;
  }
  private schedule(): void {
    const context = this.context;
    if (!context || context.state !== "running" || this.paused) return;
    this.advanceSound();
    if (this.nextNote < context.currentTime - 0.2)
      this.nextNote = context.currentTime + 0.02;
    const length = 60 / 112 / 4;
    while (this.nextNote < context.currentTime + 0.18) {
      this.scoreStep(this.nextNote, this.step++, length);
      this.nextNote += length;
    }
  }

  private scoreStep(time: number, step: number, length: number): void {
    const bar = Math.floor(step / 16) % CHORDS.length;
    const beat = step % 16;
    const chord = CHORDS[bar]!;
    const combat = this.mix.combat;
    for (const [index, weight] of audibleThemes(this.mix)) {
      const theme = THEMES[index]!;
      const space = this.mix.space * weight;
      if (beat === 0) {
        const pad = this.createVoice("music", time, length * 16, space);
        if (pad) {
          pad.theme = index;
          chord.forEach((note) =>
            this.tone(
              pad,
              time,
              frequency(note + 12),
              frequency(note + 12),
              length * 15.8,
              combat < 0.12 && theme.arp === "square" ? "triangle" : theme.arp,
              0.017 * (1 - combat * 0.5),
              0.22,
            ),
          );
        }
      }
      const voice = this.createVoice("music", time, 0.55, space);
      if (!voice) continue;
      voice.theme = index;
      if (beat % 4 === 0) {
        const note = chord[theme.order[Math.floor(beat / 4)]!]! + 24;
        this.tone(
          voice,
          time,
          frequency(note),
          frequency(note),
          length * 1.6,
          combat < 0.12 && theme.arp === "square" ? "triangle" : theme.arp,
          0.032 * (1 - combat * 0.35),
          0.016,
        );
      }
      const bass = BASS[beat];
      if (
        bass !== null &&
        bass !== undefined &&
        (combat > 0.25 || beat === 0 || beat === 8)
      )
        this.tone(
          voice,
          time,
          frequency(chord[0] + bass),
          frequency(chord[0] + bass),
          length * 1.15,
          combat < 0.12 && theme.bass === "square" ? "triangle" : theme.bass,
          0.026 + combat * 0.022,
        );
      const lead = theme.motif[(beat + (bar % 2) * 4) % 16];
      if (
        lead !== null &&
        lead !== undefined &&
        (combat > 0.08 || index >= 6 || beat % 4 === 0)
      )
        this.tone(
          voice,
          time,
          frequency(lead),
          frequency(lead),
          length * 1.35,
          combat < 0.12 && theme.lead === "square" ? "triangle" : theme.lead,
          0.011 + combat * 0.02,
          0.01,
        );
      if (voice.pending === 0) this.release(voice);
    }
    // One bounded shared power-chord voice: actual aggro, never biome/distance.
    // Boss counter-notes follow the separate real boss blend; all old theme
    // identities remain intact and at most two are selected for future notes.
    if (combat > 0.12 && beat % 2 === 0) {
      const guitar = this.createVoice("music", time, 0.24, combat * 0.8);
      if (guitar) {
        guitar.cue = "music-combat";
        const duration = length * (this.mix.boss > 0.1 ? 1.55 : 1.15);
        this.mutedString(
          guitar,
          time,
          frequency(chord[0] + 12),
          duration,
          0.026,
        );
        this.mutedString(
          guitar,
          time,
          frequency(chord[0] + 19),
          duration,
          0.014,
        );
        if (this.mix.boss > 0.02 && beat % 4 === 0)
          this.tone(
            guitar,
            time,
            frequency(chord[2] + 24),
            frequency(chord[2] + 24),
            length * 1.3,
            "triangle",
            this.mix.boss * 0.012,
            0.01,
          );
      }
    }
    // Percussion follows actual aggro, once per shared clock, independent of
    // how many theme identities currently crossfade.
    if (combat > 0.1) {
      const drum = this.createVoice("music", time, 0.16, 1);
      if (!drum) return;
      if (beat % 4 === 0)
        this.tone(drum, time, 105, 39, 0.13, "sine", combat * 0.11);
      if (beat === 4 || beat === 12)
        this.noiseBurst(drum, time, 0.1, 2100, combat * 0.055, "bandpass");
      if (combat > 0.4 && beat % 2 === 1)
        this.noiseBurst(drum, time, 0.032, 5200, combat * 0.018, "highpass");
      if (drum.pending === 0) this.release(drum);
    }
  }

  private advanceSound(): void {
    const context = this.context;
    if (!context || context.state !== "running" || this.paused || this.disposed)
      return;
    const now = context.currentTime;
    this.mix = advanceMix(this.mix, this.state, now - this.mixTime);
    this.mixTime = now;
    if (this.mix.engine > 0.008 && !this.engineVoice) {
      this.engineVoice = this.continuous("engine", now);
    }
    if (this.engineVoice) {
      this.engineVoice.gain.gain.setTargetAtTime(
        // Low-depth four-second breathing is continuous context-time motion,
        // not a fabricated beat clock or a repeatedly restarted sound sample.
        this.mix.engine *
          ENGINE_LEVEL *
          (0.97 + 0.03 * Math.sin((now * Math.PI) / 2)),
        now,
        0.18,
      );
      const bend = 1 + this.state.speed * 0.025 + this.state.throttle * 0.015;
      this.engineOscillators.forEach((source, i) =>
        source.frequency.setTargetAtTime(
          frequency(ENGINE_NOTES[i]!) * bend,
          now,
          0.24,
        ),
      );
      this.engineFilter?.frequency.setTargetAtTime(
        260 + this.state.throttle * 120 + this.state.speed * 60,
        now,
        0.18,
      );
      if (
        this.mix.engine < 0.002 &&
        (this.state.engines === 0 ||
          this.state.hull === 0 ||
          (this.state.throttle === 0 && this.state.speed === 0))
      )
        this.release(this.engineVoice);
    }
    if (this.mix.alarm > 0.01 && !this.alarmVoice)
      this.alarmVoice = this.continuous("alarm", now);
    if (this.alarmVoice) {
      // One source pair, with a two-beat envelope rather than overlapping
      // alert samples. Healing reduces the same voice smoothly to silence.
      const beat = 0.25 + 0.75 * Math.max(0, Math.sin(now * Math.PI * 4));
      this.alarmVoice.gain.gain.setTargetAtTime(
        this.mix.alarm * beat * 0.7,
        now,
        0.02,
      );
      if (this.mix.alarm < 0.002 && this.state.hull > 0.25)
        this.release(this.alarmVoice);
      if (this.state.hull === 0) this.release(this.alarmVoice);
    }
    this.updateEngineDuck(now);
  }

  private updateEngineDuck(now: number): void {
    if (now >= this.engineDuckUntil) this.engineDuckDepth = 1;
    const target = this.engineDuckDepth * (1 - this.mix.alarm * 0.4);
    if (Math.abs(target - this.engineDuckTarget) > 0.00001) {
      this.engineDuckGain?.gain.setTargetAtTime(
        target,
        now,
        target < this.engineDuckTarget ? 0.015 : 0.22,
      );
      this.engineDuckTarget = target;
    }
  }

  private resetEngineDuck(): void {
    this.engineDuckTarget = this.engineDuckDepth = 1;
    this.engineDuckUntil = 0;
  }

  private continuous(kind: "engine" | "alarm", time: number): Voice | null {
    const context = this.context;
    if (!context) return null;
    const voice = this.createVoice("sfx", time, 86400, 0, true);
    if (!voice) return null;
    let output: AudioNode = voice.gain;
    if (kind === "engine") {
      // A single lowpass shapes both tonal harmonics and the air bed; the
      // separate duck stage can lower propulsion without changing score gain.
      const filter = context.createBiquadFilter();
      filter.type = "lowpass";
      filter.frequency.value = 260;
      filter.Q.value = 0.5;
      const duck = context.createGain();
      duck.gain.setValueAtTime(this.engineDuckTarget, time);
      filter.connect(duck);
      duck.connect(voice.gain);
      voice.nodes.push(filter, duck);
      this.engineFilter = filter;
      this.engineDuckGain = duck;
      output = filter;
    }
    const oscillators: OscillatorNode[] = [];
    for (let i = 0; i < 2; i++) {
      const source = context.createOscillator();
      source.type = "sine";
      source.frequency.setValueAtTime(
        kind === "engine" ? frequency(ENGINE_NOTES[i]!) : i ? 660 : 440,
        time,
      );
      const envelope = context.createGain();
      envelope.gain.value =
        kind === "engine" ? (i ? 0.014 : 0.045) : i ? 0.06 : 0.095;
      source.connect(envelope);
      envelope.connect(output);
      voice.nodes.push(source, envelope);
      voice.sources.push(source);
      voice.pending++;
      source.onended = () => this.release(voice);
      source.start(time);
      oscillators.push(source);
    }
    if (kind === "engine" && this.noise) {
      const source = context.createBufferSource();
      source.buffer = this.noise;
      source.loop = true;
      const envelope = context.createGain();
      envelope.gain.value = 0.008;
      source.connect(envelope);
      envelope.connect(output);
      voice.nodes.push(source, envelope);
      voice.sources.push(source);
      voice.pending++;
      source.onended = () => this.release(voice);
      source.start(time);
      this.engineOscillators = oscillators;
    }
    return voice;
  }

  private mutedString(
    voice: Voice,
    time: number,
    pitch: number,
    duration: number,
    level: number,
  ): void {
    const context = this.context;
    if (!context || voice.released) return;
    const source = context.createOscillator();
    source.type = "triangle";
    source.frequency.setValueAtTime(pitch * 1.003, time);
    source.frequency.exponentialRampToValueAtTime(
      pitch,
      time + Math.min(0.045, duration),
    );
    const drive = context.createWaveShaper();
    drive.curve = STRING_DRIVE;
    drive.oversample = "none";
    const filter = context.createBiquadFilter();
    filter.type = "lowpass";
    filter.frequency.value = 1000 + this.mix.boss * 650;
    filter.Q.value = 0.5;
    source.connect(drive);
    drive.connect(filter);
    voice.nodes.push(drive, filter);
    this.attach(voice, source, time, duration, level, 0.005, filter);
  }

  private createVoice(
    lane: Lane,
    time: number,
    duration: number,
    level: number,
    persistent = false,
  ): Voice | null {
    const context = this.context;
    const bus = lane === "music" ? this.music : this.sfx;
    if (!context || !bus) return null;
    const active = [...this.voices].filter((voice) => voice.lane === lane);
    if (active.length >= (lane === "sfx" ? 16 : 32)) {
      if (lane === "music") return null;
      const transient = active.filter((voice) => !voice.persistent);
      if (!transient.length) return null;
      this.release(transient.reduce((a, b) => (a.end < b.end ? a : b)));
    }
    const gain = context.createGain();
    gain.gain.setValueAtTime(level, time);
    gain.connect(bus);
    const voice: Voice = {
      lane,
      gain,
      nodes: [gain],
      sources: [],
      pending: 0,
      end: time + duration,
      released: false,
      persistent,
      cue: null,
      theme: null,
    };
    this.voices.add(voice);
    return voice;
  }

  private tone(
    voice: Voice,
    time: number,
    start: number,
    end: number,
    duration: number,
    type: OscillatorType,
    level: number,
    attack = 0.003,
  ): void {
    const context = this.context;
    if (!context || voice.released) return;
    const source = context.createOscillator();
    source.type = type;
    source.frequency.setValueAtTime(Math.max(20, start), time);
    source.frequency.exponentialRampToValueAtTime(
      Math.max(20, end),
      time + duration,
    );
    this.attach(voice, source, time, duration, level, attack);
  }

  private noiseBurst(
    voice: Voice,
    time: number,
    duration: number,
    cutoff: number,
    level: number,
    type: BiquadFilterType,
  ): void {
    const context = this.context;
    if (!context || !this.noise || voice.released) return;
    const source = context.createBufferSource();
    source.buffer = this.noise;
    const filter = context.createBiquadFilter();
    filter.type = type;
    filter.frequency.value = Math.min(cutoff, context.sampleRate * 0.45);
    filter.Q.value = 0.7;
    source.connect(filter);
    voice.nodes.push(filter);
    this.attach(voice, source, time, duration, level, 0.002, filter);
  }

  private attach(
    voice: Voice,
    source: AudioScheduledSourceNode,
    time: number,
    duration: number,
    level: number,
    attack: number,
    output: AudioNode = source,
  ): void {
    const context = this.context;
    if (!context) return;
    const envelope = context.createGain();
    envelope.gain.setValueAtTime(FLOOR, time);
    envelope.gain.linearRampToValueAtTime(
      Math.max(FLOOR, level),
      time + Math.min(attack, duration * 0.25),
    );
    envelope.gain.exponentialRampToValueAtTime(FLOOR, time + duration);
    output.connect(envelope);
    envelope.connect(voice.gain);
    voice.nodes.push(source, envelope);
    voice.sources.push(source);
    voice.pending++;
    voice.end = Math.max(voice.end, time + duration + 0.025);
    source.onended = () => {
      voice.pending--;
      if (voice.pending <= 0) this.release(voice);
    };
    source.start(time);
    source.stop(time + duration + 0.025);
  }

  private release(voice: Voice): void {
    if (voice.released) return;
    voice.released = true;
    if (this.engineVoice === voice) {
      this.engineVoice = null;
      this.engineOscillators = [];
      this.engineFilter = null;
      this.engineDuckGain = null;
    }
    if (this.alarmVoice === voice) this.alarmVoice = null;
    if (this.tailVoice === voice) this.tailVoice = null;
    this.voices.delete(voice);
    for (const source of voice.sources) {
      source.onended = null;
      try {
        source.stop();
      } catch {
        /* Already ended. */
      }
    }
    for (const node of voice.nodes) node.disconnect();
  }
}
