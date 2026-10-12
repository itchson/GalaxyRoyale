import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { once } from "node:events";
import { makeCoopServer } from "../../../apps/coop/server.mjs";
import { fifthReasons, validateFifthReport } from "./fifth-shot.js";
import {
  root,
  stagePath,
  sha,
  moduleNames,
  preparedNames,
  fileLimit,
  ownedSources,
  verifyOwnedSource,
  verifyServiceInputs,
  serviceSources,
} from "./prepare.mjs";

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
export const wasmCheckNames = new Set([
  "host-approval-before-rtc",
  "two-native-games",
  "independent-save-origins",
  "distinct-local-profiles",
  "real-channel-ready",
  "native-admission",
  "native-complete-commit",
  "movement-fire-observed",
  "host-end-restored",
  "guest-leave-restored",
  "native-owned-export",
  "native-owned-commit",
  "new-session",
  "saves-unchanged",
  "dispose-clean",
  "operation-failed",
  "native-fifth-shot",
]);
export const initializationStages = [
  "none",
  "open_store",
  "starter_selection",
  "import_module",
  "compile_wasm",
  "create_galaxy",
  "pause_solo",
  "register_license",
  "configure_view",
  "read_owned_loadout",
  "bind_controller",
  "save_baseline",
  "read_baseline",
  "ready",
];
export const clientFailureCodes = new Set([
  ...initializationStages.map((stage) => `initialization_${stage}_failed`),
  "initialization_timeout",
  "native_frame_failed",
  "native_action_failed",
  "approval_not_pending",
  ...fifthReasons,
]);
export function validateWasmReport(o) {
  const exact = (v, keys) =>
    v &&
    typeof v === "object" &&
    !Array.isArray(v) &&
    Object.keys(v).sort().join() === [...keys].sort().join();
  const count = (v) => Number.isInteger(v) && v >= 0 && v <= 1_000_000;
  const flag = (v) => v === null || typeof v === "boolean";
  const hash = (v) =>
    v === null || (typeof v === "string" && /^[a-f0-9]{64}$/.test(v));
  const counters = [
    "created",
    "closed",
    "requests",
    "timers",
    "frames",
    "ticks",
    "captures",
    "entity_pages",
    "body_pages",
    "commits",
    "body_changes",
    "same_count_changes",
    "unchanged_bodies",
    "projectiles",
    "transport_ready",
    "admitted",
  ];
  const flags = [
    "profile_equal",
    "persisted_equal",
    "persistent",
    "progression_false",
    "movement_observed",
  ];
  const hashes = ["profile_sha", "baseline_sha", "persisted_sha"];
  const fifth = o?.version === 4;
  const owned = o?.version === 3 || fifth;
  if (
    !exact(o, [
      "version",
      "run_id",
      "backend",
      "checks",
      "clients",
      "session_distinct",
      ...(owned ? ["profile_mode"] : []),
      ...(fifth ? ["scenario"] : []),
    ]) ||
    ![2, 3, 4].includes(o.version) ||
    (fifth && o.scenario !== "renko-fifth") ||
    (owned && o.profile_mode !== "owned") ||
    !/^[a-f0-9]{16}$/.test(o.run_id) ||
    !["webgpu", "webgl2"].includes(o.backend) ||
    !flag(o.session_distinct) ||
    !Array.isArray(o.checks) ||
    o.checks.length > 64 ||
    !Array.isArray(o.clients) ||
    o.clients.length !== 2
  )
    throw new Error("Invalid sanitized WASM report.");
  for (const c of o.checks)
    if (
      !exact(c, ["name", "passed"]) ||
      !wasmCheckNames.has(c.name) ||
      typeof c.passed !== "boolean"
    )
      throw new Error("Invalid WASM check.");
  for (let i = 0; i < 2; i++) {
    const c = o.clients[i];
    if (
      !exact(c, [
        "role",
        "phase",
        "mode",
        "init_stage",
        "failure_code",
        ...counters,
        ...flags,
        ...hashes,
        ...(fifth ? ["fifth_shot"] : []),
        ...(owned
          ? [
              "owned_export",
              "owned_view",
              "selected_ship",
              "owned_slots",
              "owned_sha",
            ]
          : []),
      ]) ||
      c.role !== ["host", "guest"][i] ||
      !initializationStages.includes(c.init_stage) ||
      (c.failure_code !== null && !clientFailureCodes.has(c.failure_code)) ||
      ![
        "uninitialized",
        "idle",
        "creating",
        "joining",
        "hosting",
        "admitting",
        "loading",
        "connected",
        "ended",
        "failed",
        "disposed",
      ].includes(c.phase) ||
      !["solo", "host", "guest", "blocked"].includes(c.mode) ||
      counters.some((k) => !count(c[k])) ||
      c.closed > c.created ||
      c.admitted > 7 ||
      c.transport_ready > 7 ||
      flags.some((k) => !flag(c[k])) ||
      hashes.some((k) => !hash(c[k])) ||
      (owned &&
        (!flag(c.owned_export) ||
          !flag(c.owned_view) ||
          !hash(c.owned_sha) ||
          (c.selected_ship !== null &&
            (!Number.isInteger(c.selected_ship) ||
              c.selected_ship < 0 ||
              c.selected_ship > 9)) ||
          (c.owned_slots !== null &&
            (!Number.isInteger(c.owned_slots) ||
              c.owned_slots < 0 ||
              c.owned_slots > 3))))
    )
      throw new Error("Invalid WASM client observations.");
    if (fifth) validateFifthReport(c.fifth_shot, c.role);
  }
  if (
    o.checks.some(
      (c) =>
        c.name === "native-fifth-shot" &&
        (!fifth ||
          (c.passed &&
            o.clients.some(
              (client) => client.fifth_shot.state !== "complete",
            ))),
    )
  )
    throw new Error(
      "Fifth-shot check requires both complete native observations.",
    );
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
    service = `http://127.0.0.1:${port}`,
    guestOrigin = `http://guest.localhost:${port}`;
  const manifest = JSON.parse(
    await fs.readFile(path.join(stage, "manifest.json"), "utf8"),
  );
  const names = new Set(preparedNames(manifest)),
    wasm = manifest.mode === "wasm";
  if (
    manifest.typescript !== "5.9.3" ||
    Object.keys(manifest.files).length !== names.size ||
    Object.keys(manifest.files).some((n) => !names.has(n))
  )
    throw new Error("Invalid prepared manifest.");
  if (manifest.version === 3) {
    const provenance = await verifyOwnedSource({
      sourceRoot: manifest.source_root,
      sourceCommit: manifest.source_commit,
      dist: manifest.dist,
    });
    if (
      JSON.stringify(provenance.gitInputs) !==
        JSON.stringify(manifest.git_inputs) ||
      JSON.stringify(provenance.buildStamp) !==
        JSON.stringify(manifest.build_stamp) ||
      provenance.compiledSourceCommit !== manifest.compiled_source_commit ||
      JSON.stringify(provenance.compiledGitInputs) !==
        JSON.stringify(manifest.compiled_git_inputs)
    )
      throw new Error("Owned prepared provenance changed.");
  }
  await verifyServiceInputs(manifest.service_inputs);
  if (
    serviceSources.some(
      (source) => manifest.inputs?.[source] !== manifest.service_inputs[source],
    )
  )
    throw new Error("Prepared service provenance is incomplete.");
  for (const [source, hash] of Object.entries(manifest.inputs)) {
    const sourceRoot =
      manifest.version === 3 && ownedSources.includes(source)
        ? manifest.source_root
        : root;
    const absolute = path.resolve(sourceRoot, source);
    if (
      !absolute.startsWith(sourceRoot + path.sep) ||
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
    if (!stat.isFile() || stat.isSymbolicLink() || stat.size > fileLimit(name))
      throw new Error("Invalid prepared file.");
    const value = await fs.readFile(absolute);
    if (sha(value) !== manifest.files[name])
      throw new Error("Prepared file digest mismatch.");
    bytes.set(name, value);
  }
  const origins = wasm ? [origin, guestOrigin] : [origin];
  const server = makeCoopServer({ allowedOrigins: origins });
  const [signaling] = server.listeners("request");
  server.removeListener("request", signaling);
  server.on("request", async (req, res) => {
    try {
      const url = new URL(req.url, service);
      if (
        ![...origins, service]
          .map((v) => new URL(v).host)
          .includes(req.headers.host) ||
        ![...origins, service].includes(url.origin)
      )
        throw new Error("Invalid QA host.");
      if (url.pathname.startsWith("/v1/")) return signaling(req, res);
      res.setHeader("Cache-Control", "no-store");
      res.setHeader("X-Content-Type-Options", "nosniff");
      res.setHeader(
        "Content-Security-Policy",
        wasm
          ? `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; connect-src 'self' ${service}; frame-src ${origin} ${guestOrigin}; media-src 'none'; object-src 'none'; frame-ancestors ${origin}`
          : `default-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self' ${service}; media-src 'none'; object-src 'none'; frame-ancestors 'none'`,
      );
      if (
        req.method === "GET" &&
        url.pathname === "/qa/config" &&
        !url.search
      ) {
        res.setHeader("Content-Type", "application/json");
        return res.end(
          JSON.stringify({
            modules: wasm ? 6 : moduleNames.length,
            service,
            page_origin: origin,
            inputs: manifest.inputs,
            scope: manifest.scope,
            ...(wasm
              ? {
                  mode: "wasm",
                  guest_origin: guestOrigin,
                  builds: manifest.builds,
                  artifacts: manifest.artifacts,
                  client_role:
                    req.headers.host === new URL(guestOrigin).host
                      ? "guest"
                      : "host",
                  profile_mode: manifest.profile_mode ?? "factory",
                  ...(manifest.fifth_shot === true
                    ? {
                        scenarios:
                          manifest.version === 3
                            ? ["default", "renko-fifth"]
                            : ["default"],
                      }
                    : {}),
                  ...(manifest.version === 3
                    ? {
                        source_commit: manifest.source_commit,
                        build_stamp: manifest.build_stamp,
                      }
                    : {}),
                }
              : {}),
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
        const report = (wasm ? validateWasmReport : validateReport)(
          await body(req),
        );
        if (
          wasm &&
          report.version !== manifest.version &&
          !(
            manifest.version === 3 &&
            manifest.fifth_shot === true &&
            report.version === 4
          )
        )
          throw new Error("Report mode does not match prepared provenance.");
        const captured = {
          ...report,
          source_inputs: manifest.inputs,
          captured_at: new Date().toISOString(),
          scope:
            manifest.version === 3
              ? "Root-driven browser observations from real native owned-loadout export and two-Galaxy WASM/controller/RTC; no shared progression/NAT/performance/every-ability acceptance"
              : wasm
                ? "Root-driven browser observations from real two-Galaxy WASM/controller/RTC on independent QA origins; v1 factory guest/no shared progression; no NAT/performance acceptance"
                : "actual same-page browser RTC loopback; declared framing admission; no Game authority/NAT",
          ...(wasm
            ? { builds: manifest.builds, artifacts: manifest.artifacts }
            : {}),
          ...(manifest.version === 3
            ? {
                source_commit: manifest.source_commit,
                git_inputs: manifest.git_inputs,
                build_stamp: manifest.build_stamp,
              }
            : {}),
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
      const name =
        url.pathname === "/"
          ? wasm
            ? "wasm-review.html"
            : "review.html"
          : url.pathname.slice(1);
      if (req.method !== "GET" || url.search || !names.has(name)) {
        res.writeHead(404);
        return res.end("Not found");
      }
      res.setHeader(
        "Content-Type",
        name.endsWith(".html")
          ? "text/html; charset=utf-8"
          : name.endsWith(".wasm")
            ? "application/wasm"
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
