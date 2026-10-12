# Detailed flight-icon asset handoff

Frozen asset commit: `749b85a6a08d42330d3aa0b115ebe7d7bd8e88ea`.
Portability parent: `28e350e6977bf4d7cbf8acbc6ce3ed7c3748ca89`; parent may cherry-pick only the icon commit when the repair is already integrated.

The atlas is version1/palette/sprites, with 21 shared colors, 64 by64 source dimensions, per-sprite row-major indices plus explicit accent_cells/emitter_cells linear masks. Atlas SHA256: `e86980bbb99f3c1423900283f78e7bba2aac4b90d02b4a0855291c2e5ff8ee73`. All ten objects cost14,709 occupied cells; status/map/settings/save cost6,632. Immutable metal and secondary colors must not flash or tint; Root's adapter owns accent tinting and emitter-only animation. Geometry never changes for control state.

Actual11/11 Windows and11/11 Ubuntu24.04 WSL tests passed. The same regeneration matches the entire committed atlas, source PNGs, provenance, manifest and both review PNGs as rawbytes. PNG encoding uses an explicit stored-DEFLATE stream, avoiding linked compressor differences. No native binary or browser was executed for these specialist checks. Review PNGs are declared authoring/software samples; compact rows use14/28/42 physical sample pixels at14logical units for1x/2x/3x, then enlarge for inspection. Fine detail is not promised to remain visible at1x.

Reproduce with pinned scripts/asset-requirements.txt and `python scripts/asset-flight-icons-tests.py`. Exact native landmarks (source x/y), intentional empty apertures and mask probes are in that test's LANDMARKS constant, independently written against actual designed parts. `docs/FLIGHT_ICON_ART.md` describes full geometry/budget/art/animation contracts. Source tool authors cells directly; no image generation or historical32-cell raster upsample was used.

Gameplay's independent read-only source/software review found no art/schema blocker. Actual native adapter, normal GPU presentation, state animation and human/phone input/readability remain separate Root/Engine acceptance. Existing32-cell assets remain historical API/source; this asset commit does not claim all game artwork is reauthored.

Recorded inputs/results are pinned in evidence.json. Work-only evidence is not a shipped game dependency. No push, preview server, production ledger/API, browser or native source mutation was performed by this task.
