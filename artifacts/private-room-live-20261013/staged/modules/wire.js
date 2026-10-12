// Actual source SHA256 a4d1c4ac96f7dcc02c035d8c078db7c65af79dfa2c3d6265a724f565256eb6a4
/** Transport bounds only. Rust remains authoritative for accepted game payloads. */
export const PROTOCOL = "galaxy-coop-1";
export const CHANNEL = "galaxy-coop-1";
export const WIRE_VERSION = 1;
export const MAX_FRAME_BYTES = 65_536;
export const MAX_BUFFERED_BYTES = 262_144;
export const LOW_BUFFERED_BYTES = 65_536;
export class TransportError extends Error {
    code;
    constructor(code) {
        super(code);
        this.code = code;
        this.name = "TransportError";
    }
}
export function fail(code) {
    throw new TransportError(code);
}
export const id = (value) => typeof value === "string" && /^[0-9a-f]{32}$/.test(value);
export const uint = (value) => Number.isInteger(value) && Number(value) >= 0 && Number(value) <= 0xffff_ffff;
export function record(value, keys) {
    if (!value ||
        typeof value !== "object" ||
        Array.isArray(value) ||
        ![Object.prototype, null].includes(Object.getPrototypeOf(value)) ||
        (keys && Object.keys(value).some((key) => !keys.includes(key))))
        fail("invalid_object");
    return value;
}
/** Reject nonfinite/deep/unbounded trees and explicit persisted-profile containers. */
export function json(value) {
    let nodes = 0;
    const visit = (v, depth) => {
        if (++nodes > 16_384 || depth > 12)
            fail("payload_bound");
        if (v === null || typeof v === "boolean")
            return;
        if (typeof v === "number") {
            if (!Number.isFinite(v))
                fail("nonfinite_payload");
            return;
        }
        if (typeof v === "string") {
            if (v.length > 16_384)
                fail("string_bound");
            return;
        }
        if (Array.isArray(v)) {
            if (v.length > 1_024)
                fail("array_bound");
            for (const x of v)
                visit(x, depth + 1);
            return;
        }
        const o = record(v);
        if (Object.keys(o).length > 64)
            fail("object_bound");
        for (const [key, x] of Object.entries(o)) {
            if (key.length > 64 ||
                ["__proto__", "constructor", "prototype", "profile", "save"].includes(key))
                fail("forbidden_payload_key");
            visit(x, depth + 1);
        }
    };
    visit(value, 0);
}
export function spec(value) {
    const o = record(value, ["local_seed", "identity", "owned_loadout"]);
    if (!uint(o.local_seed) || o.local_seed === 0)
        fail("invalid_local_seed");
    if (o.owned_loadout !== undefined)
        ownedLoadout(o.owned_loadout);
    if (o.identity === null)
        return;
    const i = record(o.identity, [
        "kind",
        "corporation",
        "sub_corporation",
        "hex_id",
    ]);
    if (!["dora", "doro", "talo"].includes(String(i.kind)) ||
        !["pan", "renko", "hugo"].includes(String(i.corporation)) ||
        !["pan", "renko", "hugo"].includes(String(i.sub_corporation)) ||
        typeof i.hex_id !== "string" ||
        !/^[0-9a-f]{16}$/.test(i.hex_id))
        fail("invalid_identity");
}
/** Shape bounds only. Registry, ownership declarations and compatibility are native checks. */
export function ownedLoadout(value) {
    const o = record(value, [
        "version",
        "selected_ship",
        "owned_ships",
        "patches",
        "royal_victory",
        "ship_xp",
        "hardware",
        "slots",
        "trained_starter",
        "relic_count",
        "echo_evolution",
        "nova_evolution",
    ]);
    const bounded = (n, min, max) => uint(n) && Number(n) >= min && Number(n) <= max;
    if (Object.keys(o).length !==
        10 +
            Number(Object.hasOwn(o, "echo_evolution")) +
            Number(Object.hasOwn(o, "nova_evolution")) ||
        o.version !== 1 ||
        !bounded(o.selected_ship, 0, 9) ||
        !bounded(o.owned_ships, 1, 1023) ||
        !bounded(o.patches, 0, 255) ||
        typeof o.royal_victory !== "boolean" ||
        !uint(o.ship_xp) ||
        typeof o.trained_starter !== "boolean" ||
        !bounded(o.relic_count, 0, 20) ||
        !Array.isArray(o.hardware) ||
        o.hardware.length !== 6 ||
        o.hardware.some((n) => !bounded(n, 0, 5)) ||
        !Array.isArray(o.slots) ||
        o.slots.length !== 3)
        fail("invalid_owned_loadout");
    for (const slot of o.slots) {
        if (slot === null)
            continue;
        const s = record(slot, ["index", "rank"]);
        if (Object.keys(s).length !== 2 ||
            !bounded(s.index, 0, 255) ||
            !bounded(s.rank, 1, 5))
            fail("invalid_owned_loadout");
    }
    if (Object.hasOwn(o, "echo_evolution") &&
        (!bounded(o.echo_evolution, 1, 5) ||
            o.slots[0] === null ||
            record(o.slots[0]).index !== 3))
        fail("invalid_owned_loadout");
    if (Object.hasOwn(o, "nova_evolution") &&
        (!bounded(o.nova_evolution, 1, 5) ||
            o.slots[1] === null ||
            record(o.slots[1]).index !== 4))
        fail("invalid_owned_loadout");
}
function input(value) {
    const o = record(value, ["epoch", "seq", "input"]);
    if (!uint(o.seq) || !uint(o.epoch))
        fail("invalid_input_seq");
    const i = record(o.input, [
        "joystick",
        "destination",
        "aim",
        "thrust",
        "fire",
    ]);
    if (typeof i.fire !== "boolean")
        fail("invalid_input");
    for (const key of ["joystick", "destination", "aim", "thrust"]) {
        const v = i[key];
        if (v === null || v === undefined)
            continue;
        if (!Array.isArray(v) ||
            v.length !== 2 ||
            v.some((n) => typeof n !== "number" ||
                !Number.isFinite(n) ||
                Math.abs(n) > (key === "joystick" ? 1 : 1_000_000)))
            fail("invalid_input");
    }
}
export function validatePayload(kind, payload) {
    json(payload);
    if (kind === "hello") {
        const o = record(payload, ["protocol", "peer_id"]);
        if (o.protocol !== PROTOCOL || !id(o.peer_id))
            fail("invalid_hello");
    }
    else if (kind === "join")
        spec(payload);
    else if (kind === "input")
        input(payload);
    else if (kind === "admission") {
        const o = record(payload, ["accepted", "slot", "epoch", "reason"]);
        if (typeof o.accepted !== "boolean" ||
            (o.accepted &&
                (!uint(o.epoch) ||
                    !Number.isInteger(o.slot) ||
                    Number(o.slot) < 1 ||
                    Number(o.slot) > 7)) ||
            (!o.accepted && (o.slot !== undefined || o.epoch !== undefined)) ||
            (o.reason !== undefined &&
                (typeof o.reason !== "string" || o.reason.length > 128)))
            fail("invalid_admission");
    }
    else if (kind === "snapshot")
        snapshot(payload);
    else if (kind === "publication")
        publication(payload);
    else if (kind === "publication-end" || kind === "publication-ack") {
        const p = record(payload, ["acquisition", "revision"]);
        if (!uint(p.acquisition) || p.acquisition === 0 || !uint(p.revision))
            fail("invalid_publication_token");
    }
    else if (kind === "body-query") {
        const q = record(payload, ["kind", "expected_revision", "offset", "id"]);
        if (!uint(q.expected_revision) ||
            !uint(q.offset) ||
            !["entities", "body"].includes(String(q.kind)) ||
            (q.kind === "body" ? !entityId(q.id) : q.id !== undefined))
            fail("invalid_body_query");
    }
    else if (kind === "body")
        page(payload);
    else {
        const n = record(payload, ["code", "ended"]);
        if (typeof n.code !== "string" ||
            !/^[a-z0-9_-]{1,64}$/.test(n.code) ||
            (n.ended !== undefined && typeof n.ended !== "boolean"))
            fail("invalid_notice");
    }
}
/** Exact native capture header. Cells follow in bounded native page messages. */
export function publication(value) {
    const p = record(value, ["acquisition", "snapshot", "counts"]);
    if (!uint(p.acquisition) || p.acquisition === 0)
        fail("invalid_publication_token");
    snapshot(p.snapshot);
    const c = record(p.counts, [
        "entities",
        "occupied_cells",
        "storage_cells",
        "entity_pages",
        "body_pages",
    ]);
    if (Object.keys(c).length !== 5 ||
        Object.values(c).some((x) => !uint(x)) ||
        Number(c.entities) > 512 ||
        Number(c.storage_cells) > 1_048_576 ||
        Number(c.occupied_cells) > Number(c.storage_cells) ||
        c.entity_pages !== Math.ceil(Number(c.entities) / 64) ||
        Number(c.body_pages) > 1_536 ||
        (c.entities === 0 && (c.storage_cells !== 0 || c.occupied_cells !== 0)))
        fail("invalid_publication_counts");
}
const entityId = (v) => typeof v === "string" && /^[0-9a-f]{16}$/.test(v);
function dimensions(o) {
    if (!Number.isInteger(o.width) ||
        Number(o.width) < 1 ||
        Number(o.width) > 512 ||
        !Number.isInteger(o.height) ||
        Number(o.height) < 1 ||
        Number(o.height) > 512 ||
        !uint(o.occupied) ||
        Number(o.occupied) > Number(o.width) * Number(o.height))
        fail("invalid_dimensions");
}
function page(value) {
    const o = record(value);
    if (!id(o.session) ||
        !uint(o.tick) ||
        !uint(o.revision) ||
        !uint(o.offset) ||
        (o.next_offset !== null &&
            (!uint(o.next_offset) || o.next_offset <= Number(o.offset))))
        fail("invalid_page");
    if (Array.isArray(o.entities)) {
        record(o, [
            "session",
            "tick",
            "revision",
            "offset",
            "next_offset",
            "entities",
        ]);
        if (o.entities.length > 64)
            fail("entity_page_bound");
        const seen = new Set();
        for (const value of o.entities) {
            const e = record(value, [
                "id",
                "kind",
                "participant",
                "position",
                "previous",
                "velocity",
                "heading",
                "previous_heading",
                "scale",
                "radius",
                "hull",
                "max_hull",
                "width",
                "height",
                "occupied",
                "body_signature",
            ]);
            if (!entityId(e.id) ||
                seen.has(e.id) ||
                !["player", "asteroid", "enemy", "boss", "segment"].includes(String(e.kind)) ||
                (e.participant !== null && !id(e.participant)))
                fail("invalid_entity");
            seen.add(e.id);
            if (e.body_signature !== undefined &&
                (typeof e.body_signature !== "string" ||
                    !/^[0-9a-f]{16}$/.test(e.body_signature)))
                fail("invalid_body_signature");
            dimensions(e);
            for (const key of ["position", "previous", "velocity"])
                vector(e[key]);
            for (const key of ["heading", "previous_heading"])
                finite(e[key], -1_000_000, 1_000_000);
            finite(e.scale, Number.MIN_VALUE, 100);
            finite(e.radius, 0, 10_000);
            finite(e.hull, 0, 100_000);
            finite(e.max_hull, 0, 100_000);
            // Existing Segment metadata has surviving health but no stored maximum.
            // Zero is the native unknown-maximum sentinel only for this entity kind.
            if (Number(e.hull) > Number(e.max_hull) &&
                !(e.kind === "segment" && e.max_hull === 0))
                fail("invalid_vitals");
        }
    }
    else {
        record(o, [
            "session",
            "tick",
            "revision",
            "id",
            "width",
            "height",
            "occupied",
            "offset",
            "next_offset",
            "cells",
        ]);
        dimensions(o);
        if (!entityId(o.id) || !Array.isArray(o.cells) || o.cells.length > 1_024)
            fail("body_page_bound");
        const seen = new Set();
        for (const cell of o.cells) {
            if (!Array.isArray(cell) ||
                cell.length !== 2 ||
                !uint(cell[0]) ||
                !uint(cell[1]) ||
                cell[0] >= Number(o.width) * Number(o.height) ||
                seen.has(cell[0]))
                fail("invalid_cell");
            seen.add(cell[0]);
        }
    }
}
function vector(value, maximum = 1_000_000) {
    if (!Array.isArray(value) ||
        value.length !== 2 ||
        value.some((n) => typeof n !== "number" || !Number.isFinite(n) || Math.abs(n) > maximum))
        fail("invalid_vector");
}
function finite(value, minimum, maximum) {
    if (typeof value !== "number" ||
        !Number.isFinite(value) ||
        value < minimum ||
        value > maximum)
        fail("invalid_scalar");
}
export function snapshot(value) {
    const s = record(value, [
        "protocol",
        "session",
        "world_seed",
        "tick",
        "revision",
        "progression",
        "dropped_notices",
        "participants",
        "projectiles",
        "nova_fx",
    ]);
    if (s.protocol !== PROTOCOL ||
        !id(s.session) ||
        !uint(s.world_seed) ||
        s.world_seed === 0 ||
        !uint(s.tick) ||
        !uint(s.revision) ||
        !uint(s.dropped_notices) ||
        s.progression !== false ||
        !Array.isArray(s.participants) ||
        s.participants.length > 8 ||
        !Array.isArray(s.projectiles) ||
        s.projectiles.length > 180)
        fail("invalid_snapshot");
    const ids = new Set(), slots = new Set(), shots = new Set();
    const epochs = new Map();
    for (const value of s.participants) {
        const p = record(value, [
            "id",
            "slot",
            "epoch",
            "local_seed",
            "identity",
            "selected_ship",
            "owned_loadout",
            "position",
            "previous",
            "velocity",
            "heading",
            "previous_heading",
            "hull",
            "max_hull",
            "shield",
            "max_shield",
            "throttle",
            "last_sequence",
        ]);
        if (!id(p.id) ||
            ids.has(p.id) ||
            !Number.isInteger(p.slot) ||
            Number(p.slot) < 0 ||
            Number(p.slot) > 7 ||
            slots.has(p.slot) ||
            !uint(p.epoch) ||
            !Number.isInteger(p.selected_ship) ||
            Number(p.selected_ship) < 0 ||
            Number(p.selected_ship) > 9 ||
            (p.last_sequence !== null && !uint(p.last_sequence)))
            fail("invalid_snapshot_participant");
        ids.add(p.id);
        slots.add(p.slot);
        epochs.set(p.id, p.epoch);
        spec({ local_seed: p.local_seed, identity: p.identity });
        if (p.owned_loadout !== undefined) {
            ownedLoadout(p.owned_loadout);
            if (p.owned_loadout.selected_ship !== p.selected_ship)
                fail("invalid_snapshot_participant");
        }
        for (const key of ["position", "previous", "velocity"])
            vector(p[key]);
        for (const key of ["heading", "previous_heading"])
            finite(p[key], -1_000_000, 1_000_000);
        for (const key of ["hull", "max_hull", "shield", "max_shield"])
            finite(p[key], 0, 100_000);
        if (Number(p.hull) > Number(p.max_hull) ||
            Number(p.shield) > Number(p.max_shield))
            fail("invalid_vitals");
        finite(p.throttle, 0, 1);
    }
    for (const value of s.projectiles) {
        const p = record(value, [
            "id",
            "owner",
            "owner_epoch",
            "position",
            "previous",
            "velocity",
            "damage",
            "life",
            "remaining_hits",
        ]);
        if (typeof p.id !== "string" ||
            !/^[0-9a-f]{16}$/.test(p.id) ||
            shots.has(p.id) ||
            (p.owner === null
                ? p.owner_epoch !== null
                : !id(p.owner) || !uint(p.owner_epoch)) ||
            !Number.isInteger(p.remaining_hits) ||
            Number(p.remaining_hits) < 0 ||
            Number(p.remaining_hits) > 255)
            fail("invalid_projectile");
        shots.add(p.id);
        for (const key of ["position", "previous", "velocity"])
            vector(p[key], 1_100_000);
        finite(p.damage, 0, 100_000);
        finite(p.life, 0, 60);
    }
    if (Object.hasOwn(s, "nova_fx"))
        novaFx(s.nova_fx, Number(s.tick), epochs);
}
/** Visual births only; native complete-commit code owns deduplication and aging. */
function novaFx(value, tick, epochs) {
    const journal = record(value, ["sequence", "dropped", "records"]);
    if (Object.keys(journal).length !== 3 ||
        !uint(journal.sequence) ||
        !uint(journal.dropped) ||
        !Array.isArray(journal.records) ||
        journal.records.length > 32)
        fail("invalid_nova_fx");
    let sequence = 0, birthTick = 0;
    for (const value of journal.records) {
        const birth = record(value, [
            "sequence",
            "tick",
            "emitter",
            "epoch",
            "position",
            "rays",
        ]);
        if (Object.keys(birth).length !== 6 ||
            !uint(birth.sequence) ||
            birth.sequence <= sequence ||
            birth.sequence > journal.sequence ||
            !uint(birth.tick) ||
            birth.tick < birthTick ||
            birth.tick > tick ||
            !id(birth.emitter) ||
            !uint(birth.epoch) ||
            birth.epoch === 0 ||
            epochs.get(birth.emitter) !== birth.epoch ||
            !uint(birth.rays) ||
            birth.rays > 36)
            fail("invalid_nova_fx");
        vector(birth.position);
        sequence = birth.sequence;
        birthTick = birth.tick;
    }
}
export function allowed(kind, sender) {
    return (kind === "hello" ||
        (sender === "host"
            ? [
                "admission",
                "snapshot",
                "publication",
                "publication-end",
                "body",
                "notice",
            ].includes(kind)
            : ["join", "input", "body-query", "publication-ack"].includes(kind)));
}
export function encode(frame) {
    validateFrame(frame);
    const text = JSON.stringify(frame);
    if (new TextEncoder().encode(text).length > MAX_FRAME_BYTES)
        fail("frame_bound");
    return text;
}
function validateFrame(value) {
    const f = record(value, ["v", "session", "seq", "kind", "payload"]);
    if (f.v !== WIRE_VERSION ||
        !id(f.session) ||
        !uint(f.seq) ||
        f.seq === 0 ||
        ![
            "hello",
            "join",
            "admission",
            "input",
            "snapshot",
            "publication",
            "publication-end",
            "publication-ack",
            "body-query",
            "body",
            "notice",
        ].includes(String(f.kind)))
        fail("invalid_frame");
    validatePayload(f.kind, f.payload);
}
export function decode(text) {
    if (typeof text !== "string" ||
        text.length > MAX_FRAME_BYTES ||
        new TextEncoder().encode(text).length > MAX_FRAME_BYTES)
        fail("frame_bound");
    let value;
    try {
        value = JSON.parse(text);
    }
    catch {
        return fail("invalid_json");
    }
    validateFrame(value);
    return value;
}
