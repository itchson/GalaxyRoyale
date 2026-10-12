import "./meishi.css";
import type { Champion } from "./campaign";
import type { IdentityHud } from "./kikai";
import { paintLicensePortrait } from "./kikai-portrait";
import type { LoadoutShipHud } from "./loadout";

/** Read-only card data from the authoritative Game HUD, never a second save. */
export interface MeishiHud {
  identity?: IdentityHud | null;
  campaign: {
    selected_ship: number;
    ships: Pick<LoadoutShipHud, "id" | "name">[];
    champions: Pick<Champion, "index" | "name" | "patch">[];
  };
  ship_level: number;
  currency: number[];
}

interface PatchSlot {
  row: HTMLElement;
  well: HTMLElement;
  icon: HTMLElement;
  empty: HTMLElement;
  caption: HTMLElement;
}
interface CardView {
  card: HTMLElement;
  portraitFrame: HTMLElement;
  seal: HTMLElement;
  issuer: HTMLElement;
  portrait: HTMLElement;
  designation: HTMLElement;
  manufacturer: HTMLElement;
  shipIcon: HTMLElement;
  shipName: HTMLElement;
  progress: HTMLElement;
  points: HTMLElement[];
  patches: PatchSlot[];
  patchCount: HTMLElement;
}
const views = new WeakMap<HTMLElement, CardView>();
const corporations = ["pan", "renko", "hugo"] as const;
const kinds = ["dora", "doro", "talo"] as const;
const colors = ["offense", "defense", "passive"];

function element(tag: string, className: string, text = ""): HTMLElement {
  const node = document.createElement(tag);
  node.className = className;
  if (text) node.textContent = text;
  return node;
}

function createView(target: HTMLElement): CardView {
  const card = element("article", "meishi-card");
  card.setAttribute("aria-label", "MEISHI CARD / pilot and current RAICHO");
  const heading = element("header", "meishi-heading");
  const portraitFrame = element("section", "meishi-portrait-frame");
  portraitFrame.dataset.pixelKind = "meishi-portrait-frame";
  const portrait = element("section", "meishi-portrait");
  portrait.setAttribute("role", "img");
  portrait.hidden = true;
  const identity = element("div", "meishi-identity");
  const designation = element("p", "meishi-designation");
  const manufacturer = element("p", "meishi-manufacturer");
  const title = element("h3", "meishi-title", "MEISHI CARD");
  title.id = "meishi-card-title";
  const seal = element("section", "meishi-seal");
  seal.dataset.pixelKind = "meishi-seal";
  seal.setAttribute("aria-hidden", "true");
  const issuer = element("p", "meishi-issuer");
  const masthead = element("div", "meishi-masthead");
  const brand = element("div", "meishi-brand");
  brand.append(title, issuer);
  masthead.append(seal, brand);
  identity.append(designation, manufacturer);
  portraitFrame.append(portrait);
  heading.append(portraitFrame, identity);

  const raicho = element("section", "meishi-raicho");
  raicho.setAttribute("aria-label", "Current RAICHO");
  const shipIcon = element("section", "meishi-ship-icon");
  shipIcon.setAttribute("aria-hidden", "true");
  const ship = element("div", "meishi-ship-summary");
  const shipName = element("h3", "meishi-ship-name");
  const progress = element("p", "meishi-progress");
  ship.append(shipName, progress);
  raicho.append(shipIcon, ship);

  const balances = element("section", "meishi-balances");
  balances.setAttribute("aria-label", "Retained corporate point balances");
  const points = colors.map((color) => {
    const row = element("div", `upgrade-category ${color}`);
    const caption = element("p", "meishi-points");
    row.append(caption);
    balances.append(row);
    return caption;
  });

  const collection = element("section", "meishi-patches");
  const patchCount = element("p", "meishi-patch-count");
  const patchList = element("ol", "meishi-patch-grid");
  patchList.setAttribute("aria-label", "Eight arena patch slots");
  const patches = Array.from({ length: 8 }, (_, index): PatchSlot => {
    const row = element("li", "meishi-patch-slot");
    const well = element("section", "meishi-patch-well");
    well.dataset.pixelKind = "meishi-socket";
    well.setAttribute("aria-hidden", "true");
    const icon = element("section", "meishi-patch-icon");
    icon.dataset.pixelKind = "campaign-patch";
    icon.dataset.pixelValue = String(index);
    icon.dataset.active = "false";
    icon.setAttribute("aria-hidden", "true");
    icon.hidden = true;
    const empty = element("span", "meishi-patch-empty");
    empty.setAttribute("aria-hidden", "true");
    const caption = element("p", "meishi-patch-caption");
    well.append(icon, empty);
    row.append(well, caption);
    patchList.append(row);
    return { row, well, icon, empty, caption };
  });
  collection.append(patchCount, patchList);
  card.append(masthead, heading, raicho, balances, collection);
  target.replaceChildren(card);
  const view = {
    card,
    portraitFrame,
    seal,
    issuer,
    portrait,
    designation,
    manufacturer,
    shipIcon,
    shipName,
    progress,
    points,
    patches,
    patchCount,
  };
  views.set(target, view);
  return view;
}

function number(value: number): string {
  return Number.isFinite(value) && value >= 0 ? value.toFixed(0) : "--";
}

/** Stable nodes retain scroll/focus. Native profile animation uses Game time. */
export function paintMeishiCard(target: HTMLElement, hud: MeishiHud): boolean {
  const previous = views.get(target);
  const view = previous ?? createView(target);
  let changed = !previous;
  const write = (node: HTMLElement, value: string) => {
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
  const data = (node: HTMLElement, key: string, value: string) => {
    if (node.dataset[key] !== value) {
      node.dataset[key] = value;
      changed = true;
    }
  };
  const hide = (node: HTMLElement, hidden: boolean) => {
    if (node.hidden !== hidden) {
      node.hidden = hidden;
      changed = true;
    }
  };
  const identity = hud.identity;
  const validIdentity =
    identity &&
    kinds.includes(identity.kind) &&
    corporations.includes(identity.corporation) &&
    corporations.includes(identity.sub_corporation);
  const corporation = validIdentity
    ? corporations.indexOf(identity.corporation)
    : 0;
  data(view.card, "corporation", corporations[corporation]);
  data(view.portraitFrame, "pixelValue", String(corporation));
  data(view.seal, "pixelValue", String(corporation));
  write(
    view.issuer,
    validIdentity
      ? ["PAN DYNAMICS", "RENKO CYBERNETICS", "HUGO CORP"][corporation]
      : "LEGACY RECORD",
  );
  hide(view.portrait, !validIdentity);
  if (validIdentity) {
    changed = paintLicensePortrait(view.portrait, identity) || changed;
    write(view.designation, identity.short_designation);
    write(
      view.manufacturer,
      `${identity.kind.toUpperCase()} / ${identity.corporation.toUpperCase()} / ${identity.sub_corporation.toUpperCase()}`,
    );
  } else {
    write(view.designation, "LEGACY PILOT");
    write(view.manufacturer, "NO ISSUED CORPORATE IDENTITY");
  }

  const selected = hud.campaign.selected_ship;
  const ship = hud.campaign.ships.find((entry) => entry.id === selected);
  const validShip =
    Number.isInteger(selected) && selected >= 0 && selected <= 9 && !!ship;
  const licensed = validShip && selected === 0 && validIdentity;
  hide(view.shipIcon, !validShip);
  data(
    view.shipIcon,
    "pixelKind",
    licensed ? "licensed-ship" : "campaign-ship",
  );
  data(
    view.shipIcon,
    "pixelValue",
    String(
      licensed
        ? corporations.indexOf(identity.corporation)
        : selected === 0
          ? 9
          : selected - 1,
    ),
  );
  data(view.shipIcon, "active", "true");
  write(view.shipName, validShip ? ship.name : "RAICHO UNAVAILABLE");
  write(view.progress, `LV ${number(hud.ship_level)}`);
  attribute(
    view.progress,
    "aria-label",
    `RAICHO level ${number(hud.ship_level)}`,
  );
  view.points.forEach((node, index) => {
    const issuer = ["RENKO", "HUGO", "PAN"][index];
    const role = ["OFFENSIVE", "DEFENSIVE", "PASSIVE"][index];
    const balance = number(hud.currency[index]);
    write(node, `${issuer} / ${balance}`);
    attribute(node, "aria-label", `${issuer} / ${role} POINTS ${balance}`);
  });

  let count = 0;
  view.patches.forEach((slot, index) => {
    const champion = hud.campaign.champions.find(
      (entry) => entry.index === index,
    );
    const earned = champion?.patch === true;
    count += Number(earned);
    hide(slot.icon, !earned);
    hide(slot.empty, earned);
    data(slot.icon, "active", String(earned));
    data(slot.well, "active", String(earned));
    data(slot.well, "pixelValue", String(corporation * 8 + index));
    attribute(
      slot.row,
      "aria-label",
      earned
        ? `Arena patch ${index + 1}: ${champion.name}, earned`
        : `Arena patch ${index + 1}: empty`,
    );
    write(slot.caption, String(index + 1).padStart(2, "0"));
  });
  write(view.patchCount, `ARENA PATCHES ${count}/8`);
  return changed;
}
