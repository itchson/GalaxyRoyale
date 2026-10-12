# Galaxy Royale development

Follow the user's connected-environment instructions. Root owns integration and shared manifests. Specialists own only their explicitly assigned directories.

This is production source: no fabricated agent activity, reviews, builds, screenshots, or passing test results. Record actual evidence in the local studio; unfinished and physically untested work remains visible. The shipped game must not depend on the studio service.

Rust simulation is authoritative. ECS entities represent objects/subobjects; cells belong to packed PixelBody data. Keep gameplay independent of render quality and both WebGPU/WebGL2 backends. Save collected progression, not arbitrary combat state.

Local Spark tasks use the installed worker wrapper, explicit UTF-8 task files under work/, and the actual sandbox/approval policy. At most two Spark workers; never recurse. Do not expose secrets.

Use a separate `CARGO_TARGET_DIR` for each native evidence scope. Never run a newer art, QA or review scope in a frozen proof target. Before freezing evidence, copy native artifacts with `scripts/qa/freeze-native-proof.mjs --source <artifact> --out work/<scope>/proof-binaries`; record compiler inputs, commands and logs separately. The helper preserves bytes and rejects overwrite/corruption; it does not infer compiler provenance. If an old artifact becomes unavailable, append a correction rather than changing its original manifest or relabelling a replacement.

