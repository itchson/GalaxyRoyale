import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { once } from "node:events";
import { makeCoopServer } from "../../../apps/coop/server.mjs";
import { root, stagePath, sha, moduleNames, pageNames } from "./prepare.mjs";

const names = new Set([
  ...pageNames,
  ...moduleNames.map((n) => `modules/${n}.js`),
]);
const checkNames = new Set([
  "real-channel-ready",
  "admission-pending",
  "new-session-ready",
  "rejoin-admission-pending",
  "distinct-session",
  "join-sent",
  "authenticated-join",
  "input-before-admission-refused",
  "fixture-admission-sent",
  "input-sent",
  "input-delivered",
  "simulation-still-waiting",
  "host-end-clean",
  "dispose-clean",
  "operation-failed",
]);
export function validateReport(o) {
  const exact = (value, keys) =>
    value &&
    typeof value === "object" &&
    !Array.isArray(value) &&
    Object.keys(value).every((k) => keys.includes(k));
  const count = (n) => Number.isInteger(n) && n >= 0 && n <= 32;
  if (
    !exact(o, [
      "version",
      "run_id",
      "checks",
      "created",
      "closed",
      "secure_context",
      "rtc_available",
      "rtc_errors",
    ]) ||
    o.version !== 1 ||
    !/^[a-f0-9]{16}$/.test(o.run_id) ||
    !count(o.created) ||
    !count(o.closed) ||
    o.closed > o.created ||
    typeof o.secure_context !== "boolean" ||
    typeof o.rtc_available !== "boolean" ||
    (o.rtc_errors !== undefined &&
      (!Array.isArray(o.rtc_errors) ||
        o.rtc_errors.length > 8 ||
        o.rtc_errors.some(
          (d) =>
            ![
              "data-channel-failure",
              "dtls-failure",
              "fingerprint-failure",
              "hardware-encoder-error",
              "hardware-encoder-not-available",
              "sctp-failure",
              "sdp-syntax-error",
            ].includes(d),
        ))) ||
    !Array.isArray(o.checks) ||
    o.checks.length > 64
  )
    throw new Error("Invalid sanitized report.");
  for (const c of o.checks) {
    if (
      !exact(c, ["name", "passed", "counts"]) ||
      !checkNames.has(c.name) ||
      typeof c.passed !== "boolean" ||
      !exact(c.counts, ["created", "closed"]) ||
      Object.values(c.counts).some((n) => !count(n))
    )
      throw new Error("Invalid sanitized check.");
  }
  return o;
}
async function body(req) {
  let size = 0;
  const chunks = [];
  for await (const chunk of req) {
    if ((size += chunk.length) > 24_000)
      throw new Error("Report bound exceeded.");
    chunks.push(chunk);
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf8"));
}
export async function start({
  stage: value,
  port = 4189,
  maxMs = 20 * 60_000,
} = {}) {
  if (
    !Number.isInteger(port) ||
    port < 1 ||
    port > 65535 ||
    !Number.isInteger(maxMs) ||
    maxMs < 1000 ||
    maxMs > 60 * 60_000
  )
    throw new Error("Invalid bounded QA listener options.");
  const stage = stagePath(value),
    origin = `http://localhost:${port}`,
    service = `http://127.0.0.1:${port}`;
  const manifest = JSON.parse(
    await fs.readFile(path.join(stage, "manifest.json"), "utf8"),
  );
  if (
    manifest.version !== 1 ||
    manifest.typescript !== "5.9.3" ||
    Object.keys(manifest.files).length !== names.size ||
    Object.keys(manifest.files).some((n) => !names.has(n))
  )
    throw new Error("Invalid prepared manifest.");
  for (const [source, hash] of Object.entries(manifest.inputs)) {
    const absolute = path.resolve(root, source);
    if (
      !absolute.startsWith(root + path.sep) ||
      sha(await fs.readFile(absolute)) !== hash
    )
      throw new Error(
        "Prepared source changed; prepare a new stage before launch.",
      );
  }
  const bytes = new Map();
  for (const name of names) {
    const absolute = path.join(stage, name),
      stat = await fs.lstat(absolute);
    if (!stat.isFile() || stat.isSymbolicLink() || stat.size > 100_000)
      throw new Error("Invalid prepared file.");
    const value = await fs.readFile(absolute);
    if (sha(value) !== manifest.files[name])
      throw new Error("Prepared file digest mismatch.");
    bytes.set(name, value);
  }
  const server = makeCoopServer({ allowedOrigins: [origin] });
  const [signaling] = server.listeners("request");
  server.removeListener("request", signaling);
  server.on("request", async (req, res) => {
    try {
      const url = new URL(req.url, service);
      if (
        ![new URL(origin).host, new URL(service).host].includes(
          req.headers.host,
        ) ||
        ![origin, service].includes(url.origin)
      )
        throw new Error("Invalid QA host.");
      if (url.pathname.startsWith("/v1/")) return signaling(req, res);
      res.setHeader("Cache-Control", "no-store");
      res.setHeader("X-Content-Type-Options", "nosniff");
      res.setHeader(
        "Content-Security-Policy",
        `default-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self' ${service}; media-src 'none'; object-src 'none'; frame-ancestors 'none'`,
      );
      if (
        req.method === "GET" &&
        url.pathname === "/qa/config" &&
        !url.search
      ) {
        res.setHeader("Content-Type", "application/json");
        return res.end(
          JSON.stringify({
            modules: moduleNames.length,
            service,
            page_origin: origin,
            inputs: manifest.inputs,
            scope: manifest.scope,
          }),
        );
      }
      if (
        req.method === "POST" &&
        url.pathname === "/qa/report" &&
        !url.search
      ) {
        if (
          req.headers.origin !== origin ||
          req.headers["content-type"] !== "application/json"
        )
          throw new Error("Invalid QA origin/content.");
        const report = validateReport(await body(req));
        const captured = {
          ...report,
          source_inputs: manifest.inputs,
          captured_at: new Date().toISOString(),
          scope:
            "actual same-page browser RTC loopback; declared framing admission; no Game authority/NAT",
        };
        const encoded = JSON.stringify(captured, null, 2) + "\n",
          hash = sha(encoded);
        await fs.mkdir(path.join(stage, "captures"), { recursive: true });
        // Immutable per-action files; no prior capture can be replaced.
        await fs.writeFile(
          path.join(stage, "captures", `${report.run_id}-${hash}.json`),
          encoded,
          { flag: "wx" },
        );
        res.setHeader("Content-Type", "application/json");
        return res.end(JSON.stringify({ captured: true, sha256: hash }));
      }
      const name = url.pathname === "/" ? "review.html" : url.pathname.slice(1);
      if (req.method !== "GET" || url.search || !names.has(name)) {
        res.writeHead(404);
        return res.end("Not found");
      }
      res.setHeader(
        "Content-Type",
        name.endsWith(".html")
          ? "text/html; charset=utf-8"
          : "text/javascript; charset=utf-8",
      );
      res.end(bytes.get(name));
    } catch {
      req.resume();
      if (!res.headersSent) res.writeHead(400);
      res.end("QA request rejected; no sensitive data logged.");
    }
  });
  let stopping;
  const stop = () =>
    (stopping ??= (async () => {
      clearTimeout(timer);
      await server.coop.stop();
    })());
  const timer = setTimeout(() => void stop(), maxMs);
  timer.unref();
  try {
    server.listen(port, "127.0.0.1");
    await once(server, "listening");
  } catch {
    clearTimeout(timer);
    await server.coop.stop();
    throw new Error(
      `QA port${port} unavailable; no existing process was stopped.`,
    );
  }
  return { server, stop, origin, service, stage };
}
if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const args = process.argv.slice(2),
    allowed = new Set(["--stage", "--port", "--max-ms"]),
    options = {};
  if (
    args.length % 2 ||
    args.some((arg, i) => i % 2 === 0 && !allowed.has(arg))
  )
    throw new Error(
      "Usage: node scripts/qa/co-op-transport/server.mjs --stage work/co-op-transport/stage [--port4189] [--max-ms1200000]",
    );
  for (let i = 0; i < args.length; i += 2) {
    if (Object.hasOwn(options, args[i]))
      throw new Error("Duplicate QA option.");
    options[args[i]] = args[i + 1];
  }
  const tool = await start({
    stage: options["--stage"],
    port: Number(options["--port"] ?? 4189),
    maxMs: Number(options["--max-ms"] ?? 1_200_000),
  });
  process.once("SIGINT", () => void tool.stop());
  process.once("SIGTERM", () => void tool.stop());
  console.log(
    JSON.stringify({
      scope: "temporary local RTC review",
      pid: process.pid,
      url: tool.origin,
      stage: tool.stage,
      autoStopMs: Number(options["--max-ms"] ?? 1_200_000),
    }),
  );
}
