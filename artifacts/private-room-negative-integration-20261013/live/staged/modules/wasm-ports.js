// Actual source SHA256 a7c9a1125ce2ca3f54acd1c5709e6e54fe497f65658002878fc904fed24b4384
import { fail, id, publication, record, uint, validatePayload, } from "./wire.js";
import { ownedLoadout } from "./wire.js";
const MAX_REQUEST = 65_536;
const bytes = (text) => new TextEncoder().encode(text).length;
/** Extract a bounded equipment declaration from the actual native solo Profile. */
export function readOwnedLoadout(galaxy) {
    if (typeof galaxy.co_op_batch !== "function")
        fail("native_unavailable");
    const raw = galaxy.co_op_batch(JSON.stringify({ v: 1, operations: [{ op: "export_loadout" }] }));
    if (typeof raw !== "string" || bytes(raw) > MAX_REQUEST)
        fail("native_loadout_bound");
    let decoded;
    try {
        decoded = JSON.parse(raw);
    }
    catch {
        return fail("native_loadout_response");
    }
    const response = record(decoded, ["v", "results"]);
    if (response.v !== 1 ||
        !Array.isArray(response.results) ||
        response.results.length !== 1)
        fail("native_loadout_response");
    const result = record(response.results[0], ["ok", "value", "error"]);
    if (result.ok !== true || result.error !== undefined)
        fail("native_loadout_unavailable");
    ownedLoadout(result.value);
    return result.value;
}
/** Call synchronous native transactions with bounded, strict response framing. */
export function createGalaxyCoopPorts(galaxy) {
    const batch = typeof galaxy.co_op_batch === "function";
    const replica = typeof galaxy.co_op_replica === "function";
    let session = null;
    let acquisition = null;
    let revision = null;
    const call = (request) => {
        const encoded = JSON.stringify({ v: 1, ...request });
        if (bytes(encoded) > MAX_REQUEST)
            fail("replica_request_bound");
        const raw = galaxy.co_op_replica.call(galaxy, encoded);
        if (typeof raw !== "string" || bytes(raw) > MAX_REQUEST)
            fail("replica_response_bound");
        let parsed;
        try {
            parsed = JSON.parse(raw);
        }
        catch {
            return fail("replica_response");
        }
        const response = record(parsed, ["v", "ok", "value", "error"]);
        if (response.v !== 1)
            fail("replica_response");
        if (response.ok === false) {
            if (response.value !== undefined ||
                typeof response.error !== "string" ||
                !/^[a-z0-9_]{1,64}$/.test(response.error))
                fail("replica_response");
            return false;
        }
        if (response.ok !== true || response.error !== undefined)
            fail("replica_response");
        // Replica operations return native unit, never a replacement world/Profile.
        if (response.value !== undefined && response.value !== null)
            fail("replica_response");
        return true;
    };
    const port = {
        begin(expectedSession, peer, epoch, worldSeed) {
            if (session)
                fail("replica_active");
            if (!id(expectedSession) ||
                !id(peer) ||
                !uint(epoch) ||
                epoch === 0 ||
                !uint(worldSeed) ||
                worldSeed === 0)
                fail("replica_begin");
            session = expectedSession;
            if (!call({
                op: "replica_begin",
                session: expectedSession,
                peer,
                epoch,
                world_seed: worldSeed,
            })) {
                session = null;
                return false;
            }
            acquisition = revision = null;
            return true;
        },
        stage(header) {
            publication(header);
            if (!session || header.snapshot.session !== session)
                fail("replica_session");
            if (!call({ op: "replica_stage", header: header }))
                return false;
            acquisition = header.acquisition;
            revision = header.snapshot.revision;
            return true;
        },
        page(page) {
            validatePayload("body", page);
            const p = record(page);
            if (!session ||
                acquisition === null ||
                p.session !== session ||
                p.revision !== revision)
                fail("replica_page");
            return call({
                op: Array.isArray(p.entities) ? "replica_entities" : "replica_body",
                acquisition,
                page,
            });
        },
        commit(expectedAcquisition, expectedRevision) {
            if (acquisition === null ||
                expectedAcquisition !== acquisition ||
                expectedRevision !== revision)
                fail("replica_commit");
            if (!call({ op: "replica_commit", acquisition, revision: expectedRevision }))
                return false;
            acquisition = revision = null;
            return true;
        },
        end() {
            if (!session)
                return;
            if (!call({ op: "replica_end", session }))
                fail("replica_end_rejected");
            session = null;
            acquisition = revision = null;
        },
    };
    return {
        native: {
            co_op_batch(request) {
                if (!batch)
                    fail("native_unavailable");
                if (typeof request !== "string" || bytes(request) > MAX_REQUEST)
                    fail("native_batch_bound");
                return galaxy.co_op_batch.call(galaxy, request);
            },
        },
        replica: replica ? port : undefined,
        availability: { host: batch, guest: replica },
    };
}
