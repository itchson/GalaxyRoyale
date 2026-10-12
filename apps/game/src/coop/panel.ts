/** Semantic dock view only. Controller/native authority owns every session state. */
import { MAX_ROOM_CODE_LENGTH, normalizeRoomCode } from "./room-code.ts";
import type { JoinRequest } from "./transport.ts";
export interface CoopPanelState {
  phase:
    | "idle"
    | "creating"
    | "joining"
    | "hosting"
    | "admitting"
    | "loading"
    | "connected"
    | "ended";
  role: "host" | "guest" | null;
  members: number;
  transport_ready: number;
  admitted: number;
  reason: string | null;
  message: string;
  invite_available: boolean;
  joining_locked: boolean;
  awaiting_approval: boolean;
  join_requests: JoinRequest[];
  progression: false;
}

export interface CoopPanelActions {
  create(): Promise<void>;
  join(invite: string): Promise<void>;
  leave(): Promise<void>;
  end(): Promise<void>;
  /** Explicit capability accessor; never place its result in diagnostics or saves. */
  invite(): string | null;
  /** Called only by the user's COPY action. */
  copy(invite: string): Promise<void>;
  replaceCode?(): Promise<void>;
  setJoiningLocked?(locked: boolean): Promise<void>;
  approve?(peer: string, accept: boolean): Promise<void>;
  changed?(): void;
}

export const MAX_JOIN_LENGTH = MAX_ROOM_CODE_LENGTH;
const views = new WeakMap<HTMLElement, View>();
const phases = {
  idle: ["CO-OP AT HAVEN", "Enter a room code or host your universe."],
  creating: ["CREATING ROOM", "Waiting for the session service."],
  joining: ["JOINING ROOM", "Waiting for the host's approval."],
  hosting: ["HOSTING", "Share your room code. Accept each joining pilot."],
  admitting: ["WAITING FOR ADMISSION", "The host must admit this pilot."],
  loading: ["LOADING SHARED WORLD", "Waiting for a complete world snapshot."],
  connected: ["CONNECTED", "The shared world is active."],
  ended: ["SESSION ENDED", "You can start another session at Haven."],
} as const;
const reasons: Record<string, string> = {
  invalid_invite: "That room code is invalid or unavailable.",
  invite_not_found: "That room code was not found.",
  stale_invite: "That code was replaced. Ask the host for the new code.",
  joining_locked: "The host has locked joining.",
  code_replaced: "The host replaced the code. Request the new code to join.",
  join_declined: "The host declined your request.",
  approval_timeout: "The join request expired. You can request again.",
  reservation_timeout: "The connection could not start in time. Try again.",
  request_unavailable: "That request is no longer available.",
  session_full: "The session is full.",
  full: "The session is full.",
  not_at_dock: "Return to Haven to start co-op.",
  admission_rejected: "The host did not admit this pilot.",
  host_left: "The host ended the session.",
  session_ended: "The session ended.",
  session_expired: "The session expired.",
  host_ended: "The host ended the session.",
  host_end: "The host ended the session.",
  restore_failed: "Solo restoration failed.",
  replica_unavailable: "The shared-world view is unavailable.",
};

/** Shared canonical parser; short codes are lookup identifiers, not credentials. */
export function joinCode(value: string): string | null {
  return normalizeRoomCode(value);
}

interface View {
  root: HTMLElement;
  state: CoopPanelState;
  actions: CoopPanelActions;
  canStart: boolean;
  pending: "start" | "copy" | "exit" | null;
  request: number;
  retired: boolean;
  note: string;
  phase: CoopPanelState["phase"] | null;
  tab: "join" | "host";
  joinTab: HTMLButtonElement;
  hostTab: HTMLButtonElement;
  hostPane: HTMLElement;
  requests: HTMLElement;
  requestRows: Map<
    string,
    { root: HTMLElement; accept: HTMLButtonElement; decline: HTMLButtonElement }
  >;
  heading: HTMLElement;
  status: HTMLElement;
  members: HTMLElement;
  roster: HTMLElement;
  reason: HTMLElement;
  noteNode: HTMLElement;
  start: HTMLElement;
  invitation: HTMLElement;
  ending: HTMLElement;
  input: HTMLInputElement;
  output: HTMLInputElement;
  create: HTMLButtonElement;
  join: HTMLButtonElement;
  copy: HTMLButtonElement;
  replace: HTMLButtonElement;
  lock: HTMLButtonElement;
  leave: HTMLButtonElement;
  end: HTMLButtonElement;
}

function element(tag: string, className = "", value = ""): HTMLElement {
  const node = document.createElement(tag);
  node.className = className;
  node.textContent = value;
  return node;
}
function field(id: string, label: string, readonly = false): HTMLInputElement {
  const input = document.createElement("input");
  input.id = id;
  input.className = "coop-input";
  input.type = "text";
  input.readOnly = readonly;
  input.maxLength = readonly ? 10 : MAX_JOIN_LENGTH;
  input.autocomplete = "off";
  input.spellcheck = false;
  input.setAttribute("autocapitalize", "off");
  input.setAttribute("autocorrect", "off");
  input.setAttribute("aria-label", label);
  input.dataset.pixelKind = "input";
  input.setAttribute("data-private", "true");
  return input;
}
function action(id: string, label: string): HTMLButtonElement {
  const button = document.createElement("button");
  button.id = id;
  button.type = "button";
  button.textContent = label;
  return button;
}
function validState(state: CoopPanelState): boolean {
  return (
    Object.hasOwn(phases, state.phase) &&
    [null, "host", "guest"].includes(state.role) &&
    state.progression === false &&
    typeof state.joining_locked === "boolean" &&
    typeof state.awaiting_approval === "boolean" &&
    Array.isArray(state.join_requests) &&
    state.join_requests.length <= 7 &&
    state.join_requests.every(
      (r) =>
        r !== null &&
        typeof r === "object" &&
        /^[0-9a-f]{32}$/.test(r.id) &&
        Number.isSafeInteger(r.expires_at) &&
        r.expires_at >= 0 &&
        Number.isSafeInteger(r.room_revision) &&
        r.room_revision >= 0,
    ) &&
    new Set(state.join_requests.map((r) => r.id)).size ===
      state.join_requests.length &&
    [
      [state.members, 8],
      [state.transport_ready, 7],
      [state.admitted, 7],
    ].every(
      ([value, max]) => Number.isInteger(value) && value >= 0 && value <= max,
    )
  );
}

function render(view: View): boolean {
  let changed = false;
  const text = (node: HTMLElement, value: string) => {
    if (node.textContent !== value) {
      node.textContent = value;
      changed = true;
    }
  };
  const hidden = (node: HTMLElement, value: boolean) => {
    if (node.hidden !== value) {
      node.hidden = value;
      changed = true;
    }
  };
  const disabled = (
    node: HTMLButtonElement | HTMLInputElement,
    value: boolean,
  ) => {
    if (node.disabled !== value) {
      node.disabled = value;
      changed = true;
    }
  };
  const state = view.state;
  const valid = validState(state);
  const phaseChanged = view.phase !== state.phase;
  if (phaseChanged) {
    view.phase = state.phase;
    view.note = "";
    if (state.phase === "ended" || state.phase === "idle") {
      view.request++;
      view.pending = null;
      if (view.input.value) {
        view.input.value = "";
        changed = true;
      }
    }
  }
  const idle = valid && (state.phase === "idle" || state.phase === "ended");
  const active = valid && !idle;
  const restoreFailed = state.reason === "restore_failed";
  const recovering = valid && restoreFailed;
  const exitAvailable = active || recovering;
  const busy = view.pending !== null || !valid || restoreFailed;
  const copyable =
    valid &&
    state.role === "host" &&
    state.invite_available &&
    (state.phase === "hosting" || state.phase === "connected");
  let invite: string | null = null;
  if (copyable) {
    try {
      invite = view.actions.invite();
    } catch {
      /* No raw error or capability in copy. */
    }
    if (invite === null || joinCode(invite) !== invite) invite = null;
  }
  if (view.output.value !== (invite ?? "")) {
    view.output.value = invite ?? "";
    changed = true;
  }
  text(view.heading, valid ? phases[state.phase][0] : "SESSION UNAVAILABLE");
  hidden(view.heading, valid && state.phase === "idle");
  hidden(view.status, valid && state.phase === "idle" && !recovering);
  text(
    view.status,
    recovering
      ? "Retry restoration before returning to solo play."
      : valid
        ? phases[state.phase][1]
        : "Close the panel and try again.",
  );
  text(view.members, valid ? `${state.members} / 8 PILOTS` : "");
  const admitted =
    !valid || restoreFailed
      ? 0
      : state.role === "host" && active
        ? 1 + state.admitted
        : state.role === "guest" && state.phase === "connected"
          ? state.members
          : 0;
  const rosterValue = String(Math.max(0, Math.min(8, admitted)));
  if (view.roster.dataset.pixelValue !== rosterValue) {
    view.roster.dataset.pixelValue = rosterValue;
    changed = true;
  }
  text(
    view.reason,
    valid && state.reason !== null
      ? (reasons[state.reason] ?? "Connection stopped. You can try again.")
      : "",
  );
  text(view.noteNode, view.note);
  hidden(view.reason, !view.reason.textContent);
  hidden(view.noteNode, !view.note);
  hidden(view.start, view.tab !== "join" || !idle);
  hidden(view.hostPane, view.tab !== "host");
  hidden(view.invitation, invite === null);
  hidden(view.create, !idle);
  hidden(view.join, !idle);
  hidden(view.input, !idle);
  hidden(view.ending, !exitAvailable);
  hidden(
    view.end,
    !active || recovering || state.role !== "host" || view.tab !== "host",
  );
  hidden(view.replace, invite === null);
  hidden(view.lock, invite === null);
  text(
    view.leave,
    recovering
      ? "RETRY RESTORE"
      : state.phase === "creating" ||
          state.phase === "joining" ||
          state.phase === "admitting" ||
          state.phase === "loading"
        ? "CANCEL"
        : "LEAVE SESSION",
  );
  disabled(view.create, busy || !view.canStart || !idle || view.tab !== "host");
  disabled(view.input, busy || !view.canStart || !idle);
  disabled(
    view.join,
    busy ||
      !view.canStart ||
      !idle ||
      view.tab !== "join" ||
      joinCode(view.input.value) === null,
  );
  disabled(view.copy, busy || invite === null || view.tab !== "host");
  disabled(
    view.replace,
    busy || invite === null || view.tab !== "host" || !view.actions.replaceCode,
  );
  disabled(
    view.lock,
    busy ||
      invite === null ||
      view.tab !== "host" ||
      !view.actions.setJoiningLocked,
  );
  text(view.lock, state.joining_locked ? "UNLOCK JOINING" : "LOCK JOINING");
  for (const [button, selected] of [
    [view.joinTab, view.tab === "join"],
    [view.hostTab, view.tab === "host"],
  ] as const) {
    if (button.getAttribute("aria-selected") !== String(selected)) {
      button.setAttribute("aria-selected", String(selected));
      changed = true;
    }
  }
  const pending =
    valid && state.role === "host" && active && !restoreFailed
      ? state.join_requests
      : [];
  if ([...view.requestRows.keys()].join() !== pending.map((r) => r.id).join()) {
    view.requestRows.clear();
    for (const request of pending) {
      const row = element("div", "loadout-actions coop-request");
      const label = element(
        "p",
        "loadout-note",
        `PILOT ${request.id.slice(-8).toUpperCase()}`,
      );
      const accept = action(`coop-accept-${request.id}`, "ACCEPT");
      const decline = action(`coop-decline-${request.id}`, "DECLINE");
      accept.dataset.pixelAction = "primary-button";
      row.append(label, accept, decline);
      view.requestRows.set(request.id, { root: row, accept, decline });
      for (const [button, accepted] of [
        [accept, true],
        [decline, false],
      ] as const) {
        button.addEventListener("click", () => {
          if (
            !button.disabled &&
            !view.retired &&
            view.state.role === "host" &&
            view.state.join_requests.some((r) => r.id === request.id) &&
            view.actions.approve
          )
            invoke(view, () => view.actions.approve!(request.id, accepted));
        });
      }
    }
    view.requests.replaceChildren(
      ...[...view.requestRows.values()].map((r) => r.root),
    );
    changed = true;
  }
  hidden(view.requests, pending.length === 0);
  for (const row of view.requestRows.values()) {
    disabled(row.accept, busy || !view.actions.approve || view.tab !== "host");
    disabled(row.decline, busy || !view.actions.approve || view.tab !== "host");
  }
  disabled(view.leave, view.pending === "exit" || !exitAvailable);
  disabled(
    view.end,
    view.pending === "exit" ||
      !active ||
      recovering ||
      state.role !== "host" ||
      view.tab !== "host",
  );
  const availability = restoreFailed
    ? "Solo play stays blocked until restoration succeeds."
    : idle && !view.canStart
      ? "Return to Haven to host or join."
      : "Native RAICHO stats; primary fire only. Solo progress is restored on exit. Shared rewards are disabled.";
  const guidance = view.root.querySelector<HTMLElement>(".coop-guidance")!;
  text(guidance, availability);
  if (view.root.getAttribute("aria-busy") !== String(view.pending !== null)) {
    view.root.setAttribute("aria-busy", String(view.pending !== null));
    changed = true;
  }
  return changed;
}

function refresh(view: View): void {
  if (!view.retired && render(view)) view.actions.changed?.();
}
function invoke(
  view: View,
  run: () => Promise<void>,
  mode: "start" | "copy" | "exit" = "start",
): void {
  if (
    view.retired ||
    view.pending === "exit" ||
    (view.pending !== null && mode !== "exit")
  )
    return;
  view.pending = mode;
  view.note = "";
  const request = ++view.request;
  refresh(view);
  // No local phase transition: only a controller update may report success/admission.
  // Invoke in the actual click, including clipboard user activation. A later
  // teardown cannot cause a queued callback to start a fresh session.
  let result: Promise<void>;
  try {
    result = run();
  } catch {
    result = Promise.reject();
  }
  void Promise.resolve(result).then(
    () => {
      if (view.retired || request !== view.request) return;
      view.pending = null;
      if (mode === "copy") view.note = "Room code copied.";
      refresh(view);
    },
    () => {
      if (view.retired || request !== view.request) return;
      view.pending = null;
      view.note =
        mode === "copy"
          ? "Copy unavailable. Select the room code and copy it."
          : "The action did not complete. Try again.";
      if (mode === "copy" && view.output.value) {
        view.output.focus();
        view.output.select();
      }
      refresh(view);
    },
  );
}

function build(
  target: HTMLElement,
  state: CoopPanelState,
  actions: CoopPanelActions,
): View {
  const root = element("div", "forge-view coop-panel");
  const heading = element("h3");
  const status = element("p", "loadout-note");
  const members = element("p", "collection-state");
  members.dataset.pixelKind = "collection-state";
  const roster = element("span", "coop-roster");
  roster.dataset.pixelKind = "co-op-roster";
  roster.dataset.pixelValue = "0";
  roster.setAttribute("aria-hidden", "true");
  const reason = element("p", "loadout-reason");
  const noteNode = element("p", "loadout-note");
  noteNode.setAttribute("role", "status");
  const start = element("section", "coop-start");
  const tabs = element("div", "menu-tabs coop-tabs");
  tabs.setAttribute("role", "tablist");
  const joinTab = action("coop-tab-join", "JOIN");
  const hostTab = action("coop-tab-host", "HOST");
  for (const button of [joinTab, hostTab]) button.setAttribute("role", "tab");
  tabs.append(joinTab, hostTab);
  const hostPane = element("section", "coop-host");
  const create = action("coop-create", "HOST ROOM");
  const inviteArt = element("span", "coop-symbol");
  inviteArt.dataset.pixelKind = "co-op-invite";
  inviteArt.setAttribute("aria-hidden", "true");
  const inviteLabel = element("span", "coop-button-label", "HOST ROOM");
  create.replaceChildren(inviteLabel, inviteArt);
  create.dataset.pixelAction = "primary-button";
  const label = element("label", "coop-label");
  label.append(element("p", "loadout-note", "ROOM CODE"));
  const input = field("coop-join-input", "Room code to join");
  input.placeholder = "#A7C2-9F34";
  label.append(input);
  const join = action("coop-join", "JOIN");
  const joinArt = element("span", "coop-symbol");
  joinArt.dataset.pixelKind = "co-op-join";
  joinArt.setAttribute("aria-hidden", "true");
  const joinLabel = element("span", "coop-button-label", "JOIN");
  join.replaceChildren(joinLabel, joinArt);
  const startActions = element("div", "loadout-actions");
  startActions.append(join);
  start.append(label, startActions);
  const invitation = element("section", "coop-invitation");
  const output = field("coop-invite-output", "Your room code", true);
  const copy = action("coop-copy", "COPY CODE");
  const replace = action("coop-replace", "REPLACE CODE");
  const lock = action("coop-lock", "LOCK JOINING");
  const copyActions = element("div", "loadout-actions");
  copyActions.append(copy, replace, lock);
  invitation.append(
    element("p", "loadout-note", "ROOM CODE"),
    output,
    copyActions,
  );
  const requests = element("section", "coop-requests");
  hostPane.append(create, invitation, requests);
  const ending = element("div", "loadout-actions");
  const leave = action("coop-leave", "LEAVE SESSION");
  const end = action("coop-end", "STOP HOSTING");
  ending.append(leave, end);
  root.append(
    heading,
    members,
    roster,
    status,
    reason,
    tabs,
    start,
    hostPane,
    ending,
    element("p", "loadout-note coop-guidance"),
    noteNode,
  );
  target.replaceChildren(root);
  const view: View = {
    root,
    state,
    actions,
    canStart: false,
    pending: null,
    request: 0,
    retired: false,
    note: "",
    phase: null,
    tab: "join",
    joinTab,
    hostTab,
    hostPane,
    requests,
    requestRows: new Map(),
    heading,
    status,
    members,
    roster,
    reason,
    noteNode,
    start,
    invitation,
    ending,
    input,
    output,
    create,
    join,
    copy,
    replace,
    lock,
    leave,
    end,
  };
  for (const [button, tab] of [
    [joinTab, "join"],
    [hostTab, "host"],
  ] as const) {
    button.addEventListener("click", () => {
      if (!view.retired && view.tab !== tab) {
        view.tab = tab;
        refresh(view);
      }
    });
  }
  create.addEventListener("click", () => {
    if (!view.create.disabled) invoke(view, () => view.actions.create());
  });
  const submit = () => {
    const code = joinCode(view.input.value);
    if (!view.join.disabled && code !== null)
      invoke(view, () => view.actions.join(code));
  };
  join.addEventListener("click", submit);
  input.addEventListener("input", () => refresh(view));
  input.addEventListener("keydown", (event) => {
    if (event.key === "Enter" && !event.isComposing) {
      event.preventDefault();
      submit();
    }
  });
  copy.addEventListener("click", () => {
    if (!view.copy.disabled && view.output.value) {
      const code = view.output.value;
      invoke(view, () => view.actions.copy(code), "copy");
    }
  });
  replace.addEventListener("click", () => {
    if (!view.replace.disabled && view.actions.replaceCode)
      invoke(view, () => view.actions.replaceCode!());
  });
  lock.addEventListener("click", () => {
    if (!view.lock.disabled && view.actions.setJoiningLocked)
      invoke(view, () =>
        view.actions.setJoiningLocked!(!view.state.joining_locked),
      );
  });
  leave.addEventListener("click", () => {
    if (!view.leave.disabled) invoke(view, () => view.actions.leave(), "exit");
  });
  end.addEventListener("click", () => {
    if (!view.end.disabled) invoke(view, () => view.actions.end(), "exit");
  });
  return view;
}

/** Keeps the real input, scroll and focus intact across controller/HUD updates. */
export function paintCoopPanel(
  target: HTMLElement,
  state: CoopPanelState,
  actions: CoopPanelActions,
  canStart = false,
): boolean {
  let view = views.get(target);
  let mounted = false;
  if (!view) {
    view = build(target, state, actions);
    views.set(target, view);
    mounted = true;
  }
  view.state = state;
  view.actions = actions;
  view.canStart = canStart;
  return render(view) || mounted;
}

/** Call on panel teardown; never preserves invitation/draft in saves or history. */
export function disposeCoopPanel(target: HTMLElement): void {
  const view = views.get(target);
  if (!view) return;
  view.retired = true;
  view.request++;
  view.input.value = "";
  view.output.value = "";
  view.requestRows.clear();
  views.delete(target);
  target.replaceChildren();
}
