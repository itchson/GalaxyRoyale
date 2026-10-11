import {createHash} from 'node:crypto';
import {readdir,readFile,writeFile,mkdir,lstat} from 'node:fs/promises';
import {resolve,relative} from 'node:path';
import {execFileSync} from 'node:child_process';
export async function treeHash(root,paths,exclude=()=>false){
 const hash=createHash('sha256');
 async function visit(path){
  const name=relative(root,path).replaceAll('\\','/');if(exclude(name))return;
  const info=await lstat(path);if(info.isSymbolicLink())throw Error('Linked build input: '+name);
  if(info.isDirectory()){for(const entry of(await readdir(path)).sort())await visit(resolve(path,entry));}
  else if(info.isFile())hash.update(name).update('\0').update(await readFile(path));
 }
 for(const path of [...paths].sort())await visit(resolve(root,path));return hash.digest('hex');
}
const runtimePaths=['Cargo.toml','Cargo.lock','rust-toolchain.toml','package.json','pnpm-lock.yaml','tsconfig.json','crates','assets','apps/game','scripts/build.mjs','scripts/build-wasm.mjs','scripts/pwa.mjs','scripts/wasm-artifacts.mjs','scripts/preview-build.mjs'];
const excluded=name=>/^apps\/game\/(?:dist|node_modules|public\/wasm)(?:\/|$)/.test(name);
const git=(root,...args)=>execFileSync('git',args,{cwd:root,encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
export async function buildIdentity(root){return {head:git(root,'rev-parse','HEAD'),branch:git(root,'branch','--show-current'),sourceHash:await treeHash(root,runtimePaths,excluded)};}
export async function writePreviewBuild(root,expected){
 const identity=await buildIdentity(root);
 if(expected&&JSON.stringify(identity)!==JSON.stringify(expected))throw Error('Source or revision changed during the build. Freeze inputs and rebuild; no current preview stamp was written.');
 const stamp={schemaVersion:1,...identity,packageHash:await treeHash(root,['dist']),builtAt:new Date().toISOString()};
 await mkdir(resolve(root,'work'),{recursive:true});await writeFile(resolve(root,'work/preview-build.json'),JSON.stringify(stamp,null,2));return stamp;
}
export async function verifyPreviewBuild(root){
 const branch=git(root,'branch','--show-current');if(branch!=='codex/testing')throw Error('Game preview requires codex/testing; current branch is '+branch);
 let stamp;try{stamp=JSON.parse(await readFile(resolve(root,'work/preview-build.json'),'utf8'));}catch{throw Error('No testing build record. Run pnpm build before preview.');}
 if(stamp.schemaVersion!==1||stamp.head!==git(root,'rev-parse','HEAD')||stamp.branch!==branch||stamp.sourceHash!==await treeHash(root,runtimePaths,excluded)||stamp.packageHash!==await treeHash(root,['dist']))throw Error('Testing build is stale or changed. Run pnpm build before preview.');
 return stamp;
}
