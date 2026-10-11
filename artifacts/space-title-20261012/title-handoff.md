# Native sports-league title freeze

Source commit: **af78ce9ddbe77f621c648f244447fee6a1956e41** on codex/galaxy-title-mark.
The worktree is clean. Root can cherry-pick the single scoped commit; existing
logo routing/8:3 shell contracts remain. The title-backdrop arm retains Root's
requested alpha246. No shell/audio/shader/Studio edits or preview server.

Actual built-in imagegen reference c995256c188af6c24ec94f0a062d17c0c8aa4b12fb73fef62b175474159b52a2
is retained byte-for-byte in assets/ui/title-mark/reference.png. Model/version
was not exposed. Root inspected its sports identity and prepared source field.
The asset README/provenance/prompt are current; ee16cea's fantasy mark and
work/title-mark evidence are explicitly historical.

Final atlas SHA256: **503fd06162f9c9d99c51596a202d5b7af856742979e6a335011cc2d7af83b15f**.
Source512x192: **44,985 occupied cells /9 transforms**, below45k. Eight static
tiles contain44,520 cells;465 gold award-face cells move at most0.35 source
cell. There are no additional glitter objects. Animated masks: energy6704,
metal scan24280, award465. Letter/stadium geometry remains static, with
independent actual corner probes. Energy/award channels stay within7%/6%; the
metal sweep occupies only12 source columns. Reduced Motion freezes both colors
and transforms. The complete joined title fixture retains all isolated
backdrop/copy/Launch/Settings/Quit components: **48,894 cells /157 transforms**.
The unchanged60k/120k UI quotas charge all submitted cells, even clippedalpha0;
whole-logo preflight rejects atomically. Batch splitting beyond128 transforms
is tested, not rejected or used to lower detail.

Explicit lossy conversion:2048x768 ->512x192 nearest sampling, alpha>=128,
maxRGB>=40 dark cutout,63 opaque quantized colors+transparent0. This removes
13,080 near-black sampled cells, with61,052 sampled alpha changes and RGB-channel
MAE3.7624096921/max77. No component-size filter or lossless reference claim.
Exact index/RGBA union is preserved after conversion. JSON writers and local
asset-folder Git attributes are LF-canonical; compression variants reuse PNG
bytes only after independent complete decoded RGBA/dimension equality.

Actual final checks, exit0:

- cargo test --locked -p game_web title_art:: -- --nocapture:9 passed.
- Pinned Python3.12.10/Pillow12.3.0 asset-title-mark-tests.py:7 passed.
- cargo clippy --locked -p game_web --all-targets -- -D warnings:passed.
- git diff --check:passed before commit.

native-tests.log preserves the initial superseded one-sixth animation-bound
failure. The new source energy mask is larger; final tests separately prove
the material amplitudes/stripe and bound total changed colors below20%
(actual7887/44985). No runtime amplitude was increased to satisfy that test.

Current native packet exports:packets/still.json,scan.json,award.json. They were
produced by the registered Cargo test at the final source commit. The software
sheet is an actual packet-derived4x supersample/BOX flat-unlit approximation,
inspected locally, not a GPU capture. It keeps readable athletic lettering,
the stadium/award crest and negative-space wing silhouette in all three poses.
manifest.json pins24 committed source files and10 retained evidence files;
all raw/committed inputs match modulo explicitly recorded text CRLF handling.

Root owns dual-WASM build and actual title/viewport/animation checks. No
WebGPU/WebGL2, subjective user approval, concept-perfect parity, performance,
physical-phone/soak, music or listening acceptance is claimed here.
