import "./loadout.css";

export interface EquippedAbilityHud {
  id: string;
  index: number;
  name: string;
  category: number;
  rank: number;
  effective_rank: number;
}
export interface AbilityHud extends EquippedAbilityHud {
  description: string;
  blueprint: boolean;
  owned: boolean;
  max_rank: number;
  compatible: boolean;
  equipped: boolean;
  required_level: number;
  material_tier: number;
  material_cost: number;
  point_cost: number;
  can_craft: boolean;
  can_equip: boolean;
  can_upgrade: boolean;
  craft_reason: string;
  equip_reason: string;
  upgrade_reason: string;
  training_recipe: boolean;
  required_training_material: {
    id: string;
    name: string;
    amount: number;
  } | null;
}
export interface HardwareHud {
  id: string;
  name: string;
  category: number;
  rank: number;
  effective_rank: number;
  max_rank: number;
  description: string;
  required_level: number;
  material_tier: number;
  material_cost: number;
  point_cost: number;
  can_purchase: boolean;
  purchase_reason: string;
}
export interface LoadoutShipHud {
  id: number;
  name: string;
  ability: string;
  family: number;
  owned: boolean;
  selected: boolean;
  category: number;
  craftable: boolean;
  level: number;
  xp: number;
  next_xp: number;
  rank_cap: number;
  point_cost: number;
  material_tier: number;
  material_cost: number;
  can_craft: boolean;
  can_select: boolean;
  craft_reason: string;
  select_reason: string;
}
export interface ShipHangarHud {
  identity?: import("./kikai").IdentityHud | null;
  dock: boolean;
  currency: number[];
  material_stash: number[];
  material_cargo: number[];
  material_training_cargo?: number[];
  material_training_stash?: number[];
  material_loss: number[];
  material_loss_reason: string;
  campaign: { selected_ship: number; ships: LoadoutShipHud[] };
}
export interface ForgeHud extends ShipHangarHud {
  ship_level: number;
  ship_xp: number;
  ship_next_xp: number;
  rank_cap: number;
  abilities: AbilityHud[];
  loadout: (EquippedAbilityHud | null)[];
  upgrades: HardwareHud[];
}
export interface ForgeActions {
  purchase: (id: string) => void;
  craftAbility: (id: string) => void;
  upgradeAbility: (id: string) => void;
  equipAbility: (category: number, id: string | null) => void;
  busy?: boolean;
}
export interface ShipHangarActions {
  craft: (champion: number) => void;
  select: (ship: number) => void;
  busy?: boolean;
}

const categories = ["OFFENSIVE", "DEFENSIVE", "PASSIVE"];
const colors = ["offense", "defense", "passive"];
interface ForgeView {
  category: number | null;
  ship: number;
}
const views = new WeakMap<HTMLElement, ForgeView>();

function text(tag: "h3" | "p", value: string, className = ""): HTMLElement {
  const node = document.createElement(tag);
  node.textContent = value;
  node.className = className;
  return node;
}
function section(className: string): HTMLElement {
  const node = document.createElement("div");
  node.className = className;
  return node;
}
function button(
  label: string,
  key: string,
  action: () => void,
  disabled = false,
  category?: number,
): HTMLButtonElement {
  const node = document.createElement("button");
  node.textContent = label;
  node.dataset.loadoutAction = key;
  node.disabled = disabled;
  if (category !== undefined)
    node.dataset.pixelAction = `${colors[category]}-button`;
  node.addEventListener("click", () => {
    if (!node.disabled) action();
  });
  return node;
}
function progress(level: number, xp: number, nextXp: number): string {
  return `RAICHO LV ${level} / ${xp} XP / ${level >= 100 ? "MAX LEVEL" : `NEXT ${nextXp} XP`}`;
}
function tiers(values: number[]): string {
  return values.map((value, index) => `T${index + 1} ${value}`).join(" / ");
}
function rank(
  ability: Pick<EquippedAbilityHud, "rank" | "effective_rank">,
): string {
  return ability.rank === ability.effective_rank
    ? `RANK ${ability.rank}`
    : `OWNED RANK ${ability.rank} / ACTIVE RANK ${ability.effective_rank}`;
}
function categoryHeading(category: number, points: number): HTMLElement {
  const heading = section(`upgrade-category ${colors[category]}`);
  heading.append(text("p", `${categories[category]} / ${points} PTS`));
  return heading;
}
function recipe(
  category: number,
  pointCost: number,
  materialTier: number,
  materialCost: number,
  hud: ShipHangarHud,
  named?: AbilityHud["required_training_material"],
): HTMLElement {
  const box = section("loadout-recipe");
  if (named) {
    const index = [
      "phase-silicate",
      "iron-filament",
      "coolant-crystal",
    ].indexOf(named.id);
    box.append(
      text(
        "p",
        `FIRST UPGRADE / ${named.name.toUpperCase()} ${named.amount} / BANKED ${hud.material_training_stash?.[index] ?? 0}`,
      ),
    );
    return box;
  }
  box.append(
    text(
      "p",
      `${categories[category]} POINTS ${pointCost} / HAVE ${hud.currency[category] ?? 0}`,
    ),
  );
  box.append(
    text(
      "p",
      `T${materialTier} MATERIALS ${materialCost} / BANKED ${hud.material_stash[materialTier - 1] ?? 0}`,
    ),
  );
  return box;
}
function materialNotes(hud: ShipHangarHud): HTMLElement {
  const box = section("loadout-material-notes");
  box.append(
    text(
      "p",
      "CARGO: Bank by arriving at Haven. Lost on death, restart, title or reload.",
      "loadout-note",
    ),
  );
  if (hud.material_cargo.some((value) => value > 0))
    box.append(
      text(
        "p",
        `CARRIED / ${tiers(hud.material_cargo)} / NOT BANKED`,
        "loadout-loss",
      ),
    );
  if (hud.material_loss.some((value) => value > 0))
    box.append(
      text(
        "p",
        `MATERIALS LOST${hud.material_loss_reason ? ` / ${hud.material_loss_reason.toUpperCase()}` : ""} / ${tiers(hud.material_loss)}`,
        "loadout-loss",
      ),
    );
  if (!hud.dock)
    box.append(
      text("p", "Return to Haven to craft, upgrade or equip.", "loadout-note"),
    );
  return box;
}
function materials(hud: ShipHangarHud): HTMLElement {
  const box = section("loadout-summary");
  box.append(
    text("p", `DOCK STASH / ${tiers(hud.material_stash)}`),
    materialNotes(hud),
  );
  return box;
}
function restore(
  target: HTMLElement,
  focusKey: string | undefined,
  scroll: number,
): void {
  target.scrollTop = scroll;
  if (focusKey) {
    const match = Array.from(
      target.querySelectorAll<HTMLButtonElement>("button[data-loadout-action]"),
    ).find((node) => node.dataset.loadoutAction === focusKey && !node.disabled);
    match?.focus({ preventScroll: true });
  }
}

/** Reset only view navigation; the game owns inventory, prices and progression. */
export function resetForge(target: HTMLElement): void {
  views.delete(target);
}

/** Dedicated Forge page. Availability/reasons are supplied by the real Game HUD. */
export function paintForge(
  target: HTMLElement,
  hud: ForgeHud,
  actions: ForgeActions,
): void {
  const current = views.get(target);
  const view =
    current?.ship === hud.campaign.selected_ship
      ? current
      : { category: null, ship: hud.campaign.selected_ship };
  views.set(target, view);
  const active = document.activeElement;
  const focusKey =
    active instanceof HTMLElement && target.contains(active)
      ? active.dataset.loadoutAction
      : undefined;
  const scroll = target.scrollTop;
  target.replaceChildren();
  const root = section("forge-view");
  const ship = hud.campaign.ships.find(
    (item) => item.id === hud.campaign.selected_ship,
  );
  const shipSummary = section("loadout-ship-summary");
  shipSummary.append(text("h3", ship?.name ?? "ACTIVE RAICHO"));
  shipSummary.append(
    text(
      "p",
      `${progress(hud.ship_level, hud.ship_xp, hud.ship_next_xp)} / RANK CAP ${hud.rank_cap}/5`,
    ),
  );
  const navigate = (category: number | null, nextFocus: string) => {
    view.category = category;
    paintForge(target, hud, actions);
    restore(target, nextFocus, 0);
  };
  if (view.category !== null) {
    const category = view.category;
    const editorHeading = section("loadout-editor-heading");
    const back = button("BACK TO LOADOUT", "back", () =>
      navigate(null, `manage-${category}`),
    );
    back.classList.add("loadout-back");
    editorHeading.append(
      back,
      categoryHeading(category, hud.currency[category] ?? 0),
    );
    root.append(editorHeading, shipSummary);
    root.append(
      text(
        "p",
        "Craft to own. Equip swaps this RAICHO's slot; replaced abilities stay owned.",
        "loadout-note",
      ),
    );
    const equipped = hud.loadout[category];
    if (equipped)
      root.append(
        button(
          "EMPTY SLOT",
          `empty-${category}`,
          () => actions.equipAbility(category, null),
          !hud.dock || actions.busy,
          category,
        ),
      );
    const list = section("ability-list");
    const order = (ability: AbilityHud) =>
      ability.equipped
        ? 0
        : ability.owned && ability.compatible
          ? 1
          : ability.can_craft
            ? 2
            : ability.owned
              ? 3
              : 4;
    for (const ability of hud.abilities
      .filter(
        (item) => item.category === category && (item.blueprint || item.owned),
      )
      .sort((a, b) => order(a) - order(b) || a.index - b.index)) {
      const card = section("ability-card");
      card.dataset.equipped = String(ability.equipped);
      card.append(text("h3", ability.name), text("p", ability.description));
      if (ability.owned)
        card.append(
          text(
            "p",
            `${rank(ability)}${ability.equipped ? " / EQUIPPED" : " / OWNED"}`,
          ),
        );
      const controls = section("loadout-actions");
      if (!ability.owned) {
        card.append(
          recipe(
            category,
            ability.point_cost,
            ability.material_tier,
            ability.material_cost,
            hud,
          ),
        );
        if (ability.craft_reason)
          card.append(text("p", ability.craft_reason, "loadout-reason"));
        controls.append(
          button(
            "CRAFT",
            `craft-${ability.id}`,
            () => actions.craftAbility(ability.id),
            !ability.can_craft || actions.busy,
            category,
          ),
        );
      } else {
        if (!ability.equipped) {
          if (ability.equip_reason)
            card.append(text("p", ability.equip_reason, "loadout-reason"));
          controls.append(
            button(
              equipped ? "REPLACE" : "EQUIP",
              `equip-${ability.id}`,
              () => actions.equipAbility(category, ability.id),
              !ability.can_equip || actions.busy,
              category,
            ),
          );
        }
        if (ability.rank < ability.max_rank) {
          card.append(
            recipe(
              category,
              ability.point_cost,
              ability.material_tier,
              ability.material_cost,
              hud,
              ability.required_training_material,
            ),
          );
          if (ability.upgrade_reason)
            card.append(text("p", ability.upgrade_reason, "loadout-reason"));
          controls.append(
            button(
              "UPGRADE",
              `upgrade-${ability.id}`,
              () => actions.upgradeAbility(ability.id),
              !ability.can_upgrade || actions.busy,
              category,
            ),
          );
        } else card.append(text("p", "MAX OWNED RANK", "loadout-note"));
      }
      card.append(controls);
      list.append(card);
    }
    root.append(list);
  } else {
    root.append(
      shipSummary,
      text("p", `DOCK STASH / ${tiers(hud.material_stash)}`),
      text("h3", "AUTOMATIC ABILITY SLOTS"),
    );
    const slots = section("loadout-slots");
    for (let category = 0; category < 3; category++) {
      const row = section("loadout-slot");
      const content = section("loadout-card-content");
      content.append(categoryHeading(category, hud.currency[category] ?? 0));
      const equipped = hud.loadout[category];
      content.append(text("h3", equipped?.name ?? "EMPTY SLOT"));
      if (equipped) content.append(text("p", rank(equipped)));
      row.append(
        content,
        button(
          "MANAGE",
          `manage-${category}`,
          () => navigate(category, "back"),
          false,
          category,
        ),
      );
      slots.append(row);
    }
    root.append(slots, materialNotes(hud), text("h3", "RAICHO MODULES"));
    root.append(
      text(
        "p",
        "Six RAICHO hardware upgrades. No ability slot required.",
        "loadout-note",
      ),
    );
    const hardware = section("hardware-list");
    for (const item of hud.upgrades) {
      const card = section("hardware-card");
      card.append(
        categoryHeading(item.category, hud.currency[item.category] ?? 0),
      );
      card.append(
        text("h3", item.name),
        text("p", item.description),
        text("p", `${rank(item)} / MAX ${item.max_rank}`),
      );
      if (item.rank < item.max_rank)
        card.append(
          recipe(
            item.category,
            item.point_cost,
            item.material_tier,
            item.material_cost,
            hud,
          ),
        );
      if (item.purchase_reason)
        card.append(text("p", item.purchase_reason, "loadout-reason"));
      card.append(
        button(
          item.rank >= item.max_rank ? "MAX RANK" : "UPGRADE",
          `hardware-${item.id}`,
          () => actions.purchase(item.id),
          !item.can_purchase || actions.busy,
          item.category,
        ),
      );
      hardware.append(card);
    }
    root.append(hardware);
  }
  target.append(root);
  restore(target, focusKey, current === view ? scroll : 0);
}

/** Ship construction and selection, with XP belonging to each individual hull. */
export function paintShipHangar(
  target: HTMLElement,
  hud: ShipHangarHud,
  actions: ShipHangarActions,
): void {
  const active = document.activeElement;
  const focusKey =
    active instanceof HTMLElement && target.contains(active)
      ? active.dataset.loadoutAction
      : undefined;
  const scroll = target.scrollTop;
  target.replaceChildren();
  const root = section("ship-hangar-view");
  root.append(materials(hud));
  root.append(
    text(
      "p",
      "RAICHO XP, hardware and equipped slots stay with their hull. Crafted RAICHO start at LV 1. Select a craft to fly it.",
      "loadout-note",
    ),
  );
  const roster = section("ship-hangar-roster");
  for (const ship of hud.campaign.ships) {
    const card = section("ship-hangar-card");
    card.dataset.selected = String(ship.selected);
    const icon = section("campaign-icon campaign-ship");
    const licensed = ship.id === 0 && hud.identity;
    icon.dataset.pixelKind = licensed ? "licensed-ship" : "campaign-ship";
    icon.dataset.pixelValue = String(
      licensed
        ? ["pan", "renko", "hugo"].indexOf(licensed.corporation)
        : ship.id === 0
          ? 9
          : ship.id - 1,
    );
    icon.dataset.active = String(ship.selected);
    icon.setAttribute("aria-hidden", "true");
    const content = section("loadout-card-content");
    content.append(text("h3", ship.name), text("p", ship.ability));
    if (ship.owned) {
      content.append(text("p", progress(ship.level, ship.xp, ship.next_xp)));
      if (ship.select_reason && !ship.selected)
        content.append(text("p", ship.select_reason, "loadout-reason"));
      content.append(
        button(
          ship.selected ? "ACTIVE RAICHO" : "SELECT RAICHO",
          `select-ship-${ship.id}`,
          () => actions.select(ship.id),
          !ship.can_select || ship.selected || actions.busy,
        ),
      );
    } else {
      const hasRecipe = ship.point_cost > 0 || ship.material_cost > 0;
      if (hasRecipe)
        content.append(
          recipe(
            ship.category,
            ship.point_cost,
            ship.material_tier,
            ship.material_cost,
            hud,
          ),
        );
      if (ship.craft_reason)
        content.append(text("p", ship.craft_reason, "loadout-reason"));
      content.append(
        button(
          hasRecipe || ship.can_craft ? "CRAFT RAICHO" : "LOCKED RAICHO",
          `craft-ship-${ship.id}`,
          () => actions.craft(ship.id - 1),
          !ship.can_craft || actions.busy,
        ),
      );
    }
    card.append(icon, content);
    roster.append(card);
  }
  root.append(roster);
  target.append(root);
  restore(target, focusKey, scroll);
}
