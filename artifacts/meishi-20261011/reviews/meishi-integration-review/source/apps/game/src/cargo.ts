import type { FlightMenuHud } from "./flight-menu";
import type { Quest } from "./storage";
import "./cargo.css";

export interface CargoHud extends Pick<
  FlightMenuHud,
  | "material_cargo"
  | "material_stash"
  | "material_training_cargo"
  | "material_training_stash"
  | "cargo_materials"
  | "cargo_capacity"
> {
  quest_log: Quest[];
}
type Row = { label: string; carried: number; banked: number };
const namedMaterials = ["PHASE SILICATE", "IRON FILAMENT", "COOLANT KESSHO"];

/** Named training parts are subsets of T1, never additional inventory. */
export function cargoRows(hud: CargoHud): Row[] {
  const carried = hud.material_training_cargo ?? [0, 0, 0];
  const banked = hud.material_training_stash ?? [0, 0, 0];
  return [
    ...namedMaterials.map((label, i) => ({
      label,
      carried: carried[i] ?? 0,
      banked: banked[i] ?? 0,
    })),
    ...Array.from({ length: 5 }, (_, i) => ({
      label: i === 0 ? "OTHER T1 MATERIALS" : `T${i + 1} MATERIALS`,
      carried: Math.max(
        0,
        (hud.material_cargo[i] ?? 0) -
          (i === 0 ? carried.reduce((a, b) => a + b, 0) : 0),
      ),
      banked: Math.max(
        0,
        (hud.material_stash[i] ?? 0) -
          (i === 0 ? banked.reduce((a, b) => a + b, 0) : 0),
      ),
    })),
  ].filter((row) => row.carried > 0 || row.banked > 0);
}

const views = new WeakMap<HTMLElement, { key: string }>();
function text(tag: string, value: string): HTMLElement {
  const node = document.createElement(tag);
  node.textContent = value;
  return node;
}

/** Snapshot-driven: no DOM work unless the actual inventory or missions change. */
export function paintCargo(target: HTMLElement, hud: CargoHud): boolean {
  const rows = cargoRows(hud);
  const deliveries = hud.quest_log.filter(
    (q) => q.event === "courier" && !q.completed,
  );
  const key = JSON.stringify([
    hud.cargo_materials,
    hud.cargo_capacity,
    rows,
    deliveries,
  ]);
  if (views.get(target)?.key === key) return false;
  const scroll = target.scrollTop;
  const summary = text(
    "p",
    `CARRIED ${hud.cargo_materials}/${hud.cargo_capacity}`,
  );
  summary.className = hud.cargo_materials > 0 ? "cargo-risk" : "";
  const note = text(
    "p",
    "Materials bank at Haven. Carried materials are lost on RAICHO loss.",
  );
  const list = document.createElement("section");
  list.className = "cargo-ledger";
  list.setAttribute("aria-label", "Crafting materials, carried and banked");
  for (const value of ["MATERIAL", "CARRIED", "BANKED"])
    list.append(text("h3", value));
  for (const row of rows) {
    list.append(
      text("p", row.label),
      text("p", String(row.carried)),
      text("p", String(row.banked)),
    );
  }
  if (!rows.length) list.append(text("p", "NO MATERIALS YET"));
  const missions = document.createElement("section");
  missions.className = "cargo-missions";
  missions.append(text("h3", "MISSION ITEMS"));
  if (!deliveries.length) missions.append(text("p", "NO ACTIVE DELIVERIES"));
  for (const quest of deliveries) {
    missions.append(
      text(
        "p",
        `COURIER PACKET / ${quest.site_id} / ${quest.progress >= quest.target ? "DELIVERED TO HAVEN - REPORT TO SOURCE" : "RETURN TO HAVEN"}`,
      ),
    );
  }
  if (deliveries.length)
    missions.append(
      text(
        "p",
        "Mission packets are retained and use no material cargo slots.",
      ),
    );
  target.replaceChildren(summary, note, list, missions);
  target.scrollTop = scroll;
  views.set(target, { key });
  return true;
}
