/** Semantic dock view only. Controller/native authority owns every session state. */
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
  changed?(): void;
}

export const MAX_JOIN_LENGTH = 1100;
const views = new WeakMap<HTMLElement, View>();
const phases = {
  idle: ["CO-OP AT HAVEN", "Invite pilots or paste an invitation to join."],
  creating: ["CREATING INVITATION", "Waiting for the session service."],
  joining: ["JOINING SESSION", "Requesting a place in the session."],
  hosting: ["HOSTING", "Share your invitation with other pilots."],
  admitting: ["WAITING FOR ADMISSION", "The host must admit this pilot."],
  loading: ["LOADING SHARED WORLD", "Waiting for a complete world snapshot."],
  connected: ["CONNECTED", "The shared world is active."],
  ended: ["SESSION ENDED", "You can start another session at Haven."],
} as const;
const reasons: Record<string, string> = {
  invalid_invite: "That invitation is not valid.",
  invite_expired: "That invitation has expired.",
  session_full: "The session is full.",
  full: "The session is full.",
  not_at_dock: "Return to Haven to start co-op.",
  admission_rejected: "The host did not admit this pilot.",
  host_left: "The host ended the session.",
  session_ended: "The session ended.",
  session_expired: "The session expired.",
  host_ended: "The host ended the session.",
  restore_failed: "Solo restoration failed. Return to title.",
  replica_unavailable: "The shared-world view is unavailable.",
};

/** Mirrors the transport's bounded opaque hex envelope; never decodes its contents. */
export function joinCode(value: string): string | null {
  if (value.length > MAX_JOIN_LENGTH) return null;
  const code = value.trim();
  const hex = code.replace(/^#/, "");
  return /^[0-9a-f]{64,1024}$/i.test(hex) && hex.length % 2 === 0 ? code : null;
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
  heading: HTMLElement;
  status: HTMLElement;
  members: HTMLElement;
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
  input.maxLength = readonly ? 1025 : MAX_JOIN_LENGTH;
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
    if (invite === null || joinCode(invite) !== invite || invite.length > 1025)
      invite = null;
  }
  if (view.output.value !== (invite ?? "")) {
    view.output.value = invite ?? "";
    changed = true;
  }
  text(view.heading, valid ? phases[state.phase][0] : "SESSION UNAVAILABLE");
  text(
    view.status,
    valid ? phases[state.phase][1] : "Close the panel and try again.",
  );
  text(view.members, valid ? `${state.members} / 8 PILOTS` : "");
  text(
    view.reason,
    valid && state.reason !== null
      ? (reasons[state.reason] ?? "Connection stopped. You can try again.")
      : "",
  );
  text(view.noteNode, view.note);
  hidden(view.reason, !view.reason.textContent);
  hidden(view.noteNode, !view.note);
  hidden(view.start, !idle);
  hidden(view.invitation, invite === null);
  hidden(view.ending, !active);
  hidden(view.end, state.role !== "host");
  text(
    view.leave,
    state.phase === "creating" ||
      state.phase === "joining" ||
      state.phase === "admitting" ||
      state.phase === "loading"
      ? "CANCEL"
      : "LEAVE SESSION",
  );
  disabled(view.create, busy || !view.canStart || !idle);
  disabled(view.input, busy || !view.canStart || !idle);
  disabled(
    view.join,
    busy || !view.canStart || !idle || joinCode(view.input.value) === null,
  );
  disabled(view.copy, busy || invite === null);
  disabled(view.leave, view.pending === "exit" || !active);
  disabled(
    view.end,
    view.pending === "exit" || !active || state.role !== "host",
  );
  const availability = restoreFailed
    ? "Co-op actions are unavailable."
    : idle && !view.canStart
      ? "Return to Haven to invite or join."
      : "Solo progress is restored on exit. Shared rewards are not enabled yet.";
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
      if (mode === "copy") view.note = "Invitation copied.";
      refresh(view);
    },
    () => {
      if (view.retired || request !== view.request) return;
      view.pending = null;
      view.note =
        mode === "copy"
          ? "Copy unavailable. Select the invitation and copy it."
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
  const reason = element("p", "loadout-reason");
  const noteNode = element("p", "loadout-note");
  noteNode.setAttribute("role", "status");
  const start = element("section", "coop-start");
  const create = action("coop-create", "INVITE");
  create.dataset.pixelAction = "primary-button";
  const label = element("label", "coop-label");
  label.append(element("p", "loadout-note", "INVITATION CODE"));
  const input = field("coop-join-input", "Invitation code to join");
  label.append(input);
  const join = action("coop-join", "JOIN");
  const startActions = element("div", "loadout-actions");
  startActions.append(create, join);
  start.append(label, startActions);
  const invitation = element("section", "coop-invitation");
  const output = field("coop-invite-output", "Your private invitation", true);
  const copy = action("coop-copy", "COPY INVITATION");
  const copyActions = element("div", "loadout-actions");
  copyActions.append(copy);
  invitation.append(
    element("p", "loadout-note", "SHARE ONLY WITH YOUR PILOTS"),
    output,
    copyActions,
  );
  const ending = element("div", "loadout-actions");
  const leave = action("coop-leave", "LEAVE SESSION");
  const end = action("coop-end", "END SESSION");
  ending.append(leave, end);
  root.append(
    heading,
    members,
    status,
    reason,
    start,
    invitation,
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
    heading,
    status,
    members,
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
    leave,
    end,
  };
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
  views.delete(target);
  target.replaceChildren();
}
