# Retained-origin upgrade and map inspection (interim)

Observed on the existing Chrome profile at `http://127.0.0.1:4212/?renderer=webgl2`. The former ephemeral tab closed; its replacement was tab `1254884539`. No origin storage, service-worker cache, profile, extension permission or extension setting was reset or changed.

The inspected intermediate package was `galaxy-965c10c6fd0039b0`, with `index-CzK_qQke.js`, `index-BapSg79s.css`, and resource-observed WebGL2 glue/WASM under `webgl2-e93f4425ad151df865204b4e003a3fde9175d83144d132c847e6c5ecaa504532`. The pageAssets inventory is preserved in `pre-apply-assets.json`.

The existing worker displayed the update notice. Before applying it, Launch and Save opened the actual backup page successfully; the previously observed `cancel_course is not a function` failure did not recur. Apply update at Haven reloaded the title. Launch then opened working live status with the retained Scout level 1, XP 4, banked T1 1, points 0/1/1 and empty ability slots. This is retained-origin upgrade evidence, not a fresh-profile replacement test.

Actual 1280×720 status was readable with native icon actions and a scrollable lower inventory section. The map showed authoritative Haven radius 240, belt 700–1400 and zero charted discoveries. Zoom in changed the visible span from 14583 to 9722. No unvisited beacons were displayed. This does not test populated discoveries or natural exploration.

Two concrete presentation findings were sent for correction:

- `map-button` and `flight-settings` had computed opacity 1 on a pixel-ready page, while pause/save were 0. Their DOM MAP/SET labels covered native toolbar art. Root subsequently built the selector correction; that later package has not yet been accepted here.
- `map-desktop.jpg` shows selected-location text crossing into the saved-status footer. The bridge's scroll ancestor selector omitted `#map-content`; map art also needs the content ancestor intersected with the closer `#map-chart` clip when the page scrolls. Engine was assigned the correction and regression.

The final 844×390 and 320×180 matrix is pending the bridge correction and rebuilt package. The temporary viewport override was reset successfully. A later attempted close found the menu already hidden; the current page was ordinary flight at 219 m and full hull. No cause or user intent is inferred from that state. The snapshot and logs are retained. Logged warnings in that snapshot were from a Chrome extension; no new game exception appeared in the captured warning/error list.

This inspection does not establish phone touch, performance, populated-map navigation, forced GPU loss, owner-authenticated private execution, human pacing, listening or full natural campaign acceptance.

## Second-update lifecycle finding

After Root's final map clipping rebuild (`galaxy-3aad1a2d2d75f0ab`), ordinary reload still loaded the internally coherent cached 965c shell and its bound e93f runtime. HTTP inspection separately confirmed that the owned preview served current `/sw.js` with `Cache-Control: no-cache`. Opening, closing and reopening the actual Save page still exposed no Apply update action. No Check update action existed in that old shell. `waiting-update-absent.txt`, `waiting-update-absent-assets.json` and `waiting-update-absent.jpg` preserve the observed page. No hidden registration/cache APIs were used, so no waiting/installing worker state is asserted.

At Root's request, the selected Chrome agent tab inventory was verified to contain only the owned `1254884539` game tab. That tab was closed through the documented browser API to permit ordinary worker activation when clients close. Reopening on the same origin/profile awaits the forthcoming lifecycle repair build. This is manual close/reopen recovery; it must not be reported as a successful second Apply update action. The profile/cache were not reset, and no user tab was closed.
