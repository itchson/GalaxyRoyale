interface ProfileBase {
  revision: number;
  seed: number;
  currency: number[];
  ranks: number[];
  unlocked: boolean[];
  kills: number[];
  boss_kills: number[];
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
export interface Profile extends ProfileBase {
  schema_version: 2;
  discoveries: string[];
  quests: Quest[];
  relics: string[];
}
export interface LegacyProfile extends ProfileBase {
  schema_version: 1;
}
export type SupportedProfile = Profile | LegacyProfile;
export interface Settings {
  master: number;
  music: number;
  sfx: number;
  shake: boolean;
  reduced_motion: boolean;
  cosmetic_density: "full" | "reduced";
}
export const defaultSettings: Settings = {
  master: 0.65,
  music: 0.45,
  sfx: 0.75,
  shake: true,
  reduced_motion: false,
  cosmetic_density: "full",
};
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
    (p.schema_version === 1 || p.schema_version === 2) &&
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
  return (
    Array.from(p.quests).every(validQuest) &&
    new Set(p.quests.map((quest) => quest.site_id)).size === p.quests.length
  );
}
/** Pure format migration; no currency, ranks, rewards or revision are changed. */
export function normalizeProfile(value: unknown): Profile | null {
  if (!validProfile(value)) return null;
  return {
    schema_version: 2,
    revision: value.revision,
    seed: value.seed,
    currency: [...value.currency],
    ranks: [...value.ranks],
    unlocked: [...value.unlocked],
    kills: [...value.kills],
    boss_kills: [...value.boss_kills],
    pity: [...value.pity],
    pending: [...value.pending],
    discoveries: value.schema_version === 2 ? [...value.discoveries] : [],
    quests:
      value.schema_version === 2
        ? value.quests.map((quest) => ({
            site_id: quest.site_id,
            event: quest.event,
            family: quest.family,
            target: quest.target,
            progress: quest.progress,
            completed: quest.completed,
          }))
        : [],
    relics: value.schema_version === 2 ? [...value.relics] : [],
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
      if (normalizedCurrent && current.schema_version === 1)
        store.put(normalizedCurrent, "current");
      if (normalizedPrevious && previous.schema_version === 1)
        store.put(normalizedPrevious, "previous");
      await done;
      if (normalizedCurrent) {
        this.latest = structuredClone(normalizedCurrent);
        if (current.schema_version === 1)
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
