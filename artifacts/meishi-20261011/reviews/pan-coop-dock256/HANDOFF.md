# Detailed PAN dock candidate — frozen

Commit: `92fb9f677a94c6740b6a14380a7dd403219795b2` (isolated codex/pan-coop-dock-candidate).
Prior frozen128 dependency: `a32434bf93c8dbfcc812880043ab4d8a617f0e2f`; all prior60 pins still verify unchanged.

Candidate only. No shipping dock/runtime/Game/Studio/API/browser/server changes; no push.
Exact source1254 PNG retained byte-for-byte, SHA eab5fa7c3b604451fdb5fadff75c2138d614aa8d8bbcedd31402527c59380918.
Original provenance promptFile `prompt.txt` maps to copied `generation-prompt.txt`; new native256 README explains this explicitly.

## Actual checks

- 11 Python tests passed, including exact committed authoring regeneration, palette/alpha/source occupancy/material partition, same-occupancy lamps, tile seams/cutouts/landmarks, PNG roundtrip and lossless archive corruption checks.
- 5 actual native PixelRig/FrameBuilder/CameraView tests passed: 28,549 occupied cells /16 transforms, all-corner nested reflection/shear, continuous zoom, real damage holes, whole-group quota rejection, 128-transform batching.
- Strict all-target native Clippy passed.
- Existing native PixelBody utility emitted3 actual states at0/700/1400ms; exact source-color/four-corner audit has maximum error0.
- Actual4PixelRig extraction retained3 color states; actualCameraView projected state0 at0.6/1.0/1.6, viewport320x240. Software previews viewed; no GPU raster/lighting claim.
- Read-only verify checks107 inputs, all3body/3rig/3projection packets and exact decoded hashes. Prior128verify60 also passed.

The initial added archive-corruption test caught an import-local exception-class fixture mismatch; corrected expectation then all11 passed. Failed log is retained separately as python-tests-archive-fixture-error.log. This did not change art/pack/proof bytes.

## Artifact pins

- `pack.json`: `36031366f135b12b290f59e3f2c382e56774f9663e5fdb5241584051ad24832b`
- `validation.json`: `c8b6ee428fa0343a1a44faa9711748605e151eba58a1658c7b34e3726a65ddc1`
- `archive.json`: `3d1645e6d3d40ab4b9c9ade2e685c5d372a733bafb1a7e9220e71cef0c1e48b6`
- `authoring-states.png`: `1334b7471aad3dd378b627ee70d5e7a625a42e4a9fd587c236edc69ca817e535`
- `native-preview.png`: `77fc314215a6bd054b08428c535b37c05c0e5df290cf4fa89efb08acab75f5aa`
- `zoom-preview.png`: `19f24d866a37243d98466b4ca267d96ef642343558a2ea3ffbcd16138d0dab19`

Three native exports retain exact `35638621` decoded bytes in `2758393` compressed bytes with timestamp0/empty-name gzip headers. Raw originals remain under work/pan-coop-dock256/raw, and committed archives include decoded SHA/length. No proof rows were removed. Compressed bytes are pinned; cross-zlib encoded equivalence is not claimed.

## Runtime boundary

The detailed256 source is split into4 native128 quadrants ×4 logical material groups, same64 palette, parent scale0.5, reference bounds[-63,-63,63,63]. Local proof host limits32000cells/16transforms and extent1000 are not new engine/global caps. Proof utilities rebuild bodies/rigs offline; any future runtime adapter must cache them and preserve actual damaged cells. Lamp host policy selects explicit700ms states and Reduced Motion freezes state0/time0. All sprite/color conversions are declared lossy; native-cell import is exact after that conversion.

Native256 preserves materially finer vents/rims/pad/crystal structure than128; this is source/software specialist review, not final human quality approval. Shipping/collision/interaction integration, near/far GPU presentations, live animation/cue readability, co-op operation and physical phone/performance remain pending.

## Additional CI commands

```text
python scripts/qa/pan-coop-dock/test_256.py
python scripts/qa/pan-coop-dock/verify_256.py
cargo test --locked --manifest-path scripts/qa/pan-coop-dock/native256/Cargo.toml
cargo clippy --locked --manifest-path scripts/qa/pan-coop-dock/native256/Cargo.toml --all-targets -- -D warnings
```

Existing128commands remain unchanged. Work logs: python-tests.log, native-tests.log, clippy.log, body-run.log, rig-run.log, verify.log.
