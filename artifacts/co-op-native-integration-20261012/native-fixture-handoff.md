# Native fixture replay amendment

Frozen commit `61c45d210e99a0e14f08a42d690951987a979911` on `codex/co-op-wasm-controller`. This amendment changes only `scripts/test/coop-wasm-ports.test.mjs` and two finite native fixture/provenance JSON files. Controller, ports, wire and transport production bytes remain identical to the earlier `cae3127` freeze. The earlier HANDOFF/CONTRACT/manifest and their 36-test evidence remain historical and unchanged.

Executed here: **37/37 focused Node tests** (28 controller + 9 ports), TypeScript noEmit, scoped Prettier check and git diff --check; all exit 0. Logs in this folder are the actual runs. No native executable, browser, server or combined game build was run for this amendment.

The added recording is the exact Engine `ef5b55bd303b422aea6a2a93748370128839277c` export `game_core co_op_replica::tests::export_native_replica_operations`: 154,158 canonical UTF-8 LF bytes, SHA256 `aed4db590288faaaba68183fb2eee2280245449e287f09cc26c61328e6bd56fc`. It contains nine operations: begin, stage, one entity page, four body pages, commit and end. The native fixture contains two admitted Game ECS pilots and one explicitly arranged three-cell asteroid; three entities, 1,974 occupied cells and 4,614 storage slots. Its actual native Results were mapped to the agreed unit response `{v:1,ok:true,value:null}` by the exporter. It is not a WASM wrapper execution.

The new test sends those recorded operations through the real TypeScript ports and compares delegated requests and replies to the retained recording, including a successful complete transaction and cleanup. It does not manufacture success responses. Node replay still does not instantiate Rust Game, Replica, WASM or RTC. Native source/runtime evidence belongs to Engine; the integrated Root native source is `31261d33246d9b05bb8b3529ca8d5ddf14fe9586`.

All existing five publication exports from exact `91a18ab` remain pinned. Canonical native/Git LF byte pins are checked after the explicit Windows checkout CRLF normalization introduced in `cae3127`; JSON values are unchanged. Real combined WASM rendering, transport cadence, host/guest movement and destruction, body cache updates and independent durable save preservation remain browser acceptance work.

`manifest.json` pins all fifteen committed runtime/test/fixture inputs and the four executed check logs. It also verifies production bytes did not change in this test-only amendment. Root owns integration, and no push or production checkout mutation was performed here.
