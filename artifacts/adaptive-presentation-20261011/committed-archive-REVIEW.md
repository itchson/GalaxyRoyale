# Exact committed source and official archive audit

No blocker was found. The official archive matches the frozen normal cc8d package and committed source; this is a source/package acceptance gate, not deployment or device acceptance.

- Source and observed HEAD: `3bf0c02b5e9fc0011c0ce6921bfaf0fbe664c65d`.
- Archive: `work/galaxy-royale-adaptive-cc8-private.tar.gz`; **2,357,438 bytes**.
- Archive SHA-256: `baed5574e8bf4151b2d3f9b9dfea724fd92827e5b7e6a46d06ad293df4bb9cd9`.
- Recomputed cache: `galaxy-cc8da99d023ae67a`.
- 142 reviewed Git inputs match: 127 byte-exact and 15 text-only CRLF/LF representation differences. All nine generated public WASM/build-map inputs remain uncommitted and match the reviewed generated package.
- Exactly 21 regular tar files: all 20 pinned game dist files plus `dist/.openai/hosting.json`. No outside-root, traversal, duplicate, link/special or unexpected payload was accepted. Nothing was extracted to disk. Every game file matches the frozen normal package and current dist byte-for-byte.
- Hosting metadata matches the current project binding, with static root `dist`; project `appgprj_6aca50f5b1bc81918526a6ee969aac93`. Studio and API payload are absent.
- Both actual archived WASM modules parse with Node `WebAssembly.Module`: WebGPU 58 exports; WebGL2 54 exports. Current status-bound, map, equipment and recovery methods are present; `checkpoint_contract` is absent from binaries, binding glue and declarations.

The independent source/package baseline remains unchanged. This addendum records the exact official archive only after Root reported packaging success; no partial archive was opened. No source, dist, archive, Git state, build or publication was mutated by this audit.

Read-only repeat verification:

```powershell
python work/adaptive-label-final-release-review/archive-stage/verify.py --live
```

`archive-evidence.json` retains each accepted source/blob representation and each actual archive member hash. `frozen-hashes.json` pins this owned addendum. Runtime browser captures, real audio listening, physical devices, natural-play balance and extended soak retain their separate scopes. Root owns private publication and final browser evidence.
