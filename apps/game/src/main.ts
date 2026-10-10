import "./style.css";
import { FlightInput } from "./input";
import { AudioEngine } from "./audio";
import { PixelUiBridge } from "./pixel-ui";
import { DebugConsole } from "./debug-console";
import { DialoguePanel } from "./dialogue";
import { paintCampaign, paintHangar, paintCredits, type CampaignHud, type CampaignActions } from "./campaign";
import { NarrativeDirector, type NarrativeContext } from "./narrative";
import {
  SaveStore,
  SaveConflict,
  loadSettings,
  saveSettings,
  validProfile,
  type Profile,
} from "./storage";

interface Upgrade {
  id: string;
  name: string;
  category: number;
  rank: number;
  max_rank: number;
  description: string;
  unlocked: boolean;
  required_level: number;
  evolution: number;
}
interface Hud {
  position: [number, number];
  camera: [number, number];
  heading: number;
  hull: number;
  max_hull: number;
  shield: number;
  max_shield: number;
  zone: string;
  danger: number;
  safe: boolean;
  dock: boolean;
  currency: number[];
  kills: number[];
  boss: { name: string; health: number; max_health: number; family: number; generation: number; phase: string; telegraph: boolean } | null;
  boss_progress: { family: number; defeats: number[]; queued: number[] }[];
  upgrades: Upgrade[];
  time: number;
  shake: number;
  entities: number;
  particles: number;
  biome: { name: string; kind: number };
  site: {
    id: string;
    name: string;
    event: string;
    description: string;
    status: string;
    can_interact: boolean;
  } | null;
  quest_log: Profile["quests"];
  discoveries: number;
  relics: number;
  campaign: CampaignHud;
  ship_level: number;
  ship_xp: number;
  ship_next_xp: number;
  rank_cap: number;
  cargo_materials: number;
  cargo_capacity: number;
  stored_materials: number;
  last_run: { time: number; distance_peak: number; enemies: number; asteroids: number; champions: number; points_gained: number[]; xp_gained: number; materials_gained: number } | null;
}
interface Cue {
  kind: string;
  x: number;
  y: number;
  intensity: number;
}
interface Step {
  hud: Hud;
  events: Cue[];
  profile: Profile | null;
  pixels: number;
  simulation_ms: number;
  render_ms: number;
}
interface GalaxyRuntime {
  step(milliseconds: number, input: unknown): Step;
  hud(): Hud;
  profile(): Profile;
  purchase(id: string): void;
  resize(width: number, height: number): void;
  viewport(): Float32Array;
  world_viewport(): Float32Array;
  set_zoom(value: number): void;
  zoom(): number;
  set_paused(value: boolean): void;
  set_shake(value: boolean): void;
  set_ui_mode(mode: number): void;
  set_touch_ui(value: boolean): void;
  set_ui_scene(scene: unknown): void;
  set_reduced_motion(value: boolean): void;
  set_ui_insets(top: number, right: number, bottom: number, left: number): void;
  set_cosmetic_density(value: boolean): void;
  interact_site(): void;
  start_champion(index: number): void;
  start_tournament(): void;
  craft_ship(index: number): void;
  switch_ship(id: number): void;
  acknowledge_credits(): void;
  return_to_home(): void;
  diagnostic(command: string): void;
  boss_ui_bounds(): Float32Array;
  recover(): Promise<void>;
  lose_device(): void;
  free(): void;
}
const element = <T extends HTMLElement>(id: string) =>
  document.getElementById(id) as T;
let canvas = element<HTMLCanvasElement>("universe");
const store = new SaveStore();
const audio = new AudioEngine();
const settings = loadSettings();
document.body.classList.toggle("reduced-motion", settings.reduced_motion);
audio.setLevels(settings.master, settings.music, settings.sfx);
const narrative = new NarrativeDirector({ voiceEnabled: settings.dialogue_voice, voiceVolume: settings.master * settings.voice_volume });
const launch = element<HTMLButtonElement>("launch");
launch.disabled = true;
const loadStatus = element("load-status");
const menu = element("menu-screen");
const launchScreen = element("launch-screen");
const deathScreen = element("death-screen");
const params = new URLSearchParams(location.search);
let waitingUpdate: ServiceWorker | null = null;
let applyingUpdate = false;
const diagnosticMode = params.has("qa");
const touchUi =
  matchMedia("(pointer:coarse)").matches ||
  (diagnosticMode && params.has("touch"));
document.body.classList.toggle("touch-ui", touchUi);
let galaxy: GalaxyRuntime;
let input: FlightInput;
let pixelUi: PixelUiBridge;
let consoleWasRunning = false;
let dialogueWasRunning = false;
let beforeLaunchMenu = false;
type MenuTab = "upgrades" | "settings" | "journal" | "campaign" | "hangar" | "credits";
let menuTab: MenuTab = "upgrades";
let creditsPresented = false;
let transitionEpoch = 0;
let preparingEncounter: number | null = null;
let hud: Hud;
let launched = false;
let paused = true;
let recovering = false;
let graphicsFailed = false;
let lastTime = 0;
let previousDock = true;
let destination: [number, number] | null = null;
let toastTimeout: number | undefined;
let backend = "";
const safeAreaProbe = document.createElement("div");
safeAreaProbe.className = "safe-area-probe";
document.body.append(safeAreaProbe);
let frames = 0;
let fps = 0;
let fpsTime = 0;
let lastHudPaint = 0;
let lastPerformancePaint = 0;
const frameTimes: number[] = [];
const simulationTimes: number[] = [];
const renderTimes: number[] = [];
let lastPixels = 0;
let captureStarted = 0;
let scenario = params.get("scene") ?? "dock";
let minPixels = Infinity;
let maxPixels = 0;
const debugConsole = new DebugConsole({
  ready: () => Boolean(galaxy),
  onOpen: () => {
    dialoguePanel.close();
    element("quick-menu").hidden = true;
    consoleWasRunning = launched && !paused;
    paused = true;
    input.enabled = false;
    input.reset();
    galaxy.set_paused(true);
    galaxy.set_ui_mode(0);
    document.body.classList.remove("engine-flight");
    audio.suspend();
    pixelUi?.invalidate();
  },
  onClose: () => {
    if (
      consoleWasRunning &&
      launched && menu.hidden && deathScreen.hidden && preparingEncounter === null &&
      !dialoguePanel.opened &&
      !recovering &&
      !graphicsFailed &&
      !document.hidden
    ) {
      paused = false;
      input.enabled = true;
      galaxy.set_paused(false);
      lastTime = 0;
      void audio.resume();
    }
    if (!dialoguePanel.opened) restoreUi();
    pixelUi?.invalidate();
  },
  run: async (command) => {
    const [verb, ...args] = command.trim().toLowerCase().split(/\s+/);
    if (verb === "help")
      return "HELP / STATS / PROFILE / RENDERER / RECOVER\nSCENE dock|belt|combat|site|biome-riveter|boss-dart[-1..5]|boss-bastion[-1..5]|boss-coil[-1..5]|champion-0..7|tournament|stress|die (QA mode)\nCLEAR / Up, Down: history / Backtick or Escape: close";
    if (verb === "stats") {
      const panel = Array.from(galaxy.boss_ui_bounds(), value => value.toFixed(0));
      return `${backend.toUpperCase()} / ${fps} FPS / p95 ${percentile(frameTimes, 0.95).toFixed(2)} ms\n${lastPixels} quads / ${hud.entities} entities / ${hud.particles} particles\nPOSITION ${hud.position.map((v) => v.toFixed(1)).join(", ")}\n${hud.biome.name} / ${hud.zone} / danger ${hud.danger}\nHULL ${hud.hull.toFixed(0)}/${hud.max_hull.toFixed(0)} / SHIELD ${hud.shield.toFixed(0)}\nZOOM ${galaxy.zoom().toFixed(2)} / WORLD ${Array.from(galaxy.world_viewport(), v => v.toFixed(0)).join(" x ")} / UI ${galaxy.viewport().join(" x ")}\nSHIP LV ${hud.ship_level} / XP ${hud.ship_xp} / CARGO ${hud.cargo_materials}/${hud.cargo_capacity}\nLAST BOSS UI ${panel.length ? panel.join(" / ") : "NONE"}`;
    }
    if (verb === "profile") {
      const p = galaxy.profile();
      return `SCHEMA ${p.schema_version} / revision ${p.revision} / seed ${p.seed}\nCURRENCY ${p.currency.join(" / ")}\nBOSS GENERATIONS ${p.boss_generations?.map(row => row.join(",")).join(" / ") ?? "legacy"}\nDISCOVERIES ${p.discoveries.length} / quests ${p.quests.length} / relics ${p.relics.length}\nPATCHES ${p.campaign.patches.filter(Boolean).length}/8 / ship ${p.campaign.selected_ship} / Royale ${p.campaign.victory_claimed ? "WON" : "PENDING"}\n${store.message}`;
    }
    if (verb === "renderer")
      return `${backend.toUpperCase()} / ${galaxy.viewport().join(" x ")} logical pixels / eight bounded lights`;
    if (verb === "recover")
      return (await recover("console recovery request"))
        ? "Graphics restored; expedition retained."
        : "Recovery unavailable or failed; see graphics status.";
    if (verb === "scene") {
      if (!diagnosticMode)
        return "Validation scenes require ?qa=1. No progression is awarded by scene setup.";
      const name = args[0];
      if (
        ![
          "dock",
          "home",
          "belt",
          "combat",
          "site",
          "biome-riveter",
          "boss-dart",
          "boss-bastion",
          "boss-coil",
          ...["dart", "bastion", "coil"].flatMap(family =>
            [1, 2, 3, 4, 5].map(generation => `boss-${family}-${generation}`)),
          "stress",
          "die",
          ...Array.from({length: 8}, (_, i) => `champion-${i}`),
          "tournament",
        ].includes(name)
      )
        return "Unknown scene. Type HELP for supported names.";
      galaxy.diagnostic(name);
      hud = galaxy.hud();
      scenario = name;
      destination = null;
      previousDock = true;
      frameTimes.length = 0;
      simulationTimes.length = 0;
      renderTimes.length = 0;
      minPixels = Infinity;
      maxPixels = 0;
      captureStarted = 0;
      paintHud();
      return `SCENE ${name.toUpperCase()} / ready. Close console to simulate.`;
    }
    return "Unknown command. Type HELP.";
  },
});
const dialoguePanel = new DialoguePanel({
  onOpen: () => {
    element("quick-menu").hidden = true;
    dialogueWasRunning = launched && !paused;
    paused = true;
    input.enabled = false;
    input.reset();
    galaxy.set_paused(true);
    galaxy.set_ui_mode(0);
    document.body.classList.remove("engine-flight");
    audio.suspend();
    pixelUi?.invalidate();
  },
  onClose: () => {
    narrative.close();
    if (
      dialogueWasRunning &&
      launched && menu.hidden && deathScreen.hidden && preparingEncounter === null &&
      !debugConsole.opened &&
      !recovering &&
      !graphicsFailed &&
      !document.hidden
    ) {
      paused = false;
      input.enabled = true;
      galaxy.set_paused(false);
      lastTime = 0;
      void audio.resume();
    }
    if (!debugConsole.opened) restoreUi();
    pixelUi?.invalidate();
  },
});
function restoreUi(): void {
  galaxy.set_ui_mode(
    !deathScreen.hidden ? 0 : launched ? (menu.hidden ? 1 : 3) : beforeLaunchMenu ? 3 : 2,
  );
  document.body.classList.toggle("engine-flight", launched && menu.hidden && deathScreen.hidden);
}
const percentile = (values: number[], p: number): number => {
  const sorted = [...values].sort((a, b) => a - b);
  return (
    sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))] ?? 0
  );
};
const safePercent = (amount: number, max: number) =>
  `${max > 0 ? Math.max(0, Math.min(100, (amount / max) * 100)) : 0}%`;
function toast(message: string): void {
  const target = element("toast");
  target.textContent = message;
  target.classList.add("visible");
  clearTimeout(toastTimeout);
  toastTimeout = window.setTimeout(
    () => target.classList.remove("visible"),
    3500,
  );
}
function save(profile: Profile): void {
  void store.save(profile).catch((error) => {
    if (error instanceof SaveConflict && !paused) pause(false);
    toast(store.message);
  });
}
function resize(): void {
  if (!galaxy) return;
  const ratio = Math.min(devicePixelRatio || 1, 3);
  galaxy.resize(
    Math.round(innerWidth * ratio),
    Math.round(innerHeight * ratio),
  );
  if (input) {
    const v = galaxy.viewport();
    const world = galaxy.world_viewport();
    input.viewport = [world[0], world[1]];
    document.documentElement.style.setProperty(
      "--ui-scale",
      String(innerHeight / v[1]),
    );
    const safe = getComputedStyle(safeAreaProbe);
    const uiScale = innerHeight / v[1];
    document.body.classList.toggle("narrow-ui", v[0] < 430);
    galaxy.set_ui_insets(
      ...([
        safe.paddingTop,
        safe.paddingRight,
        safe.paddingBottom,
        safe.paddingLeft,
      ].map((value) => Number.parseFloat(value) / uiScale) as [
        number,
        number,
        number,
        number,
      ]),
    );
    input.reset();
    pixelUi?.invalidate();
  }
  if (
    matchMedia("(pointer:coarse) and (orientation:portrait)").matches &&
    launched &&
    !paused
  )
    pause(false);
}
function pause(dock: boolean): void {
  if (!galaxy || !launched) return;
  transitionEpoch++;
  paused = true;
  input.enabled = false;
  input.reset();
  galaxy.set_paused(true);
  galaxy.set_ui_mode(3);
  document.body.classList.remove("engine-flight");
  audio.suspend();
  destination = null;
  element("quick-menu").hidden = true;
  deathScreen.hidden = true;
  menu.hidden = false;
  element("menu-title").textContent = dock
    ? "Evolve your ship"
    : "Flight paused";
  element("menu-description").textContent = dock
    ? "Spend your collected points. Every rank costs 10 matching points."
    : "Collected upgrades are permanent. Return to Haven to evolve your ship.";
  hud = galaxy.hud();
  paintUpgrades(dock);
  element("apply-update").hidden = !waitingUpdate;
  element<HTMLButtonElement>("apply-update").disabled = !hud.dock;
  element("save-status").textContent = store.message;
  save(galaxy.profile());
  beforeLaunchMenu = false;
  selectMenuTab(dock ? "upgrades" : "settings");
}
const campaignActions: CampaignActions = {
  navigate: (position) => { void navigate(position); },
  challenge: (index) => { void beginChampion(index).catch(error => toast(String(error))); },
  tournament: () => { void beginTournament().catch(error => toast(String(error))); },
  craft: (index) => {
    try {
      galaxy.craft_ship(index);
      hud = galaxy.hud();
      save(galaxy.profile());
      paintHangar(element("hangar-content"), hud.campaign, hud.dock, hud.currency, campaignActions, hud.stored_materials);
      paintHud();
      toast("SHIP CRAFTED / select it at Haven");
    } catch (error) { toast(String(error)); }
  },
  select: (id) => {
    try {
      galaxy.switch_ship(id);
      hud = galaxy.hud();
      save(galaxy.profile());
      paintHangar(element("hangar-content"), hud.campaign, hud.dock, hud.currency, campaignActions, hud.stored_materials);
      paintUpgrades(true);
      paintHud();
      toast("SHIP SELECTED");
    } catch (error) { toast(String(error)); }
  },
  credits: openCredits,
};
async function navigate(position: [number, number]): Promise<void> {
  await resume();
  if (paused) return;
  input.setCourse(position);
  toast("COURSE SET / fly or steer to your destination");
}
async function beginEncounter(start: () => void): Promise<void> {
  const epoch = ++transitionEpoch;
  preparingEncounter = epoch;
  try {
    start();
    hud = galaxy.hud();
    await store.save(galaxy.profile());
    await store.flush();
    if (epoch !== transitionEpoch || !launched || !deathScreen.hidden) return;
    menu.hidden = true;
    preparingEncounter = null;
    dialoguePanel.close();
    input.reset();
    destination = null;
    previousDock = hud.dock;
    paintHud();
    await resume();
  } finally {
    if (preparingEncounter === epoch) preparingEncounter = null;
  }
}
function beginChampion(index: number): Promise<void> {
  return beginEncounter(() => galaxy.start_champion(index));
}
function beginTournament(): Promise<void> {
  return beginEncounter(() => galaxy.start_tournament());
}
function openCredits(): void {
  dialoguePanel.close();
  pause(false);
  element("menu-title").textContent = "GALAXY ROYALE WON";
  element("menu-description").textContent = "Your Sovereign ship is ready. Thank you for flying.";
  selectMenuTab("credits");
}
function finishCredits(): void {
  galaxy.acknowledge_credits();
  save(galaxy.profile());
  hud = galaxy.hud();
  element("menu-title").textContent = "HANGAR";
  element("menu-description").textContent = hud.dock ? "Your collection awaits." : "Return to Haven to select your Sovereign.";
  selectMenuTab("hangar");
}
function selectMenuTab(tab: MenuTab): void {
  menuTab = tab;
  const headings: Record<MenuTab, [string, string]> = {
    upgrades: [hud.dock ? "EVOLVE YOUR SHIP" : "SHIP UPGRADES", "Each rank costs 10 matching points and requires its ship level."],
    hangar: ["SHIP HANGAR", hud.dock ? "Craft blueprints and choose your next ship." : "Return to Haven to craft or switch ships."],
    campaign: ["ARENA CHAMPIONS", "Eight patches reveal the Galaxy Royale tournament."],
    settings: ["SETTINGS", "Tune your flight. Settings stay on this device."],
    journal: ["EXPEDITION JOURNAL", "Discover sanctuaries and return to claim their rewards."],
    credits: ["GALAXY ROYALE WON", "Your Sovereign ship is ready. Thank you for flying."],
  };
  element("menu-title").textContent = headings[tab][0];
  element("menu-description").textContent = headings[tab][1];
  element("export-save").hidden = tab !== "settings";
  element("import-label").hidden = tab !== "settings";
  element("menu-fullscreen").hidden = tab !== "settings";
  element("return-title").hidden = tab !== "settings" || beforeLaunchMenu;
  element("upgrade-content").hidden = tab !== "upgrades";
  element("settings-content").hidden = tab !== "settings";
  element("quest-content").hidden = tab !== "journal";
  element("hangar-content").hidden = tab !== "hangar";
  element("campaign-content").hidden = tab !== "campaign" && tab !== "credits";
  for (const page of ["upgrades", "settings", "journal", "hangar", "campaign"])
    element(`tab-${page}`).dataset.active = String(page === tab);
  if (tab === "campaign") paintCampaign(element("campaign-content"), hud.campaign, campaignActions);
  if (tab === "hangar") paintHangar(element("hangar-content"), hud.campaign, hud.dock, hud.currency, campaignActions, hud.stored_materials);
  if (tab === "credits") paintCredits(element("campaign-content"), { ...campaignActions, credits: finishCredits });
  if (tab === "journal") {
    const target = element("quest-content");
    target.replaceChildren();
    const heading = document.createElement("p");
    heading.textContent = `${hud.discoveries} discoveries / ${hud.relics} relics. Each relic adds 1 hull, up to 20.`;
    target.append(heading);
    const list = document.createElement("ul");
    for (const quest of hud.quest_log) {
      const item = document.createElement("li");
      item.textContent = `${quest.site_id} / ${quest.event.toUpperCase()} / ${quest.completed ? "REWARD CLAIMED" : `${quest.progress}/${quest.target} - return to source`}`;
      list.append(item);
    }
    if (!hud.quest_log.length) {
      const hint = document.createElement("p");
      hint.textContent =
        "Seek sanctuary beacons beyond Haven. Accept a courier delivery or a family bounty, then return for a unique relic.";
      target.append(hint);
    }
    target.append(list);
    const bossHeading = document.createElement("h3");
    bossHeading.textContent = "HOSTILE EVOLUTION";
    const bossHint = document.createElement("p");
    bossHint.textContent = "24 family kills summon the first boss. Two defeated bosses summon their next generation. Unfinished queues survive retreat and ship loss.";
    target.append(bossHeading, bossHint);
    for (const family of hud.boss_progress ?? []) {
      const familyHeading = document.createElement("h3");
      familyHeading.textContent = ["DART", "BASTION", "COIL"][family.family] ?? "HOSTILE";
      const generations = document.createElement("ul");
      for (let generation = 0; generation < 5; generation++) {
        const item = document.createElement("li");
        const queued = family.queued[generation] ?? 0;
        item.textContent = `G${generation + 1} / ${family.defeats[generation] ?? 0} defeated${queued > 0 ? ` / ${queued} queued` : ""}`;
        generations.append(item);
      }
      target.append(familyHeading, generations);
    }
  }
  if (tab === "upgrades" && !hud.dock) {
    const hint = document.createElement("p");
    hint.textContent = "Return to Haven dock to spend your upgrade points.";
    element("upgrade-content").replaceChildren(hint);
  }
  pixelUi?.invalidate();
}
function openLaunchSettings(): void {
  if (!galaxy) return;
  beforeLaunchMenu = true;
  launchScreen.hidden = true;
  menu.hidden = false;
  galaxy.set_ui_mode(3);
  element("menu-title").textContent = "SETTINGS";
  element("menu-description").textContent =
    "Tune your expedition. Settings stay on this device.";
  element("resume").textContent = "RETURN TO TITLE";
  hud = galaxy.hud();
  selectMenuTab("settings");
}
async function resume(): Promise<void> {
  if (menuTab === "credits" && hud?.campaign.credits_pending) {
    galaxy.acknowledge_credits();
    save(galaxy.profile());
    hud = galaxy.hud();
  }
  if (beforeLaunchMenu) {
    beforeLaunchMenu = false;
    menu.hidden = true;
    launchScreen.hidden = false;
    element("resume").textContent = "RETURN TO FLIGHT";
    galaxy.set_ui_mode(2);
    pixelUi?.invalidate();
    return;
  }
  if (
    !galaxy ||
    preparingEncounter !== null ||
    recovering ||
    graphicsFailed ||
    debugConsole.opened ||
    dialoguePanel.opened ||
    document.hidden ||
    matchMedia("(pointer:coarse) and (orientation:portrait)").matches
  )
    return;
  menu.hidden = true;
  deathScreen.hidden = true;
  launchScreen.hidden = true;
  launched = true;
  paused = false;
  lastTime = 0;
  input.enabled = true;
  galaxy.set_paused(false);
  galaxy.set_ui_mode(1);
  document.body.classList.add("engine-flight");
  await audio.resume();
  if (
    paused ||
    debugConsole.opened ||
    dialoguePanel.opened ||
    document.hidden
  ) {
    paused = true;
    input.enabled = false;
    input.reset();
    galaxy.set_paused(true);
    audio.suspend();
  }
}
function paintUpgrades(dock: boolean): void {
  const target = element("upgrade-content");
  target.replaceChildren();
  if (!dock) return;
  const level = document.createElement("p");
  const progress = hud.ship_level >= 100
    ? "MAX LEVEL"
    : `NEXT LEVEL ${hud.ship_next_xp} XP`;
  level.textContent = `SHIP LEVEL ${hud.ship_level} / ${hud.ship_xp} XP / ${progress}. Rank cap ${hud.rank_cap}/5.`;
  target.append(level);
  const columns = document.createElement("div");
  columns.className = "upgrade-columns";
  ["OFFENSIVE", "DEFENSIVE", "PASSIVE"].forEach((name, category) => {
    const column = document.createElement("div");
    const title = document.createElement("div");
    title.className = `upgrade-category ${["offense", "defense", "passive"][category]}`;
    title.innerHTML = `<span>${name}</span><b>${hud.currency[category]} PTS</b>`;
    column.append(title);
    for (const upgrade of hud.upgrades.filter((u) => u.category === category)) {
      const card = document.createElement("article");
      card.className = "upgrade-card";
      const heading = document.createElement("h3");
      heading.textContent = `${upgrade.name}${upgrade.evolution > 0 ? ` / E${upgrade.evolution}` : ""}`;
      const description = document.createElement("p");
      description.textContent = upgrade.description;
      const ranks = document.createElement("div");
      ranks.className = "rank-dots";
      for (let i = 0; i < upgrade.max_rank; i++) {
        const dot = document.createElement("i");
        if (i < upgrade.rank) dot.className = "earned";
        ranks.append(dot);
      }
      const buy = document.createElement("button");
      buy.disabled =
        !upgrade.unlocked ||
        upgrade.rank >= upgrade.max_rank ||
        hud.currency[category] < 10 ||
        hud.ship_level < upgrade.required_level;
      buy.textContent = !upgrade.unlocked
        ? "BOSS SIGNATURE REQUIRED"
        : upgrade.rank >= upgrade.max_rank
          ? "FULLY EVOLVED"
          : hud.ship_level < upgrade.required_level
            ? `SHIP LEVEL ${upgrade.required_level} REQUIRED`
          : "UPGRADE / 10 PTS";
      buy.addEventListener("click", async () => {
        try {
          galaxy.purchase(upgrade.id);
          hud = galaxy.hud();
          save(galaxy.profile());
          await audio.resume();
          audio.play({ kind: "upgrade", x: 0, y: 0, intensity: 1 });
          paintUpgrades(true);
          paintHud();
        } catch (error) {
          toast(String(error));
        }
      });
      card.append(heading, description, ranks, buy);
      column.append(card);
    }
    columns.append(column);
  });
  target.append(columns);
}
function paintHud(): void {
  if (!hud) return;
  element("sector").textContent =
    `${hud.biome.name.toUpperCase()} / ${hud.safe ? "PROTECTED SPACE" : `DANGER ${hud.danger}`}`;
  for (let i = 0; i < 3; i++)
    element(`currency-${i}`).textContent = String(hud.currency[i]);
  element("hull-number").textContent =
    `${Math.ceil(hud.hull)} / ${Math.ceil(hud.max_hull)}`;
  element("hull-fill").style.width = safePercent(hud.hull, hud.max_hull);
  element("shield-fill").style.width = safePercent(hud.shield, hud.max_shield);
  const distance = Math.hypot(...hud.position);
  element("distance").textContent =
    `${Math.round(distance).toLocaleString()} m FROM HAVEN`;
  element("danger").textContent = hud.safe
    ? "SANCTUARY"
    : hud.danger === 0
      ? "ASTEROID FRONTIER"
      : `THREAT LEVEL ${hud.danger}`;
  element("dock-button").hidden = !hud.dock || paused || !launched;
  const siteButton = element<HTMLButtonElement>("site-button");
  const arena = hud.campaign.arena;
  const gate = hud.campaign.tournament_unlocked && hud.campaign.tournament_nearby;
  siteButton.hidden = (!hud.site && !arena && !gate) || paused || !launched || !!hud.campaign.encounter;
  if (arena || gate) {
    siteButton.textContent = arena ? `${arena.name} / CHAMPION [E]` : "GALAXY ROYALE / ENTER [E]";
    siteButton.disabled = false;
    siteButton.title = arena ? "Contact this arena champion." : "Enter the round robin tournament.";
  } else if (hud.site) {
    siteButton.textContent = `${hud.site.name} / CONTACT [E]`;
    siteButton.disabled = false;
    siteButton.title = hud.site.description;
  }
  element("boss-hud").hidden = !hud.boss;
  if (hud.boss) {
    element("boss-name").textContent = `${hud.boss.generation > 0 ? `G${hud.boss.generation} / ` : ""}${hud.boss.name}`;
    element("boss-phase").textContent = `${hud.boss.telegraph ? "WARNING / " : ""}${hud.boss.phase || "HOSTILE SIGNATURE"}`;
    element("boss-health").style.width = safePercent(
      hud.boss.health,
      hud.boss.max_health,
    );
  }
  const radar = element<HTMLCanvasElement>("radar").getContext("2d");
  if (radar) {
    radar.clearRect(0, 0, 112, 112);
    radar.strokeStyle = "#4c8ba54a";
    radar.lineWidth = 1;
    for (const radius of [20, 38, 52]) {
      radar.beginPath();
      radar.arc(56, 56, radius, 0, Math.PI * 2);
      radar.stroke();
    }
    radar.beginPath();
    radar.moveTo(4, 56);
    radar.lineTo(108, 56);
    radar.moveTo(56, 4);
    radar.lineTo(56, 108);
    radar.stroke();
    radar.fillStyle = "#7df2ff";
    radar.fillRect(54, 54, 4, 4);
    if (distance > 40) {
      const dx = -hud.position[0] / distance,
        dy = -hud.position[1] / distance;
      radar.fillStyle = "#ffd783";
      radar.beginPath();
      radar.arc(56 + dx * 43, 56 + dy * 43, 3, 0, Math.PI * 2);
      radar.fill();
    }
    radar.strokeStyle = "#75efff";
    radar.beginPath();
    radar.moveTo(56, 56);
    radar.lineTo(
      56 + Math.cos(hud.heading) * 13,
      56 + Math.sin(hud.heading) * 13,
    );
    radar.stroke();
  }
}
function paintDestination(): void {
  const marker = element("destination-marker");
  if (destination) pixelUi?.invalidate();
  if (!destination || paused) {
    marker.hidden = true;
    return;
  }
  if (
    Math.hypot(
      destination[0] - hud.position[0],
      destination[1] - hud.position[1],
    ) < 8
  ) {
    destination = null;
    marker.hidden = true;
    return;
  }
  const v = input.viewport;
  const x =
      innerWidth / 2 + ((destination[0] - hud.camera[0]) * innerWidth) / v[0],
    y =
      innerHeight / 2 + ((destination[1] - hud.camera[1]) * innerHeight) / v[1];
  marker.hidden = x < 0 || y < 0 || x > innerWidth || y > innerHeight;
  marker.style.left = `${x}px`;
  marker.style.top = `${y}px`;
}
async function recover(error: unknown): Promise<boolean> {
  if (recovering) return false;
  recovering = true;
  input.reset();
  input.enabled = false;
  galaxy.set_paused(true);
  toast("Reconnecting the graphics engine…");
  try {
    await galaxy.recover();
    graphicsFailed = false;
    document.body.classList.add("pixel-ready");
    resize();
    const canRun =
      !paused &&
      !debugConsole.opened &&
      !dialoguePanel.opened &&
      !document.hidden;
    paused = !canRun;
    galaxy.set_paused(paused);
    input.enabled = canRun;
    toast("Graphics restored. Your expedition is intact.");
    return true;
  } catch {
    graphicsFailed = true;
    document.body.classList.remove("pixel-ready");
    pause(false);
    toast(
      `Graphics recovery failed: ${String(error)}. Reload to return safely to Haven.`,
    );
    return false;
  } finally {
    recovering = false;
    lastTime = 0;
  }
}
function frame(time: number): void {
  requestAnimationFrame(frame);
  if (!galaxy || document.hidden || recovering || graphicsFailed) return;
  const rawDt = lastTime ? time - lastTime : 0;
  const dt = Math.min(100, rawDt);
  lastTime = time;
  try {
    pixelUi?.sync();
    const controls = input.sample();
    if (launched && !paused && hud.dock && controls.fire) {
      pause(true);
      return;
    }
    const result = galaxy.step(dt, controls);
    hud = result.hud;
    input.camera = hud.camera;
    const worldViewport = galaxy.world_viewport();
    input.viewport = [worldViewport[0], worldViewport[1]];
    if (result.profile) save(result.profile);
    for (const event of result.events) {
      audio.play(event);
      if (event.kind === "death") {
        showDeath();
      }
      if (event.kind === "champion-win") toast("ARENA PATCH + BLUEPRINT ACQUIRED");
      if (event.kind === "tournament-win") toast("ROYALE DUEL WON / standings updated");
      if (event.kind === "tournament-loss") toast("ROYALE DUEL LOST / standings updated");
      if (event.kind === "pickup" && event.intensity >= 2)
        toast("NEW SIGNATURE ABILITY / evolve at Haven");
    }
    audio.setDanger(hud.danger, hud.safe);
    previousDock = hud.dock;
    if (launched && deathScreen.hidden && preparingEncounter === null && hud.campaign.credits_pending && !creditsPresented && !debugConsole.opened && !dialoguePanel.opened) {
      creditsPresented = true;
      openCredits();
    }
    if (time - lastHudPaint > 80) {
      paintHud();
      lastHudPaint = time;
    }
    paintDestination();
    frames++;
    if (time - fpsTime > 1000) {
      fps = Math.round((frames * 1000) / (time - fpsTime));
      frames = 0;
      fpsTime = time;
    }
    if (diagnosticMode && rawDt > 0 && launched && !paused) {
      if (!captureStarted) captureStarted = time;
      frameTimes.push(rawDt);
      simulationTimes.push(result.simulation_ms);
      renderTimes.push(result.render_ms);
      if (frameTimes.length > 3600) {
        frameTimes.shift();
        simulationTimes.shift();
        renderTimes.shift();
      }
      lastPixels = result.pixels;
      minPixels = Math.min(minPixels, result.pixels);
      maxPixels = Math.max(maxPixels, result.pixels);
      if (time - lastPerformancePaint > 500) {
        lastPerformancePaint = time;
        element("performance").textContent =
          `${backend.toUpperCase()}  ${fps} FPS\n${result.pixels.toLocaleString()} PIXEL QUADS\n${hud.entities} ENTITIES / ${hud.particles} PARTICLES\np95 ${percentile(frameTimes, 0.95).toFixed(1)} ms\nSIM ${result.simulation_ms.toFixed(2)} / RENDER ${result.render_ms.toFixed(2)} ms`;
        element("performance").dataset.report = JSON.stringify({
          backend,
          scenario,
          time: hud.time,
          paused,
          position: hud.position,
          durationSeconds: captureStarted ? (time - captureStarted) / 1000 : 0,
          samples: frameTimes.length,
          quads: result.pixels,
          quadsRange: [minPixels, maxPixels],
          entities: hud.entities,
          particles: hud.particles,
          frameMs: {
            p50: percentile(frameTimes, 0.5),
            p95: percentile(frameTimes, 0.95),
            p99: percentile(frameTimes, 0.99),
          },
          simulationMs: {
            p50: percentile(simulationTimes, 0.5),
            p95: percentile(simulationTimes, 0.95),
          },
          renderMs: {
            p50: percentile(renderTimes, 0.5),
            p95: percentile(renderTimes, 0.95),
          },
          viewport: [innerWidth, innerHeight],
          memory: (() => {
            const memory = (
              performance as Performance & {
                memory?: {
                  usedJSHeapSize: number;
                  totalJSHeapSize: number;
                  jsHeapSizeLimit: number;
                };
              }
            ).memory;
            return memory
              ? {
                  usedJSHeapSize: memory.usedJSHeapSize,
                  totalJSHeapSize: memory.totalJSHeapSize,
                  jsHeapSizeLimit: memory.jsHeapSizeLimit,
                }
              : null;
          })(),
        });
      }
    }
  } catch (error) {
    console.error(error);
    void recover(error);
  }
}
async function init(): Promise<void> {
  const profile = await store.open();
  const forceWebgl = params.get("renderer") === "webgl2";
  const gpuAvailable = "gpu" in navigator;
  const candidates =
    forceWebgl || !gpuAvailable ? ["webgl2"] : ["webgpu", "webgl2"];
  const errors: string[] = [];
  for (const candidate of candidates) {
    try {
      loadStatus.textContent = `LOADING ${candidate.toUpperCase()} PIXEL ENGINE`;
      if (diagnosticMode && params.has("fail-webgpu") && candidate === "webgpu")
        throw new Error("QA-requested WebGPU initialization failure");
      const module = await import(
        /* @vite-ignore */ `/wasm/${candidate}/galaxy.js`
      );
      await module.default();
      galaxy = await module.Galaxy.create(canvas, profile);
      backend = candidate;
      break;
    } catch (error) {
      errors.push(`${candidate}: ${String(error)}`);
      console.warn(`Renderer ${candidate} initialization failed`, error);
      const replacement = canvas.cloneNode(false) as HTMLCanvasElement;
      canvas.replaceWith(replacement);
      canvas = replacement;
    }
  }
  if (!galaxy) {
    loadStatus.textContent = "GRAPHICS INITIALIZATION FAILED";
    throw new Error(errors.join("\n"));
  }
  input = new FlightInput(
    canvas,
    element("joystick"),
    element("joystick-stick"),
    element("fire-button"),
    (world) => {
      destination = world;
    },
    () => {
      if (launched && !paused && hud?.dock) pause(true);
    },
  );
  canvas.addEventListener("wheel", onFlightWheel, {passive:false});
  galaxy.set_paused(true);
  galaxy.set_shake(settings.shake);
  galaxy.set_reduced_motion(settings.reduced_motion);
  galaxy.set_cosmetic_density(settings.cosmetic_density === "reduced");
  galaxy.set_touch_ui(touchUi);
  galaxy.set_zoom(settings.zoom);
  resize();
  pixelUi = new PixelUiBridge(galaxy, () => innerHeight / galaxy.viewport()[1]);
  document.body.classList.add("pixel-ready");
  hud = galaxy.hud();
  paintHud();
  loadStatus.textContent = "SYSTEMS ONLINE / READY TO FLY";
  launch.disabled = false;
  element("save-status").textContent = store.message;
  if (diagnosticMode) {
    element("performance").hidden = !params.has("metrics");
    const scene = params.get("scene");
    if (scene) galaxy.diagnostic(scene);
  }
  if (diagnosticMode) {
    const controls = document.createElement("aside");
    controls.className = "qa-controls";
    controls.hidden = !params.has("toolbar");
    controls.setAttribute("aria-label", "Validation scenes");
    for (const scene of [
      "dock",
      "belt",
      "site",
      "combat",
      "boss-dart",
      "boss-bastion",
      "boss-coil",
      "stress",
      "die",
    ]) {
      const button = document.createElement("button");
      button.textContent = scene;
      button.addEventListener("click", () => {
        scenario = scene;
        minPixels = Infinity;
        maxPixels = 0;
        galaxy.diagnostic(scene);
        frameTimes.length = 0;
        simulationTimes.length = 0;
        renderTimes.length = 0;
        captureStarted = 0;
        destination = null;
        previousDock = true;
        input.reset();
      });
      controls.append(button);
    }
    const recoverButton = document.createElement("button");
    recoverButton.textContent = "Recover GPU";
    recoverButton.addEventListener(
      "click",
      () => void recover("diagnostic recovery request"),
    );
    controls.append(recoverButton);
    const loseButton = document.createElement("button");
    loseButton.textContent = "Lose GPU";
    loseButton.addEventListener("click", () => galaxy.lose_device());
    controls.append(loseButton);
    document.body.append(controls);
    (window as unknown as { galaxyQA: unknown }).galaxyQA = {
      hud: () => galaxy.hud(),
      profile: () => galaxy.profile(),
      frames: () => [...frameTimes],
      backend: () => backend,
      report: () => ({
        backend,
        userAgent: navigator.userAgent,
        viewport: [innerWidth, innerHeight],
        durationSeconds: captureStarted
          ? (performance.now() - captureStarted) / 1000
          : 0,
        samples: frameTimes.length,
        quads: lastPixels,
        entities: hud.entities,
        particles: hud.particles,
        frameMs: {
          p50: percentile(frameTimes, 0.5),
          p95: percentile(frameTimes, 0.95),
          p99: percentile(frameTimes, 0.99),
        },
        simulationMs: {
          p50: percentile(simulationTimes, 0.5),
          p95: percentile(simulationTimes, 0.95),
        },
        renderMs: {
          p50: percentile(renderTimes, 0.5),
          p95: percentile(renderTimes, 0.95),
        },
        memory:
          (performance as Performance & { memory?: unknown }).memory ?? null,
      }),
    };
  }
  requestAnimationFrame(frame);
  if ("serviceWorker" in navigator && !import.meta.env.DEV) {
    void navigator.serviceWorker
      .register("/sw.js", { scope: "/" })
      .then((registration) => {
        waitingUpdate = registration.waiting;
        navigator.serviceWorker.addEventListener("controllerchange", () => {
          if (applyingUpdate) location.reload();
        });
        registration.addEventListener("updatefound", () => {
          const worker = registration.installing;
          worker?.addEventListener("statechange", () => {
            if (
              worker.state === "installed" &&
              navigator.serviceWorker.controller
            ) {
              waitingUpdate = registration.waiting;
              toast(
                "A new flight build is ready. Apply it from the Haven menu.",
              );
            }
          });
        });
      })
      .catch((error) =>
        console.warn("Offline installation unavailable", error),
      );
  }
}
launch.addEventListener("click", async () => {
  await audio.unlock();
  await resume();
});
element("resume").addEventListener("click", () => void resume());
element("launch-settings").addEventListener("click", openLaunchSettings);
for (const tab of ["upgrades", "settings", "journal", "hangar", "campaign"] as const)
  element(`tab-${tab}`).addEventListener("click", () => selectMenuTab(tab));
element("pause-button").addEventListener("click", () => {
  if (touchUi && !paused) {
    const quick = element("quick-menu");
    quick.hidden = !quick.hidden;
    element("pause-button").setAttribute("aria-expanded", String(!quick.hidden));
    pixelUi.invalidate();
  } else if (paused) void resume();
  else pause(false);
});
element("quick-settings").addEventListener("click", () => pause(false));
element("quick-title").addEventListener("click", returnToTitle);
element("return-title").addEventListener("click", returnToTitle);
element("death-title").addEventListener("click", returnToTitle);
element("retry").addEventListener("click", () => void resume());
function showDeath(): void {
  transitionEpoch++;
  consoleWasRunning = dialogueWasRunning = false;
  paused = true;
  input.enabled = false;
  input.reset();
  destination = null;
  galaxy.set_paused(true);
  galaxy.set_ui_mode(0);
  menu.hidden = true;
  launchScreen.hidden = true;
  deathScreen.hidden = false;
  element("quick-menu").hidden = true;
  document.body.classList.remove("engine-flight");
  audio.suspend();
  narrative.close();
  const target = element("run-summary");
  target.replaceChildren();
  const summary = hud.last_run;
  const report = document.createElement("p");
  report.textContent = summary
    ? "LAST RECORDED EXPEDITION"
    : "NO EXPEDITION RECORDED";
  target.append(report);
  if (summary) for (const line of [
    `${Math.round(summary.time)} SECONDS / ${Math.round(summary.distance_peak)} M FURTHEST FROM HAVEN`,
    `${summary.enemies} ENEMIES / ${summary.asteroids} ASTEROIDS / ${summary.champions} CHAMPIONS`,
    `BANKED POINTS / ${summary.points_gained.join(" / ")}`,
    `${summary.xp_gained} SHIP XP / ${summary.materials_gained} MATERIALS COLLECTED`,
  ]) {
    const row = document.createElement("p");
    row.textContent = line;
    target.append(row);
  }
  const bank = document.createElement("p");
  bank.textContent = `DOCK STASH ${hud.stored_materials} MATERIALS / SHIP LEVEL ${hud.ship_level}`;
  target.append(bank);
  save(galaxy.profile());
  pixelUi.invalidate();
  element("retry").focus();
}
function returnToTitle(): void {
  if (!galaxy) return;
  transitionEpoch++;
  preparingEncounter = null;
  consoleWasRunning = dialogueWasRunning = false;
  // Returning ends an active encounter through the same Rust retreat rules.
  galaxy.return_to_home();
  hud = galaxy.hud();
  save(galaxy.profile());
  narrative.close();
  dialoguePanel.close();
  debugConsole.close();
  launched = false;
  paused = true;
  beforeLaunchMenu = false;
  menu.hidden = true;
  deathScreen.hidden = true;
  launchScreen.hidden = false;
  element("quick-menu").hidden = true;
  destination = null;
  input.enabled = false;
  input.reset();
  galaxy.set_paused(true);
  galaxy.set_ui_mode(2);
  document.body.classList.remove("engine-flight");
  audio.suspend();
  pixelUi.invalidate();
  launch.focus();
}
element("dock-button").addEventListener("click", () => pause(true));
function interactSite(): void {
  if (paused || debugConsole.opened || hud.campaign.encounter) return;
  if (hud.campaign.arena) {
    const arena = hud.campaign.arena;
    const champion = hud.campaign.champions[arena.index];
    if (!champion) return;
    showConversation({ id: champion.id, speaker: champion.name, personality: champion.index,
      description: `${champion.species} arena champion. I fly the ${champion.ship}. ${champion.ability}`,
      status: champion.patch ? "completed" : "available",
      facts: [champion.patch ? "You earned my patch and blueprint. Craft my ship at Haven." : "Win this duel to earn my arena patch and ship blueprint.", "The trial boundary is marked. Retreat or ship loss ends your attempt."] },
      arena.can_challenge ? "CHALLENGE" : undefined, arena.can_challenge ? () => beginChampion(arena.index) : undefined);
    return;
  }
  if (hud.campaign.tournament_unlocked && hud.campaign.tournament_nearby) {
    showConversation({ id: "galaxy-royale", speaker: "ROYALE MARSHAL", personality: 1,
      description: "Welcome to Galaxy Royale. Rival champions from eight universes await. Every entrant meets every other entrant once.",
      status: hud.campaign.tournament_won ? "completed" : "available",
      facts: ["Win duels for three points. Finish first to unlock the Sovereign ship and complete the campaign.", "NPC versus NPC results are seeded simulations. Your eight matches are real duels."] },
      hud.campaign.tournament_won ? "END CREDITS" : hud.campaign.tournament_complete ? "NEW SEASON" : "ENTER DUEL", hud.campaign.tournament_won ? openCredits : beginTournament);
    return;
  }
  if (!hud.site) return;
  const site = hud.site;
  showConversation({id: site.id, speaker: site.name, description: site.description,
    personality: [...site.id].reduce((sum, c) => sum + c.charCodeAt(0), 0) % 3,
    status: site.status as NarrativeContext["status"]}, site.can_interact
      ? site.status === "ready"
        ? "CLAIM RELIC"
        : "ACCEPT EVENT"
      : undefined, site.can_interact ? commitSite : undefined);
}
function showConversation(context: NarrativeContext, label?: string, action?: () => void | Promise<void>): void {
  const pages = narrative.open(context);
  const draw = (): void => {
    const page = narrative.current();
    if (!page) return;
    const index = pages.findIndex(candidate => candidate.id === page.id);
    dialoguePanel.show({speaker: page.speaker, text: page.text, portrait: page.portrait,
      page: index, pageCount: pages.length,
      actionLabel: label, onAction: index === pages.length - 1 ? action : undefined,
      onPrevious: index > 0 ? () => { narrative.previous(); draw(); } : undefined,
      onNext: index < pages.length - 1 ? () => { narrative.next(); draw(); } : undefined,
      onReplay: settings.dialogue_voice ? () => { void narrative.replay().then(() => {
        if (["unavailable", "error"].includes(narrative.voiceStatus)) toast("Voice unavailable; dialogue remains readable.");
      }); } : undefined});
  };
  draw();
}
function commitSite(): void {
  try {
    galaxy.interact_site();
    hud = galaxy.hud();
    save(galaxy.profile());
    paintHud();
    dialoguePanel.close();
    toast(
      hud.site?.status === "completed"
        ? "RELIC ACQUIRED / +1 MAX HULL"
        : "EVENT ACCEPTED / see Journal for progress",
    );
  } catch (error) {
    toast(String(error));
  }
}
element("site-button").addEventListener("click", interactSite);
element("apply-update").addEventListener("click", async () => {
  if (!waitingUpdate || !galaxy.hud().dock) return;
  try {
    await store.save(galaxy.profile());
    await store.flush();
    applyingUpdate = true;
    waitingUpdate.postMessage("ACTIVATE_AT_DOCK");
  } catch {
    toast("Save failed. The update remains queued.");
  }
});
function toggleFullscreen(): void {
  if (document.fullscreenElement) void document.exitFullscreen();
  else
    void document.documentElement
      .requestFullscreen()
      .catch(() => toast("Fullscreen is unavailable in this browser."));
}
element("fullscreen").addEventListener("click", toggleFullscreen);
element("menu-fullscreen").addEventListener("click", toggleFullscreen);
window.addEventListener("keydown", (event) => {
  if (!deathScreen.hidden) {
    if (event.key === "Tab") {
      event.preventDefault();
      const retry = element("retry");
      const title = element("death-title");
      (document.activeElement === retry ? title : retry).focus();
    }
    return;
  }
  if (!launched) {
    if (event.key === "Escape" && beforeLaunchMenu) void resume();
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    if (paused) void resume();
    else pause(false);
  }
  if (event.key.toLowerCase() === "e" && hud?.dock && !paused) pause(true);
  else if (event.key.toLowerCase() === "e" && (hud?.site || hud?.campaign.arena || hud?.campaign.tournament_nearby) && !paused)
    interactSite();
});
window.addEventListener("resize", resize);
document.addEventListener("visibilitychange", () => {
  if (document.hidden && galaxy) {
    if (launched && !paused) pause(false);
    input?.reset();
    audio.suspend();
    narrative.pause();
    save(galaxy.profile());
  }
});
for (const key of ["master", "music", "sfx"] as const) {
  const slider = element<HTMLInputElement>(`${key}-volume`);
  slider.value = String(settings[key]);
  slider.addEventListener("input", () => {
    settings[key] = Number(slider.value);
    audio.setLevels(settings.master, settings.music, settings.sfx);
    narrative.setVoiceVolume(settings.master * settings.voice_volume);
    saveSettings(settings);
  });
}
const shake = element<HTMLInputElement>("shake-setting");
shake.checked = settings.shake;
shake.addEventListener("change", () => {
  settings.shake = shake.checked;
  galaxy?.set_shake(settings.shake);
  saveSettings(settings);
});
const motion = element<HTMLInputElement>("motion-setting");
motion.checked = settings.reduced_motion;
motion.addEventListener("change", () => {
  settings.reduced_motion = motion.checked;
  document.body.classList.toggle("reduced-motion", motion.checked);
  galaxy?.set_reduced_motion(motion.checked);
  saveSettings(settings);
});
const density = element<HTMLInputElement>("density-setting");
density.checked = settings.cosmetic_density === "reduced";
density.addEventListener("change", () => {
  settings.cosmetic_density = density.checked ? "reduced" : "full";
  galaxy?.set_cosmetic_density(density.checked);
  saveSettings(settings);
});
const voice = element<HTMLInputElement>("voice-setting");
voice.checked = settings.dialogue_voice;
voice.addEventListener("change", () => {
  settings.dialogue_voice = voice.checked;
  narrative.setVoiceEnabled(voice.checked);
  saveSettings(settings);
});
const voiceVolume = element<HTMLInputElement>("voice-volume");
voiceVolume.value = String(settings.voice_volume);
voiceVolume.addEventListener("input", () => {
  settings.voice_volume = Number(voiceVolume.value);
  narrative.setVoiceVolume(settings.master * settings.voice_volume);
  saveSettings(settings);
});
const zoomLevel = element<HTMLInputElement>("zoom-level");
zoomLevel.value = String(settings.zoom);
function setZoom(value: number): void {
  if (!Number.isFinite(value)) return;
  settings.zoom = Math.max(0.6, Math.min(1.6, value));
  galaxy?.set_zoom(settings.zoom);
  zoomLevel.value = String(settings.zoom);
  saveSettings(settings);
}
zoomLevel.addEventListener("input", () => setZoom(Number(zoomLevel.value)));
function onFlightWheel(event: WheelEvent): void {
  if (!launched || paused || debugConsole.opened || dialoguePanel.opened) return;
  event.preventDefault();
  const factor = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? innerHeight : 1;
  const delta = Math.max(-240, Math.min(240, event.deltaY * factor));
  setZoom(settings.zoom * Math.exp(-delta * 0.001));
}
element("export-save").addEventListener("click", () => {
  const url = URL.createObjectURL(
    new Blob([JSON.stringify(galaxy.profile(), null, 2)], {
      type: "application/json",
    }),
  );
  const a = document.createElement("a");
  a.href = url;
  a.download = "galaxy-royale-save.json";
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
});
element<HTMLInputElement>("import-save").addEventListener(
  "change",
  async (event) => {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;
    try {
      if (file.size > 65536) throw new Error("Save file is too large.");
      const value: unknown = JSON.parse(await file.text());
      if (!validProfile(value)) throw new Error("Unsupported or invalid save.");
      pause(false);
      await store.replace(value);
      location.reload();
    } catch (error) {
      toast(String(error));
    } finally {
      target.value = "";
    }
  },
);
void init().catch((error) => {
  console.error(error);
  loadStatus.textContent = "UNABLE TO START";
  const explanation = document.createElement("p");
  explanation.textContent = `${String(error)}. Enable hardware acceleration or try another browser.`;
  element("launch-screen").append(explanation);
});
