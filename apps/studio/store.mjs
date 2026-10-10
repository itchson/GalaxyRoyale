import { readFile, writeFile, rename, mkdir, realpath, stat } from 'node:fs/promises';
import { resolve, dirname, relative, sep } from 'node:path';
import { randomUUID } from 'node:crypto';

export const statuses = ['backlog', 'ready', 'active', 'review', 'complete', 'blocked'];
export const runStates = ['running', 'completed', 'interrupted', 'failed', 'blocked'];
const artifactRoots = ['work', 'artifacts', 'dist', 'apps/game/dist'];
export class ApiError extends Error { constructor(status, message) { super(message); this.status = status; } }
const requireValue = (condition, message) => { if (!condition) throw new ApiError(400, message); };
const cleanString = (v, field, max = 5000) => { requireValue(typeof v === 'string' && v.trim().length > 0 && v.length <= max, `${field} must be a nonempty string (maximum ${max} characters).`); return v.trim(); };

export async function artifactPath(root, input) {
  requireValue(typeof input === 'string' && input.length < 1000 && !input.includes('\0'), 'Invalid artifact path.');
  const requested = resolve(root, input);
  const allowed = artifactRoots.some(folder => { const rel = relative(resolve(root, folder), requested); return rel && rel !== '..' && !rel.startsWith(`..${sep}`) && !rel.startsWith('/') && !/^[A-Za-z]:/.test(rel); });
  requireValue(allowed, 'Artifacts must be inside work/, artifacts/, dist/, or apps/game/dist/.');
  let actual;
  try { actual = await realpath(requested); } catch { throw new ApiError(404, 'Artifact does not exist.'); }
  const escaped = relative(resolve(root), actual);
  requireValue(escaped && escaped !== '..' && !escaped.startsWith(`..${sep}`) && !/^[A-Za-z]:/.test(escaped), 'Artifact resolves outside the workspace.');
  // Resolve each allowlisted directory too: symlinks cannot broaden the allowlist.
  requireValue(artifactRoots.some(folder => { const rel = relative(resolve(root, folder), actual); return rel && rel !== '..' && !rel.startsWith(`..${sep}`) && !/^[A-Za-z]:/.test(rel); }), 'Artifact resolves outside approved directories.');
  requireValue((await stat(actual)).isFile(), 'Artifact must be a file.');
  return actual;
}

function validateEvidence(evidence) {
  requireValue(Array.isArray(evidence) && evidence.length <= 100, 'Evidence must be an array.');
  return evidence.map(item => {
    requireValue(item && typeof item === 'object', 'Invalid evidence item.');
    const result = { label: cleanString(item.label, 'Evidence label', 300), kind: item.kind || 'note' };
    if (item.path) result.path = cleanString(item.path, 'Evidence path', 1000);
    if (item.url) {
      let u; try { u = new URL(item.url, 'http://127.0.0.1:4175'); } catch { throw new ApiError(400, 'Invalid evidence URL.'); }
      requireValue(['http:', 'https:'].includes(u.protocol), 'Evidence links must use HTTP or HTTPS.');
      result.url = u.href;
    }
    if (item.details) result.details = cleanString(item.details, 'Evidence details');
    requireValue(result.path || result.url || result.details, 'Evidence requires a file, URL, or descriptive result.');
    return result;
  });
}

function validateTask(task, project) {
  task.title = cleanString(task.title, 'Title', 300);
  task.department = cleanString(task.department, 'Department', 100);
  requireValue(statuses.includes(task.status), 'Unknown task status.');
  requireValue(['normal', 'high', 'low'].includes(task.priority), 'Unknown priority.');
  requireValue(typeof task.owner === 'string' && task.owner.length < 200, 'Invalid owner.');
  requireValue(typeof task.description === 'string' && task.description.length < 10000, 'Invalid description.');
  requireValue(Array.isArray(task.dependencies) && task.dependencies.every(id => typeof id === 'string' && id !== task.id && project.tasks.some(t => t.id === id)), 'Dependencies must reference other existing tasks.');
  const visit = (id, seen) => { if (id === task.id) throw new ApiError(400, 'Task dependencies cannot form a cycle.'); if (seen.has(id)) return; seen.add(id); for (const d of project.tasks.find(t => t.id === id)?.dependencies || []) visit(d, seen); };
  for (const id of task.dependencies) visit(id, new Set());
  requireValue(Array.isArray(task.acceptanceCriteria) && task.acceptanceCriteria.length <= 40, 'Invalid acceptance criteria.');
  task.acceptanceCriteria = task.acceptanceCriteria.map(c => ({ text: cleanString(c.text, 'Acceptance criterion', 1000), met: c.met === true }));
  task.evidence = validateEvidence(task.evidence);
  requireValue(typeof task.blocker === 'string' && task.blocker.length < 5000, 'Invalid blocker.');
  if (task.status === 'blocked') requireValue(task.blocker.trim(), 'Blocked tasks need a recorded blocker.');
  if (task.status === 'active') requireValue(task.owner.trim(), 'Active tasks need an owner.');
  if (task.status === 'complete') {
    requireValue(task.acceptanceCriteria.length && task.acceptanceCriteria.every(c => c.met), 'Completion requires all acceptance criteria to be met.');
    requireValue(task.evidence.length, 'Completion requires recorded evidence.');
    requireValue(task.dependencies.every(id => project.tasks.find(t => t.id === id)?.status === 'complete'), 'Completion requires completed dependencies.');
  }
}

export class ProjectStore {
  constructor(file, root) { this.file = file; this.root = root; this.queue = Promise.resolve(); this.listeners = new Set(); }
  async load() {
    const project = JSON.parse(await readFile(this.file, 'utf8'));
    requireValue(project.schemaVersion === 1 && Array.isArray(project.tasks), 'Unsupported production store schema.');
    if (project.runRecords === undefined) project.runRecords = [];
    requireValue(Array.isArray(project.runRecords), 'Invalid recorded run history.');
    this.project = project; return this;
  }
  snapshot() { return structuredClone(this.project); }
  async mutate(kind, payload) {
    const operation = this.queue.then(() => this.apply(kind, payload));
    this.queue = operation.catch(() => {}); return operation;
  }
  async apply(kind, payload) {
    requireValue(payload && typeof payload === 'object', 'Expected a JSON object.');
    const next = this.snapshot();
    const author = cleanString(payload.author, 'Author', 200);
    const source = cleanString(payload.source, 'Source', 1000);
    const now = new Date().toISOString();
    let item;
    if (kind === 'task.create') {
      item = { id: `task-${randomUUID().slice(0, 8)}`, title: '', description: '', department: 'Production', owner: '', status: 'backlog', priority: 'normal', dependencies: [], acceptanceCriteria: [], evidence: [], blocker: '', revision: 1, createdAt: now, updatedAt: now };
      Object.assign(item, Object.fromEntries(Object.entries(payload.task || {}).filter(([key]) => ['title','description','department','owner','status','priority','dependencies','acceptanceCriteria','evidence','blocker'].includes(key))));
      validateTask(item, next); next.tasks.push(item);
    } else if (kind === 'task.update') {
      item = next.tasks.find(t => t.id === payload.id);
      if (!item) throw new ApiError(404, 'Task not found.');
      if (payload.expectedRevision !== item.revision) throw new ApiError(409, `Task changed. Current revision is ${item.revision}; refresh before editing.`);
      requireValue(payload.patch && typeof payload.patch === 'object', 'Expected a task patch.');
      const keys = ['title','description','department','owner','status','priority','dependencies','acceptanceCriteria','evidence','blocker'];
      requireValue(Object.keys(payload.patch).every(k => keys.includes(k)), 'Task patch contains unsupported fields.');
      Object.assign(item, payload.patch); validateTask(item, next); item.revision++; item.updatedAt = now;
    } else if (kind === 'run') {
      const input = payload.record;
      requireValue(input && typeof input === 'object', 'Expected a run record.');
      const agentId = cleanString(input.agentId, 'Agent identity', 200);
      requireValue(/^[A-Za-z0-9/][A-Za-z0-9_./:-]*$/.test(agentId), 'Agent identity must be a stable identifier without spaces.');
      const agentLabel = cleanString(input.agentLabel, 'Agent label', 200);
      requireValue(runStates.includes(input.state), 'Unknown run state.');
      const observedAt = cleanString(input.observedAt, 'Observation timestamp', 100);
      const observationTime = Date.parse(observedAt);
      requireValue(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$/.test(observedAt) && Number.isFinite(observationTime) && new Date(observationTime).toISOString().replace('.000Z','Z') === observedAt.replace('.000Z','Z'), 'Observation timestamp must be a valid UTC ISO date.');
      requireValue(observationTime <= Date.parse(now) + 30000, 'Observation timestamp cannot be in the future.');
      requireValue(Array.isArray(input.taskIds) && input.taskIds.length > 0 && input.taskIds.length <= 40 && new Set(input.taskIds).size === input.taskIds.length && input.taskIds.every(id => next.tasks.some(t => t.id === id)), 'Run task IDs must uniquely reference existing tasks.');
      const evidence = validateEvidence(input.evidence || []);
      requireValue(evidence.length > 0, 'Run states require evidence describing the actual observation.');
      item = { id:`run-${randomUUID().slice(0,8)}`, title:`${agentLabel} · ${input.state}`, agentId, agentLabel, state:input.state, observedAt:new Date(observationTime).toISOString(), taskIds:input.taskIds, summary:cleanString(input.summary, 'Run summary', 5000), evidence, author, source, createdAt:now };
      next.runRecords.push(item);
    } else if (['review','decision','build','feedback'].includes(kind)) {
      const input = payload.record;
      requireValue(input && typeof input === 'object', 'Expected a record.');
      item = { id: `${kind}-${randomUUID().slice(0, 8)}`, title: cleanString(input.title, 'Record title', 300), body: cleanString(input.body, 'Record body', 20000), author, source, createdAt: now };
      if (kind === 'review') {
        requireValue(Array.isArray(input.participants) && input.participants.every(p => typeof p === 'string'), 'Review participants are required.');
        const taskIds = input.taskIds === undefined ? [] : input.taskIds;
        requireValue(Array.isArray(taskIds) && taskIds.length <= 40 && new Set(taskIds).size === taskIds.length && Array.from(taskIds).every(id => typeof id === 'string' && next.tasks.some(t => t.id === id)), 'Review task IDs must be an array of at most 40 unique existing task IDs.');
        item.participants = input.participants; item.taskIds = [...taskIds];
      }
      if (kind === 'decision') { requireValue(['proposed','accepted','rejected','superseded'].includes(input.status), 'Invalid decision status.'); item.status = input.status; }
      if (kind === 'build') {
        item.revision = cleanString(input.revision, 'Build source revision', 200); item.evidence = validateEvidence(input.evidence || []);
        requireValue(Array.isArray(input.checks) && input.checks.every(c => typeof c.name === 'string' && ['pass','fail','not_run'].includes(c.result)), 'Build checks must explicitly record pass, fail, or not_run.');
        item.checks = input.checks; if (input.url) item.url = validateEvidence([{label:'Playable build',url:input.url}])[0].url;
        item.knownIssues = typeof input.knownIssues === 'string' ? input.knownIssues : '';
      }
      if (kind === 'feedback') { item.status = 'queued'; item.revision = 1; item.response = ''; }
      next[`${kind === 'feedback' ? 'feedback' : kind + 's'}`].push(item);
    } else if (kind === 'feedback.update') {
      item = next.feedback.find(f => f.id === payload.id);
      if (!item) throw new ApiError(404, 'Feedback not found.');
      if (payload.expectedRevision !== item.revision) throw new ApiError(409, `Feedback changed. Current revision is ${item.revision}.`);
      requireValue(['queued','acknowledged','resolved'].includes(payload.status), 'Unknown feedback status.');
      item.status = payload.status; item.response = typeof payload.response === 'string' ? payload.response.slice(0, 10000) : item.response;
      if (item.status === 'resolved') requireValue(item.response.trim(), 'Resolved feedback requires a recorded response.');
      item.revision++; item.updatedAt = now;
    } else throw new ApiError(404, 'Unknown operation.');
    for (const evidence of item.evidence || []) if (evidence.path) await artifactPath(this.root, evidence.path);
    next.revision++; next.updatedAt = now;
    next.activity.push({ id: randomUUID(), revision: next.revision, kind, itemId: item.id, title: item.title, author, source, createdAt: now });
    await mkdir(dirname(this.file), { recursive: true });
    const temporary = `${this.file}.${randomUUID()}.tmp`;
    await writeFile(temporary, JSON.stringify(next, null, 2) + '\n', { encoding: 'utf8', flag: 'wx' });
    // Backup the previous complete document before replacing it; a failed write never updates memory.
    await writeFile(`${this.file}.backup`, JSON.stringify(this.project, null, 2) + '\n', 'utf8');
    await rename(temporary, this.file);
    this.project = next;
    for (const listener of this.listeners) listener(next.revision);
    return { item, revision: next.revision };
  }
}
