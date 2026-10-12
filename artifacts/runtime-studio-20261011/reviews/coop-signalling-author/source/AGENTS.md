# Galaxy Royale development

Follow the user's connected-environment instructions. Root owns integration and shared manifests. Specialists own only their explicitly assigned directories.

This is production source: no fabricated agent activity, reviews, builds, screenshots, or passing test results. Record actual evidence in the local studio; unfinished and physically untested work remains visible. The shipped game must not depend on the studio service.

Rust simulation is authoritative. ECS entities represent objects/subobjects; cells belong to packed PixelBody data. Keep gameplay independent of render quality and both WebGPU/WebGL2 backends. Save collected progression, not arbitrary combat state.

Local Spark tasks use the installed worker wrapper, explicit UTF-8 task files under work/, and the actual sandbox/approval policy. At most two Spark workers; never recurse. Do not expose secrets.

