// Finite loopback-only helpers for native UI negative demonstrations.
import http from 'node:http';
import net from 'node:net';
import {writeFile} from 'node:fs/promises';
const mode=process.argv[2],output=process.argv[3];
const request=(method,path,body)=>new Promise((resolve,reject)=>{
 const text=body?JSON.stringify(body):'';
 const call=http.request({host:'127.0.0.1',port:8089,path,method,headers:{'Content-Type':'application/json','Content-Length':Buffer.byteLength(text)}},response=>{let value='';response.on('data',chunk=>value+=chunk);response.on('end',()=>resolve({status:response.statusCode,value:JSON.parse(value)}));});
 call.setTimeout(3000,()=>call.destroy(new Error('bounded local probe timeout')));call.on('error',reject);call.end(text);
});
if(mode==='evict'){
 const childListenerClosed=await new Promise(resolve=>{const socket=net.connect(51218,'127.0.0.1');socket.on('connect',()=>{socket.destroy();resolve(false);});socket.on('error',()=>resolve(true));socket.setTimeout(1000,()=>{socket.destroy();resolve(false);});});
 if(!childListenerClosed)throw new Error('previous controlled worker listener still open');
 const created=await request('POST','/v1/runs',{seed:9002,duration_seconds:1,scenario:'straight',position_sigma:0,velocity_sigma:0,acceleration_density:.1,outage:null});
 if(created.status!==202)throw new Error(`unexpectedadmission${created.status}`);
 let state;const deadline=Date.now()+5000;
 do{state=await request('GET',`/v1/runs/${created.value.id}`);if(state.value.state!=='running')break;await new Promise(resolve=>setTimeout(resolve,10));}while(Date.now()<deadline);
 if(state.value.state!=='failed')throw new Error('controlled noisy worker did not fail');
 const list=await request('GET','/v1/runs');
 const report={method:'Real service with existing controlled worker; this is lifecycle/error evidence, not numerical artifact evidence',child_listener_closed_after_UI_cancel:childListenerClosed,admitted:created.value.id,terminal:state.value,list:list.value};await writeFile(output,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
}else if(mode==='overload'){
 let sockets=[];const health=[];let stopping=false;
 const open=async()=>{for(const socket of sockets)socket.destroy();sockets=[];for(let index=0;index<32;index++)await new Promise((resolve,reject)=>{const socket=net.connect(8089,'127.0.0.1',()=>{socket.write('POST /v1/runs HTTP/1.1\r\nHost:127.0.0.1:8089\r\nContent-Type:application/json\r\nContent-Length:4096\r\n\r\n{');resolve();});socket.on('error',reject);sockets.push(socket);});};
 await open();await new Promise(resolve=>setTimeout(resolve,100));health.push((await request('GET','/v1/health')).status);if(health[0]!==503)throw new Error('did not establish32inflightrequests');
 console.log('Established32boundedpartialrequests; expected UIRefreshHTTP503; harddeadline20seconds');
 const refresh=setInterval(()=>open().catch(error=>{console.error(error.message);stop();}),2500);
 async function stop(){if(stopping)return;stopping=true;clearInterval(refresh);clearTimeout(timer);for(const socket of sockets)socket.destroy();const report={concurrent_partial_requests:32,initial_health_status:health[0],fixture_deadline_seconds:20,status:'closed'};await writeFile(output,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));process.stdin.pause();}
 const timer=setTimeout(stop,20000);process.stdin.setEncoding('utf8');process.stdin.on('data',text=>{if(text.trim()==='stop')stop();});
}else throw new Error('Usage: browser_control_support.mjs evict|overload OUTPUT');
