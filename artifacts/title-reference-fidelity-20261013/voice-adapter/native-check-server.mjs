import fs from 'node:fs/promises';
import http from 'node:http';
import crypto from 'node:crypto';
import ts from 'typescript';
const root=import.meta.dirname;
const source=await fs.readFile(root+'/periodic-patch.ts');
const js=ts.transpileModule(source.toString('utf8'),{compilerOptions:{module:ts.ModuleKind.ES2022,target:ts.ScriptTarget.ES2022}}).outputText;
const html=await fs.readFile(root+'/native-check.html');
const pins={scope:'Own source-adapter native check only; no game instance',sourceSha256:crypto.createHash('sha256').update(source).digest('hex'),helperSha256:crypto.createHash('sha256').update(html).digest('hex'),typescript:ts.version};
await fs.writeFile(root+'/native-input-pins.json',JSON.stringify(pins,null,2)+'\n');
const server=http.createServer(async(req,res)=>{
  if(req.method==='GET'&&(req.url==='/'||req.url==='/periodic-patch.js')) {
    res.writeHead(200,{'content-type':req.url==='/'?'text/html; charset=utf-8':'text/javascript; charset=utf-8','cache-control':'no-store'});res.end(req.url==='/'?html:js);return;
  }
  if(req.method==='POST'&&req.url==='/report'&&req.headers.origin==='http://127.0.0.1:4198'){
    const parts=[];let length=0;
    for await(const part of req){length+=part.length;if(length>12000){res.writeHead(413);res.end();return;}parts.push(part);}
    try {
      const report=JSON.parse(Buffer.concat(parts).toString('utf8'));
      if(!Array.isArray(report.cases)||report.cases.length!==6)throw new Error('Unsupported report');
      await fs.writeFile(root+'/native-report.json',JSON.stringify({...report,inputs:pins},null,2)+'\n');res.writeHead(201);res.end('Saved');
      console.log(JSON.stringify({actualReport:root+'/native-report.json',pass:report.pass}));
    }catch {res.writeHead(400);res.end();}return;
  }
  res.writeHead(404);res.end();
});
server.listen(4198,'127.0.0.1',()=>console.log('Temporary offline-audio check http://127.0.0.1:4198/'));
const timeout=setTimeout(()=>server.close(),180000);
const stop=()=>{clearTimeout(timeout);server.close();};
process.on('SIGINT',stop);process.on('SIGTERM',stop);
