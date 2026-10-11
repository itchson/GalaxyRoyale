import "./flight-menu.css";
import type { CampaignHud } from "./campaign";
import type { EquippedAbilityHud } from "./loadout";
import type { IdentityHud } from "./kikai";
import { paintLicensePortrait } from "./kikai-portrait";

export interface FlightMenuHud {
  identity?: IdentityHud | null;
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
    shield_recharge_unlocked?: boolean;
    affinity?: {
      corporation: "pan" | "renko" | "hugo";
      slot: 0 | 1 | 2;
      active: boolean;
      source: "factory_license" | "roster_manufacturer" | "sovereign_license";
    } | null;
  };
  loadout: (EquippedAbilityHud | null)[];
  material_cargo: number[];
  material_stash: number[];
  material_training_cargo?: number[];
  material_training_stash?: number[];
  cargo_materials: number;
  cargo_capacity: number;
  currency: number[];
}

interface FlightView {
  portrait: HTMLElement;
  icon: HTMLElement;
  title: HTMLElement;
  progress: HTMLElement;
  license: HTMLElement;
  fullLicense: HTMLElement;
  affinity: HTMLElement;
  stats: HTMLElement[];
  slots: HTMLElement[];
  cargo: HTMLElement;
  tiers: HTMLElement;
  stash: HTMLElement;
  points: HTMLElement;
  namedCargo: HTMLElement;
  namedStash: HTMLElement;
  details: HTMLElement;
}
const views = new WeakMap<HTMLElement, FlightView>();
let nextDetailsId = 0;
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
  const portrait = element("section", "flight-license-portrait");
  portrait.hidden = true;
  portrait.setAttribute("role", "img");
  const icon = element("section", "campaign-icon campaign-ship");
  icon.dataset.pixelKind = "campaign-ship";
  icon.dataset.active = "true";
  icon.setAttribute("aria-hidden", "true");
  const heading = element("div");
  const title = element("h3");
  const progress = element("p");
  const license = element("p", "flight-license");
  const fullLicense = element("p", "flight-license-detail");
  const affinity = element("p", "flight-affinity");
  affinity.hidden = true;
  heading.append(title, license, progress);
  identity.append(portrait, heading, icon);
  const vitals = element("section", "flight-status-stats");
  vitals.setAttribute("aria-label", "Current effective RAICHO statistics");
  const stats = Array.from({ length: 7 }, () => element("p"));
  vitals.append(...stats.slice(0, 2));
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
  const namedCargo = element("p");
  const namedStash = element("p");
  inventory.append(cargo, points);
  const details = element("section", "flight-status-details");
  details.id = `flight-status-details-${++nextDetailsId}`;
  details.hidden = true;
  details.setAttribute(
    "aria-label",
    "Detailed RAICHO statistics and inventory",
  );
  const detailStats = element("div", "flight-status-detail-stats");
  detailStats.append(...stats.slice(2));
  const detailInventory = element("div", "flight-status-detail-inventory");
  detailInventory.append(
    tiers,
    namedCargo,
    stash,
    namedStash,
    element(
      "p",
      "flight-status-note",
      "HOME BANKS CARGO / DEATH, RESTART, TITLE OR RELOAD LOSES CARGO",
    ),
  );
  details.append(fullLicense, affinity, detailStats, detailInventory);
  const toggle = element(
    "button",
    "flight-status-toggle",
    "DETAILS",
  ) as HTMLButtonElement;
  toggle.type = "button";
  toggle.setAttribute("aria-expanded", "false");
  toggle.setAttribute("aria-controls", details.id);
  toggle.addEventListener("click", () => {
    details.hidden = !details.hidden;
    toggle.setAttribute("aria-expanded", String(!details.hidden));
    toggle.textContent = details.hidden ? "DETAILS" : "HIDE DETAILS";
  });
  target.replaceChildren(
    identity,
    vitals,
    abilities,
    inventory,
    toggle,
    details,
  );
  const view = {
    portrait,
    icon,
    title,
    progress,
    license,
    fullLicense,
    affinity,
    stats,
    slots,
    cargo,
    tiers,
    stash,
    points,
    namedCargo,
    namedStash,
    details,
  };
  views.set(target, view);
  return view;
}
function number(value: number, decimals = 0): string {
  return Number.isFinite(value) ? value.toFixed(decimals) : "--";
}
function tiers(values: number[]): string {
  return (
    Array.from({ length: 5 }, (_, index) =>
      values[index] === 0 ? "" : `T${index + 1} ${number(values[index])}`,
    )
      .filter(Boolean)
      .join(" / ") || "NONE"
  );
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
  if (view.portrait.hidden !== !hud.identity) {
    view.portrait.hidden = !hud.identity;
    changed = true;
  }
  if (hud.identity) {
    changed = paintLicensePortrait(view.portrait, hud.identity) || changed;
  }
  const iconValue = String(
    hud.campaign.selected_ship === 0 && hud.identity
      ? ["pan", "renko", "hugo"].indexOf(hud.identity.corporation)
      : hud.campaign.selected_ship === 0
        ? 9
        : hud.campaign.selected_ship - 1,
  );
  const iconKind =
    hud.campaign.selected_ship === 0 && hud.identity
      ? "licensed-ship"
      : "campaign-ship";
  if (view.icon.dataset.pixelKind !== iconKind) {
    view.icon.dataset.pixelKind = iconKind;
    changed = true;
  }
  if (view.icon.dataset.pixelValue !== iconValue) {
    view.icon.dataset.pixelValue = iconValue;
    changed = true;
  }
  write(view.title, ship?.name ?? "RAICHO STATUS");
  write(
    view.license,
    hud.identity
      ? `LICENSE ${hud.identity.short_designation}`
      : "LEGACY LICENSE",
  );
  write(
    view.fullLicense,
    hud.identity
      ? `${hud.identity.designation} / ${hud.identity.corporation_role}`
      : "LEGACY PILOT LICENSE",
  );
  write(
    view.progress,
    `LV ${number(hud.ship_level)} / XP ${number(hud.ship_xp)}${hud.ship_level >= 100 ? " / MAX LEVEL" : `/${number(hud.ship_next_xp)}`}`,
  );
  const stats = hud.ship_stats;
  const affinity = stats.affinity;
  const affinityText = affinity
    ? `${affinity.source === "sovereign_license" ? "LICENSED " : ""}${affinity.corporation.toUpperCase()} / ${categories[affinity.slot]} AFFINITY ${affinity.active ? "ACTIVE" : "INACTIVE"}`
    : "";
  if (view.affinity.hidden !== !affinityText) {
    view.affinity.hidden = !affinityText;
    changed = true;
  }
  write(view.affinity, affinityText);
  const values = [
    `HULL ${number(hud.hull)} / ${number(hud.max_hull)}`,
    `SHIELD ${number(hud.shield)} / ${number(hud.max_shield)}`,
    `SPEED ${number(stats.speed)} / ${number(stats.top_speed)} U/S`,
    `ACCEL ${number(stats.acceleration)} U/S2`,
    `PRIMARY ${number(stats.primary_damage, 1)} DMG / ${number(stats.fire_rate, 1)}/S`,
    `HULL REPAIR ${number(stats.hull_regen, 1)}/S AFTER 5S NO HIT`,
    stats.shield_recharge_unlocked === false || stats.shield_regen <= 0
      ? "SHIELD RECHARGE LOCKED / UPGRADE AEGIS OR GUARD COIL"
      : `SHIELD RECHARGE ${number(stats.shield_regen, 1)}/S AFTER 4S NO HIT`,
  ];
  values.forEach((value, index) => write(view.stats[index], value));
  // Slot order is authoritative: offense, defense, passive.
  view.slots.forEach((node, index) => {
    const slot = hud.loadout[index];
    write(
      node,
      `${categories[index]} / ${slot ? `${slot.name} / R${number(slot.effective_rank)}` : "EMPTY"}`,
    );
  });
  write(
    view.cargo,
    `CARGO ${number(hud.cargo_materials)}/${number(hud.cargo_capacity)}${hud.cargo_materials > 0 ? " / AT RISK" : ""}`,
  );
  write(view.tiers, `CARRIED / ${tiers(hud.material_cargo)}`);
  write(view.stash, `BANKED / ${tiers(hud.material_stash)}`);
  for (const [node, counts, label] of [
    [view.namedCargo, hud.material_training_cargo, "CARRIED T1 PARTS"],
    [view.namedStash, hud.material_training_stash, "BANKED T1 PARTS"],
  ] as const) {
    const parts =
      counts?.flatMap((count, index) =>
        count > 0
          ? [
              `${["PHASE SILICATE", "IRON FILAMENT", "COOLANT KESSHO"][index]} ${number(count)}`,
            ]
          : [],
      ) ?? [];
    const value = parts.length ? `${label} / ${parts.join(" / ")}` : "";
    if (node.hidden !== !value) {
      node.hidden = !value;
      changed = true;
    }
    write(node, value);
  }
  write(
    view.points,
    `POINTS OFF ${number(hud.currency[0])} / DEF ${number(hud.currency[1])} / PASS ${number(hud.currency[2])}`,
  );
  return changed;
}
