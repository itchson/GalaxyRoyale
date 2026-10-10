/** Real canvas lifecycle only; renderer reconstruction and Game state belong to the shell. */
export interface WebglLifecycle {
  readonly lost: boolean;
  readonly restoreReady: boolean;
  readonly disposed: boolean;
  dispose(): void;
}

export function bindWebglLifecycle(
  canvas: HTMLCanvasElement,
  callbacks: {
    lost(event: WebGLContextEvent): void;
    restored(event: WebGLContextEvent): void;
  },
): WebglLifecycle {
  let lost = false;
  let restoreReady = false;
  let disposed = false;
  let readyTimer: ReturnType<typeof setTimeout> | undefined;
  const clearReady = (): void => {
    if (readyTimer !== undefined) clearTimeout(readyTimer);
    readyTimer = undefined;
    restoreReady = false;
  };
  const onLost = (event: Event): void => {
    // Khronos requires cancellation to permit restoration. Do this before any
    // application callback, even if that callback throws.
    event.preventDefault();
    lost = true;
    clearReady();
    // restoreContext is invalid until the entire lost event dispatch ends.
    // A later task is deliberate; a microtask can run between DOM listeners.
    readyTimer = setTimeout(() => {
      readyTimer = undefined;
      if (!disposed && lost) restoreReady = true;
    }, 0);
    callbacks.lost(event as WebGLContextEvent);
  };
  const onRestored = (event: Event): void => {
    lost = false;
    clearReady();
    callbacks.restored(event as WebGLContextEvent);
  };
  canvas.addEventListener("webglcontextlost", onLost, { passive: false });
  canvas.addEventListener("webglcontextrestored", onRestored);
  return {
    get lost() {
      return lost;
    },
    get restoreReady() {
      return restoreReady;
    },
    get disposed() {
      return disposed;
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      clearReady();
      canvas.removeEventListener("webglcontextlost", onLost);
      canvas.removeEventListener("webglcontextrestored", onRestored);
    },
  };
}

export type WebglProbeRequest =
  | "requested"
  | "disposed"
  | "already-lost"
  | "not-lost"
  | "not-requested"
  | "await-lost-event"
  | "already-requested";

/** Optional QA control. Call only after this canvas has a working WebGL2 renderer.
 * Uses the real extension; no event is synthesized and no lifecycle flag is set.
 * A requested restoration is usable only after the actual restored callback.
 */
export function createWebglLossProbe(
  canvas: HTMLCanvasElement,
  lifecycle: WebglLifecycle,
): {
  requestLoss(): WebglProbeRequest;
  requestRestore(): WebglProbeRequest;
} | null {
  if (lifecycle.disposed) return null;
  const gl = canvas.getContext("webgl2");
  const extension = gl?.getExtension("WEBGL_lose_context");
  if (!gl || !extension) return null;
  let requestedLoss = false;
  let requestedRestore = false;
  return {
    requestLoss() {
      if (lifecycle.disposed) return "disposed";
      if (lifecycle.lost || gl.isContextLost()) return "already-lost";
      extension.loseContext();
      requestedLoss = true;
      requestedRestore = false;
      return "requested";
    },
    requestRestore() {
      if (lifecycle.disposed) return "disposed";
      if (!gl.isContextLost()) return "not-lost";
      // The extension can restore only a loss initiated through its own API.
      if (!requestedLoss) return "not-requested";
      if (!lifecycle.lost || !lifecycle.restoreReady) return "await-lost-event";
      if (requestedRestore) return "already-requested";
      extension.restoreContext();
      requestedRestore = true;
      return "requested";
    },
  };
}
