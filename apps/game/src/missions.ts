import "./missions.css";
import {
  paintCampaign,
  type CampaignActions,
  type CampaignHud,
} from "./campaign";
import type { OnboardingHud } from "./kikai";
import type { Profile } from "./storage";

interface MissionsHud {
  discoveries: number;
  relics: number;
  onboarding: OnboardingHud | null;
  quest_log: Profile["quests"];
  boss_progress: { family: number; defeats: number[]; queued: number[] }[];
  campaign: CampaignHud;
}
type Section = "expeditions" | "league";
interface View {
  section: Section;
  signature: string;
  body: HTMLElement;
  buttons: HTMLButtonElement[];
  render: () => void;
}
const views = new WeakMap<HTMLElement, View>();
function text(tag: "h3" | "p" | "li", value: string): HTMLElement {
  const node = document.createElement(tag);
  node.textContent = value;
  return node;
}

/** View-only Haven scene. Rust owns discovery, mission progress and rewards. */
export function paintMissions(
  target: HTMLElement,
  hud: MissionsHud,
  sites: { id: string; name: string }[],
  actions: CampaignActions,
  contact: () => void,
  changed: () => void,
): boolean {
  let view = views.get(target);
  if (!view) {
    const nav = document.createElement("nav");
    nav.className = "menu-tabs mission-tabs";
    nav.setAttribute("aria-label", "Mission sections");
    const body = document.createElement("section");
    body.className = "mission-body";
    view = {
      section: "expeditions",
      signature: "",
      body,
      buttons: [],
      render: () => {},
    };
    const current = view;
    for (const section of ["expeditions", "league"] as const) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "text-button";
      button.textContent =
        section === "expeditions" ? "EXPEDITIONS" : "ARENA LEAGUE";
      button.addEventListener("click", () => {
        if (current.section === section) return;
        current.section = section;
        current.render();
        target.scrollTop = 0;
        changed();
      });
      nav.append(button);
      view.buttons.push(button);
    }
    target.replaceChildren(nav, body);
    views.set(target, view);
  }
  const current = view;
  const signature = () =>
    JSON.stringify(
      current.section === "league"
        ? [
            current.section,
            { ...hud.campaign, encounter: Boolean(hud.campaign.encounter) },
          ]
        : [
            current.section,
            hud.discoveries,
            hud.relics,
            hud.onboarding,
            hud.quest_log,
            hud.boss_progress,
            sites,
          ],
    );
  current.render = () => {
    const focused =
      document.activeElement instanceof HTMLElement &&
      current.body.contains(document.activeElement)
        ? document.activeElement.id
        : "";
    const scroll = target.scrollTop;
    current.signature = signature();
    for (const [index, button] of current.buttons.entries())
      button.setAttribute(
        "aria-pressed",
        String((index === 0) === (current.section === "expeditions")),
      );
    current.body.replaceChildren();
    if (current.section === "league") {
      paintCampaign(current.body, hud.campaign, actions);
      return;
    }
    current.body.append(
      text("p", `${hud.discoveries} DISCOVERIES / ${hud.relics} RELICS`),
    );
    if (hud.onboarding) {
      const training = document.createElement("section");
      training.className = "mission-training upgrade-category passive";
      const title = text("h3", `APR-1337 / ${hud.onboarding.title}`);
      title.id = "training-title";
      const description = text("p", hud.onboarding.description);
      description.id = "training-description";
      const button = document.createElement("button");
      button.className = "primary-button";
      button.id = "mission-guide-contact";
      button.textContent = "CONTACT APR-1337";
      button.addEventListener("click", contact);
      training.append(title, description, button);
      current.body.append(training);
    }
    current.body.append(text("h3", "MISSION LOG"));
    const names = new Map(sites.map((site) => [site.id, site.name]));
    const list = document.createElement("ul");
    list.className = "mission-log";
    for (const mission of hud.quest_log)
      list.append(
        text(
          "li",
          `${names.get(mission.site_id) ?? mission.site_id} / ${mission.event === "courier" ? "DELIVERY" : "FAMILY BOUNTY"} / ${mission.completed ? "REWARD CLAIMED" : `${mission.progress}/${mission.target} - return to source`}`,
        ),
      );
    if (!hud.quest_log.length)
      current.body.append(
        text(
          "p",
          "Discover sanctuary beacons beyond Haven. Accept a mission and return to its source to claim your reward.",
        ),
      );
    current.body.append(list);
    current.body.append(text("h3", "HOSTILE EVOLUTION"));
    current.body.append(
      text(
        "p",
        "24 family kills summon a boss. Two defeated bosses promote the next generation. Unfinished encounters remain queued after retreat or RAICHO loss.",
      ),
    );
    const families = document.createElement("div");
    families.className = "mission-families";
    for (const family of hud.boss_progress ?? []) {
      const group = document.createElement("section");
      group.append(
        text("h3", ["DART", "BASTION", "COIL"][family.family] ?? "HOSTILE"),
      );
      for (let generation = 0; generation < 5; generation++) {
        const queued = family.queued[generation] ?? 0;
        group.append(
          text(
            "p",
            `G${generation + 1} / ${family.defeats[generation] ?? 0} defeated${queued ? ` / ${queued} queued` : ""}`,
          ),
        );
      }
      families.append(group);
    }
    current.body.append(families);
    target.scrollTop = scroll;
    if (focused)
      current.body
        .querySelector<HTMLElement>(`#${focused}`)
        ?.focus({ preventScroll: true });
  };
  if (signature() === current.signature) return false;
  current.render();
  return true;
}
