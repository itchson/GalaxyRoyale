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
  collection: boolean;
  filter: AbilityFilter;
}
type HullFilter = "all" | "owned" | "blueprint";
type AbilityFilter = HullFilter | "compatible";
const views = new WeakMap<HTMLElement, ForgeView>();
const hullViews = new WeakMap<HTMLElement, { filter: HullFilter }>();

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
function rank(
  ability: Pick<EquippedAbilityHud, "rank" | "effective_rank">,
): string {
  return ability.rank === ability.effective_rank
    ? `RANK ${ability.rank}`
    : `OWNED RANK ${ability.rank} / ACTIVE RANK ${ability.effective_rank}`;
}
function categoryHeading(category: number, points: number): HTMLElement {
  const heading = section(`upgrade-category ${colors[category]}`);
  heading.append(text("p", `${categories[category]} / ${points} MEISHI`));
  return heading;
}
function filters<T extends string>(
  name: string,
  choices: readonly T[],
  selected: T,
  change: (value: T, focus: string) => void,
): HTMLElement {
  const row = section("collection-filters");
  row.setAttribute("role", "group");
  row.setAttribute("aria-label", `${name} filters`);
  for (const value of choices) {
    const key = `${name}-filter-${value}`;
    const control = button(value.toUpperCase(), key, () => change(value, key));
    control.dataset.selected = String(selected === value);
    control.setAttribute("aria-pressed", String(selected === value));
    row.append(control);
  }
  return row;
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
      `${categories[category]} MEISHI ${pointCost} / HAVE ${hud.currency[category] ?? 0}`,
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
function restore(
  target: HTMLElement,
  focusKey: string | undefined,
  scroll: number,
): void {
  target.scrollTop = scroll;
  if (focusKey) {
    const controls = Array.from(
      target.querySelectorAll<HTMLButtonElement>("button[data-loadout-action]"),
    ).filter((node) => !node.disabled);
    const replacement = focusKey.startsWith("craft-")
      ? focusKey.replace(/^craft-/, "equip-")
      : focusKey.startsWith("empty-")
        ? focusKey.replace(/^empty-/, "ability-category-")
        : undefined;
    const match =
      controls.find((node) => node.dataset.loadoutAction === focusKey) ??
      controls.find((node) => node.dataset.loadoutAction === replacement) ??
      controls.find((node) => node.dataset.selected === "true") ??
      controls[0];
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
  const view: ForgeView =
    current?.ship === hud.campaign.selected_ship
      ? current
      : {
          category: null,
          ship: hud.campaign.selected_ship,
          collection: false,
          filter: "all",
        };
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
  const navigate = (
    category: number | null,
    nextFocus: string,
    collection = category !== null,
  ) => {
    view.category = category;
    view.collection = collection;
    paintForge(target, hud, actions);
    restore(target, nextFocus, 0);
  };
  if (view.collection) {
    const editorHeading = section("loadout-editor-heading");
    const back = button("BACK TO LOADOUT", "back", () =>
      navigate(
        null,
        view.category === null ? "browse-abilities" : `manage-${view.category}`,
      ),
    );
    back.classList.add("loadout-back");
    editorHeading.append(back, text("h3", "ABILITY COLLECTION"));
    root.append(editorHeading, shipSummary);
    root.append(
      text(
        "p",
        "Craft to own. Equip swaps this RAICHO's slot; replaced abilities stay owned.",
        "loadout-note",
      ),
    );
    const categoryControls = section("collection-filters");
    categoryControls.setAttribute("role", "group");
    categoryControls.setAttribute("aria-label", "Ability categories");
    for (const category of [null, 0, 1, 2]) {
      const key = `ability-category-${category ?? "all"}`;
      const control = button(
        category === null ? "ALL SLOTS" : categories[category],
        key,
        () => navigate(category, key, true),
        false,
        category ?? undefined,
      );
      control.dataset.selected = String(view.category === category);
      control.setAttribute("aria-pressed", String(view.category === category));
      categoryControls.append(control);
    }
    root.append(
      categoryControls,
      filters(
        "ability",
        ["all", "owned", "blueprint", "compatible"] as const,
        view.filter,
        (value, focus) => {
          view.filter = value;
          paintForge(target, hud, actions);
          restore(target, focus, 0);
        },
      ),
    );
    const known = hud.abilities.filter((item) => item.blueprint || item.owned);
    root.append(
      text(
        "p",
        `${known.length} FOUND / ${known.filter((item) => item.owned).length} OWNED`,
        "collection-summary",
      ),
    );
    const slot = view.category;
    if (slot !== null && hud.loadout[slot])
      root.append(
        button(
          "EMPTY SLOT",
          `empty-${slot}`,
          () => actions.equipAbility(slot, null),
          !hud.dock || actions.busy,
          slot,
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
    const visible = known
      .filter(
        (item) =>
          (view.category === null || item.category === view.category) &&
          (view.filter === "all" ||
            (view.filter === "owned" && item.owned) ||
            (view.filter === "blueprint" && item.blueprint && !item.owned) ||
            (view.filter === "compatible" && item.compatible)),
      )
      .sort((a, b) => order(a) - order(b) || a.index - b.index);
    for (const ability of visible) {
      const category = ability.category;
      const equipped = hud.loadout[category];
      const card = section("ability-card");
      card.dataset.equipped = String(ability.equipped);
      card.dataset.abilityId = ability.id;
      card.append(text("h3", ability.name), text("p", ability.description));
      if (ability.owned)
        card.append(
          text(
            "p",
            `${rank(ability)}${ability.equipped ? " / EQUIPPED" : " / OWNED"}`,
          ),
        );
      else
        card.append(text("p", "BLUEPRINT / NOT CRAFTED", "collection-state"));
      card.append(
        text(
          "p",
          `${categories[category]} / ${ability.compatible ? "COMPATIBLE" : "INCOMPATIBLE"} WITH ACTIVE HULL`,
          "loadout-note",
        ),
      );
      if (ability.rank < ability.max_rank)
        card.append(
          text(
            "p",
            `REQUIRES RAICHO LV ${ability.required_level}`,
            "loadout-note",
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
    if (!visible.length)
      list.append(
        text("p", "No found abilities match these filters.", "loadout-note"),
      );
    root.append(list);
  } else {
    root.append(shipSummary, text("h3", "AUTOMATIC ABILITY SLOTS"));
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
    root.append(
      slots,
      button("BROWSE ABILITIES", "browse-abilities", () =>
        navigate(null, "back", true),
      ),
    );
    if (!hud.dock)
      root.append(
        text(
          "p",
          "Return to Haven to craft, upgrade or equip.",
          "loadout-note",
        ),
      );
    root.append(text("h3", "RAICHO MODULES"));
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
          text(
            "p",
            `REQUIRES RAICHO LV ${item.required_level}`,
            "loadout-note",
          ),
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
  const view: { filter: HullFilter } = hullViews.get(target) ?? {
    filter: "all",
  };
  hullViews.set(target, view);
  const active = document.activeElement;
  const focusKey =
    active instanceof HTMLElement && target.contains(active)
      ? active.dataset.loadoutAction
      : undefined;
  const scroll = target.scrollTop;
  target.replaceChildren();
  const root = section("ship-hangar-view");
  root.append(
    text("h3", "HULL COLLECTION"),
    text(
      "p",
      `${hud.campaign.ships.length} FOUND / ${hud.campaign.ships.filter((ship) => ship.owned).length} OWNED`,
      "collection-summary",
    ),
    filters(
      "hull",
      ["all", "owned", "blueprint"] as const,
      view.filter,
      (value, focus) => {
        view.filter = value;
        paintShipHangar(target, hud, actions);
        restore(target, focus, 0);
      },
    ),
  );
  root.append(
    text(
      "p",
      "RAICHO XP, hardware and equipped slots stay with their hull. Crafted RAICHO start at LV 1. Select a craft to fly it.",
      "loadout-note",
    ),
  );
  const roster = section("ship-hangar-roster");
  // The core supplies only genuinely found hulls; there is no local full catalog.
  const visible = hud.campaign.ships.filter(
    (ship) =>
      view.filter === "all" ||
      (view.filter === "owned" && ship.owned) ||
      (view.filter === "blueprint" && !ship.owned && ship.craftable),
  );
  for (const ship of visible) {
    const card = section("ship-hangar-card");
    card.dataset.selected = String(ship.selected);
    card.dataset.shipId = String(ship.id);
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
    content.append(
      text(
        "p",
        ship.selected
          ? "ACTIVE / OWNED"
          : ship.owned
            ? "OWNED"
            : ship.craftable
              ? "BLUEPRINT / NOT CRAFTED"
              : "FOUND / BLUEPRINT REQUIRED",
        "collection-state",
      ),
    );
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
  if (!visible.length)
    roster.append(
      text("p", "No found hulls match this filter.", "loadout-note"),
    );
  root.append(roster);
  target.append(root);
  restore(target, focusKey, scroll);
}
