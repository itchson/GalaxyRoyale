export interface DialoguePage {
  id: string;
  speaker: string;
  text: string;
  portrait: number;
  role: "greeting" | "briefing" | "state" | "fact";
  source: "authored" | "local";
}

export interface NarrativeContext {
  id: string;
  speaker: string;
  description: string;
  personality?: number;
  status?: "available" | "active" | "ready" | "completed" | "locked";
  progress?: { current: number; total: number };
  facts?: string[];
}

export interface NarrativeTextRequest {
  context: NarrativeContext;
  baseline: string;
  instruction: string;
  maxCharacters: number;
}
export interface NarrativeVoiceRequest {
  characterId: string;
  speaker: string;
  personality: number;
  text: string;
  style: string;
}
export interface NarrativeAudio {
  bytes: Uint8Array;
  mimeType: "audio/wav" | "audio/mpeg" | "audio/ogg";
}
export interface NarrativeBridge {
  generate?: (
    request: NarrativeTextRequest,
    signal: AbortSignal,
  ) => Promise<unknown>;
  synthesize?: (
    request: NarrativeVoiceRequest,
    signal: AbortSignal,
  ) => Promise<unknown>;
}
export type VoiceStatus =
  | "disabled"
  | "idle"
  | "loading"
  | "speaking"
  | "paused"
  | "unavailable"
  | "error";
export interface NarrativePlayback {
  play(
    request: NarrativeVoiceRequest & {
      volume: number;
      audio?: NarrativeAudio;
      onStarted: () => void;
    },
    signal: AbortSignal,
  ): Promise<void>;
  pause(): void;
  resume(): void;
  cancel(): void;
  setVolume(volume: number): void;
}
export interface NarrativeOptions {
  bridge?: NarrativeBridge;
  playback?: NarrativePlayback;
  voiceEnabled?: boolean;
  voiceVolume?: number;
  timeoutMs?: number;
  onUpdate?: (pages: DialoguePage[]) => void;
  onVoiceStatus?: (status: VoiceStatus) => void;
}

const PAGE_LIMIT = 256;
const AUDIO_LIMIT = 8 * 1024 * 1024;
const GREETINGS = [
  "Steady hands, pilot. Read the brief before committing.",
  "Check your vectors. Preparation is part of every good flight.",
  "Keep the channel clear. We will take this one step at a time.",
  "A little courage travels a long way. Start with the facts.",
  "Precision first. Make every decision count.",
  "Welcome aboard this channel. Let us look at the brief together.",
  "Listen closely, pilot. The stars reward patience.",
  "You have my attention. Show me a considered approach.",
] as const;
const VOICE_STYLES = [
  "Calm steady adult voice, clear measured delivery.",
  "Brisk analytical adult voice, clipped precise delivery.",
  "Reserved grounded adult voice, restrained delivery.",
  "Warm optimistic adult voice, lively delivery.",
  "Focused confident adult voice, deliberate delivery.",
  "Friendly reassuring adult voice, relaxed delivery.",
  "Thoughtful seasoned adult voice, patient delivery.",
  "Direct competitive adult voice, controlled delivery.",
] as const;
const STATE_LINES = {
  available: "This objective is available.",
  active: "This objective is in progress.",
  ready: "The listed action is ready.",
  completed: "This objective is recorded as complete.",
  locked: "This objective is locked.",
};

const volume = (value: number): number =>
  Number.isFinite(value) ? Math.max(0, Math.min(1, value)) : 0;
const plain = (value: unknown, limit: number): string | null => {
  if (typeof value !== "string" || value.length > limit) return null;
  // Pixel glyphs are ASCII. Markup, control codes and bidi text have no place here.
  if (/[^\x20-\x7e\r\n\t]|[<>]/.test(value)) return null;
  return value.replace(/\s+/g, " ").trim() || null;
};

export function validateFlavor(value: unknown): string | null {
  if (!value || typeof value !== "object") return null;
  const flavor = plain((value as { flavor?: unknown }).flavor, PAGE_LIMIT);
  // Generated flavor cannot report numerical rewards or issue gameplay orders.
  if (
    !flavor ||
    /\d|https?:|javascript:|\b(credits?|currency|rewards?|unlocks?|grant|purchase|claim|completed|defeated|craft|tournament|inventory)\b/i.test(
      flavor,
    )
  )
    return null;
  return flavor;
}

export function validateNarrativeAudio(value: unknown): NarrativeAudio | null {
  if (!value || typeof value !== "object") return null;
  const { bytes, mimeType } = value as Partial<NarrativeAudio>;
  if (
    !(bytes instanceof Uint8Array) ||
    bytes.length < 12 ||
    bytes.length > AUDIO_LIMIT
  )
    return null;
  const magic = (start: number, text: string) =>
    [...text].every((letter, i) => bytes[start + i] === letter.charCodeAt(0));
  const valid =
    (mimeType === "audio/wav" && magic(0, "RIFF") && magic(8, "WAVE")) ||
    (mimeType === "audio/ogg" && magic(0, "OggS")) ||
    (mimeType === "audio/mpeg" &&
      (magic(0, "ID3") || (bytes[0] === 0xff && (bytes[1] & 0xe0) === 0xe0)));
  return valid ? { bytes: bytes.slice(), mimeType: mimeType! } : null;
}

function snapshotContext(input: NarrativeContext): NarrativeContext {
  const id = plain(input.id, 128);
  const speaker = plain(input.speaker, 64);
  const description = plain(input.description, 2048);
  if (!id || !speaker || !description)
    throw new Error("Invalid narrative context.");
  const personality = input.personality ?? 0;
  if (!Number.isInteger(personality) || personality < 0 || personality > 7)
    throw new Error("Personality must be an integer from 0 to 7.");
  if (input.status !== undefined && !Object.hasOwn(STATE_LINES, input.status))
    throw new Error("Unknown narrative status.");
  const progress = input.progress ? { ...input.progress } : undefined;
  if (
    progress &&
    (!Number.isSafeInteger(progress.current) ||
      !Number.isSafeInteger(progress.total) ||
      progress.current < 0 ||
      progress.total < 1 ||
      progress.total > 1_000_000 ||
      progress.current > progress.total)
  )
    throw new Error("Invalid narrative progress.");
  if (input.facts && (!Array.isArray(input.facts) || input.facts.length > 8))
    throw new Error("At most eight narrative facts are allowed.");
  const inputFacts = input.facts ?? [];
  if (
    Array.from({ length: inputFacts.length }, (_, i) => i).some(
      (i) => !Object.hasOwn(inputFacts, i),
    )
  )
    throw new Error("Narrative facts cannot contain missing entries.");
  const facts = inputFacts.map((fact) => {
    const text = plain(fact, PAGE_LIMIT);
    if (!text) throw new Error("Invalid narrative fact.");
    return text;
  });
  return {
    id,
    speaker,
    description,
    personality,
    status: input.status,
    progress,
    facts,
  };
}

export function splitPages(text: string): string[] {
  const result: string[] = [];
  while (text.length > PAGE_LIMIT) {
    let end = text.lastIndexOf(" ", PAGE_LIMIT);
    if (end <= 0) end = PAGE_LIMIT;
    result.push(text.slice(0, end));
    text = text.slice(end).trimStart();
  }
  if (text) result.push(text);
  return result;
}

/** Injected inference receives narrative facts only, never a gameplay API. */
export class NarrativeDirector {
  private context: NarrativeContext | null = null;
  private pages: DialoguePage[] = [];
  private index = 0;
  private epoch = 0;
  private textAbort: AbortController | null = null;
  private voiceAbort: AbortController | null = null;
  private voicePageId: string | null = null;
  private enabled: boolean;
  private voiceVolume: number;
  private status: VoiceStatus;
  private readonly timeoutMs: number;
  private readonly playback: NarrativePlayback;

  constructor(private readonly options: NarrativeOptions = {}) {
    this.enabled = options.voiceEnabled === true;
    this.voiceVolume = volume(options.voiceVolume ?? 0.7);
    this.status = this.enabled ? "idle" : "disabled";
    this.timeoutMs = Number.isFinite(options.timeoutMs)
      ? Math.max(10, Math.min(15_000, options.timeoutMs!))
      : 3500;
    this.playback = options.playback ?? new BrowserNarrativePlayback();
  }

  get voiceStatus(): VoiceStatus {
    return this.status;
  }

  open(input: NarrativeContext): DialoguePage[] {
    const context = snapshotContext(input);
    this.close();
    this.context = context;
    const epoch = this.epoch;
    const add = (text: string, role: DialoguePage["role"]): DialoguePage => ({
      id: `${context.id}:${role}:${this.pages.length}`,
      speaker: context.speaker,
      text,
      portrait: context.personality!,
      role,
      source: "authored",
    });
    this.pages.push(add(GREETINGS[context.personality!], "greeting"));
    for (const text of splitPages(context.description))
      this.pages.push(add(text, "briefing"));
    if (context.status || context.progress) {
      const progress = context.progress;
      const text = [
        context.status ? STATE_LINES[context.status] : "",
        progress ? `Progress: ${progress.current} / ${progress.total}.` : "",
      ]
        .filter(Boolean)
        .join(" ");
      this.pages.push(add(text, "state"));
    }
    for (const fact of context.facts!) this.pages.push(add(fact, "fact"));
    if (this.options.bridge?.generate) void this.generateFlavor(epoch);
    return this.copyPages();
  }

  current(): DialoguePage | null {
    return this.pages[this.index] ? { ...this.pages[this.index] } : null;
  }

  next(): DialoguePage | null {
    this.cancelVoice();
    this.index = Math.min(Math.max(0, this.pages.length - 1), this.index + 1);
    return this.current();
  }

  previous(): DialoguePage | null {
    this.cancelVoice();
    this.index = Math.max(0, this.index - 1);
    return this.current();
  }

  private copyPages(): DialoguePage[] {
    return this.pages.map((page) => ({ ...page }));
  }

  private setStatus(status: VoiceStatus): void {
    if (this.status === status) return;
    this.status = status;
    this.options.onVoiceStatus?.(status);
  }

  private async request<T>(
    operation: (signal: AbortSignal) => Promise<T>,
    controller: AbortController,
  ): Promise<T> {
    const timer = setTimeout(
      () => controller.abort(new Error("Narrative request timed out.")),
      this.timeoutMs,
    );
    let onAbort: () => void = () => {};
    const abort = new Promise<never>((_, reject) => {
      onAbort = () =>
        reject(controller.signal.reason ?? new Error("Cancelled."));
      if (controller.signal.aborted) onAbort();
      else controller.signal.addEventListener("abort", onAbort, { once: true });
    });
    try {
      return await Promise.race([
        Promise.resolve().then(() => {
          if (controller.signal.aborted) throw controller.signal.reason;
          return operation(controller.signal);
        }),
        abort,
      ]);
    } finally {
      clearTimeout(timer);
      controller.signal.removeEventListener("abort", onAbort);
    }
  }

  private async generateFlavor(epoch: number): Promise<void> {
    const context = this.context!;
    const controller = new AbortController();
    this.textAbort = controller;
    try {
      const response = await this.request(
        (signal) =>
          this.options.bridge!.generate!(
            {
              context: snapshotContext(context),
              baseline: this.pages[0].text,
              instruction:
                "Return only {flavor:string}: a short fictional radio greeting. Do not issue actions, numerical claims, rewards, unlocks or progression facts. The game's briefing is authoritative.",
              maxCharacters: PAGE_LIMIT,
            },
            signal,
          ),
        controller,
      );
      const flavor = validateFlavor(response);
      if (this.epoch !== epoch || controller.signal.aborted || !flavor) return;
      if (this.voicePageId === this.pages[0].id) this.cancelVoice();
      this.pages[0] = { ...this.pages[0], text: flavor, source: "local" };
      this.options.onUpdate?.(this.copyPages());
    } catch {
      // Immediate authored content remains usable on failure, timeout or close.
    } finally {
      if (this.textAbort === controller) this.textAbort = null;
    }
  }

  setVoiceEnabled(enabled: boolean): void {
    this.enabled = enabled === true;
    this.cancelVoice();
  }

  setVoiceVolume(value: number): void {
    this.voiceVolume = volume(value);
    this.playback.setVolume(this.voiceVolume);
    if (this.voiceVolume === 0) this.cancelVoice();
  }

  async replay(page = this.current()): Promise<void> {
    this.cancelVoice();
    if (!this.enabled || !this.context || !page || this.voiceVolume === 0)
      return;
    const canonical = this.pages.find((item) => item.id === page.id);
    if (!canonical || canonical.text !== page.text) return;
    const epoch = this.epoch;
    const controller = new AbortController();
    this.voiceAbort = controller;
    this.voicePageId = canonical.id;
    const request: NarrativeVoiceRequest = {
      characterId: this.context.id,
      speaker: canonical.speaker,
      personality: canonical.portrait,
      text: canonical.text,
      style: VOICE_STYLES[canonical.portrait],
    };
    this.setStatus("loading");
    let audio: NarrativeAudio | undefined;
    if (this.options.bridge?.synthesize) {
      try {
        const value = await this.request(
          (signal) => this.options.bridge!.synthesize!({ ...request }, signal),
          controller,
        );
        audio = validateNarrativeAudio(value) ?? undefined;
      } catch {
        if (this.epoch !== epoch || this.voiceAbort !== controller) return;
        // A timed-out optional TTS request may still use platform speech.
        this.voiceAbort = new AbortController();
      }
    }
    const playbackController = this.voiceAbort;
    if (
      this.epoch !== epoch ||
      !playbackController ||
      playbackController.signal.aborted
    )
      return;
    try {
      await this.playback.play(
        {
          ...request,
          volume: this.voiceVolume,
          audio,
          onStarted: () => {
            if (this.epoch === epoch && this.voiceAbort === playbackController)
              this.setStatus("speaking");
          },
        },
        playbackController.signal,
      );
      if (this.epoch === epoch && this.voiceAbort === playbackController)
        this.setStatus("idle");
    } catch (error) {
      if (this.epoch === epoch && this.voiceAbort === playbackController)
        this.setStatus(
          error instanceof VoiceUnavailable ? "unavailable" : "error",
        );
    } finally {
      if (this.voiceAbort === playbackController) {
        this.voiceAbort = null;
        this.voicePageId = null;
      }
    }
  }

  pause(): void {
    if (this.status !== "speaking") return;
    this.playback.pause();
    this.setStatus("paused");
  }

  resume(): void {
    if (this.status !== "paused") return;
    this.playback.resume();
    this.setStatus("speaking");
  }

  cancelVoice(): void {
    this.voiceAbort?.abort();
    this.voiceAbort = null;
    this.voicePageId = null;
    this.playback.cancel();
    this.setStatus(this.enabled ? "idle" : "disabled");
  }

  close(): void {
    this.epoch++;
    this.textAbort?.abort();
    this.textAbort = null;
    this.cancelVoice();
    this.context = null;
    this.pages = [];
    this.index = 0;
  }
}

export class VoiceUnavailable extends Error {}

/** Platform voices are optional and device-dependent; no neural cast is implied. */
export class BrowserNarrativePlayback implements NarrativePlayback {
  private media: HTMLAudioElement | null = null;
  private utterance: SpeechSynthesisUtterance | null = null;
  private stop: (() => void) | null = null;

  play(
    request: NarrativeVoiceRequest & {
      volume: number;
      audio?: NarrativeAudio;
      onStarted: () => void;
    },
    signal: AbortSignal,
  ): Promise<void> {
    this.cancel();
    return new Promise((resolve, reject) => {
      let url: string | null = null;
      let settled = false;
      const timer = setTimeout(() => {
        this.media?.pause();
        if (this.utterance) speechSynthesis.cancel();
        finish(new Error("Voice playback timed out."));
      }, 45_000);
      const cleanup = () => {
        clearTimeout(timer);
        signal.removeEventListener("abort", abort);
        if (this.media) {
          this.media.onplaying = null;
          this.media.onended = null;
          this.media.onerror = null;
          this.media.pause();
          this.media.removeAttribute("src");
          this.media.load();
        }
        if (this.utterance) {
          this.utterance.onstart = null;
          this.utterance.onend = null;
          this.utterance.onerror = null;
        }
        if (url) URL.revokeObjectURL(url);
        this.media = null;
        this.utterance = null;
        this.stop = null;
      };
      const finish = (error?: Error) => {
        if (settled) return;
        settled = true;
        cleanup();
        error ? reject(error) : resolve();
      };
      const abort = () => {
        this.media?.pause();
        if (this.utterance && typeof speechSynthesis !== "undefined")
          speechSynthesis.cancel();
        finish(new Error("Voice cancelled."));
      };
      this.stop = abort;
      signal.addEventListener("abort", abort, { once: true });
      if (signal.aborted) {
        abort();
        return;
      }
      try {
        if (request.audio) {
          const bytes = new Uint8Array(request.audio.bytes);
          url = URL.createObjectURL(
            new Blob([bytes], { type: request.audio.mimeType }),
          );
          const media = new Audio(url);
          this.media = media;
          media.volume = request.volume;
          media.onplaying = request.onStarted;
          media.onended = () => finish();
          media.onerror = () =>
            finish(new Error("Voice audio could not be decoded."));
          void media
            .play()
            .catch((error: unknown) =>
              finish(error instanceof Error ? error : new Error(String(error))),
            );
        } else if (
          typeof speechSynthesis !== "undefined" &&
          typeof SpeechSynthesisUtterance !== "undefined"
        ) {
          const utterance = new SpeechSynthesisUtterance(request.text);
          this.utterance = utterance;
          utterance.volume = request.volume;
          utterance.rate = [0.95, 1.06, 0.92, 1.04, 0.98, 1, 0.9, 1.02][
            request.personality
          ];
          utterance.pitch = [0.95, 1.04, 0.87, 1.12, 0.93, 1.05, 0.85, 0.98][
            request.personality
          ];
          utterance.lang = "en";
          const voices = speechSynthesis
            .getVoices()
            .filter((voice) => voice.lang.startsWith("en"));
          if (voices.length)
            utterance.voice = voices[request.personality % voices.length];
          utterance.onstart = request.onStarted;
          utterance.onend = () => finish();
          utterance.onerror = () =>
            finish(new Error("Platform speech failed."));
          speechSynthesis.speak(utterance);
        } else finish(new VoiceUnavailable("No platform voice is available."));
      } catch (error) {
        finish(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  pause(): void {
    this.media?.pause();
    if (this.utterance) speechSynthesis.pause();
  }
  resume(): void {
    if (this.media) void this.media.play().catch(() => this.stop?.());
    if (this.utterance) speechSynthesis.resume();
  }
  cancel(): void {
    this.stop?.();
  }
  setVolume(value: number): void {
    if (this.media) this.media.volume = value;
    if (this.utterance) this.utterance.volume = value;
  }
}
