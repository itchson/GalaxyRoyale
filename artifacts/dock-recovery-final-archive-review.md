# Dock / console release: committed archive review

Independent read-only check passed for runtime commit 991153c187b0eb3920829dc3ff94ea04f99ee864 and normal package cache galaxy-ac19daf73e83b006.

The named archive work/galaxy-dock-console-release-20261011.tar.gz is 2,160,676 bytes; SHA-256 499f0979bdf351a32365786ca883a5e2b64b76c8a888d5d57f27a619071ed065. It contains exactly the 19 pinned game files plus dist/.openai/hosting.json. Every archived game byte matches the final dist/package pins; hosting metadata matches the local static-dist configuration. No links, duplicate paths, traversal paths, Studio, work files or separate API payload were admitted.

All 94 pinned runtime/build/static inputs match the committed source: 77 exact byte representations and 17 text-only CRLF/LF representations. Both actual WASM binaries remain pinned (51 WebGPU / 47 WebGL2 exports), with generated public copies equal to dist. The normal exported facility/recovery APIs are present; the inspected test checkpoint and obsolete recovery exports are absent.

Executed: python work/dock-recovery-final-package-review/check-archive.py --source 991153c187b0eb3920829dc3ff94ea04f99ee864 --archive C:/Users/mitch/Documents/ChatGPT/galaxy-royale/work/galaxy-dock-console-release-20261011.tar.gz (exit 0). Machine evidence: work/dock-recovery-final-package-review/archive-evidence.json, SHA-256 75a2fc99332bb3dbc6fa4f623eeb69758515ca0b8d28d05e99f6b817d0fa012. The earlier final-package report and its freeze manifest remain unchanged.

This establishes committed-source / local-package / archive parity. It does not assert private deployment success, remote-response identity, new browser interaction, physical-device behavior or soak acceptance. No source or dist was rebuilt or changed.
