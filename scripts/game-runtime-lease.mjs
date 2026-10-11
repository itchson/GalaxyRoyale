import { createServer } from 'node:net';

// An OS-held lease spans checkouts and dev/preview ports. It cannot leave a stale
// lock after a crash, and an occupied port is never resolved by killing its owner.
export const GAME_LEASE_PORT = 4172;
export async function acquireGameLease(port = GAME_LEASE_PORT) {
  const server = createServer(socket => socket.destroy());
  server.maxConnections = 1;
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen({ host: '127.0.0.1', port, exclusive: true }, () => {
      server.removeListener('error', reject);
      resolve();
    });
  }).catch(error => {
    if (error.code === 'EADDRINUSE') {
      throw new Error(`Galaxy Royale game runtime is already reserved on 127.0.0.1:${port}. Stop the existing game before launching another. If another application owns this port, inspect it; never kill an unknown process.`, { cause: error });
    }
    throw error;
  });
  server.unref();
  let closing;
  return {
    port: server.address().port,
    close() {
      return closing ??= new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve()));
    },
  };
}

export function singleGameRuntime() {
  async function reserve(server) {
    const release = await retainProcessLease();
    const onClose = () => { void release().catch(error => console.error('Game runtime cleanup:', error.message)); };
    server.httpServer.once('close', onClose);
    server.httpServer.once('error', onClose);
    // Vite creates its replacement before closing the old server. References
    // share the same process lease across config rebundles during that handoff.
    // A never-listened dev server also needs explicit cleanup (no HTTP close).
    if (typeof server.close === 'function') {
      const close = server.close.bind(server);
      server.close = async (...args) => { try { return await close(...args); } finally { await release(); } };
    }
  }
  return { name: 'galaxy-single-game-runtime', configureServer: reserve, configurePreviewServer: reserve };
}

async function retainProcessLease() {
  const key = Symbol.for('galaxy-royale.game-runtime-lease.v1');
  const state = globalThis[key] ??= { owners: new Set(), opening: null, closing: null };
  while (state.closing) await state.closing;
  const owner = Symbol();
  state.owners.add(owner);
  state.opening ??= acquireGameLease();
  let lease;
  try { lease = await state.opening; } catch (error) {
    state.owners.delete(owner);
    if (!state.owners.size) state.opening = null;
    throw error;
  }
  let released;
  return () => released ??= (async () => {
    state.owners.delete(owner);
    if (state.owners.size) return;
    state.closing = lease.close();
    try { await state.closing; } finally { state.opening = null; state.closing = null; }
  })();
}
