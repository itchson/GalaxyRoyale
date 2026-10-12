# Background and UI style — independent baseline review

Exact source: **4e014bff3a84b356c5254b4291560e38accef64d**. This is a read-only review; no runtime files, servers, builds, or Studio records were changed. `source-audit.json` pins 20 committed files. The three cosmic source/test blobs equal Engine's frozen **c3904b8f57b9c96cf5e5f2b5672b3bc9e3d020a5** exactly. Our checkout remains on the separate frozen asteroid branch.

## Priorities for the authors

1. **Replace the legacy ribbon and clockwork remnant first.** `regions.rs:404` makes its 32×20 nebula from a sinusoidal strip, two colors and an every-third-cell diagonal exclusion. This regular pattern is visibly designed rather than cloud-like. `regions.rs:598` adds 24 equally spaced radial sparks to its nova; `:727` repeats a 12-second pulse within a 90-second cycle. Prefer sparse broken shell filaments, unequal knots, overlapping quiet cloud masses and substantial dark openings. Avoid a regular expanding warning ring or repeated flash. These are cosmetic motifs; leave node, belt and reward semantics intact.

2. **Give the galaxy mass and hierarchy.** `cosmic_background.rs:267` draws four spiral lanes through an angular sine distance and nearly constant narrow width. Four arms are plausible; the thin, evenly patterned treatment is the issue. Preserve the bounded cached local-cell machinery, but add a faint irregular disk, tapering clustered arm widths, a warmer central concentration and dark gaps. Do not solve it by making the existing wires uniformly brighter. The seeded origin, parallax and identity must remain stable when the camera or quality changes.

3. **Keep the new nebula volume as the useful foundation.** Its multiscale billows, warped dust channel and subordinate hard ridges are coherent in the inspected native software sheet. Retain black negative space, restrained local curl and a small palette hierarchy. All instances currently share one cached source pattern with seeded transforms/tint; a small finite bank of stable variants could reduce repeated silhouettes, if justified within cache and quota limits. Per-frame random noise or full-screen fog would weaken both form and threat visibility.

4. **Refine planets without losing their good sphere/ring structure.** The fixed directional shading in `regions.rs:443` reads as a globe, and back-ring → surface → front-ring order is correct. Cached repeated crater layouts and the thin algebraic crystalline veins (`:498`) are the likely artificial details. Prefer compact mineral/facet clusters and uneven bands while retaining a solid shaded body. Do not introduce decorative straight seams like the rejected asteroid revision. Quality may change sampling, not the seeded object's identity.

5. **Make UI chrome quieter than its content.** Native `ui_scene.rs`/`pixel_ui.rs` paints the real panel and controls; `.pixel-ready` hides their DOM visual fallback. CSS palette-only edits will not change shipped native chrome. Keep unlit crisp text, measured controls, semantic offense/defense/passive colors and main-manufacturer framing. Avoid world lighting on UI, generic glow/blur and redundant nested frames. Sub-manufacturer remains horn styling; sponsor must not determine the pilot/craft palette.

Permanent dotted constellation connectors (`cosmic_background.rs:801`) are a diagram convention, not visible matter. The user previously requested constellations, so this is not a recommendation to remove them automatically. Stars should dominate faint connectors; compare them against course/target lines so they do not suggest an interactive route.

## Images actually inspected

Engine's frozen `native-software-contact.png` models actual native packets, not GPU execution. I inspected its full/reduced, time0/18/36 and zoom0.6/1/1.6 views: muted purple mass and a persistent torn channel remain recognizable; reduced sampling is broader and less detailed. This does not prove browser edge coverage, frame time or human combat readability.

I also inspected Root's four supplied browser baseline JPEGs in `artifacts/space-sports-style-20261012`, with seed **871238449** supplied by Root. Their exact bytes are pinned. I did not operate the browser or independently identify backend, camera, phase or quota from those images.

- **Settings:** labels and controls are legible at 1280×720. The dock silhouette remains visible directly behind slider/checkbox rows, while fine bright cyan outer rails attract attention. Favor more consistent content/background separation and less edge sparkle; verify the actual composition rather than assuming a source alpha proves opacity.
- **Galaxy:** only faint thin broken curves are identifiable behind the ship. This supports the mass/arm hierarchy recommendation, but one frame cannot establish motion or coverage.
- **Planet:** the globe and ring silhouette read clearly. Its bright cyan face directly behind the craft is a concrete foreground contrast case to retain for A/B.
- **Remnant:** no identifiable remnant is visible in this one capture. I cannot claim a missing object or assess its shape without a declared admitted/visible phase and camera.

These observations concern baseline presentation. No candidate or user art acceptance is asserted.

## Physical references and interpretation

NASA's [Whirlpool Galaxy](https://science.nasa.gov/3d-resources/whirlpool-galaxy/) describes arms containing stars, gas and dust, with younger stars in arms and older populations toward the center. NASA's [Hubble Eagle Nebula and M51 images](https://science.nasa.gov/asset/hubble/hubbles-15th-anniversary-images-of-the-whirlpool-galaxy-m51-and-eagle-nebula/) show illumination/sculpting around dense dust structures. ESA/Webb's [Orion Bar](https://esawebb.org/images/weic2315b/) is useful for cavities, filaments and uneven ridges. NASA/Webb's [Cassiopeia A](https://science.nasa.gov/asset/webb/cassiopeia-a-nircam-compass-image/) supports broken knots and filaments rather than evenly spaced spokes.

The proposed game forms are an inference from those morphological references, not an astronomical simulation. Infrared filter colors are assigned representations, not literal naked-eye colors; keep the game's original restrained palette and do not copy telescope imagery or melodies into assets.

## Constraints and acceptance checks

Existing tests were read, not rerun in this review. Engine's retained tests/logs remain attributed to Engine. The review's executed checks are committed-blob parity, input/evidence hashes and inspection of the five images above.

| Area | Keep and verify in the next scoped change |
| --- | --- |
| Counts | Custom background 8,000/4,000, including 1,800/900 star/system reserve; separate region 6,000/3,000, including gas 800/400. The combined background bound is 14,000/7,000 before foreground/UI, not a whole-Game cap. Charge every submitted occupied cell, including alpha-zero cells. |
| Whole groups | Nebula currently 2,128/538 cells, four transforms. Normal planet/rings 5,004 full/1,364 reduced; single planet 4,404/1,152. Gas and earlier layers can legitimately omit later whole groups. Test exact/one-short quotas, prior packets and 127-slot rollover; no floating rings or quality-dependent different-art substitution. |
| Geometry | Borrow cached local cells under composed affine groups; retain one camera subtraction and .08 motif parallax. Test zoom .6/1/1.6, reflected/nonuniform/sheared transforms and actual CameraView corners near ±999k. Nominal full/low footprints agree; occupied extrema need not be bit-identical. No minimum hard-cell size or texture raster workaround. |
| Determinism | Stable birth-space IDs/placement, shuffled query order and seed revisits; Reduced Motion exact packet freeze. Preserve canonical biome/node/belt centers, collision/sanctuary radii, Profile/HUD/events and combat/loot RNG across render settings. |
| Composition | Matched GPU/GL frames at admitted galaxy/remnant/planet/cloud positions, full/low, stationary/moving camera and crowded real hostile warnings/bullets. Inspect cloud/planet against the ship and Mote cue, not just an isolated contact sheet. No broad wash, false telegraph flashes or disappearing silhouettes at min zoom. |
| UI | Actual Settings/collection native packets and browser BCR at wide/compact/portrait: long labels, disabled states, focus, 44px semantic hit areas, clipping and Close clearance. Shared UI cap remains 60k with bounded work; new ornament should not crowd portraits or replace authoritative controls. |

No source-level safety blocker was found in the retained budgets/group path. Style improvements above are concrete authoring priorities; candidate code, actual GPU/GL comparisons, moving combat readability, cold/warm performance and physical-device acceptance remain separate gates.
