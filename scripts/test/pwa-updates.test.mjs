import test from "node:test";
import assert from "node:assert/strict";
import { watchPwaUpdates } from "../../apps/game/src/pwa-updates.ts";

class Worker extends EventTarget {
  constructor(state = "installing") {
    super();
    this.state = state;
  }
  change(state) {
    this.state = state;
    this.dispatchEvent(new Event("statechange"));
  }
}
class Registration extends EventTarget {
  waiting = null;
  installing = null;
  found(worker) {
    this.installing = worker;
    this.dispatchEvent(new Event("updatefound"));
  }
}
function fixture(controlled = true) {
  const registration = new Registration(),
    notifications = [];
  return {
    registration,
    notifications,
    start: () =>
      watchPwaUpdates(
        registration,
        () => controlled,
        (worker) => notifications.push(worker),
      ),
  };
}
test("already waiting update is offered immediately and never twice", () => {
  const f = fixture(),
    worker = new Worker("installed");
  f.registration.waiting = worker;
  f.registration.installing = worker;
  f.start();
  f.registration.dispatchEvent(new Event("updatefound"));
  assert.deepEqual(f.notifications, [worker]);
});
test("installation begun before registration resolves is still watched", () => {
  const f = fixture(),
    worker = new Worker();
  f.registration.installing = worker;
  f.start();
  worker.change("installed");
  assert.deepEqual(f.notifications, [worker]);
  f.registration.waiting = worker;
  f.registration.dispatchEvent(new Event("updatefound"));
  assert.deepEqual(f.notifications, [worker]);
});
test("installed state before waiting assignment still offers the actual worker", () => {
  const f = fixture(),
    worker = new Worker("installed");
  f.registration.installing = worker;
  f.start();
  assert.deepEqual(f.notifications, [worker]);
});
test("first installation and failed workers do not offer an update", () => {
  const fresh = fixture(false),
    worker = new Worker();
  fresh.start();
  fresh.registration.found(worker);
  worker.change("installed");
  assert.deepEqual(fresh.notifications, []);
  const failed = fixture(),
    broken = new Worker();
  failed.start();
  failed.registration.found(broken);
  broken.change("redundant");
  assert.deepEqual(failed.notifications, []);
});
test("later updates are observed and disposal detaches pending listeners", () => {
  const f = fixture(),
    first = new Worker(),
    second = new Worker();
  const dispose = f.start();
  f.registration.found(first);
  first.change("installed");
  f.registration.waiting = null;
  f.registration.found(second);
  dispose();
  second.change("installed");
  f.registration.waiting = second;
  f.registration.dispatchEvent(new Event("updatefound"));
  assert.deepEqual(f.notifications, [first]);
});
