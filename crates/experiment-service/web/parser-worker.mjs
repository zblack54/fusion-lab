import {CsvRecordingParser,Replay,selectPlotRows,rowErrors} from './data.mjs';

let generation=0,controller=null,reader=null,job=null,recording=null,replay=null;
let framePending=false,dirtyFrame=false,lastFrame=0;
let observedParser=null,lastControlId=null;
let statistics={};
function metrics(){return {...statistics,numeric_buffers_bytes:recording?.allocatedNumericBytes??observedParser?.allocatedNumericBytes??0,input_readers:Number(reader!==null),recording_owners:Number(recording!==null),parser_owners:Number(observedParser!==null),frame_in_flight:Number(framePending),coalesced_frame_pending:Number(dirtyFrame)};}
const yieldTask=()=>new Promise(resolve=>setTimeout(resolve,0));
const send=(type,payload={},transfer=[])=>postMessage({type,generation,...payload},transfer);
function clear(){recording=null;replay=null;framePending=false;dirtyFrame=false;lastControlId=null;}
async function stop(){controller?.abort();try{await reader?.cancel();}catch{/* The original load reports its own failure. */}await job;clear();}

async function load(message){
 let segmentStarted=performance.now(),suspendedMs=0,yieldRows=0;
 const mine=generation;controller=new AbortController();
 let parser=new CsvRecordingParser();
 observedParser=parser;
 statistics={yield_count:0,max_slice_ms:0,max_active_work_before_yield_ms:0,max_rows_between_yields:0,parsed_rows:0,yield_trigger_ms:4,approved_work_budget_ms:8,excluded_await_idle_ms:0,control_callback_max_ms:0,timing_method:'Continuous load segment wall time minus suspended fetch/read/cancel awaits; only a real timer yield resets the segment. Includes loop, notifications, Replay, finish, plot and cleanup bookkeeping.',phases:{csv:{max_slice_ms:0,total_ms:0},finish:{max_slice_ms:0,total_ms:0},plot:{max_slice_ms:0,total_ms:0}}};
 function observeActive(){
  const active=Math.max(0,performance.now()-segmentStarted-suspendedMs);
  statistics.max_active_work_before_yield_ms=Math.max(statistics.max_active_work_before_yield_ms,active);
  return active;
 }
 async function awaitIdle(pending){
  // The operation is called before entering this helper: its synchronous setup
  // remains active work. Only the time our continuation is suspended is removed.
  const before=performance.now();
  try{return await pending;}
  finally{const idle=performance.now()-before;suspendedMs+=idle;statistics.excluded_await_idle_ms+=idle;}
 }
 function observeWork(phase,started,rows=0){
  const elapsed=performance.now()-started;yieldRows+=rows;
  statistics.max_slice_ms=Math.max(statistics.max_slice_ms,elapsed);
  statistics.max_rows_between_yields=Math.max(statistics.max_rows_between_yields,yieldRows);
  statistics.phases[phase].max_slice_ms=Math.max(statistics.phases[phase].max_slice_ms,elapsed);
  statistics.phases[phase].total_ms+=elapsed;
  observeActive();
 }
 function ensureCurrent(){if(mine!==generation||controller.signal.aborted)throw new DOMException('Aborted','AbortError');}
 async function cooperate(){
  // Earlier yields reserve headroom within the approved8ms work budget.
  if(yieldRows>=2048||observeActive()>=4){
   statistics.yield_count++;const pending=yieldTask();observeActive();await pending;
   segmentStarted=performance.now();suspendedMs=0;yieldRows=0;ensureCurrent();
  }
 }
 try{
  let stream;
  if(message.file){if(message.file.size>268435456)throw new Error('File exceeds256MiB input limit');stream=message.file.stream();}
  else{const response=await awaitIdle(fetch(message.url,{signal:controller.signal}));if(!response.ok)throw new Error(`Recording download HTTP${response.status}`);const size=response.headers.get('content-length');if(size!==null&&Number(size)>268435456)throw new Error('Artifact exceeds256MiB input limit');stream=response.body;}
  if(!stream)throw new Error('Streaming body unavailable');reader=stream.getReader();
  let lastProgress=0;
  while(true){
   const result=await awaitIdle(reader.read());if(result.done)break;
   // Split by complete row boundaries as well as bytes, so no parser call can
   // postpone the row/yield budget behind a large browser-owned chunk.
   const bytes=result.value;let start=0,rows=0,batchStarted=performance.now();
   for(let i=0;i<bytes.length;i++){
    if(bytes[i]===10)rows++;
    if(rows>=64||i-start>=8191||i===bytes.length-1){
     ensureCurrent();const before=parser.parsedRows;
     try{parser.write(bytes.subarray(start,i+1));}finally{observeWork('csv',batchStarted,parser.parsedRows-before);}
     start=i+1;rows=0;
     statistics.parsed_rows=parser.parsedRows;
     await cooperate();
     batchStarted=performance.now();
    }
   }
   if(performance.now()-lastProgress>=150){send('progress',{rows:parser.parsedRows,metrics:metrics()});lastProgress=performance.now();await cooperate();}
  }
  ensureCurrent();
  const finishStarted=performance.now(),beforeFinish=parser.parsedRows;
  try{recording=parser.finish();}finally{observeWork('finish',finishStarted,parser.parsedRows-beforeFinish);}
  statistics.parsed_rows=recording.rowCount;parser=null;observedParser=null;replay=new Replay(recording);await cooperate();
  let plotStarted=performance.now();
  const indices=selectPlotRows(recording),points=new Float64Array(indices.length*20);
  observeWork('plot',plotStarted);await cooperate();plotStarted=performance.now();
  for(let i=0;i<indices.length;i++){
   const row=recording.row(indices[i]),errors=rowErrors(row);
   points.set([row.tick,row.timeSeconds,...row.truth.slice(0,2),...row.estimate.slice(0,2),...row.held,...row.integrated,...(row.position??[NaN,NaN]),...(row.velocity??[NaN,NaN]),Number(row.outage),errors.filter,errors.hold,errors.integration,row.covariance[0],row.covariance[5]],i*20);
   if(i%64===63||i===indices.length-1){observeWork('plot',plotStarted);await cooperate();plotStarted=performance.now();}
  }
  send('loaded',{rowCount:recording.rowCount,allocatedNumericBytes:recording.allocatedNumericBytes,points,metrics:metrics()},[points.buffer]);
  sendFrame(true);
  await cooperate();
 }catch(error){clear();if(error.name!=='AbortError'&&mine===generation)send('error',{message:String(error.message).slice(0,512)});}
 finally{try{if(reader)await awaitIdle(reader.cancel());}catch{/* Bounded input teardown; no service mutation. */}reader?.releaseLock();reader=null;controller=null;parser=null;observedParser=null;observeActive();if(mine===generation)send('metrics',{metrics:metrics()});observeActive();}
}
function sendFrame(force=false){const now=performance.now();if(!replay)return;if(framePending){dirtyFrame ||= force;return;}if(!force&&now-lastFrame<1000/30)return;lastFrame=now;framePending=true;dirtyFrame=false;send('frame',{row:replay.frame(now),playing:replay.playing,controlId:lastControlId});}
// Reset is serialized: the main thread has one pending replacement and waits
// for this acknowledgment before sending it. No concurrent parser/store owners.
onmessage=async({data:message})=>{
 const callbackStarted=performance.now();
 if(message.type==='resetInput'){generation=message.generation;await stop();send('resetAck',{metrics:metrics(),controlId:message.controlId});return;}
 if(message.generation!==generation)return;
 if(message.type==='load'){if(job||recording)return;job=load(message);await job;job=null;return;}
 if(message.type==='frameAck'){framePending=false;if(dirtyFrame)sendFrame(true);statistics.control_callback_max_ms=Math.max(statistics.control_callback_max_ms??0,performance.now()-callbackStarted);if(!framePending&&!replay?.playing)send('metrics',{metrics:metrics()});return;}
 if(!replay)return;
 if(Number.isSafeInteger(message.controlId))lastControlId=message.controlId;
 try{
  const now=performance.now();
  if(message.type==='play')replay.play(now);
  if(message.type==='pause')replay.pause(now);
  if(message.type==='seek')replay.seek(message.index);
  if(message.type==='resetCursor')replay.reset();
  if(message.type==='speed')replay.setSpeed(message.rate,now);
  sendFrame(true);
 }catch(error){send('error',{message:String(error.message).slice(0,512)});}
 finally{statistics.control_callback_max_ms=Math.max(statistics.control_callback_max_ms??0,performance.now()-callbackStarted);}
};
setInterval(()=>{if(replay?.playing)sendFrame();},34);
