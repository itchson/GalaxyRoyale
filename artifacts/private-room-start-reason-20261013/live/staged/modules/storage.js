// Actual source SHA256 7be13895761a2e14c21249473797956f0147ea58e3c761921daf6739e21f9dd8
const abilityCategories = [0, 1, 2, 0, 1, 2, 0, 0, 0, 0, 2, 1, 0, 0, 0, 1, 2];
const shipFamilies = [0, 0, 1, 2, 0, 0, 1, 2, 1];
function compatibleAbility(index, ship) {
    if (index >= 14)
        return ship === 9;
    return (index < 6 || ship === 9 || shipFamilies[ship] === shipFamilies[index - 5]);
}
const emptyLoadout = () => ({
    hardware: Array(6).fill(0),
    slots: [null, null, null],
});
export const emptyCampaign = () => ({
    patches: Array(8).fill(false),
    blueprints: Array(8).fill(false),
    crafted: Array(8).fill(false),
    selected_ship: 0,
    royale_results: Array(8).fill(0),
    tournament_attempt: 0,
    victory_claimed: false,
    credits_seen: false,
    ship_xp: Array(10).fill(0),
    cargo_materials: Array(5).fill(0),
    stored_materials: Array(5).fill(0),
    cargo_training_materials: Array(3).fill(0),
    stored_training_materials: Array(3).fill(0),
    components: Array(3).fill(0),
    abilities: Array.from({ length: 17 }, (_, i) => ({
        blueprint: i < 3,
        rank: 0,
    })),
    loadouts: Array.from({ length: 10 }, emptyLoadout),
    last_run: null,
});
function wonLeague(seed, campaign) {
    const mix = (value) => {
        value ^= value >>> 16;
        value = Math.imul(value, 0x7feb352d);
        value ^= value >>> 15;
        value = Math.imul(value, 0x846ca68b);
        return (value ^ (value >>> 16)) >>> 0;
    };
    const scores = Array(9).fill(0);
    for (let a = 0; a < 8; a++) {
        for (let b = a + 1; b < 8; b++)
            scores[(mix(seed ^ (a * 7919) ^ (b * 104729) ^ 0x6e706373) & 1) === 0 ? a : b] += 3;
        scores[campaign.royale_results[a] === 1 ? 8 : a] += 3;
    }
    const tie = (entrant) => mix(seed ^ Math.imul(entrant, 0x9e3779b9) ^ 0x726f796c);
    return scores.every((score, entrant) => score < scores[8] ||
        (score === scores[8] && (tie(entrant) > tie(8) || entrant === 8)));
}
export const defaultSettings = {
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
function normalizedBossGenerations(p) {
    const totals = p.boss_kills;
    if (!Object.hasOwn(p, "boss_generations"))
        return totals.map((total) => [total, 0, 0, 0, 0]);
    const input = p.boss_generations;
    if (!Array.isArray(input) || input.length !== 3)
        return null;
    const result = [];
    for (let family = 0; family < 3; family++) {
        const row = input[family];
        if (!Array.isArray(row) ||
            row.length !== 5 ||
            !Array.from(row).every((v) => typeof v === "number" &&
                Number.isInteger(v) &&
                v >= 0 &&
                v <= 0xffffffff))
            return null;
        const defeats = [...row];
        if (defeats.reduce((sum, value) => sum + value, 0) === 0 &&
            totals[family] > 0)
            defeats[0] = totals[family];
        if (Math.min(0xffffffff, defeats.reduce((sum, value) => sum + value, 0)) !== totals[family])
            return null;
        for (let generation = 1; generation < 5; generation++)
            if (defeats[generation] > Math.floor(defeats[generation - 1] / 2))
                return null;
        result.push(defeats);
    }
    return result;
}
export function validProfile(value) {
    if (!value || typeof value !== "object")
        return false;
    const p = value;
    const integer = (v, max = 0xffffffff) => typeof v === "number" && Number.isSafeInteger(v) && v >= 0 && v <= max;
    const nums = (key, length, max = 0xffffffff) => Array.isArray(p[key]) &&
        p[key].length === length &&
        Array.from(p[key]).every((v) => integer(v, max));
    const bools = (key) => Array.isArray(p[key]) &&
        p[key].length === 3 &&
        Array.from(p[key]).every((v) => typeof v === "boolean");
    if (!((p.schema_version === 1 ||
        p.schema_version === 2 ||
        p.schema_version === 3 ||
        p.schema_version === 4 ||
        p.schema_version === 5 ||
        p.schema_version === 6 ||
        p.schema_version === 7) &&
        integer(p.revision, Number.MAX_SAFE_INTEGER) &&
        integer(p.seed) &&
        p.seed !== 0 &&
        nums("currency", 3) &&
        nums("ranks", 9, 5) &&
        nums("kills", 3) &&
        nums("boss_kills", 3) &&
        nums("pity", 3, 255) &&
        bools("unlocked") &&
        bools("pending")))
        return false;
    if (!normalizedBossGenerations(p))
        return false;
    if (p.schema_version >= 5) {
        if (p.identity !== null) {
            if (!p.identity ||
                typeof p.identity !== "object" ||
                Array.isArray(p.identity))
                return false;
            const identity = p.identity;
            if (typeof identity.kind !== "string" ||
                !["dora", "doro", "talo"].includes(identity.kind) ||
                typeof identity.corporation !== "string" ||
                !["pan", "renko", "hugo"].includes(identity.corporation) ||
                typeof identity.sub_corporation !== "string" ||
                !["pan", "renko", "hugo"].includes(identity.sub_corporation) ||
                typeof identity.hex_id !== "string" ||
                !/^[0-9a-f]{16}$/.test(identity.hex_id))
                return false;
        }
        if (p.onboarding !== null) {
            if (p.identity === null ||
                !p.onboarding ||
                typeof p.onboarding !== "object" ||
                Array.isArray(p.onboarding))
                return false;
            const onboarding = p.onboarding;
            if (!integer(onboarding.flags, 127) || !integer(onboarding.stage, 6))
                return false;
            const flags = onboarding.flags;
            const required = [1, 2, 4, 8, 16, 96];
            let stage = 0;
            while (stage < 6 && (flags & required[stage]) === required[stage])
                stage++;
            if (onboarding.stage !== stage)
                return false;
            if (p.schema_version >= 6) {
                if (onboarding.version !== 2 ||
                    !integer(onboarding.asteroids_destroyed, 10) ||
                    !integer(onboarding.materials_collected, 10) ||
                    typeof onboarding.legacy_complete !== "boolean")
                    return false;
                const kills = onboarding.asteroids_destroyed;
                const collected = onboarding.materials_collected;
                if (onboarding.legacy_complete) {
                    if (flags !== 127 || kills !== 0 || collected !== 0)
                        return false;
                }
                else if (((flags & 4) !== 0) !== (kills === 10) ||
                    ((flags & 8) !== 0) !== (collected === 10) ||
                    (collected > 0 && kills < 10) ||
                    ((flags & 16) !== 0 && (flags & 8) === 0) ||
                    ((flags & 96) !== 0 && ((flags & 96) !== 96 || (flags & 16) === 0)))
                    return false;
            }
        }
    }
    const ranks = p.ranks;
    if (p.schema_version < 4 &&
        !p.unlocked.every((unlocked, i) => unlocked ? ranks[i + 6] >= 1 : ranks[i + 6] === 0))
        return false;
    if (p.schema_version === 1)
        return true;
    const siteId = (v) => typeof v === "string" && /^-?\d+:-?\d+$/.test(v);
    const sites = (v) => Array.isArray(v) &&
        v.length <= 256 &&
        Array.from(v).every(siteId) &&
        new Set(v).size === v.length;
    if (!sites(p.discoveries) ||
        !sites(p.relics) ||
        !Array.isArray(p.quests) ||
        p.quests.length > 256)
        return false;
    const validQuest = (value) => {
        if (!value || typeof value !== "object" || Array.isArray(value))
            return false;
        const quest = value;
        return (siteId(quest.site_id) &&
            (quest.event === "courier" || quest.event === "bounty") &&
            integer(quest.family, 2) &&
            integer(quest.target, 12) &&
            quest.target > 0 &&
            integer(quest.progress, quest.target) &&
            typeof quest.completed === "boolean");
    };
    const explorationValid = Array.from(p.quests).every(validQuest) &&
        new Set(p.quests.map((quest) => quest.site_id)).size === p.quests.length;
    if (!explorationValid || p.schema_version === 2)
        return explorationValid;
    if (!p.campaign ||
        typeof p.campaign !== "object" ||
        Array.isArray(p.campaign))
        return false;
    const c = p.campaign;
    if (p.schema_version === 7 &&
        (!Array.isArray(c.components) ||
            c.components.length !== 3 ||
            !Array.from(c.components).every((n) => integer(n, 99))))
        return false;
    const flags = (key) => Array.isArray(c[key]) &&
        c[key].length === 8 &&
        Array.from(c[key]).every((v) => typeof v === "boolean");
    if (!flags("patches") ||
        !flags("blueprints") ||
        !flags("crafted") ||
        !integer(c.selected_ship, 9) ||
        !integer(c.tournament_attempt) ||
        typeof c.victory_claimed !== "boolean" ||
        typeof c.credits_seen !== "boolean" ||
        !Array.isArray(c.royale_results) ||
        c.royale_results.length !== 8 ||
        !Array.from(c.royale_results).every((v) => integer(v, 2)))
        return false;
    const patches = c.patches, blueprints = c.blueprints, crafted = c.crafted;
    const unlocked = patches.every(Boolean);
    const results = c.royale_results;
    if (!Array.isArray(c.ship_xp) ||
        c.ship_xp.length !== 10 ||
        !Array.from(c.ship_xp).every((v) => integer(v)))
        return false;
    const tiers = (value) => Array.isArray(value) &&
        value.length === 5 &&
        Array.from(value).every((v) => integer(v));
    if (p.schema_version >= 4) {
        if (!tiers(c.cargo_materials) ||
            c.cargo_materials.reduce((sum, n) => sum + n, 0) > 16 ||
            !tiers(c.stored_materials) ||
            !Array.isArray(c.abilities) ||
            c.abilities.length !== 17 ||
            !Array.isArray(c.loadouts) ||
            c.loadouts.length !== 10)
            return false;
        if (p.schema_version >= 6) {
            const named = (value, total) => Array.isArray(value) &&
                value.length === 3 &&
                Array.from(value).every((n) => integer(n)) &&
                value.reduce((sum, n) => sum + n, 0) <= total;
            if (!named(c.cargo_training_materials, c.cargo_materials[0]) ||
                !named(c.stored_training_materials, c.stored_materials[0]))
                return false;
            const progress = p.onboarding;
            if (progress &&
                !progress.legacy_complete &&
                (progress.flags & 96) === 96) {
                const identity = p.identity;
                const index = { pan: 2, renko: 0, hugo: 1 }[identity.corporation];
                if (!c.abilities[index] ||
                    c.abilities[index].rank < 2)
                    return false;
            }
        }
        const abilities = c.abilities;
        if (!Array.from(abilities).every((state, i) => state &&
            typeof state === "object" &&
            !Array.isArray(state) &&
            typeof state.blueprint === "boolean" &&
            integer(state.rank, 5) &&
            (state.rank === 0 || state.blueprint) &&
            (i < 14 || state.rank === (c.victory_claimed ? 5 : 0)) &&
            (i >= 3 ||
                (p.schema_version >= 5 && p.identity !== null) ||
                state.blueprint) &&
            (i < 6 ||
                (i < 14
                    ? state.blueprint === patches[i - 6]
                    : state.blueprint === c.victory_claimed))))
            return false;
        const owned = (ship) => ship === 0 || (ship === 9 ? c.victory_claimed : crafted[ship - 1]);
        const loadouts = c.loadouts;
        if (!Array.from(loadouts).every((loadout, ship) => loadout &&
            typeof loadout === "object" &&
            !Array.isArray(loadout) &&
            Array.isArray(loadout.hardware) &&
            loadout.hardware.length === 6 &&
            Array.from(loadout.hardware).every((n) => integer(n, 5)) &&
            Array.isArray(loadout.slots) &&
            loadout.slots.length === 3 &&
            Array.from(loadout.slots).every((id, slot) => id === null ||
                (integer(id, 16) &&
                    abilityCategories[id] === slot &&
                    compatibleAbility(id, ship) &&
                    abilities[id].rank > 0)) &&
            (owned(ship) ||
                (loadout.hardware.every((n) => n === 0) &&
                    loadout.slots.every((id) => id === null)))))
            return false;
        if (!p.unlocked.every((unlocked, i) => unlocked === abilities[i + 3].blueprint) ||
            !ranks.every((rank, i) => rank ===
                (i < 6
                    ? loadouts[c.selected_ship].hardware[i]
                    : abilities[i - 3].rank)))
            return false;
    }
    else if (!integer(c.cargo_materials, 16) || !integer(c.stored_materials))
        return false;
    if (c.last_run !== null) {
        if (!c.last_run ||
            typeof c.last_run !== "object" ||
            Array.isArray(c.last_run))
            return false;
        const run = c.last_run;
        if (typeof run.time !== "number" ||
            !Number.isFinite(run.time) ||
            run.time < 0 ||
            run.time > 1e9 ||
            typeof run.distance_peak !== "number" ||
            !Number.isFinite(run.distance_peak) ||
            run.distance_peak < 0 ||
            run.distance_peak > 1_500_000 ||
            ![
                "enemies",
                "asteroids",
                "champions",
                "xp_gained",
                "materials_gained",
            ].every((key) => integer(run[key])) ||
            !Array.isArray(run.points_gained) ||
            run.points_gained.length !== 3 ||
            !Array.from(run.points_gained).every((v) => integer(v)) ||
            (p.schema_version >= 4 && !tiers(run.materials_lost)))
            return false;
    }
    return (patches.every((patch, i) => patch === blueprints[i] && (!crafted[i] || blueprints[i])) &&
        (c.selected_ship === 0 ||
            (c.selected_ship === 9
                ? c.victory_claimed
                : crafted[c.selected_ship - 1])) &&
        (unlocked || results.every((v) => v === 0)) &&
        (!c.victory_claimed ||
            (unlocked &&
                results.every((v) => v > 0) &&
                wonLeague(p.seed, c))) &&
        (!c.credits_seen || c.victory_claimed));
}
/** Pure format migration. Live cargo is preserved; Game owns run abandonment. */
export function normalizeProfile(value) {
    if (!validProfile(value))
        return null;
    const campaign = migrateCampaign(value);
    return {
        schema_version: 7,
        identity: (value.schema_version === 5 ||
            value.schema_version === 6 ||
            value.schema_version === 7) &&
            value.identity
            ? { ...value.identity }
            : null,
        onboarding: (value.schema_version === 6 || value.schema_version === 7) &&
            value.onboarding
            ? { ...value.onboarding }
            : value.schema_version === 5 && value.onboarding
                ? {
                    version: 2,
                    flags: value.onboarding.flags === 127
                        ? 127
                        : value.onboarding.flags & 3,
                    stage: value.onboarding.flags === 127
                        ? 6
                        : (value.onboarding.flags & 1) === 0
                            ? 0
                            : (value.onboarding.flags & 2) === 0
                                ? 1
                                : 2,
                    asteroids_destroyed: 0,
                    materials_collected: 0,
                    legacy_complete: value.onboarding.flags === 127,
                }
                : null,
        revision: value.revision,
        seed: value.seed,
        currency: [...value.currency],
        ranks: [
            ...campaign.loadouts[campaign.selected_ship].hardware,
            ...campaign.abilities.slice(3, 6).map((a) => a.rank),
        ],
        unlocked: campaign.abilities.slice(3, 6).map((a) => a.blueprint),
        kills: [...value.kills],
        boss_kills: [...value.boss_kills],
        boss_generations: normalizedBossGenerations(value),
        pity: [...value.pity],
        pending: [...value.pending],
        discoveries: value.schema_version >= 2
            ? [...value.discoveries]
            : [],
        quests: value.schema_version >= 2
            ? value.quests.map((quest) => ({
                site_id: quest.site_id,
                event: quest.event,
                family: quest.family,
                target: quest.target,
                progress: quest.progress,
                completed: quest.completed,
            }))
            : [],
        relics: value.schema_version >= 2
            ? [...value.relics]
            : [],
        campaign,
    };
}
function migrateCampaign(value) {
    const campaign = emptyCampaign();
    if (value.schema_version === 3 ||
        value.schema_version === 4 ||
        value.schema_version === 5 ||
        value.schema_version === 6 ||
        value.schema_version === 7) {
        const previous = value.campaign;
        // Copy only schema-owned fields: an older backup may contain unrelated keys.
        campaign.patches = [...previous.patches];
        campaign.blueprints = [...previous.blueprints];
        campaign.crafted = [...previous.crafted];
        campaign.selected_ship = previous.selected_ship;
        campaign.royale_results = [...previous.royale_results];
        campaign.tournament_attempt = previous.tournament_attempt;
        campaign.victory_claimed = previous.victory_claimed;
        campaign.credits_seen = previous.credits_seen;
        campaign.ship_xp = [...previous.ship_xp];
        const run = previous.last_run;
        campaign.last_run = run
            ? {
                time: run.time,
                distance_peak: run.distance_peak,
                enemies: run.enemies,
                asteroids: run.asteroids,
                champions: run.champions,
                points_gained: [...run.points_gained],
                xp_gained: run.xp_gained,
                materials_gained: run.materials_gained,
                materials_lost: value.schema_version === 4 ||
                    value.schema_version === 5 ||
                    value.schema_version === 6 ||
                    value.schema_version === 7
                    ? [...value.campaign.last_run.materials_lost]
                    : Array(5).fill(0),
            }
            : null;
    }
    if (value.schema_version === 4 ||
        value.schema_version === 5 ||
        value.schema_version === 6 ||
        value.schema_version === 7) {
        campaign.cargo_materials = [...value.campaign.cargo_materials];
        campaign.stored_materials = [...value.campaign.stored_materials];
        if (value.schema_version === 6 || value.schema_version === 7) {
            campaign.cargo_training_materials = [
                ...value.campaign.cargo_training_materials,
            ];
            campaign.stored_training_materials = [
                ...value.campaign.stored_training_materials,
            ];
        }
        if (value.schema_version === 7)
            campaign.components = [...value.campaign.components];
        campaign.abilities = value.campaign.abilities.map(({ blueprint, rank }) => ({ blueprint, rank }));
        campaign.loadouts = value.campaign.loadouts.map(({ hardware, slots }) => ({
            hardware: [...hardware],
            slots: [...slots],
        }));
        return campaign;
    }
    if (value.schema_version === 3) {
        const previous = value.campaign;
        campaign.cargo_materials = Array(5).fill(0);
        campaign.stored_materials = [
            Math.min(0xffffffff, previous.stored_materials + previous.cargo_materials),
            0,
            0,
            0,
            0,
        ];
    }
    campaign.loadouts[campaign.selected_ship].hardware = [
        ...value.ranks.slice(0, 6),
    ];
    for (let family = 0; family < 3; family++)
        if (value.unlocked[family])
            campaign.abilities[family + 3] = {
                blueprint: true,
                rank: Math.max(1, value.ranks[family + 6]),
            };
    for (let champion = 0; champion < 8; champion++)
        campaign.abilities[champion + 6] = {
            blueprint: campaign.blueprints[champion],
            rank: campaign.crafted[champion] ? 1 : 0,
        };
    if (campaign.victory_claimed)
        for (let id = 14; id < 17; id++)
            campaign.abilities[id] = { blueprint: true, rank: 5 };
    for (let ship = 0; ship < 10; ship++) {
        if (ship !== 0 &&
            !(ship === 9 ? campaign.victory_claimed : campaign.crafted[ship - 1]))
            continue;
        for (let family = 0; family < 3; family++)
            if (campaign.abilities[family + 3].rank > 0)
                campaign.loadouts[ship].slots[family] = family + 3;
        if (ship > 0 && ship < 9) {
            const id = ship + 5;
            campaign.loadouts[ship].slots[abilityCategories[id]] = id;
        }
        if (ship === 9)
            campaign.loadouts[ship].slots = [14, 15, 16];
    }
    return campaign;
}
function request(r) {
    return new Promise((resolve, reject) => {
        r.onsuccess = () => resolve(r.result);
        r.onerror = () => reject(r.error);
    });
}
function committed(tx) {
    return new Promise((resolve, reject) => {
        tx.oncomplete = () => resolve();
        tx.onabort = () => reject(tx.error ?? new Error("Save transaction aborted"));
        tx.onerror = () => reject(tx.error);
    });
}
export class SaveConflict extends Error {
}
export class SaveStore {
    db;
    queue = Promise.resolve();
    latest;
    imported = false;
    invalidated = false;
    message = "Progress stays on this device.";
    get persistent() {
        return Boolean(this.db);
    }
    async open(onStatus = () => { }) {
        this.db?.close();
        this.db = undefined;
        this.latest = undefined;
        this.imported = false;
        this.invalidated = false;
        this.message = "Progress stays on this device.";
        let loaded = null;
        try {
            // A database version barrier prevents cached schema-six writers from
            // treating the new format as corrupt and overwriting it with an old run.
            const r = indexedDB.open("galaxy-royale", 5);
            r.onblocked = () => {
                this.message =
                    "Close other Galaxy Royale tabs to upgrade your save safely.";
                onStatus(this.message);
            };
            r.onupgradeneeded = () => {
                if (!r.result.objectStoreNames.contains("profile"))
                    r.result.createObjectStore("profile");
            };
            this.db = await request(r);
            this.db.onversionchange = () => {
                this.db?.close();
                this.db = undefined;
                this.invalidated = true;
                this.message =
                    "The save format changed in another tab. Reload before continuing.";
                onStatus(this.message);
            };
            const tx = this.db.transaction("profile", "readwrite");
            const done = committed(tx);
            // Observe an early transaction failure even when one of the reads rejects first.
            void done.catch(() => { });
            const store = tx.objectStore("profile");
            const [current, previous] = await Promise.all([
                request(store.get("current")),
                request(store.get("previous")),
            ]);
            const normalizedCurrent = normalizeProfile(current);
            const normalizedPrevious = normalizeProfile(previous);
            loaded = normalizedCurrent ?? normalizedPrevious;
            const currentMigrated = normalizedCurrent &&
                JSON.stringify(current) !== JSON.stringify(normalizedCurrent);
            const previousMigrated = normalizedPrevious &&
                JSON.stringify(previous) !== JSON.stringify(normalizedPrevious);
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
        }
        catch (error) {
            if (error instanceof DOMException && error.name === "VersionError") {
                this.invalidated = true;
                this.message =
                    "This save needs a newer game version. Reload or update Galaxy Royale.";
                throw new SaveConflict(this.message);
            }
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
    save(profile) {
        if (this.invalidated)
            return Promise.reject(new SaveConflict(this.message));
        const copy = normalizeProfile(profile);
        if (!copy)
            return Promise.reject(new Error("Invalid profile snapshot"));
        // The caller reloads after import. Old simulation snapshots must not replace the imported run.
        if (this.imported)
            return this.queue;
        this.queue = this.queue
            .catch(() => { })
            .then(async () => {
            if (this.invalidated)
                throw new SaveConflict(this.message);
            if (this.latest && copy.revision <= this.latest.revision)
                return;
            if (!this.db) {
                this.latest = copy;
                return;
            }
            const tx = this.db.transaction("profile", "readwrite");
            const done = committed(tx);
            const store = tx.objectStore("profile");
            try {
                const persisted = normalizeProfile(await request(store.get("current")));
                // Check inside the readwrite transaction so two tabs cannot roll back or fork one save.
                if (persisted &&
                    (!this.latest ||
                        JSON.stringify(persisted) !== JSON.stringify(this.latest))) {
                    throw new SaveConflict("Progress changed in another session. Reload to use the latest save.");
                }
                if (this.latest)
                    store.put(this.latest, "previous");
                store.put(copy, "current");
                await done;
            }
            catch (error) {
                try {
                    tx.abort();
                }
                catch {
                    /* A completed or aborted transaction cannot be aborted again. */
                }
                await done.catch(() => { });
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
    replace(profile) {
        if (this.invalidated)
            return Promise.reject(new SaveConflict(this.message));
        const copy = normalizeProfile(profile);
        if (!copy)
            return Promise.reject(new Error("This is not a supported Galaxy Royale save."));
        if (!this.db)
            return Promise.reject(new Error("Storage is unavailable; an imported save cannot be kept across reload. Enable storage and retry."));
        if (this.imported)
            return Promise.reject(new Error("An imported save is already pending. Reload to continue."));
        this.imported = true;
        this.queue = this.queue
            .catch(() => { })
            .then(async () => {
            if (this.invalidated)
                throw new SaveConflict(this.message);
            const tx = this.db.transaction("profile", "readwrite");
            const done = committed(tx);
            const store = tx.objectStore("profile");
            try {
                const persisted = normalizeProfile(await request(store.get("current")));
                const previous = persisted ?? this.latest;
                if (previous)
                    store.put(previous, "previous");
                store.put(copy, "current");
                await done;
            }
            catch (error) {
                try {
                    tx.abort();
                }
                catch {
                    /* The transaction may have already completed. */
                }
                await done.catch(() => { });
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
    flush() {
        return this.queue.catch(() => { });
    }
}
export function loadSettings() {
    try {
        return normalizeSettings(JSON.parse(localStorage.getItem("galaxy-settings") ?? "null"));
    }
    catch {
        return { ...defaultSettings };
    }
}
export function normalizeSettings(value) {
    const s = value && typeof value === "object"
        ? value
        : {};
    const level = (n, fallback) => typeof n === "number" && Number.isFinite(n)
        ? Math.max(0, Math.min(1, n))
        : fallback;
    return {
        master: level(s.master, defaultSettings.master),
        music: level(s.music, defaultSettings.music),
        sfx: level(s.sfx, defaultSettings.sfx),
        shake: typeof s.shake === "boolean" ? s.shake : defaultSettings.shake,
        reduced_motion: typeof s.reduced_motion === "boolean"
            ? s.reduced_motion
            : defaultSettings.reduced_motion,
        cosmetic_density: s.cosmetic_density === "full" || s.cosmetic_density === "reduced"
            ? s.cosmetic_density
            : defaultSettings.cosmetic_density,
        dialogue_voice: typeof s.dialogue_voice === "boolean" ? s.dialogue_voice : false,
        voice_volume: level(s.voice_volume, defaultSettings.voice_volume),
        zoom: typeof s.zoom === "number" && Number.isFinite(s.zoom)
            ? Math.max(0.6, Math.min(1.6, s.zoom))
            : 1,
    };
}
export function saveSettings(settings) {
    try {
        localStorage.setItem("galaxy-settings", JSON.stringify(normalizeSettings(settings)));
    }
    catch {
        /* Session controls still work when browser storage is disabled. */
    }
}
