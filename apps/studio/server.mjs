import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { resolve, extname } from 'node:path';
import { ProjectStore, ApiError, artifactPath } from './store.mjs';

export function makeServer(store) {
  const json = (res, status, value) => { res.writeHead(status, {'Content-Type':'application/json; charset=utf-8','Cache-Control':'no-store'}); res.end(JSON.stringify(value)); };
  return createServer(async (req, res) => {
    try {
      const host = (req.headers.host || '').split(':')[0];
      if (!['127.0.0.1','localhost'].includes(host)) throw new ApiError(403, 'Local requests only.');
      const url = new URL(req.url, 'http://127.0.0.1');
      if (req.method === 'GET' && url.pathname === '/api/project') return json(res, 200, store.snapshot());
      if (req.method === 'GET' && url.pathname === '/api/events') {
        res.writeHead(200, {'Content-Type':'text/event-stream','Cache-Control':'no-store','Connection':'keep-alive'});
        const update = revision => res.write(`event: update\ndata: ${JSON.stringify({revision})}\n\n`);
        store.listeners.add(update); update(store.project.revision);
        const heartbeat = setInterval(() => res.write(': keepalive\n\n'), 20000);
        res.on('close', () => { clearInterval(heartbeat); store.listeners.delete(update); }); return;
      }
      if (req.method === 'GET' && url.pathname === '/artifacts') {
        const path = await artifactPath(store.root, url.searchParams.get('path'));
        const types = {'.txt':'text/plain','.log':'text/plain','.json':'application/json','.png':'image/png','.jpg':'image/jpeg','.webp':'image/webp','.mp4':'video/mp4','.wav':'audio/wav','.mp3':'audio/mpeg'};
        res.writeHead(200, {'Content-Type':types[extname(path)] || 'application/octet-stream','X-Content-Type-Options':'nosniff','Content-Disposition':'inline','Cache-Control':'no-store'}); res.end(await readFile(path)); return;
      }
      if (!['POST','PATCH'].includes(req.method)) throw new ApiError(404, 'Not found.');
      if (req.headers.origin && !['http://localhost:4175','http://127.0.0.1:4175','http://localhost:4174','http://127.0.0.1:4174'].includes(req.headers.origin)) throw new ApiError(403, 'Mutation origin is not allowed.');
      if (!req.headers['content-type']?.startsWith('application/json')) throw new ApiError(415, 'Use application/json.');
      let body = ''; for await (const chunk of req) { body += chunk; if (body.length > 1024 * 1024) throw new ApiError(413, 'Request is too large.'); }
      let payload; try { payload = JSON.parse(body); } catch { throw new ApiError(400, 'Invalid JSON.'); }
      if (!payload || typeof payload !== 'object' || Array.isArray(payload)) throw new ApiError(400, 'Expected a JSON object.');
      let kind;
      if (req.method === 'PATCH' && /^\/api\/tasks\/[^/]+$/.test(url.pathname)) { kind = 'task.update'; payload.id = decodeURIComponent(url.pathname.split('/').at(-1)); }
      if (req.method === 'PATCH' && /^\/api\/feedback\/[^/]+$/.test(url.pathname)) { kind = 'feedback.update'; payload.id = decodeURIComponent(url.pathname.split('/').at(-1)); }
      if (req.method === 'POST') kind = {'/api/tasks':'task.create','/api/reviews':'review','/api/decisions':'decision','/api/builds':'build','/api/feedback':'feedback','/api/runs':'run'}[url.pathname];
      if (!kind) throw new ApiError(404, 'Not found.');
      return json(res, req.method === 'POST' ? 201 : 200, await store.mutate(kind, payload));
    } catch (error) { json(res, error instanceof ApiError ? error.status : 500, { error: error instanceof ApiError ? error.message : 'Production service failed to persist the update.' }); }
  });
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = fileURLToPath(new URL('../../', import.meta.url));
  const store = await new ProjectStore(resolve(root, 'production/project.json'), root).load();
  makeServer(store).listen(4174, '127.0.0.1', () => console.log('Galaxy Royale production API: http://127.0.0.1:4174'));
}
