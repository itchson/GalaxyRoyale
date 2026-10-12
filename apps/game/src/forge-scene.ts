import {
  paintForge,
  paintShipHangar,
  type ForgeHud,
  type ForgeActions,
  type ShipHangarActions,
} from "./loadout";
import "./forge-scene.css";
type Section = "abilities" | "components" | "hulls" | "hardware";
interface View {
  section: Section;
  body: HTMLElement;
  nav: HTMLButtonElement[];
  render: () => void;
}
const views = new WeakMap<HTMLElement, View>();
export function resetForgeScene(target: HTMLElement): void {
  views.delete(target);
}

/** Scene navigation only; discovered recipes, prices and transactions stay native. */
export function paintForgeScene(
  target: HTMLElement,
  hud: ForgeHud,
  abilities: ForgeActions,
  hulls: ShipHangarActions,
  changed: () => void,
): void {
  let view = views.get(target);
  if (!view) {
    const nav = document.createElement("nav");
    nav.className = "menu-tabs forge-sections";
    nav.setAttribute("aria-label", "Forge sections");
    const body = document.createElement("section");
    body.className = "forge-workbench";
    view = { section: "abilities", body, nav: [], render: () => {} };
    const current = view;
    for (const [section, label] of [
      ["abilities", "ABILITIES"],
      ["components", "ITEMS & MATERIALS"],
      ["hulls", "RAICHO BLUEPRINTS"],
      ["hardware", "HARDWARE"],
    ] as const) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "text-button";
      button.textContent = label;
      button.addEventListener("click", () => {
        if (current.section === section) return;
        current.section = section;
        current.render();
        target.scrollTop = 0;
        changed();
      });
      nav.append(button);
      current.nav.push(button);
    }
    target.replaceChildren(nav, body);
    views.set(target, view);
  }
  const current = view;
  current.render = () => {
    const sections: Section[] = [
      "abilities",
      "components",
      "hulls",
      "hardware",
    ];
    current.nav.forEach((button, index) =>
      button.setAttribute(
        "aria-pressed",
        String(current.section === sections[index]),
      ),
    );
    if (current.section === "hulls")
      paintShipHangar(current.body, hud, { ...hulls, constructionOnly: true });
    else paintForge(current.body, hud, abilities, current.section);
  };
  current.render();
}
