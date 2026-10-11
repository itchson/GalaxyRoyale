import { createServer } from "node:http";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { CoopError, LIMITS, PROTOCOL, SessionStore } from "./store.mjs";

export const DEFAULT_ORIGINS = Object.freeze([
  "http://localhost:4173",
  "http://127.0.0.1:4173",
]);
function origins(values) {
  if (!Array.isArray(values) || values.length < 1 || values.length > 16)
    throw new Error("Configure 1–16 exact game origins.");
  for (const value of values) {
    const url = new URL(value);
    if (
      !["http:", "https:"].includes(url.protocol) ||
      url.origin !== value ||
      url.username ||
      url.password
    )
      throw new Error(
        "Origins must be exact HTTP(S) origins without paths or credentials.",
      );
  }
  return new Set(values);
}
function bearer(req) {
  const match = /^Bearer ([0-9a-f]{64})$/.exec(req.headers.authorization ?? "");
  if (!match)
    throw new CoopError(
      401,
      "unauthorized",
      "A participant bearer credential is required.",
    );
  return match[1];
}
function body(req, maximum) {
  if (req.headers["content-type"]?.split(";")[0].trim() !== "application/json")
    throw new CoopError(415, "content_type", "Use application/json.");
  if (
    req.headers["content-length"] !== undefined &&
    Number(req.headers["content-length"]) > maximum
  )
    throw new CoopError(
      413,
      "body_limit",
      "Request body exceeds the documented bound.",
    );
  return new Promise((accept, reject) => {
    const chunks = [];
    let size = 0;
    const cleanup = () => {
      req.off("data", data);
      req.off("end", end);
      req.off("error", error);
      req.off("aborted", aborted);
    };
    const error = () => {
      cleanup();
      reject(
        new CoopError(400, "invalid_request", "Request could not be read."),
      );
    };
    const aborted = error;
    const data = (chunk) => {
      size += chunk.length;
      if (size > maximum) {
        cleanup();
        req.resume();
        reject(
          new CoopError(
            413,
            "body_limit",
            "Request body exceeds the documented bound.",
          ),
        );
      } else chunks.push(chunk);
    };
    const end = () => {
      cleanup();
      try {
        accept(JSON.parse(Buffer.concat(chunks).toString("utf8")));
      } catch {
        reject(
          new CoopError(
            400,
            "invalid_json",
            "Request must contain valid JSON.",
          ),
        );
      }
    };
    req.on("data", data);
    req.on("end", end);
    req.on("error", error);
    req.on("aborted", aborted);
  });
}

/** Separate localhost signaling service; importing it never starts a listener. */
export function makeCoopServer({
  store = new SessionStore(),
  allowedOrigins = DEFAULT_ORIGINS,
} = {}) {
  const allowed = origins(allowedOrigins),
    polls = new Map(),
    sockets = new Set();
  const json = (res, status, value) => {
    if (res.destroyed || res.writableEnded) return;
    res.writeHead(status, {
      "Content-Type": "application/json; charset=utf-8",
      "Cache-Control": "no-store",
      "X-Content-Type-Options": "nosniff",
      ...(status >= 400 ? { Connection: "close" } : {}),
    });
    res.end(JSON.stringify(value));
  };
  const poll = (req, res, token, after, wait) => {
    const first = store.events(token, after);
    if (
      first.events.length ||
      first.session.state !== "open" ||
      first.participant.status === "left" ||
      wait === 0
    )
      return first;
    const key = `${first.session.id}/${first.participant.id}`;
    if (polls.has(key))
      throw new CoopError(
        409,
        "concurrent_poll",
        "Only one pending read per participant is allowed.",
      );
    return new Promise((accept, reject) => {
      let timer;
      const finish = (error = null, disconnected = false) => {
        if (!polls.has(key)) return;
        clearTimeout(timer);
        polls.delete(key);
        store.off("changed", changed);
        req.off("aborted", abort);
        res.off("close", abort);
        if (disconnected) return accept(null);
        if (error) return reject(error);
        try {
          accept(store.events(token, after));
        } catch (e) {
          reject(e);
        }
      };
      const changed = (id) => {
        if (id !== first.session.id) return;
        try {
          const next = store.events(token, after);
          if (
            next.events.length ||
            next.session.state !== "open" ||
            next.participant.status === "left"
          )
            finish();
        } catch (e) {
          finish(e);
        }
      };
      const abort = () => finish(null, true);
      polls.set(key, finish);
      store.on("changed", changed);
      req.on("aborted", abort);
      res.on("close", abort);
      timer = setTimeout(() => finish(), wait);
      changed(first.session.id); // Close the subscribe/read race.
    });
  };
  const server = createServer(async (req, res) => {
    try {
      if (
        !req.url?.startsWith("/") ||
        req.url.startsWith("//") ||
        req.url.length > 2048
      )
        throw new CoopError(400, "invalid_url", "Use a local service path.");
      const url = new URL(req.url, "http://127.0.0.1");
      if (req.method === "GET" && url.pathname === "/health")
        return json(res, 200, {
          status: "ok",
          protocol: PROTOCOL,
          players: LIMITS.players,
        });
      if (!allowed.has(req.headers.origin))
        throw new CoopError(403, "origin", "This game origin is not allowed.");
      res.setHeader("Access-Control-Allow-Origin", req.headers.origin);
      res.setHeader("Vary", "Origin");
      if (req.method === "OPTIONS") {
        if (
          !["GET", "POST"].includes(
            req.headers["access-control-request-method"],
          )
        )
          throw new CoopError(
            405,
            "method",
            "Only GET and POST are supported.",
          );
        const headers = (req.headers["access-control-request-headers"] ?? "")
          .toLowerCase()
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean);
        if (headers.some((h) => !["authorization", "content-type"].includes(h)))
          throw new CoopError(
            403,
            "cors_headers",
            "Unsupported request headers.",
          );
        res.writeHead(204, {
          "Access-Control-Allow-Methods": "GET, POST",
          "Access-Control-Allow-Headers": "Authorization, Content-Type",
          "Cache-Control": "no-store",
        });
        return res.end();
      }
      if (req.method === "GET" && url.pathname === "/v1/events") {
        if (
          [...url.searchParams.keys()].some(
            (k) => !["after", "wait_ms"].includes(k),
          ) ||
          ["after", "wait_ms"].some(
            (k) => url.searchParams.getAll(k).length > 1,
          )
        )
          throw new CoopError(400, "invalid_query", "Unexpected event query.");
        const parse = (key, fallback) => {
          const s = url.searchParams.get(key);
          if (s === null) return fallback;
          if (!/^(0|[1-9][0-9]{0,15})$/.test(s))
            throw new CoopError(
              400,
              "invalid_query",
              "Use an unsigned integer cursor and wait.",
            );
          return Number(s);
        };
        const after = parse("after", 0),
          wait = parse("wait_ms", 0);
        if (wait > store.limits.pollMs)
          throw new CoopError(
            400,
            "invalid_wait",
            "Polling wait exceeds the documented bound.",
          );
        const result = await poll(req, res, bearer(req), after, wait);
        if (result) json(res, 200, result);
        return;
      }
      const route = url.pathname;
      if (
        req.method !== "POST" ||
        ![
          "/v1/sessions",
          "/v1/join",
          "/v1/replace",
          "/v1/heartbeat",
          "/v1/leave",
          "/v1/end",
          "/v1/signals",
        ].includes(route)
      )
        throw new CoopError(404, "route", "Signaling route not found.");
      if (url.search)
        throw new CoopError(
          400,
          "invalid_query",
          "Mutation routes do not accept query parameters.",
        );
      const client = store.clientKey(req.socket.remoteAddress ?? "local");
      const token = ["/v1/sessions", "/v1/join"].includes(route)
        ? null
        : bearer(req);
      if (route === "/v1/sessions" || route === "/v1/join")
        store.admitAnonymous(route === "/v1/join" ? "join" : "create", client);
      else if (route === "/v1/replace") store.admitReplacement(token);
      const payload = await body(req, store.limits.bodyBytes);
      const handlers = {
        "/v1/sessions": () => store.create(payload, client),
        "/v1/join": () => store.join(payload),
        "/v1/replace": () => store.create(payload, client, token),
        "/v1/heartbeat": () => store.heartbeat(token, payload),
        "/v1/leave": () => store.leave(token, payload),
        "/v1/end": () => store.end(token, payload),
        "/v1/signals": () => store.signal(token, payload),
      };
      json(
        res,
        ["/v1/sessions", "/v1/join", "/v1/replace"].includes(route) ? 201 : 200,
        handlers[route](),
      );
    } catch (e) {
      req.resume();
      json(res, e instanceof CoopError ? e.status : 500, {
        error: {
          code: e instanceof CoopError ? e.code : "internal",
          message:
            e instanceof CoopError ? e.message : "Signaling service failed.",
        },
      });
    }
  });
  server.requestTimeout = 10_000;
  server.headersTimeout = 5_000;
  server.keepAliveTimeout = 2_000;
  server.maxRequestsPerSocket = 200;
  server.on("connection", (socket) => {
    if (sockets.size >= 128) return socket.destroy();
    sockets.add(socket);
    socket.once("close", () => sockets.delete(socket));
  });
  const maintenance = setInterval(() => store.sweep(), 250);
  maintenance.unref();
  server.once("close", () => {
    clearInterval(maintenance);
    for (const finish of polls.values()) finish(null, true);
  });
  Object.defineProperty(server, "coop", {
    value: {
      store,
      pendingPolls: () => polls.size,
      async stop() {
        store.shutdown();
        await new Promise((accept) => {
          const timeout = setTimeout(() => server.closeAllConnections(), 500);
          timeout.unref();
          server.close(() => {
            clearTimeout(timeout);
            accept();
          });
          server.closeIdleConnections();
        });
      },
    },
  });
  return server;
}

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const port = Number(process.env.COOP_PORT ?? 4176);
  if (!Number.isInteger(port) || port < 1 || port > 65535)
    throw new Error("COOP_PORT must be a valid TCP port.");
  const allowedOrigins =
    process.env.COOP_ALLOWED_ORIGINS?.split(",").map((s) => s.trim()) ??
    DEFAULT_ORIGINS;
  const server = makeCoopServer({ allowedOrigins });
  server.on("error", () => {
    console.error(
      "Co-op signaling listener failed; no credentials or signal data were logged.",
    );
    process.exitCode = 1;
  });
  server.listen(port, "127.0.0.1", () =>
    console.log(`Ephemeral co-op signaling: http://127.0.0.1:${port}`),
  );
  let stopping = false;
  const stop = () => {
    if (!stopping) {
      stopping = true;
      void server.coop.stop();
    }
  };
  process.once("SIGINT", stop);
  process.once("SIGTERM", stop);
}
