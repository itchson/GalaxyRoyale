# Actual gallery CI regression and repair

PR7 first candidate7354c5b, run38110173136: clean Ubuntu native simulation/Clippy passed, but the shell job failed in the existing gallery caption test. The test selected the final media entry with `.at(-1)` and therefore mistook the newly appended actual desktop Settings screenshot for the historical flat software contact sheet. The assertion correctly required software-raster scope, but applied it to the wrong image. This failed run is retained as failure evidence, not represented as passing integration CI.

Commita8d8b315df441b84b7e9714b4b30e59d62ff43be selects the historical and current artifacts by their exact catalog paths. All previous scope assertions remain. The current screenshot additionally requires its runtime-screenshot classification, desktop WebGPU caption, physical-phone limitation, exact JPEG bytes and readable acceptance provenance. No assertion is removed to bypass an unknown failure.

Root reran the complete293-test shell suite after the catalog and test changes:292 passed, one Windows file-symlink capability skip, zero failed. `current-ui-final-shell-tests.log` retains the actual output. Clean final integration CI run38110408367 is a separate exact-head gate; its final outcomes are recorded in the final integration JSON/build record.
