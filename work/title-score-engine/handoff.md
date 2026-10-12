# Native score candidate handoff

Branch: `codex/title-score-engine`, based on `8c284637f3dfd4f86982f2dc01b10fb8a70c8162`, in `C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale`. The prior `codex/hangar-garage-ui` branch at `493dabde0c44d7dfe81a741421c3abad08e4881c` was preserved. The worktree was clean before switching. Root checkout, manifests, studio and renderer files were not edited.

This is an **authored reference-inspired synthesis draft**, not an exact transcription or a perceived-match acceptance. Source notes, octaves, gates, the F# cadence and 32-bar title form are provisional. Source analysis came from Root's `work/title-score-20261013/musical-report.md` and `score-plan.json`. The supplied reference remains outside shipped assets, unchanged at SHA256 `95fcb149bb55da752c4d93c18fcc1913a23b70ccafecb52d8db5548dc64755d6`.

Implemented:

- Removed the runtime title fetch/decode/recording-loop player. `TITLE_THEME` now holds explicitly authored note events, enable flag, source-candidate tempo and loop length. No URL field or fake media marker remains.
- Existing WebAudio `AudioEngine` owns one context, scheduler, music bus, bounded voice pool and pure `MusicClock`. Rust HUD intent still selects gameplay states; no Rust DSP claim is made.
- Title is a provisional B-minor chip/bubble/arpeggio/noise/bass arrangement at 169.275 BPM. Its 512 sixteenth steps are 32 authored bars, about45.37 seconds. This is a new draft loop form, not the measured42.68-second source form.
- PAN/RENKO/HUGO hub arrangements and established area/champion contours share B-minor harmony. Quiet world arrangements receive sparse title pitch-class replies. Combat/boss layers remain on the same chord phase. The thrust bed is retuned from A–E to B–F#; cue timbres/contracts and dry effects bus remain intact.
- Continue preserves beat/harmony phase and decelerates over8 beats to120 BPM. Real combat intent queues a4-beat ramp to166. Release holds2 beats, then ramps over8 beats to120; re-chase replaces pending release from the current boundary tempo.
- Tempo integration is linear in beat space. Future music voices are canceled/requeued at the change boundary, missed notes are skipped after stalls, and the0.18-second queue remains bounded. Already audible chord-pad fade/stop times follow the retimed harmonic boundary. Pause/resume keeps the existing phase through the short resume lead-in. Title cannot restart propulsion/alarm tails.
- Shared32-music/16-effects voice budgets, separate controls, source cleanup, death-tail intent, mobile interruption and asynchronous resume guards remain tested.

Exact authorized Main caller amendment (one matching guard existed in this frozen base):

```diff
-    if (!launched && !document.hidden && TITLE_THEME.url) void audio.unlock();
+    if (!launched && !document.hidden && TITLE_THEME.enabled) void audio.unlock();
```

No other Main content changed. Root's request mentioned two URL guards; inspection of this base found only the single guard above.

Verification actually executed, on Node24.18.0 / lockfile TypeScript5.9.3:

```powershell
node node_modules/typescript/bin/tsc --noEmit
node --experimental-transform-types --test scripts/test/music-clock.test.mjs scripts/test/title-theme.test.mjs scripts/test/audio-state.test.mjs scripts/test/audio.test.mjs
```

Both commands exited0. All49 focused tests passed. Raw logs are `typecheck.log` and `focused-tests.log`; `execution-evidence.json` records actual commands/exit statuses/scope, and `source-pins.json` hashes tested working-file bytes and logs. The pure transport tests independently integrate ramp durations and exercise inversion, boundary continuity, release cancellation, reanchoring and bounded history. Recorded WebAudio API tests exercise actual AudioEngine calls, future cancellation, pad stop retiming, Continue into all three hubs, stalls, mute, rapid state changes and existing lifecycle/effects behavior.

The30-second rapid-state fixture measured maximum21 music voices and12 future music voices, all future onsets inside the0.18-second lookahead. These are recorded API/resource measurements, not browser PCM peaks or audible results. Existing maximum-bus envelope tests still provide scalar/graph checks only.

Required Root integration/QA work remains:

1. Add `music-clock.ts` and `music-score.ts` to the actual OfflineAudioContext QA module preparation and served allowlist, and rewrite their import suffixes for the QA modules. Existing QA source preparation does not yet know these new modules. This specialist did not edit QA/studio shared integration files.
2. Render actual production-engine title/hub/exploration/chase/boss/release WAVs in native OfflineAudioContext with hashes and real peak/RMS/nonfinite evidence. Include Continue at multiple title positions, rapid re-chase, mute/resume, return-to-title and death.
3. Listen directly to the reference and rendered native draft. Recover/adjust principal hook, octaves, bass roots, chromatic10–12-second source region, rests and cadence. Exact transcription and recognizable-song acceptance remain open.
4. Physically listen on the real output device with representative game effects/volume and check balance, timbre fatigue, transitions and clicks. No physical listening, browser session, broad build, screenshot, game playback or native DSP render was performed in this candidate pass.

No dependencies, servers, workers, GPU models, generated source PCM or MP3 assets were added. The shipped engine does not depend on the studio or work directory.
