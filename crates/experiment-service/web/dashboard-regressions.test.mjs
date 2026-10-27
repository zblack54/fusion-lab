// Production-owned adapter regressions. These execute the real module with
// controlled browser ports; native-browser acceptance is separately required.
import test from 'node:test';
import assert from 'node:assert/strict';

async function adapter(caseId){
 const elements=new Map();
 const element=id=>{if(!elements.has(id))elements.set(id,{value:id==='speed'?'4':'0',textContent:'',disabled:false,options:[],replaceChildren(){this.options=[];},add(option){this.options.push(option);}});return elements.get(id);};
 const messages=[];let port;
 const keys=['document','window','Worker','ResizeObserver','requestAnimationFrame','fetch','Option'];
 const saved=new Map(keys.map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]));
 globalThis.document={getElementById:element};globalThis.window={addEventListener(){}};
 globalThis.Worker=class{constructor(){port=this;}postMessage(message){messages.push(message);}};
 globalThis.ResizeObserver=class{observe(){}};globalThis.requestAnimationFrame=()=>0;
 globalThis.Option=class{constructor(text,value){this.textContent=text;this.value=value;}};
 globalThis.fetch=async url=>({ok:true,text:async()=>url==='/v1/runs'?'[]':JSON.stringify({id:'run-2',state:'completed',sequence:2,config:{seed:42,duration_seconds:1,scenario:'straight',position_sigma:0,velocity_sigma:0,acceleration_density:.1,outage:null},samples:101,error:null,artifacts:['/v1/runs/run-2/artifacts/samples.csv']})});
 try{await import(`./dashboard.mjs?regression=${caseId}`);await new Promise(resolve=>setImmediate(resolve));}
 catch(error){for(const [key,descriptor]of saved)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else delete globalThis[key];throw error;}
 return {element,messages,port,restore(){for(const [key,descriptor]of saved)if(descriptor)Object.defineProperty(globalThis,key,descriptor);else delete globalThis[key];}};
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
