# Haven facilities and full-screen scenes

Latest requested layout: Forge, Hangar, MISSIONS, MULTIPLAYER and VAULT are five separate floating, top-down spacehub objects around the main Haven. MISSIONS combines expedition lore/training/missions and the arena league. The central dock no longer opens multiplayer. Labels appear only on hover or keyboard focus; hit tests remain native world-space tests, independent of zoom or shake.

Facility centers are (-120,-80), (120,-80), (-120,95), (120,95), (0,170), with a 24-unit native hit radius. Higher-detail cached packed cells and object-local signal animation should match the approved central dock's alloy, recesses and instrumentation. No baked menu labels, frame-time asset clones or duplicated full-size central packs.

Menus use full-screen native pixel scenes. Their measured semantic controls supply input/accessibility and clipping; Rust paints the visible background, chrome, text, icons and art. A stable top-right close returns to flight. Scene navigation remains local to its purpose: MISSIONS has Expeditions and League; multiplayer has Join and Host. MEISHI remains a collectible card inside its scene, without collection tabs on the card. The console remains its top drawer and character dialogue remains its bottom panel.

The world continues during inspection. Opening a scene resets interrupted flight controls, rather than changing simulation authority or granting safety. Co-op restrictions continue to prevent editing or saving the boxed local world while visiting another world.

Hangar showcases the current RAICHO in an overhead native garage. A horizontal index contains owned, discovered hulls; switching uses native availability and per-hull XP. Clicking the showcase opens a separate statistics/three-ability-slot scene. Forge opens on the discovered ability index, with local component/material-recipe, hull-blueprint and hardware sections. Recipes and costs remain authoritative Rust data; crafting never automatically selects a hull.

Vault shows banked crafting materials, owned components and retained mission items. Ship scenes have no material inventory-slot presentation. The user explicitly confirmed that collected unbanked salvage still deposits on return to Haven and is lost on death; this UI change does not silently alter those rules. Upgrade currency remains on MEISHI.

Relative object height, contact shadows and hub-surface lighting are a separate active engine prototype. Native receiver occupancy, transformed source geometry, zoom, normal/emission shading and both renderer backends need actual review; no GPU visual acceptance is asserted here.

Acceptance: five distinct high-detail silhouettes at native footprints; hover-only labels; correct zoomed pointer hits and centered keyboard activation; Hangar showcase/index/details, Vault accounting and discovered-only Forge recipes; MISSIONS training and arena actions accessible; multiplayer only at its dedicated hub/HUD; full-screen opaque native backing with feedback above it; readable layouts, content scrolling and top-right close at wide, narrow and short aspect ratios; MEISHI card framing preserved; both WebGPU/WebGL2. Physical-device and performance acceptance remains separate.
