import "./style.css";
import { FlightInput } from "./input";
import { watchPwaUpdates } from "./pwa-updates";
import { AudioEngine, type AudioState } from "./audio";
import { PixelUiBridge } from "./pixel-ui";
import { paintFlightMenu, type FlightMenuHud } from "./flight-menu";
import { paintUniverseMap, type UniverseMapData } from "./universe-map";
import {
  bindWebglLifecycle,
  createWebglLossProbe,
  type WebglLifecycle,
} from "./webgl-lifecycle";
import { DebugConsole } from "./debug-console";
import { DialoguePanel } from "./dialogue";
import type { PortraitIdentity } from "./kikai-portrait";
import { DroidCreation, type IdentityHud, type OnboardingHud } from "./kikai";
import {
  paintCampaign,
  paintCredits,
  type CampaignHud,
  type CampaignActions,
} from "./campaign";
import {
  paintForge,
  resetForge,
  paintShipHangar,
  type ForgeHud,
} from "./loadout";
import {
  NarrativeDirector,
  splitPages,
  type NarrativeContext,
} from "./narrative";
import doraGuideSource from "../../../assets/narrative/dora-initial-mission.json?raw";
import {
  SaveStore,
  SaveConflict,
  loadSettings,
  saveSettings,
  validProfile,
  type Profile,
  type RunSummary,
} from "./storage";

declare const __WASM_BUILDS__: Record<"webgpu" | "webgl2", string>;

interface Hud extends ForgeHud, FlightMenuHud {
  identity: IdentityHud | null;
  onboarding: OnboardingHud | null;
  audio: AudioState;
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
  boss: {
    name: string;
    health: number;
    max_health: number;
    family: number;
    generation: number;
    phase: string;
    telegraph: boolean;
  } | null;
  boss_progress: { family: number; defeats: number[]; queued: number[] }[];
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
  navigation: {
    home: [number, number];
    quest: {
      position: [number, number];
      kind: number;
      site_id: string;
      name: string;
    } | null;
  };
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
  last_run: RunSummary | null;
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
interface RendererRecovery {
  free(): void;
}
interface GalaxyRuntime {
  step(milliseconds: number, input: unknown): Step;
  hud(): Hud;
  universe_map(): UniverseMapData;
  profile(): Profile;
  purchase(id: string): void;
  craft_ability(id: string): void;
  upgrade_ability(id: string): void;
  equip_ability(slot: number, id: string | null): void;
  resize(width: number, height: number): void;
  viewport(): Float32Array;
  world_viewport(): Float32Array;
  flight_status_bounds(): Float32Array;
  flight_navigation_bounds(): Float32Array;
  flight_status_text_bounds(): Float32Array;
  set_zoom(value: number): void;
  zoom(): number;
  set_paused(value: boolean): void;
  cancel_course(): void;
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
  register_droid(
    kind: string,
    corporation: string,
    subCorporation: string,
    hexId: string,
  ): void;
  acknowledge_license(): void;
  dock_facility_at(x: number, y: number): number;
  diagnostic(command: string): void;
  boss_ui_bounds(): Float32Array;
  install_recovery(renderer: RendererRecovery): void;
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
const narrative = new NarrativeDirector({
  voiceEnabled: settings.dialogue_voice,
  voiceVolume: settings.master * settings.voice_volume,
  onVoiceStatus: (status) => dialoguePanel?.setSpeaking(status === "speaking"),
});
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
let prepareRecovery: (canvas: HTMLCanvasElement) => Promise<RendererRecovery>;
let input: FlightInput;
let pixelUi: PixelUiBridge;
let flightSafeInsets: [number, number] = [0, 0];
let beforeLaunchMenu = false;
type MenuTab =
  | "pause"
  | "upgrades"
  | "hangar"
  | "campaign"
  | "journal"
  | "map"
  | "settings"
  | "archive"
  | "credits";
let menuTab: MenuTab = "upgrades";
let lastMapPaint = 0;
const facilityPages = ["upgrades", "hangar", "journal", "campaign"] as const;
const facilityPositions = [
  [-120, -80],
  [120, -80],
  [-120, 95],
  [120, 95],
] as const;
let pendingSaves = 0;
let saveFailed = false;
let creditsPresented = false;
let transitionEpoch = 0;
let preparingEncounter: number | null = null;
let hud: Hud;
let launched = false;
let needsDroidCreation = false;
let lastTrainingStage = -1;
let paused = true;
let recovering = false;
let graphicsFailed = false;
let webglLifecycle: WebglLifecycle | null = null;
let recoverAfterContextRestore = false;
let lastTime = 0;
let previousDock = true;
let destination: [number, number] | null = null;
let toastTimeout: number | undefined;
let graphicsNotice: string | null = null;
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
    syncFlightState();
    if (!dialoguePanel.opened) restoreUi();
    pixelUi?.invalidate();
  },
  run: async (command) => {
    const [verb, ...args] = command.trim().toLowerCase().split(/\s+/);
    if (verb === "help")
      return "HELP / STATS / PROFILE / RENDERER / RECOVER\nSCENE dock|belt|combat|site|biome-riveter|biome-mote|boss-dart[-1..5]|boss-bastion[-1..5]|boss-coil[-1..5]|champion-0..7|tournament|stress|die (QA mode)\nWARP x y (QA mode, world coordinates within +/-1000000)\nCLEAR / Up, Down: history / Backtick or Escape: close";
    if (verb === "stats") {
      const panel = Array.from(galaxy.boss_ui_bounds(), (value) =>
        value.toFixed(0),
      );
      return `${backend.toUpperCase()} / ${fps} FPS / p95 ${percentile(frameTimes, 0.95).toFixed(2)} ms\n${lastPixels} quads / ${hud.entities} entities / ${hud.particles} particles\nPOSITION ${hud.position.map((v) => v.toFixed(1)).join(", ")}\n${hud.biome.name} / ${hud.zone} / danger ${hud.danger}\nHULL ${hud.hull.toFixed(0)}/${hud.max_hull.toFixed(0)} / SHIELD ${hud.shield.toFixed(0)}\nZOOM ${galaxy.zoom().toFixed(2)} / WORLD ${Array.from(galaxy.world_viewport(), (v) => v.toFixed(0)).join(" x ")} / UI ${galaxy.viewport().join(" x ")}\nRAICHO LV ${hud.ship_level} / XP ${hud.ship_xp} / CARGO ${hud.cargo_materials}/${hud.cargo_capacity}\nLAST BOSS UI ${panel.length ? panel.join(" / ") : "NONE"}`;
    }
    if (verb === "profile") {
      const p = galaxy.profile();
      const training = p.onboarding
        ? `TRAINING v${p.onboarding.version} / STAGE ${p.onboarding.stage} / ASTEROIDS ${p.onboarding.asteroids_destroyed}/10 / MATERIALS ${p.onboarding.materials_collected}/10`
        : "TRAINING NONE";
      return `SCHEMA ${p.schema_version} / revision ${p.revision} / seed ${p.seed}\nCURRENCY ${p.currency.join(" / ")}\nBOSS GENERATIONS ${p.boss_generations?.map((row) => row.join(",")).join(" / ") ?? "legacy"}\nDISCOVERIES ${p.discoveries.length} / missions ${p.quests.length} / relics ${p.relics.length}\nPATCHES ${p.campaign.patches.filter(Boolean).length}/8 / RAICHO ${p.campaign.selected_ship} / Royale ${p.campaign.victory_claimed ? "WON" : "PENDING"}\nRAICHO XP ${p.campaign.ship_xp.join(" / ")}\nMATERIAL CARGO ${p.campaign.cargo_materials.join(" / ")} / STASH ${p.campaign.stored_materials.join(" / ")}\nNAMED T1 CARGO ${p.campaign.cargo_training_materials.join(" / ")} / STASH ${p.campaign.stored_training_materials.join(" / ")}\n${training}\nSLOTS ${p.campaign.loadouts[p.campaign.selected_ship].slots.map((id) => id ?? "EMPTY").join(" / ")}\n${store.message}`;
    }
    if (verb === "renderer")
      return `${backend.toUpperCase()} / ${galaxy.viewport().join(" x ")} logical pixels / eight bounded lights`;
    if (verb === "recover")
      return (await recover("console recovery request"))
        ? "Graphics restored; expedition retained."
        : "Recovery unavailable or failed; see graphics status.";
    if (verb === "warp") {
      if (!diagnosticMode) return "WARP requires ?qa=1.";
      const position = args.map(Number);
      if (
        position.length !== 2 ||
        position.some(
          (value) => !Number.isFinite(value) || Math.abs(value) > 1_000_000,
        )
      )
        return "Usage: WARP x y / finite world coordinates within +/-1000000.";
      galaxy.diagnostic(`warp ${position.join(" ")}`);
      hud = galaxy.hud();
      scenario = "warp";
      destination = null;
      previousDock = true;
      paintHud();
      return `WARP ${position.join(", ")} / QA combat and site rewards disabled. Run ended; carried materials discarded; active duel forfeited.`;
    }
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
          "biome-mote",
          "boss-dart",
          "boss-bastion",
          "boss-coil",
          ...["dart", "bastion", "coil"].flatMap((family) =>
            [1, 2, 3, 4, 5].map((generation) => `boss-${family}-${generation}`),
          ),
          "stress",
          "die",
          ...Array.from({ length: 8 }, (_, i) => `champion-${i}`),
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
    syncFlightState();
    if (!debugConsole.opened) restoreUi();
    pixelUi?.invalidate();
  },
});
const droidCreation = new DroidCreation({
  onOpen: () => {
    launchScreen.hidden = true;
    galaxy.set_ui_mode(0);
    input.reset();
    input.enabled = false;
    pixelUi?.invalidate();
  },
  onClose: () => {
    pixelUi?.invalidate();
    if (!needsDroidCreation) {
      void resume().then(() => {
        if (!launched) {
          launchScreen.hidden = false;
          galaxy.set_ui_mode(2);
          pixelUi?.invalidate();
          launch.focus();
          return;
        }
        showDoraGuide();
      });
    } else {
      launchScreen.hidden = false;
      galaxy.set_ui_mode(2);
      launch.focus();
    }
  },
  create: async (identity) => {
    // Retry the same issued license after a failed save, without another grant.
    if (!galaxy.hud().identity)
      galaxy.register_droid(
        identity.kind,
        identity.corporation,
        identity.sub_corporation,
        identity.hex_id,
      );
    hud = galaxy.hud();
    await persist(galaxy.profile());
    needsDroidCreation = false;
    launch.textContent = "CONTINUE EXPEDITION";
    paintHud();
  },
});
function showDoraGuide(): void {
  const training = hud?.onboarding;
  if (!training || !hud.identity || !launched || !deathScreen.hidden) return;
  if (!menu.hidden) menu.hidden = true;
  narrative.close();
  const content = JSON.parse(doraGuideSource) as {
    stages: { stage: number; pages: { text: string; expression: string }[] }[];
  };
  const authored = content.stages.find(
    (stage) => stage.stage === training.stage,
  )?.pages ?? [{ text: training.description, expression: "neutral" }];
  const pages = authored.flatMap((page) =>
    splitPages(
      page.text
        .replaceAll(
          "{material}",
          training.material_name ?? "matching materials",
        )
        .replaceAll(
          "{asteroid}",
          training.asteroid_name ?? "designated asteroids",
        )
        .replaceAll(
          "{starter}",
          training.starter_name ?? hud.identity!.starter_ability,
        )
        .replaceAll(
          "{slot}",
          training.starter_slot ?? hud.identity!.corporation_role.toLowerCase(),
        ),
    ).map((text) => ({
      text,
      expression: ["neutral", "focused", "concerned", "pleased"].indexOf(
        page.expression,
      ),
    })),
  );
  if (training.stage >= 2 && training.stage <= 5 && training.version === 2)
    pages.unshift(
      ...splitPages(training.description).map((text) => ({
        text,
        expression: 1,
      })),
    );
  if (training.stage === 0)
    pages.unshift({
      text: `Welcome, ${hud.identity.short_designation}. Your KIKAI license includes ${hud.identity.starter_ship} and ${hud.identity.starter_ability}.`,
      expression: 3,
    });
  let index = 0;
  const draw = (): void => {
    dialoguePanel.show({
      speaker: "APR-1337 / DORA",
      text: pages[index].text,
      portrait: 8,
      expression: Math.max(0, pages[index].expression),
      page: index,
      pageCount: pages.length,
      onPrevious:
        index > 0
          ? () => {
              index--;
              draw();
            }
          : undefined,
      onNext:
        index < pages.length - 1
          ? () => {
              index++;
              draw();
            }
          : undefined,
      actionLabel: training.stage === 0 ? "BEGIN TRAINING" : "CONTINUE FLIGHT",
      onAction:
        index === pages.length - 1
          ? async () => {
              const presentation = dialoguePanel.presentationToken;
              if (hud.onboarding?.stage === 0) {
                galaxy.acknowledge_license();
                hud = galaxy.hud();
                await persist(galaxy.profile());
              }
              if (
                !dialoguePanel.opened ||
                dialoguePanel.presentationToken !== presentation
              )
                return;
              dialoguePanel.close();
              toast(hud.onboarding?.description ?? "Your expedition is ready.");
            }
          : undefined,
    });
  };
  draw();
  pixelUi?.invalidate();
}
function restoreUi(): void {
  if (
    graphicsFailed &&
    launched &&
    menu.hidden &&
    deathScreen.hidden &&
    preparingEncounter === null &&
    !debugConsole.opened &&
    !dialoguePanel.opened
  ) {
    pause(false);
    return;
  }
  galaxy.set_ui_mode(
    !deathScreen.hidden
      ? 0
      : launched
        ? menu.hidden
          ? 1
          : 3
        : beforeLaunchMenu
          ? 3
          : 2,
  );
  document.body.classList.toggle(
    "engine-flight",
    launched && menu.hidden && deathScreen.hidden,
  );
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
  target.textContent = graphicsNotice ?? message;
  target.classList.add("visible");
  clearTimeout(toastTimeout);
  toastTimeout = undefined;
  if (graphicsNotice === null)
    toastTimeout = window.setTimeout(
      () => target.classList.remove("visible"),
      3500,
    );
}
function paintSaveStatus(): void {
  const state = saveFailed || !store.persistent ? 2 : pendingSaves > 0 ? 1 : 0;
  const indicator = element("save-indicator");
  indicator.dataset.pixelValue = String(state);
  indicator.title =
    state === 1
      ? "Saving progression…"
      : state === 2
        ? store.message
        : "Progress saved automatically on this device.";
  indicator.setAttribute(
    "aria-label",
    `${state === 1 ? "Saving" : state === 2 ? "Save needs attention" : "Progress saved"}. Open save and backup`,
  );
  const footer = element("save-status");
  footer.textContent =
    state === 1
      ? "Saving…"
      : state === 2
        ? "Save needs attention."
        : "Saved on this device.";
  footer.title = indicator.title;
  pixelUi?.invalidate();
}
async function persist(profile: Profile): Promise<void> {
  pendingSaves++;
  paintSaveStatus();
  try {
    await store.save(profile);
    saveFailed = false;
  } catch (error) {
    saveFailed = true;
    throw error;
  } finally {
    pendingSaves--;
    paintSaveStatus();
  }
}
function save(profile: Profile): void {
  if (needsDroidCreation) return;
  void persist(profile).catch((error) => {
    if (error instanceof SaveConflict && launched && menu.hidden) pause(false);
    toast(store.message);
  });
}
function resize(): void {
  // Window events can arrive while a replacement device is being prepared.
  // Configure only a live renderer; successful recovery applies the latest size.
  if (recovering || graphicsFailed || webglLifecycle?.lost) return;
  resizeGraphics();
}
function resizeGraphics(): void {
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
    flightSafeInsets = [
      Number.parseFloat(safe.paddingTop) || 0,
      Number.parseFloat(safe.paddingBottom) || 0,
    ];
    const uiScale = innerHeight / v[1];
    document.body.classList.toggle("narrow-ui", v[0] < 430);
    document.body.classList.toggle("short-ui", v[1] < 240);
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
    syncFlightStatusLayout();
    pixelUi?.invalidate();
  }
  if (
    matchMedia("(pointer:coarse) and (orientation:portrait)").matches &&
    launched &&
    !paused
  )
    pause(false);
  if (launched && !recovering && !graphicsFailed && !webglLifecycle?.lost)
    syncFlightState();
}
function syncFlightStatusLayout(): void {
  if (!galaxy) return;
  const viewport = galaxy.viewport();
  const scale = innerHeight / viewport[1];
  if (!Number.isFinite(scale) || scale <= 0) return;
  const strip = element("flight-status-strip");
  const writeBounds = (prefix: string, bounds: Float32Array): boolean => {
    const valid =
      bounds.length === 4 &&
      Array.from(bounds).every(Number.isFinite) &&
      bounds[2] > 0 &&
      bounds[3] > 0;
    for (const [i, property] of ["x", "y", "width", "height"].entries()) {
      const value = `${valid ? bounds[i] * scale : 0}px`;
      const name = `--flight-${prefix}-${property}`;
      if (strip.style.getPropertyValue(name) !== value)
        strip.style.setProperty(name, value);
    }
    return valid;
  };
  writeBounds("strip", galaxy.flight_status_bounds());
  const text = galaxy.flight_status_text_bounds();
  const hasContext = writeBounds("text", text);
  strip.dataset.contextAvailable = String(hasContext);
  // Match the shared CSS --ui-hit: max(44px, 22 logical cells).
  const hitHeight = Math.max(44, 22 * scale);
  const hitY = hasContext
    ? Math.max(
        flightSafeInsets[0],
        Math.min(
          innerHeight - flightSafeInsets[1] - hitHeight,
          (text[1] + text[3] / 2) * scale - hitHeight / 2,
        ),
      )
    : 0;
  const hitTop = `${hitY}px`;
  if (strip.style.getPropertyValue("--flight-hit-y") !== hitTop)
    strip.style.setProperty("--flight-hit-y", hitTop);
  const hit = {
    left: text[0] * scale,
    top: hitY,
    right: (text[0] + text[2]) * scale,
    bottom: hitY + hitHeight,
  };
  const overlaps = (rect: {
    left: number;
    top: number;
    right: number;
    bottom: number;
  }): boolean =>
    hit.left < rect.right &&
    hit.right > rect.left &&
    hit.top < rect.bottom &&
    hit.bottom > rect.top;
  const navigation = galaxy.flight_navigation_bounds();
  // The accessible context hit can be taller than its glyphs. Keep the entire
  // target clear of the native map, as well as measured shell controls.
  const blocksNavigation =
    navigation.length === 4 &&
    Array.from(navigation).every(Number.isFinite) &&
    navigation[2] > 0 &&
    navigation[3] > 0 &&
    overlaps({
      left: navigation[0] * scale,
      top: navigation[1] * scale,
      right: (navigation[0] + navigation[2]) * scale,
      bottom: (navigation[1] + navigation[3]) * scale,
    });
  const blocked =
    blocksNavigation ||
    [
      "save-indicator",
      "pause-button",
      "map-button",
      "flight-settings",
      ...(touchUi ? ["joystick", "fire-button"] : []),
    ].some((id) => {
      const rect = element(id).getBoundingClientRect();
      return rect.width > 0 && rect.height > 0 && overlaps(rect);
    });
  strip.dataset.contextInteractive = String(
    hasContext &&
      !blocked &&
      hit.top >= flightSafeInsets[0] &&
      hit.bottom <= innerHeight - flightSafeInsets[1],
  );
}
function pause(dock: boolean): void {
  if (!galaxy || !launched) return;
  transitionEpoch++;
  input.enabled = false;
  input.reset();
  galaxy.cancel_course();
  galaxy.set_ui_mode(3);
  document.body.classList.remove("engine-flight");
  destination = null;
  element("quick-menu").hidden = true;
  deathScreen.hidden = true;
  menu.hidden = false;
  // A delayed save conflict can open this menu while a contact is active.
  // Close after showing the menu so the contact restores the current UI mode.
  dialoguePanel.close();
  element("menu-title").textContent = dock
    ? "Evolve your RAICHO"
    : "RAICHO STATUS";
  element("menu-description").textContent = dock
    ? "Craft abilities and improve this RAICHO with banked points and materials."
    : "Live telemetry / the universe keeps running.";
  hud = galaxy.hud();
  paintUpgrades(dock);
  element("apply-update").hidden = !waitingUpdate;
  element<HTMLButtonElement>("apply-update").disabled = !hud.dock;
  paintSaveStatus();
  save(galaxy.profile());
  beforeLaunchMenu = false;
  selectMenuTab(dock ? "upgrades" : "pause");
  element("menu-close").focus();
  syncFlightState();
}
// Menus consume controls without stopping the world. Lifecycle interruptions
// and intentional debug/dialogue/death transitions remain separate gates.
function canSimulateFlight(): boolean {
  return (
    launched &&
    deathScreen.hidden &&
    preparingEncounter === null &&
    !recovering &&
    !graphicsFailed &&
    !webglLifecycle?.lost &&
    !debugConsole.opened &&
    !dialoguePanel.opened &&
    !document.hidden &&
    !matchMedia("(pointer:coarse) and (orientation:portrait)").matches
  );
}
function syncFlightState(): void {
  if (!galaxy) return;
  const run = canSimulateFlight();
  const nextPaused = !run;
  if (paused !== nextPaused) {
    paused = nextPaused;
    galaxy.set_paused(paused);
    lastTime = 0;
  }
  input.enabled = run && menu.hidden;
  if (run) {
    // A profile/world transition may precede the next simulation frame.
    const soundHud = galaxy.hud();
    audio.update({
      ...soundHud.audio,
      corporation: soundHud.identity?.corporation ?? null,
    });
    void audio.resume().catch(() => {});
  } else audio.suspend();
}
function openMenu(page: MenuTab): void {
  if (!galaxy) return;
  if (page === "upgrades") resetForge(element("upgrade-content"));
  if (launched) pause(false);
  else {
    transitionEpoch++;
    beforeLaunchMenu = true;
    launchScreen.hidden = true;
    menu.hidden = false;
    hud = galaxy.hud();
    galaxy.set_ui_mode(3);
  }
  selectMenuTab(page);
  element("menu-close").focus();
}
function interactDock(world: [number, number]): boolean {
  if (!launched || paused || !menu.hidden || !hud.dock) return false;
  const index = galaxy.dock_facility_at(...world);
  if (index < 0 || index >= facilityPages.length) return false;
  openMenu(facilityPages[index]);
  return true;
}
const campaignActions: CampaignActions = {
  navigate: (position) => {
    void navigate(position).catch((error) => toast(String(error)));
  },
  challenge: (index) => {
    void beginChampion(index).catch((error) => toast(String(error)));
  },
  tournament: () => {
    void beginTournament().catch((error) => toast(String(error)));
  },
  craft: (index) => {
    try {
      galaxy.craft_ship(index);
      hud = galaxy.hud();
      save(galaxy.profile());
      paintShipHangar(element("hangar-content"), hud, campaignActions);
      paintHud();
      toast("RAICHO CRAFTED / select it at Haven");
    } catch (error) {
      toast(String(error));
    }
  },
  select: (id) => {
    try {
      galaxy.switch_ship(id);
      hud = galaxy.hud();
      save(galaxy.profile());
      paintShipHangar(element("hangar-content"), hud, campaignActions);
      paintUpgrades(true);
      paintHud();
      toast("RAICHO SELECTED");
    } catch (error) {
      toast(String(error));
    }
  },
  credits: openCredits,
};
async function navigate(position: [number, number]): Promise<void> {
  const epoch = transitionEpoch;
  await resume();
  if (
    epoch !== transitionEpoch ||
    paused ||
    !launched ||
    !menu.hidden ||
    !input.enabled
  )
    return;
  input.setCourse(position);
  toast("COURSE SET / fly or steer to your destination");
}
async function beginEncounter(start: () => void): Promise<void> {
  const epoch = ++transitionEpoch;
  preparingEncounter = epoch;
  syncFlightState();
  try {
    start();
    hud = galaxy.hud();
    await persist(galaxy.profile());
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
    if (preparingEncounter === epoch) {
      preparingEncounter = null;
      syncFlightState();
    }
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
  element("menu-description").textContent =
    "Your Sovereign RAICHO is ready. Thank you for flying.";
  selectMenuTab("credits");
}
function finishCredits(): void {
  galaxy.acknowledge_credits();
  save(galaxy.profile());
  hud = galaxy.hud();
  element("menu-title").textContent = "HANGAR";
  element("menu-description").textContent = hud.dock
    ? "Your collection awaits."
    : "Return to Haven to select your Sovereign.";
  selectMenuTab("hangar");
}
function selectMenuTab(tab: MenuTab): void {
  menuTab = tab;
  const headings: Record<MenuTab, [string, string]> = {
    pause: ["RAICHO", "FLIGHT LIVE"],
    upgrades: ["HAVEN FORGE", "Craft, equip and upgrade."],
    hangar: [
      "RAICHO HANGAR",
      hud.dock ? "Choose your RAICHO." : "Return to Haven to craft or switch.",
    ],
    campaign: [
      "ARENA CHAMPIONS",
      "Eight patches reveal the Galaxy Royale tournament.",
    ],
    settings: ["SETTINGS", ""],
    archive: ["SAVE & BACKUP", "Saved on this device."],
    journal: [
      "EXPEDITION JOURNAL",
      "Discover sanctuaries and return to claim their rewards.",
    ],
    map: ["UNIVERSE MAP", "Discovered beacons / flight live."],
    credits: [
      "GALAXY ROYALE WON",
      "Your Sovereign RAICHO is ready. Thank you for flying.",
    ],
  };
  element("menu-title").textContent = headings[tab][0];
  element("menu-description").textContent = headings[tab][1];
  element("menu-description").hidden = !headings[tab][1];
  element("pause-content").hidden = tab !== "pause";
  element("upgrade-content").hidden = tab !== "upgrades";
  element("settings-content").hidden = tab !== "settings";
  element("archive-content").hidden = tab !== "archive";
  element("quest-content").hidden = tab !== "journal";
  element("map-content").hidden = tab !== "map";
  element("hangar-content").hidden = tab !== "hangar";
  element("campaign-content").hidden = tab !== "campaign" && tab !== "credits";
  menu.classList.toggle("pause-menu", tab === "pause");
  menu.setAttribute("aria-labelledby", "menu-title");
  if (tab === "pause") paintFlightMenu(element("flight-status"), hud);
  if (tab === "map") {
    paintUniverseMap(
      element("map-content"),
      galaxy.universe_map(),
      campaignActions.navigate,
    );
    lastMapPaint = performance.now();
  }
  if (tab === "upgrades") paintUpgrades(hud.dock);
  if (tab === "campaign")
    paintCampaign(element("campaign-content"), hud.campaign, campaignActions);
  if (tab === "hangar")
    paintShipHangar(element("hangar-content"), hud, campaignActions);
  if (tab === "credits")
    paintCredits(element("campaign-content"), {
      ...campaignActions,
      credits: finishCredits,
    });
  if (tab === "journal") {
    const target = element("quest-content");
    target.replaceChildren();
    const heading = document.createElement("p");
    heading.textContent = `${hud.discoveries} discoveries / ${hud.relics} relics. Each relic adds 1 hull, up to 20.`;
    const missionHeading = document.createElement("h3");
    missionHeading.textContent = "MISSIONS";
    target.append(heading, missionHeading);
    if (hud.onboarding) {
      const training = document.createElement("section");
      training.className = "upgrade-category passive";
      const title = document.createElement("h3");
      title.id = "training-title";
      title.textContent = `APR-1337 / ${hud.onboarding.title}`;
      const description = document.createElement("p");
      description.id = "training-description";
      description.textContent = hud.onboarding.description;
      const contact = document.createElement("button");
      contact.textContent = "CONTACT APR-1337";
      contact.addEventListener("click", showDoraGuide);
      training.append(title, description, contact);
      target.append(training);
    }
    const list = document.createElement("ul");
    const knownSites = new Map(
      galaxy.universe_map().sites.map((site) => [site.id, site.name]),
    );
    for (const quest of hud.quest_log) {
      const item = document.createElement("li");
      item.textContent = `${knownSites.get(quest.site_id) ?? quest.site_id} / ${quest.event === "courier" ? "DELIVERY" : "FAMILY BOUNTY"} / ${quest.completed ? "REWARD CLAIMED" : `${quest.progress}/${quest.target} - return to source`}`;
      list.append(item);
    }
    if (!hud.quest_log.length) {
      const hint = document.createElement("p");
      hint.textContent =
        "Seek sanctuary beacons beyond Haven. Accept a delivery or bounty mission, then return for a unique relic.";
      target.append(hint);
    }
    target.append(list);
    const bossHeading = document.createElement("h3");
    bossHeading.textContent = "HOSTILE EVOLUTION";
    const bossHint = document.createElement("p");
    bossHint.textContent =
      "24 family kills summon the first boss. Two defeated bosses summon their next generation. Unfinished queues survive retreat and RAICHO loss.";
    target.append(bossHeading, bossHint);
    for (const family of hud.boss_progress ?? []) {
      const familyHeading = document.createElement("h3");
      familyHeading.textContent =
        ["DART", "BASTION", "COIL"][family.family] ?? "HOSTILE";
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
  openMenu("settings");
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
    galaxy.set_ui_mode(2);
    pixelUi?.invalidate();
    return;
  }
  if (
    !galaxy ||
    preparingEncounter !== null ||
    recovering ||
    graphicsFailed ||
    webglLifecycle?.lost ||
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
  lastTime = 0;
  syncFlightState();
  galaxy.set_ui_mode(1);
  document.body.classList.add("engine-flight");
  await audio.resume().catch(() => {});
  if (
    paused ||
    recovering ||
    graphicsFailed ||
    webglLifecycle?.lost ||
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
function forgeMutation(action: () => void, message: string): void {
  try {
    action();
    hud = galaxy.hud();
    save(galaxy.profile());
    paintUpgrades(hud.dock);
    paintHud();
    pixelUi?.invalidate();
    toast(message);
    void audio
      .resume()
      .then(() => audio.play({ kind: "upgrade", x: 0, y: 0, intensity: 1 }))
      .catch(() => {
        /* Gameplay and saving continue when audio is unavailable. */
      });
  } catch (error) {
    toast(String(error));
  }
}
function paintUpgrades(_dock: boolean): void {
  paintForge(element("upgrade-content"), hud, {
    purchase: (id) =>
      forgeMutation(() => galaxy.purchase(id), "RAICHO HARDWARE UPGRADED"),
    craftAbility: (id) =>
      forgeMutation(
        () => galaxy.craft_ability(id),
        "ABILITY CRAFTED / choose EQUIP to install it",
      ),
    upgradeAbility: (id) =>
      forgeMutation(() => galaxy.upgrade_ability(id), "ABILITY UPGRADED"),
    equipAbility: (slot, id) =>
      forgeMutation(
        () => galaxy.equip_ability(slot, id),
        id ? "ABILITY EQUIPPED" : "ABILITY SLOT EMPTY",
      ),
  });
}
function paintHud(): void {
  if (!hud) return;
  if (hud.onboarding) {
    const title = document.getElementById("training-title");
    const description = document.getElementById("training-description");
    const label = `APR-1337 / ${hud.onboarding.title}`;
    if (title && title.textContent !== label) title.textContent = label;
    if (description && description.textContent !== hud.onboarding.description)
      description.textContent = hud.onboarding.description;
  }
  if (
    launched &&
    hud.onboarding &&
    hud.onboarding.stage !== lastTrainingStage
  ) {
    if (lastTrainingStage >= 0)
      toast(
        `APR-1337 / ${hud.onboarding.title}: ${hud.onboarding.description}`,
      );
    lastTrainingStage = hud.onboarding.stage;
  }
  syncFlightStatusLayout();
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
  element("dock-button").hidden = true;
  const siteButton = element<HTMLButtonElement>("site-button");
  const arena = hud.campaign.arena;
  const gate =
    hud.campaign.tournament_unlocked && hud.campaign.tournament_nearby;
  siteButton.hidden =
    (!hud.site && !arena && !gate) ||
    paused ||
    !menu.hidden ||
    !launched ||
    !!hud.campaign.encounter;
  if (arena || gate) {
    siteButton.textContent = arena
      ? `${arena.name} / CHAMPION [E]`
      : "GALAXY ROYALE / ENTER [E]";
    siteButton.disabled = false;
    siteButton.title = arena
      ? "Contact this arena champion."
      : "Enter the round robin tournament.";
  } else if (hud.site) {
    siteButton.textContent = `${hud.site.name} / CONTACT [E]`;
    siteButton.disabled = false;
    siteButton.title = hud.site.description;
  }
  element("boss-hud").hidden = !hud.boss;
  if (hud.boss) {
    element("boss-name").textContent =
      `${hud.boss.generation > 0 ? `G${hud.boss.generation} / ` : ""}${hud.boss.name}`;
    element("boss-phase").textContent =
      `${hud.boss.telegraph ? "WARNING / " : ""}${hud.boss.phase || "HOSTILE SIGNATURE"}`;
    element("boss-health").style.width = safePercent(
      hud.boss.health,
      hud.boss.max_health,
    );
  }
  // Native pixel objects paint the compass; this hidden semantic counterpart
  // follows the same authoritative target without maintaining a Canvas2D HUD.
  const quest = hud.navigation.quest;
  element("radar").setAttribute(
    "aria-label",
    `Haven ${Math.round(distance)} metres away` +
      (quest
        ? `. ${quest.kind === 0 ? `Deliver the ${quest.name} mission packet to Haven` : quest.kind === 1 ? `Claim mission reward at ${quest.name}` : `Bounty mission beacon ${quest.name}; matching enemies may be defeated elsewhere`}, ${Math.round(Math.hypot(quest.position[0] - hud.position[0], quest.position[1] - hud.position[1]))} metres away.`
        : ". No active mission waypoint."),
  );
}
function paintFacilities(): void {
  const viewport = galaxy.world_viewport();
  const bounds = canvas.getBoundingClientRect();
  const scale = innerHeight / galaxy.viewport()[1];
  const reserved = [
    galaxy.flight_status_bounds(),
    galaxy.flight_navigation_bounds(),
  ];
  const blockers = [
    "save-indicator",
    "pause-button",
    "map-button",
    "flight-settings",
    ...(touchUi ? ["joystick", "fire-button"] : []),
  ]
    .map((id) => element(id).getBoundingClientRect())
    .filter((r) => r.width > 0 && r.height > 0);
  for (const rect of reserved)
    if (
      rect.length === 4 &&
      Array.from(rect).every(Number.isFinite) &&
      rect[2] > 0 &&
      rect[3] > 0
    )
      blockers.push({
        left: rect[0] * scale,
        top: rect[1] * scale,
        right: (rect[0] + rect[2]) * scale,
        bottom: (rect[1] + rect[3]) * scale,
      } as DOMRect);
  const clear = (
    left: number,
    top: number,
    width: number,
    height: number,
  ): boolean =>
    [left, top, width, height, scale, ...viewport].every(Number.isFinite) &&
    scale > 0 &&
    viewport[0] > 0 &&
    viewport[1] > 0 &&
    left >= bounds.left + 2 * scale &&
    left + width <= bounds.right - 2 * scale &&
    top >= Math.max(bounds.top, flightSafeInsets[0]) + 2 * scale &&
    top + height <=
      Math.min(bounds.bottom, innerHeight - flightSafeInsets[1]) - 2 * scale &&
    !blockers.some(
      (r) =>
        left < r.right + 2 * scale &&
        left + width > r.left - 2 * scale &&
        top < r.bottom + 2 * scale &&
        top + height > r.top - 2 * scale,
    );
  for (const [index, position] of facilityPositions.entries()) {
    const node = element(`facility-${index}`);
    const x =
      bounds.left +
      bounds.width * (0.5 + (position[0] - hud.camera[0]) / viewport[0]);
    const y =
      bounds.top +
      bounds.height * (0.5 + (position[1] - hud.camera[1]) / viewport[1]);
    const size = Math.max(44, (48 * bounds.width) / viewport[0]);
    node.hidden =
      !launched ||
      paused ||
      !menu.hidden ||
      !hud.dock ||
      !clear(x - size / 2, y - size / 2, size, size);
    if (!node.hidden) {
      node.style.left = `${x}px`;
      node.style.top = `${y}px`;
      node.style.width = `${size}px`;
      node.style.height = `${size}px`;
      // Match the native100x10 glyph slot; move the label, never its world hit.
      const label = clear(
        x - 50 * scale,
        y + size / 2 + 2 * scale,
        100 * scale,
        10 * scale,
      )
        ? 0
        : clear(
              x - 50 * scale,
              y - size / 2 - 12 * scale,
              100 * scale,
              10 * scale,
            )
          ? 1
          : 2;
      node.dataset.pixelValue = String(label);
      node.dataset.labelPosition = ["below", "above", "hidden"][label];
    }
  }
  pixelUi?.invalidate();
}
function paintDestination(): void {
  const marker = element("destination-marker");
  if (destination) pixelUi?.invalidate();
  if (!destination || paused || !menu.hidden) {
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
function onWebglLost(): void {
  input.reset();
  input.enabled = false;
  galaxy.set_paused(true);
  audio.suspend();
  lastTime = 0;
  document.body.classList.remove("pixel-ready");
  graphicsNotice = "Graphics context lost. Waiting for browser restoration.";
  toast(graphicsNotice);
}
function onWebglRestored(): void {
  if (recovering) recoverAfterContextRestore = true;
  else {
    recoverAfterContextRestore = false;
    void recover("WebGL context restored");
  }
}
async function recover(_trigger: unknown): Promise<boolean> {
  if (!galaxy || recovering || webglLifecycle?.lost) return false;
  recovering = true;
  document.body.classList.remove("pixel-ready");
  input.reset();
  input.enabled = false;
  galaxy.set_paused(true);
  audio.suspend();
  graphicsNotice = "Reconnecting the graphics engine…";
  toast(graphicsNotice);
  try {
    // Preparation owns its new GPU resources, without borrowing the live Game.
    // UI, visibility and save handlers remain safe while this promise is pending.
    const renderer = await prepareRecovery(canvas);
    if (webglLifecycle?.lost || recoverAfterContextRestore) {
      renderer.free();
      return false;
    }
    galaxy.install_recovery(renderer);
    graphicsFailed = false;
    document.body.classList.add("pixel-ready");
    resizeGraphics();
    const canRun =
      launched &&
      deathScreen.hidden &&
      preparingEncounter === null &&
      !debugConsole.opened &&
      !dialoguePanel.opened &&
      !document.hidden &&
      !matchMedia("(pointer:coarse) and (orientation:portrait)").matches;
    paused = !canRun;
    galaxy.set_paused(paused);
    input.enabled = canRun && menu.hidden;
    if (canRun) {
      const soundHud = galaxy.hud();
      audio.update({
        ...soundHud.audio,
        corporation: soundHud.identity?.corporation ?? null,
      });
      void audio.resume().catch(() => {});
    } else if (
      launched &&
      menu.hidden &&
      deathScreen.hidden &&
      preparingEncounter === null &&
      !debugConsole.opened &&
      !dialoguePanel.opened &&
      !document.hidden
    )
      pause(false);
    graphicsNotice = null;
    toast("Graphics restored. Your expedition is intact.");
    return true;
  } catch (recoveryError) {
    if (webglLifecycle?.lost || recoverAfterContextRestore) return false;
    graphicsFailed = true;
    document.body.classList.remove("pixel-ready");
    paused = true;
    if (
      launched &&
      menu.hidden &&
      deathScreen.hidden &&
      preparingEncounter === null &&
      !debugConsole.opened &&
      !dialoguePanel.opened
    )
      pause(false);
    graphicsNotice = `Graphics recovery failed: ${String(recoveryError)}. Reload to return safely to Haven.`;
    toast(graphicsNotice);
    return false;
  } finally {
    recovering = false;
    lastTime = 0;
    if (recoverAfterContextRestore && !webglLifecycle?.lost) {
      recoverAfterContextRestore = false;
      void recover("WebGL restored during preparation");
    }
  }
}
function frame(time: number): void {
  requestAnimationFrame(frame);
  if (
    !galaxy ||
    document.hidden ||
    recovering ||
    graphicsFailed ||
    webglLifecycle?.lost
  )
    return;
  const rawDt = lastTime ? time - lastTime : 0;
  const dt = Math.min(100, rawDt);
  lastTime = time;
  try {
    pixelUi?.sync();
    const controls = input.sample();
    const result = galaxy.step(dt, controls);
    hud = result.hud;
    input.camera = hud.camera;
    const worldViewport = galaxy.world_viewport();
    input.viewport = [worldViewport[0], worldViewport[1]];
    paintFacilities();
    if (result.profile) save(result.profile);
    for (const event of result.events) {
      audio.play(event);
      if (event.kind === "death") {
        showDeath();
      }
      if (event.kind === "champion-win")
        toast("ARENA PATCH + BLUEPRINT ACQUIRED");
      if (event.kind === "tournament-win")
        toast("ROYALE DUEL WON / standings updated");
      if (event.kind === "tournament-loss")
        toast("ROYALE DUEL LOST / standings updated");
      if (event.kind === "pickup" && event.intensity >= 2)
        toast("NEW SIGNATURE ABILITY / evolve at Haven");
    }
    audio.update({
      ...hud.audio,
      corporation: hud.identity?.corporation ?? null,
    });
    previousDock = hud.dock;
    if (
      launched &&
      deathScreen.hidden &&
      preparingEncounter === null &&
      hud.campaign.credits_pending &&
      !creditsPresented &&
      !debugConsole.opened &&
      !dialoguePanel.opened
    ) {
      creditsPresented = true;
      openCredits();
    }
    if (time - lastHudPaint > 80) {
      paintHud();
      if (!menu.hidden && menuTab === "pause") {
        if (paintFlightMenu(element("flight-status"), hud))
          pixelUi?.invalidate();
      }
      lastHudPaint = time;
    }
    if (!menu.hidden && menuTab === "map" && time - lastMapPaint > 1000) {
      if (
        paintUniverseMap(
          element("map-content"),
          galaxy.universe_map(),
          campaignActions.navigate,
        )
      )
        pixelUi?.invalidate();
      lastMapPaint = time;
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
          audio: audio.diagnostics(),
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
  const profile = await store.open((message) => {
    loadStatus.textContent = message;
  });
  needsDroidCreation = profile === null;
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
        /* @vite-ignore */ __WASM_BUILDS__[candidate as "webgpu" | "webgl2"]
      );
      await module.default();
      galaxy = await module.Galaxy.create(canvas, profile);
      prepareRecovery = (target) => module.prepare_renderer_recovery(target);
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
    undefined,
    interactDock,
    () => {
      galaxy.cancel_course();
      destination = null;
    },
  );
  canvas.addEventListener("wheel", onFlightWheel, { passive: false });
  if (backend === "webgl2") {
    webglLifecycle = bindWebglLifecycle(canvas, {
      lost: onWebglLost,
      restored: onWebglRestored,
    });
  }
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
  launch.textContent = needsDroidCreation
    ? "CREATE YOUR DROID"
    : "CONTINUE EXPEDITION";
  paintSaveStatus();
  save(galaxy.profile());
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
    if (webglLifecycle) {
      const probe = createWebglLossProbe(canvas, webglLifecycle);
      if (probe) {
        for (const [label, request] of [
          ["Lose WebGL context", () => probe.requestLoss()],
          ["Restore WebGL context", () => probe.requestRestore()],
        ] as const) {
          const button = document.createElement("button");
          button.textContent = label;
          button.addEventListener("click", () =>
            toast(`${label}: ${request()}`),
          );
          controls.append(button);
        }
      }
    }
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
        navigator.serviceWorker.addEventListener("controllerchange", () => {
          if (applyingUpdate) location.reload();
        });
        watchPwaUpdates(
          registration,
          () => !!navigator.serviceWorker.controller,
          (worker) => {
            waitingUpdate = worker;
            element("apply-update").hidden = false;
            element<HTMLButtonElement>("apply-update").disabled = !hud?.dock;
            pixelUi?.invalidate();
            toast("A new flight build is ready. Apply it from the Haven menu.");
          },
        );
        // Check only after observers are attached. A recent navigation's
        // implicit update check must not be our sole source of notification.
        void registration.update().catch(() => {
          /* Offline play keeps the current coherent cached build. */
        });
      })
      .catch((error) =>
        console.warn("Offline installation unavailable", error),
      );
  }
}
launch.addEventListener("click", async () => {
  const epoch = ++transitionEpoch;
  await audio.unlock();
  if (epoch !== transitionEpoch || beforeLaunchMenu || launched) return;
  if (needsDroidCreation) {
    droidCreation.open();
    return;
  }
  await resume();
  if (hud.onboarding?.stage === 0) showDoraGuide();
});
element("resume").addEventListener("click", () => void resume());
element("launch-settings").addEventListener("click", openLaunchSettings);
element("menu-close").addEventListener("click", () => void resume());
element("pause-settings").addEventListener("click", () => openMenu("settings"));
element("settings-backup").addEventListener("click", () => openMenu("archive"));
// Only the hovered or keyboard-focused action explains itself. Its native text
// shares the toolbar's stable edge; controls never move to make room for it.
const toolbar =
  element("game-ui").querySelector<HTMLElement>(".header-actions")!;
const toolbarHint = element("toolbar-hint");
function showToolbarHint(button: HTMLButtonElement | null): void {
  if (!button || !document.body.classList.contains("engine-flight")) {
    toolbarHint.hidden = true;
    return;
  }
  const labels: Record<string, string> = {
    "pause-button": "RAICHO / ESC",
    "map-button": "UNIVERSE MAP",
    "flight-settings": "SETTINGS",
    "save-indicator": "AUTOSAVE / BACKUP",
    fullscreen: "FULLSCREEN",
  };
  const label = labels[button.id];
  if (!label) return;
  const scale = innerHeight / galaxy.viewport()[1];
  const anchor = toolbar.getBoundingClientRect();
  const map = galaxy.flight_navigation_bounds();
  const left = Math.max(8, anchor.left);
  let top = anchor.bottom + 4 * scale;
  let width = Math.min(160 * scale, innerWidth - left - 8);
  if (map.length === 4 && map[2] > 0 && map[3] > 0) {
    const clearWidth = map[0] * scale - left - 4 * scale;
    if (clearWidth >= 100) width = Math.min(width, clearWidth);
    else top = Math.max(top, (map[1] + map[3]) * scale + 4 * scale);
  }
  toolbarHint.style.left = `${left}px`;
  toolbarHint.style.top = `${top}px`;
  toolbarHint.style.width = `${Math.max(0, width)}px`;
  toolbarHint.textContent = label;
  toolbarHint.hidden = false;
  pixelUi?.invalidate();
}
toolbar.addEventListener("pointerover", (event) =>
  showToolbarHint((event.target as Element).closest("button")),
);
toolbar.addEventListener("pointerout", (event) => {
  if (!toolbar.contains((event as PointerEvent).relatedTarget as Node | null))
    showToolbarHint(null);
});
toolbar.addEventListener("pointerdown", () => showToolbarHint(null));
document.addEventListener("focusin", (event) =>
  showToolbarHint(
    toolbar.contains(event.target as Node)
      ? (event.target as Element).closest("button")
      : null,
  ),
);
window.addEventListener("resize", () => showToolbarHint(null));
element("save-indicator").addEventListener("click", () => openMenu("archive"));
element("restart-flight").addEventListener("click", () => {
  void beginEncounter(() => galaxy.return_to_home()).catch((error) =>
    toast(String(error)),
  );
});
for (const [index, page] of facilityPages.entries()) {
  element(`facility-${index}`).addEventListener("click", () => {
    if (
      launched &&
      !paused &&
      menu.hidden &&
      hud.dock &&
      galaxy.dock_facility_at(
        facilityPositions[index][0],
        facilityPositions[index][1],
      ) === index
    )
      openMenu(page);
  });
}
element("pause-button").addEventListener("click", () => {
  if (!menu.hidden) void resume();
  else pause(false);
});
element("quick-settings").addEventListener("click", () => openMenu("settings"));
element("flight-settings").addEventListener("click", () =>
  openMenu("settings"),
);
element("map-button").addEventListener("click", () => {
  if (!menu.hidden && menuTab === "map") void resume();
  else openMenu("map");
});
element("settings-title").addEventListener("click", returnToTitle);
element("quick-title").addEventListener("click", returnToTitle);
element("return-title").addEventListener("click", returnToTitle);
element("death-title").addEventListener("click", returnToTitle);
element("quit-game").addEventListener("click", () => void quitGame());
element("retry").addEventListener("click", () => void resume());
function showDeath(): void {
  transitionEpoch++;
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
  audio.finishDeath();
  narrative.close();
  const target = element("run-summary");
  target.replaceChildren();
  const summary = hud.last_run;
  const report = document.createElement("p");
  report.textContent = summary
    ? "LAST RECORDED EXPEDITION"
    : "NO EXPEDITION RECORDED";
  target.append(report);
  if (summary)
    for (const line of [
      `${Math.round(summary.time)} SECONDS / ${Math.round(summary.distance_peak)} M FURTHEST FROM HAVEN`,
      `${summary.enemies} ENEMIES / ${summary.asteroids} ASTEROIDS / ${summary.champions} CHAMPIONS`,
      `BANKED POINTS / ${summary.points_gained.join(" / ")}`,
      `${summary.xp_gained} RAICHO XP RETAINED / ${summary.materials_gained} MATERIALS COLLECTED`,
      `CARRIED MATERIALS LOST / ${summary.materials_lost.map((n, tier) => `T${tier + 1} ${n}`).join(" / ")}`,
    ]) {
      const row = document.createElement("p");
      row.textContent = line;
      target.append(row);
    }
  const bank = document.createElement("p");
  bank.textContent = `DOCK STASH ${hud.stored_materials} MATERIALS / RAICHO LEVEL ${hud.ship_level}`;
  target.append(bank);
  save(galaxy.profile());
  pixelUi.invalidate();
  element("retry").focus();
}
function returnToTitle(): void {
  if (!galaxy) return;
  transitionEpoch++;
  preparingEncounter = null;
  // Returning ends an active encounter through the same Rust retreat rules.
  galaxy.return_to_home();
  hud = galaxy.hud();
  save(galaxy.profile());
  launched = false;
  narrative.close();
  dialoguePanel.close();
  debugConsole.close();
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
async function quitGame(): Promise<void> {
  if (launched) return;
  transitionEpoch++;
  if (!galaxy) {
    window.close();
    loadStatus.textContent = "Close this tab or app to quit.";
    pixelUi?.invalidate();
    return;
  }
  const button = element<HTMLButtonElement>("quit-game");
  button.disabled = true;
  try {
    if (!needsDroidCreation) await persist(galaxy.profile());
    await store.flush();
    audio.suspend();
    narrative.close();
    // Browsers allow a script-opened window to close; an ordinary browser tab
    // may stay open. Keep its saved title screen usable in that case.
    window.close();
    loadStatus.textContent = needsDroidCreation
      ? "Close this tab or app to quit."
      : "PROGRESS SAVED / close this tab or app to quit.";
  } catch (error) {
    loadStatus.textContent =
      "SAVE NEEDS ATTENTION / use Settings > Save & backup.";
    toast(String(error));
  } finally {
    button.disabled = false;
    pixelUi?.invalidate();
  }
}
element("dock-button").addEventListener("click", () => pause(true));
function interactSite(): void {
  if (paused || !menu.hidden || debugConsole.opened || hud.campaign.encounter)
    return;
  if (hud.campaign.arena) {
    const arena = hud.campaign.arena;
    const champion = hud.campaign.champions[arena.index];
    if (!champion) return;
    showConversation(
      {
        id: champion.id,
        speaker: champion.identity?.short_designation ?? champion.name,
        personality: champion.index,
        description: champion.introduction,
        status: champion.patch ? "completed" : "available",
        facts: [
          champion.patch
            ? "You earned my patch and blueprint. Craft my RAICHO at Haven."
            : "Win this duel to earn my arena patch and RAICHO blueprint.",
          "The trial boundary is marked. Retreat or RAICHO loss ends your attempt.",
        ],
      },
      arena.can_challenge ? "CHALLENGE" : undefined,
      arena.can_challenge ? () => beginChampion(arena.index) : undefined,
      champion.identity ?? undefined,
    );
    return;
  }
  if (hud.campaign.tournament_unlocked && hud.campaign.tournament_nearby) {
    showConversation(
      {
        id: "galaxy-royale",
        speaker: "ROYALE MARSHAL",
        personality: 1,
        description:
          "Welcome to Galaxy Royale. Rival champions from eight universes await. Every entrant meets every other entrant once.",
        status: hud.campaign.tournament_won ? "completed" : "available",
        facts: [
          "Win duels for three points. Finish first to unlock the Sovereign RAICHO and complete the campaign.",
          "NPC versus NPC results are seeded simulations. Your eight matches are real duels.",
        ],
      },
      hud.campaign.tournament_won
        ? "END CREDITS"
        : hud.campaign.tournament_complete
          ? "NEW SEASON"
          : "ENTER DUEL",
      hud.campaign.tournament_won ? openCredits : beginTournament,
    );
    return;
  }
  if (!hud.site) return;
  const site = hud.site;
  showConversation(
    {
      id: site.id,
      speaker: site.name,
      description: site.description,
      personality:
        [...site.id].reduce((sum, c) => sum + c.charCodeAt(0), 0) % 3,
      status: site.status as NarrativeContext["status"],
    },
    site.can_interact
      ? site.status === "ready"
        ? "CLAIM RELIC"
        : "ACCEPT EVENT"
      : undefined,
    site.can_interact ? commitSite : undefined,
  );
}
function showConversation(
  context: NarrativeContext,
  label?: string,
  action?: () => void | Promise<void>,
  appearance?: PortraitIdentity,
): void {
  const pages = narrative.open(context);
  const draw = (): void => {
    const page = narrative.current();
    if (!page) return;
    const index = pages.findIndex((candidate) => candidate.id === page.id);
    dialoguePanel.show({
      speaker: page.speaker,
      text: page.text,
      portrait: page.portrait,
      portraitIdentity: appearance,
      page: index,
      pageCount: pages.length,
      actionLabel: label,
      onAction: index === pages.length - 1 ? action : undefined,
      onPrevious:
        index > 0
          ? () => {
              narrative.previous();
              draw();
            }
          : undefined,
      onNext:
        index < pages.length - 1
          ? () => {
              narrative.next();
              draw();
            }
          : undefined,
      onReplay: settings.dialogue_voice
        ? () => {
            void narrative.replay().then(() => {
              if (["unavailable", "error"].includes(narrative.voiceStatus))
                toast("Voice unavailable; dialogue remains readable.");
            });
          }
        : undefined,
    });
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
    await persist(galaxy.profile());
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
  if (!menu.hidden && event.key === "Tab") {
    const controls = Array.from(
      menu.querySelectorAll<HTMLElement>("button,input,select,a[href]"),
    ).filter(
      (node) =>
        !node.closest("[hidden]") &&
        !(node instanceof HTMLButtonElement && node.disabled),
    );
    if (controls.length) {
      event.preventDefault();
      const current = controls.indexOf(document.activeElement as HTMLElement);
      const next =
        current < 0
          ? event.shiftKey
            ? controls.length - 1
            : 0
          : (current + (event.shiftKey ? -1 : 1) + controls.length) %
            controls.length;
      controls[next].focus();
    }
    return;
  }
  if (!launched) {
    if (event.key === "Escape" && beforeLaunchMenu) void resume();
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    if (event.repeat) return;
    if (!menu.hidden) void resume();
    else pause(false);
  }
  if (
    event.key.toLowerCase() === "m" &&
    !event.repeat &&
    !debugConsole.opened &&
    !dialoguePanel.opened &&
    !(
      event.target instanceof HTMLElement &&
      event.target.matches("input,textarea,select,[contenteditable]")
    )
  ) {
    event.preventDefault();
    if (!menu.hidden && menuTab === "map") void resume();
    else openMenu("map");
  }
  if (event.key.toLowerCase() === "e" && hud?.dock && input.enabled)
    pause(true);
  else if (
    event.key.toLowerCase() === "e" &&
    (hud?.site || hud?.campaign.arena || hud?.campaign.tournament_nearby) &&
    input.enabled
  )
    interactSite();
});
window.addEventListener("resize", resize);
document.addEventListener("visibilitychange", () => {
  if (!galaxy) return;
  if (document.hidden) {
    input?.reset();
    galaxy.cancel_course();
    destination = null;
    audio.suspend();
    narrative.pause();
    save(galaxy.profile());
  }
  syncFlightState();
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
  if (
    !launched ||
    !input.enabled ||
    debugConsole.opened ||
    dialoguePanel.opened
  )
    return;
  event.preventDefault();
  const factor =
    event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? innerHeight : 1;
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
element("import-label").addEventListener("click", () =>
  element<HTMLInputElement>("import-save").click(),
);
element<HTMLInputElement>("import-save").addEventListener(
  "change",
  async (event) => {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;
    let importEpoch: number | null = null;
    try {
      if (file.size > 65536) throw new Error("Save file is too large.");
      const value: unknown = JSON.parse(await file.text());
      if (!validProfile(value)) throw new Error("Unsupported or invalid save.");
      pause(false);
      importEpoch = ++transitionEpoch;
      preparingEncounter = importEpoch;
      syncFlightState();
      await store.replace(value);
      location.reload();
    } catch (error) {
      if (importEpoch !== null && preparingEncounter === importEpoch) {
        preparingEncounter = null;
        syncFlightState();
      }
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
