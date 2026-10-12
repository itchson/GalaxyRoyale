import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// Wait for every owned child to close, including after a partial startup fails.
export function supervise(commands, { cwd, spawnChild = spawn } = {}) {
  const children = [];
  let stopping = false, result = 0, forceTimer;
  const stop = () => {
    if (stopping) return;
    stopping = true;
    for (const child of children) if (child.exitCode === null && child.signalCode === null) child.kill();
    forceTimer = setTimeout(() => {
      for (const child of children) if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
    }, 5000);
    forceTimer.unref();
  };
  const done = Promise.all(commands.map(args => new Promise(resolveDone => {
    const child = spawnChild(process.execPath, args, { cwd, stdio: 'inherit', windowsHide: true });
    children.push(child);
    child.once('error', error => { console.error(error.message); result = 1; stop(); });
    child.once('close', (code) => {
      if (!stopping) { result = code || 1; stop(); }
      resolveDone();
    });
  }))).then(() => { clearTimeout(forceTimer); return result; });
  return { children, stop, done };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = resolve(import.meta.dirname, '..');
  const args = process.argv.slice(2);
  if (args.some(arg => arg !== '--studio-only')) throw Error('Usage: node scripts/dev.mjs [--studio-only]');
  const vite = resolve(root, 'node_modules/vite/bin/vite.js');
  const commands = [
    [resolve(root, 'apps/studio/server.mjs')],
    [vite, '--config', 'apps/studio/vite.config.ts'],
  ];
  if (!args.includes('--studio-only')) commands.push([vite, '--config', 'apps/game/vite.config.ts']);
  const run = supervise(commands, { cwd: root });
  process.once('SIGINT', run.stop);
  process.once('SIGTERM', run.stop);
  process.exitCode = await run.done;
  process.removeListener('SIGINT', run.stop);
  process.removeListener('SIGTERM', run.stop);
}
