# Integrated Haven, private rooms and native score

The normal dual-renderer package was built from `f4b8882b568339f0aa164c6aaf99a174b59cc105` on `codex/testing`. Runtime source hash is `783b68a02f90b0b0ccd7861ce2e11edb5d2b49a3b5e70fe1f8ce9aa4742d872e`; package hash is `a1afe6da521b45b60a6e1eae1c6d0269c8837615ffcee478e168d1457e1e242c`. Build finished at `2026-10-12T17:41:53.879Z`. Later documentation/evidence commits may carry those unchanged bytes with the original source/build facts; they are not rebuilds.

Five cached top-down floating hubs now route Forge, Hangar, MISSIONS, MULTIPLAYER and VAULT. MISSIONS combines expedition/training/lore and the arena league. The central dock does not open multiplayer. Scene art/chrome is native pixel-quad rendering; measured browser controls supply input and accessibility. Hangar has a native overhead garage, an owned hull index and a separate three-slot/statistics scene. Vault distinguishes banked materials/components/mission items and unbanked at-risk salvage. Existing material deposit/death rules and Rust recipes are retained. MEISHI remains a separate collectible card without collection tabs on the card.

Source/model fixture review preserved all 33 original scene cases and added six, passing 39/39 after real initial failures. The first broad integration also preserved stale scene assertions and a strict-Clippy duplicate-color-branch failure. Root fixed the semantic fixtures and branch duplication without changing art colors. These failures remain in the earlier checks directories.

At D028, combined native tests passed 898 cases with 17 explicitly ignored cases; strict native and both WASM Clippy and formatting passed. No crate source changed from D028 to F4; both compiled WASM addresses remain identical. At F4, 635 shell cases passed with three declared skips, typecheck passed and the normal WebGPU/WebGL2/Vite/PWA/Studio build completed with frozen before/after identity. See `checks/combined-20261013/` and `../title-score-20261013/width-final/checks/retry/` for commands and raw results.

## Actual browser operations

One normal game preview was used, with backend changes in the same tab. Studio and local signaling remained separate services. Temporary audio QA was stopped after capture. Tests used actual input/accessibility controls; no hidden application API, inventory injection, Warp or artificial award was used on the saved profile.

| Scope | Actual observation | Limit |
| --- | --- | --- |
| D028 WebGL2 / 1280×720 | PWA update; five facilities; native Hangar garage, owned hull and details; Missions training and local League; Forge discovered abilities | No natural purchase, craft, collection or champion run |
| D028 WebGL2 / 848×480 | Forge/Missions/room scene navigation; one actual host created, copied/replaced code, locked joining and stopped hosting; 1/8 displayed | No second client, approval, full room, failed ICE or TURN run |
| D028 WebGPU | Both-backend initialization; native garage; safe ordinary home flight above hub to `[29.07539,-24.22950]`; live simulation continues in menus | Initialization/home flight are not accepted shadow/parity/recovery/performance tests |
| F4 WebGL2 | Activated waiting PWA update through Settings at Haven; actual shell `index-C34zheRV.js`; Vault 1280×720 and 848×480; Hangar→details at 848×480 | User's muted audio settings were preserved; live state is not direct audible acceptance |
| F4 WebGPU | Same current shell, Vault 1280×720, MEISHI and Join-default Multiplayer scene; native score clock settles at PAN Haven 120 BPM; world `paused:false` | Not full all-menu/all-aspect/device acceptance |

The D028 Vault initially rendered as a 760×650 popup due to an obsolete specific Cargo CSS rule. Its screenshot remains `runtime/vault-webgl2-1280.jpg`. Root removed that obsolete rule. F4 DOM bounds measured `[0,0,1280,720]` and `[0,0,848,480]`, with an opaque full-screen native backing and visible top-right close. Corrected screenshots use `vault-final-*`; they do not erase the initial defect. MEISHI's separate inner card framing was observed after this change.

Before and after the PWA update/current scenes, displayed retained points were RENKO 16, HUGO 12 and PAN 9. Current PAN HIKARI remained level 3 with 119/120 XP, hull 100/100, no equipped defense and Flux Drive R1. Vault displayed Phase Silicate 9, Iron Filament 6, Coolant KESSHO 9 and other T1 6, with no carried material, components or courier. This is an observed UI continuity check, not an independent IndexedDB digest or a material-death/purchase acceptance run.

Raw `*-report.json` files are DOM-exposed live telemetry. Their short desktop frame samples are not a physical-phone target, a 30-minute soak, sustained eight-player performance or a memory-leak acceptance claim. Screenshot bytes are unmodified browser captures.

The new native title synthesis, three auditions, source reviews, actual PCM and pending fidelity gates are in `../title-score-20261013/`. Relative hub-shadow source/CPU geometry evidence and its independent review are in `../hub-shadows-20261013/`; GPU appearance, receiver gaps, hole/recovery/zoom/light behavior and performance remain open.
