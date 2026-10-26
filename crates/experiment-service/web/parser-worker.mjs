import {CsvRecordingParser,Replay,selectPlotRows,rowErrors} from './data.mjs';

let generation=0,controller=null,reader=null,job=null,recording=null,replay=null;
let framePending=false,dirtyFrame=false,lastFrame=0;
const yieldTask=()=>new Promise(resolve=>setTimeout(resolve,0));
const send=(type,payload={},transfer=[])=>postMessage({type,generation,...payload},transfer);
function clear(){recording=null;replay=null;framePending=false;dirtyFrame=false;}
async function stop(){controller?.abort();try{await reader?.cancel();}catch{/* The original load reports its own failure. */}await job;clear();}

async function load(message){
 const mine=generation;controller=new AbortController();
 let parser=new CsvRecordingParser();
 try{
  let stream;
  if(message.file){if(message.file.size>268435456)throw new Error('File exceeds256MiB input limit');stream=message.file.stream();}
  else{const response=await fetch(message.url,{signal:controller.signal});if(!response.ok)throw new Error(`Recording download HTTP${response.status}`);const size=response.headers.get('content-length');if(size!==null&&Number(size)>268435456)throw new Error('Artifact exceeds256MiB input limit');stream=response.body;}
  if(!stream)throw new Error('Streaming body unavailable');reader=stream.getReader();
  let lastYield=performance.now(),yieldRows=0,lastProgress=0;
  while(true){
   const result=await reader.read();if(result.done)break;
   // Split by complete row boundaries as well as bytes, so no parser call can
   // postpone the row/yield budget behind a large browser-owned chunk.
   const bytes=result.value;let start=0,rows=0;
   for(let i=0;i<bytes.length;i++){
    if(bytes[i]===10)rows++;
    if(rows>=64||i-start>=8191||i===bytes.length-1){
     if(mine!==generation||controller.signal.aborted)throw new DOMException('Aborted','AbortError');
     const before=parser.parsedRows;parser.write(bytes.subarray(start,i+1));yieldRows+=parser.parsedRows-before;start=i+1;rows=0;
     if(yieldRows>=2048||performance.now()-lastYield>=8){await yieldTask();yieldRows=0;lastYield=performance.now();}
    }
   }
   if(performance.now()-lastProgress>=150){send('progress',{rows:parser.parsedRows});lastProgress=performance.now();}
  }
  if(mine!==generation||controller.signal.aborted)throw new DOMException('Aborted','AbortError');
  recording=parser.finish();parser=null;replay=new Replay(recording);
  const indices=selectPlotRows(recording),points=new Float64Array(indices.length*20);
  lastYield=performance.now();
  for(let i=0;i<indices.length;i++){
   const row=recording.row(indices[i]),errors=rowErrors(row);
   points.set([row.tick,row.timeSeconds,...row.truth.slice(0,2),...row.estimate.slice(0,2),...row.held,...row.integrated,...(row.position??[NaN,NaN]),...(row.velocity??[NaN,NaN]),Number(row.outage),errors.filter,errors.hold,errors.integration,row.covariance[0],row.covariance[5]],i*20);
   if(i%64===63&&performance.now()-lastYield>=8){await yieldTask();lastYield=performance.now();if(mine!==generation||controller.signal.aborted)throw new DOMException('Aborted','AbortError');}
  }
  send('loaded',{rowCount:recording.rowCount,allocatedNumericBytes:recording.allocatedNumericBytes,points},[points.buffer]);
  sendFrame(true);
 }catch(error){clear();if(error.name!=='AbortError'&&mine===generation)send('error',{message:String(error.message).slice(0,512)});}
 finally{try{await reader?.cancel();}catch{/* Bounded input teardown; no service mutation. */}reader?.releaseLock();reader=null;controller=null;parser=null;}
}
function sendFrame(force=false){const now=performance.now();if(!replay)return;if(framePending){dirtyFrame ||= force;return;}if(!force&&now-lastFrame<1000/30)return;lastFrame=now;framePending=true;dirtyFrame=false;send('frame',{row:replay.frame(now),playing:replay.playing});}
// Reset is serialized: the main thread has one pending replacement and waits
// for this acknowledgment before sending it. No concurrent parser/store owners.
onmessage=async({data:message})=>{
 if(message.type==='resetInput'){generation=message.generation;await stop();send('resetAck');return;}
 if(message.generation!==generation)return;
 if(message.type==='load'){if(job||recording)return;job=load(message);await job;job=null;return;}
 if(message.type==='frameAck'){framePending=false;if(dirtyFrame)sendFrame(true);return;}
 if(!replay)return;
 try{
  const now=performance.now();
  if(message.type==='play')replay.play(now);
  if(message.type==='pause')replay.pause(now);
  if(message.type==='seek')replay.seek(message.index);
  if(message.type==='resetCursor')replay.reset();
  if(message.type==='speed')replay.setSpeed(message.rate,now);
  sendFrame(true);
 }catch(error){send('error',{message:String(error.message).slice(0,512)});}
};
setInterval(()=>{if(replay?.playing)sendFrame();},34);
