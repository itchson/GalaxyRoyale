# Detailed native shooting star

`crates/game_core/src/shooting_star.rs` provides one pure presentation group: an original overhead ice nucleus, a sparse coma and a seeded particulate tail. The nucleus has broad mineral facets, recessed grooves/crater shading and a heated leading rim. Its 48×32 authored cells fit at fractional scale 0.25. There is no minimum hard-cell size or coarse head LOD.

The group submits **644 nucleus +176 coma +64 tail =884 cells** in full density and **852 cells** with 32 tail particles in reduced density. Both modes retain all head/coma cells and use four transforms. Whole-group preflight charges the existing caller budget once; rejection emits neither an orphan tail nor a partial head. Alpha-zero submissions at the fade edges are included. Fixed-size arrays and cached PixelBodies avoid a per-frame particle allocation.

The existing layer-2 field hash selects the original angle, 80-second cycle, 18-second pass and 22-world-unit/second motion. Parallax remains 0.48, encoded once from the unmodified birth origin. The host supplies the eased world viewport; zoom changes projection rather than authored occupancy or scale. Relative tail particles move through deterministic age/side oscillation and remain within the same head group. Reduced Motion freezes the cosmetic clock, including head motion, colors and tail positions. The runtime uses existing hard and radially filtered soft extraction with emission 0.18–0.26 and no light response or new light source. It changes no gameplay, save, combat RNG or audio state.

The particle clock now uses the same seeded phase as the head. Previously, dust reset at absolute 80-second boundaries while the head could still be mid-pass; visible dust jumped despite continuous head motion. The correction resets the trail only at the seeded cycle's invisible boundary. Native continuity checks cover full/reduced density and exact Reduced Motion packets; the clock change adds no allocation, entity or transform.

## Native integration

The group is registered in the native core and extracted from the existing layer-2 field. The following records its original integration contract; the clock refinement changes no registration or caller behavior:

```rust
crate::shooting_star::extract(
    scene.frame,
    &crate::shooting_star::SceneInput {
        origin, // Unencoded seeded field anchor, not `position` or `head`.
        camera: scene.camera,
        viewport: scene.half * 2.0,
        time, // Game already supplies its frozen cosmetic_time().
        birth_hash: h,
        tint: biome.palette,
        reduced,
        reduced_motion: false,
    },
    &mut scene.remaining,
);
```

Keep the original seeded-field/reduced-density gates. Use `shooting_star::STREAM_RADIUS` (300 world units) only for the layer-2 tile query instead of its existing 180-unit margin. The unchanged head travels ±198 units, and the tail extends another 89; this margin includes its entire visible pass. Other layer margins stay unchanged. The extractor's `LOCAL_RADIUS` is 96 around the moving head. Existing region material test oracles must allow the explicit bounded soft encodings `-1.26/-1.18/-1.20` and hard head emission `0.18`; this is actual emission rather than an unlit `0/-1` packet.

## Reproduction and evidence

The source PNGs are original authored art. `scripts/asset-shooting-star.py` creates their exact binary-alpha pack through `asset-pack.py`, using the existing pinned Pillow requirement. No resize, threshold, dithering or component removal occurs. The pack/provenance lives under `assets/pixels/shooting-star/`.

```powershell
python -m pip install -r scripts/asset-requirements.txt
python scripts/asset-shooting-star.py
python scripts/asset-shooting-star-tests.py
powershell -NoProfile -File scripts/asset-shooting-star-native.ps1 -Python python
```

Two Python checks verify exact indexed round-trip and deterministic authoring. Ten native checks cover detailed cached geometry, original timing/heading/speed, atomic budget boundaries including zero-alpha fade cells, exact reduced-density head preservation, actual emission, fractional cells, parallax/zoom, visible Reduced Motion freezing, invalid/far inputs and the 128-transform batch boundary. The scoped native script compiles the real module/engine, runs those checks and strict library Clippy, and extracts eight posed frames through the actual `FrameBuilder`/`CameraView` APIs.

`artifacts/shooting-star/native-contact-sheet.png` is a supersampled flat alpha-composite of those native packet corners, with a magnified head inset. Its material brightness approximation does not reproduce GPU radial coverage, linear blending or actual moving browser playback. `validation.json` pins the actual inputs, counts and test logs. Normal registration/dual-WASM integration and browser/human/physical-phone readability remain Root-owned acceptance gates. This scoped pass does not claim the broader whole-game art overhaul is complete.
