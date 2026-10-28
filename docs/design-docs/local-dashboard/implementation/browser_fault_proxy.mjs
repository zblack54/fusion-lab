// Finite loopback verification fixture. Never part of the production service.
// Origin translation is confined to this fixture; production origin checks stay
// unchanged. Use only with the controlled local worker on8089, never real data.
import http from 'node:http';
import {writeFile} from 'node:fs/promises';
const mode=process.argv[2],output=process.argv[3];
const failRecovery=process.argv.includes('--fail-refresh');
if(!['drop-post','drop-sse'].includes(mode)||!output)throw new Error('Usage: node browser_fault_proxy.mjs drop-post|drop-sse OUTPUT');
const started=Date.now(),sockets=new Set();
const report={mode,fail_recovery:failRecovery,user_agent:null,started_utc:new Date().toISOString(),backend:'127.0.0.1:8089',fixture:'127.0.0.1:8090',counts:{},accepted_posts:[],sse_open_ms:[],origin_translations:0,status:'running'};
let stopping=false;
async function stop(){if(stopping)return;stopping=true;clearTimeout(deadline);server.close();for(const socket of sockets)socket.destroy();report.status='stopped';report.elapsed_ms=Date.now()-started;await writeFile(output,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));process.stdin.pause();}
const server=http.createServer((request,response)=>{
 const path=request.url.split('?')[0];
 if(report.user_agent===null&&request.headers['user-agent'])report.user_agent=String(request.headers['user-agent']).slice(0,512);
 if(path==='/_probe/summary'){response.writeHead(200,{'Content-Type':'application/json'});response.end(JSON.stringify(report));return;}
 const category=path==='/v1/runs'?'runs':path.endsWith('/events')?'events':path.startsWith('/v1/runs/')?'run-status':'asset';
 const key=`${request.method}:${category}`;report.counts[key]=(report.counts[key]??0)+1;
 if(failRecovery&&report.accepted_posts.length&&request.method==='GET'&&path==='/v1/runs'){response.writeHead(503,{'Content-Type':'application/json'});response.end('{"error":{"code":"injected_recovery_failure"}}');return;}
 const headers={...request.headers,host:'127.0.0.1:8089'};
 if(headers.origin){headers.origin='http://127.0.0.1:8089';report.origin_translations++;}
 const upstream=http.request({host:'127.0.0.1',port:8089,path:request.url,method:request.method,headers},reply=>{
  if(mode==='drop-post'&&request.method==='POST'&&path==='/v1/runs'&&reply.statusCode===202){
   let text='';reply.on('data',chunk=>{if(text.length+chunk.length<=8192)text+=chunk.toString('utf8');else upstream.destroy();});
   reply.on('end',()=>{const snapshot=JSON.parse(text);report.accepted_posts.push({id:snapshot.id,state:snapshot.state,after_ms:Date.now()-started});response.destroy();});return;
  }
  response.writeHead(reply.statusCode,reply.headers);
  if(mode==='drop-sse'&&path.endsWith('/events')&&reply.statusCode===200){
   if(report.sse_open_ms.length<16)report.sse_open_ms.push(Date.now()-started);
   reply.once('data',chunk=>{response.write(chunk);setTimeout(()=>{response.destroy();upstream.destroy();},30);});return;
  }
  reply.pipe(response);
 });
 upstream.on('error',()=>{if(!response.headersSent)response.writeHead(502);response.end('local verification upstream unavailable');});
 response.on('close',()=>upstream.destroy());request.pipe(upstream);
});
server.on('connection',socket=>{if(sockets.size>=96){socket.destroy();return;}sockets.add(socket);socket.on('close',()=>sockets.delete(socket));});
server.listen(8090,'127.0.0.1',()=>console.log(`Bounded ${mode} fixture listening127.0.0.1:8090, backend8089; hard deadline120seconds; stdin stop ends early`));
const deadline=setTimeout(stop,120000);
process.stdin.setEncoding('utf8');process.stdin.on('data',data=>{if(data.trim()==='stop')stop();});process.on('SIGINT',stop);
