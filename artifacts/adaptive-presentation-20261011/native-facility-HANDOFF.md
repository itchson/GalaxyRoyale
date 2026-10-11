# Native facility-label placement handoff

Production ownership is frozen: only ui_scene.rs and new ui_facility_label_tests.rs changed. The branch uses the existing finite/floor/clamp selection helper:0below (the original exact packet),1above at rect.y-12,2hidden/no glyphs. The label remains100x10logical, horizontally centered on the measured facility rectangle. Font1 and unlit cached RoyaleMono bodies are unchanged. Nonfinite values select0; no body/hit area, bridge, main, CSS, renderer, physics or game asset was edited here.

Four actual native extraction tests pass: the retained original below-label test plus three new tests proving occupied corners fit above/below rectangles, font1 packet equivalence with arbitrary node.font, no hidden transforms/cells, nonfinite/default packet parity and a bottom-facility fixture where above clears the footer. Strict native all-target game_web Clippy passes. Actual command/log/source snapshots are retained here. This is native geometry evidence, not browser acceptance; Root owns final normal build and responsive full-label placement selection.

Previous a9c3/c707 package reviews remain immutable historical evidence. A new final package audit must include these changed Rust inputs and the Root shell label-placement changes.
