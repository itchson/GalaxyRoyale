# Boss browser art review

Read-only visual inspection of actual browser captures supplied by root. These captures demonstrate their recorded desktop browser states; they do not establish phone performance or acceptance.

## boss-coil-g5-webgpu.jpg

1280×720 initial flight/diagnostic view. The authored Worldeater head is outside the top extent, while the initial module train runs vertically across the camera center. This capture therefore cannot validate head silhouette or exposed-core presentation. The visible modules retain the authored metal seams, bright rims and jade center cells without a texture/sprite substitute.

The boss title, generation, warning/phase text and health bar fit the dark panel and have strong contrast. The currency and biome blocks also remain readable, with adequate screen margins. The subdued starfield and ringed background object leave combat foreground legible.

Two concrete presentation observations were sent to root: the player hull is absent in this frozen initial frame (consistent with the initial invulnerability blink, but a subsequent visible-player capture is required), and the unbacked LV/CARGO caption lies directly over a similarly colored boss module and loses contrast. The diagnostic framing should show the full head and a visible player at zoom 0.6 before it is used as a boss-art acceptance image.

Further Dart/Bastion, zoom-limit and 720-wide captures are pending at this review point.

## boss-settings-720-zoom-default.jpg

Actual 720×360 settings capture. The single centered panel, nonrepeating rail ornaments, tabs, voice/zoom sliders and footer actions remain within screen margins. The Flight Zoom thumb visibly sits around 40% along its track, matching a default zoom of 1.0 within the 0.6–1.6 range. Lower-row labels and action captions are readable against the dark panel.

Concrete presentation defect sent to root: the first row's CAMERA SHAKE, REDUCED MOTION and cosmetic-detail labels lose their upper glyph rows at the scroll-content top, while checkbox squares are still visible. Lower DIALOGUE VOICE and slider labels retain their complete glyphs. This points to first-row vertical placement/clip padding rather than a global font or GPU buffer failure. It needs a corrected capture before the narrow settings layout is accepted.

Root traced the state to native slider focus scrolling a 67-pixel content viewport containing 120 pixels of settings. The clipping follows the actual scroll state correctly, but the default short-landscape layout is too cramped. Root revised the short-landscape footer to one row with five actions and 44-pixel minimum targets, and reduced the settings gap to eight pixels. A freshly navigated replacement capture is pending; this note records the cause and proposed correction, not visual acceptance of an unseen result.

The observed pre-fix settings image is retained as `artifacts/boss-settings-720-before-layout-fix.jpg` so replacing the default snapshot cannot erase the evidence cited above.

## Replaced boss-settings-720-zoom-default.jpg

Inspected the actual replacement saved at 04:05:51 local time. All nine control labels now have complete glyph rows: master/music/effects volume, camera shake, reduced motion, cosmetic detail, dialogue voice, voice volume and flight zoom. Five footer actions fit in one row with intact captions and rails. The prior clipping/cramped-content finding is resolved in this captured default state. The panel and all actions retain screen margins; this is a narrow desktop viewport capture, not a physical-phone test.

## boss-coil-g5-zoom-min.jpg

Actual 720×360 flight capture at the minimum zoom. A visible player remains at full hull, and the corrected initial Coil train extends away from the player instead of through the ship. The cargo caption now sits within the expanded dark meter backplate and remains readable. The boss generation/title, Vortex Lunge phase, health bar, biome and currencies are also legible.

The upper head/middle reactor overlaps the fixed boss HUD panel. As root also observed, this framing cannot validate the complete Worldeater silhouette or exposed-core presentation. Visible outside-panel portions retain the authored metal contour and cells. This is a framing/occlusion limitation of this particular capture, with no new broken pixel extraction, text clipping or contrast defect found. Additional family/zoom/backend captures are still pending.

## boss-dart-g5-zoom-min.jpg

Actual 720×360 minimum-zoom flight capture. The visible lower Imperatrix forks, pale metal plates and pink firing pixels are clearly different from the Coil head/module geometry. G5/Dart Imperatrix, Royal Cross and the current 31% player hull meter remain legible; the screenshot shows live combat damage rather than a placeholder health bar.

The same fixed boss panel obscures most upper hull/core detail, so this image supports combat/HUD integration and family distinction, not complete silhouette or reactor acceptance. Root received the concrete capture recommendation to use a later frame with the head farther down in the viewport. No new clipped labels, restored pixel holes or obvious renderer defect is visible in the portions shown.

## boss-settings-720-zoom-max.jpg

Actual 720×360 settings capture saved at 04:11:11 local time. All nine labels and five footer actions retain complete glyph rows and screen margins. The enlarged world scenery behind the panel does not move or resize the menu controls; the Flight Zoom thumb is at the right endpoint. No new clipped glyph, repeated rail motif or contrast defect appears in the menu.

The initial `boss-browser-zoom-max.txt` contained only a zoom scalar of 0.76 from an earlier console state. Root subsequently replaced it with a fresh console snapshot recording actual zoom 1.60, world viewport 450×225 and UI viewport 720×360 after setting the real Settings range to its End position. The endpoint and numeric maximum are therefore corroborated for this browser session. Earlier 0.76 entries remain in the console history and are not maximum-zoom results.

## boss-bastion-g5-zoom-max.jpg

Actual 720×360 WebGPU flight capture saved at 04:14:41 local time. Fresh console data records settled zoom 1.60, world viewport 450×225 and UI viewport 720×360. The enlarged G5 Bastion Oblivion is naturally cropped at the top of this frame. Its visible lower armor retains distinct slate/pale-metal plate contours, small ventilation seams, gold edge pixels and open bays with background visible through them. These are occupied-cell silhouette details, not a single scaled solid rectangle.

The player ship remains visible below the boss, with cyan engine/exhaust cells and pale-gold hull separated from the dark background. Boss title, Rotating Rails label, health bar, currency counts, biome/threat block, Home compass and backed LV/CARGO/hull meter all fit their panels. The 58% hull value is readable. The transient incoming signature caption is visible over the encounter region, while the fixed boss HUD obscures some armor. No clipped HUD glyph, missing quad layer or broken bay geometry is apparent.

This frame establishes representative metal detail, HUD contrast and maximum-scale behavior. It cannot establish a full Oblivion silhouette or exposed-core recovery presentation because the upper head is outside the viewport and another portion is behind the boss panel.

## boss-bastion-g5-webgl2.jpg

Actual visible-tab WebGL2 flight capture saved at 04:14:56 local time. The backing image is 1280×720, while the recorded logical UI viewport is 533×300 and the world viewport is 333×188 at zoom 1.60. This differs from the 720×360 logical WebGPU capture; the images are representative backend checks, not a controlled pixel-identical comparison.

The visible Oblivion armor/bays preserve the same recognizable plated geometry and golden rims. The player hull, two exhaust trails, magenta combat pixels and subdued layered scenery remain visible together. The boss title, phase label, health bar and bottom LV/CARGO/36% hull meter have complete glyphs and clear contrast. Biome/threat, currency and Home blocks retain screen margins. No observed frame loses the boss, player, text or HUD batch.

At this maximum zoom, the upper head again extends beyond the top edge and the boss HUD occludes middle armor. The visible golden reactor detail does not independently prove open-core phase timing. No new concrete presentation defect was found in the captured portions. These desktop captures do not establish mobile hardware performance, touch ergonomics during a fight, all fifteen full silhouettes, or every attack/recovery phase.
