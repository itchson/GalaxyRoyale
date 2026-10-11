# Map scroll clipping repair

Frozen bridge adds `#map-content` and intersects every enclosing semantic scroll region, including both `#map-chart` and its parent map content, with the root panel on both axes. Partly visible nodes retain exact common clips; wholly clipped artwork, text and controls are omitted. This repairs the observed footer/header overpaint without changing font, geometry, gameplay or source assets.

The actual TypeScript bridge is transpiled and executed in the existing VM harness. Two added regressions cover details/actions crossing the saved-status footer and a scrolled chart crossing its parent content/header at fractional UI scales. All 15 focused map/retained UI checks passed, and TypeScript passed. Commands/results are in checks.json, raw source identities in source-pins.json. No normal build or browser validation performed by this specialist. Earlier freezes remain unchanged.
