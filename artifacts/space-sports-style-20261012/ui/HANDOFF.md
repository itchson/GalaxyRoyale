# Native sports UI skin — frozen handoff

Source commit `4d032e99bfdf1da056ba72516667c0f7745bd246`, branch `codex/sports-ui-skin`, based on `4e014bf`.
The commit changes exactly six native chrome/palette/test files, listed in `manifest.json`.
No title art, MEISHI, portrait data, flight icon atlas, measured hitbox, shell, gameplay, shader, or budget ceiling changes.

## Implemented art

Common panels now have opaque neutral graphite backing, original cached cut-metal corners,
layered platinum/alloy rails, recessed fasteners and quiet gold locator pins. Buttons retain
short semantic/corporate enamel accents instead of repeated bright frame washes. Idle
Settings rows have only a bottom separator; focus adds a quiet graphite surface and fine
chevron. Heading/body text uses a coherent platinum/slate palette. Existing 64-cell icon
art and masked state animations remain unchanged.

The panel corner is 24×24 / 288 occupied cells; button corner 12×12 / 83; focus chevron
6×6 / 12. A complete panel submits 1,168 cells and a complete button 348, including at
tiny fractional fits. No compressed-detail LOD or cell-size floor is introduced.

## Contract and admission

`sports_ui::append` uses a fixed 32-part local stack, cached borrowed bodies and whole-element
pixel/work preflight. It charges clipped alpha-zero source cells, rejects invalid/zero/collinear
parent affines without mutation, composes reflected/sheared parents, and clips fine cells by
all four local corners. Stretched backing/rails intersect the semantic clip so a partly
scrolled surface remains opaque within its visible bounds. Labels and control rectangles
stay under their existing independent measured routes. Warming the art avoids repeated
body allocation; cold LazyLock initialization and FrameBuilder capacity growth can allocate.

The UI limits remain 60,000 pixels / 120,000 work; 128-transform batching is unchanged.
Actual realistic five-icon/largest-DORA scene: 47,360 cells / 262 transforms, with exact
joined-versus-isolated component equality. Existing dense five-icon interface: 13,934 cells;
with DORA/DORO/TALO: 48,528 / 42,458 / 38,196. All ten controls plus DORA now fit at
55,820. Deliberately doubled ten-control interface is 42,452 and rejects each whole portrait
atomically when the total exceeds 60,000. The title fixture retains all 44,985 logo cells,
copy and controls at 49,021 cells / 189 transforms.

## Actual verification

At the final committed source:

- `cargo test --locked -p game_web --lib`: 258 passed, zero failed (`web-library-committed.log`).
- `cargo test --locked -p game_web sports_ui:: -- --nocapture`: seven passed (`focused-native-final.log`).
- `cargo clippy --locked -p game_web --all-targets -- -D warnings`: exit 0 (`clippy-final.log`).
- Twelve existing semantic button checks and the title joined fixture passed before the
  final determinant admission guard; the final 258-test run includes them again.
- `git diff --check` passed before commit; the checkout was clean afterward.

`packets/menu.json` is actual registered native UiScene / FrameBuilder output. It was exported
before the final determinant-only guard; the fixture's valid identity parent and packet geometry
are unchanged, and the final focused run independently records the same 47,360 / 262 counts.
`native-software-menu.png` renders that packet with 4× supersampling and BOX downsampling,
using flat unlit alpha. This is native packet/software evidence, not a browser, GPU, actual DOM
layout, physical-device or subjective readability result. Root inspected this software image.
Root owns fresh normal dual-backend browser validation, including compact Settings scrolling.

Gameplay supplied an actual source review of stack/preflight/clip/palette behavior. Its draft
singular-affine caveat was addressed by the final guard and negative regression; its final
independent frozen verdict is a separate peer record, not assumed here.

## Scope remaining

The common material treatment is implemented. It does not mean every world asset has been
reauthored to the approved style. Fresh GPU/WebGL presentation and user style acceptance,
physical phones, sustained play and a 30-minute soak remain outside this handoff.
