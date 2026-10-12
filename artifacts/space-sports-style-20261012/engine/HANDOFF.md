# Background immersion handoff

Frozen source: `61492ce2d3e360375c2a3eb047ca330fd4c937c4` on `codex/background-immersion`, parent `4e014bff3a84b356c5254b4291560e38accef64d`. Five owned Rust files; tracked checkout clean. Root owns integration and actual browser acceptance.

The galaxy now combines a cached, inclined, irregular stellar disk with an absent dust band, tapered logarithmic arm clusters and a warm bulge. It stays stationary apart from small local curl/color breathing. The final revision adds real interior mass after Root and Gameplay rejected the first wire-like software result. The final contact sheet was inspected by Engine and Root before freeze.

Cosmic remnants retain torn filaments and a compact offset source; their whole-body rotation is removed and scale varies only 0.99–1.01. Regional remnants replace the opaque glowing disc and 24 equal radial spokes with sparse open filaments, fixed geometry and alpha65–73. The old two-color, every-third-cell regional ribbon is omitted in favor of the existing approved gas. Seven exact seeded stellar neighborhood positions remain; chart connection strokes are removed (7 cells/1 transform per admitted neighborhood). These are intentional presentation changes.

Planets retain the same spherical silhouette and full/reduced occupancy. Albedo texture is shaded before lighting, so craters, atmospheric belts and ice fields cannot brighten the night side. Natural type ramps replace saturated biome-only shading; biome contributes a restrained hue bias. Ring order remains back/body/front, and a complete group is admitted atomically. Ring material/brightness is quieter and avoids periodic shade bands.

| Complete group | Full cells / transforms | Reduced cells / transforms |
| --- | --- | --- |
| Galaxy | 1,826 / 3 | 454 / 3 |
| Approved deep nebula | 2,128 / 4 | 538 / 4 |
| Cosmic remnant | 857 / 2 | 210 / 2 |
| Rocky / ice planet | 4,404 / 1 | 1,152 / 1 |
| Ringed planet | 5,004 / 3 | 1,364 / 3 |
| Regional remnant | 323 / 1 | 141 / 1 |

The galaxy disk is389/103 source cells, 28×28/14×14 grids at2.5/5 world spacing; radially filtered footprints6/12 before the shared0.62 inclination. It uses existing soft material with light_response0/emission0 (encoded tag−1), no new shader/light. Nominal disk canvas70×70 and parent/parallax are identical across quality. Lower sampling may broaden occupied edge/cavity cells; nominal footprint is preserved, exact occupied-corner equality across quality is not claimed.

Custom8,000/4,000 and reserved stars1,800/900 stay unchanged. Regional6,000/3,000 includes the existing800/400 gas quota. Galaxy/remnant fit the existing2,200/1,100 whole-motif ceilings. Reducing one object's cost may permit a later existing selection to fit; this is not baseline aggregate packet parity. No simulation, combat RNG, node/belt, mission, collision, save, global biome or shader changes. Region authoritative prefix and seeded stellar-position function compare byte-exact in LF domain. Approved nebula source sections are equal; all emitted corners/colors match baseline in4 actual full/low/time0/18 packets (12 zoom views).

Actual checks: full core384 passed,0 failed,3 explicit exporters ignored. This run preceded only QA expected-float precision and export scope-text cleanup; final two native QA selection assertions pass after cleanup. Final native all-target and core WASM library strict Clippy pass. Both explicit final exporters pass at this commit. Meaningful checks cover night-side darkness, all-biome maximum channels, atomic quotas/batch boundaries, reflected fractional planet transforms, far-origin bounds, exact7 stars, deterministic revisits, Reduced Motion packet freeze, calm galaxy/remnant basis changes and actual seed selection. Earlier failures were obsolete old-art cost/color assertions and a missing test-only Vec3 import; fixed without removing atomic/order/coverage checks. Native Clippy initially rejected3 overly precise QA literals; final fixes retain identical f32 expected coordinates.

`native-software-contact-final.png` has40 displayed views from actual packets: baseline/new galaxy and cosmic remnant plus four regional helper forms at zoom0.6/1/1.6. Native exports are posed helper outputs, not complete Game scenes. The offline linear-alpha/radial filter model reuses the earlier owned raster function and does not execute WGSL, world lights or browser rasterization. Final32 packet sets are pinned; 96 stored zoom views. The first `native-software-contact.png` and `after/` remain preliminary history, not final galaxy evidence. No GPU, phone, motion contrast, cold-cache latency, RSS or performance claim. Root captured baseline live objects at the same seed/camera and will compare final WebGPU/WebGL2.

Actual native QA seed871238449 (camera=unencoded origin/parallax):

- Galaxy grid[3,-4], hash2591400577, `WARP 59725.0078 -49541.4727` (.08).
- Cosmic remnant grid[-3,1], hash1888154543, `WARP -35332.8203 22642.1445` (.08).
- Ringed planet grid[-1,1], hash2928467406, `WARP -1243.7285 5313.4268` (.22), birth biome0.
- Historical ribbon grid[-1,1], hash2130583060, `WARP -12094.4385 26339.6113` (.075); ribbon omitted in candidate.
- Regional remnant grid[0,1], hash885732694, `WARP 4949.4165 30685.7207` (.075). Baseline flashed only during its12/90s phase; candidate is calm and continuously present.

Read-only verification: `python work/background-immersion/verify.py`. Focused native checks: `cargo test -p game_core cosmic_background --locked` and `cargo test -p game_core regions:: --locked`. These do not execute ignored output exporters. No services/preview were started and no Root source was edited.
