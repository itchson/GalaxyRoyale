import { readFile } from 'node:fs/promises';

const [resource, action, ...args] = process.argv.slice(2);
const base = 'http://127.0.0.1:4174';
try {
  let url = '/api/project', options;
  if (resource !== 'snapshot') {
    const flag = args.indexOf('--file');
    if (flag < 0 || !args[flag + 1]) throw new Error('Provide an explicit UTF-8 JSON input using --file <path>.');
    const payload = JSON.parse(await readFile(args[flag + 1], 'utf8'));
    if (['task','feedback'].includes(resource) && action === 'update') {
      const id = args[0]; if (!id || id.startsWith('--')) throw new Error('Update requires an ID.');
      url = `/api/${resource === 'task' ? 'tasks' : 'feedback'}/${encodeURIComponent(id)}`; options = {method:'PATCH'};
    } else if (action === 'create' && ['task','review','decision','build','feedback','run'].includes(resource)) {
      url = `/api/${resource === 'feedback' ? 'feedback' : resource + 's'}`; options = {method:'POST'};
    } else throw new Error('Usage: snapshot | task/feedback update <id> --file <json> | task/review/decision/build/feedback/run create --file <json>');
    options.headers = {'Content-Type':'application/json'}; options.body = JSON.stringify(payload);
  }
  const response = await fetch(base + url, options); const result = await response.json();
  if (!response.ok) throw new Error(`HTTP ${response.status}: ${result.error}`);
  console.log(JSON.stringify(result, null, 2));
} catch (error) {
  console.error(error.cause?.code === 'ECONNREFUSED' ? 'Production API is offline. Start npm run dev; the CLI never writes the store directly.' : error.message);
  process.exitCode = 1;
}
