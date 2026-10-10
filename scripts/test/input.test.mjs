import test from "node:test";
import assert from "node:assert/strict";
import { FlightInput } from "../../apps/game/src/input.ts";

class Surface extends EventTarget {
  style = {};
  classList = { add() {}, remove() {} };
  rect = { left: 0, top: 0, width: 100, height: 100 };
  captures = [];
  setPointerCapture(id) {
    this.captures.push(id);
  }
  getBoundingClientRect() {
    return this.rect;
  }
}
function pointer(target, type, props) {
  const event = new Event(type, { cancelable: true });
  Object.assign(event, {
    pointerId: 1,
    pointerType: "touch",
    clientX: 50,
    clientY: 50,
    button: 0,
    buttons: 1,
    ...props,
  });
  target.dispatchEvent(event);
}
function setup() {
  globalThis.window = new EventTarget();
  const canvas = new Surface(),
    area = new Surface(),
    stick = new Surface(),
    fire = new Surface();
  let selected;
  const input = new FlightInput(
    canvas,
    area,
    stick,
    fire,
    (world) => (selected = world),
  );
  input.enabled = true;
  input.camera = [100, 200];
  input.viewport = [800, 360];
  return { input, canvas, area, fire, selected: () => selected };
}
test("simultaneous joystick and fire use independent pointer channels", () => {
  const { input, area, fire } = setup();
  pointer(area, "pointerdown", { pointerId: 3, clientX: 84 });
  pointer(fire, "pointerdown", { pointerId: 7 });
  assert.deepEqual(input.sample(), {
    joystick: [1, 0],
    destination: null,
    aim: null,
    fire: true,
  });
  pointer(area, "pointercancel", { pointerId: 3 });
  assert.deepEqual(input.sample(), {
    joystick: null,
    destination: null,
    aim: null,
    fire: true,
  });
  pointer(fire, "pointerup", { pointerId: 7 });
  assert.equal(input.sample().fire, false);
});
test("blur resets every interrupted input, including mouse buttons", () => {
  const { input, area, fire, canvas } = setup();
  pointer(area, "pointerdown", { clientX: 70 });
  pointer(fire, "pointerdown", { pointerId: 2 });
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    button: 2,
    buttons: 2,
  });
  window.dispatchEvent(new Event("blur"));
  assert.deepEqual(input.sample(), {
    joystick: null,
    destination: null,
    aim: null,
    fire: false,
  });
});
test("desktop destination maps from unshaken camera and is submitted once", () => {
  const { input, canvas, selected } = setup();
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    clientX: 75,
    clientY: 25,
  });
  assert.deepEqual(selected(), [300, 110]);
  assert.deepEqual(input.sample().destination, [300, 110]);
  assert.equal(input.sample().destination, null);
  pointer(canvas, "pointermove", {
    pointerType: "mouse",
    clientX: 90,
    clientY: 50,
    buttons: 3,
  });
  assert.deepEqual(input.sample(), {
    joystick: null,
    destination: [420, 200],
    aim: [420, 200],
    fire: true,
  });
  pointer(canvas, "pointercancel", { pointerType: "mouse" });
  assert.deepEqual(input.sample(), {
    joystick: null,
    destination: null,
    aim: null,
    fire: false,
  });
});
test("disabled or cancelled control cannot continue firing", () => {
  const { input, fire } = setup();
  pointer(fire, "pointerdown", {});
  input.enabled = false;
  assert.equal(input.sample().fire, false);
  pointer(fire, "lostpointercapture", {});
  input.enabled = true;
  assert.equal(input.sample().fire, false);
});
test("right mouse fires and Space is not a weapon shortcut", () => {
  const { input, canvas } = setup();
  const space = new Event("keydown", { cancelable: true });
  Object.assign(space, { key: " " });
  window.dispatchEvent(space);
  assert.equal(input.sample().fire, false);
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    button: 2,
    buttons: 2,
  });
  assert.equal(input.sample().fire, true);
  pointer(canvas, "pointerup", { pointerType: "mouse", button: 2, buttons: 0 });
  assert.equal(input.sample().fire, false);
});
test("campaign navigation submits a real course once and respects interrupted controls", () => {
  const { input, selected } = setup();
  const course = [4200, -8000];
  input.setCourse(course);
  course[0] = 0;
  assert.deepEqual(input.sample().destination, [4200, -8000]);
  assert.deepEqual(selected(), [4200, -8000]);
  assert.equal(input.sample().destination, null);
  input.setCourse([NaN, 1]);
  assert.equal(input.sample().destination, null);
  input.enabled = false;
  input.setCourse([1, 2]);
  input.enabled = true;
  assert.equal(input.sample().destination, null);
  input.setCourse([1, 2]);
  input.reset();
  assert.equal(input.sample().destination, null);
});
test("Haven can intercept even a short weapon press from mouse or touch without a lingering shot", () => {
  globalThis.window = new EventTarget();
  const canvas = new Surface(),
    area = new Surface(),
    stick = new Surface(),
    fire = new Surface();
  let intents = 0;
  let input;
  input = new FlightInput(
    canvas,
    area,
    stick,
    fire,
    () => {},
    () => {
      intents++;
      input.enabled = false;
      input.reset();
    },
  );
  input.enabled = true;
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    button: 2,
    buttons: 2,
  });
  assert.equal(intents, 1);
  assert.equal(input.sample().fire, false);
  assert.equal(input.sample().aim, null);
  input.enabled = true;
  pointer(fire, "pointerdown", { pointerId: 2 });
  assert.equal(intents, 2);
  assert.equal(input.sample().fire, false);
  assert.equal(input.sample().aim, null);
});

test("right mouse aims at its cursor world point without submitting a navigation course", () => {
  const { input, canvas, selected } = setup();
  canvas.rect = { left: 10, top: 20, width: 200, height: 100 };
  input.camera = [100, -200];
  input.viewport = [400, 200];
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    button: 2,
    buttons: 2,
    clientX: 160,
    clientY: 45,
  });
  assert.deepEqual(input.sample(), {
    joystick: null,
    destination: null,
    aim: [200, -250],
    fire: true,
  });
  assert.equal(selected(), undefined);
  assert.deepEqual(input.sample().aim, [200, -250]);
});

test("held stationary mouse aim follows the current unshaken camera and eased world viewport", () => {
  const { input, canvas } = setup();
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    button: 2,
    buttons: 2,
    clientX: 75,
    clientY: 25,
  });
  assert.deepEqual(input.sample().aim, [300, 110]);
  input.camera = [120, 230];
  input.viewport = [400, 180];
  const frame = input.sample();
  assert.deepEqual(frame.aim, [220, 185]);
  frame.aim[0] = -999;
  assert.deepEqual(input.sample().aim, [220, 185]);
  pointer(canvas, "pointermove", {
    pointerType: "mouse",
    buttons: 2,
    clientX: 25,
    clientY: 75,
  });
  assert.deepEqual(input.sample().aim, [20, 275]);
});

test("mouse aim stays held while the one-shot left destination is consumed and then released independently", () => {
  const { input, canvas, selected } = setup();
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    button: 0,
    buttons: 1,
    clientX: 75,
    clientY: 25,
  });
  pointer(canvas, "pointermove", {
    pointerType: "mouse",
    buttons: 3,
    clientX: 90,
    clientY: 50,
  });
  assert.deepEqual(input.sample(), {
    joystick: null,
    destination: [420, 200],
    aim: [420, 200],
    fire: true,
  });
  assert.deepEqual(input.sample().aim, [420, 200]);
  assert.equal(input.sample().destination, null);
  pointer(canvas, "pointerup", {
    pointerType: "mouse",
    button: 0,
    buttons: 2,
    clientX: 80,
    clientY: 30,
  });
  assert.deepEqual(input.sample().aim, [340, 128]);
  assert.deepEqual(selected(), [420, 200]);
  pointer(canvas, "pointerup", { pointerType: "mouse", button: 2, buttons: 0 });
  assert.deepEqual(input.sample(), {
    joystick: null,
    destination: null,
    aim: null,
    fire: false,
  });
});

test("keyboard thrust and mobile fire retain their own directions while mouse aim is a separate channel", () => {
  const { input, canvas, area, fire } = setup();
  const key = new Event("keydown", { cancelable: true });
  Object.assign(key, { key: "w" });
  window.dispatchEvent(key);
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    button: 2,
    buttons: 2,
    clientX: 75,
    clientY: 50,
  });
  assert.deepEqual(input.sample(), {
    joystick: [0, -1],
    destination: null,
    aim: [300, 200],
    fire: true,
  });
  window.dispatchEvent(new Event("blur"));
  pointer(area, "pointerdown", { pointerId: 3, clientX: 84 });
  pointer(fire, "pointerdown", { pointerId: 7 });
  assert.deepEqual(input.sample(), {
    joystick: [1, 0],
    destination: null,
    aim: null,
    fire: true,
  });
  pointer(canvas, "pointermove", {
    pointerType: "touch",
    buttons: 2,
    clientX: 0,
    clientY: 0,
  });
  assert.equal(input.sample().aim, null);
});

test("touch canvas releases cannot clear an independently held right-mouse aim", () => {
  const { input, canvas } = setup();
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    button: 2,
    buttons: 2,
    clientX: 75,
    clientY: 25,
  });
  pointer(canvas, "pointerup", { pointerType: "touch", buttons: 0 });
  assert.deepEqual(input.sample().aim, [300, 110]);
  assert.equal(input.sample().fire, true);
});

test("mouse release, cancellation, capture loss, reset and disable cannot resurrect stale aim", () => {
  for (const stop of [
    ({ canvas }) =>
      pointer(canvas, "pointermove", { pointerType: "mouse", buttons: 0 }),
    ({ canvas }) => pointer(canvas, "pointercancel", { pointerType: "mouse" }),
    ({ canvas }) =>
      pointer(canvas, "lostpointercapture", { pointerType: "mouse" }),
    ({ input }) => input.reset(),
    ({ input }) => {
      input.enabled = false;
      input.enabled = true;
    },
  ]) {
    const controls = setup();
    pointer(controls.canvas, "pointerdown", {
      pointerType: "mouse",
      button: 2,
      buttons: 2,
      clientX: 75,
      clientY: 25,
    });
    assert.deepEqual(controls.input.sample().aim, [300, 110]);
    stop(controls);
    assert.deepEqual(controls.input.sample(), {
      joystick: null,
      destination: null,
      aim: null,
      fire: false,
    });
  }
});

test("malformed or zero-size projections cannot send a nonfinite cursor target or destination", () => {
  for (const invalidate of [
    ({ canvas }) => {
      canvas.rect.width = 0;
    },
    ({ canvas }) => {
      canvas.rect.height = 0;
    },
    ({ input }) => {
      input.camera = [NaN, 0];
    },
    ({ input }) => {
      input.viewport = [Infinity, 360];
    },
    ({ input }) => {
      input.viewport = [800, -1];
    },
  ]) {
    const controls = setup();
    invalidate(controls);
    pointer(controls.canvas, "pointerdown", {
      pointerType: "mouse",
      button: 2,
      buttons: 2,
      clientX: 75,
      clientY: 25,
    });
    assert.equal(controls.input.sample().aim, null);
    pointer(controls.canvas, "pointerdown", {
      pointerType: "mouse",
      button: 0,
      buttons: 1,
      clientX: 75,
      clientY: 25,
    });
    assert.equal(controls.input.sample().destination, null);
    assert.equal(controls.selected(), undefined);
  }
});

test("pressing right during a captured left drag triggers Haven interception exactly once", () => {
  globalThis.window = new EventTarget();
  const canvas = new Surface(),
    area = new Surface(),
    stick = new Surface(),
    fire = new Surface();
  let input;
  let intents = 0;
  input = new FlightInput(
    canvas,
    area,
    stick,
    fire,
    () => {},
    () => {
      intents++;
      input.enabled = false;
    },
  );
  input.enabled = true;
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    button: 0,
    buttons: 1,
    clientX: 75,
  });
  pointer(canvas, "pointermove", {
    pointerType: "mouse",
    buttons: 3,
    clientX: 90,
  });
  assert.equal(intents, 1);
  assert.deepEqual(input.sample(), {
    joystick: null,
    destination: null,
    aim: null,
    fire: false,
  });
});

function interactionSetup(onInteract, onFireStart = () => {}) {
  globalThis.window = new EventTarget();
  const canvas = new Surface(),
    area = new Surface(),
    stick = new Surface(),
    fire = new Surface();
  const courses = [];
  const input = new FlightInput(
    canvas,
    area,
    stick,
    fire,
    (point) => courses.push(point),
    onFireStart,
    onInteract,
  );
  input.enabled = true;
  input.camera = [100, 200];
  input.viewport = [800, 360];
  return { input, canvas, area, fire, courses };
}

test("initial left facility press consumes the world point before movement capture and blocks its drag", () => {
  const interactions = [];
  const { input, canvas, courses } = interactionSetup((point) => {
    interactions.push(point);
    return true;
  });
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    clientX: 75,
    clientY: 25,
  });
  assert.deepEqual(interactions, [[300, 110]]);
  assert.deepEqual(canvas.captures, []);
  assert.equal(input.sample().destination, null);
  pointer(canvas, "pointermove", {
    pointerType: "mouse",
    clientX: 90,
    buttons: 1,
  });
  assert.equal(input.sample().destination, null);
  assert.deepEqual(courses, []);
  pointer(canvas, "pointerup", { pointerType: "mouse", buttons: 0 });
  pointer(canvas, "pointerdown", { pointerType: "mouse", clientX: 25 });
  assert.equal(interactions.length, 2);
});

test("declined facility press navigates normally and held left dragging never invokes interaction", () => {
  const interactions = [];
  const { input, canvas, courses } = interactionSetup((point) => {
    interactions.push(point);
    return false;
  });
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    clientX: 75,
    clientY: 25,
  });
  assert.deepEqual(canvas.captures, [1]);
  assert.deepEqual(input.sample().destination, [300, 110]);
  pointer(canvas, "pointermove", {
    pointerType: "mouse",
    clientX: 90,
    buttons: 1,
  });
  assert.deepEqual(input.sample().destination, [420, 200]);
  assert.equal(interactions.length, 1);
  assert.equal(courses.length, 2);
});

test("canvas touch facility taps consume while empty-space touches preserve joystick controls", () => {
  const interactions = [];
  const { input, canvas, area, courses } = interactionSetup((point) => {
    interactions.push(point);
    return point[0] === 300;
  });
  pointer(canvas, "pointerdown", { clientX: 75, clientY: 25 });
  pointer(canvas, "pointerup", { buttons: 0 });
  pointer(canvas, "pointerdown", { clientX: 25, clientY: 50 });
  pointer(canvas, "pointermove", { clientX: 0 });
  assert.deepEqual(interactions, [
    [300, 110],
    [-100, 200],
  ]);
  assert.deepEqual(canvas.captures, []);
  assert.deepEqual(courses, []);
  assert.equal(input.sample().destination, null);
  pointer(area, "pointerdown", { pointerId: 3, clientX: 84 });
  assert.deepEqual(input.sample().joystick, [1, 0]);
  assert.equal(interactions.length, 2);
});

test("right aim and mobile fire do not invoke facilities and retain optional fire-start compatibility", () => {
  let interactions = 0,
    starts = 0;
  const { input, canvas, fire } = interactionSetup(
    () => {
      interactions++;
      return true;
    },
    () => starts++,
  );
  pointer(canvas, "pointerdown", {
    pointerType: "mouse",
    button: 2,
    buttons: 2,
    clientX: 75,
  });
  assert.deepEqual(input.sample().aim, [300, 200]);
  assert.equal(input.sample().fire, true);
  pointer(canvas, "pointerup", { pointerType: "mouse", button: 2, buttons: 0 });
  pointer(fire, "pointerdown", { pointerId: 7 });
  assert.equal(input.sample().fire, true);
  assert.equal(interactions, 0);
  assert.equal(starts, 2);
});

test("facility callback disabling input cannot restore a course or consumed held gesture", () => {
  let input;
  const controls = interactionSetup(() => {
    input.enabled = false;
    return true;
  });
  input = controls.input;
  pointer(controls.canvas, "pointerdown", {
    pointerType: "mouse",
    clientX: 75,
  });
  assert.deepEqual(controls.canvas.captures, []);
  assert.deepEqual(input.sample(), {
    joystick: null,
    destination: null,
    aim: null,
    fire: false,
  });
  pointer(controls.canvas, "pointermove", { pointerType: "mouse", buttons: 1 });
  input.enabled = true;
  assert.deepEqual(input.sample(), {
    joystick: null,
    destination: null,
    aim: null,
    fire: false,
  });
});

test("invalid or disabled projection never invokes a facility callback", () => {
  let interactions = 0;
  const { input, canvas } = interactionSetup(() => {
    interactions++;
    return true;
  });
  canvas.rect.width = 0;
  pointer(canvas, "pointerdown", { pointerType: "mouse" });
  pointer(canvas, "pointerdown", { pointerType: "touch" });
  canvas.rect.width = 100;
  input.enabled = false;
  pointer(canvas, "pointerdown", { pointerType: "mouse" });
  assert.equal(interactions, 0);
});

test("release outside an uncaptured facility press admits the next initial click", () => {
  let interactions = 0;
  const { input, canvas } = interactionSetup(() => {
    interactions++;
    return true;
  });
  pointer(canvas, "pointerdown", { pointerType: "mouse", clientX: 75 });
  pointer(window, "pointerup", { pointerType: "mouse", buttons: 0 });
  pointer(canvas, "pointerdown", { pointerType: "mouse", clientX: 75 });
  assert.equal(interactions, 2);
  assert.equal(input.sample().destination, null);
  assert.deepEqual(canvas.captures, []);
});
