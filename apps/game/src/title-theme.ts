/** Supply an edited, original title loop here when the composer delivers it.
 * The game makes no request while url is null. Seconds refer to decoded audio.
 */
export type TitleTheme = {
  url: string | null;
  loopStart: number;
  loopEnd: number | null;
};
export const TITLE_THEME: TitleTheme = {
  url: null,
  loopStart: 0,
  loopEnd: null,
};

const FADE_SECONDS = 1.2;
const MAX_ENCODED_BYTES = 16 * 1024 * 1024;
const MAX_DECODED_BYTES = 64 * 1024 * 1024;

/** One decoded loop and one source, on the game's existing AudioContext.
 * A stopped/fading source is disconnected before replacement, even during
 * rapid title/flight changes. Loading never creates or resumes a context.
 */
export class TitleTrack {
  private buffer: AudioBuffer | null = null;
  private source: AudioBufferSourceNode | null = null;
  private gain: GainNode;
  private request: AbortController | null = null;
  private loading: Promise<void> | null = null;
  private stopping = false;
  private disposed = false;
  private active = false;
  private level = 0;
  private end = 0;
  status: "absent" | "idle" | "loading" | "ready" | "unavailable" = "absent";

  constructor(
    private context: AudioContext,
    destination: AudioNode,
    private theme: TitleTheme,
  ) {
    this.gain = context.createGain();
    this.gain.gain.value = 0;
    this.gain.connect(destination);
    if (theme.url) this.status = "idle";
  }

  load(): Promise<void> {
    if (
      this.loading ||
      this.status !== "idle" ||
      !this.theme.url ||
      this.disposed
    )
      return this.loading ?? Promise.resolve();
    const request = new AbortController();
    this.request = request;
    this.status = "loading";
    this.loading = (async () => {
      try {
        const response = await fetch(this.theme.url!, {
          signal: request.signal,
        });
        if (!response.ok) throw Error("Title track unavailable");
        const advertised = Number(response.headers.get("content-length"));
        if (advertised > MAX_ENCODED_BYTES)
          throw Error("Title track exceeds encoded budget");
        const reader = response.body?.getReader();
        if (!reader) throw Error("Title track has no body");
        const chunks: Uint8Array[] = [];
        let length = 0;
        try {
          while (true) {
            const { value, done } = await reader.read();
            if (done) break;
            length += value.byteLength;
            if (length > MAX_ENCODED_BYTES)
              throw Error("Title track exceeds encoded budget");
            chunks.push(value);
          }
        } finally {
          await reader.cancel().catch(() => {});
          reader.releaseLock();
        }
        const encoded = new Uint8Array(length);
        let offset = 0;
        for (const chunk of chunks) {
          encoded.set(chunk, offset);
          offset += chunk.length;
        }
        const buffer = await this.context.decodeAudioData(encoded.buffer);
        const end = this.theme.loopEnd ?? buffer.duration;
        if (
          buffer.numberOfChannels > 2 ||
          buffer.duration > 120 ||
          buffer.length * buffer.numberOfChannels * 4 > MAX_DECODED_BYTES ||
          !Number.isFinite(this.theme.loopStart) ||
          this.theme.loopStart < 0 ||
          !Number.isFinite(end) ||
          end > buffer.duration ||
          end - this.theme.loopStart < 0.1
        )
          throw Error("Invalid or oversized title loop");
        if (this.disposed || request.signal.aborted) return;
        this.buffer = buffer;
        this.end = end;
        this.status = "ready";
        this.update(this.active, this.level);
      } catch {
        if (!this.disposed) this.status = "unavailable";
      } finally {
        if (this.request === request) this.request = null;
      }
    })();
    return this.loading;
  }

  update(active: boolean, level: number): void {
    this.active = active;
    this.level = Number.isFinite(level) ? Math.max(0, Math.min(1, level)) : 0;
    if (this.disposed) return;
    const now = this.context.currentTime;
    if (active && this.buffer && this.context.state === "running") {
      if (this.stopping) this.stop();
      if (!this.source) {
        const source = this.context.createBufferSource();
        source.buffer = this.buffer;
        source.loop = true;
        source.loopStart = this.theme.loopStart;
        source.loopEnd = this.end;
        source.connect(this.gain);
        source.onended = () => {
          source.disconnect();
          if (this.source === source) this.source = null;
        };
        this.source = source;
        this.gain.gain.setValueAtTime(0, now);
        source.start(now, this.theme.loopStart);
      }
      this.gain.gain.setTargetAtTime(this.level, now, FADE_SECONDS / 5);
    } else if (this.source && !this.stopping) {
      this.gain.gain.setTargetAtTime(0, now, FADE_SECONDS / 5);
      this.source.stop(now + FADE_SECONDS);
      this.stopping = true;
    }
  }

  stop(): void {
    if (this.source) {
      this.source.onended = null;
      try {
        this.source.stop();
      } catch {
        /* Already stopped. */
      }
      this.source.disconnect();
    }
    this.source = null;
    this.stopping = false;
    this.gain.gain.cancelScheduledValues(this.context.currentTime);
    this.gain.gain.setValueAtTime(0, this.context.currentTime);
  }

  dispose(): void {
    this.disposed = true;
    this.active = false;
    this.request?.abort();
    this.stop();
    this.buffer = null;
    this.gain.disconnect();
  }
}
