# Native lost-context preparation guard

Authorized production change is limited to `crates/game_web/src/browser.rs`. Before independent WebGL renderer preparation, the existing canvas's WebGL2 context is queried and its native `isContextLost()` flag checked. A lost context returns a controlled `JsValue` error before wgpu/glow initialization. The live Galaxy is neither borrowed nor modified. WebGPU preparation is unchanged.

This closes the source-confirmed pre-event boundary: native loss can precede the queued DOM lost event, while glow 0.17.0 unwraps VERSION during WebGL initialization. It does not promise protection against a subsequent hardware loss during construction. The shell's actual restoration-event and fresh-preparation sequence remains necessary.

Locked strict Clippy passed for both WASM feature paths:

```
cargo clippy -p game_web --target wasm32-unknown-unknown --locked --no-default-features --features webgl -- -D warnings
cargo clippy -p game_web --target wasm32-unknown-unknown --locked -- -D warnings
```

Frozen browser SHA256: `b534932734d538c808b13acb76d178c2835fc5f262ae3666f6016327fa489af0`. No renderer, lifecycle, shell, dependencies or distribution were edited. Actual extension loss immediately followed by preparation remains a browser acceptance gate; this handoff records source and compile evidence only. The earlier reconstruction review remains immutable.
