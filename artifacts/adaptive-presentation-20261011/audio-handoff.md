# Authored adaptive audio — source handoff

Production source is frozen at manifest SHA-256 `2df45d5676eda537dc1d0acb29f19905b54dbb932708977d2299d19e4a77ad70`. Seven owned files and byte-identical source snapshots, eleven inspected inputs and five actual execution logs are recorded in `frozen-hashes.json`. This handoff does not assert a shipping package, browser DSP, human listening or phone acceptance.

The soundtrack now follows the actual Haven/Expanse/biome/champion region, and its combat arrangement follows living hostile heads inside the real AI pursuit extent. An idle canonical champion visit selects that champion's motif without adding combat: identity is obtained from the existing nearby-site cache and the same 320-unit contact region as the campaign HUD. Only the real active opponent or ordinary boss supplies the extra combat intensity. Haven and site sanctuary suppress aggro. Asteroids and radial difficulty never manufacture combat.

Fifteen authored score identities share the existing A-minor harmony and 112 BPM clock: Haven, five other biome identities, a separate outside-Haven Expanse arrangement, and all eight champion motifs. Each has a distinct contour/rest pattern, oscillator palette and arpeggio order. Theme weights crossfade with a 1.4-second time constant; combat rises with .45 seconds and recedes with 1.6 seconds. At most two identities are **selected for newly scheduled notes per step**; already-scheduled bounded note/pad tails can retain earlier identities. `diagnostics().themes` is this selection, while `scheduledThemes` records identities belonging to live scheduled music voices. Neither is a measurement of audible DSP.

## Caller contract

`audio.update(hud.audio)` uses the exported `AudioState` type:

```ts
type AudioState = {
  area: "haven" | "belt" | "reach" | "site" | "arena";
  biome: number;       // authoritative 0..5
  combat: number;      // finite 0..1, actual hostile pursuit
  hostiles: number;    // living hostile HEAD count, not Coil modules
  champion: number | null; // visited or active canonical champion 0..7
  boss: boolean;      // live ordinary active boss in pursuit range
  hull: number;       // real hull / effective maximum hull, 0..1
  speed: number;      // real speed / effective maximum speed, capped 0..1
  throttle: number;   // actual controller throttle, 0..1
  engines: number;    // surviving authored exhaust ports, 0..2
};
```

The Rust `AudioHud` is public through `game_core::AudioHud`; `Hud.audio` is additive. No profile field, schema, migration, economy, gameplay RNG or physical controller changes were made. Area `belt` groups the quiet drift and initial asteroid corridor outside Haven and below the combat radius. Area `arena` also represents an idle champion contact zone, so consumers must use `combat`/`hostiles`, not the area name, to decide combat intensity. Calling `update()` before unlock or during a hard stop only caches sanitized source state; it never creates or resumes a context.

The caller should supply its latest authoritative HUD before resuming a different world/profile. Root owns the main loop: live Escape/settings overlays continue updating sound with the running simulation; console/dialogue, hidden tab, portrait/recovery, title and imports use the existing hard lifecycle gates. `unlock()` remains the sole AudioContext creation path and must originate from a gesture. `resume()` reuses that context; intent epochs prevent older rejected/deferred resumes from changing the newest state. Unexpected interruption releases voices and clears stale engine/alarm/combat mix. A late running state event during a hard stop reasserts suspension.

`suspend()` immediately gates new playback, stops scheduling, releases all voices and cancels the one death-tail timeout. `dispose()` also disconnects the existing graph and closes the same context idempotently. No context is created by `play`, `update`, `diagnostics` or death handling.

After `play(actualDeathEvent)`, `finishDeath()` gates all new audio and keeps only that already-created death voice. Music, continuous engine/alarm and other voices are released immediately; both dry music and echo output are gated with an 8ms target to silence. The death sources end at approximately .725 seconds, with a hard timeout no longer than .8 seconds, followed by suspension of the same context. Retry/resume, hidden/full suspension or disposal cancels this timeout; a stale callback cannot stop resumed music. Calling it without a real current death voice performs a hard suspend and creates no cue.

`diagnostics()` is a bounded observational getter intended for the existing metrics cadence: actual context state/enabled/disposed/deathTail, source area/biome/champion/hostiles/boss/hull/throttle/speed/ports, current combat/alarm/engine mix, target combat, voice counts and selected/scheduled score identities. Root reads it at 500ms metrics cadence. Values describe resources/state, not sound quality or output loudness.

## Effects and bounds

The existing master/music/effects buses, limiter, deterministic procedural noise and exact Mote charge/pulse cues are retained. One continuous engine voice uses two oscillators and filtered looping noise; its level/pitch follow real throttle, speed and surviving ports. One low-hull alarm voice uses two sine oscillators and a two-beat envelope, starts at hull <=25%, and recedes on healing. Dead hull and destroyed ports cannot sustain an engine source. Both count inside the existing **16 total effects voices**, leaving at most fourteen simultaneous transient voices when both are active. They are protected from ordinary transient stealing; the oldest finite transient is released when capacity is full. Music remains bounded at 32 voices. Unknown events create no voice.

Actual source events added: `ability-echo`, `ability-nova`, `ability-wake`, `ability-signature-6/7/8/9/12/13`, `ability-royal`, `ability-repair`, `boss-charge`. Each belongs to a real volley, repair, nova, trail node or telegraph entry; none is emitted per pixel/ray. Wake output is additionally limited to one short sound per .6 seconds in the audio layer, without delayed replay. Boss arrival retains its original arrival cue, while subsequent actual Tell entries get a short .14s warning rather than a misleading sustained charge.

The `shot` event now requires a real new primary projectile. A full bullet cap still preserves the pre-existing gameplay shot counter/cooldown behavior; it produces no fictional primary or Echo cue. Signature/royal cues compare actual projectile counts, and repair requires a positive change to a living hull. Existing `hit` event compatibility and lighting are preserved: its established .25 occupied-target-cell intensity and .9 player-impact intensity select different timbres. Shield absorption and real Nova use separate sounds; the former Nova shield-light path is preserved under the new Nova event alias. No save or reward event is fabricated.

## Executed evidence

- `cargo test -p game_core --locked`: **223/223** actual native core tests pass, including **nine new** audio-state/effect tests. Those cover real source pursuit/safety, finite hull/motion/erased exhaust ports, actual primary/Echo cap gating, all six authored shot signatures, real shield/Nova/wake, actual boss phase transitions, all eight real champion fights and idle visits/retreats, plus actual royal volley/heal/full/dead-hull cases. Existing death, lifecycle, occupied-cell collision, cadence, progression and asteroid/connected-body tests remain unchanged and pass.
- `node --experimental-transform-types --test scripts/test/audio.test.mjs scripts/test/audio-state.test.mjs`: **23/23** checks pass: eighteen actual AudioEngine checks against a deterministic **recorded Web Audio API**, plus five pure score/mix checks. These prove graph/scheduling/resource contracts, individual dry timbres, Mote cue preservation, continuous reuse/caps, source sanitization and crossfades, healing/interruption/recovery, independent buses/mute, exact bounded death retention, stale timeout rejection, late resume intent and cleanup. They do not run browser DSP.
- Strict TypeScript, native all-target game_core Clippy and WASM library game_core Clippy pass with warnings denied.

No public/dist build, dependency addition, network track/model download, studio write, source commit or publication was performed by this task. The prior asteroid/connected-body and earlier audio DSP evidence remain historical and unchanged. New normal-package browser context/transition checks, actual output peak/RMS recordings, audible balance/listening, crowded-combat clarity and physical mobile/device recovery remain separate pending gates. A graph cap and limiter are not a measured clipping or phone-performance guarantee.
