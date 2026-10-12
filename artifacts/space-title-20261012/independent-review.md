# Title audio and hull display independent review

Final reviewed Root source: `344f8a2ca02f17d44e5ddf2d50c081738535ab3f`. This review is read-only; the specialist's own runtime branch remains `c3904b8`. Source/evidence hashes and committed-blob equivalence are recorded in `manifest.json`.

No remaining scoped blocker found after two lifecycle repairs:

- Initial working-source review found that successful title-screen graphics recovery suspended audio without restoring title intent. Root now calls `syncFlightState()` after clearing `recovering`, except while chaining another recovery. The focused actual-source fixture passes for visible and hidden titles, with simulation/input still stopped.
- The original visibility handler returned before suspending audio when no `galaxy` existed. A configured title track could therefore finish decoding while hidden during asynchronous initialization. Root now suspends before the Game guard and rejects hidden title gestures. The actual extracted handler's pre-init hide/show fixture passes. An additional independent probe executes the current callback with the real `TitleTrack` and recorded AudioContext: hiding before a deferred decode completes yields one suspend and zero created sources, both before and after Game initialization.

`TitleTrack` retains one decoded loop and one live/fading source on the existing AudioEngine context. Rapid reentry stops/disconnects the preceding source; suspend/disposal stop it, disposal aborts loading, and late decode cannot install a disposed buffer. Encoded/decode acceptance bounds and validated loop ranges are present. Current `TITLE_THEME.url` is null: the composer asset remains pending, so no audible title-music or listening acceptance is claimed.

The hull display is capacity-driven: 20 HP per mark, five marks for the default 100 HP hull, and fractional final capacity/charge slices. Current authoritative maximum is 260 HP (100 base + 75 hardware + 20 relics + 65 Sovereign), covered by thirteen bounded marks. Hull and shield compress their complete horizontal fields while retaining gaps, share the 44×12 logical reservation, and receive actual `hud.max_hull`/`max_shield`. The authored native tests cover default five/three marks, partial upgrades versus damage, capacities through 260, finite corners and bounded work. These native tests were inspected, not independently rerun in this review.

Independent checks executed: six title-track lifecycle tests, two focused actual-source recovery/visibility cases, and the deferred-decode visibility probe. All pass. The first probe attempt encountered a ReferenceError because Root replaced the inline callback with a named handler between reads; that harness mismatch is retained separately and is not a runtime failure or negative playback reproduction.

Limits: recorded Web Audio calls and source review, not real browser playback/DSP; no browser, preview, service, native rebuild or Root source changes. Root's reported full-suite results remain separate. Deep-nebula packet/software proof is frozen separately under `work/deep-nebula-gas`.
