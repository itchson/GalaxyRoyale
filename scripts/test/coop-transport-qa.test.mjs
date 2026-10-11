import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { createServer } from "node:net";
import { once } from "node:events";
import {
  prepare,
  root,
  sha,
  stagePath,
} from "../qa/co-op-transport/prepare.mjs";
import { start, validateReport } from "../qa/co-op-transport/server.mjs";

test("QA prepare pins actual source and bounded HTTP serves only allowlisted modules; captures are sanitized", async (t) => {
  const stage = await fs.mkdtemp(
    path.join(root, "work", "co-op-transport-qa-"),
  );
  // Test listeners use an OS-chosen free port and are always stopped.
  const reservation = createServer();
  reservation.listen(0, "127.0.0.1");
  await once(reservation, "listening");
  const port = reservation.address().port;
  await new Promise((r) => reservation.close(r));
  let tool;
  t.after(async () => {
    await tool?.stop();
    assert.ok(!tool || !tool.server.listening);
    await fs.rm(stage, { recursive: true });
  });
  const manifest = await prepare(stage);
  assert.equal(Object.keys(manifest.files).length, 5);
  const source = await fs.readFile(
    path.join(root, "apps/game/src/coop/transport.ts"),
  );
  assert.equal(manifest.inputs["apps/game/src/coop/transport.ts"], sha(source));
  const js = await fs.readFile(
    path.join(stage, "modules/transport.js"),
    "utf8",
  );
  assert.ok(js.includes('"./wire.js"'));
  assert.ok(!js.includes('"./wire.ts"'));
  tool = await start({ stage, port, maxMs: 5000 });
  assert.equal((await fetch(tool.origin + "/")).status, 200);
  assert.equal(
    (await fetch(tool.origin + "/modules/transport.js")).status,
    200,
  );
  assert.equal((await fetch(tool.origin + "/manifest.json")).status, 404);
  assert.equal(
    (await fetch(tool.origin + "/review.js?leak=anything")).status,
    404,
  );
  const config = await (await fetch(tool.origin + "/qa/config")).json();
  assert.equal(config.modules, 3);
  assert.equal(config.service, tool.service);
  assert.equal(config.page_origin, tool.origin);
  assert.notEqual(config.page_origin, config.service);
  const cors = {
    Origin: config.page_origin,
    "Content-Type": "application/json",
  };
  const grant = await (
    await fetch(tool.service + "/v1/sessions", {
      method: "POST",
      headers: cors,
      body: JSON.stringify({ seed: 123, protocol: "galaxy-coop-1" }),
    })
  ).json();
  assert.ok(grant.participant?.token);
  const bearer = { Authorization: `Bearer ${grant.participant.token}` };
  // Actual browser same-origin GET omits Origin; retain the service rejection.
  assert.equal(
    (
      await fetch(tool.service + "/v1/events?after=0&wait_ms=0", {
        headers: bearer,
      })
    ).status,
    403,
  );
  assert.equal(
    (
      await fetch(tool.service + "/v1/events?after=0&wait_ms=0", {
        headers: { ...bearer, Origin: config.page_origin },
      })
    ).status,
    200,
  );
  const headers = (await fetch(tool.origin + "/")).headers;
  assert.ok(headers.get("content-security-policy").includes(config.service));
  const report = {
    version: 1,
    run_id: "a".repeat(16),
    checks: [
      {
        name: "real-channel-ready",
        passed: true,
        counts: { created: 2, closed: 0 },
      },
    ],
    created: 2,
    closed: 0,
    secure_context: true,
    rtc_available: true,
  };
  const post = (body, origin = tool.origin) =>
    fetch(tool.origin + "/qa/report", {
      method: "POST",
      headers: { Origin: origin, "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
  assert.equal((await post(report, "http://other.localhost")).status, 400);
  assert.equal(
    (await post({ ...report, token: "never capture credentials" })).status,
    400,
  );
  assert.equal((await post(report)).status, 200);
  const [name] = await fs.readdir(path.join(stage, "captures"));
  const captured = JSON.parse(
    await fs.readFile(path.join(stage, "captures", name), "utf8"),
  );
  assert.equal(
    captured.scope,
    "actual same-page browser RTC loopback; declared framing admission; no Game authority/NAT",
  );
  // This test report is deliberately supplied metadata, not an RTC execution.
  assert.deepEqual(captured.source_inputs, manifest.inputs);
  await assert.rejects(start({ stage, port, maxMs: 5000 }), /unavailable/);
  await tool.stop();
  assert.equal(tool.server.coop.pendingPolls(), 0);
  await fs.appendFile(path.join(stage, "modules/wire.js"), "\n//changed");
  await assert.rejects(start({ stage, port, maxMs: 5000 }), /digest/);
});
test("QA reports exclude arbitrary telemetry/credentials and prepare refuses outside work", () => {
  assert.throws(() => stagePath(path.join(root, "apps")));
  assert.throws(() => stagePath(undefined));
  const base = {
    version: 1,
    run_id: "a".repeat(16),
    checks: [],
    created: 0,
    closed: 0,
    secure_context: true,
    rtc_available: true,
  };
  for (const invalid of [
    { ...base, run_id: "../../secret" },
    { ...base, closed: 1 },
    {
      ...base,
      checks: [
        { name: "real-channel-ready", passed: true, counts: { token: 1 } },
      ],
    },
    { ...base, checks: [{ name: "SDP", passed: true, counts: {} }] },
  ])
    assert.throws(() => validateReport(invalid));
});
