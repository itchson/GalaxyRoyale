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
  required_component: {
    id: string;
    name: string;
    amount: number;
    owned: number;
  } | null;
}
export interface ComponentHud {
  id: string;
  index: number;
  name: string;
  category: number;
  description: string;
  count: number;
  max_count: number;
  required_level: number;
  materials: number[];
  can_craft: boolean;
  craft_reason: string;
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
  components: ComponentHud[];
  loadout: (EquippedAbilityHud | null)[];
  upgrades: HardwareHud[];
}
export interface ForgeActions {
  purchase: (id: string) => void;
  craftAbility: (id: string) => void;
  craftComponent: (id: string) => void;
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
  workshop: boolean;
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
  node.dataset.pixelKind =
    tag === "h3"
      ? "collection-name"
      : className === "loadout-reason"
        ? "collection-reason"
        : className === "collection-state"
          ? "collection-state"
          : "collection-meta";
  return node;
}
function section(className: string): HTMLElement {
  const node = document.createElement("div");
  node.className = className;
  return node;
}
/** Presentation flags only: availability, prices and inventory come from Rust. */
function collectionCard(
  className: string,
  category = -1,
  selected = false,
): HTMLElement {
  const node = section(className);
  node.dataset.pixelKind = "collection-entry";
  node.dataset.pixelValue = String(category);
  node.dataset.active = String(selected);
  return node;
}
function symbol(
  kind: "category-symbol" | "corporation-symbol",
  value: number,
): HTMLElement {
  const node = section("collection-symbol");
  node.dataset.pixelKind = kind;
  node.dataset.pixelValue = String(value);
  node.setAttribute("aria-hidden", "true");
  return node;
}
function collectionHeading(name: string, category: number): HTMLElement {
  const node = section("collection-heading");
  node.append(symbol("category-symbol", category), text("h3", name));
  return node;
}
function collectionState(value: string, state: number): HTMLElement {
  const node = text("p", value, "collection-state");
  node.dataset.pixelValue = String(state);
  return node;
}
function cost(value: string): HTMLElement {
  const node = text("p", value);
  node.dataset.pixelKind = "collection-cost";
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
  heading.append(
    symbol("corporation-symbol", [1, 2, 0][category]),
    text(
      "p",
      `${categories[category]} / ${points} ${["RENKO", "HUGO", "PAN"][category]} POINTS`,
    ),
  );
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
  component?: AbilityHud["required_component"],
): HTMLElement {
  const box = section("loadout-recipe");
  if (named) {
    const index = [
      "phase-silicate",
      "iron-filament",
      "coolant-crystal",
    ].indexOf(named.id);
    box.append(
      cost(
        `FIRST UPGRADE / ${named.name.toUpperCase()} ${named.amount} / BANKED ${hud.material_training_stash?.[index] ?? 0}`,
      ),
    );
    return box;
  }
  box.append(
    cost(
      `${["RENKO", "HUGO", "PAN"][category]} POINTS ${pointCost} / HAVE ${hud.currency[category] ?? 0}`,
    ),
  );
  box.append(
    cost(
      `T${materialTier} MATERIALS ${materialCost} / BANKED ${hud.material_stash[materialTier - 1] ?? 0}`,
    ),
  );
  if (component)
    box.append(
      cost(
        `${component.name.toUpperCase()} ${component.amount} / BANKED ${component.owned}`,
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
          workshop: false,
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
    const previousScroll =
      !view.workshop && view.collection === collection ? target.scrollTop : 0;
    view.category = category;
    view.collection = collection;
    view.workshop = false;
    paintForge(target, hud, actions);
    restore(target, nextFocus, previousScroll);
  };
  const workshopButton = () =>
    button("CRAFT COMPONENTS", "browse-components", () => {
      view.workshop = true;
      paintForge(target, hud, actions);
      restore(target, "components-back", 0);
    });
  if (view.workshop) {
    const heading = section("loadout-editor-heading");
    heading.append(
      button("BACK TO LOADOUT", "components-back", () =>
        navigate(null, "browse-components"),
      ),
      text("h3", "COMPONENT WORKSHOP"),
    );
    root.append(
      heading,
      shipSummary,
      text(
        "p",
        "Crafted parts stay at Haven. Use them for advanced ability upgrades.",
        "loadout-note",
      ),
    );
    const workshop = section("component-list");
    for (const component of hud.components) {
      const card = collectionCard("component-card", component.category);
      card.setAttribute("aria-description", component.description);
      const header = section("component-heading");
      const icon = section("component-icon");
      icon.dataset.pixelKind = "craft-component";
      icon.dataset.pixelValue = String(component.index);
      icon.setAttribute("aria-hidden", "true");
      const label = section("loadout-card-content");
      label.append(
        text("h3", component.name),
        text(
          "p",
          `${["RENKO", "HUGO", "PAN"][component.category]} / ${categories[component.category]}`,
        ),
      );
      header.append(icon, label);
      card.append(
        header,
        text(
          "p",
          `BANKED ${component.count}/${component.max_count} / REQUIRES RAICHO LV ${component.required_level}`,
        ),
      );
      const costs = section("loadout-recipe");
      component.materials.forEach((amount, tier) => {
        if (amount > 0)
          costs.append(
            cost(
              `T${tier + 1} MATERIALS ${amount} / BANKED ${hud.material_stash[tier] ?? 0}`,
            ),
          );
      });
      card.append(costs);
      if (component.craft_reason)
        card.append(text("p", component.craft_reason, "loadout-reason"));
      card.append(
        button(
          "CRAFT ONE",
          `component-${component.id}`,
          () => actions.craftComponent(component.id),
          !component.can_craft || actions.busy,
          component.category,
        ),
      );
      workshop.append(card);
    }
    root.append(workshop);
    if (!hud.components.length)
      root.append(
        text("p", "No component recipes discovered.", "loadout-note"),
      );
  } else if (view.collection) {
    const editorHeading = section("loadout-editor-heading");
    const back = button("BACK TO LOADOUT", "back", () =>
      navigate(
        null,
        view.category === null ? "browse-abilities" : `manage-${view.category}`,
      ),
    );
    back.classList.add("loadout-back");
    editorHeading.append(back);
    root.append(
      editorHeading,
      text(
        "h3",
        `${ship?.name ?? "ACTIVE RAICHO"} / LV ${hud.ship_level} / RANK CAP ${hud.rank_cap}`,
      ),
    );
    if (hud.components.length) root.append(workshopButton());
    root.append(
      text(
        "p",
        "Craft to collect. Equip swaps your slot; replaced abilities stay owned.",
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
          restore(target, focus, target.scrollTop);
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
      const card = collectionCard("ability-card", category, ability.equipped);
      card.dataset.equipped = String(ability.equipped);
      card.dataset.abilityId = ability.id;
      card.append(
        collectionHeading(ability.name, category),
        text("p", ability.description),
      );
      if (ability.owned)
        card.append(
          collectionState(
            `${rank(ability)}${ability.equipped ? " / EQUIPPED" : " / OWNED"}`,
            ability.equipped ? 3 : 2,
          ),
        );
      else card.append(collectionState("BLUEPRINT / NOT CRAFTED", 1));
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
              ability.required_component,
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
      const row = collectionCard(
        "loadout-slot",
        category,
        !!hud.loadout[category],
      );
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
    if (hud.components.length) root.append(workshopButton());
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
      const card = collectionCard("hardware-card", item.category);
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
        restore(target, focus, target.scrollTop);
      },
    ),
  );
  root.append(
    text(
      "p",
      "Each RAICHO keeps its own XP and slots. Crafted hulls start at LV 1.",
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
    // Construction currency is not the hull's manufacturer.
    const card = collectionCard("ship-hangar-card", -1, ship.selected);
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
      collectionState(
        ship.selected
          ? "ACTIVE / OWNED"
          : ship.owned
            ? "OWNED"
            : ship.craftable
              ? "BLUEPRINT / NOT CRAFTED"
              : "FOUND / BLUEPRINT REQUIRED",
        ship.selected ? 3 : ship.owned ? 2 : ship.craftable ? 1 : 0,
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
