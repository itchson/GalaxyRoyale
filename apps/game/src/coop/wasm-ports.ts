import type { Publication, ReplicaPort } from "./controller.ts";
import {
  fail,
  id,
  publication,
  record,
  uint,
  validatePayload,
} from "./wire.ts";
import type { Json } from "./wire.ts";

/** Structural generated-WASM boundary; no Game/Profile or JS simulation. */
export interface GalaxyCoop {
  co_op_batch?: (request: string) => string;
  co_op_replica?: (request: string) => string;
}
export interface GalaxyCoopPorts {
  native: { co_op_batch(request: string): string };
  replica: ReplicaPort | undefined;
  availability: { host: boolean; guest: boolean };
}
const MAX_REQUEST = 65_536;
const bytes = (text: string): number => new TextEncoder().encode(text).length;

/** Call synchronous native transactions with bounded, strict response framing. */
export function createGalaxyCoopPorts(galaxy: GalaxyCoop): GalaxyCoopPorts {
  const batch = typeof galaxy.co_op_batch === "function";
  const replica = typeof galaxy.co_op_replica === "function";
  let session: string | null = null;
  let acquisition: number | null = null;
  let revision: number | null = null;
  const call = (request: Record<string, Json>): boolean => {
    const encoded = JSON.stringify({ v: 1, ...request });
    if (bytes(encoded) > MAX_REQUEST) fail("replica_request_bound");
    const raw = galaxy.co_op_replica!.call(galaxy, encoded);
    if (typeof raw !== "string" || bytes(raw) > MAX_REQUEST)
      fail("replica_response_bound");
    let parsed: unknown;
    try {
      parsed = JSON.parse(raw);
    } catch {
      return fail("replica_response");
    }
    const response = record(parsed, ["v", "ok", "value", "error"]);
    if (response.v !== 1) fail("replica_response");
    if (response.ok === false) {
      if (
        response.value !== undefined ||
        typeof response.error !== "string" ||
        !/^[a-z0-9_]{1,64}$/.test(response.error)
      )
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
  const port: ReplicaPort = {
    begin(expectedSession, peer, epoch, worldSeed) {
      if (session) fail("replica_active");
      if (
        !id(expectedSession) ||
        !id(peer) ||
        !uint(epoch) ||
        epoch === 0 ||
        !uint(worldSeed) ||
        worldSeed === 0
      )
        fail("replica_begin");
      session = expectedSession;
      if (
        !call({
          op: "replica_begin",
          session: expectedSession,
          peer,
          epoch,
          world_seed: worldSeed,
        })
      ) {
        session = null;
        return false;
      }
      acquisition = revision = null;
      return true;
    },
    stage(header: Publication) {
      publication(header);
      if (!session || header.snapshot.session !== session)
        fail("replica_session");
      if (!call({ op: "replica_stage", header: header as unknown as Json }))
        return false;
      acquisition = header.acquisition;
      revision = header.snapshot.revision;
      return true;
    },
    page(page: Json) {
      validatePayload("body", page);
      const p = record(page);
      if (
        !session ||
        acquisition === null ||
        p.session !== session ||
        p.revision !== revision
      )
        fail("replica_page");
      return call({
        op: Array.isArray(p.entities) ? "replica_entities" : "replica_body",
        acquisition,
        page,
      });
    },
    commit(expectedAcquisition, expectedRevision) {
      if (
        acquisition === null ||
        expectedAcquisition !== acquisition ||
        expectedRevision !== revision
      )
        fail("replica_commit");
      if (
        !call({ op: "replica_commit", acquisition, revision: expectedRevision })
      )
        return false;
      acquisition = revision = null;
      return true;
    },
    end() {
      if (!session) return;
      if (!call({ op: "replica_end", session })) fail("replica_end_rejected");
      session = null;
      acquisition = revision = null;
    },
  };
  return {
    native: {
      co_op_batch(request) {
        if (!batch) fail("native_unavailable");
        if (typeof request !== "string" || bytes(request) > MAX_REQUEST)
          fail("native_batch_bound");
        return galaxy.co_op_batch!.call(galaxy, request);
      },
    },
    replica: replica ? port : undefined,
    availability: { host: batch, guest: replica },
  };
}
