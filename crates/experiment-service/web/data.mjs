// Pure recorded-data boundaries: no browser state or numerical estimator work.
const COLUMNS='tick,time_s,true_x,true_y,true_vx,true_vy,estimated_x,estimated_y,estimated_vx,estimated_vy,position_x,position_y,velocity_x,velocity_y,hold_x,hold_y,integrated_x,integrated_y,position_outage,p00,p01,p02,p03,p10,p11,p12,p13,p20,p21,p22,p23,p30,p31,p32,p33';
const NUM=/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/;
const U64=18446744073709551615n;
const WIRE_NUMBER=Symbol('parsed JSON number');
function invalid(message){const error=new Error(message);error.code='invalid_recording';return error;}
function unsigned(value,name){if(typeof value!=='string'||!/^(?:0|[1-9]\d*)$/.test(value)||BigInt(value)>U64)throw invalid(`invalid ${name} integer range`);return value;}
function number(value){if(!NUM.test(value)||!Number.isFinite(Number(value)))throw invalid('invalid finite number');return Number(value);}
class Recording {
 #chunks;
 constructor(chunks,count){this.#chunks=chunks;this.rowCount=count;this.allocatedNumericBytes=chunks.reduce((sum,chunk)=>sum+chunk.byteLength,0);}
 row(index){if(!Number.isInteger(index)||index<0||index>=this.rowCount)throw new RangeError('row index');const chunk=this.#chunks[Math.floor(index/1024)];const start=(index%1024)*35;const values=chunk.subarray(start,start+35);const array=(from,to)=>Array.from(values.subarray(from,to));return {tick:values[0],timeSeconds:values[1],truth:array(2,6),estimate:array(6,10),position:Number.isNaN(values[10])?null:array(10,12),velocity:Number.isNaN(values[12])?null:array(12,14),held:array(14,16),integrated:array(16,18),outage:values[18]===1,covariance:array(19,35)};}
}
export class CsvRecordingParser {
 #decoder=new TextDecoder('utf-8',{fatal:true}); #carry=''; #lineBytes=0; #bytes=0; #header=false; #done=false; #failed=false; #chunks=[]; #limits;
 constructor(options={}){const maxima={maxRows:360001,maxBytes:268435456,maxLineBytes:8192};this.#limits={...maxima};for(const key of Object.keys(options)){if(!(key in maxima)||!Number.isInteger(options[key])||options[key]<1||options[key]>maxima[key])throw new RangeError('maximum parser limit');this.#limits[key]=options[key];}this.parsedRows=0;}
 get allocatedNumericBytes(){return this.#chunks.reduce((sum,chunk)=>sum+chunk.byteLength,0);}
 #line(line){if(line.endsWith('\r'))line=line.slice(0,-1);if(!this.#header){if(line!==COLUMNS)throw invalid('invalid CSV schema');this.#header=true;return;}if(this.parsedRows>=this.#limits.maxRows)throw invalid('row limit exceeded');const fields=line.split(',');if(fields.length!==35)throw invalid('invalid column count');const values=new Float64Array(35);for(let i=0;i<35;i++){if(i===18){if(fields[i]!=='true'&&fields[i]!=='false')throw invalid('invalid boolean');values[i]=fields[i]==='true'?1:0;}else if(i>=10&&i<=13&&fields[i]===''){values[i]=NaN;}else values[i]=number(fields[i]);}for(const i of [10,12])if(Number.isNaN(values[i])!==Number.isNaN(values[i+1]))throw invalid('partial measurement pair');if(values[0]!==this.parsedRows||values[1]!==values[0]/100)throw invalid('invalid tick or time');const slot=this.parsedRows%1024;if(slot===0)this.#chunks.push(new Float64Array(Math.min(1024,this.#limits.maxRows-this.parsedRows)*35));this.#chunks.at(-1).set(values,slot*35);this.parsedRows++;}
 write(bytes){if(this.#done||this.#failed)throw invalid('parser finalized or failed');if(!(bytes instanceof Uint8Array))throw invalid('invalid byte input');try{this.#bytes+=bytes.byteLength;if(this.#bytes>this.#limits.maxBytes)throw invalid('byte limit exceeded');for(const byte of bytes){if(byte===10)this.#lineBytes=0;else if(++this.#lineBytes>this.#limits.maxLineBytes)throw invalid('line limit exceeded');}const text=this.#decoder.decode(bytes,{stream:true});let start=0;for(let end=text.indexOf('\n');end!==-1;end=text.indexOf('\n',start)){this.#line(this.#carry+text.slice(start,end));this.#carry='';start=end+1;}this.#carry+=text.slice(start);}catch(error){this.#failed=true;this.#chunks=[];this.#carry='';throw error;}}
 finish(){if(this.#done||this.#failed)throw invalid('parser finalized or failed');try{this.#carry+=this.#decoder.decode();if(this.#carry!=='')this.#line(this.#carry);if(!this.#header||this.parsedRows===0)throw invalid('incomplete recording');this.#carry='';this.#done=true;const result=new Recording(this.#chunks,this.parsedRows);this.#chunks=[];return result;}catch(error){this.#failed=true;this.#chunks=[];throw error;}}
}
export function rowErrors(row){const distance=point=>Math.hypot(point[0]-row.truth[0],point[1]-row.truth[1]);return {filter:distance(row.estimate),hold:distance(row.held),integration:distance(row.integrated)};}
function productError(a,b,product){
 const splitter=134217729,ca=splitter*a,cb=splitter*b;
 const ah=ca-(ca-a),al=a-ah,bh=cb-(cb-b),bl=b-bh;
 return ((ah*bh-product)+ah*bl+al*bh)+al*bl;
}
export function positionEllipse(row){
 const a=row.covariance[0],d=row.covariance[5],left=row.covariance[1],right=row.covariance[4];
 if(![a,d,left,right].every(Number.isFinite))return null;
 const sum=left+right,b=Number.isFinite(sum)?sum/2:left/2+right/2;
 const tolerance=1e-9*Math.max(1,Math.abs(a),Math.abs(d));
 if(b===0){if(Math.min(a,d)<-tolerance)return null;return {major:Math.sqrt(Math.max(0,a,d))*Math.sqrt(5.991464547107979),minor:Math.sqrt(Math.max(0,Math.min(a,d)))*Math.sqrt(5.991464547107979),angle:a>=d?0:Math.PI/2};}
 // Power-of-two scaling preserves represented bits. The compensated determinant
 // avoids losing the small eigenvalue to trace-minus-discriminant cancellation.
 const magnitude=Math.max(Math.abs(a),Math.abs(b),Math.abs(d));
 const scale=2**Math.min(1023,Math.max(-1022,Math.floor(Math.log2(magnitude))));
 const x=a/scale,z=d/scale,y=b/scale;
 const first=x*z,second=y*y;
 const determinant=(first-second)+(productError(x,z,first)-productError(y,y,second));
 const major=(x/2+z/2)+Math.hypot((x-z)/2,y);
 const minor=major!==0?determinant/major:(x/2+z/2)-Math.hypot((x-z)/2,y);
 if(minor < -tolerance/scale||major < -tolerance/scale)return null;
 const factor=Math.sqrt(scale)*Math.sqrt(5.991464547107979);
 const result={major:Math.sqrt(Math.max(0,major))*factor,minor:Math.sqrt(Math.max(0,minor))*factor,angle:Math.atan2(2*y,x-z)/2};
 return Object.values(result).every(Number.isFinite)?result:null;
}
export function selectPlotRows(recording,maxPoints=4096){if(!Number.isInteger(maxPoints)||maxPoints<2)throw new RangeError('invalid maxPoints');const count=Math.min(recording.rowCount,maxPoints,4096);return Array.from({length:count},(_,index)=>count===1?0:Math.floor(index*(recording.rowCount-1)/(count-1)));}
export class Replay {
 #recording; #cursor=0; #anchor=0; #tick=0; #speed=1; playing=false;
 constructor(recording){if(!recording||recording.rowCount<1)throw new RangeError('empty recording');this.#recording=recording;}
 #position(nowMs){return this.playing?Math.min(this.#recording.rowCount-1,this.#tick+Math.max(0,nowMs-this.#anchor)*this.#speed/10):this.#tick;}
 frame(nowMs){const position=this.#position(nowMs);this.#cursor=Math.floor(position);if(this.#cursor===this.#recording.rowCount-1){this.#tick=position;this.playing=false;}return this.#recording.row(this.#cursor);}
 play(nowMs){this.#anchor=nowMs;this.playing=true;}
 pause(nowMs){const position=this.#position(nowMs);this.frame(nowMs);this.#tick=position;this.playing=false;}
 seek(index){if(!Number.isInteger(index)||index<0||index>=this.#recording.rowCount)throw new RangeError('seek index');this.#cursor=index;this.#tick=index;this.playing=false;}
 reset(){this.#cursor=0;this.#tick=0;this.playing=false;}
 setSpeed(rate,nowMs){if(![.25,1,4].includes(rate))throw new RangeError('playback speed');const position=this.#position(nowMs);this.frame(nowMs);this.#tick=position;this.#anchor=nowMs;this.#speed=rate;}
}

// Preserve number tokens before Number conversion, including inside nested
// objects. Duplicate keys are rejected instead of silently taking the last.
function wireJson(text) {
 if(typeof text!=='string'||text.length>65536)throw invalid('invalid snapshot size');
 let at=0;
 const space=()=>{while(at<text.length&&/[\t\n\r ]/.test(text[at]))at++;};
 function string(){const start=at++;while(at<text.length){if(text[at]==='\\'){at+=2;continue;}if(text[at++]==='"')return JSON.parse(text.slice(start,at));}throw invalid('invalid JSON string');}
 function value(depth=0){
  if(depth>32)throw invalid('JSON depth limit');space();const c=text[at];
  if(c==='"')return string();
  if(c==='{'){
   at++;const result=Object.create(null);space();if(text[at]==='}'){at++;return result;}
   while(true){space();if(text[at]!=='"')throw invalid('invalid JSON key');const key=string();if(Object.hasOwn(result,key))throw invalid('duplicate JSON key');space();if(text[at++]!==':')throw invalid('invalid JSON colon');result[key]=value(depth+1);space();const next=text[at++];if(next==='}')return result;if(next!==',')throw invalid('invalid JSON object');}
  }
  if(c==='['){at++;const result=[];space();if(text[at]===']'){at++;return result;}while(true){result.push(value(depth+1));space();const next=text[at++];if(next===']')return result;if(next!==',')throw invalid('invalid JSON array');}}
  for(const [token,result]of [['true',true],['false',false],['null',null]])if(text.startsWith(token,at)){at+=token.length;return result;}
  const match=/^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/.exec(text.slice(at));if(!match)throw invalid('invalid JSON value');at+=match[0].length;return {[WIRE_NUMBER]:match[0]};
 }
 const result=value();space();if(at!==text.length)throw invalid('invalid JSON trailing input');return result;
}
function numeric(token){if(!token||typeof token[WIRE_NUMBER]!=='string')throw invalid('invalid numeric token');return number(token[WIRE_NUMBER]);}
function wireUnsigned(token,name){return unsigned(token?.[WIRE_NUMBER],name);}
function configChecked(config){
 if(!config||typeof config!=='object'||Array.isArray(config))throw invalid('invalid config');
 const keys=['seed','duration_seconds','scenario','position_sigma','velocity_sigma','acceleration_density','outage'];
 if(Object.keys(config).some(key=>!keys.includes(key))||keys.some(key=>!Object.hasOwn(config,key)))throw invalid('invalid config fields');
 unsigned(config.seed,'seed');
 if(!Number.isInteger(config.duration_seconds)||config.duration_seconds<1||config.duration_seconds>3600||!['straight','maneuver'].includes(config.scenario))throw invalid('invalid duration or scenario');
 for(const name of ['position_sigma','velocity_sigma','acceleration_density'])if(typeof config[name]!=='number'||!Number.isFinite(config[name])||config[name]<0||config[name]>100)throw invalid(`invalid ${name}`);
 if(config.outage!==null){const outage=config.outage;if(!outage||Object.keys(outage).sort().join(',')!=='end,start'||!Number.isInteger(outage.start)||!Number.isInteger(outage.end)||outage.start<0||outage.start>=outage.end||outage.end>config.duration_seconds)throw invalid('invalid outage');}
 return config;
}
export function serializeConfig(config){configChecked(config);const copy={...config,seed:undefined};return '{"seed":'+config.seed+','+JSON.stringify(copy).slice(1);}
export function parseSnapshot(text){
 const raw=wireJson(text);
 if(!raw||typeof raw!=='object'||typeof raw.id!=='string'||!/^run-[1-9]\d*$/.test(raw.id)||!['running','completed','failed','cancelled'].includes(raw.state))throw invalid('invalid snapshot id or state');
 const config=raw.config;if(!config||typeof config!=='object')throw invalid('invalid config');
 const decoded={...config,seed:wireUnsigned(config.seed,'seed')};
 for(const key of ['duration_seconds','position_sigma','velocity_sigma','acceleration_density'])decoded[key]=numeric(config[key]);
 if(config.outage!==null){if(!config.outage||typeof config.outage!=='object')throw invalid('invalid outage');decoded.outage={...config.outage,start:numeric(config.outage.start),end:numeric(config.outage.end)};}
 configChecked(decoded);
 const sequence=wireUnsigned(raw.sequence,'sequence');
 const samples=raw.samples===null?null:numeric(raw.samples);
 if(samples!==null&&(!Number.isInteger(samples)||samples<0||samples>360001))throw invalid('invalid sample count');
 if(raw.error!==null&&(!raw.error||typeof raw.error.code!=='string'||typeof raw.error.message!=='string'))throw invalid('invalid snapshot error');
 if(!Array.isArray(raw.artifacts)||raw.artifacts.length>2||raw.artifacts.some(path=>typeof path!=='string'||!['samples.csv','report.txt'].some(name=>path===`/v1/runs/${raw.id}/artifacts/${name}`)))throw invalid('invalid artifact URL');
 return {id:raw.id,state:raw.state,sequence,config:decoded,samples,error:raw.error,artifacts:raw.artifacts};
}
