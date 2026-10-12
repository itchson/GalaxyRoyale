import type { ComponentHud } from "./loadout";
import { cargoRows, type CargoHud } from "./cargo";
import "./vault-scene.css";

export interface VaultHud extends CargoHud {
  components: ComponentHud[];
}

const componentCategories = ["RENKO", "HUGO", "PAN"];

const views = new WeakMap<HTMLElement, { key: string }>();

function text(tag: "h3" | "p", value: string, className = ""): HTMLElement {
  const node = document.createElement(tag);
  node.textContent = value;
  node.className = className;
  return node;
}

function section(className: string, label: string): HTMLElement {
  const node = document.createElement("section");
  node.className = className;
  node.setAttribute("aria-label", label);
  return node;
}

/**
 * Snapshot-driven: no DOM work unless actual banked inventory, components or
 * courier missions change. View only - actions and routing stay with Rust.
 */
export function paintVault(target: HTMLElement, hud: VaultHud): boolean {
  const rows = cargoRows(hud);
  const deliveries = hud.quest_log.filter(
    (q) => q.event === "courier" && !q.completed,
  );
  const owned = hud.components.filter((c) => c.count > 0);
  const key = JSON.stringify([
    hud.cargo_materials,
    rows,
    deliveries,
    owned.map((c) => [c.id, c.count]),
  ]);
  if (views.get(target)?.key === key) return false;
  target.classList.add("vault-scene");
  const scroll = target.scrollTop;

  // Banked materials: reuse the shared named-subset ledger rows.
  const banked = rows.filter((row) => row.banked > 0);
  const materials = section("vault-materials", "Banked crafting materials");
  materials.append(text("h3", "BANKED AT HAVEN"));
  if (!banked.length) {
    materials.append(text("p", "NO BANKED MATERIALS", "vault-empty"));
  } else {
    const ledger = document.createElement("div");
    ledger.className = "vault-ledger";
    ledger.setAttribute("aria-label", "Material, banked");
    for (const value of ["MATERIAL", "BANKED"])
      ledger.append(text("h3", value));
    for (const row of banked)
      ledger.append(text("p", row.label), text("p", String(row.banked)));
    materials.append(ledger);
  }

  // Compact at-risk summary: only unbanked salvage carried aboard.
  const atRisk = rows.filter((row) => row.carried > 0);
  const risk = section("vault-risk", "Unbanked salvage at risk");
  risk.append(text("h3", "UNBANKED SALVAGE"));
  if (!atRisk.length) {
    risk.append(text("p", "NOTHING CARRIED ABOARD", "vault-empty"));
  } else {
    const total = atRisk.reduce((sum, row) => sum + row.carried, 0);
    risk.append(
      text(
        "p",
        `${total} UNIT${total === 1 ? "" : "S"} ABOARD`,
        "vault-risk-total",
      ),
    );
    const list = document.createElement("div");
    list.className = "vault-risk-rows";
    for (const row of atRisk)
      list.append(
        text("p", row.label),
        text("p", String(row.carried), "vault-risk-count"),
      );
    risk.append(list);
    risk.append(
      text(
        "p",
        "Bank at Haven to keep it. Salvage is lost if the RAICHO is destroyed.",
      ),
    );
  }

  // Crafted components: owned art via the native craft-component index.
  const components = section("vault-components", "Crafted components");
  components.append(text("h3", "CRAFTED COMPONENTS"));
  if (!owned.length) {
    components.append(text("p", "NONE YET", "vault-empty"));
  } else {
    const grid = document.createElement("div");
    grid.className = "vault-component-grid";
    for (const component of owned) {
      const card = document.createElement("div");
      card.className = "vault-component";
      card.setAttribute("aria-label", component.name);
      const icon = document.createElement("div");
      icon.className = "vault-component-icon";
      icon.dataset.pixelKind = "craft-component";
      icon.dataset.pixelValue = String(component.index);
      icon.setAttribute("aria-hidden", "true");
      const meta = document.createElement("div");
      meta.className = "vault-component-meta";
      const category = componentCategories[component.category] ?? "";
      meta.append(
        text("h3", component.name),
        text("p", `${category} / BANKED ${component.count}`),
      );
      card.append(icon, meta);
      grid.append(card);
    }
    components.append(grid);
  }

  // Mission items: courier packets follow Rust retention rules.
  const missions = section("vault-missions", "Mission items");
  missions.append(text("h3", "MISSION ITEMS"));
  if (!deliveries.length) {
    missions.append(text("p", "NO ACTIVE DELIVERIES", "vault-empty"));
  } else {
    for (const quest of deliveries)
      missions.append(
        text(
          "p",
          `COURIER PACKET / ${quest.site_id} / ${quest.progress >= quest.target ? "READY FOR SOURCE" : "RETURN TO HAVEN"}`,
        ),
      );
    missions.append(
      text("p", "Courier packets are retained and use no material slots."),
    );
  }

  target.replaceChildren(materials, risk, components, missions);
  target.scrollTop = scroll;
  views.set(target, { key });
  return true;
}
