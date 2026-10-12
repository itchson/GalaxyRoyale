import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";
import * as roomCode from "../../apps/game/src/coop/room-code.ts";

const compiled = ts.transpileModule(
  readFileSync(
    new URL("../../apps/game/src/coop/panel.ts", import.meta.url),
    "utf8",
  ),
  {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
    },
  },
).outputText;

// Real panel code with a semantic DOM fixture. This does not test pixels/layout/network.
function fixture() {
  let writes = 0;
  const document = { activeElement: null };
  class Element {
    children = [];
    attributes = {};
    listeners = {};
    className = "";
    id = "";
    scrollTop = 0;
    readOnly = false;
    _text = "";
    _value = "";
    _hidden = false;
    _disabled = false;
    selectionStart = 0;
    selectionEnd = 0;
    selected = false;
    dataset = new Proxy(
      {},
      {
        set(target, key, value) {
          writes++;
          target[key] = value;
          return true;
        },
      },
    );
    get textContent() {
      return (
        this._text + this.children.map((node) => node.textContent).join("")
      );
    }
    set textContent(value) {
      writes++;
      this._text = value;
      this.children = [];
    }
    get value() {
      return this._value;
    }
    set value(value) {
      writes++;
      this._value = value;
    }
    get hidden() {
      return this._hidden;
    }
    set hidden(value) {
      writes++;
      this._hidden = value;
    }
    get disabled() {
      return this._disabled;
    }
    set disabled(value) {
      writes++;
      this._disabled = value;
    }
    append(...children) {
      writes++;
      this.children.push(...children);
    }
    replaceChildren(...children) {
      writes++;
      this.children = children;
    }
    setAttribute(key, value) {
      writes++;
      this.attributes[key] = value;
    }
    getAttribute(key) {
      return this.attributes[key] ?? null;
    }
    addEventListener(type, listener) {
      (this.listeners[type] ??= []).push(listener);
    }
    dispatch(type, fields = {}) {
      const event = {
        target: this,
        prevented: false,
        preventDefault() {
          this.prevented = true;
        },
        ...fields,
      };
      for (const listener of this.listeners[type] ?? []) listener(event);
      return event;
    }
    click() {
      return this.dispatch("click");
    }
    focus() {
      document.activeElement = this;
    }
    select() {
      this.selected = true;
      this.selectionStart = 0;
      this.selectionEnd = this.value.length;
    }
    querySelector(selector) {
      return (
        flatten(this).find(
          (node) =>
            selector.startsWith(".") &&
            node.className.split(" ").includes(selector.slice(1)),
        ) ?? null
      );
    }
  }
  document.createElement = (tagName) =>
    Object.assign(new Element(), { tagName });
  const exports = {};
  vm.runInNewContext(compiled, {
    exports,
    require: (path) => {
      assert.equal(path, "./room-code.ts");
      return roomCode;
    },
    document,
    console: {
      log() {
        throw Error("The panel must not log.");
      },
      error() {
        throw Error("The panel must not log.");
      },
    },
  });
  const target = new Element();
  const calls = [];
  const actions = {
    create: async () => {
      calls.push(["create"]);
    },
    join: async (value) => {
      calls.push(["join", value]);
    },
    leave: async () => {
      calls.push(["leave"]);
    },
    end: async () => {
      calls.push(["end"]);
    },
    invite: () => null,
    copy: async (value) => {
      calls.push(["copy", value]);
    },
    replaceCode: async () => {
      calls.push(["replace"]);
    },
    setJoiningLocked: async (locked) => {
      calls.push(["lock", locked]);
    },
    approve: async (peer, accept) => {
      calls.push(["approve", peer, accept]);
    },
    changed: () => {
      calls.push(["changed"]);
    },
  };
  return {
    ...exports,
    target,
    calls,
    actions,
    document,
    paint(state = idle(), available = true) {
      return exports.paintCoopPanel(target, state, actions, available);
    },
    hostTab() {
      this.node("coop-tab-host").click();
    },
    node(id) {
      return flatten(target).find((node) => node.id === id);
    },
    resetWrites() {
      writes = 0;
    },
    writes() {
      return writes;
    },
  };
}
function flatten(node) {
  return [node, ...node.children.flatMap(flatten)];
}
function visibleText(node) {
  return node.hidden
    ? ""
    : node._text + node.children.map(visibleText).join(" ");
}
function idle(overrides = {}) {
  return {
    phase: "idle",
    role: null,
    members: 0,
    transport_ready: 0,
    admitted: 0,
    reason: null,
    message: "",
    invite_available: false,
    joining_locked: false,
    awaiting_approval: false,
    join_requests: [],
    progression: false,
    ...overrides,
  };
}
function deferred() {
  let resolve, reject;
  const promise = new Promise((accept, fail) => {
    resolve = accept;
    reject = fail;
  });
  return { promise, resolve, reject };
}
const settle = async () => {
  await Promise.resolve();
  await Promise.resolve();
};
const code = "#A7C2-9F34";

test("room code normalizes eight hex digits and permits only bounded ASCII separators", () => {
  const f = fixture();
  assert.equal(f.joinCode("  " + code + "  "), code);
  assert.equal(f.joinCode("a7c2 9f34"), code);
  assert.equal(f.joinCode(code + " ".repeat(54)), code);
  assert.equal(f.joinCode(code + " ".repeat(55)), null);
  for (const invalid of [
    "",
    "ab".repeat(31),
    "a".repeat(65),
    "a".repeat(1026),
    "##" + code,
    "＃" + code,
    code + "z",
    "a b".repeat(32),
    "\u00a0" + code,
    "\ufeff" + code,
  ])
    assert.equal(f.joinCode(invalid), null);
});

test("default unavailable dock state cannot start sessions and exposes semantic input hooks", () => {
  const f = fixture();
  f.paintCoopPanel(f.target, idle(), f.actions);
  assert.equal(f.node("coop-create").disabled, true);
  assert.equal(f.node("coop-join-input").disabled, true);
  f.node("coop-create").click();
  assert.equal(f.calls.length, 0);
  assert.match(visibleText(f.target), /Return to Haven/);
  const input = f.node("coop-join-input");
  assert.equal(input.maxLength, 64);
  assert.equal(input.dataset.pixelKind, "input");
  assert.equal(input.getAttribute("aria-label"), "Room code to join");
  assert.equal(input.autocomplete, "off");
  assert.equal(input.spellcheck, false);
  assert.equal(f.node("coop-invite-output").readOnly, true);
  assert.equal(f.node("coop-invite-output").dataset.pixelKind, "input");
});

test("available session copy discloses current guest and reward limitations", () => {
  const f = fixture();
  f.paint();
  assert.match(visibleText(f.target), /Your local progress is restored on exit/);
  assert.match(visibleText(f.target), /Shared rewards are currently disabled/);
});

test("join forwards the full bounded code once, including Enter composition guard", async () => {
  const f = fixture();
  const waiting = deferred();
  f.actions.join = (value) => {
    f.calls.push(["join", value]);
    return waiting.promise;
  };
  f.paint();
  const input = f.node("coop-join-input");
  input.value = " " + code + " ";
  input.dispatch("input");
  assert.equal(f.node("coop-join").disabled, false);
  input.dispatch("keydown", { key: "Enter", isComposing: true });
  assert.equal(f.calls.filter(([kind]) => kind === "join").length, 0);
  assert.equal(
    input.dispatch("keydown", { key: "Enter", isComposing: false }).prevented,
    true,
  );
  f.node("coop-join").click();
  assert.deepEqual(
    f.calls.filter(([kind]) => kind === "join"),
    [["join", code]],
  );
  waiting.resolve();
  await settle();
  assert.match(visibleText(f.target), /JOIN/);
  assert.doesNotMatch(visibleText(f.target), /CONNECTED/);
});

test("create invokes inside the real click and promise success cannot invent hosting", async () => {
  const f = fixture();
  f.paint();
  f.hostTab();
  f.node("coop-create").click();
  assert.equal(f.calls[0][0], "changed");
  assert.equal(f.calls.filter(([kind]) => kind === "create").length, 1);
  f.node("coop-create").click();
  await settle();
  assert.equal(f.calls.filter(([kind]) => kind === "create").length, 1);
  assert.match(visibleText(f.target), /HOST ROOM/);
  assert.doesNotMatch(visibleText(f.target), /HOSTING|CONNECTED/);
});

test("only controller phases declare readiness, admission and coherent connection", () => {
  const f = fixture();
  for (const [phase, heading] of [
    ["creating", "CREATING ROOM"],
    ["joining", "JOINING ROOM"],
    ["admitting", "WAITING FOR ADMISSION"],
    ["loading", "LOADING SHARED WORLD"],
    ["connected", "CONNECTED"],
    ["hosting", "HOSTING"],
    ["ended", "SESSION ENDED"],
  ]) {
    f.paint(
      idle({
        phase,
        role: phase === "hosting" ? "host" : "guest",
        members: 8,
        transport_ready: 7,
        admitted: 7,
      }),
    );
    assert.match(visibleText(f.target), new RegExp(heading));
    assert.match(visibleText(f.target), /8 \/ 8 PILOTS/);
    if (phase !== "connected")
      assert.doesNotMatch(visibleText(f.target), /CONNECTED/);
  }
});

test("cancel supersedes a pending start and ignores its late rejection", async () => {
  const f = fixture();
  const waiting = deferred();
  f.actions.create = () => {
    f.paint(idle({ phase: "creating", role: "host", members: 1 }));
    return waiting.promise;
  };
  f.actions.leave = async () => {
    f.calls.push(["leave"]);
    f.paint(idle({ phase: "ended" }));
  };
  f.paint();
  f.hostTab();
  f.node("coop-create").click();
  assert.equal(f.node("coop-leave").textContent, "CANCEL");
  assert.equal(f.node("coop-leave").disabled, false);
  f.node("coop-leave").click();
  waiting.reject(Error("opaque-network-payload"));
  await settle();
  assert.equal(f.calls.filter(([kind]) => kind === "leave").length, 1);
  assert.match(visibleText(f.target), /SESSION ENDED/);
  assert.doesNotMatch(visibleText(f.target), /did not complete|opaque-network/);
});

test("guest cannot end and repeated exit clicks cannot invoke twice", async () => {
  const f = fixture();
  const waiting = deferred();
  f.actions.leave = () => {
    f.calls.push(["leave"]);
    return waiting.promise;
  };
  f.paint(idle({ phase: "connected", role: "guest", members: 2 }));
  assert.equal(f.node("coop-end").hidden, true);
  f.node("coop-end").click();
  f.node("coop-leave").click();
  f.node("coop-leave").click();
  assert.deepEqual(
    f.calls.filter(([kind]) => kind === "leave" || kind === "end"),
    [["leave"]],
  );
  waiting.resolve();
  await settle();
});

test("invitation is read only for an actual copyable host state and copied explicitly", async () => {
  const f = fixture();
  let reads = 0;
  f.actions.invite = () => {
    reads++;
    return code;
  };
  f.paint(idle({ phase: "admitting", role: "guest", invite_available: true }));
  assert.equal(reads, 0);
  f.paint(
    idle({
      phase: "hosting",
      role: "host",
      members: 1,
      invite_available: false,
    }),
  );
  assert.equal(reads, 0);
  f.paint(
    idle({
      phase: "hosting",
      role: "host",
      members: 1,
      invite_available: true,
    }),
  );
  assert.equal(f.node("coop-invite-output").value, code);
  assert.equal(f.calls.filter(([kind]) => kind === "copy").length, 0);
  for (const node of flatten(f.target)) {
    assert.equal(JSON.stringify(node.attributes).includes(code), false);
    assert.equal(JSON.stringify(node.dataset).includes(code), false);
  }
  assert.equal(visibleText(f.target).includes(code), false);
  f.hostTab();
  f.node("coop-copy").click();
  assert.deepEqual(
    f.calls.filter(([kind]) => kind === "copy"),
    [["copy", code]],
  );
  await settle();
  assert.match(visibleText(f.target), /Room code copied/);
});

test("clipboard rejection keeps full manual copy and never displays thrown capability", async () => {
  const f = fixture();
  f.actions.invite = () => code;
  f.actions.copy = async () => {
    throw Error(code);
  };
  f.paint(
    idle({
      phase: "hosting",
      role: "host",
      members: 1,
      invite_available: true,
    }),
  );
  f.hostTab();
  f.node("coop-copy").click();
  await settle();
  const output = f.node("coop-invite-output");
  assert.equal(f.document.activeElement, output);
  assert.equal(output.selected, true);
  assert.equal(output.selectionEnd, code.length);
  assert.match(visibleText(f.target), /Copy unavailable/);
  assert.equal(visibleText(f.target).includes(code), false);
});

test("unknown reasons and controller messages cannot leak secret or raw transport errors", async () => {
  const f = fixture();
  f.paint(
    idle({ phase: "ended", reason: code, message: "Network Error " + code }),
  );
  assert.match(visibleText(f.target), /Connection stopped/);
  assert.equal(visibleText(f.target).includes(code), false);
  f.paint(idle({ reason: "session_full" }));
  assert.match(visibleText(f.target), /The session is full/);
  f.actions.create = () => {
    throw Error(code);
  };
  f.hostTab();
  f.node("coop-create").click();
  await settle();
  assert.match(visibleText(f.target), /The action did not complete/);
  assert.equal(visibleText(f.target).includes(code), false);
});

test("unchanged repaint preserves input node, selection, focus, scroll and performs no writes", () => {
  const f = fixture();
  f.paint();
  const input = f.node("coop-join-input");
  input.value = code;
  input.dispatch("input");
  input.focus();
  input.selectionStart = 28;
  input.selectionEnd = 31;
  f.target.scrollTop = 73;
  f.resetWrites();
  assert.equal(f.paint(), false);
  assert.equal(f.writes(), 0);
  assert.equal(f.node("coop-join-input"), input);
  assert.equal(f.document.activeElement, input);
  assert.equal(input.selectionStart, 28);
  assert.equal(input.selectionEnd, 31);
  assert.equal(f.target.scrollTop, 73);
  assert.equal(f.paint(idle({ members: 1 })), true);
  assert.equal(f.node("coop-join-input"), input);
  assert.equal(input.value, code);
});

test("ended session and teardown clear fields and isolate late completion from a new view", async () => {
  const f = fixture();
  const waiting = deferred();
  f.actions.invite = () => code;
  f.actions.copy = () => waiting.promise;
  f.paint(
    idle({
      phase: "hosting",
      role: "host",
      members: 1,
      invite_available: true,
    }),
  );
  const oldOutput = f.node("coop-invite-output");
  const oldInput = f.node("coop-join-input");
  oldInput.value = code;
  f.hostTab();
  f.node("coop-copy").click();
  f.disposeCoopPanel(f.target);
  assert.equal(oldInput.value, "");
  assert.equal(oldOutput.value, "");
  assert.equal(f.target.children.length, 0);
  f.paint();
  waiting.resolve();
  await settle();
  assert.doesNotMatch(visibleText(f.target), /Room code copied/);
  assert.equal(f.node("coop-join-input").value, "");
  f.node("coop-join-input").value = code;
  f.paint(idle({ phase: "ended" }));
  assert.equal(f.node("coop-join-input").value, "");
  assert.equal(f.node("coop-invite-output").value, "");
});

test("invalid controller counts or progression fail closed without reading invitations", () => {
  const f = fixture();
  let reads = 0;
  f.actions.invite = () => {
    reads++;
    return code;
  };
  for (const patch of [
    { members: 9 },
    { members: -1 },
    { admitted: 8 },
    { transport_ready: 0.5 },
    { progression: true },
    { phase: "ready" },
    { role: "owner" },
  ]) {
    f.paint(
      idle({
        phase: "hosting",
        role: "host",
        invite_available: true,
        ...patch,
      }),
    );
    assert.match(visibleText(f.target), /SESSION UNAVAILABLE/);
    for (const id of [
      "coop-create",
      "coop-join",
      "coop-copy",
      "coop-leave",
      "coop-end",
    ])
      assert.equal(f.node(id).disabled, true);
  }
  assert.equal(reads, 0);
});

test("malformed capability getter remains hidden with no unsafe fallback", () => {
  const f = fixture();
  for (const value of ["abc", " " + code, "#" + "a".repeat(1026), null]) {
    f.actions.invite = () => value;
    f.paint(idle({ phase: "hosting", role: "host", invite_available: true }));
    assert.equal(f.node("coop-invite-output").value, "");
    assert.equal(f.node("coop-copy").disabled, true);
  }
  f.actions.invite = () => {
    throw Error(code);
  };
  f.paint(idle({ phase: "hosting", role: "host", invite_available: true }));
  assert.equal(f.node("coop-invite-output").value, "");
});

test("failed solo restoration keeps the actual reason and prevents another start", () => {
  const f = fixture();
  f.paint(idle({ phase: "ended", reason: "restore_failed" }));
  assert.match(visibleText(f.target), /Solo restoration failed/);
  assert.match(visibleText(f.target), /Solo play stays blocked/);
  assert.doesNotMatch(visibleText(f.target), /Solo progress is restored/);
  assert.equal(f.node("coop-create").disabled, true);
  assert.equal(f.node("coop-join-input").disabled, true);
  f.node("coop-create").click();
  assert.equal(f.calls.length, 0);
});

test("failed restoration exposes one explicit retry without claiming restored solo state", async () => {
  const f = fixture();
  const waiting = deferred();
  f.actions.leave = () => {
    f.calls.push(["leave"]);
    return waiting.promise;
  };
  const failed = idle({
    phase: "ended",
    role: "host",
    reason: "restore_failed",
  });
  f.paint(failed);
  const retry = f.node("coop-leave");
  assert.equal(retry.textContent, "RETRY RESTORE");
  assert.equal(retry.disabled, false);
  assert.match(visibleText(f.target), /RETRY RESTORE/);
  assert.doesNotMatch(
    visibleText(f.target),
    /start another session|END SESSION/,
  );
  assert.equal(f.node("coop-end").hidden, true);
  retry.click();
  retry.click();
  assert.equal(f.calls.filter(([name]) => name === "leave").length, 1);
  assert.equal(retry.disabled, true);
  waiting.resolve();
  await settle();
  // Only a controller update can report actual successful restoration.
  assert.match(visibleText(f.target), /Solo restoration failed/);
  assert.equal(f.node("coop-create").disabled, true);
  assert.equal(retry.disabled, false);
  f.paint(idle({ phase: "ended", reason: "local_left" }));
  f.hostTab();
  assert.equal(f.node("coop-create").disabled, false);
  assert.doesNotMatch(visibleText(f.target), /RETRY RESTORE/);
});

test("invalid state cannot expose recovery or invoke an exit callback", () => {
  const f = fixture();
  f.paint(idle({ phase: "ended", reason: "restore_failed", members: 9 }));
  assert.equal(f.node("coop-leave").disabled, true);
  f.node("coop-leave").click();
  assert.equal(f.calls.length, 0);
});

test("Join is the initial tab; tab selection and draft survive reactive updates", () => {
  const f = fixture();
  f.paint();
  assert.equal(f.node("coop-tab-join").getAttribute("aria-selected"), "true");
  assert.equal(f.target.querySelector(".coop-host").hidden, true);
  const input = f.node("coop-join-input");
  input.value = "a7c29f34";
  input.focus();
  input.selectionStart = 3;
  input.selectionEnd = 5;
  f.hostTab();
  assert.equal(f.node("coop-tab-host").getAttribute("aria-selected"), "true");
  assert.match(visibleText(f.target), /HOST ROOM/);
  f.paint(idle({ members: 1 }));
  assert.equal(f.node("coop-tab-host").getAttribute("aria-selected"), "true");
  f.node("coop-tab-join").click();
  assert.equal(f.node("coop-join-input"), input);
  assert.equal(input.value, "a7c29f34");
  assert.equal(input.selectionStart, 3);
  assert.equal(input.selectionEnd, 5);
});

test("host management sends explicit actions without inventing policy or stopping hosting", async () => {
  const f = fixture();
  f.actions.invite = () => code;
  const host = idle({
    phase: "hosting",
    role: "host",
    members: 2,
    invite_available: true,
  });
  f.paint(host);
  f.hostTab();
  f.node("coop-replace").click();
  await settle();
  f.node("coop-lock").click();
  await settle();
  assert.deepEqual(
    f.calls.filter(([k]) => k === "replace" || k === "lock"),
    [["replace"], ["lock", true]],
  );
  assert.equal(f.node("coop-lock").textContent, "LOCK JOINING");
  f.paint({ ...host, joining_locked: true });
  assert.equal(f.node("coop-lock").textContent, "UNLOCK JOINING");
  f.node("coop-lock").click();
  await settle();
  assert.deepEqual(f.calls.filter(([k]) => k === "lock").at(-1), [
    "lock",
    false,
  ]);
  assert.equal(
    f.calls.some(([k]) => k === "end" || k === "leave"),
    false,
  );
  assert.match(visibleText(f.target), /STOP HOSTING/);
});

test("only current host requests expose Accept/Decline and stale or disposed rows cannot act", async () => {
  const f = fixture(),
    id = "a".repeat(32),
    second = "b".repeat(32);
  const request = (id) => ({ id, expires_at: 21000, room_revision: 1 });
  const host = idle({
    phase: "hosting",
    role: "host",
    members: 3,
    join_requests: [request(id), request(second)],
  });
  f.paint(host);
  f.hostTab();
  const accept = f.node("coop-accept-" + id),
    decline = f.node("coop-decline-" + second);
  accept.click();
  await settle();
  decline.click();
  await settle();
  assert.deepEqual(
    f.calls.filter(([k]) => k === "approve"),
    [
      ["approve", id, true],
      ["approve", second, false],
    ],
  );
  f.paint({ ...host, join_requests: [] });
  accept.click();
  decline.click();
  assert.equal(f.calls.filter(([k]) => k === "approve").length, 2);
  f.paint(host);
  const retired = f.node("coop-accept-" + id);
  f.disposeCoopPanel(f.target);
  retired.click();
  assert.equal(f.calls.filter(([k]) => k === "approve").length, 2);
  f.paint({ ...host, role: "guest", awaiting_approval: true });
  f.hostTab();
  assert.equal(f.node("coop-accept-" + id), undefined);
});

test("approval and room failures use controlled useful copy", () => {
  const f = fixture();
  for (const [reason, text] of [
    ["stale_invite", /code was replaced/],
    ["joining_locked", /locked joining/],
    ["join_declined", /declined your request/],
    ["approval_timeout", /request expired/],
    ["reservation_timeout", /could not start in time/],
  ]) {
    f.paint(idle({ phase: "ended", reason }));
    assert.match(visibleText(f.target), text);
  }
  f.paint(idle({ phase: "joining", role: "guest", awaiting_approval: true }));
  assert.match(visibleText(f.target), /Waiting for the host's approval/);
});
