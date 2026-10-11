import "./flight-menu.css";
import type { CampaignHud } from "./campaign";
import type { EquippedAbilityHud } from "./loadout";

export interface FlightMenuHud {
  campaign: Pick<CampaignHud, "selected_ship" | "ships">;
  ship_level: number;
  ship_xp: number;
  ship_next_xp: number;
  hull: number;
  max_hull: number;
  shield: number;
  max_shield: number;
  ship_stats: {
    speed: number;
    top_speed: number;
    acceleration: number;
    primary_damage: number;
    fire_rate: number;
    hull_regen: number;
    shield_regen: number;
  };
  loadout: (EquippedAbilityHud | null)[];
  material_cargo: number[];
  material_stash: number[];
  cargo_materials: number;
  cargo_capacity: number;
  currency: number[];
}

interface FlightView {
  icon: HTMLElement;
  title: HTMLElement;
  progress: HTMLElement;
  stats: HTMLElement[];
  slots: HTMLElement[];
  cargo: HTMLElement;
  tiers: HTMLElement;
  stash: HTMLElement;
  points: HTMLElement;
}
const views = new WeakMap<HTMLElement, FlightView>();
const categories = ["OFFENSE", "DEFENSE", "PASSIVE"];
const colors = ["offense", "defense", "passive"];

function element(tag: string, className = "", value = ""): HTMLElement {
  const node = document.createElement(tag);
  node.className = className;
  if (value) node.textContent = value;
  return node;
}
function createView(target: HTMLElement): FlightView {
  const identity = element("header", "flight-status-identity");
  const icon = element("section", "campaign-icon campaign-ship");
  icon.dataset.pixelKind = "campaign-ship";
  icon.dataset.active = "true";
  icon.setAttribute("aria-hidden", "true");
  const heading = element("div");
  const title = element("h3");
  const progress = element("p");
  heading.append(title, progress);
  identity.append(icon, heading);
  const vitals = element("section", "flight-status-stats");
  vitals.setAttribute("aria-label", "Current effective ship statistics");
  const stats = Array.from({ length: 7 }, () => element("p"));
  vitals.append(...stats);
  const abilities = element("section", "flight-status-abilities");
  abilities.setAttribute("aria-label", "Automatic ability slots");
  const slots = categories.map((_, index) => {
    const row = element("div", `upgrade-category ${colors[index]}`);
    const label = element("p");
    row.append(label);
    abilities.append(row);
    return label;
  });
  const inventory = element("section", "flight-status-inventory");
  inventory.setAttribute(
    "aria-label",
    "Carried materials and retained inventory",
  );
  const cargo = element("p", "flight-cargo-risk");
  const tiers = element("p");
  const stash = element("p");
  const points = element("p");
  inventory.append(
    cargo,
    tiers,
    stash,
    points,
    element(
      "p",
      "flight-status-note",
      "HOME BANKS CARGO / DEATH, RESTART, TITLE OR RELOAD LOSES CARGO",
    ),
  );
  target.replaceChildren(identity, vitals, abilities, inventory);
  const view = {
    icon,
    title,
    progress,
    stats,
    slots,
    cargo,
    tiers,
    stash,
    points,
  };
  views.set(target, view);
  return view;
}
function number(value: number, decimals = 0): string {
  return Number.isFinite(value) ? value.toFixed(decimals) : "--";
}
function tiers(values: number[]): string {
  return Array.from(
    { length: 5 },
    (_, index) => `T${index + 1} ${number(values[index])}`,
  ).join(" / ");
}

/** Displays authoritative HUD values. Stable nodes preserve live-menu scroll and focus. */
export function paintFlightMenu(
  target: HTMLElement,
  hud: FlightMenuHud,
): boolean {
  const previous = views.get(target);
  const view = previous ?? createView(target);
  let changed = !previous;
  const write = (node: HTMLElement, value: string): void => {
    if (node.textContent !== value) {
      node.textContent = value;
      changed = true;
    }
  };
  const ship = hud.campaign.ships.find(
    (entry) => entry.id === hud.campaign.selected_ship,
  );
  const iconValue = String(
    hud.campaign.selected_ship === 0 ? 9 : hud.campaign.selected_ship - 1,
  );
  if (view.icon.dataset.pixelValue !== iconValue) {
    view.icon.dataset.pixelValue = iconValue;
    changed = true;
  }
  write(view.title, ship?.name ?? "SHIP STATUS");
  write(
    view.progress,
    `LV ${number(hud.ship_level)} / XP ${number(hud.ship_xp)} / ${hud.ship_level >= 100 ? "MAX LEVEL" : `NEXT ${number(hud.ship_next_xp)}`}`,
  );
  const stats = hud.ship_stats;
  const values = [
    `HULL ${number(hud.hull)} / ${number(hud.max_hull)}`,
    `SHIELD ${number(hud.shield)} / ${number(hud.max_shield)}`,
    `SPEED ${number(stats.speed)} / ${number(stats.top_speed)} U/S`,
    `ACCEL ${number(stats.acceleration)} U/S2`,
    `PRIMARY ${number(stats.primary_damage, 1)} DMG / ${number(stats.fire_rate, 1)}/S`,
    `HULL REPAIR ${number(stats.hull_regen, 1)}/S AFTER 5S NO HIT`,
    `SHIELD REGEN ${number(stats.shield_regen, 1)}/S AFTER 4S NO HIT`,
  ];
  values.forEach((value, index) => write(view.stats[index], value));
  // Slot order is authoritative: offense, defense, passive.
  view.slots.forEach((node, index) => {
    const slot = hud.loadout[index];
    write(
      node,
      `${categories[index]} / ${slot ? `${slot.name} / ACTIVE R${number(slot.effective_rank)}` : "EMPTY"}`,
    );
  });
  write(
    view.cargo,
    `CARGO ${number(hud.cargo_materials)} / ${number(hud.cargo_capacity)} / AT RISK`,
  );
  write(view.tiers, tiers(hud.material_cargo));
  write(view.stash, `BANKED / ${tiers(hud.material_stash)}`);
  write(
    view.points,
    `POINTS / OFF ${number(hud.currency[0])} / DEF ${number(hud.currency[1])} / PASS ${number(hud.currency[2])}`,
  );
  return changed;
}
