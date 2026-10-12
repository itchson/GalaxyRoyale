# MEISHI native admission handoff

Test-only commit `63070b1b95156fedeb683d0ad867781748f990b9`, parent `7d33ee4d24f85cdb5388b505e183f7aa9a8883aa`. Only `crates/game_web/src/ui_kikai_integration_tests.rs` changed (+158 lines); tracked checkout clean.

Actual focused command: `cargo test -p game_web meishi_card_all_profile_types_factory_hulls_and_eight_earned_patches_fit_60k --locked -- --nocapture`. One test passed, 209 unit tests and 31 boss integration tests filtered; no ignored exporter or broad suite executed. Log: `native-test.log`.

Nine actual UiScene extraction fixtures cover all three profile9 types and all three factory hulls, eight earned authored patches, compact headings/body text, selected collection tabs, close and Settings icons, and the existing panel. The complete mixed packet equals the concatenation of every independently extracted component, including transform/material/color bytes. Profile and patch occupied counts use independent authored-atlas oracles; their visible corners remain contained. The unchanged shared cap is asserted at 60,000.

| Profile | PAN hull | RENKO hull | HUGO hull |
|---|---:|---:|---:|
| DORA | 53,645 | 53,671 | 54,023 |
| DORO | 47,575 | 47,601 | 47,953 |
| TALO | 43,313 | 43,339 | 43,691 |

Worst headroom: 5,977 quads. Each fixture includes 2,056 patch cells and 16,333 chrome/text/icon cells; portraits submit 34,594/28,524/24,262 and hulls 662/688/1,040. No cap, runtime or art changes.

Scope is a representative logical native scene with fractional portrait fit at a 96-CSS-pixel equivalent width at scale 2.4, not measured current browser DOM or complete-game acceptance. Root owns subsequent integrated build and browser inspection. No server, full build, GPU/device/performance claim. Earlier non-Clone fixture compile mistake is retained in `first-compile-error.log`; the successful pre-Settings run is retained in `before-settings-test.log` and is historical.
