# Dialogue and optional local voice

`apps/game/src/narrative.ts` provides immediately available authored dialogue pages. It accepts a snapshot of Rust-derived speaker, description, status, progress and factual briefing text; it has no gameplay mutation callbacks. Each page contains at most 256 ASCII characters, matching the pixel font. Eight personality indices select authored greetings and modest platform-speech delivery preferences. They do not imply eight installed neural voices.

## Browser contract

```ts
const director = new NarrativeDirector({
  voiceEnabled: false,
  voiceVolume: 0.7,
  bridge: optionalInjectedBridge,
  onUpdate: (pages) => redrawCurrentConversation(pages),
  onVoiceStatus: (status) => updateReplayStatus(status),
});
const pages = director.open({
  id: "champion-id",
  speaker: "Character name",
  personality: 0,
  description: "An authored briefing copied from the current game state.",
  status: "available",
  facts: ["The actual encounter rule from Rust."],
});
director.current();
director.next();
director.previous();
await director.replay(); // Explicit user action; never called by open().
director.pause();
director.resume();
director.setVoiceVolume(0.4);
director.setVoiceEnabled(false);
director.close();
```

`open` is synchronous and returns copied `DialoguePage[]` values. Pages expose `id`, `speaker`, `text`, `portrait`, `role` (greeting/briefing/state/fact) and `source` (authored/local). Context status accepts available/active/ready/completed/locked; progress must be an integer pair with current no greater than total. At most eight bounded facts are accepted. Navigation cancels speech. Close aborts text and voice work and invalidates the conversation epoch. Integration should call close on leaving a dialogue, pause on document hiding, and update its displayed pages through `onUpdate` if an inference bridge is supplied. Gameplay actions remain explicit callbacks owned by the game shell, separate from narrative text.

Voice is disabled by default and only starts through Replay. `voiceStatus` distinguishes disabled, idle, loading, speaking, paused, unavailable and error. A missing voice does not hide text or block a gameplay action. The caller owns voice-volume settings; the module keeps voice volume independent of its audio/music engine. Muting cancels ongoing voice. The browser adapter releases its audio URL, clears event handlers and has a 45-second playback limit. Available platform voices, their pronunciation and offline behavior depend on the browser and operating system. [SpeechSynthesis API](https://developer.mozilla.org/en-US/docs/Web/API/SpeechSynthesis)

## Optional inference boundary

The browser has no hard-coded local model endpoint. A future desktop adapter can inject these two optional functions:

```ts
interface NarrativeBridge {
  generate?: (
    request: NarrativeTextRequest,
    signal: AbortSignal,
  ) => Promise<unknown>;
  synthesize?: (
    request: NarrativeVoiceRequest,
    signal: AbortSignal,
  ) => Promise<unknown>;
}
```

Text requests contain bounded narrative context, an authored baseline, instructions and a 256-character limit. Only a `{flavor: string}` greeting is accepted. Markup, controls, non-ASCII text, links, digits and common reward/progression claims are rejected. Briefing, status, progress and facts stay authored. This validator narrows the permitted presentation surface; it is not a guarantee that a language model always tells the truth. Inference output is never parsed as a command and cannot grant rewards, modify saves, unlock ships or complete quests.

Voice requests contain the exact displayed text, character ID, personality, speaker and an original adult voice-style description. Synthesized output must contain copied `Uint8Array` bytes and a matching WAV, Ogg or MP3 header, bounded to 8 MiB. Header checks are input screening; browser decoding still determines whether playback succeeds. Invalid or failed synthesis can fall back to optional platform speech.

Inference requests time out after 3.5 seconds by default (configurable within 10 ms to 15 seconds). Abort signals, conversation epochs and active request identity prevent an old response from changing a new conversation or playing after close. A provider ignoring cancellation still cannot apply its stale result. Never give this adapter agent tools, credentials, arbitrary filesystem access or gameplay authority. Do not expose the development Codex router as a game API.

## Observed local capability and practical next step

Read-only discovery in this development session found the local Codex model router healthy and cached voice-model directories under `C:\AI\models\voice-studio`: Qwen3-TTS-12Hz-1.7B-VoiceDesign, Qwen3-TTS-12Hz-1.7B-Base, Fun-CosyVoice3-0.5B-2512, CosyVoice-ttsfrd and higgs-tts-3-4b. VoiceDesign has a model weight file of approximately 3.83 GB. Its cached model card declares Apache-2.0. These are cache observations, not evidence of a running TTS service, compatible Python environment, completed voice generation or measured quality/latency. The bounded listener check found no service on ports 7860, 7861, 8000, 8001, 8080, 8880, 5000 or 1234; other ports were not ruled out. No service was started and no model was downloaded.

The practical first neural-voice experiment is the already cached Qwen3-TTS VoiceDesign/Base pair on the desktop. Official examples support described voices and reusing a generated reference with the Base model to keep a character consistent. The project declares Apache-2.0 and offers 0.6B and 1.7B variants. [Official Qwen3-TTS repository](https://github.com/QwenLM/Qwen3-TTS), [official VoiceDesign model card](https://huggingface.co/Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign)

Generate and review short clips for eight original characters, retain prompts/reference hashes and pronunciation notes, then package approved audio for replay. This is a recommended next experiment, not delivered neural voice content. A frozen asset pipeline makes the same dialogue audio portable to the web/PWA and phones without requiring a desktop GPU or inference server. Record listening acceptance and measured latency before deciding whether dynamic local synthesis adds value.

Lighter alternatives include the creator's Apache-2.0 82M-parameter Kokoro model and the current Piper engine, whose repository declares GPL-3.0. Voice/model licenses and any generated-asset distribution terms need their own recorded review. Neither alternative was installed or benchmarked here. [Kokoro model card](https://huggingface.co/hexgrad/Kokoro-82M), [current Piper repository](https://github.com/OHF-Voice/piper1-gpl)

Optional small browser text models are feasible as a separate opt-in worker: Transformers.js documents WebGPU execution, and its official examples include Qwen3-0.6B with ONNX. Model downloads, device support, GPU sharing with the renderer and memory/latency still need measurement. Do not make a large model a required download for a 60 FPS mobile game. [Transformers.js WebGPU guide](https://huggingface.co/docs/transformers.js/guides/webgpu), [official Qwen3 browser example](https://github.com/huggingface/transformers.js-examples/tree/main/qwen3-webgpu), [ONNX Runtime Web guide](https://onnxruntime.ai/docs/tutorials/web/)

On a phone, localhost refers to the phone, not this workstation. A static hosted game cannot assume access to desktop inference. A desktop-only adapter would therefore remain an explicit optional capability; the shipped authored dialogue and optional platform replay must remain usable without it.

## Verification scope

Run `node --experimental-transform-types --test scripts/test/narrative.test.mjs`. Tests use the real module with mock inference/playback and browser audio/speech surfaces. They cover bounded authored pages and state copies, invalid input, flavor isolation, stale responses, cancellation, timeouts, explicit replay, pause/resume, volume and audio URL cleanup. These tests do not claim an installed neural voice service, listening acceptance, physical-phone audio behavior or a completed tournament playthrough. Integration and campaign acceptance remain separate production tasks.
