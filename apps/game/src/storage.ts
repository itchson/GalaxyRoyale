interface ProfileBase {
  revision: number;
  seed: number;
  currency: number[];
  ranks: number[];
  unlocked: boolean[];
  kills: number[];
  boss_kills: number[];
  /** Absent in earlier schema3 checkpoints; aggregate defeats then belong to generation1. */
  boss_generations?: number[][];
  pity: number[];
  pending: boolean[];
}
export interface Quest {
  site_id: string;
  event: "courier" | "bounty";
  family: number;
  target: number;
  progress: number;
  completed: boolean;
}
export interface CampaignProgress {
  patches: boolean[];
  blueprints: boolean[];
  crafted: boolean[];
  selected_ship: number;
  royale_results: number[];
  tournament_attempt: number;
  victory_claimed: boolean;
  credits_seen: boolean;
  ship_xp: number[];
  cargo_materials: number;
  stored_materials: number;
  last_run: RunSummary | null;
}
export interface RunSummary {
  time: number;
  distance_peak: number;
  enemies: number;
  asteroids: number;
  champions: number;
  points_gained: number[];
  xp_gained: number;
  materials_gained: number;
}
export const emptyCampaign = (): CampaignProgress => ({
  patches: Array(8).fill(false),
  blueprints: Array(8).fill(false),
  crafted: Array(8).fill(false),
  selected_ship: 0,
  royale_results: Array(8).fill(0),
  tournament_attempt: 0,
  victory_claimed: false,
  credits_seen: false,
  ship_xp: Array(10).fill(0),
  cargo_materials: 0,
  stored_materials: 0,
  last_run: null,
});
function wonLeague(seed: number, campaign: CampaignProgress): boolean {
  const mix = (value: number): number => {
    value ^= value >>> 16; value = Math.imul(value, 0x7feb352d);
    value ^= value >>> 15; value = Math.imul(value, 0x846ca68b);
    return (value ^ (value >>> 16)) >>> 0;
  };
  const scores = Array<number>(9).fill(0);
  for (let a = 0; a < 8; a++) {
    for (let b = a + 1; b < 8; b++)
      scores[(mix(seed ^ (a * 7919) ^ (b * 104729) ^ 0x6e706373) & 1) === 0 ? a : b]! += 3;
    scores[campaign.royale_results[a] === 1 ? 8 : a]! += 3;
  }
  const tie = (entrant: number) => mix(seed ^ Math.imul(entrant, 0x9e3779b9) ^ 0x726f796c);
  return scores.every((score, entrant) => score < scores[8]! ||
    (score === scores[8]! && (tie(entrant) > tie(8) || entrant === 8)));
}
interface ExplorationProfile extends ProfileBase {
  discoveries: string[];
  quests: Quest[];
  relics: string[];
}
export interface Profile extends ExplorationProfile {
  schema_version: 3;
  boss_generations: number[][];
  campaign: CampaignProgress;
}
export interface PreviousProfile extends ExplorationProfile {
  schema_version: 2;
}
export interface LegacyProfile extends ProfileBase {
  schema_version: 1;
}
export type SupportedProfile = Profile | PreviousProfile | LegacyProfile;
export interface Settings {
  master: number;
  music: number;
  sfx: number;
  shake: boolean;
  reduced_motion: boolean;
  cosmetic_density: "full" | "reduced";
  dialogue_voice: boolean;
  voice_volume: number;
  zoom: number;
}
export const defaultSettings: Settings = {
  master: 0.65,
  music: 0.45,
  sfx: 0.75,
  shake: true,
  reduced_motion: false,
  cosmetic_density: "full",
  dialogue_voice: false,
  voice_volume: 0.7,
  zoom: 1,
};
function normalizedBossGenerations(p: Record<string, unknown>): number[][] | null {
  const totals = p.boss_kills as number[];
  if (!Object.hasOwn(p, "boss_generations"))
    return totals.map(total => [total, 0, 0, 0, 0]);
  const input = p.boss_generations;
  if (!Array.isArray(input) || input.length !== 3) return null;
  const result: number[][] = [];
  for (let family = 0; family < 3; family++) {
    const row: unknown = input[family];
    if (!Array.isArray(row) || row.length !== 5 ||
      !Array.from(row).every(v => typeof v === "number" && Number.isInteger(v) && v >= 0 && v <= 0xffffffff)) return null;
    const defeats = [...row] as number[];
    if (defeats.reduce((sum, value) => sum + value, 0) === 0 && totals[family]! > 0)
      defeats[0] = totals[family]!;
    if (Math.min(0xffffffff, defeats.reduce((sum, value) => sum + value, 0)) !== totals[family]) return null;
    for (let generation = 1; generation < 5; generation++)
      if (defeats[generation]! > Math.floor(defeats[generation - 1]! / 2)) return null;
    result.push(defeats);
  }
  return result;
}
export function validProfile(value: unknown): value is SupportedProfile {
  if (!value || typeof value !== "object") return false;
  const p = value as Record<string, unknown>;
  const integer = (v: unknown, max = 0xffffffff) =>
    typeof v === "number" && Number.isSafeInteger(v) && v >= 0 && v <= max;
  const nums = (key: string, length: number, max = 0xffffffff) =>
    Array.isArray(p[key]) &&
    p[key].length === length &&
    Array.from(p[key]).every((v: unknown) => integer(v, max));
  const bools = (key: string) =>
    Array.isArray(p[key]) &&
    p[key].length === 3 &&
    Array.from(p[key]).every((v: unknown) => typeof v === "boolean");
  if (!(
    (p.schema_version === 1 || p.schema_version === 2 || p.schema_version === 3) &&
    integer(p.revision, Number.MAX_SAFE_INTEGER) &&
    integer(p.seed) &&
    p.seed !== 0 &&
    nums("currency", 3) &&
    nums("ranks", 9, 5) &&
    nums("kills", 3) &&
    nums("boss_kills", 3) &&
    nums("pity", 3, 255) &&
    bools("unlocked") &&
    bools("pending")
  ))
    return false;
  if (!normalizedBossGenerations(p)) return false;
  const ranks = p.ranks as number[];
  if (
    !(p.unlocked as boolean[]).every((unlocked, i) =>
      unlocked ? ranks[i + 6] >= 1 : ranks[i + 6] === 0,
    )
  )
    return false;
  if (p.schema_version === 1) return true;
  const siteId = (v: unknown): v is string =>
    typeof v === "string" && /^-?\d+:-?\d+$/.test(v);
  const sites = (v: unknown) =>
    Array.isArray(v) &&
    v.length <= 256 &&
    Array.from(v).every(siteId) &&
    new Set(v).size === v.length;
  if (
    !sites(p.discoveries) ||
    !sites(p.relics) ||
    !Array.isArray(p.quests) ||
    p.quests.length > 256
  )
    return false;
  const validQuest = (value: unknown) => {
    if (!value || typeof value !== "object" || Array.isArray(value))
      return false;
    const quest = value as Record<string, unknown>;
    return (
      siteId(quest.site_id) &&
      (quest.event === "courier" || quest.event === "bounty") &&
      integer(quest.family, 2) &&
      integer(quest.target, 12) &&
      (quest.target as number) > 0 &&
      integer(quest.progress, quest.target as number) &&
      typeof quest.completed === "boolean"
    );
  };
  const explorationValid = (
    Array.from(p.quests).every(validQuest) &&
    new Set(p.quests.map((quest) => quest.site_id)).size === p.quests.length
  );
  if (!explorationValid || p.schema_version === 2) return explorationValid;
  if (!p.campaign || typeof p.campaign !== "object" || Array.isArray(p.campaign)) return false;
  const c = p.campaign as Record<string, unknown>;
  const flags = (key: string) => Array.isArray(c[key]) && c[key].length === 8 &&
    Array.from(c[key]).every(v => typeof v === "boolean");
  if (!flags("patches") || !flags("blueprints") || !flags("crafted") ||
    !integer(c.selected_ship, 9) || !integer(c.tournament_attempt) ||
    typeof c.victory_claimed !== "boolean" || typeof c.credits_seen !== "boolean" ||
    !Array.isArray(c.royale_results) || c.royale_results.length !== 8 ||
    !Array.from(c.royale_results).every(v => integer(v, 2))) return false;
  const patches = c.patches as boolean[], blueprints = c.blueprints as boolean[], crafted = c.crafted as boolean[];
  const unlocked = patches.every(Boolean);
  const results = c.royale_results as number[];
  if (!Array.isArray(c.ship_xp) || c.ship_xp.length !== 10 || !Array.from(c.ship_xp).every(v => integer(v)) ||
    !integer(c.cargo_materials, 16) || !integer(c.stored_materials)) return false;
  if (c.last_run !== null) {
    if (!c.last_run || typeof c.last_run !== "object" || Array.isArray(c.last_run)) return false;
    const run = c.last_run as Record<string, unknown>;
    if (typeof run.time !== "number" || !Number.isFinite(run.time) || run.time < 0 || run.time > 1e9 ||
      typeof run.distance_peak !== "number" || !Number.isFinite(run.distance_peak) || run.distance_peak < 0 || run.distance_peak > 1_500_000 ||
      !["enemies", "asteroids", "champions", "xp_gained", "materials_gained"].every(key => integer(run[key])) ||
      !Array.isArray(run.points_gained) || run.points_gained.length !== 3 || !Array.from(run.points_gained).every(v => integer(v))) return false;
  }
  return patches.every((patch, i) => patch === blueprints[i] && (!crafted[i] || blueprints[i])) &&
    (c.selected_ship === 0 || (c.selected_ship === 9 ? c.victory_claimed : crafted[(c.selected_ship as number) - 1])) &&
    (unlocked || results.every(v => v === 0)) &&
    (!c.victory_claimed || (unlocked && results.every(v => v > 0) && wonLeague(p.seed as number, c as unknown as CampaignProgress))) &&
    (!c.credits_seen || c.victory_claimed);
}
/** Pure format migration; no currency, ranks, rewards or revision are changed. */
export function normalizeProfile(value: unknown): Profile | null {
  if (!validProfile(value)) return null;
  return {
    schema_version: 3,
    revision: value.revision,
    seed: value.seed,
    currency: [...value.currency],
    ranks: [...value.ranks],
    unlocked: [...value.unlocked],
    kills: [...value.kills],
    boss_kills: [...value.boss_kills],
    boss_generations: normalizedBossGenerations(value as unknown as Record<string, unknown>)!,
    pity: [...value.pity],
    pending: [...value.pending],
    discoveries: value.schema_version >= 2 ? [...(value as ExplorationProfile).discoveries] : [],
    quests:
      value.schema_version >= 2
        ? (value as ExplorationProfile).quests.map((quest) => ({
            site_id: quest.site_id,
            event: quest.event,
            family: quest.family,
            target: quest.target,
            progress: quest.progress,
            completed: quest.completed,
          }))
        : [],
    relics: value.schema_version >= 2 ? [...(value as ExplorationProfile).relics] : [],
    campaign: value.schema_version === 3 ? structuredClone(value.campaign) : emptyCampaign(),
  };
}
function request<T>(r: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
}
function committed(tx: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onabort = () =>
      reject(tx.error ?? new Error("Save transaction aborted"));
    tx.onerror = () => reject(tx.error);
  });
}
export class SaveConflict extends Error {}
export class SaveStore {
  private db?: IDBDatabase;
  private queue: Promise<void> = Promise.resolve();
  private latest?: Profile;
  private imported = false;
  message = "Progress stays on this device.";
  get persistent(): boolean {
    return Boolean(this.db);
  }
  async open(): Promise<Profile | null> {
    this.db?.close();
    this.db = undefined;
    this.latest = undefined;
    this.imported = false;
    this.message = "Progress stays on this device.";
    let loaded: Profile | null = null;
    try {
      const r = indexedDB.open("galaxy-royale", 1);
      r.onupgradeneeded = () => {
        if (!r.result.objectStoreNames.contains("profile"))
          r.result.createObjectStore("profile");
      };
      this.db = await request(r);
      const tx = this.db.transaction("profile", "readwrite");
      const done = committed(tx);
      // Observe an early transaction failure even when one of the reads rejects first.
      void done.catch(() => {});
      const store = tx.objectStore("profile");
      const [current, previous] = await Promise.all([
        request(store.get("current")),
        request(store.get("previous")),
      ]);
      const normalizedCurrent = normalizeProfile(current);
      const normalizedPrevious = normalizeProfile(previous);
      loaded = normalizedCurrent ?? normalizedPrevious;
      const currentMigrated = normalizedCurrent && JSON.stringify(current) !== JSON.stringify(normalizedCurrent);
      const previousMigrated = normalizedPrevious && JSON.stringify(previous) !== JSON.stringify(normalizedPrevious);
      if (currentMigrated)
        store.put(normalizedCurrent, "current");
      if (previousMigrated)
        store.put(normalizedPrevious, "previous");
      await done;
      if (normalizedCurrent) {
        this.latest = structuredClone(normalizedCurrent);
        if (currentMigrated)
          this.message = "Save format upgraded. Your progression is unchanged.";
        return normalizedCurrent;
      }
      if (normalizedPrevious) {
        this.latest = structuredClone(normalizedPrevious);
        this.message = "Recovered your previous valid save.";
        return normalizedPrevious;
      }
      if (current || previous)
        this.message =
          "Saved data was invalid. A new profile is ready; export a backup after playing.";
    } catch {
      if (loaded) {
        this.latest = structuredClone(loaded);
        this.message =
          "Your progress loaded, but save migration could not be stored. Export a backup before closing.";
        return loaded;
      }
      this.message =
        "Storage is unavailable. This session is in memory; export your progress to keep it.";
    }
    return null;
  }
  save(profile: SupportedProfile): Promise<void> {
    const copy = normalizeProfile(profile);
    if (!copy) return Promise.reject(new Error("Invalid profile snapshot"));
    // The caller reloads after import. Old simulation snapshots must not replace the imported run.
    if (this.imported) return this.queue;
    this.queue = this.queue
      .catch(() => {})
      .then(async () => {
        if (this.latest && copy.revision <= this.latest.revision) return;
        if (!this.db) {
          this.latest = copy;
          return;
        }
        const tx = this.db.transaction("profile", "readwrite");
        const done = committed(tx);
        const store = tx.objectStore("profile");
        try {
          const persisted = normalizeProfile(
            await request(store.get("current")),
          );
          // Check inside the readwrite transaction so two tabs cannot roll back or fork one save.
          if (
            persisted &&
            (!this.latest ||
              JSON.stringify(persisted) !== JSON.stringify(this.latest))
          ) {
            throw new SaveConflict(
              "Progress changed in another session. Reload to use the latest save.",
            );
          }
          if (this.latest) store.put(this.latest, "previous");
          store.put(copy, "current");
          await done;
        } catch (error) {
          try {
            tx.abort();
          } catch {
            /* A completed or aborted transaction cannot be aborted again. */
          }
          await done.catch(() => {});
          throw error;
        }
        this.latest = copy;
      })
      .catch((error) => {
        this.message =
          error instanceof SaveConflict
            ? error.message
            : "Save write failed. Export your progress before closing.";
        throw error;
      });
    return this.queue;
  }
  replace(profile: SupportedProfile): Promise<void> {
    const copy = normalizeProfile(profile);
    if (!copy)
      return Promise.reject(
        new Error("This is not a supported Galaxy Royale save."),
      );
    if (!this.db)
      return Promise.reject(
        new Error(
          "Storage is unavailable; an imported save cannot be kept across reload. Enable storage and retry.",
        ),
      );
    if (this.imported)
      return Promise.reject(
        new Error("An imported save is already pending. Reload to continue."),
      );
    this.imported = true;
    this.queue = this.queue
      .catch(() => {})
      .then(async () => {
        const tx = this.db!.transaction("profile", "readwrite");
        const done = committed(tx);
        const store = tx.objectStore("profile");
        try {
          const persisted = normalizeProfile(
            await request(store.get("current")),
          );
          const previous = persisted ?? this.latest;
          if (previous) store.put(previous, "previous");
          store.put(copy, "current");
          await done;
        } catch (error) {
          try {
            tx.abort();
          } catch {
            /* The transaction may have already completed. */
          }
          await done.catch(() => {});
          throw error;
        }
        this.latest = copy;
        this.message = "Imported save is ready. Reload to continue.";
      })
      .catch((error) => {
        this.imported = false;
        this.message =
          "Import write failed. Your existing progress is unchanged.";
        throw error;
      });
    return this.queue;
  }
  flush(): Promise<void> {
    return this.queue.catch(() => {});
  }
}
export function loadSettings(): Settings {
  try {
    return normalizeSettings(
      JSON.parse(localStorage.getItem("galaxy-settings") ?? "null"),
    );
  } catch {
    return { ...defaultSettings };
  }
}
export function normalizeSettings(value: unknown): Settings {
  const s =
    value && typeof value === "object"
      ? (value as Record<string, unknown>)
      : {};
  const level = (n: unknown, fallback: number) =>
    typeof n === "number" && Number.isFinite(n)
      ? Math.max(0, Math.min(1, n))
      : fallback;
  return {
    master: level(s.master, defaultSettings.master),
    music: level(s.music, defaultSettings.music),
    sfx: level(s.sfx, defaultSettings.sfx),
    shake: typeof s.shake === "boolean" ? s.shake : defaultSettings.shake,
    reduced_motion:
      typeof s.reduced_motion === "boolean"
        ? s.reduced_motion
        : defaultSettings.reduced_motion,
    cosmetic_density:
      s.cosmetic_density === "full" || s.cosmetic_density === "reduced"
        ? s.cosmetic_density
        : defaultSettings.cosmetic_density,
    dialogue_voice: typeof s.dialogue_voice === "boolean" ? s.dialogue_voice : false,
    voice_volume: level(s.voice_volume, defaultSettings.voice_volume),
    zoom: typeof s.zoom === "number" && Number.isFinite(s.zoom) ? Math.max(0.6, Math.min(1.6, s.zoom)) : 1,
  };
}
export function saveSettings(settings: Settings): void {
  try {
    localStorage.setItem(
      "galaxy-settings",
      JSON.stringify(normalizeSettings(settings)),
    );
  } catch {
    /* Session controls still work when browser storage is disabled. */
  }
}
