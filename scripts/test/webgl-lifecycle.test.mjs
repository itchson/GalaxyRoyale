import test from "node:test";
import assert from "node:assert/strict";
import {
  bindWebglLifecycle,
  createWebglLossProbe,
} from "../../apps/game/src/webgl-lifecycle.ts";

// Unit EventTarget and extension fixtures exercise handler ordering only.
// They are not native WebGL context-loss or browser/GPU acceptance evidence.
class Canvas extends EventTarget {
  calls = [];
  context = null;
  getContext(kind) {
    this.calls.push(kind);
    return this.context;
  }
}
function event(canvas, type) {
  const e = new Event(type, { cancelable: true });
  Object.defineProperty(e, "statusMessage", { value: "fixture status" });
  canvas.dispatchEvent(e);
  return e;
}
const nextTask = () => new Promise((resolve) => setTimeout(resolve, 0));
function setup() {
  const canvas = new Canvas();
  const seen = [];
  const lifecycle = bindWebglLifecycle(canvas, {
    lost(e) {
      seen.push(["lost", e.defaultPrevented, lifecycle.lost, e.statusMessage]);
    },
    restored() {
      seen.push(["restored", lifecycle.lost]);
    },
  });
  return { canvas, lifecycle, seen };
}
function withProbe() {
  const state = setup();
  const counts = { loss: 0, restore: 0 };
  let lost = false;
  const extension = {
    loseContext() {
      counts.loss++;
      lost = true;
    },
    restoreContext() {
      counts.restore++;
    },
  };
  state.canvas.context = {
    isContextLost: () => lost,
    getExtension: (name) => {
      assert.equal(name, "WEBGL_lose_context");
      return extension;
    },
  };
  return {
    ...state,
    counts,
    extension,
    setContextLost(value) {
      lost = value;
    },
    probe: createWebglLossProbe(state.canvas, state.lifecycle),
  };
}

test("binding never acquires a context and cancels loss before reporting it", async () => {
  const { canvas, lifecycle, seen } = setup();
  assert.deepEqual(canvas.calls, []);
  assert.equal(lifecycle.lost, false);
  assert.equal(event(canvas, "webglcontextlost").defaultPrevented, true);
  assert.deepEqual(seen, [["lost", true, true, "fixture status"]]);
  assert.equal(lifecycle.restoreReady, false);
  await nextTask();
  assert.equal(lifecycle.restoreReady, true);
  lifecycle.dispose();
});

test("only the restored event clears lost state and cancels stale readiness", async () => {
  const { canvas, lifecycle, seen } = setup();
  event(canvas, "webglcontextlost");
  event(canvas, "webglcontextrestored");
  assert.deepEqual(seen[1], ["restored", false]);
  await nextTask();
  assert.equal(lifecycle.lost, false);
  assert.equal(lifecycle.restoreReady, false);
  lifecycle.dispose();
});

test("dispose removes both handlers, cancels timers and is idempotent", async () => {
  const { canvas, lifecycle, seen } = setup();
  event(canvas, "webglcontextlost");
  lifecycle.dispose();
  lifecycle.dispose();
  assert.equal(event(canvas, "webglcontextlost").defaultPrevented, false);
  event(canvas, "webglcontextrestored");
  await nextTask();
  assert.equal(lifecycle.disposed, true);
  assert.equal(lifecycle.restoreReady, false);
  assert.equal(seen.length, 1);
});

test("replaced canvases have independent lifecycle state", () => {
  const old = setup();
  const replacement = setup();
  event(old.canvas, "webglcontextlost");
  assert.equal(replacement.lifecycle.lost, false);
  old.lifecycle.dispose();
  event(replacement.canvas, "webglcontextlost");
  assert.equal(replacement.lifecycle.lost, true);
  replacement.lifecycle.dispose();
});

test("probe reports missing context or extension without requesting loss", () => {
  const { canvas, lifecycle } = setup();
  assert.equal(createWebglLossProbe(canvas, lifecycle), null);
  canvas.context = { getExtension: () => null };
  assert.equal(createWebglLossProbe(canvas, lifecycle), null);
  lifecycle.dispose();
  const calls = canvas.calls.length;
  assert.equal(createWebglLossProbe(canvas, lifecycle), null);
  assert.equal(canvas.calls.length, calls);
});

test("extension requests neither synthesize events nor prematurely restore lifecycle", async () => {
  const { canvas, lifecycle, seen, probe, counts, setContextLost } =
    withProbe();
  assert.deepEqual(canvas.calls, ["webgl2"]);
  assert.equal(probe.requestRestore(), "not-lost");
  assert.equal(probe.requestLoss(), "requested");
  assert.equal(probe.requestLoss(), "already-lost");
  assert.deepEqual(counts, { loss: 1, restore: 0 });
  assert.equal(lifecycle.lost, false);
  assert.deepEqual(seen, []);
  assert.equal(probe.requestRestore(), "await-lost-event");
  event(canvas, "webglcontextlost");
  assert.equal(probe.requestRestore(), "await-lost-event");
  await nextTask();
  assert.equal(probe.requestRestore(), "requested");
  assert.equal(probe.requestRestore(), "already-requested");
  assert.deepEqual(counts, { loss: 1, restore: 1 });
  assert.equal(lifecycle.lost, true);
  assert.equal(seen.length, 1);
  setContextLost(false);
  event(canvas, "webglcontextrestored");
  assert.equal(lifecycle.lost, false);
  assert.equal(probe.requestRestore(), "not-lost");
  lifecycle.dispose();
});

test("same loss callback cannot request restoration before dispatch completes", async () => {
  const canvas = new Canvas();
  let contextLost = false;
  let restores = 0;
  canvas.context = {
    isContextLost: () => contextLost,
    getExtension: () => ({
      loseContext() {
        contextLost = true;
      },
      restoreContext() {
        restores++;
      },
    }),
  };
  let inside;
  const lifecycle = bindWebglLifecycle(canvas, {
    lost() {
      inside = probe.requestRestore();
    },
    restored() {},
  });
  const probe = createWebglLossProbe(canvas, lifecycle);
  probe.requestLoss();
  event(canvas, "webglcontextlost");
  assert.equal(inside, "await-lost-event");
  assert.equal(restores, 0);
  await nextTask();
  assert.equal(probe.requestRestore(), "requested");
  assert.equal(restores, 1);
  lifecycle.dispose();
});

test("probe does not claim to restore a natural loss it did not request", async () => {
  const { canvas, lifecycle, probe, counts, setContextLost } = withProbe();
  setContextLost(true);
  event(canvas, "webglcontextlost");
  await nextTask();
  assert.equal(probe.requestRestore(), "not-requested");
  assert.deepEqual(counts, { loss: 0, restore: 0 });
  lifecycle.dispose();
});

test("disposed probes cannot call extensions", () => {
  const { lifecycle, probe, counts } = withProbe();
  lifecycle.dispose();
  assert.equal(probe.requestLoss(), "disposed");
  assert.equal(probe.requestRestore(), "disposed");
  assert.deepEqual(counts, { loss: 0, restore: 0 });
});

test("extension exceptions propagate and do not consume a restore request", async () => {
  const { canvas, lifecycle, probe, extension, counts } = withProbe();
  probe.requestLoss();
  event(canvas, "webglcontextlost");
  await nextTask();
  const restore = extension.restoreContext;
  extension.restoreContext = () => {
    throw new Error("extension rejected");
  };
  assert.throws(() => probe.requestRestore(), /extension rejected/);
  extension.restoreContext = restore;
  assert.equal(probe.requestRestore(), "requested");
  assert.equal(counts.restore, 1);
  lifecycle.dispose();
});

test("a later requested loss starts a fresh restore cycle", async () => {
  const { canvas, lifecycle, probe, counts, setContextLost } = withProbe();
  for (let cycle = 0; cycle < 2; cycle++) {
    assert.equal(probe.requestLoss(), "requested");
    event(canvas, "webglcontextlost");
    await nextTask();
    assert.equal(probe.requestRestore(), "requested");
    setContextLost(false);
    event(canvas, "webglcontextrestored");
  }
  assert.deepEqual(counts, { loss: 2, restore: 2 });
  lifecycle.dispose();
});
