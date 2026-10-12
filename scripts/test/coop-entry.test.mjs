import { test } from "node:test";
import assert from "node:assert/strict";
import { enterAfterSave } from "../../apps/game/src/coop/entry.ts";
const barrier = () => {
  let resolve;
  const promise = new Promise((r) => {
    resolve = r;
  });
  return { promise, resolve };
};
test("cancel during disk save never starts an invitation after the save completes", async () => {
  const disk = barrier();
  let current = true,
    entered = 0,
    flushed = 0;
  const request = enterAfterSave(
    () => disk.promise,
    async () => {
      flushed++;
    },
    () => current,
    async () => {
      entered++;
    },
  );
  const rejected = assert.rejects(request, /cancelled/);
  current = false;
  disk.resolve();
  await rejected;
  assert.equal(entered, 0);
  assert.equal(flushed, 0);
});
test("title or replacement controller during flush prevents a delayed join", async () => {
  const disk = barrier();
  let current = true,
    entered = 0;
  const request = enterAfterSave(
    async () => {},
    () => disk.promise,
    () => current,
    async () => {
      entered++;
    },
  );
  await Promise.resolve();
  const rejected = assert.rejects(request, /cancelled/);
  current = false;
  disk.resolve();
  await rejected;
  assert.equal(entered, 0);
});
test("a successful save and flush admit exactly one current request", async () => {
  const order = [];
  await enterAfterSave(
    async () => {
      order.push("save");
    },
    async () => {
      order.push("flush");
    },
    () => true,
    async () => {
      order.push("enter");
    },
  );
  assert.deepEqual(order, ["save", "flush", "enter"]);
});
