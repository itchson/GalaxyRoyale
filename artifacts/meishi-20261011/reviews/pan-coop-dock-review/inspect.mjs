import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import assert from "node:assert/strict";
import { gunzipSync } from "node:zlib";
import { createHash } from "node:crypto";
const root = process.argv[2] ?? "C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale";
const digest = (value) => createHash("sha256").update(value).digest("hex");
const read = (path) => {
  if (!path.endsWith("-extraction.json")) return JSON.parse(readFileSync(resolve(root, path), "utf8"));
  const folder = path.slice(0, path.lastIndexOf("/"));
  const name = path.slice(path.lastIndexOf("/") + 1);
  const record = JSON.parse(readFileSync(resolve(root, folder, "archive.json"), "utf8")).files[name];
  const encoded = readFileSync(resolve(root, path + ".gz"));
  assert.equal(encoded.length, record.gzip_bytes);
  assert.equal(digest(encoded), record.gzip_sha256);
  const raw = gunzipSync(encoded, { maxOutputLength: 32 * 1024 * 1024 });
  assert.equal(raw.length, record.decoded_bytes);
  assert.equal(digest(raw), record.decoded_sha256);
  return JSON.parse(raw);
};
const prefix = "assets/pipeline/pan-coop-dock/native256";
const pack = read(prefix + "/pack.json");
const rig = read(prefix + "/rig-extraction.json");
const zoom = read(prefix + "/zoom-extraction.json");
const tileCounts = [], materialCounts = [0, 0, 0, 0], stateChanged = [0, 0, 0];
const expected = [];
for (let tileIndex = 0; tileIndex < pack.pixel_rig_tiles.tiles.length; tileIndex++) {
  const tile = pack.pixel_rig_tiles.tiles[tileIndex];
  assert.deepEqual([tile.width, tile.height], [128, 128]);
  assert.equal(tile.roles.length, 16384);
  assert.equal(tile.parts.length, 4);
  let count = 0;
  for (let role = 0; role < 4; role++) {
    const colors = tile.frame_sprites.map((key) => pack.sprites[key].indices);
    for (let i = 0; i < colors[0].length; i++) {
      if (!colors[0][i] || tile.roles[i] !== role) continue;
      const row = Math.floor(i / 128), column = i % 128;
      const x = column - 63.5 + tile.position[0], y = row - 63.5 + tile.position[1];
      const global = (row + Math.floor(tileIndex / 2) * 128) * 256 + column + tileIndex % 2 * 128;
      materialCounts[role]++; count++;
      expected.push({ corners: [[x-.5,y-.5],[x+.5,y-.5],[x+.5,y+.5],[x-.5,y+.5]].map(([a,b]) => [a*.5,b*.5]), colors: colors.map((indices) => {
        const original = pack.palette[indices[i]];
        const channels = [original & 255, (original>>>8)&255, (original>>>16)&255, original>>>24];
        if (role === 3) for (let c = 0; c < 3; c++) channels[c] = Math.round(channels[c] * 0.84);
        return (channels[0] | channels[1]<<8 | channels[2]<<16 | channels[3]<<24) >>> 0;
      }), role, global });
      for (const part of ["deck", "alloy", "screens", "signals"]) {
        const indices = pack.sprites[`${tile.name}-${part}-0`].indices;
        assert.equal(indices[i], part === tile.parts[role].name ? colors[0][i] : 0);
      }
    }
  }
  tileCounts.push({ name: tile.name, count });
}
assert.equal(expected.length, 28549);
const lamps = new Set(pack.pixel_rig_tiles.lamp_cells_source);
const materials = [];
for (const [state, frame] of rig.frames.entries()) {
  assert.equal(frame.occupied_cells, expected.length); assert.equal(frame.cells.length, expected.length);
  assert.equal(frame.transforms, 16); assert.equal(frame.batches, 1);
  const seen = new Set();
  for (const [index, cell] of frame.cells.entries()) {
    const e = expected[index]; assert.deepEqual(cell.corners, e.corners);
    assert.equal(cell.color, e.colors[state]);
    const material = [cell.light_response, cell.emission];
    assert.ok(material.every(Number.isFinite));
    assert.ok(cell.emission >= 0 && cell.emission <= .6);
    materials[e.role] ??= material;
    assert.deepEqual(materials[e.role], material);
    assert.ok(Math.abs(cell.light_response - [.8,.65,.65,.5][e.role]) < .000001);
    assert.ok(Math.abs(cell.emission - [0,.031,.031,.132][e.role]) < .000001);
    assert.ok(seen.add(JSON.stringify(cell.corners)).size === index + 1);
    if (state && cell.color !== rig.frames[0].cells[index].color) {
      assert.ok(lamps.has(e.global)); stateChanged[state]++;
    }
  }
  assert.deepEqual(frame.bounds, [-63,-63,63,63]);
}
let projectedCorners = 0;
for (const [view, frame] of zoom.frames.entries()) {
  assert.equal(frame.cells.length, expected.length);
  for (let index = 0; index < expected.length; index++) {
    for (let corner = 0; corner < 4; corner++) {
      const source = expected[index].corners[corner]; const have = frame.cells[index].corners[corner];
      for (let axis = 0; axis < 2; axis++) {
        const wanted = source[axis] * zoom.zooms[view] + zoom.logical_viewport[axis] * .5;
        assert.ok(Math.abs(have[axis]-wanted) < .00003);
      }
      projectedCorners++;
    }
  }
}
const result = { scope: "Read-only independent validation of retained actual native packets; no extraction rebuild/GPU/Game claim", palette_entries: pack.palette.length, tileCounts, materialCounts, materials, stateChanged, lamps: lamps.size,
  exact_world_cells: expected.length * rig.frames.length, projectedCorners, bounds: rig.frames[0].bounds, transforms_per_frame: 16 };
writeFileSync(resolve(import.meta.dirname, "packet-inspection.json"), JSON.stringify(result, null, 2) + "\n");
console.log(JSON.stringify(result));
