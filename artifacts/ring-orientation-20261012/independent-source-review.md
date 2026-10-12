# Independent seeded ring orientation review

Candidate `cf73783f4b9fabc8295562ac500f6c317315c279`, exact parent `650a546b8f8c07150d4df27f8de7cb1d4e13d4e8`. QA inspected the committed three-file patch, supporting affine/group/camera code, actual author logs and six paired native packet exports. QA independently verified all **89 author manifest pins**, including **24 exact Git pins**, and checked **19,104 ordered paired cells / 229,248 projected corner samples** using a read-only packet checker. Its actual pass output is `check.log`; raw check details and pins are `verification.json`.

No native test, compiler, browser, renderer, listener or software image regeneration was run by QA. Author test results remain attributed to the author. Root owns actual normal-build and browser acceptance.

## Verdict

No blocker found for integration of this bounded source/geometry change. The only runtime change replaces type1's fixed `-0.2` parent rotation with a deterministic hash-derived value in `[-0.45, +0.05]` radians. It rotates the entire cached ring-back/body/ring-front group in the sky plane. It does not alter physical inclination, shader lighting, silhouettes, cache detail, palette, or animation.

The unchanged `planet_group` applies the same uniform transform to all three existing group members. Back-ring alpha120, body230 and front-ring145 remain in exactly that ordered sequence; every actual local coordinate, RGBA and material field agrees with the before packet. Only four linear affine coefficients change. Independent packet checks establish equal orthogonal basis lengths and a positive determinant, so there is no shear, reflection, translation shift or cell-size clamp. Full/reduced modes use the same angle, with the intentional twofold reduced-cache scale preserved.

Birth hash/grid, biome selection, `.22` parallax and camera-relative subtraction remain unchanged. The transform is independent of cosmetic time, user settings, saves and simulation RNG. Rotation preserves the radial footprint. Actual largest whole-cell corner radius across the six groups is below39 world units; existing conservative source radii are at most43.898, well below the unchanged135 culling margin. Grid sampling still uses its existing180 margin. No culling expansion, new allocation, new transform, cosmetic/gameplay cap increase or density-dependent identity was introduced.

The existing preflight group cost remains **5,004 full / 1,364 reduced cells**, against unchanged regional6,000/3,000 budgets. Rejection is still atomic before any back/body/front member is submitted. Three retained transforms are confirmed by the new admission test source; packet exports show the equal packed affine records rather than an orphan ring.

## Test evidence and coverage

The new angle test extracts actual `background_with_density`, with three independently fixed birth hashes and numeric expected angles. The frozen baseline failure reports actual `-0.2` at the first selected birth, then the author candidate run passes. This is meaningful regression evidence, not only a duplicate formula oracle. The full/reduced pairing and time20/time91 exact packet comparison cover stable identities and non-animated rotation. The other two new tests cover ordered pixel/material geometry, fractional projected cell widths atzoom.6/1/1.6, finite corners, existing nominal bounds and exact cost-1/cost admission boundaries.

The existing reduced whole-packet oracle initially failed because its expected angle remained `-0.2`. Only that expected parent expression changed; its full ordered comparison was retained. This preserves that broader ordinary-biome/type oracle rather than loosening it. The frozen first oracle failure and Clippy test-helper tuple warning are retained; a test-only type alias fixed the latter without suppressing lint. The author final logs report three new focused tests passing,35 region tests passing with two explicit exporters ignored, strict Clippy passing and format check passing. QA read and hash-verified those logs, without rerunning them.

These tests prove the selected native geometry/contracts. They do not statistically establish attractive distribution over every seed, nor do they prove GPU alpha composition or a visible difference at every fractional screen scale. The second and third selected new angles are close (about+0.564° and−0.492°), and the common storm surface remains shared.

## Visual scope

QA actually viewed the author's native flat-software contact sheet and Root's saved pre-change WebGL2 A/B stills. The contact shows the intended modest whole-group orientation difference with retained ring/body occlusion. Root's before views show both naturally seeded planets at the same original orientation, left of the player, with consistent baked shadow and readable ring silhouette.

No actual post-change browser still had been supplied when this review was frozen. Therefore source/packet acceptance is separate from visual acceptance. The software contact centers isolated native groups and uses supersampled flat-alpha polygons; it does not execute the shared GPU shader, foreground world, HUD, dynamic gas or game camera. Root's matched normal-build WebGPU/WebGL2 captures and art judgment remain required before claiming an improvement to real displayed immersion. This change addresses fixed orientation repetition; it does not establish diverse albedo/physical inclination or resolve every deep-background art concern.
