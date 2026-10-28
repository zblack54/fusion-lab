// Production-owned adapter regressions. These execute the real module with
// controlled browser ports; native-browser acceptance is separately required.
import test from 'node:test';
import assert from 'node:assert/strict';

async function adapter(caseId,options={}){
 const elements=new Map();
 const element=id=>{if(!elements.has(id))elements.set(id,{value:id==='speed'?'4':'0',textContent:'',disabled:false,options:[],replaceChildren(...options){this.options=[...options];this.value=options[0]?.value??'';},add(option){this.options.push(option);},getBoundingClientRect(){return {width:400,height:300};},getContext(){return {clearRect(){}};}});return elements.get(id);};
 const messages=[],frames=[];let port,clock=2000;
 const keys=['document','window','Worker','ResizeObserver','requestAnimationFrame','fetch','Option','PerformanceObserver','devicePixelRatio','performance','FormData'];
 const saved=new Map(keys.map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]));
 globalThis.document={getElementById:element};globalThis.window={addEventListener(){}};
 globalThis.Worker=class{constructor(){port=this;}postMessage(message){messages.push(message);}};
 globalThis.ResizeObserver=class{observe(){}};globalThis.requestAnimationFrame=callback=>frames.push(callback);globalThis.devicePixelRatio=1;
 if(options.observer)globalThis.PerformanceObserver=options.observer;
 if(options.clock)Object.defineProperty(globalThis,'performance',{value:{now:()=>clock},configurable:true});
 if(options.form)globalThis.FormData=class{get(name){return options.form[name]??null;}has(name){return Object.hasOwn(options.form,name);}};
 globalThis.Option=class{constructor(text,value){this.textContent=text;this.value=value;}};
 globalThis.fetch=options.fetch??(async url=>({ok:true,text:async()=>url==='/v1/runs'?'[]':JSON.stringify({id:'run-2',state:'completed',sequence:2,config:{seed:42,duration_seconds:1,scenario:'straight',position_sigma:0,velocity_sigma:0,acceleration_density:.1,outage:null},samples:101,error:null,artifacts:['/v1/runs/run-2/artifacts/samples.csv']})}));
 try{await import(`./dashboard.mjs?regression=${caseId}`);await new Promise(resolve=>setImmediate(resolve));}
 catch(error){for(const [key,descriptor]of saved)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else delete globalThis[key];throw error;}
 return {element,messages,port,runFrame(timestamp){clock=timestamp+2;const current=frames.splice(0);for(const callback of current)callback(timestamp);},restore(){for(const [key,descriptor]of saved)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else delete globalThis[key];}};
}
test('issue52 replacement recording clears previous exact metadata immediately',async()=>{
 const app=await adapter(52);
 try{app.element('sample').textContent='OLD SAMPLE tick3000';app.element('sensor').textContent='OLD SENSOR';app.element('fidelity').textContent='OLD ROWCOUNT';app.element('tick').value='3000';
  await app.element('runs').onchange({target:{value:'run-2'}});
  assert.doesNotMatch(app.element('sample').textContent,/OLD SAMPLE/);assert.doesNotMatch(app.element('sensor').textContent,/OLD SENSOR/);assert.doesNotMatch(app.element('fidelity').textContent,/OLD ROWCOUNT/);assert.equal(app.element('tick').value,'0');assert.equal(app.element('speed').disabled,true);
 }finally{app.restore();}
});
test('issue53 loaded replacement applies the visible playback speed before enabling controls',async()=>{
 const app=await adapter(53);
 try{await app.element('runs').onchange({target:{value:'run-2'}});const generation=app.messages.find(message=>message.type==='resetInput').generation;
  app.port.onmessage({data:{type:'loaded',generation,rowCount:101,allocatedNumericBytes:28280,points:new Float64Array(40)}});
  assert.ok(app.messages.some(message=>message.type==='speed'&&message.generation===generation&&message.rate===4));assert.equal(app.element('speed').disabled,false);
 }finally{app.restore();}
});

test('issue57 unavailable LongTask observer remains unavailable after observe throws',async()=>{
 const app=await adapter('57-observer',{observer:class{static supportedEntryTypes=['longtask'];observe(){throw new Error('unsupported observer');}}});
 try{app.port.onmessage({data:{type:'metrics',generation:0,metrics:{input_readers:0}}});assert.equal(JSON.parse(app.element('diagnostics').textContent).main_thread_long_tasks,null);}
 finally{app.restore();}
});

test('issue57 timing includes event delay and a later frame proxy with bounded retention',async()=>{
 const app=await adapter('57-timing',{clock:true});
 try{
  app.element('file').onchange({timeStamp:1975,target:{files:[{name:'fixture.csv'}]}});
  const reset=app.messages.find(message=>message.type==='resetInput');
  app.port.onmessage({data:{type:'resetAck',generation:reset.generation,controlId:reset.controlId,metrics:{input_readers:0}}});
  app.runFrame(2010);let diagnostics=JSON.parse(app.element('diagnostics').textContent);assert.equal(diagnostics.control_timing.samples_total,0,'drawing alone is not the presentation proxy');
  app.runFrame(2044);diagnostics=JSON.parse(app.element('diagnostics').textContent);const sample=diagnostics.control_timing.recent_samples[0];
  assert.equal(sample.event_queue_delay_ms,25);assert.equal(sample.handler_to_draw_ms,12);assert.equal(sample.input_to_draw_ms,37);assert.equal(sample.input_to_next_raf_proxy_ms,71);
  for(let i=0;i<70;i++){
   const timestamp=2100+i*100;app.runFrame(timestamp);
   app.element('file').onchange({timeStamp:timestamp,target:{files:[{name:'fixture.csv'}]}});
   const next=app.messages.at(-1);app.port.onmessage({data:{type:'resetAck',generation:next.generation,controlId:next.controlId,metrics:{input_readers:0}}});
   app.runFrame(timestamp+34);app.runFrame(timestamp+68);
  }
  diagnostics=JSON.parse(app.element('diagnostics').textContent);assert.equal(diagnostics.control_timing.samples_total,71);assert.equal(diagnostics.control_timing.samples_retained,64);assert.equal(diagnostics.control_timing.pending,0);assert.equal(diagnostics.control_timing.pending_presentation,0);
 }finally{app.restore();}
});

for(const refreshFails of [false,true])test(`issue60 uncertain POST refresh ${refreshFails?'failure stays explicit':'preserves old view without inferring a match'}`,async()=>{
 const calls=[];let submitted=false;
 const snapshot={id:'run-1',state:'completed',sequence:2,config:{seed:42,duration_seconds:1,scenario:'straight',position_sigma:0,velocity_sigma:0,acceleration_density:.1,outage:null},samples:101,error:null,artifacts:['/v1/runs/run-1/artifacts/samples.csv']};
 const app=await adapter(`60-${refreshFails}`,{form:{seed:'42',duration_seconds:'1',scenario:'straight',position_sigma:'0',velocity_sigma:'0',acceleration_density:'0.1'},fetch:async(url,options={})=>{
  calls.push({url,method:options.method??'GET'});
  if(options.method==='POST'){submitted=true;throw new TypeError('injected accepted response lost');}
  if(url==='/v1/runs'&&submitted&&refreshFails)return {ok:false,status:503,text:async()=>''};
  return {ok:true,text:async()=>JSON.stringify(url==='/v1/runs'?(submitted?[{...snapshot,id:'run-2'}]:[snapshot]):snapshot)};
 }});
 try{
  app.element('runs').value='run-1';await app.element('runs').onchange({target:{value:'run-1'}});const reset=app.messages.find(message=>message.type==='resetInput');
  app.port.onmessage({data:{type:'resetAck',generation:reset.generation,controlId:reset.controlId}});
  app.port.onmessage({data:{type:'loaded',generation:reset.generation,rowCount:101,allocatedNumericBytes:28280,points:new Float64Array(40)}});
  app.port.onmessage({data:{type:'frame',generation:reset.generation,row:{tick:90},playing:false}});
  const messagesBefore=app.messages.length;
  await app.element('configuration').onsubmit({preventDefault(){},currentTarget:{}});
  assert.equal(calls.filter(call=>call.method==='POST').length,1);
  assert.equal(calls.filter(call=>call.url==='/v1/runs'&&call.method==='GET').length,2,'one initial list and one read-only recovery refresh');
  assert.equal(app.messages.length,messagesBefore,'background recovery cannot reset worker/view/cursor');assert.equal(app.element('runs').value,'run-1');assert.equal(app.element('tick').value,'90');
  assert.match(app.element('notice').textContent,/Submission outcome uncertain/);assert.match(app.element('notice').textContent,/will not be resubmitted/);
  if(refreshFails)assert.match(app.element('notice').textContent,/refresh failed: Run list HTTP503.*Use Refresh/);
  else{assert.match(app.element('notice').textContent,/previous selection is absent.*view was retained/);assert.ok(app.element('runs').options.some(option=>option.value==='run-2'));}
 }finally{app.restore();}
});

// Native select replacement resets selection immediately; malformed recovery
// must not reach that mutation before the complete list has been validated.
test('issue60 malformed recovery list preserves native select and existing view',async()=>{
 const calls=[];let submitted=false;
 const snapshot={id:'run-1',state:'completed',sequence:2,config:{seed:42,duration_seconds:1,scenario:'straight',position_sigma:0,velocity_sigma:0,acceleration_density:.1,outage:null},samples:101,error:null,artifacts:['/v1/runs/run-1/artifacts/samples.csv']};
 const app=await adapter('60-malformed',{form:{seed:'42',duration_seconds:'1',scenario:'straight',position_sigma:'0',velocity_sigma:'0',acceleration_density:'0.1'},fetch:async(url,options={})=>{
  calls.push({url,method:options.method??'GET'});if(options.method==='POST'){submitted=true;throw new TypeError('accepted response lost');}
  return {ok:true,text:async()=>JSON.stringify(url==='/v1/runs'?(submitted?[{id:'run-2',state:'running'},{id:'invalid',state:'unknown'}]:[snapshot]):snapshot)};
 }});
 try{
  app.element('runs').value='run-1';await app.element('runs').onchange({target:{value:'run-1'}});
  const reset=app.messages.find(message=>message.type==='resetInput');
  app.port.onmessage({data:{type:'resetAck',generation:reset.generation,controlId:reset.controlId}});
  app.port.onmessage({data:{type:'loaded',generation:reset.generation,rowCount:101,allocatedNumericBytes:28280,points:new Float64Array(40)}});
  app.port.onmessage({data:{type:'frame',generation:reset.generation,row:{tick:90},playing:false}});
  const messages=app.messages.length,options=app.element('runs').options.map(option=>({text:option.textContent,value:option.value}));
  await app.element('configuration').onsubmit({preventDefault(){},currentTarget:{}});
  assert.equal(calls.filter(call=>call.method==='POST').length,1);
  assert.equal(calls.filter(call=>call.url==='/v1/runs'&&call.method==='GET').length,2);
  assert.equal(app.element('runs').value,'run-1');
  assert.deepEqual(app.element('runs').options.map(option=>({text:option.textContent,value:option.value})),options);
  assert.equal(app.element('tick').value,'90');assert.equal(app.messages.length,messages);
  assert.match(app.element('notice').textContent,/Submission outcome uncertain.*refresh failed: Invalid run list entry.*Use Refresh/);
 }finally{app.restore();}
});
