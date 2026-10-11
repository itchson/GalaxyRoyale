# DORA and native control art

**Historical 96×128 milestone.** The portrait counts, fit scales and captured proofs below describe the earlier pack. The current game uses freshly authored **192×256** DORA, DORO and TALO portraits from `assets/ui/kikai/detail/atlas.json`, documented in [KIKAI_PROFILE_ART.md](KIKAI_PROFILE_ART.md). Guide ID 8 is fixed APR-1337; licensed profile ID 9 selects the actual DORA/DORO/TALO body. The historical control-icon pack remains in use. The standalone native proof script now reads the current portrait dimensions and expected cell count from the runtime pack; old proof files are retained as historical evidence.

DORA-PAN-RENKO-1337 (`APR-1337`) is the Haven guide. A denotes female, O male and T neutral; PAN green hair denotes her main corporation, and RENKO red crystal horns her secondary manufacturer. This is the user's latest naming clarification on 2026-10-11, superseding the former OPR shorthand. Golden eyes, pale armor, long female hair and the black mechanical neck retain the accepted reference's detail. The guide's appearance is fixed explicitly by `append_dora`, independently of generic preview defaults. This milestone changes labels and proof metadata, not portrait geometry or colors.

Her youthful appearance contrasts with her extreme age: the guide existed alongside biological humans and preserves their continuity through new KIKAI. She is a friendly explorer and K-pop performer. These are character facts, not new animation, age simulation or gameplay effects.

The separate character-preview API recolors declared hair and horn cells independently for PAN, RENKO and HUGO. It retains DORA's female long-hair geometry: it is not an authored DORO short-spiky or TALO crew-cut portrait, and grants no abilities or character identity. `CorporateAppearance::default()` remains PAN/RENKO to describe the original authored reference colors.

The approved generated expression sheet is preserved unchanged at `assets/concepts/kikai/dora-reference-expression-sheet.png`. Its green hair/red crystals match the latest guide appearance and remain the historical detail reference. It is a raster detail reference, not a lossless runtime sprite. The conversion deliberately selects the first portrait, thresholds alpha at 128, retains all components, tight-crops it, resamples to 96×128 with nearest sampling and quantizes without dithering. `assets/ui/kikai/provenance.json` records that loss and both reference hashes. The image tool did not supply a model identifier; no model or licensing guarantee is invented. The current guide explicitly selects PAN/RENKO through the existing independent hair/horn palette roles.

Eye patches, mouth states and the ten 32×32 control masks are authored in `scripts/asset-kikai.py`. These authored PNGs then pass through the exact `asset-pack.py` importer: no second resize, component removal or alpha threshold. The indexed pack round-trips every palette entry, index and declared role. Its 60-color palette reserves zero for transparency.

## Native integration contract

Register `crates/game_web/src/kikai_art.rs` in the native UI host. Portrait ID 8 is DORA. ID 9 may use `append_corporate_portrait` as an explicitly labeled long-hair color preview. The host supplies `DoraInput` with the measured local rectangle, clip, full parent affine, time, expression, speaking and Reduced Motion state. `append_dora` always uses DORA's PAN/RENKO appearance. The public input/output contract is unchanged.

The default portrait uses all 96×128 authored cells: **8,656 submitted occupied cells across three transforms** for each expression/speech pose. A 48×64 logical slot fits at scale 0.5; smaller slots retain those same cells with fractional transforms, including a tested 4.8×6.4 fit at scale 0.05. There is no automatic coarse LOD. Whole-group preflight rejects a portrait if either pixel or work budget is insufficient, and reports `budget_rejected`. Masked, alpha-zero submissions still consume both budgets. Clipping tests all four cell corners before composing the parent affine. Equivalent fit/edge arithmetic receives a bounded f32 rounding allowance: two relative epsilon units capped at **0.00001 local units**, never one cell. Actual half-width crops and representable overhangs remain masked; reflected/sheared parents and the 128-transform batching boundary are covered.

`DoraExpression::{Neutral,Focused,Concerned,Pleased}` selects actual eye/mouth cells. Blink and small horn signals animate only when enabled. `speaking` cycles three authored mouth poses; this is a visual speech rhythm, not phoneme-aligned lip sync. Reduced Motion and disabled state freeze cosmetic animation. The host must derive expression/speaking from actual dialogue state rather than infer speech from hover or selection.

`icon(kind)` provides cached original 32×32 PixelBodies for `status-icon`, `map-icon`, `settings-icon`, `save-icon`, `close-icon`, `resume-icon`, `restart-icon`, `title-icon`, `quit-icon` and `fullscreen-icon`. Six named material-color roles provide shells, edges, bevels, recesses, steel and accent cells. `icon_color` accepts the host's real action accent and hover/pressed/disabled state; save pending/error remains host-authoritative. Fit all cells fractionally into the measured art rectangle (for example scale 0.75 into 24×24), retaining the existing hit area. Root integration owns actual event/state wiring.

## Art standard and verification

The accepted reference sets the detail target: readable major shapes first, then coherent color clusters, panel edges, localized highlights and mechanical recesses. Preserve meaningful fine cells under continuous transforms rather than replacing them with a coarse silhouette. Authored animation remains local to its object; visual expressions never advance progression.

Install the existing pinned Pillow requirement, then reproduce conversion and the seven authoring/round-trip tests:

```powershell
python -m pip install -r scripts/asset-requirements.txt
python scripts/asset-kikai.py
python scripts/asset-kikai-tests.py
```

On Windows with Rust/Clippy installed, the scoped proof builds the real engine dependencies, independently compiles the art module, runs its twelve native checks and strict library Clippy, then extracts actual FrameBuilder packets and creates a flat software contact sheet:

```powershell
powershell -NoProfile -File scripts/asset-kikai-native.ps1 -Python python -OutputDirectory work/apr-guide-art/native
```

The current checked-in `artifacts/kikai-apr/native-contact-sheet.png` is generated from actual native packets: six guide poses call `append_dora`, three separately labeled long-hair previews call `append_corporate_portrait`, followed by all ten controls. `artifacts/kikai-apr/validation.json` pins current art source, unchanged authored data and actual test evidence. Root owns the corresponding latest narrative/main identity update; this scoped native proof does not change those files. All earlier proof sets remain unchanged: `artifacts/kikai/` from `bcbaa670ae8335b4e5cffff3348b0ae0a3144299` records the original green/red guide with its old DPR shorthand; `artifacts/kikai-orp/` from `ac47fd6c2750a77521c9e4001062c4110ead3097` records the intervening red/green ORP guide; `artifacts/kikai-opr/` from `123758d0497a4174ba56ad6c5f153858eb5765fd` records green/red with the former OPR shorthand. These historical captions are superseded, not rewritten. This verifies native cells, transforms, clipping, colors, budgets and authored states. It does not claim current browser integration, animation playback, human listening, mobile readability or frame-time performance.

`assets/narrative/dora-initial-mission.json` supplies eleven bounded authored pages for the approved stages 0–6. The game owns license acknowledgment, departure, immutable Haven-belt destruction, actual Tier 1 pickup/banking and craft-plus-explicit-equip progress. The data grants no reward and does not advance a stage. Legacy profiles with null onboarding receive no fabricated mission progress.
