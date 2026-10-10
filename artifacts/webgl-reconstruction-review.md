# WebGL renderer reconstruction source review

Read-only review of the installed primary sources for wgpu/wgpu-core/wgpu-hal **30.0.1** and glow **0.17.0**, plus the current two-phase browser installation and shell lifecycle wiring. No production, dependency, distribution or published evidence was changed by this review. The separately authorized dock/control work has its own frozen handoff.

The reviewed cleanup does **not** show the old renderer deleting the entire context or deleting the newly prepared renderer's objects. The current single-candidate install sequence is coherent at source level. Actual native context restoration and visible first-frame acceptance remain required.

## Exact resource paths

| Installed primary source | Relevant behavior |
| --- | --- |
| `wgpu-hal/src/gles/web.rs:36–100,128` | Same canvas supplies its existing WebGL2 context; each newly created surface has its own swapchain/texture state. An adapter constructs a fresh glow wrapper around that context. |
| `glow/src/web_sys.rs:61–86,284–315` | Each wrapper has separate SlotMaps of actual WebGL DOM objects. A numeric slot key from an old wrapper is not resolved through the replacement wrapper. |
| `glow/src/web_sys.rs:1898,2592,3023,3033,3073` | Deletion removes the old wrapper's stored WebGL object and calls the corresponding native delete operation with that object. No `loseContext()` call is made. |
| `wgpu-hal/src/gles/mod.rs:349,379` | Old Device/Queue destruction removes their own VAO, FBOs and zero buffer. |
| `wgpu-core/src/instance.rs:721` and `wgpu-hal/src/gles/web.rs:447` | Old surface teardown unconfigures its own stored framebuffer and texture. The new surface's independent state is not looked up here. |
| `wgpu-core/src/device/global.rs:1945` | `Device.destroy()` marks the device invalid; it does not use WEBGL_lose_context. The ordinary diagnostic loss flag is a separate path from native context loss. |
| `wgpu-core/src/device/queue.rs:275` | Old queue teardown waits idle and retires submissions using its last successful submission index. GLES wait-idle calls `finish()`. No whole-context destruction appears in these paths. |

After a real restoration, previous WebGL objects are invalid and resources/state must be rebuilt. That is why constructing a fresh Renderer is appropriate rather than reusing its previous buffers or shaders. See the [Khronos context restoration specification](https://registry.khronos.org/webgl/specs/1.0.3/#5.15.3). Separate DOM object identity and per-wrapper tables support the cleanup inference; they are not a browser execution proof.

## Installation and the VAO constraint

`Galaxy.install_recovery` validates canvas identity, replaces the old Renderer, resizes/reconfigures the replacement and updates world extent. Old resources are dropped before the replacement's resize and first draw. The constructor creates its own buffers, transform tables, bind groups, target and pipelines; ordinary draw can grow its initial 131,072-instance/32-table capacities and rebuild world and UI bindings. No old GPU object is carried into the replacement. The live Game is retained.

One important constraint is real: GLES `Adapter::open` creates and **binds** the device's main VAO (`adapter.rs:1088`); `Queue::reset_state` resets several GL states but does not rebind that VAO (`queue.rs:88`). Deleting a currently bound stale prepared candidate can therefore disturb shared context binding state. Keep exactly one preparation at a time, and when discarding a stale prepared renderer, create a **fresh** renderer after that disposal before drawing again. Do not dispose a more recently created candidate and then resume an earlier renderer on the same context. The reviewed shell follows free-stale → fresh-prepare through its deferred restoration retry. No additional resource registry, dependency patch or context replacement is justified by this source review.

## Initialization while genuinely lost

glow's WebGL2 constructor queries VERSION and unwraps a string (`web_sys.rs:284–299`). Calling it with a genuinely lost native context can therefore panic instead of returning the normal Renderer error. The current shell waits on its observed loss/restoration events, which covers the normal flow. However, the real extension sets the native lost flag before its queued lost event is delivered; the helper correctly does not fabricate the lifecycle flag. An immediate extension loss → Recover within the same task is a useful actual-browser boundary test.

If that boundary is reachable, the smallest remedy is a WebGL-only native `isContextLost()` precheck in independent preparation, returning a controlled wait error before adapter construction. It must query the already selected WebGL context, without attempting another backend or making the Game unavailable. This remedy was proposed to the parent and is not implemented by this read-only review. A precheck cannot guarantee protection against every out-of-process hardware loss during construction, so do not promise that stronger property.

## Current normal-build browser gates

Use a disposable profile and a foreground fresh normal-build tab with `?qa=1&renderer=webgl2&toolbar=1`. Pin the actual cache/module and use RENDERER to verify the selected backend.

1. Pause a live diagnostic encounter and capture HUD/profile/world before loss. Use the actual **Lose WebGL context** button. Verify native event cancellation, stopped simulation/presentation, readable fallback, usable profile/save/modal access, and absence of unsafe-borrow errors. The extension request status alone is not the loss-event evidence.
2. Use **Restore WebGL context** in a later task. Wait for the native restored event and completed resource reconstruction. Verify actual world/body holes, UI, material/lighting and input rendering. Profile and paused encounter state should remain intact; a live unpaused loss may occur after a final valid simulation step.
3. Repeat native loss/restoration three times, then use explicit **Recover GPU** once. This exercises old-context cleanup, newly bound VAO, fresh first submission and restoration after later cleanup. Capture actual GL errors/console, not only the success toast.
4. Cover resize and one modal change while lost, and min/max zoom with a large scene whose first restored frame grows GPU buffers. Latest viewport and current intent must prevail; no held-input replay or unintended resume.
5. Separately test loss/restoration while a real preparation is pending if an isolated controlled gate becomes available. Observe stale candidate disposal **before** fresh preparation. AST/binding proofs establish ordering contracts but do not count as this GPU case.
6. Exercise the immediate request-loss → Recover event-delivery boundary. If restoration is unavailable or refused, retain semantic UI/checkpoint and truthfully offer retry/reload; do not label an extension request as successful restoration.

No browser, GPU or physical-device experiment was executed by this review. Manual RECOVER, native extension restoration and callback-only WebGPU loss remain distinct evidence categories. The source hashes and primary dependency files are pinned in `frozen-hashes.json`; earlier frozen gap/lifecycle reports are unchanged.
