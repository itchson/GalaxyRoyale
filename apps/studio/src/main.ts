import './style.css';
import './wiki.css';
import { WikiView } from './wiki';

type Status = 'backlog' | 'ready' | 'active' | 'review' | 'complete' | 'blocked';
interface Evidence { label: string; kind: string; path?: string; url?: string; details?: string }
interface Task { id: string; title: string; description: string; department: string; owner: string; status: Status; priority: string; dependencies: string[]; acceptanceCriteria: {text:string;met:boolean}[]; evidence: Evidence[]; blocker: string; revision: number; updatedAt: string }
interface RecordItem { id: string; title: string; body: string; author: string; source: string; createdAt: string; status?: string; participants?: string[]; taskIds?: string[]; revision?: string; url?: string; evidence?: Evidence[]; checks?: {name:string;result:string;details?:string}[]; knownIssues?: string; response?: string }
interface Activity { id:string; revision:number; kind:string; itemId:string; title:string; author:string; source:string; createdAt:string }
interface RunRecord { id:string; agentId:string; agentLabel:string; state:'running'|'completed'|'interrupted'|'failed'|'blocked'; observedAt:string; createdAt:string; taskIds:string[]; summary:string; source:string; author:string; evidence:Evidence[] }
interface Project { schemaVersion:number; revision:number; name:string; phase:string; phaseDescription?:string; updatedAt:string; tasks:Task[]; reviews:RecordItem[]; decisions:RecordItem[]; builds:RecordItem[]; feedback:RecordItem[]; activity:Activity[]; runRecords:RunRecord[] }
const app = document.querySelector<HTMLDivElement>('#app')!;
const labels: Record<Status,string> = {backlog:'Backlog',ready:'To do',active:'In action',review:'Review',complete:'Completed',blocked:'Blocked'};
const nav = [['board','Mission board','▦'],['wiki','DevBible','◈'],['reviews','Review room','◇'],['decisions','Decisions','⌁'],['builds','Build archive','▣'],['activity','Activity log','≋'],['feedback','Feedback queue','↗']] as const;
let project: Project | null = null;
const wiki = new WikiView();
const initialRoute = /^#wiki(?:\/([a-z0-9-]+))?$/.exec(location.hash);
let view: string = initialRoute ? 'wiki' : 'board';
wiki.selected = initialRoute?.[1] || '';
let department = 'all';
let status = 'all';
let search = '';
let connected = false;
let loading = false;
let latestSeenRevision = 0;
const escape = (v: unknown) => String(v ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]!));
const date = (s: string) => new Intl.DateTimeFormat('en-AU', {timeZone:'Australia/Brisbane',day:'2-digit',month:'short',hour:'2-digit',minute:'2-digit'}).format(new Date(s));
const departmentClass = (s: string) => s.toLowerCase().replace(/[^a-z]/g, '');
function evidenceLink(e: Evidence) {
  const label = escape(e.label);
  if (e.path) return `<a href="/artifacts?path=${encodeURIComponent(e.path)}" target="_blank" rel="noopener">${label} ↗</a>`;
  if (e.url && /^https?:\/\//.test(e.url)) return `<a href="${escape(e.url)}" target="_blank" rel="noopener">${label} ↗</a>`;
  return `<span>${label}</span>`;
}
function toast(text: string, error = false) {
  document.querySelector('.toast')?.remove();
  const node = document.createElement('div'); node.className = `toast ${error ? 'error' : ''}`; node.setAttribute('role', error ? 'alert' : 'status'); node.textContent = text; document.body.append(node); setTimeout(() => node.remove(), 7000);
}
async function refresh() {
  if (loading) return;
  loading = true;
  try { const response = await fetch('/api/project', {cache:'no-store'}); if (!response.ok) throw new Error('API unavailable'); project = await response.json() as Project; connected = true; }
  catch { connected = false; }
  finally { loading = false; render(); if (connected && project && project.revision < latestSeenRevision) void refresh(); }
}
async function mutate(path: string, method: string, payload: unknown) {
  const response = await fetch(path, {method, headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)});
  const result = await response.json() as {error?:string};
  if (!response.ok) { if (response.status === 409) { await refresh(); throw new Error('This record changed elsewhere. Close and reopen it to reconcile the latest revision. Your changes were not saved.'); } throw new Error(result.error || 'Update failed.'); }
  await refresh();
}
function card(task: Task) {
  return `<button class="task-card ${task.priority === 'high' ? 'priority' : ''}" data-task="${escape(task.id)}"><span class="card-top"><span class="dept ${departmentClass(task.department)}">${escape(task.department)}</span><span class="task-id">${escape(task.id)}</span></span><span class="task-title">${escape(task.title)}</span><span class="card-description">${escape(task.description)}</span>${task.blocker ? `<span class="blocker">${escape(task.blocker)}</span>` : ''}<span class="card-footer"><span class="owner-dot"></span><span>${escape(task.owner || 'Unassigned')}</span><span class="criterion-count">${task.acceptanceCriteria.filter(c=>c.met).length}/${task.acceptanceCriteria.length} checks</span></span></button>`;
}
function runPanel(history = false) {
  const seen = new Set<string>();
  const runs = (project!.runRecords || []).slice().reverse().filter(r => { if (history) return true; if (seen.has(r.agentId)) return false; seen.add(r.agentId); return true; });
  const header = `<div class="run-heading"><div><h2>${history?'Agent run history':'Last recorded agent states'}</h2><p>Explicit observations with their source. A completed scope does not complete its linked tasks; these records do not indicate current presence.</p></div></div>`;
  return `<${history?'section':'details'} class="run-panel ${history?'run-history':'run-compact'}" aria-label="Recorded agent runs">${history ? header : `<summary>${header}<div class="run-chips">${runs.map(r=>`<span>${escape(r.agentLabel)}<b class="run-state ${escape(r.state)}">${escape(r.state)}</b></span>`).join('')}<small>Expand observations ↓</small></div></summary>`}<div class="run-list">${runs.length ? runs.map(r=>`<article class="run-row"><div class="run-identity"><strong>${escape(r.agentLabel)}</strong><small>${escape(r.agentId)}</small><span class="run-state ${escape(r.state)}">${escape(r.state)}</span></div><div class="run-content"><p>${escape(r.summary)}</p><div class="run-tasks">${r.taskIds.map(id=>`<button data-task="${escape(id)}">${escape(id)}</button>`).join('')}</div><div class="run-time">Observed ${escape(date(r.observedAt))} AEST · recorded ${escape(date(r.createdAt))} AEST · ${escape(r.author)}</div><div class="run-source">${escape(r.source)}</div>${r.evidence.length?`<div class="run-evidence">${r.evidence.map(e=>`<div>${evidenceLink(e)}${e.details?`<small>${escape(e.details)}</small>`:''}</div>`).join('')}</div>`:''}</div></article>`).join('') : '<p class="run-empty">No agent run observations recorded yet.</p>'}</div>${!history?'<button class="secondary" data-view="activity">Full run history ↗</button>':''}</${history?'section':'details'}>`;
}
function board() {
  const p = project!;
  const departments = [...new Set(p.tasks.map(t=>t.department))];
  const filtered = p.tasks.filter(t => (department === 'all' || t.department === department) && (status === 'all' || t.status === status) && `${t.title} ${t.id} ${t.department} ${t.owner}`.toLowerCase().includes(search.toLowerCase()));
  const displayed = (Object.keys(labels) as Status[]).filter(s=>status==='all'||s===status);
  return `<section class="hero"><div><div class="eyebrow"><span class="tiny-star">✦</span> ${escape(p.phase || "Phase not recorded")}</div><h1>A universe in the making.</h1><p class="phase-summary">${escape(p.phaseDescription || "Every task, decision and result accounted for. Implementation and verification have separate recorded gates.")}</p><button class="secondary phase-update" id="update-phase">Update project phase</button><div class="hero-tags"><span>RUST + WASM</span><span>SOLO PWA</span><span>NEO-RETRO COSMIC</span></div></div><div class="orbit-art" aria-hidden="true"><div class="orbit o1"></div><div class="orbit o2"></div><div class="orbit o3"></div><div class="planet"></div><div class="orbit-ship">◆</div><span class="star s1">✦</span><span class="star s2">·</span><span class="star s3">+</span></div></section>
  <section class="stats"><div><span class="stat-label">IN ACTION</span><strong>${p.tasks.filter(t=>t.status==='active').length}<small>assigned tasks</small></strong></div><div><span class="stat-label">AWAITING REVIEW</span><strong>${p.tasks.filter(t=>t.status==='review').length}<small>ready for assessment</small></strong></div><div><span class="stat-label">ACCEPTED WORK</span><strong>${p.tasks.filter(t=>t.status==='complete').length}<small>criteria + evidence</small></strong></div><div><span class="stat-label">RECORDED BUILDS</span><strong>${p.builds.length}<small>${p.builds.length ? 'see validation coverage' : 'first build pending'}</small></strong></div></section>
  ${runPanel()}
  <section class="board-header"><div><h2>Mission board <span>${filtered.length}</span></h2><p>Work states reflect recorded updates, not live agent presence.</p></div><button class="primary" id="new-task">+ New task</button></section>
  <div class="filters"><label class="search-label"><span>⌕</span><input id="search" type="search" placeholder="Find a task, owner or department" value="${escape(search)}" aria-label="Search tasks"></label><select id="department" aria-label="Filter department"><option value="all">All departments</option>${departments.map(d=>`<option ${department===d?'selected':''}>${escape(d)}</option>`).join('')}</select><select id="status" aria-label="Filter status"><option value="all">All states</option>${Object.entries(labels).map(([s,l])=>`<option value="${s}" ${status===s?'selected':''}>${l}</option>`).join('')}</select></div>
  <section class="kanban" style="--lanes:${displayed.length}">${displayed.map(s=>`<div class="lane ${s}"><div class="lane-heading"><span class="status-dot"></span><h3>${labels[s]}</h3><span>${filtered.filter(t=>t.status===s).length}</span></div><div class="lane-cards">${filtered.filter(t=>t.status===s).map(card).join('') || '<div class="lane-empty">No recorded tasks</div>'}</div></div>`).join('')}</section>`;
}
function records() {
  const p = project!;
  if (view === 'activity') return `<div class="page-intro"><span class="eyebrow">TRACEABLE PROGRESS</span><h1>Activity log</h1><p>Recorded changes with their source. No inferred meetings or automated completion.</p></div>${runPanel(true)}<section class="timeline">${p.activity.slice().reverse().map(a=>`<article><div class="timeline-dot"></div><div class="record-meta">${escape(date(a.createdAt))} · revision ${a.revision} · ${escape(a.kind)}</div><h3>${escape(a.title)}</h3><p>${escape(a.author)} · ${escape(a.itemId)}</p><div class="source">${escape(a.source)}</div></article>`).join('')}</section>`;
  const config: Record<string,{title:string;description:string;empty:string;action?:string}> = {
    reviews:{title:'Review room',description:'Real asynchronous exchanges, outcomes and follow-up work.',empty:'No review exchanges have been recorded.',action:'Record review'},
    decisions:{title:'Design decisions',description:'The chosen direction, its source, and proposals still being considered.',empty:'No decisions have been recorded.',action:'Add decision'},
    builds:{title:'Build archive',description:'Playable outputs and recorded validation. Physical devices and emulation stay distinct.',empty:'The first playable build is pending. Build links and check results appear after actual output is recorded.'},
    feedback:{title:'Feedback queue',description:'Your requests for the development team. An active Codex session reads this queue; submitting does not wake an agent.',empty:'No feedback queued. Add a note after playing or reviewing the current direction.',action:'Add feedback'}
  };
  const cfg = config[view];
  const items = p[view as 'reviews'|'decisions'|'builds'|'feedback'];
  return `<div class="page-intro"><div><span class="eyebrow">MISSION CONTROL / ${escape(view.toUpperCase())}</span><h1>${cfg.title}</h1><p>${cfg.description}</p></div>${cfg.action?`<button class="primary" id="new-record">+ ${cfg.action}</button>`:''}</div><section class="record-grid">${items.length ? items.slice().reverse().map(r=>`<article class="record-card"><div class="record-meta">${escape(date(r.createdAt))} · ${escape(r.author)}${r.status?`<span class="record-status ${escape(r.status)}">${escape(r.status)}</span>`:''}</div><h2>${escape(r.title)}</h2>${r.participants?`<div class="participants">${r.participants.map(a=>`<span>${escape(a)}</span>`).join('')}</div>`:''}<p class="record-body">${escape(r.body)}</p>${r.revision?`<div class="build-revision">Source revision: <code>${escape(r.revision)}</code></div>`:''}${r.url && /^https?:\/\//.test(r.url)?`<a class="primary build-link" href="${escape(r.url)}" target="_blank" rel="noopener">Open playable build ↗</a>`:''}${r.checks?`<div class="build-checks">${r.checks.map(c=>`<div><span class="check-result ${escape(c.result)}">${escape(c.result.replace('_',' '))}</span><span>${escape(c.name)}</span>${c.details?`<small>${escape(c.details)}</small>`:''}</div>`).join('')}</div>`:''}${r.knownIssues?`<div class="known-issues"><strong>Known issues</strong><p>${escape(r.knownIssues)}</p></div>`:''}${r.evidence?.length?`<div class="evidence-list">${r.evidence.map(e=>`<div>${evidenceLink(e)}${e.details?`<p>${escape(e.details)}</p>`:''}</div>`).join('')}</div>`:''}${r.response ? `<div class="known-issues"><strong>Recorded response</strong><p>${escape(r.response)}</p></div>` : ""}<footer class="source">${escape(r.source)}</footer></article>`).join('') : `<div class="empty-records"><span>◇</span><h2>${view==='builds'?'First flight ahead':'Room to create'}</h2><p>${cfg.empty}</p></div>`}</section>`;
}
function render() {
  const wikiFocused = document.activeElement?.id === 'wiki-search';
  const p = project;
  app.innerHTML = `<div class="workspace"><aside class="sidebar"><a class="brand" href="#" aria-label="Galaxy Royale mission board"><span class="pixel-logo"><i></i><i></i><i></i><i></i><i></i></span><span>GALAXY<br><b>ROYALE</b><small>MISSION CONTROL</small></span></a><div class="sidebar-section">WORKSPACE</div><nav aria-label="Production views">${nav.map(([key,label,icon])=>`<button data-view="${key}" aria-current="${view===key?'page':'false'}" class="nav-link ${view===key?'selected':''}"><span>${icon}</span>${label}${key==='feedback'&&p?.feedback.length?`<small>${p.feedback.length}</small>`:''}</button>`).join('')}</nav><div class="sidebar-note"><span class="eyebrow">BUILDING WITH INTENT</span><p>A pixel universe.<br>A clear record of the work.</p><span class="private-label">◉ Local & private</span></div><a class="launch-link" href="http://127.0.0.1:4173" target="_blank" rel="noopener">Open game workspace <span>↗</span></a><div class="launch-note">Development preview; availability depends on the game server.</div></aside><div class="main-shell"><header class="topbar"><div><span class="breadcrumb">Galaxy Royale</span><span class="slash">/</span><span>${nav.find(n=>n[0]===view)?.[1]}</span></div><div class="service-state ${connected?'online':'offline'}"><i></i>${connected?'Local API connected':'Local API offline'}</div></header><main>${view==='wiki' ? wiki.render() : p ? (view==='board'?board():records()) : `<div class="empty-records"><span>◇</span><h1>${connected?'Loading mission control':'Production service unavailable'}</h1><p>Start the local development services to load the recorded project. No demonstration data is substituted.</p><button class="primary" id="retry">Retry connection</button></div>`}</main><footer class="workspace-footer"><span>${p?`Snapshot r${p.revision} · last recorded ${escape(date(p.updatedAt))} AEST`:'No project snapshot loaded'}</span><span>Recorded work · no inferred agent presence</span></footer></div></div>`;
  app.querySelectorAll<HTMLButtonElement>('[data-view]').forEach(b=>b.onclick=()=>{view=b.dataset.view!;history.replaceState(null,'',view==='wiki'?'#wiki':`#${view}`);render();});
  app.querySelector('.brand')?.addEventListener('click', e=>{e.preventDefault();view='board';history.replaceState(null,'','#board');render();});
  app.querySelectorAll<HTMLButtonElement>('[data-task]').forEach(b=>b.onclick=()=>openTask(p!.tasks.find(t=>t.id===b.dataset.task)!));
  if(view==='wiki') { wiki.bind(app,render); if(wikiFocused) app.querySelector<HTMLInputElement>('#wiki-search')?.focus(); if(!wiki.catalog && !wiki.loading && !wiki.error) void wiki.load(render); }
  app.querySelector('#update-phase')?.addEventListener('click',openProjectPhase);
  app.querySelector('#new-task')?.addEventListener('click',()=>openTask());
  app.querySelector('#new-record')?.addEventListener('click',openRecord);
  app.querySelector('#retry')?.addEventListener('click',refresh);
  app.querySelector<HTMLSelectElement>('#department')?.addEventListener('change',e=>{department=(e.target as HTMLSelectElement).value;render();});
  app.querySelector<HTMLSelectElement>('#status')?.addEventListener('change',e=>{status=(e.target as HTMLSelectElement).value;render();});
  app.querySelector<HTMLInputElement>('#search')?.addEventListener('input',e=>{const input=e.target as HTMLInputElement;const position=input.selectionStart;search=input.value;render();const updated=app.querySelector<HTMLInputElement>('#search')!;updated.focus();if(updated.type!=='search')updated.setSelectionRange(position,position);});
}
function dialog(content: string) {
  const node = document.createElement('dialog'); node.className='editor-dialog'; node.innerHTML=content; document.body.append(node); node.showModal();
  node.querySelector('[data-close]')?.addEventListener('click',()=>node.close());
  node.addEventListener('close',()=>node.remove());
  node.addEventListener('click',e=>{if(e.target===node){const r=node.getBoundingClientRect();if(e.clientX<r.left||e.clientX>r.right||e.clientY<r.top||e.clientY>r.bottom)node.close();}});
  return node;
}
function openProjectPhase() {
  const p=project!;
  const node=dialog(`<form><header class="dialog-heading"><div><span class="eyebrow">PROJECT / SNAPSHOT ${p.revision}</span><h2>Update project phase</h2></div><button type="button" data-close class="close-button" aria-label="Close phase editor">×</button></header><div class="form-body"><label>Phase<input name="phase" required maxlength="200" value="${escape(p.phase||'')}"></label><label>Scope and verification boundary<textarea name="phaseDescription" maxlength="2000" rows="4">${escape(p.phaseDescription||'')}</textarea></label><label>Author<input name="author" required maxlength="200" placeholder="Person making this recorded update"></label><label>Source<input name="source" required maxlength="1000" placeholder="Actual review, instruction or evidence behind the phase"></label><p>Changes only project metadata. Tasks, evidence and append-only history stay intact. A stale snapshot is rejected.</p><p class="form-error" role="alert"></p></div><footer class="dialog-footer"><button type="button" data-close class="secondary">Cancel</button><button class="primary">Save phase</button></footer></form>`);
  node.querySelectorAll<HTMLButtonElement>('[data-close]').forEach(b=>b.onclick=()=>node.close());
  node.querySelector('form')!.onsubmit=async event=>{
    event.preventDefault(); const values=new FormData(event.currentTarget as HTMLFormElement);
    const button=node.querySelector<HTMLButtonElement>('.primary')!;button.disabled=true;
    try { await mutate('/api/project','PATCH',{expectedRevision:p.revision,author:values.get('author'),source:values.get('source'),patch:{phase:values.get('phase'),phaseDescription:values.get('phaseDescription')}});node.close();toast('Project phase recorded.'); }
    catch(error) { node.querySelector('.form-error')!.textContent=error instanceof Error?error.message:'Could not update the phase.';button.disabled=false; }
  };
}
function openTask(task?: Task) {
  const node=dialog(`<form><header class="dialog-heading"><div><span class="eyebrow">${task?`${escape(task.id)} / REVISION ${task.revision}`:'NEW MISSION'}</span><h2>${task?'Task details':'Create task'}</h2></div><button type="button" data-close class="close-button" aria-label="Close task">×</button></header><div class="form-body"><label>Title<input name="title" required maxlength="300" value="${escape(task?.title||'')}"></label><label>Description<textarea name="description" rows="3">${escape(task?.description||'')}</textarea></label><div class="form-row"><label>Department<select name="department">${['Production','Engine','Gameplay','Platform','Audio','Visual','QA'].map(d=>`<option ${task?.department===d?'selected':''}>${d}</option>`).join('')}</select></label><label>Priority<select name="priority">${['normal','high','low'].map(d=>`<option ${task?.priority===d?'selected':''}>${d}</option>`).join('')}</select></label></div><div class="form-row"><label>Owner<input name="owner" value="${escape(task?.owner||'')}" placeholder="Person or assigned specialist"></label><label>Work state<select name="status">${Object.entries(labels).map(([s,l])=>`<option value="${s}" ${task?.status===s?'selected':''}>${l}</option>`).join('')}</select></label></div><label>Dependencies<select name="dependencies" multiple size="3">${project!.tasks.filter(t=>t.id!==task?.id).map(t=>`<option value="${escape(t.id)}" ${task?.dependencies.includes(t.id)?'selected':''}>${escape(t.title)}</option>`).join('')}</select><small>Ctrl / Cmd click to select multiple tasks. Completed work requires completed dependencies.</small></label><label>Recorded blocker<textarea name="blocker" rows="2" placeholder="Required when blocked">${escape(task?.blocker||'')}</textarea></label><div class="criteria-editor"><h3>Acceptance criteria</h3>${task?task.acceptanceCriteria.map((c,i)=>`<label class="criterion"><input type="checkbox" name="criterion-${i}" ${c.met?'checked':''}><span>${escape(c.text)}</span></label>`).join(''):''}<label>${task?'Add criteria (one per line)':'Criteria (one per line)'}<textarea name="newCriteria" rows="3" placeholder="Describe the behavior to validate"></textarea></label></div><div class="existing-evidence"><h3>Recorded evidence</h3>${task?.evidence.length?task.evidence.map(e=>`<div>${evidenceLink(e)}${e.details?`<p>${escape(e.details)}</p>`:''}</div>`).join(''):'<p>No evidence recorded yet.</p>'}</div><label>Add evidence note<textarea name="evidenceNote" rows="3" placeholder="Actual result, review or test outcome; include useful limits"></textarea></label><label>Optional artifact path<input name="evidencePath" placeholder="work/validation.log"><small>Must refer to an existing file in an approved artifact directory.</small></label><p class="form-hint">Completing a task requires every criterion to be met and evidence recorded. An assignment does not indicate a live agent.</p><p class="form-error" role="alert"></p></div><footer class="dialog-footer"><span>Recorded as a user dashboard action</span><button class="primary" type="submit">${task?'Save changes':'Create task'}</button></footer></form>`);
  const form=node.querySelector<HTMLFormElement>('form')!;
  form.onsubmit=async e=>{
    e.preventDefault(); const button=form.querySelector<HTMLButtonElement>('[type=submit]')!;button.disabled=true;
    const data=new FormData(form); const value=(name:string)=>String(data.get(name)||'').trim();
    const criteria=(task?.acceptanceCriteria||[]).map((c,i)=>({text:c.text,met:data.has(`criterion-${i}`)})).concat(value('newCriteria').split('\n').map(s=>s.trim()).filter(Boolean).map(text=>({text,met:false})));
    const evidence=[...(task?.evidence||[])];if(value('evidenceNote')||value('evidencePath'))evidence.push({label:value('evidencePath')?'Recorded artifact':'User validation note',kind:value('evidencePath')?'artifact':'note',...(value('evidencePath')?{path:value('evidencePath')}:{ }),...(value('evidenceNote')?{details:value('evidenceNote')}:{ })});
    const patch={title:value('title'),description:value('description'),department:value('department'),owner:value('owner'),status:value('status'),priority:value('priority'),dependencies:data.getAll('dependencies').map(String),blocker:value('blocker'),acceptanceCriteria:criteria,evidence};
    try { await mutate(task?`/api/tasks/${encodeURIComponent(task.id)}`:'/api/tasks',task?'PATCH':'POST',{author:'Itchson',source:'Explicit user dashboard action',...(task?{expectedRevision:task.revision,patch}:{task:patch})});node.close();toast(task?'Task update recorded.':'Task created.'); }
    catch(err){form.querySelector('.form-error')!.textContent=err instanceof Error?err.message:'Unable to save.';}
    finally{button.disabled=false;}
  };
}
function openRecord() {
  const kind=view==='reviews'?'review':view==='decisions'?'decision':'feedback';
  const node=dialog(`<form><header class="dialog-heading"><div><span class="eyebrow">RECORD ${kind.toUpperCase()}</span><h2>${kind==='review'?'Record an actual exchange':kind==='decision'?'Add a design decision':'Share feedback'}</h2></div><button type="button" data-close class="close-button" aria-label="Close">×</button></header><div class="form-body"><label>Title<input name="title" required maxlength="300"></label>${kind==='review'?'<label>Actual participants<input name="participants" required placeholder="Names or roles, separated by commas"></label>':''}${kind==='decision'?'<label>State<select name="status"><option value="proposed">Proposed</option><option value="accepted">Accepted</option><option value="rejected">Rejected</option><option value="superseded">Superseded</option></select></label>':''}<label>${kind==='review'?'Contributions, outcomes and follow-up work':'Note'}<textarea name="body" rows="8" required></textarea></label><label>Source / context<input name="source" required value="Explicit user dashboard action" placeholder="Link or reference to the actual discussion"></label>${kind==='feedback'?'<p class="form-hint">This creates a queued request. An active Codex session must read it; the dashboard does not wake agents.</p>':''}<p class="form-error" role="alert"></p></div><footer class="dialog-footer"><span>Saved with author and timestamp</span><button class="primary" type="submit">Record ${kind}</button></footer></form>`);
  const form=node.querySelector<HTMLFormElement>('form')!;form.onsubmit=async e=>{e.preventDefault();const data=new FormData(form);const v=(k:string)=>String(data.get(k)||'').trim();const button=form.querySelector<HTMLButtonElement>('[type=submit]')!;button.disabled=true;try{await mutate(`/api/${kind==='feedback'?'feedback':kind+'s'}`,'POST',{author:'Itchson',source:v('source'),record:{title:v('title'),body:v('body'),...(kind==='review'?{participants:v('participants').split(',').map(s=>s.trim()).filter(Boolean),taskIds:[]}:{}),...(kind==='decision'?{status:v('status')}:{})}});node.close();toast(`${kind[0].toUpperCase()+kind.slice(1)} recorded.`);}catch(err){form.querySelector('.form-error')!.textContent=err instanceof Error?err.message:'Unable to record.';}finally{button.disabled=false;}};
}
render();void refresh();
const events=new EventSource('/api/events');
events.addEventListener('update',e=>{const data=JSON.parse((e as MessageEvent).data) as {revision:number};latestSeenRevision=Math.max(latestSeenRevision,data.revision);connected=true;if(data.revision!==project?.revision)void refresh();else render();});
events.onerror=()=>{connected=false;render();};
window.addEventListener('focus',()=>void refresh());

window.addEventListener('hashchange',()=>{const route=/^#wiki(?:\/([a-z0-9-]+))?$/.exec(location.hash);if(route){view='wiki';wiki.selected=route[1]||'';render();}});
