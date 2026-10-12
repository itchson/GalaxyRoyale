// Actual source SHA256 49ebdb7750f053f263a5b3741348e6d2f0171d3b16fd1dc7665242d5e313e433
import { fail, id, PROTOCOL, record, uint, TransportError } from "./wire.js";
import { normalizeRoomCode } from "./room-code.js";
export function session(value) {
    const o = record(value, [
        "id",
        "seed",
        "protocol",
        "host_id",
        "expires_at",
        "state",
        "reason",
        "joining_locked",
        "room_revision",
        "room_code",
    ]);
    if (!id(o.id) ||
        !id(o.host_id) ||
        !uint(o.seed) ||
        o.seed === 0 ||
        o.protocol !== PROTOCOL ||
        !Number.isSafeInteger(o.expires_at) ||
        Number(o.expires_at) < 0 ||
        typeof o.joining_locked !== "boolean" ||
        !Number.isSafeInteger(o.room_revision) ||
        Number(o.room_revision) < 0 ||
        (o.room_code !== null && normalizeRoomCode(o.room_code) !== o.room_code) ||
        !["open", "ended"].includes(String(o.state)) ||
        (o.reason !== null &&
            (typeof o.reason !== "string" || o.reason.length > 128)))
        fail("invalid_session");
    return o;
}
export function participant(value) {
    const o = record(value);
    if (!id(o.id) ||
        !["host", "guest"].includes(String(o.role)) ||
        !["pending", "active", "left", "ended"].includes(String(o.status)))
        fail("invalid_participant");
    return {
        id: o.id,
        role: o.role,
        status: o.status,
    };
}
export function grant(value, role) {
    const o = record(value, ["session", "participant", "cursor", "invite"]);
    const s = session(o.session), p = participant(o.participant), secret = record(o.participant).token;
    if (p.role !== role ||
        p.status !== (role === "host" ? "active" : "pending") ||
        s.state !== "open" ||
        (role === "host" && p.id !== s.host_id) ||
        (role === "guest" && p.id === s.host_id) ||
        typeof secret !== "string" ||
        !/^[0-9a-f]{64}$/.test(secret) ||
        !Number.isSafeInteger(o.cursor) ||
        Number(o.cursor) < 0 ||
        (role === "host" && (s.room_code === null || o.invite !== s.room_code)) ||
        (role === "guest" && (s.room_code !== null || o.invite !== undefined)))
        fail("invalid_grant");
    return {
        session: s,
        participant: { ...p, token: secret },
        cursor: Number(o.cursor),
        ...(role === "host" ? { invite: o.invite } : {}),
    };
}
/** Bounded no-redirect/no-cookie HTTP. Credentials never leave the configured origin. */
export class Signaling {
    fetcher;
    url;
    constructor(url, fetcher = globalThis.fetch.bind(globalThis)) {
        this.fetcher = fetcher;
        const u = new URL(url);
        if (!["http:", "https:"].includes(u.protocol) ||
            u.username ||
            u.password ||
            u.search ||
            u.hash ||
            !["", "/"].includes(u.pathname))
            fail("invalid_service_url");
        this.url = u.origin;
    }
    async request(path, token, body, signal) {
        const response = await this.fetcher(this.url + path, {
            method: body === undefined ? "GET" : "POST",
            signal,
            redirect: "error",
            credentials: "omit",
            referrerPolicy: "no-referrer",
            headers: {
                ...(token ? { Authorization: `Bearer ${token}` } : {}),
                ...(body === undefined ? {} : { "Content-Type": "application/json" }),
            },
            ...(body === undefined ? {} : { body: JSON.stringify(body) }),
        });
        if (response.redirected ||
            (response.url && new URL(response.url).origin !== this.url))
            fail("service_redirect");
        const declared = Number(response.headers.get("content-length") ?? 0);
        if (!Number.isFinite(declared) || declared > 300_000)
            fail("response_bound");
        const reader = response.body?.getReader();
        if (!reader)
            fail("empty_response");
        let size = 0;
        const chunks = [];
        try {
            while (true) {
                const part = await reader.read();
                if (part.done)
                    break;
                size += part.value.length;
                if (size > 300_000)
                    fail("response_bound");
                chunks.push(part.value);
            }
        }
        catch (error) {
            await reader.cancel().catch(() => { });
            throw error;
        }
        finally {
            reader.releaseLock();
        }
        const bytes = new Uint8Array(size);
        let offset = 0;
        for (const chunk of chunks) {
            bytes.set(chunk, offset);
            offset += chunk.length;
        }
        let value;
        try {
            value = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
        }
        catch {
            return fail("invalid_service_json");
        }
        if (!response.ok) {
            // Never echo a remote response, SDP, token or URL into diagnostics/errors.
            const allowed = ROOM_ERRORS[response.status];
            const remote = value !== null && typeof value === "object" && !Array.isArray(value)
                ? value.error
                : null;
            const remoteCode = remote !== null && typeof remote === "object" && !Array.isArray(remote)
                ? remote.code
                : null;
            const code = typeof remoteCode === "string" && allowed?.has(remoteCode)
                ? remoteCode
                : response.status === 401
                    ? "unauthorized"
                    : response.status === 410
                        ? "session_ended"
                        : response.status === 429
                            ? "signaling_limit"
                            : "signaling_request";
            throw new TransportError(code);
        }
        return value;
    }
}
/** Only protocol-owned, status-matched codes may reach the user. */
const ROOM_ERRORS = {
    400: new Set(["invalid_invite", "invalid_request"]),
    401: new Set(["unauthorized", "invalid_invite"]),
    403: new Set(["host_only", "approval_required"]),
    404: new Set(["invite_not_found"]),
    409: new Set(["session_full", "joining_locked", "request_unavailable"]),
    410: new Set(["stale_invite", "session_ended", "participant_left"]),
    429: new Set(["signaling_limit"]),
};
