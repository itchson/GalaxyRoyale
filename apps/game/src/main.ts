import "./style.css";
import { FlightInput } from "./input";
import { AudioEngine } from "./audio";
import { PixelUiBridge } from "./pixel-ui";
import { DebugConsole } from "./debug-console";
import { DialoguePanel } from "./dialogue";
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
  boss: { name: string; health: number; max_health: number } | null;
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
  set_paused(value: boolean): void;
  set_shake(value: boolean): void;
  set_ui_mode(mode: number): void;
  set_touch_ui(value: boolean): void;
  set_ui_scene(scene: unknown): void;
  set_reduced_motion(value: boolean): void;
  set_ui_insets(top: number, right: number, bottom: number, left: number): void;
  set_cosmetic_density(value: boolean): void;
  interact_site(): void;
  diagnostic(command: string): void;
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
const launch = element<HTMLButtonElement>("launch");
launch.disabled = true;
const loadStatus = element("load-status");
const menu = element("menu-screen");
const launchScreen = element("launch-screen");
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
let menuTab: "upgrades" | "settings" | "journal" = "upgrades";
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
      return "HELP / STATS / PROFILE / RENDERER / RECOVER\nSCENE dock|belt|combat|site|boss-dart|boss-bastion|boss-coil|stress|die (QA mode)\nCLEAR / Up, Down: history / Backtick or Escape: close";
    if (verb === "stats")
      return `${backend.toUpperCase()} / ${fps} FPS / p95 ${percentile(frameTimes, 0.95).toFixed(2)} ms\n${lastPixels} quads / ${hud.entities} entities / ${hud.particles} particles\nPOSITION ${hud.position.map((v) => v.toFixed(1)).join(", ")}\n${hud.biome.name} / ${hud.zone} / danger ${hud.danger}\nHULL ${hud.hull.toFixed(0)}/${hud.max_hull.toFixed(0)} / SHIELD ${hud.shield.toFixed(0)}`;
    if (verb === "profile") {
      const p = galaxy.profile();
      return `SCHEMA ${p.schema_version} / revision ${p.revision} / seed ${p.seed}\nCURRENCY ${p.currency.join(" / ")}\nDISCOVERIES ${p.discoveries.length} / quests ${p.quests.length} / relics ${p.relics.length}\n${store.message}`;
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
          "boss-dart",
          "boss-bastion",
          "boss-coil",
          "stress",
          "die",
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
    if (
      dialogueWasRunning &&
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
    launched ? (menu.hidden ? 1 : 3) : beforeLaunchMenu ? 3 : 2,
  );
  document.body.classList.toggle("engine-flight", launched && menu.hidden);
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
    input.viewport = [v[0], v[1]];
    document.documentElement.style.setProperty(
      "--ui-scale",
      String(innerHeight / v[1]),
    );
    const safe = getComputedStyle(safeAreaProbe);
    const uiScale = innerHeight / v[1];
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
  paused = true;
  input.enabled = false;
  input.reset();
  galaxy.set_paused(true);
  galaxy.set_ui_mode(3);
  document.body.classList.remove("engine-flight");
  audio.suspend();
  destination = null;
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
function selectMenuTab(tab: "upgrades" | "settings" | "journal"): void {
  menuTab = tab;
  element("export-save").hidden = tab !== "settings";
  element("import-label").hidden = tab !== "settings";
  element("menu-fullscreen").hidden = tab !== "settings";
  element("upgrade-content").hidden = tab !== "upgrades";
  element("settings-content").hidden = tab !== "settings";
  element("quest-content").hidden = tab !== "journal";
  for (const page of ["upgrades", "settings", "journal"])
    element(`tab-${page}`).dataset.active = String(page === tab);
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
    recovering ||
    graphicsFailed ||
    debugConsole.opened ||
    dialoguePanel.opened ||
    document.hidden ||
    matchMedia("(pointer:coarse) and (orientation:portrait)").matches
  )
    return;
  menu.hidden = true;
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
      heading.textContent = upgrade.name;
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
        hud.currency[category] < 10;
      buy.textContent = !upgrade.unlocked
        ? "BOSS SIGNATURE REQUIRED"
        : upgrade.rank >= upgrade.max_rank
          ? "FULLY EVOLVED"
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
  siteButton.hidden = !hud.site || paused || !launched;
  if (hud.site) {
    siteButton.textContent = `${hud.site.name} / CONTACT [E]`;
    siteButton.disabled = false;
    siteButton.title = hud.site.description;
  }
  element("boss-hud").hidden = !hud.boss;
  if (hud.boss) {
    element("boss-name").textContent = hud.boss.name;
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
    const result = galaxy.step(dt, input.sample());
    hud = result.hud;
    input.camera = hud.camera;
    if (result.profile) save(result.profile);
    for (const event of result.events) {
      audio.play(event);
      if (event.kind === "death") {
        toast("SHIP LOST / collected points retained");
        destination = null;
      }
      if (event.kind === "boss") toast("BOSS SIGNATURE / incoming");
      if (event.kind === "pickup" && event.intensity >= 2)
        toast("NEW SIGNATURE ABILITY / evolve at Haven");
    }
    audio.setDanger(hud.danger, hud.safe);
    if (launched && !paused && !previousDock && hud.dock) pause(true);
    previousDock = hud.dock;
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
  );
  galaxy.set_paused(true);
  galaxy.set_shake(settings.shake);
  galaxy.set_reduced_motion(settings.reduced_motion);
  galaxy.set_cosmetic_density(settings.cosmetic_density === "reduced");
  galaxy.set_touch_ui(touchUi);
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
for (const tab of ["upgrades", "settings", "journal"] as const)
  element(`tab-${tab}`).addEventListener("click", () => selectMenuTab(tab));
element("pause-button").addEventListener("click", () =>
  paused ? void resume() : pause(false),
);
element("dock-button").addEventListener("click", () => pause(true));
function interactSite(): void {
  if (!hud.site || paused || debugConsole.opened) return;
  const site = hud.site;
  dialoguePanel.show({
    speaker: site.name,
    text: site.description,
    portrait: [...site.id].reduce((sum, c) => sum + c.charCodeAt(0), 0) % 3,
    actionLabel: site.can_interact
      ? site.status === "ready"
        ? "CLAIM RELIC"
        : "ACCEPT EVENT"
      : undefined,
    onAction: site.can_interact ? commitSite : undefined,
  });
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
  else if (event.key.toLowerCase() === "e" && hud?.site && !paused)
    interactSite();
});
window.addEventListener("resize", resize);
document.addEventListener("visibilitychange", () => {
  if (document.hidden && galaxy) {
    if (launched && !paused) pause(false);
    input?.reset();
    audio.suspend();
    save(galaxy.profile());
  }
});
for (const key of ["master", "music", "sfx"] as const) {
  const slider = element<HTMLInputElement>(`${key}-volume`);
  slider.value = String(settings[key]);
  slider.addEventListener("input", () => {
    settings[key] = Number(slider.value);
    audio.setLevels(settings.master, settings.music, settings.sfx);
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
