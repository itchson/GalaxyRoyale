import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import {
  publication,
  validatePayload,
  decode,
  encode,
  MAX_FRAME_BYTES,
} from "../../apps/game/src/coop/wire.ts";

// Exact native111a Value exports. Node executes the strict consumer, not Rust
// gameplay, a browser/RTC connection or a new audio/render capture.
const root = new URL("./fixtures/co-op-wake/", import.meta.url);
const raw = (name) => readFileSync(new URL(`${name}.json`, root), "utf8");
const fixture = (name) => JSON.parse(raw(name));
const frame = (page) => ({
  v: 1,
  session: page.session,
  seq: 1,
  kind: "body",
  payload: page,
});

test("all22 test fixtures retain exact native Git111a bytes", () => {
  const provenance = JSON.parse(
    readFileSync(new URL("provenance.json", root), "utf8"),
  );
  assert.equal(
    provenance.nativeSourceCommit,
    "111a477fe507cd544902c8f44f07198dc31933d6",
  );
  assert.equal(provenance.files.length, 22);
  for (const pin of provenance.files) {
    const bytes = readFileSync(new URL(pin.file, root));
    // Canonical JSON fixture hashes are independent of Windows checkout CRLF.
    const canonical = Buffer.from(
      bytes.toString("utf8").replace(/\r\n/g, "\n"),
    );
    assert.equal(canonical.length, pin.bytes);
    assert.equal(
      createHash("sha256").update(canonical).digest("hex"),
      pin.sha256,
    );
  }
});

test("all exact modern native headers and diagnostic pages cross their distinct strict variants", () => {
  for (const name of [
    "actual-host",
    "initial",
    "empty",
    "expired",
    "maximum",
  ]) {
    publication(fixture(`${name}-header`));
    validatePayload("body", fixture(`${name}-page`));
  }
  publication(fixture("legacy-header"));
  validatePayload("body", fixture("scalar-stress-page"));
  validatePayload("body", fixture("raw-life-short-valid"));
  assert.throws(() => validatePayload("body", fixture("raw-life-underflow")));
  assert.throws(() => validatePayload("body", fixture("raw-life-round-down")));
});

test("Wake metadata distinguishes absence from malformed presence", () => {
  for (const value of [
    null,
    undefined,
    {},
    { points: -1 },
    { points: 193 },
    { points: 0.5 },
    { points: 1, extra: 0 },
  ]) {
    const h = fixture("initial-header");
    h.snapshot.wake_fx = value;
    assert.throws(() => publication(h));
  }
  const h = fixture("initial-header");
  delete h.snapshot.wake_fx;
  publication(h);
  h.snapshot.wake_fx = { points: 0 };
  publication(h);
});

test("Wake pages reject mixed shapes, bad raw/post-f32 numbers and chronology", () => {
  for (const mutate of [
    (p) => (p.offset = 0),
    (p) => (p.cells = []),
    (p) => (p.entities = []),
    (p) => delete p.points[0].life,
    (p) => (p.points[0].rank = 5),
    (p) => (p.points[0].slot = 8),
    (p) => (p.points[0].epoch = 0),
    (p) => (p.points[0].born_tick = p.tick + 1),
    (p) => (p.points[0].position = [1_000_014.01, 0]),
    (p) => (p.points[0].position = [-1_000_014.01, 0]),
    (p) => (p.points[0].position = [Infinity, 0]),
    (p) => (p.points[0].position = [0]),
    (p) => (p.points[0].life = 0),
    (p) => (p.points[0].life = -1),
    (p) => (p.points[0].life = 1e-46),
    (p) => (p.points[0].life = 3.4000001),
    (p) => (p.points[0].life = NaN),
    (p) => p.points.reverse(),
    (p) => p.points.splice(1, 0, structuredClone(p.points[0])),
  ]) {
    const p = fixture("initial-page");
    mutate(p);
    assert.throws(() => validatePayload("body", p));
  }
  const p = fixture("maximum-page");
  p.points.push(structuredClone(p.points[0]));
  assert.throws(() => validatePayload("body", p));
  const over = fixture("maximum-page");
  over.points = Array.from({ length: 25 }, (_, born_tick) => ({
    ...over.points[0],
    born_tick,
  }));
  assert.throws(() => validatePayload("body", over));
  const valid = fixture("initial-page");
  valid.points[0].life = Math.fround(3.4);
  valid.points[0].position = [1_000_014, -1_000_014];
  validatePayload("body", valid);
});

test("actual raw transport fixtures decode and both encoded wrapper budgets remain bounded", () => {
  for (const name of [
    "maximum-transport-publication",
    "maximum-transport-body",
    "scalar-stress-transport-body",
  ]) {
    const parsed = decode(raw(name));
    const encoded = encode(parsed);
    assert.ok(Buffer.byteLength(encoded) <= MAX_FRAME_BYTES);
    assert.deepEqual(JSON.parse(encoded), JSON.parse(raw(name)));
  }
  const max = fixture("maximum-page");
  assert.equal(max.points.length, 192);
  const maxFrame = encode(frame(max));
  assert.ok(Buffer.byteLength(maxFrame) < MAX_FRAME_BYTES);
  console.log(
    JSON.stringify({
      scope: "exact native fixture replay and JS serialization, no RTC",
      pageBytes: Buffer.byteLength(JSON.stringify(max)),
      bodyFrameBytes: Buffer.byteLength(maxFrame),
    }),
  );
});

test("raw32KiB page span is enforced independently of normalized value/frame bytes", () => {
  const page = fixture("initial-page");
  const compact = JSON.stringify(frame(page));
  const make = (size) =>
    compact.replace('"points":[', `"points":[${" ".repeat(size)}`);
  const baseBytes = Buffer.byteLength(JSON.stringify(page));
  const boundary = make(32768 - baseBytes);
  assert.ok(Buffer.byteLength(boundary) < MAX_FRAME_BYTES);
  decode(boundary);
  assert.throws(() => decode(make(32769 - baseBytes)), /wake_page_bound/);
  const escaped = make(33000).replace('"payload":', '"pa\\u0079load":');
  assert.throws(() => decode(escaped), /wake_page_bound/);
  assert.deepEqual(
    decode(compact.replace('"payload":', '"pa\\u0079load":')).payload,
    page,
  );
  assert.deepEqual(
    decode(compact.replace('"points":', '"point\\u0073":')).payload,
    page,
  );
  assert.deepEqual(
    decode(compact.replace('"session":', '"ses\\u0073ion":')).payload,
    page,
  );
  const duplicate = `{"v":1,"session":"${page.session}","seq":1,"kind":"body","payload":{},"payload":${JSON.stringify(page)}}`;
  assert.throws(() => decode(duplicate), /duplicate_payload/);
  const spelled = compact.replace(
    '"life":3.4000000953674316',
    `"life":3.${"4" + "0".repeat(32900)}`,
  );
  assert.ok(Buffer.byteLength(spelled) < MAX_FRAME_BYTES);
  assert.ok(
    Buffer.byteLength(JSON.stringify(JSON.parse(spelled).payload)) < 32768,
  );
  assert.throws(() => decode(spelled), /wake_page_bound/);
  // Unicode whitespace recognized by JSON.parse is restricted to ASCII JSON
  // whitespace; Unicode in an extra key must fail shape before raw scanning.
  const unicode = JSON.stringify(frame({ ...page, é: 0 }));
  assert.throws(() => decode(unicode), /invalid_object/);
  const byteOversize =
    compact.slice(0, -1) + `,"extra":"${"é".repeat(32600)}"}`;
  assert.ok(byteOversize.length < MAX_FRAME_BYTES);
  assert.ok(Buffer.byteLength(byteOversize) > MAX_FRAME_BYTES);
  assert.throws(() => decode(byteOversize), /frame_bound/);
});
