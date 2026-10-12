# Bounded league-panel hierarchy review

Reviewed the persisted `artifacts/co-op-native-integration-20261012/panel-webgpu-wide.jpg` and current sports mark `assets/ui/title-mark/source-cells.png` using `view_image`. The screenshot is Root's actual 1280x720 WebGPU capture from runtime `48bec87d504dc81ecf768fa22525d29cff8d6f37`, as identified by its artifact README. It is not a newly observed browser frame or evidence of the current dirty co-op source. Root HEAD at source inspection was `3fae2b0c86604cb0d7e685f26dfa12e2a690280d`; `panel.ts` was dirty and evolving. Only this work review was written.

## Observations and concrete next-pass recommendations

1. **Calm material backing now works.** The saved frame has opaque neutral graphite, a cut platinum perimeter, a small cyan header cue, clear platinum body text and quieter disabled JOIN. The world no longer bleeds through this captured panel. There is no visible heading/Close/input/button/footer clipping in this one wide frame. These observations do not establish narrow-layout, hover, moving-scene or universal readability acceptance.

2. **The primary path is diluted by repeated copy and oversized empty space.** `HAVEN CO-OP`, `CO-OP AT HAVEN`, the host/eight-pilot description, and the invitation instruction all precede the actual input/actions. Roughly the bottom third of the full viewport card is empty. INVITE and JOIN are each almost half the 1280px screenshot width despite their short label/symbol. Keep the shell's close and fixed footer contract, but use a co-op-specific compact card or a bounded inner column; one heading, one phase/membership row, and one short instruction are enough. The global `.menu-card` width/height at `style.css:280` explains why this short form inherits a much larger collection surface. Do not reduce actual hit targets to achieve density.

3. **The input-to-action relationship could be clearer.** The full-width invitation field sits above the shared INVITE/JOIN row. It visually appears to belong to both actions, although INVITE creates a new session and JOIN consumes the field. `panel.ts:344` builds this exact order. A distinct compact INVITE action and an input-with-JOIN group would communicate the two routes better without a new tab or extra frame. Preserve opaque-code privacy, full input value/caret clipping, bounded admission states, and explicit copy only after a real invitation exists.

4. **Sports identity should remain concentrated.** The forward athletic title has layered platinum faces, graphite recesses, cyan orbital energy and a gold award. The panel's neutral surfaces and angled corners belong to that family; the tiny header facet is very subtle at this captured size. If another art pass is needed, strengthen a short header shoulder/fitting with a single restrained award cue rather than repeating full-bright rails around every control. Keep body text and actionable states primary; corporation/category accents must not wash the whole background.

The saved footer guidance still says factory guest RAICHO, while the current dirty `panel.ts` now says `Native RAICHO stats; primary fire only...`. This is a historical source/build boundary, not a newly reproduced copy defect. Current members/roster counts derive from actual controller admission in the inspected source; this screenshot shows an idle 0/8 state and proves no RTC connection or eight-player capacity.

## Source checks and limits

Read `sports_ui.rs` common panel/control construction, `ui_scene.rs` routing, Root `pixel-ui.ts` measured label and co-op-art ordering, `style.css` menu/co-op geometry, and `coop/panel.ts` semantic controls/state. The panel backing is explicitly neutral `RECESS rgb(18,18,18)` and panel focus cannot invoke the control's strong color. Text and hitbox geometry are independent of chrome. No concrete source change is proposed as a blocker from this bounded screenshot review. Source tests were not rerun because no source was changed; prior author/Root tests remain separately attributed.

This review does not establish final co-op gameplay, saved-profile restoration, network delivery, compact geometry, user visual approval, physical phones, performance or soak acceptance. Root owns the next actual browser captures. It recommends a smaller hierarchy around the existing truthful state and detailed native objects, not further runtime architecture.

## Exact inputs

Raw SHA256 values are appended from the read inputs; the dirty source pin is a snapshot, not a frozen release declaration.

| Input | Raw SHA256 |
| --- | --- |
| `artifacts/co-op-native-integration-20261012/panel-webgpu-wide.jpg` | `f5d42a2c78b74a32e2caf1963007b8573bb8d3231b968728d33fd1d0a0b259dd` |
| `artifacts/co-op-native-integration-20261012/README.md` | `fe0c7e5f310d4f27d9b8c6a53e1769fa9baa1a6db25476f04d4b31412f900ac1` |
| `assets/ui/title-mark/source-cells.png` | `3e30eb0790702e0fe3cb14a3935b324d71381077db93b69c808c81385695d372` |
| `assets/ui/title-mark/README.md` | `b0d70b67f2b564499e0721302911848765fdc17075d16679c5d9860832608524` |
| `crates/game_web/src/sports_ui.rs` | `86d898fca39c088ef3a0635aaf26325cdaa80188dc8e822b226c70d9f00ed32f` |
| `crates/game_web/src/ui_scene.rs` | `53586794642d5076e8c6167c619a122499e5fae6030b846e706a2a23a2fbec37` |
| `apps/game/src/coop/panel.ts` | `b8ea9d72228de967311ea05489a7cd805fb0e72eed89a142eee5295138f3a12e` |
| `apps/game/src/style.css` | `87b6b686b9dec5e238bb09f505d25c54488aec477c363e12425cdc5aa24c50a6` |
| `apps/game/src/pixel-ui.ts` | `5f83c4e60b752011e267aaf96f23d00d8c92ce1ccb3779b82c8ae346250d4521` |
