import "./hangar-scene.css";
import type { LoadoutShipHud, ShipHangarHud } from "./loadout";

export interface HangarSceneActions {
  select(id: number): void;
  details(): void;
  busy?: boolean;
}
type Hull = Pick<LoadoutShipHud, "id" | "name" | "level" | "xp" | "can_select">;
interface Model {
  corporation: number;
  dock: boolean;
  busy: boolean;
  selected: number | null;
  hulls: Hull[];
}
interface HullView {
  root: HTMLElement;
  button: HTMLButtonElement;
  art: HTMLElement;
  name: HTMLElement;
  progress: HTMLElement;
}
interface View {
  root: HTMLElement;
  target: HTMLElement;
  hero: HTMLButtonElement;
  heroArt: HTMLElement;
  name: HTMLElement;
  progress: HTMLElement;
  index: HTMLElement;
  guidance: HTMLElement;
  rows: Map<number, HullView>;
  model: Model;
  actions: HangarSceneActions;
  signature: string;
}
const views = new WeakMap<HTMLElement, View>();
const corporations = ["pan", "renko", "hugo"];

function model(hud: ShipHangarHud, actions: HangarSceneActions): Model {
  // The native HUD already owns discovery/ownership. No local hull catalog,
  // blueprint inference, inventory, prices or gameplay state lives here.
  const ids = new Set<number>();
  const hulls: Hull[] = [];
  for (const ship of hud.campaign.ships) {
    if (
      !ship.owned ||
      !Number.isSafeInteger(ship.id) ||
      ship.id < 0 ||
      ids.has(ship.id)
    )
      continue;
    if (
      typeof ship.name !== "string" ||
      !ship.name.length ||
      !Number.isSafeInteger(ship.level) ||
      ship.level < 1 ||
      !Number.isSafeInteger(ship.xp) ||
      ship.xp < 0
    )
      continue;
    ids.add(ship.id);
    hulls.push({
      id: ship.id,
      name: ship.name,
      level: ship.level,
      xp: ship.xp,
      can_select: ship.can_select === true,
    });
  }
  const selected = hulls.some((ship) => ship.id === hud.campaign.selected_ship)
    ? hud.campaign.selected_ship
    : null;
  return {
    corporation: Math.max(
      0,
      corporations.indexOf(hud.identity?.corporation ?? "pan"),
    ),
    dock: hud.dock === true,
    busy: actions.busy === true,
    selected,
    hulls,
  };
}
function element(tag: string, className: string): HTMLElement {
  const node = document.createElement(tag);
  node.className = className;
  return node;
}
function artwork(className: string): HTMLElement {
  const node = element("span", className);
  node.setAttribute("aria-hidden", "true");
  return node;
}
function caption(tag: "h3" | "p", className: string): HTMLElement {
  const node = element(tag, className);
  node.dataset.pixelKind = tag === "h3" ? "collection-name" : "collection-meta";
  return node;
}
function build(
  target: HTMLElement,
  current: Model,
  actions: HangarSceneActions,
): View {
  const root = element("div", "hangar-garage");
  const hero = document.createElement("button");
  hero.type = "button";
  hero.className = "hangar-showcase";
  hero.dataset.pixelKind = "garage-bay";
  const heroArt = artwork("hangar-showcase-art");
  hero.append(heroArt);
  const name = caption("h3", "hangar-active-name");
  const progress = caption("p", "hangar-active-progress");
  const summary = element("div", "hangar-active-caption");
  summary.append(name, progress);
  const index = element("ul", "hangar-index");
  index.setAttribute("aria-label", "Owned RAICHO index");
  const guidance = caption("p", "hangar-guidance");
  guidance.setAttribute("role", "status");
  root.append(hero, summary, index, guidance);
  target.replaceChildren(root);
  const view: View = {
    root,
    target,
    hero,
    heroArt,
    name,
    progress,
    index,
    guidance,
    rows: new Map(),
    model: current,
    actions,
    signature: "",
  };
  hero.addEventListener("click", () => {
    if (target.contains(root) && !hero.disabled && view.model.selected !== null)
      view.actions.details();
  });
  index.addEventListener("keydown", (event) => {
    if (
      !["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key) ||
      event.isComposing ||
      event.altKey ||
      event.ctrlKey ||
      event.metaKey
    )
      return;
    const buttons = [...view.rows.values()]
      .map((row) => row.button)
      .filter((button) => !button.disabled);
    const current = buttons.indexOf(
      document.activeElement as HTMLButtonElement,
    );
    if (current < 0 || !target.contains(root)) return;
    event.preventDefault();
    const next =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? buttons.length - 1
          : Math.max(
              0,
              Math.min(
                buttons.length - 1,
                current + (event.key === "ArrowRight" ? 1 : -1),
              ),
            );
    buttons[next].focus();
    buttons[next].scrollIntoView({ block: "nearest", inline: "nearest" });
  });
  return view;
}
function hullView(view: View, ship: Hull): HullView {
  const root = element("li", "hangar-hull-item");
  const button = document.createElement("button");
  button.type = "button";
  button.id = `hangar-hull-${ship.id}`;
  button.className = "hangar-hull";
  button.dataset.pixelKind = "collection-entry";
  button.dataset.pixelValue = "-1";
  const art = artwork("hangar-hull-art");
  const name = caption("h3", "hangar-hull-name");
  const progress = caption("p", "hangar-hull-progress");
  button.append(art, name, progress);
  root.append(button);
  const row = { root, button, art, name, progress };
  button.addEventListener("click", () => {
    const current = view.model.hulls.find((hull) => hull.id === ship.id);
    if (
      !view.target.contains(view.root) ||
      view.rows.get(ship.id) !== row ||
      !current ||
      button.disabled ||
      view.model.busy ||
      !view.model.dock ||
      !current.can_select ||
      view.model.selected === ship.id
    )
      return;
    // The next authoritative HUD chooses the showcased hull; clicking cannot
    // optimistically change ownership, selection or the separate detail scene.
    view.actions.select(ship.id);
  });
  return row;
}

/** Existing native garage/art nodes paint this semantic scene. */
export function paintHangarScene(
  target: HTMLElement,
  hud: ShipHangarHud,
  actions: HangarSceneActions,
): boolean {
  const current = model(hud, actions);
  let view = views.get(target),
    mounted = false;
  if (!view || !target.contains(view.root)) {
    view = build(target, current, actions);
    views.set(target, view);
    mounted = true;
  }
  view.model = current;
  view.actions = actions;
  const signature = JSON.stringify(current);
  if (!mounted && signature === view.signature) return false;
  view.signature = signature;
  let changed = mounted;
  const text = (node: HTMLElement, value: string) => {
    if (node.textContent !== value) {
      node.textContent = value;
      changed = true;
    }
  };
  const attribute = (node: HTMLElement, key: string, value: string) => {
    if (node.getAttribute(key) !== value) {
      node.setAttribute(key, value);
      changed = true;
    }
  };
  const hidden = (node: HTMLElement, value: boolean) => {
    if (node.hidden !== value) {
      node.hidden = value;
      changed = true;
    }
  };
  const disabled = (node: HTMLButtonElement, value: boolean) => {
    if (node.disabled !== value) {
      node.disabled = value;
      changed = true;
    }
  };
  const art = (node: HTMLElement, id: number) => {
    attribute(
      node,
      "data-pixel-kind",
      id === 0 ? "licensed-ship" : "campaign-ship",
    );
    attribute(
      node,
      "data-pixel-value",
      String(id === 0 ? current.corporation : id - 1),
    );
  };
  const progress = (ship: Hull) => `LV ${ship.level} / ${ship.xp} XP`;
  const selected = current.hulls.find((ship) => ship.id === current.selected);
  attribute(view.hero, "data-pixel-value", String(current.corporation));
  attribute(
    view.hero,
    "aria-label",
    selected ? `Open ${selected.name} details` : "No active RAICHO",
  );
  disabled(view.hero, !selected);
  hidden(view.heroArt, !selected);
  if (selected) art(view.heroArt, selected.id);
  text(view.name, selected?.name ?? "NO ACTIVE RAICHO");
  text(view.progress, selected ? progress(selected) : "");
  text(
    view.guidance,
    !current.hulls.length
      ? "NO OWNED RAICHO"
      : current.busy
        ? "UPDATING RAICHO"
        : !current.dock
          ? "RETURN TO HAVEN TO CHANGE RAICHO"
          : "SELECT A RAICHO / OPEN THE SHOWCASE FOR DETAILS",
  );
  const membershipChanged =
    [...view.rows.keys()].join() !==
    current.hulls.map((ship) => ship.id).join();
  if (membershipChanged) {
    const next = new Map<number, HullView>();
    for (const ship of current.hulls)
      next.set(ship.id, view.rows.get(ship.id) ?? hullView(view, ship));
    view.rows = next;
    // Reorder only when actual owned membership/order changes. Existing node
    // identities and horizontal scroll remain intact during frequent HUD polls.
    const scroll = view.index.scrollLeft,
      focus = document.activeElement as HTMLElement | null;
    const hadFocus = focus !== null && view.index.contains(focus);
    view.index.replaceChildren(...[...next.values()].map((row) => row.root));
    view.index.scrollLeft = scroll;
    if (hadFocus && focus && view.index.contains(focus))
      focus.focus({ preventScroll: true });
    changed = true;
  }
  for (const ship of current.hulls) {
    const row = view.rows.get(ship.id)!;
    const active = ship.id === current.selected;
    art(row.art, ship.id);
    text(row.name, ship.name);
    text(row.progress, progress(ship));
    attribute(
      row.button,
      "aria-label",
      `${ship.name} / ${progress(ship)}${active ? " / CURRENT RAICHO" : ""}`,
    );
    attribute(row.button, "aria-pressed", String(active));
    attribute(row.button, "data-active", String(active));
    // The current row stays focusable; it has no select mutation. Real native
    // can_select/dock gates disable other rows, independently of presentation.
    disabled(
      row.button,
      current.busy || (!active && (!current.dock || !ship.can_select)),
    );
  }
  return changed;
}
