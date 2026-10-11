import {spawn,execFileSync} from 'node:child_process';
import {readFile,readdir,realpath,writeFile,unlink} from 'node:fs/promises';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {verifyPreviewBuild} from './preview-build.mjs';
export function isProjectGameProcess(command,root){
 const normalize=s=>s.replaceAll('\\','/').toLowerCase();
 const text=normalize(command),project=normalize(root).replace(/\/$/,'');
 return (text.includes(project+'/node_modules/vite/bin/vite.js')||(isRelativeGameProcess(text)&&text.includes(project+'/')))&&/--config\s+"?apps\/game\/vite\.config\.ts(?:"|\s|$)/.test(text);
}
export function isRelativeGameProcess(command){
 const text=command.replaceAll('\\','/').toLowerCase();
 return /(?:^|\s|\")node_modules\/vite\/bin\/vite\.js\"?\s/.test(text)&&/--config\s+"?apps\/game\/vite\.config\.ts(?:"|\s|$)/.test(text);
}
export async function gameProcesses(root){
 if(process.platform==='win32'){
  const raw=execFileSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',"Get-CimInstance Win32_Process -Filter \"Name='node.exe'\" | Select-Object ProcessId,CommandLine | ConvertTo-Json -Compress"],{encoding:'utf8',windowsHide:true});
  const rows=raw.trim()?JSON.parse(raw):[];
  return [].concat(rows).filter(p=>p.ProcessId!==process.pid&&(isProjectGameProcess(p.CommandLine||'',root)||isRelativeGameProcess(p.CommandLine||''))).map(p=>({pid:p.ProcessId,command:p.CommandLine,ownership:isProjectGameProcess(p.CommandLine||'',root)?'owned':'unverified-relative-path'}));
 }
 const rows=[];for(const id of(await readdir('/proc')).filter(s=>/^\d+$/.test(s))){try{
  const command=(await readFile('/proc/'+id+'/cmdline','utf8')).replaceAll('\0',' ');
  if(Number(id)!==process.pid&&(isProjectGameProcess(command,root)||isRelativeGameProcess(command))&&await realpath('/proc/'+id+'/cwd')===await realpath(root))rows.push({pid:Number(id),command,ownership:'owned'});
 }catch{/* Process exited during discovery. */}}return rows;
}
export async function startPreview(root){
 const stamp=await verifyPreviewBuild(root),existing=await gameProcesses(root);
 if(existing.length)throw Error('Game Vite process already running (PID '+existing.map(p=>p.pid+' '+p.ownership).join(', ')+'). Identify and stop the owned preview before starting another; an unverified relative-path process is never killed automatically.');
 const child=spawn(process.execPath,[resolve(root,'node_modules/vite/bin/vite.js'),'preview','--config','apps/game/vite.config.ts'],{cwd:root,stdio:'inherit',windowsHide:true});
 const record=resolve(root,'work/game-preview.json');
 let stopping=false;const stop=()=>{if(!stopping){stopping=true;child.kill();}};
 process.on('SIGINT',stop);process.on('SIGTERM',stop);
 child.once('error',error=>{console.error(error.message);process.exitCode=1;});
 child.once('exit',async(code,signal)=>{process.removeListener('SIGINT',stop);process.removeListener('SIGTERM',stop);try{const current=JSON.parse(await readFile(record,'utf8'));if(current.pid===child.pid)await unlink(record);}catch{}process.exitCode=code??(stopping?0:1);});
 try{
  await writeFile(record,JSON.stringify({...stamp,pid:child.pid,port:4173,startedAt:new Date().toISOString()},null,2));
  if(child.exitCode!==null||child.signalCode!==null){const current=JSON.parse(await readFile(record,'utf8'));if(current.pid===child.pid)await unlink(record);}
 }catch(error){stop();throw Error('Could not record the owned preview; spawned process stopped. '+error.message);}
 return child;
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 try{await startPreview(resolve(import.meta.dirname,'..'));}catch(error){console.error(error.message);process.exitCode=1;}
}
