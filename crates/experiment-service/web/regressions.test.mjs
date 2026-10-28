// Production-owned regressions; the externally approved harness is unchanged.
import test from 'node:test';
import assert from 'node:assert/strict';
import {Worker} from 'node:worker_threads';
import {parseSnapshot,Replay,positionEllipse} from './data.mjs';

test('issue51 anisotropic covariance retains its positive minor axis',()=>{
 const covariance=Array(16).fill(0);covariance[0]=1e16;covariance[5]=1;
 const diagonal=positionEllipse({covariance});assert.ok(Math.abs(diagonal.minor-2.447746830680816)<1e-12);
 covariance[0]=5e15;covariance[5]=5e15;covariance[1]=5e15-1;covariance[4]=5e15-1;
 const rotated=positionEllipse({covariance});assert.ok(Math.abs(rotated.minor-2.447746830680816)<1e-12);assert.ok(Math.abs(rotated.angle-Math.PI/4)<1e-12);
 covariance[0]=1e308;covariance[5]=1;covariance[1]=0;covariance[4]=0;
 const large=positionEllipse({covariance});assert.ok(Number.isFinite(large.major));assert.ok(Math.abs(large.minor-2.447746830680816)<1e-12);
 covariance[0]=Number.MAX_VALUE;covariance[5]=Number.MAX_VALUE;covariance[1]=1;covariance[4]=1;
 const maximum=positionEllipse({covariance});assert.ok(maximum);assert.ok(Object.values(maximum).every(Number.isFinite));assert.ok(Math.abs(maximum.major/(Math.sqrt(Number.MAX_VALUE)*2.447746830680816)-1)<1e-14);
 covariance[0]=1e-300;covariance[5]=1e-300;covariance[1]=5e-301;covariance[4]=5e-301;
 const tiny=positionEllipse({covariance});assert.ok(Math.abs(tiny.major/(Math.sqrt(1.5e-300)*2.447746830680816)-1)<1e-14);assert.ok(Math.abs(tiny.minor/(Math.sqrt(.5e-300)*2.447746830680816)-1)<1e-14);
});

test('issue49 JSON objects cannot impersonate numeric tokens',()=>{
 const config={seed:42,duration_seconds:1,scenario:'straight',position_sigma:0,velocity_sigma:0,acceleration_density:.1,outage:{start:0,end:1}};
 const snapshot={id:'run-1',state:'running',sequence:1,config,samples:null,error:null,artifacts:[]};
 const forged=()=>({integerToken:'1'});
 for(const key of ['seed','duration_seconds','position_sigma','velocity_sigma','acceleration_density'])assert.throws(()=>parseSnapshot(JSON.stringify({...snapshot,config:{...config,[key]:forged()}})),/invalid/i,key);
 for(const key of ['sequence','samples'])assert.throws(()=>parseSnapshot(JSON.stringify({...snapshot,[key]:forged()})),/invalid/i,key);
 for(const key of ['start','end'])assert.throws(()=>parseSnapshot(JSON.stringify({...snapshot,config:{...config,outage:{...config.outage,[key]:forged()}}})),/invalid/i,key);
});

test('issue50 fractional replay time survives speed changes and pause/resume',()=>{
 const recording={rowCount:101,row:index=>({tick:index})};
 const replay=new Replay(recording);replay.play(0);assert.equal(replay.frame(5).tick,0);replay.setSpeed(4,5);assert.equal(replay.frame(7).tick,1);
 const paused=new Replay(recording);paused.play(0);paused.pause(5);paused.play(100);assert.equal(paused.frame(105).tick,1);
});

async function assertControlledWorker(delayLoadedMs=0){
 const moduleUrl=new URL(process.env.FUSION_WORKER_MODULE??'./parser-worker.mjs',import.meta.url).href;
 const adapter=`const {parentPort}=require('node:worker_threads');globalThis.onmessage=null;globalThis.postMessage=(message,transfer)=>{if(message.type==='loaded'){const until=performance.now()+${delayLoadedMs};while(performance.now()<until){}}parentPort.postMessage(message,transfer);};import(${JSON.stringify(moduleUrl)}).then(()=>{parentPort.on('message',data=>globalThis.onmessage({data}));parentPort.postMessage({type:'adapterReady'});});`;
 const worker=new Worker(adapter,{eval:true});
 const queued=[],waiters=[];
 worker.on('message',message=>{const at=waiters.findIndex(waiter=>waiter.type===message.type);if(at>=0){const waiter=waiters.splice(at,1)[0];clearTimeout(waiter.timer);waiter.resolve(message);}else queued.push(message);});
 function next(type){const at=queued.findIndex(message=>message.type===type);if(at>=0)return Promise.resolve(queued.splice(at,1)[0]);return new Promise((resolve,reject)=>{const waiter={type,resolve,timer:setTimeout(()=>{waiters.splice(waiters.indexOf(waiter),1);reject(new Error(`No ${type} message: final paused seek was lost`));},2000)};waiters.push(waiter);});}
 try{
  await next('adapterReady');worker.postMessage({type:'resetInput',generation:1});await next('resetAck');
  const header='tick,time_s,true_x,true_y,true_vx,true_vy,estimated_x,estimated_y,estimated_vx,estimated_vy,position_x,position_y,velocity_x,velocity_y,hold_x,hold_y,integrated_x,integrated_y,position_outage,p00,p01,p02,p03,p10,p11,p12,p13,p20,p21,p22,p23,p30,p31,p32,p33';
  const first='0,0,0,0,1,1,0,0,1,1,0,0,1,1,0,0,0,0,false,1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1';
  const second='1,0.01,0.01,0.01,1,1,0.01,0.01,1,1,,,,,0,0,0.01,0.01,true,1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1';
  worker.postMessage({type:'load',generation:1,file:new Blob([header+'\n'+first+'\n'+second])});
  await next('loaded');const initial=await next('frame');assert.equal(initial.row.tick,0);assert.equal(initial.playing,false);
  // Intentionally withhold the first frame ACK until the later seek is queued.
  worker.postMessage({type:'seek',generation:1,index:1});worker.postMessage({type:'frameAck',generation:1});
  const final=await next('frame');assert.equal(final.row.tick,1);assert.equal(final.playing,false);assert.equal(final.generation,1);
  const completedMetrics=(await next('metrics')).metrics;assert.equal(completedMetrics.input_readers,0);assert.equal(completedMetrics.recording_owners,1);assert.equal(completedMetrics.parser_owners,0);assert.ok(completedMetrics.numeric_buffers_bytes>0);assert.ok(completedMetrics.numeric_buffers_bytes<=100800280);
  if(delayLoadedMs)assert.ok(completedMetrics.max_active_work_before_yield_ms>=delayLoadedMs*.8,'continuous timing must observe synchronous notification work outside parser snippets');
  worker.postMessage({type:'resetInput',generation:2});const reset=await next('resetAck');assert.equal(reset.metrics.input_readers,0);assert.equal(reset.metrics.recording_owners,0);assert.equal(reset.metrics.numeric_buffers_bytes,0);
 }finally{await worker.terminate();for(const waiter of waiters)clearTimeout(waiter.timer);}
}
test('issue48 final paused seek is delivered after an outstanding frame acknowledgment',()=>assertControlledWorker());
test('issue57 continuous worker meter detects a deliberately slow final notification',()=>assertControlledWorker(12));
