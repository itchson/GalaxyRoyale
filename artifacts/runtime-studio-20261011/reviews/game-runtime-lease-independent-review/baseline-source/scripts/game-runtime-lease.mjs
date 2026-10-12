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
    const lease = await acquireGameLease();
    const release = () => { void lease.close().catch(error => console.error('Game runtime cleanup:', error.message)); };
    server.httpServer.once('close', release);
    server.httpServer.once('error', release);
  }
  return { name: 'galaxy-single-game-runtime', configureServer: reserve, configurePreviewServer: reserve };
}
