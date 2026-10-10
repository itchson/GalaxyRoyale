import { readdir, readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { resolve, relative } from "node:path";
const dist = resolve(import.meta.dirname, "../dist");
async function files(dir) {
  const list = [];
  for (const item of await readdir(dir, { withFileTypes: true })) {
    const path = resolve(dir, item.name);
    if (item.isDirectory()) list.push(...(await files(path)));
    else if (item.name !== "sw.js") list.push(path);
  }
  return list;
}
const paths = (await files(dist)).sort();
const hash = createHash("sha256");
for (const path of paths) {
  hash.update(relative(dist, path).replaceAll("\\", "/"));
  hash.update(await readFile(path));
}
const cache = `galaxy-${hash.digest("hex").slice(0, 16)}`;
const assets = paths.map(
  (path) => "/" + relative(dist, path).replaceAll("\\", "/"),
);
await writeFile(
  resolve(dist, "sw.js"),
  `const CACHE=${JSON.stringify(cache)};const ASSETS=${JSON.stringify(assets)};self.addEventListener('install',event=>event.waitUntil(caches.open(CACHE).then(cache=>cache.addAll(ASSETS))));self.addEventListener('message',event=>{if(event.data==='ACTIVATE_AT_DOCK')self.skipWaiting();});self.addEventListener('activate',event=>event.waitUntil(caches.keys().then(keys=>Promise.all(keys.filter(key=>key.startsWith('galaxy-')&&key!==CACHE).map(key=>caches.delete(key))))));self.addEventListener('fetch',event=>{const url=new URL(event.request.url);if(event.request.method!=='GET'||url.origin!==self.location.origin||url.pathname.startsWith('/api/')||url.pathname.startsWith('/studio'))return;if(event.request.mode==='navigate'){event.respondWith(fetch(event.request).catch(()=>caches.match('/index.html')));return;}if(ASSETS.includes(url.pathname))event.respondWith(caches.match(url.pathname).then(cached=>cached||fetch(event.request)));});`,
);
console.log(`PWA: ${assets.length} game assets cached under ${cache}`);
