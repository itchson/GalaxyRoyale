// Actual source: apps/game/src/audio-state.ts; SHA256 c3c9a3c5b34b1001e4d1dd1de531e70dbcd945826b31022e0619187d0ec1b76f
export const QUIET_STATE = {
    area: "haven",
    biome: 0,
    combat: 0,
    hostiles: 0,
    champion: null,
    boss: false,
    hull: 1,
    speed: 0,
    throttle: 0,
    engines: 0,
    corporation: null,
};
export const unit = (n) => Number.isFinite(n) ? Math.min(1, Math.max(0, n)) : 0;
export function cleanState(state) {
    const area = ["haven", "belt", "reach", "site", "arena"].includes(state.area)
        ? state.area
        : "haven";
    const champion = Number.isInteger(state.champion) &&
        state.champion !== null &&
        state.champion >= 0 &&
        state.champion < 8
        ? state.champion
        : null;
    return {
        area,
        biome: Number.isInteger(state.biome) && state.biome >= 0 && state.biome < 6
            ? state.biome
            : 0,
        combat: area === "haven" || area === "site" ? 0 : unit(state.combat),
        hostiles: Number.isFinite(state.hostiles)
            ? Math.min(256, Math.max(0, Math.floor(state.hostiles)))
            : 0,
        champion: area === "arena" ? champion : null,
        boss: state.boss === true,
        hull: unit(state.hull),
        speed: unit(state.speed),
        throttle: unit(state.throttle),
        engines: Number.isFinite(state.engines)
            ? Math.min(2, Math.max(0, Math.floor(state.engines)))
            : 0,
        corporation: state.corporation === "pan" ||
            state.corporation === "renko" ||
            state.corporation === "hugo"
            ? state.corporation
            : null,
    };
}
// Preserve identity contours and rests, transposed into the shared B-minor score.
// These authored variants remain independent of the renderer and beat tempo.
const ORIGINAL_IDENTITIES = [
    {
        name: "Haven / Return lights",
        lead: "triangle",
        arp: "sine",
        bass: "sine",
        order: [0, 2, 1, 3],
        motif: [
            69,
            null,
            null,
            null,
            72,
            null,
            null,
            null,
            76,
            null,
            72,
            null,
            69,
            null,
            null,
            null,
        ],
    },
    {
        name: "Amethyst / Glass tide",
        lead: "sine",
        arp: "triangle",
        bass: "triangle",
        order: [3, 1, 2, 0],
        motif: [
            81,
            null,
            79,
            null,
            null,
            76,
            null,
            72,
            76,
            null,
            null,
            79,
            null,
            76,
            null,
            null,
        ],
    },
    {
        name: "Aurora / Signal bloom",
        lead: "triangle",
        arp: "sine",
        bass: "triangle",
        order: [0, 1, 3, 2],
        motif: [
            72,
            null,
            76,
            79,
            null,
            null,
            81,
            null,
            79,
            null,
            76,
            null,
            72,
            null,
            74,
            null,
        ],
    },
    {
        name: "Iron / Foundry relay",
        lead: "square",
        arp: "triangle",
        bass: "square",
        order: [0, 0, 2, 1],
        motif: [
            69,
            null,
            null,
            69,
            76,
            null,
            72,
            null,
            69,
            null,
            67,
            null,
            64,
            null,
            67,
            null,
        ],
    },
    {
        name: "Radial / Isotope drift",
        lead: "sawtooth",
        arp: "sine",
        bass: "triangle",
        order: [1, 3, 0, 2],
        motif: [
            76,
            null,
            null,
            77,
            null,
            76,
            null,
            null,
            72,
            null,
            71,
            null,
            69,
            null,
            null,
            71,
        ],
    },
    {
        name: "Cryo / Distant ice",
        lead: "sine",
        arp: "sine",
        bass: "sine",
        order: [3, 2, 1, 0],
        motif: [
            81,
            null,
            null,
            null,
            79,
            null,
            null,
            null,
            76,
            null,
            null,
            72,
            null,
            null,
            76,
            null,
        ],
    },
    {
        name: "Lyra Voss / Needle dance",
        lead: "square",
        arp: "triangle",
        bass: "square",
        order: [0, 2, 3, 1],
        motif: [
            69,
            72,
            null,
            76,
            72,
            null,
            79,
            null,
            76,
            72,
            null,
            69,
            67,
            null,
            69,
            null,
        ],
    },
    {
        name: "Korr-9 / Broadside march",
        lead: "sawtooth",
        arp: "square",
        bass: "square",
        order: [0, 0, 1, 0],
        motif: [
            57,
            null,
            null,
            64,
            57,
            null,
            null,
            null,
            60,
            null,
            null,
            64,
            57,
            null,
            null,
            55,
        ],
    },
    {
        name: "Sable Myrr / Night pursuit",
        lead: "triangle",
        arp: "sine",
        bass: "triangle",
        order: [2, 0, 3, 1],
        motif: [
            76,
            null,
            72,
            null,
            null,
            71,
            null,
            69,
            null,
            76,
            null,
            79,
            76,
            null,
            71,
            null,
        ],
    },
    {
        name: "Xil / Prism lattice",
        lead: "square",
        arp: "triangle",
        bass: "sine",
        order: [0, 1, 2, 3],
        motif: [
            72,
            76,
            79,
            null,
            76,
            79,
            81,
            null,
            79,
            76,
            72,
            null,
            74,
            77,
            81,
            null,
        ],
    },
    {
        name: "Maera Quill / Solar wind",
        lead: "triangle",
        arp: "sine",
        bass: "triangle",
        order: [0, 3, 2, 1],
        motif: [
            69,
            null,
            72,
            76,
            null,
            79,
            null,
            81,
            79,
            null,
            76,
            72,
            null,
            74,
            76,
            null,
        ],
    },
    {
        name: "Orun Vale / Iron wake",
        lead: "sawtooth",
        arp: "triangle",
        bass: "square",
        order: [0, 2, 0, 1],
        motif: [
            57,
            null,
            60,
            null,
            64,
            null,
            null,
            67,
            64,
            null,
            60,
            null,
            57,
            null,
            55,
            null,
        ],
    },
    {
        name: "Vexa Null / Coil fugue",
        lead: "sine",
        arp: "square",
        bass: "triangle",
        order: [3, 0, 2, 1],
        motif: [
            69,
            71,
            72,
            null,
            76,
            null,
            77,
            76,
            null,
            72,
            71,
            null,
            67,
            null,
            69,
            null,
        ],
    },
    {
        name: "Atlas-0 / Sunspear",
        lead: "square",
        arp: "sine",
        bass: "square",
        order: [0, 3, 1, 2],
        motif: [
            81,
            null,
            null,
            80,
            null,
            76,
            null,
            72,
            76,
            null,
            null,
            80,
            83,
            null,
            81,
            null,
        ],
    },
    {
        name: "Expanse / Scouting signal",
        lead: "triangle",
        arp: "sine",
        bass: "sine",
        order: [2, 1, 0, 3],
        motif: [
            64,
            null,
            67,
            null,
            69,
            null,
            null,
            72,
            69,
            null,
            67,
            null,
            64,
            null,
            null,
            67,
        ],
    },
    {
        name: "PAN / Verdant lanterns",
        lead: "sine",
        arp: "sine",
        bass: "sine",
        order: [0, 2, 3, 1],
        motif: [
            69,
            null,
            null,
            76,
            null,
            null,
            72,
            null,
            71,
            null,
            null,
            69,
            null,
            64,
            null,
            null,
        ],
    },
    {
        name: "RENKO / Ember courtyard",
        lead: "triangle",
        arp: "sine",
        bass: "sine",
        order: [1, 0, 2, 3],
        motif: [
            64,
            null,
            69,
            null,
            null,
            72,
            null,
            null,
            76,
            null,
            null,
            74,
            72,
            null,
            69,
            null,
        ],
    },
    {
        name: "HUGO / Blue observatory",
        lead: "sine",
        arp: "triangle",
        bass: "sine",
        order: [3, 1, 2, 0],
        motif: [
            76,
            null,
            null,
            null,
            72,
            null,
            71,
            null,
            null,
            67,
            null,
            null,
            69,
            null,
            64,
            null,
        ],
    },
];
export const THEMES = ORIGINAL_IDENTITIES.map((theme) => ({
    ...theme,
    motif: theme.motif.map((note) => (note === null ? null : note + 2)),
}));
export function newMix() {
    return {
        themes: THEMES.map((_, i) => (i === 0 ? 1 : 0)),
        combat: 0,
        boss: 0,
        alarm: 0,
        engine: 0,
        space: 0.55,
    };
}
export function advanceMix(previous, raw, seconds) {
    const state = cleanState(raw);
    const dt = Number.isFinite(seconds)
        ? Math.min(0.25, Math.max(0, seconds))
        : 0;
    const theme = state.champion === null
        ? state.area === "haven"
            ? state.corporation === "pan"
                ? 15
                : state.corporation === "renko"
                    ? 16
                    : state.corporation === "hugo"
                        ? 17
                        : 0
            : state.biome === 0
                ? 14
                : state.biome
        : 6 + state.champion;
    const toward = (value, target, tau) => value + (target - value) * (1 - Math.exp(-dt / tau));
    const themes = previous.themes.map((weight, i) => toward(weight, i === theme ? 1 : 0, 1.4));
    const alarm = state.hull > 0 && state.hull <= 0.25
        ? 0.15 + 0.85 * unit((0.25 - state.hull) / 0.17)
        : 0;
    const engine = state.hull > 0
        ? ((state.throttle * 0.88 + state.speed * 0.12) * state.engines) / 2
        : 0;
    const boss = state.boss && state.hostiles > 0 && state.combat > 0 ? 1 : 0;
    return {
        themes,
        combat: toward(previous.combat, state.combat, state.combat > previous.combat ? 0.45 : 1.6),
        boss: toward(previous.boss, boss, boss > previous.boss ? 0.4 : 1.4),
        alarm: toward(previous.alarm, alarm, alarm > previous.alarm ? 0.12 : 0.25),
        engine: toward(previous.engine, engine, engine > previous.engine ? 0.08 : 0.16),
        space: toward(previous.space, state.area === "haven"
            ? 0.55
            : state.area === "site"
                ? 0.65
                : state.area === "belt"
                    ? 0.75
                    : 1, 1.1),
    };
}
/** Select at most two identities for new notes; bounded earlier note/pad tails finish naturally. */
export function audibleThemes(mix) {
    return mix.themes
        .map((weight, i) => [i, weight])
        .filter(([, weight]) => weight > 0.025)
        .sort((a, b) => b[1] - a[1] || a[0] - b[0])
        .slice(0, 2);
}
