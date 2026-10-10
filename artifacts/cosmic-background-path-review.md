# Cosmic background integration path review

Read-only author integration audit after the source freeze; Engine's separate independent review is complementary. No production, prior freeze, manifest or dist mutation was made for this review.

Pinned cosmic module: `dee899be884b308fe02d447743f58b7ca3125b53a1cef39c0be78f97f40e1023`. Scoped Game lib: `b79373610953196fb946467ae280a96e1041878187654cdf20b02c6bba887f62`. Native proof freeze: `85b564b8cbc0e4363dd88ce3389f8e1fcf4e1189a0984ffd6e0588553b2e6c50`. Current source-path pins additionally inspected:

| Source | SHA256 |
|---|---|
| engine frame | 4d8b1a965e1e98168dc62db3bf2f3aaf707b6b7be4d9d0b56225a73d0630ebc6 |
| engine camera | b08164aa6c9a7e8364ee90e7205e70b702d2013c1861254da33610cb81fdf759 |
| browser | fec6a493319332e05b871dc712dc448f4650e75e276945372ed3ea12c9ebc9e8 |
| renderer | 046bdac6b14ce98f478e7b5b3acae88de48286168e4fae49c59f2f28de286f12 |
| WGSL | 18ed413c5a88739de5e79a2c9fd741c70f6fdde0e603bce8efe482cd0d585376 |
| lighting | 5fba0a93baa79e7d9f0b62bd2d88ec893c1e766a000544116113544bf2f0d409 |

No concrete blocking source defect was found in the reviewed path.

`Game::render_into` passes the same interpolated unshaken `render_camera(alpha)` to custom and existing region extraction. Browser then adds presentation shake and passes that camera to Renderer. Each cosmic parent encodes parallax once; Renderer uses `CameraView::relative_transform` to subtract its presentation camera before the shader multiplies local cells. Stars have small field-local centers and one affine parent per nonempty field; motifs have cached local body cells and two composed affine children. No tiny absolute-position/size `push_pixel` is used in the new module. Normal shake moves the background with the presentation view, and Reduced Motion disables shake through the existing Browser gate. Decorative edge cells can crop naturally; no hostile-warning visibility guarantee is implied.

The custom budget is genuinely separate from region/Scar: motifs receive6200/3100 and stars submit at most1800/900, proving8000/4000 maximum additional cells. Every occupied cell is charged, including faint/overlapping ones. Both motif bodies are preflighted together before either is emitted; rejection leaves prior cells intact. The existing6000/3000 region cap still applies independently, so the sum can reach14000/7000 background quads before foreground/UI. Renderer retains its explicit total resource limit; this audit does not label8000 as a complete Game ceiling.

`push_cells_affine_material` rejects invalid transforms/cells, adds no transform for an empty stream and splits tables through the same128-slot `add_transform` used by cached bodies. The custom Stats count only successful nonempty star groups plus two transforms per accepted motif. Actual32-frame inspection verifies those counts and unlit material values. Existing pooled FrameBuilder allocation/reuse semantics remain unchanged. Cache initialization and buffer growth are not zero-allocation guarantees.

The finite viewport/camera gate and8192/±1M clamping keep field loops bounded. Conservative180-world motif culling exceeds every authored transformed child extent at the admitted0.8..1.3 parent scales, including low bodies at double local scale and nebula offsets/nova expansion. Local far-world spacing remains intact before CPU camera subtraction; global parent translation can still quantize by a float ULP. At very wide extents hard star/motif budgets can omit later fields/layers and change which identities are admitted as view bounds move. They do not substitute different motif art. Ordinary533×300 and the seeded zoom fixtures stay below those ceilings.

Density and colors depend on immutable birth-space field corners/star positions, not the player's current biome or camera-global selection. Shared corner interpolation is continuous. Current region biome lookup is discrete, but it is sampled at fixed field corners; camera motion does not change those samples. Layer0.09 birth-space blends span broader physical distances than near layers, as expected for the distant plane. This produces a broad visual transition rather than a one-to-one match with the HUD's instantaneous nearest-region name. The deterministic fixture evidence is584 Haven versus2162 outer stars across32 seeds; perceived biome distinction remains a human art judgment.

Reduced Motion reaches the Game setter, freezes host cosmetic time and independently forces the module's local time0. It freezes star twinkle, galaxy arm rotation, gas thread sway/color and nova geometry/opacity; parallax still follows camera movement. Eight full/reduced native packet pairs are exact. Materials0/0 bypass both the new actor ambient branch and local lights in WGSL; stars/nebulae retain their authored palettes and cannot become a large illuminated backdrop. The neutral clear comes from Root's renderer, not a giant quad in this module.

Evidence already completed:9 focused tests, all140 core tests, strict all-target core Clippy,32 actual custom packet frames and24 software art views. The standalone warm benchmark does not measure first-motif LazyLock construction, full Game ticks, WASM/GPU, foreground mixing or phone thermals. No tests/builds were repeated for this read-only audit.

Remaining gates: Root's actual normal WebGPU/WebGL2 package, the three seeded warp centers, actual Reduced Motion/quality toggles, moving/crowded Mote and boss contrast, perceived thin-galaxy detail at minimum zoom, and physical Pixel6a/iPhone12. The software sheet deliberately omits region art, threats and UI, so it cannot establish those composition/readability results.
