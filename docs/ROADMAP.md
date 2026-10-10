# Accepted future scope

Mission Control is the authoritative task/status record. This document preserves the latest direction relayed by the Producer from the user's feedback; it does not mark future features implemented or tested. The current playable build remains a solo local-progression slice.

## Owned docks and arenas

The solo champion campaign is now implemented: eight seeded NPC arenas, distinct pixel hulls and attacks, single-claim patches/blueprints, Haven crafting/switching, and a nine-entrant round robin. The eight player duels run live; the other 28 pairings are explicitly simulated. An actual first-place result grants the Sovereign and credits once; completed losses permit another season. Per-ship XP, level-gated purchases, bounded material cargo, death reports, and 0.6–1.6× world zoom are implemented. Native and component tests cover their rules; a complete eight-champion browser playthrough, balance review and physical-device validation remain pending. See CAMPAIGN.md and NARRATIVE.md. Optional local text/voice providers are injection contracts with safe authored fallback; no neural dialogue or voice service is shipped or claimed as running.

Future player-owned docks will support upgrades to their protection radius and arena encounters. Protection must use the same radius for enemy spawning, combat interaction, visuals and player eligibility. Upgrade costs, radius bounds, arena entry/exit rules and rewards still need design approval and play testing. Persistent ownership and upgrades need migration and single-claim reward checks. Multiplayer dock ownership must be enforced by the authoritative session rather than accepted from a client save.

## Co-op and network authority

Invite-based co-op remains backlog. A separate authority milestone must define shared world ownership, session/invite lifecycle, deterministic seeds, trusted simulation, synchronization of destruction and quest rewards, reconnect, and compatible profile versions. That milestone is also a prerequisite for multiplayer battle royale. No network service, co-op session or matchmaking has been accepted in the current slice.

## Separate battle royale mode

The future multiplayer mode uses a compact seeded circular universe with equally spaced zones, arena siege play and a last-player-standing outcome. Match state and rewards must remain separate from solo exploration progression. The accepted direction does not yet specify player counts, zone contents, siege damage rules, elimination/respawn rules, match length or matchmaking; these remain design and balance work.

Crossing the circular boundary must transfer the ship to the antipodal edge, preserve its heading and speed, and rebase the camera so smooth following does not sweep across the universe. The portable, opt-in `CircularTopology` primitive in `crates/engine_core/src/topology.rs` is implemented. Native tests verify overshoot and repeated/oblique crossings, explicit invalid/tangent handling, and shared rebase of motion, heading, camera and destination. It is unused by the solo game. Integrating this primitive with a seeded zone layout, interpolation, attached effects, nearby projectiles and authoritative multiplayer state remains future work requiring visible play review.

## Acceptance remains explicit

The console's semantic controls and bounded history/output have passed TypeScript and six regression checks. The top black drawer slides over 180 ms, retains pause/input capture until closure and supports reduced motion. The Producer recorded desktop browser acceptance for console/RoyaleMono/material lighting, compact HUD, WebGPU/WebGL2 startup, synthetic initializer fallback and actual device-loss recovery retaining expedition state. The bottom contextual portrait dialogue displays sanctuary descriptions; its accept action and retained quest after offline reload passed in the browser. Quest completion/reward claim play review remains pending despite native single-claim reward tests.

Desktop packaged-game offline reload and launch passed with the local preview server stopped: title, WASM and flight loaded from the service-worker cache. Physical Pixel 6a/iPhone 12 acceptance, Safari compatibility, device audio/PWA installation/update lifecycle, and sustained 30-minute mobile soak remain pending until measured on named hardware. Full boss encounter play review and audio listening are also pending. No future mode inherits acceptance from desktop screenshots, synthetic input or the portable topology primitive.
