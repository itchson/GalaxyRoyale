/** Watch both an existing installation and later updates before requesting one. */
export function watchPwaUpdates(
  registration: ServiceWorkerRegistration,
  controlled: () => boolean,
  ready: (worker: ServiceWorker) => void,
): () => void {
  const watched = new Map<ServiceWorker, () => void>();
  let announced: ServiceWorker | null = null;
  let disposed = false;
  const notify = (worker: ServiceWorker | null) => {
    if (!disposed && worker && controlled() && worker !== announced) {
      announced = worker;
      ready(worker);
    }
  };
  const watch = () => {
    notify(registration.waiting);
    const worker = registration.installing;
    if (!worker || watched.has(worker)) return;
    const changed = () => {
      if (worker.state === "installed") {
        // waiting can be assigned after this statechange. The installed worker
        // itself accepts the same dock-activation message in that narrow gap.
        notify(registration.waiting ?? worker);
      }
      if (["installed", "activated", "redundant"].includes(worker.state)) {
        worker.removeEventListener("statechange", changed);
        watched.delete(worker);
      }
    };
    watched.set(worker, changed);
    worker.addEventListener("statechange", changed);
    changed();
  };
  registration.addEventListener("updatefound", watch);
  watch();
  return () => {
    disposed = true;
    registration.removeEventListener("updatefound", watch);
    for (const [worker, changed] of watched)
      worker.removeEventListener("statechange", changed);
    watched.clear();
  };
}
