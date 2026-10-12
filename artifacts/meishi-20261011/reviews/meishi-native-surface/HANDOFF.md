# Standalone MEISHI native surface

Commit `e77298c51daebc8180c391aa352b1fd14b9af587`, parent `e6be2e847bedd1e477e4f3576b7e6bf654f7492b`. Only `crates/game_web/src/ui_scene.rs` changes, including two inline tests. Root integrated it as `293f41c4002c45b82e8c1cc62e968104fc11f8ea`.

`meishi-panel` uses the existing UiNode/UiGroup ABI and relative unlit quad utilities. Value 0/1/2 selects PAN green (109,170,113), RENKO red (195,113,107), HUGO blue (89,148,191); nonfinite values default PAN. Five quiet black fill strips (4,7,10), eight single-border/stepped-corner strips (35,46,55), and one short corporate trim total 14 quads. Fractional/tiny extents proportionally fit the same shape. Both quotas preflight 14 before any append; rejected work preserves earlier frame data. Partial clips charge all 14 attempted units and submit only surviving intersected rectangles. Shared 60k pixel/120k work caps unchanged.

Actual command `cargo test -p game_web meishi_panel_ --locked -- --nocapture`: two tests pass, 210 unit and 31 boss tests filtered, no ignored exporter/full suite/build/server. Tests cover all corporate trims/nonfinite default, translated relative group with nonzero camera, all four corners at fractional and tiny extents and partial scroll clip, unlit/opaque materials/open external corners, and either-quota exhaustion with existing packet retention. Actual log `native-tests.log`.

This is native source/packet proof. Root owns combined shell checks, normal builds, measured DOM/both-backend captures and visual acceptance. The earlier composition fixture remains historical and is not rerun for this lower-cost surface.
