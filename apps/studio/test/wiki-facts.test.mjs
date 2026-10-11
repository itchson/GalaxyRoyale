import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
const root = fileURLToPath(new URL("../../../", import.meta.url));
const catalog = JSON.parse(
  await readFile(join(root, "docs/wiki/catalog.json"), "utf8"),
);
const source = (path) => readFile(join(root, path), "utf8");
const data = (id) => catalog.entries.find((e) => e.id === id).data;
test("all eight Wiki champion identities, hulls, categories, seed grids and base hulls match the authoritative Rust table", async () => {
  const rust = await source("crates/game_core/src/campaign.rs");
  const rows = [
    ...rust.matchAll(
      /Champion \{\s*name: "([^"]+)",\s*species: "([^"]+)",\s*legacy_species: "([^"]+)",\s*identity: RosterIdentity \{\s*kind: KikaiKind::(\w+),\s*corporation: Corporation::(\w+),\s*sub_corporation: Corporation::(\w+),\s*hex_id: "([^"]+)",\s*\},\s*sponsor: Corporation::(\w+),\s*ship: "([^"]+)",\s*legacy_model: "([^"]+)",\s*ability: "([^"]+)",\s*silhouette: "([^"]+)",\s*family: (\d),\s*category: (\d),\s*grid: \[(-?\d+), (-?\d+)\],\s*color: \[(\d+), (\d+), (\d+)\],\s*health: ([\d.]+)/g,
    ),
  ];
  assert.equal(rows.length, 8);
  rows.forEach((m, index) => {
    const entry = catalog.entries.find((e) => e.id === "champion-" + index);
    assert.equal(entry.title, m[1]);
    assert.deepEqual(entry.data, {
      index,
      species: m[2],
      legacySpecies: m[3],
      identity: {
        kind: m[4].toLowerCase(),
        corporation: m[5].toLowerCase(),
        sub_corporation: m[6].toLowerCase(),
        hex_id: m[7],
        short_designation:
          { Dora: "A", Doro: "O", Talo: "T" }[m[4]] +
          m[5][0] +
          m[6][0] +
          "#" +
          m[7].toUpperCase(),
      },
      sponsor: m[8].toLowerCase(),
      ship: m[9],
      legacyModel: m[10],
      ability: m[11],
      silhouette: m[12],
      family: Number(m[13]),
      constructionCategory: Number(m[14]),
      grid: [Number(m[15]), Number(m[16])],
      color: [Number(m[17]), Number(m[18]), Number(m[19])],
      baseHull: Number(m[20]),
    });
  });
});
test("all seventeen ability IDs, effect categories and descriptions match stable Rust registry order", async () => {
  const rust = await source("crates/game_core/src/abilities.rs");
  const rows = [
    ...rust.matchAll(
      /def\(\s*"([^"]+)",\s*"([^"]+)",\s*(\d),\s*Blueprint::([^,]+),\s*Effect::(\w+),\s*"([^"]+)"/g,
    ),
  ];
  assert.equal(
    data("abilities").count,
    Number(/pub const COUNT: usize = (\d+)/.exec(rust)[1]),
  );
  assert.equal(rows.length, 17);
  assert.deepEqual(
    data("abilities").abilities,
    rows.map((m, index) => ({
      index,
      key: m[1],
      name: m[2],
      category: Number(m[3]),
      blueprint: m[4],
      effect: m[5],
      description: m[6],
    })),
  );
  assert.equal(
    data("ships").shipCount,
    Number(/pub const SHIP_COUNT: u8 = (\d+)/.exec(rust)[1]),
  );
  assert.equal(
    data("ships").royalId,
    Number(/pub const ROYAL_SHIP: u8 = (\d+)/.exec(rust)[1]),
  );
});
test("six biome labels/spawn mixes and seed/belt parameters agree with actual Rust generation", async () => {
  const rust = await source("crates/game_core/src/regions.rs");
  const names = [
    .../const NAMES:[\s\S]*?= \[([\s\S]*?)\];/
      .exec(rust)[1]
      .matchAll(/"([^"]+)"/g),
  ].map((m) => m[1]);
  const weights = [
    .../const WEIGHTS:[\s\S]*?= \[([\s\S]*?)\];/
      .exec(rust)[1]
      .matchAll(/\[(\d+), (\d+), (\d+)\]/g),
  ].map((m) => m.slice(1).map(Number));
  assert.equal(names.length, 6);
  assert.equal(weights.length, 6);
  names.forEach((name, i) => {
    assert.equal(
      catalog.entries.find((e) => e.id === "biome-" + i).title,
      name,
    );
    assert.deepEqual(data("biome-" + i), { kind: i, spawnWeights: weights[i] });
  });
  assert.equal(
    data("world-seed").regionSize,
    Number(/REGION_SIZE: f32 = ([\d.]+)/.exec(rust)[1]),
  );
  assert.equal(
    data("world-seed").hubBiomeRadius,
    Number(/HUB_RADIUS: f32 = ([\d.]+)/.exec(rust)[1]),
  );
  assert.match(rust, /inner: site.radius \+ 240\.0/);
  assert.match(rust, /outer = site.radius \+ 520\.0/);
  assert.match(rust, /recommended_level: \[1, 3, 5, 10, 12\]/);
});
test("boss generations, ship-level gates, save schema and pixel packing facts remain source-backed", async () => {
  const bosses = await source("crates/game_core/src/bosses.rs");
  const names = [
    .../pub fn name\([\s\S]*?\{([\s\S]*?)\}\r?\n/
      .exec(bosses)[1]
      .matchAll(/"([^"]+)"/g),
  ].map((m) => m[1]);
  assert.deepEqual(
    names,
    data("bosses").names.flatMap((family, i) =>
      family.map((name) => ["Dart", "Bastion", "Coil"][i] + " " + name),
    ),
  );
  const campaign = await source("crates/game_core/src/campaign.rs");
  assert.match(campaign, /while level < 100/);
  assert.match(campaign, /\[1, 3, 5, 8, 12\]/);
  assert.equal(data("ship-levels").maxLevel, 100);
  assert.deepEqual(data("ship-levels").rankGates, [1, 3, 5, 8, 12]);
  assert.match(campaign, /let mut remaining = 16/);
  const lib = await source("crates/game_core/src/lib.rs");
  assert.equal(
    data("cargo").schemaVersion,
    Number(/schema_version: (\d+),/.exec(lib)[1]),
  );
  assert.equal(
    data("sanctuaries").havenSafeRadius,
    Number(/SAFE_RADIUS: f32 = ([\d.]+)/.exec(lib)[1]),
  );
  const frame = await source("crates/engine_core/src/frame.rs");
  assert.equal(
    data("pixel-engine").transformSlots,
    Number(/MAX_BATCH_TRANSFORMS: usize = (\d+)/.exec(frame)[1]),
  );
  assert.match(frame, /size_of::<PixelInstance>\(\), 16/);
  assert.match(frame, /size_of::<GpuTransform>\(\), 32/);
});
test("Wiki variant cell counts match actual runtime atlas occupancy and provenance; missions keep player-facing terminology", async () => {
  for (const [id, path] of [
    ["riveter", "assets/pixels/riveter-atlas.json"],
    ["radiation-mote", "assets/pixels/mote-atlas.json"],
  ]) {
    const atlas = JSON.parse(await source(path));
    const body = atlas.sprites?.riveter || atlas;
    assert.equal(body.indices.filter((i) => i !== 0).length, data(id).occupied);
    assert.equal(body.width, 40);
    assert.equal(body.height, 40);
    assert.equal(atlas.palette.length, 64);
  }
  assert.ok(
    catalog.entries.some((e) => e.title === "Courier & bounty missions"),
  );
  assert.ok(
    catalog.entries.every((e) => !e.title.toLowerCase().includes("quest")),
  );
  assert.deepEqual(data("missions").events, ["courier", "bounty"]);
});

test("curated reader text retains UTF-8 symbols without legacy decoding artifacts", () => {
  assert.doesNotMatch(JSON.stringify(catalog), /Â|Ã|â/);
  assert.equal(data("world-seed").regionSize, 4200);
  assert.ok(
    catalog.entries
      .find((e) => e.id === "world-seed")
      .facts.some((f) => f.value.includes("±1,000,000")),
  );
});

test("courier guidance includes the actual Haven delivery and source-beacon claim stages", async () => {
  const lib = await source("crates/game_core/src/lib.rs");
  assert.match(lib, /if quest\.event == "courier"[\s\S]*?quest\.progress = 1/);
  assert.match(
    lib,
    /Deliver the retained cargo to Haven dock, then return here\./,
  );
  const entry = catalog.entries.find((e) => e.id === "missions");
  assert.match(
    entry.facts.find((f) => f.label === "Courier").value,
    /Collect the relay packet, visit Haven, then return to the source beacon to claim/,
  );
  assert.match(
    entry.sections.flatMap((s) => s.paragraphs).join(" "),
    /Visiting Haven marks the delivery complete; claiming[\s\S]*same source beacon/,
  );
});
