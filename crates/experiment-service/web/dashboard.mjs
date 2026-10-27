import {parseSnapshot,serializeConfig,rowErrors,positionEllipse} from './data.mjs';

const byId=id=>document.getElementById(id);
const worker=new Worker('/assets/parser-worker.mjs',{type:'module'});
let activeWorkers=1;
let generation=0,resetPending=false,pendingInput=null,selected='',events=null,retryTimer=null,retries=0,lastSequence=-1n;
let points=null,current=null,rowCount=0,playing=false,dirty=false,renderedAt=0,loadedRun='',statusGeneration=0,submitting=false;
const colors=['#f0f4fa','#85e1c4','#e9bb73','#d2a6ef','#77b9ff'];
const measuredControls=new Map(),controlLatencies=[],presentationPending=new Map();
let controlSerial=0,supersededControls=0,totalControlSamples=0,presentationScheduled=false;
const renderedControlIds=new Set();
let longTasks=null;
try{
 if(PerformanceObserver.supportedEntryTypes.includes('longtask')){
  const observed={count:0,max_ms:0};
  new PerformanceObserver(list=>{for(const entry of list.getEntries()){observed.count++;observed.max_ms=Math.max(observed.max_ms,entry.duration);}}).observe({type:'longtask'});
  longTasks=observed;
 }
}catch{/* Unsupported performance observation is explicitly unavailable. */}
let workerMetrics=null;
function renderDiagnostics(){
 const inputSamples=controlLatencies.map(sample=>sample.input_to_next_raf_proxy_ms).filter(value=>value!==null).sort((a,b)=>a-b);
 const p95=inputSamples.length?inputSamples[Math.ceil(inputSamples.length*.95)-1]:null;
 byId('diagnostics').textContent=JSON.stringify({
  generation,active_workers:activeWorkers,selected_sse:Number(events!==null),pending_selection:Number(pendingInput!==null),reset_pending:resetPending,
  numeric_buffers_bytes:workerMetrics?.numeric_buffers_bytes??null,worker:workerMetrics,
  retained_plot_payloads:Number(points!==null),plot_payload_bytes:points?.byteLength??0,
  canvas_pixels:['trajectory','timeline'].map(id=>({id,width:byId(id).width,height:byId(id).height})),
  control_timing:{samples_retained:controlLatencies.length,samples_total:totalControlSamples,input_timestamp_samples:inputSamples.length,p95_input_to_next_raf_proxy_ms:p95,max_input_to_next_raf_proxy_ms:inputSamples.at(-1)??null,pending:measuredControls.size,pending_presentation:presentationPending.size,superseded:supersededControls,retention_limit:64,recent_samples:controlLatencies,presentation_limitation:'The next animation-frame opportunity is a presentation proxy, not proof of physical display. Missing input timestamps remain null.'},
  main_thread_long_tasks:longTasks,renderer_rss_bytes:null,renderer_rss_note:'Use the separately attributed native process collector.'
 },null,2);
}
function notice(text){byId('notice').textContent=String(text).slice(0,1024);}
function inputTiming(event){
 const handlerAt=performance.now(),stamp=event?.timeStamp;
 const inputAt=typeof stamp==='number'&&Number.isFinite(stamp)&&stamp>=0&&stamp<=handlerAt&&handlerAt-stamp<600000?stamp:null;
 return {handlerAt,inputAt};
}
function beginControl(type,timing){
 const previous=measuredControls.get(type);if(previous){supersededControls++;renderedControlIds.delete(previous.id);}
 const id=++controlSerial;measuredControls.set(type,{id,type,...timing});return id;
}
function presentControls(timestamp){
 for(const [type,entry]of presentationPending){
  if(timestamp<=entry.drawFrameAt)continue;
  const observed=performance.now();
  controlLatencies.push({type,event_queue_delay_ms:entry.inputAt===null?null:entry.handlerAt-entry.inputAt,handler_to_draw_ms:entry.drawAt-entry.handlerAt,input_to_draw_ms:entry.inputAt===null?null:entry.drawAt-entry.inputAt,input_to_next_raf_proxy_ms:entry.inputAt===null?null:observed-entry.inputAt,handler_to_next_raf_proxy_ms:observed-entry.handlerAt});
  if(controlLatencies.length>64)controlLatencies.shift();totalControlSamples++;presentationPending.delete(type);
 }
 if(presentationPending.size)requestAnimationFrame(presentControls);else presentationScheduled=false;
 renderDiagnostics();
}
function command(type,payload={}){
 const message={type,generation,...payload};
 if(['play','pause','seek','resetCursor','speed','resetInput'].includes(type)&&payload.timing&&payload.measure!==false){
  message.controlId=beginControl(type,payload.timing);
 }
 worker.postMessage(message);
}
function controls(enabled){for(const id of ['play','pause','reset','cursor','tick','speed'])byId(id).disabled=!enabled;}
function selectInput(input,timing=null){pendingInput=input;if(resetPending){const pending=measuredControls.get('resetInput');if(pending&&timing){Object.assign(pending,timing);supersededControls++;}return;}resetPending=true;generation++;points=null;current=null;rowCount=0;playing=false;supersededControls+=measuredControls.size;measuredControls.clear();renderedControlIds.clear();controls(false);byId('fidelity').textContent='No recording loaded';byId('sensor').textContent='Sensor status awaits the selected recording.';byId('sample').textContent='No sample from the selected recording is loaded.';for(const id of ['cursor','tick']){byId(id).value='0';byId(id).max='0';}dirty=true;command('resetInput',{timing});}
function acknowledgeControl(id){if([...measuredControls.values()].some(entry=>entry.id===id))renderedControlIds.add(id);}
worker.onmessage=({data:message})=>{
 if(message.generation!==generation)return;
 if(message.metrics){workerMetrics=message.metrics;renderDiagnostics();}
 if(message.type==='resetAck'){
  acknowledgeControl(message.controlId);dirty=true;
  resetPending=false;const input=pendingInput;pendingInput=null;
  if(input)command('load',input);return;
 }
 if(message.type==='progress')notice(`Loading recording: ${message.rows.toLocaleString()} rows parsed…`);
 if(message.type==='loaded'){
  points=message.points;rowCount=message.rowCount;
  byId('fidelity').textContent=`${points.length/20} plotted / ${rowCount.toLocaleString()} recorded samples · 12-decimal CSV`;
  for(const id of ['cursor','tick']){byId(id).max=String(rowCount-1);byId(id).value='0';}
  command('speed',{rate:Number(byId('speed').value),measure:false});controls(true);notice(`Recording ready · ${(message.allocatedNumericBytes/1048576).toFixed(2)} MiB numeric buffers. Replay controls do not alter experiments.`);dirty=true;
 }
 if(message.type==='frame'){current=message.row;playing=message.playing;acknowledgeControl(message.controlId);byId('cursor').value=String(current.tick);byId('tick').value=String(current.tick);dirty=true;command('frameAck');}
 if(message.type==='error'){points=null;current=null;controls(false);notice(message.message);dirty=true;}
};
worker.onerror=()=>{controls(false);notice('The parsing worker failed. Reload the page to restore its single owner.');};

function closeEvents(){events?.close();events=null;clearTimeout(retryTimer);retryTimer=null;}
async function getStatus(id,mine){const response=await fetch(`/v1/runs/${id}`);if(!response.ok)throw new Error(`Run status HTTP${response.status}${response.status===404?' (recording evicted)':''}`);const snapshot=parseSnapshot(await response.text());if(mine!==statusGeneration||id!==selected)return null;return snapshot;}
function acceptSnapshot(snapshot,mine){
 if(mine!==statusGeneration||snapshot.id!==selected)return;
 const sequence=BigInt(snapshot.sequence);if(sequence<lastSequence)return;lastSequence=sequence;
 for(const option of byId('runs').options)if(option.value===snapshot.id)option.textContent=`${snapshot.id} · ${snapshot.state}`;
 byId('cancel').disabled=snapshot.state!=='running';
 notice(`${snapshot.id} · ${snapshot.state}${snapshot.error?` · ${snapshot.error.code}: ${snapshot.error.message}`:''}${snapshot.state==='running'?' · Numerical traces become available after completion.':''}`);
 if(snapshot.state!=='running')closeEvents();
 if(snapshot.state==='completed'&&loadedRun!==snapshot.id){loadedRun=snapshot.id;selectInput({url:`/v1/runs/${snapshot.id}/artifacts/samples.csv`});}
}
function connectEvents(id,mine){
 if(mine!==statusGeneration||id!==selected)return;
 events=new EventSource(`/v1/runs/${id}/events`);
 const update=event=>{try{const snapshot=parseSnapshot(event.data);acceptSnapshot(snapshot,mine);}catch(error){closeEvents();notice(error.message);}};
 events.addEventListener('snapshot',update);events.onmessage=update;
 events.onerror=()=>{
  closeEvents();if(mine!==statusGeneration||id!==selected)return;
  if(retries>=3){notice('Status connection lost after three recovery attempts. Use Refresh to reconnect.');return;}
  const delay=[1000,2000,4000][retries++];
  retryTimer=setTimeout(async()=>{try{const snapshot=await getStatus(id,mine);if(snapshot){acceptSnapshot(snapshot,mine);if(snapshot.state==='running')connectEvents(id,mine);}}catch(error){notice(error.message);if(mine===statusGeneration&&retries<3)connectEvents(id,mine);}},delay);
 };
}
async function chooseRun(id,timing=null){
 closeEvents();selected=id;loadedRun='';lastSequence=-1n;retries=0;const mine=++statusGeneration;
 selectInput(null,timing);byId('cancel').disabled=true;byId('selected-name').textContent=id||'Recorded replay';
 if(!id){notice('Choose a completed run or import a CSV.');return;}
 try{const snapshot=await getStatus(id,mine);if(snapshot){acceptSnapshot(snapshot,mine);if(snapshot.state==='running')connectEvents(id,mine);}}catch(error){if(mine===statusGeneration)notice(error.message);}
}
async function refresh(timing=null){
 try{const response=await fetch('/v1/runs');if(!response.ok)throw new Error(`Run list HTTP${response.status}`);const list=JSON.parse(await response.text());if(!Array.isArray(list)||list.length>32)throw new Error('Invalid bounded run list');
  const select=byId('runs');select.replaceChildren(new Option('Choose a run',''));
  for(const item of list){if(!item||typeof item.id!=='string'||!/^run-[1-9]\d*$/.test(item.id)||!['running','completed','failed','cancelled'].includes(item.state))throw new Error('Invalid run list entry');select.add(new Option(`${item.id} · ${item.state}`,item.id));}
  select.value=selected;if(selected&&!list.some(item=>item.id===selected)){notice('Selected run was evicted. Its already-loaded browser recording can still be replayed.');closeEvents();byId('cancel').disabled=true;}else if(selected)await chooseRun(selected,timing);else notice('Local service connected. Start an experiment or import a recording.');
 }catch(error){notice(error.message);}
}
byId('runs').onchange=event=>chooseRun(event.target.value,inputTiming(event));
byId('refresh').onclick=event=>refresh(inputTiming(event));
byId('configuration').onsubmit=async event=>{
 event.preventDefault();if(submitting)return;
 const form=new FormData(event.currentTarget);let body;
 try{body=serializeConfig({seed:String(form.get('seed')),duration_seconds:Number(form.get('duration_seconds')),scenario:form.get('scenario'),position_sigma:Number(form.get('position_sigma')),velocity_sigma:Number(form.get('velocity_sigma')),acceleration_density:Number(form.get('acceleration_density')),outage:form.has('outage_enabled')?{start:Number(form.get('outage_start')),end:Number(form.get('outage_end'))}:null});}catch(error){notice(error.message);return;}
 submitting=true;byId('create').disabled=true;
 try{const response=await fetch('/v1/runs',{method:'POST',headers:{'Content-Type':'application/json'},body});if(!response.ok){const text=await response.text();notice(`Submission HTTP${response.status}: ${text.slice(0,512)}`);return;}const snapshot=parseSnapshot(await response.text());selected=snapshot.id;await refresh();}
 catch(error){notice(`Submission outcome uncertain: ${error.message}. It will not be resubmitted. Refresh the run list to inspect the outcome.`);}
 finally{submitting=false;byId('create').disabled=false;}
};
byId('cancel').onclick=async()=>{const id=selected,mine=statusGeneration;byId('cancel').disabled=true;notice('Cancellation requested; waiting for service confirmation and cleanup.');try{const response=await fetch(`/v1/runs/${id}`,{method:'DELETE'});if(!response.ok)throw new Error(`Cancellation HTTP${response.status}: ${(await response.text()).slice(0,256)}`);const snapshot=parseSnapshot(await response.text());acceptSnapshot(snapshot,mine);}catch(error){notice(error.message);if(mine===statusGeneration)byId('cancel').disabled=false;}};
byId('file').onchange=event=>{const timing=inputTiming(event),file=event.target.files[0];if(!file)return;closeEvents();selected='';statusGeneration++;loadedRun='';byId('runs').value='';byId('cancel').disabled=true;byId('selected-name').textContent=`Local file · ${file.name.slice(0,160)}`;notice('Loading local CSV; no upload or experiment mutation.');selectInput({file},timing);event.target.value='';};
for(const [id,type]of [['play','play'],['pause','pause'],['reset','resetCursor']])byId(id).onclick=event=>command(type,{timing:inputTiming(event)});
byId('speed').onchange=event=>command('speed',{rate:Number(event.target.value),timing:inputTiming(event)});
for(const id of ['cursor','tick'])byId(id).oninput=event=>{const timing=inputTiming(event),index=Number(event.target.value);if(Number.isInteger(index)&&index>=0&&index<rowCount)command('seek',{index,timing});};

function canvasSetup(id){const canvas=byId(id);const bounds=canvas.getBoundingClientRect();const width=Math.min(4096,Math.max(1,Math.round(bounds.width*devicePixelRatio))),height=Math.min(2160,Math.max(1,Math.round(bounds.height*devicePixelRatio)));if(canvas.width!==width||canvas.height!==height){canvas.width=width;canvas.height=height;}const context=canvas.getContext('2d');context.clearRect(0,0,width,height);context.font=`${Math.max(12,12*devicePixelRatio)}px Segoe UI`;return {context,width,height};}
function renderTrajectory(){
 const {context:c,width:w,height:h}=canvasSetup('trajectory');if(!points||!current)return;
 let minX=Infinity,maxX=-Infinity,minY=Infinity,maxY=-Infinity;
 for(let i=0;i<points.length;i+=20)for(const offset of [2,4,6,8,10]){const x=points[i+offset],y=points[i+offset+1];if(Number.isFinite(x)&&Number.isFinite(y)){minX=Math.min(minX,x);maxX=Math.max(maxX,x);minY=Math.min(minY,y);maxY=Math.max(maxY,y);}}
 const ellipse=positionEllipse(current);if(ellipse){minX=Math.min(minX,current.estimate[0]-ellipse.major);maxX=Math.max(maxX,current.estimate[0]+ellipse.major);minY=Math.min(minY,current.estimate[1]-ellipse.major);maxY=Math.max(maxY,current.estimate[1]+ellipse.major);}
 const pad=35*devicePixelRatio,scale=Math.min((w-2*pad)/Math.max(1e-6,maxX-minX),(h-2*pad)/Math.max(1e-6,maxY-minY));
 const x=value=>w/2+(value-(minX+maxX)/2)*scale,y=value=>h/2-(value-(minY+maxY)/2)*scale;
 c.strokeStyle='#283b4d';c.lineWidth=1;c.strokeRect(pad,pad,w-2*pad,h-2*pad);
 for(let trace=0;trace<4;trace++){c.beginPath();c.strokeStyle=colors[trace];c.lineWidth=(trace===1?2:1.3)*devicePixelRatio;c.setLineDash(trace===2?[8,5]:trace===3?[2,4]:[]);for(let i=0;i<points.length;i+=20){const offset=2+trace*2;if(i===0)c.moveTo(x(points[i+offset]),y(points[i+offset+1]));else c.lineTo(x(points[i+offset]),y(points[i+offset+1]));}c.stroke();}
 c.setLineDash([]);c.strokeStyle=colors[4];c.lineWidth=devicePixelRatio;
 for(let i=0;i<points.length;i+=20)if(Number.isFinite(points[i+10])){const px=x(points[i+10]),py=y(points[i+11]),size=2.5*devicePixelRatio;c.beginPath();c.moveTo(px-size,py-size);c.lineTo(px+size,py+size);c.moveTo(px-size,py+size);c.lineTo(px+size,py-size);c.stroke();}
 if(ellipse){c.strokeStyle=colors[1];c.lineWidth=2*devicePixelRatio;c.beginPath();c.ellipse(x(current.estimate[0]),y(current.estimate[1]),ellipse.major*scale,ellipse.minor*scale,-ellipse.angle,0,Math.PI*2);c.stroke();}
 for(const [point,color]of [[current.truth,colors[0]],[current.estimate,colors[1]]]){c.fillStyle=color;c.beginPath();c.arc(x(point[0]),y(point[1]),4*devicePixelRatio,0,Math.PI*2);c.fill();}
 c.fillStyle='#b0c1d1';c.fillText(`x ${minX.toFixed(2)} … ${maxX.toFixed(2)} m`,pad,h-8*devicePixelRatio);c.fillText(`y ${minY.toFixed(2)} … ${maxY.toFixed(2)} m`,pad,20*devicePixelRatio);
}
function renderTimeline(){
 const {context:c,width:w,height:h}=canvasSetup('timeline');if(!points||!current)return;const pad=30*devicePixelRatio,last=points.at(-19);let max=1e-9;
 for(let i=0;i<points.length;i+=20)for(const offset of [15,16,17])max=Math.max(max,points[i+offset]);
 for(let i=0;i<points.length;i+=20)max=Math.max(max,Math.sqrt(Math.max(0,points[i+18]+points[i+19])));
 const x=t=>pad+(w-2*pad)*t/Math.max(.01,last),y=value=>h-pad-(h-2*pad)*value/max;
 c.fillStyle='#e9bb7324';for(let i=0;i<points.length;i+=20)if(points[i+14]){const end=i+20<points.length?points[i+21]:points[i+1];c.fillRect(x(points[i+1]),pad,Math.max(1,x(end)-x(points[i+1])),h-2*pad);}
 for(let trace=0;trace<4;trace++){c.strokeStyle=trace===3?colors[4]:colors[trace+1];c.setLineDash(trace===1?[8,5]:trace===2?[2,4]:trace===3?[7,3,1,3]:[]);c.lineWidth=1.5*devicePixelRatio;c.beginPath();for(let i=0;i<points.length;i+=20){const value=trace===3?Math.sqrt(Math.max(0,points[i+18]+points[i+19])):points[i+15+trace];if(i===0)c.moveTo(x(points[i+1]),y(value));else c.lineTo(x(points[i+1]),y(value));}c.stroke();}
 c.setLineDash([]);c.strokeStyle='#edf3f8';c.beginPath();c.moveTo(x(current.timeSeconds),pad);c.lineTo(x(current.timeSeconds),h-pad);c.stroke();c.fillStyle='#b0c1d1';c.fillText(`0 … ${last.toFixed(2)} s · scale 0 … ${max.toFixed(3)} m`,pad,h-8*devicePixelRatio);
}
function render(now){requestAnimationFrame(render);if(!dirty||now-renderedAt<1000/30)return;dirty=false;renderedAt=now;renderTrajectory();renderTimeline();if(current){const errors=rowErrors(current),ellipse=positionEllipse(current);byId('sensor').textContent=`Tick ${current.tick} · ${current.timeSeconds.toFixed(2)} s · Position: ${current.outage?'recorded outage':current.position?'received sample':'unscheduled tick'} · Velocity: ${current.velocity?'received sample':'unscheduled tick'} · ${playing?'playing':'paused'}`;byId('sample').textContent=JSON.stringify({...current,positionErrors:errors,descriptive95Ellipse:ellipse??'invalid rounded covariance'},null,2);}
 for(const [type,entry]of measuredControls)if(renderedControlIds.has(entry.id)){
  if(presentationPending.has(type))supersededControls++;
  presentationPending.set(type,{...entry,drawAt:performance.now(),drawFrameAt:now});measuredControls.delete(type);
 }
 if(presentationPending.size&&!presentationScheduled){presentationScheduled=true;requestAnimationFrame(presentControls);}
 renderedControlIds.clear();renderDiagnostics();
}
new ResizeObserver(()=>{dirty=true;}).observe(byId('trajectory'));
window.addEventListener('pagehide',()=>{closeEvents();worker.terminate();activeWorkers=0;});
requestAnimationFrame(render);refresh();
