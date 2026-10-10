/** Authored procedural score and effects: no samples, generated tracks, or network assets. */
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
};
const FLOOR = 0.0001;
const clamp = (value: number): number =>
  Number.isFinite(value) ? Math.min(1, Math.max(0, value)) : 0;
const frequency = (note: number): number => 440 * 2 ** ((note - 69) / 12);

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
const LEAD: readonly (readonly (number | null)[])[] = [
  [
    69,
    null,
    null,
    72,
    null,
    76,
    null,
    null,
    72,
    null,
    null,
    69,
    null,
    67,
    null,
    null,
  ],
  [
    69,
    null,
    null,
    72,
    null,
    null,
    77,
    null,
    76,
    null,
    72,
    null,
    69,
    null,
    null,
    null,
  ],
  [
    72,
    null,
    76,
    null,
    null,
    79,
    null,
    null,
    76,
    null,
    74,
    null,
    72,
    null,
    null,
    null,
  ],
  [
    71,
    null,
    null,
    74,
    null,
    79,
    null,
    null,
    77,
    null,
    74,
    null,
    71,
    null,
    null,
    null,
  ],
  [
    76,
    null,
    72,
    null,
    69,
    null,
    null,
    72,
    null,
    76,
    null,
    79,
    null,
    76,
    null,
    null,
  ],
  [
    77,
    null,
    null,
    76,
    null,
    72,
    null,
    null,
    69,
    null,
    72,
    null,
    76,
    null,
    null,
    null,
  ],
  [
    74,
    null,
    77,
    null,
    81,
    null,
    null,
    77,
    null,
    74,
    null,
    72,
    null,
    69,
    null,
    null,
  ],
  [
    71,
    null,
    null,
    76,
    null,
    80,
    null,
    null,
    83,
    null,
    80,
    null,
    76,
    null,
    71,
    null,
  ],
];
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
  private noise: AudioBuffer | null = null;
  private graph: AudioNode[] = [];
  private voices = new Set<Voice>();
  private timer: number | null = null;
  private nextNote = 0;
  private step = 0;
  private danger = 0;
  private targetDanger = 0;
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
          if (this.context?.state === "running" && !this.paused)
            this.startScheduler();
          else this.stopScheduler();
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
        this.levels[index]!,
        context.currentTime,
        0.025,
      );
    });
  }

  setDanger(danger: number, safe: boolean): void {
    this.targetDanger = safe
      ? 0
      : Math.min(
          1,
          0.35 + Math.max(0, Number.isFinite(danger) ? danger : 0) * 0.12,
        );
  }

  play(event: SoundEvent): void {
    const context = this.context;
    if (!context || context.state !== "running" || this.paused || this.disposed)
      return;
    const now = context.currentTime;
    const voice = this.createVoice(
      "sfx",
      now,
      0.9,
      0.75 + clamp(event.intensity) * 0.25,
    );
    if (!voice) return;
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
    switch (event.kind) {
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
  }

  suspend(): void {
    this.paused = true;
    this.stopScheduler();
    for (const voice of [...this.voices]) this.release(voice);
    if (this.context && this.context.state !== "closed")
      void this.context.suspend().catch(() => undefined);
  }

  async resume(): Promise<void> {
    if (this.disposed || !this.context || this.context.state === "closed")
      return;
    this.paused = false;
    try {
      await this.context.resume();
    } catch {
      this.paused = true;
      return;
    }
    if (!this.disposed && !this.paused && this.context.state === "running")
      this.startScheduler();
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
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
    this.noise = null;
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
    const warmth = context.createBiquadFilter();
    warmth.type = "lowpass";
    warmth.frequency.value = 5600;
    warmth.Q.value = 0.45;
    const echo = context.createDelay(1);
    echo.delayTime.value = (60 / 112) * 0.75;
    const feedback = context.createGain();
    feedback.gain.value = 0.16;
    const wet = context.createGain();
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
    limiter.connect(context.destination);
    this.graph = [
      this.master,
      this.music,
      this.sfx,
      limiter,
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
    if (this.nextNote < context.currentTime - 0.2)
      this.nextNote = context.currentTime + 0.02;
    const length = 60 / 112 / 4;
    while (this.nextNote < context.currentTime + 0.18) {
      this.danger += (this.targetDanger - this.danger) * 0.045;
      this.scoreStep(this.nextNote, this.step++, length);
      this.nextNote += length;
    }
  }

  private scoreStep(time: number, step: number, length: number): void {
    const bar = Math.floor(step / 16) % CHORDS.length;
    const beat = step % 16;
    const chord = CHORDS[bar]!;
    if (beat === 0) {
      const pad = this.createVoice("music", time, length * 16, 1);
      if (pad)
        chord.forEach((note) =>
          this.tone(
            pad,
            time,
            frequency(note + 12),
            frequency(note + 12),
            length * 15.8,
            "triangle",
            0.025 * (1 - this.danger * 0.4),
            0.12,
          ),
        );
    }
    const voice = this.createVoice("music", time, 0.55, 1);
    if (!voice) return;
    if (beat % 4 === 0) {
      const note = chord[[0, 2, 1, 3][Math.floor(beat / 4)]!]! + 24;
      this.tone(
        voice,
        time,
        frequency(note),
        frequency(note),
        length * 1.6,
        "triangle",
        0.045 * (1 - this.danger * 0.35),
      );
    }
    const bass = BASS[beat];
    if (
      bass !== null &&
      bass !== undefined &&
      (this.danger > 0.25 || beat === 0 || beat === 8)
    )
      this.tone(
        voice,
        time,
        frequency(chord[0] + bass),
        frequency(chord[0] + bass),
        length * 1.15,
        "square",
        0.035 + this.danger * 0.025,
      );
    const lead = LEAD[bar]![beat];
    if (lead !== null && lead !== undefined && this.danger > 0.28)
      this.tone(
        voice,
        time,
        frequency(lead),
        frequency(lead),
        length * 1.35,
        "square",
        this.danger * 0.024,
      );
    if (this.danger > 0.2 && beat % 4 === 0)
      this.tone(voice, time, 105, 39, 0.13, "sine", this.danger * 0.13);
    if (this.danger > 0.3 && (beat === 4 || beat === 12))
      this.noiseBurst(voice, time, 0.1, 2100, this.danger * 0.065, "bandpass");
    if (this.danger > 0.4 && beat % 2 === 1)
      this.noiseBurst(
        voice,
        time,
        0.032,
        5200,
        this.danger * 0.021,
        "highpass",
      );
    if (voice.pending === 0) this.release(voice);
  }

  private createVoice(
    lane: Lane,
    time: number,
    duration: number,
    level: number,
  ): Voice | null {
    const context = this.context;
    const bus = lane === "music" ? this.music : this.sfx;
    if (!context || !bus) return null;
    const active = [...this.voices].filter((voice) => voice.lane === lane);
    if (active.length >= (lane === "sfx" ? 16 : 32)) {
      if (lane === "music") return null;
      this.release(active.reduce((a, b) => (a.end < b.end ? a : b)));
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
