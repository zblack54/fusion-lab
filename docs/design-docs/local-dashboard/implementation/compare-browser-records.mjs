// Compare actual DOM exports with independent raw-CSV field positions. Does not
// import the application's parser; streams source artifacts without buffering.
import {createReadStream} from 'node:fs';
import {readFile,writeFile,stat,mkdir,copyFile} from 'node:fs/promises';
import {createInterface} from 'node:readline';
import assert from 'node:assert/strict';
const evidence='docs/design-docs/local-dashboard/implementation';
const source='results/dashboard-final-review-20261026';
const observations=[];
for(const [run,ticks] of [['run-1',[0,1500,2100]],['run-2',[7777]]]){
 const file=`${source}/${run}/samples.csv`;
 const selected=new Set(ticks);
 for await(const line of createInterface({input:createReadStream(file),crlfDelay:Infinity})){
  if(line.startsWith('tick,'))continue;
  const fields=line.split(','),tick=Number(fields[0]);if(!selected.has(tick))continue;
  const observed=JSON.parse(await readFile(`${evidence}/browser-sample${tick}.json`,'utf8'));
  const flattened=[observed.tick,observed.timeSeconds,...observed.truth,...observed.estimate,...(observed.position??[null,null]),...(observed.velocity??[null,null]),...observed.held,...observed.integrated,observed.outage,...observed.covariance];
  const expected=fields.map((value,index)=>index===18?value==='true':value===''?null:Number(value));
  assert.equal(expected.length,35);assert.deepEqual(flattened,expected);
  observations.push({run,tick,verified_fields:35,result:'pass'});selected.delete(tick);if(!selected.size)break;
 }
 assert.equal(selected.size,0);
}
await mkdir('results/dashboard-recordings',{recursive:true});
await copyFile(`${source}/run-1/samples.csv`,'results/dashboard-recordings/outage30.csv');
await copyFile(`${source}/run-2/samples.csv`,'results/dashboard-recordings/maximum3600.csv');
const report={captured_utc:new Date().toISOString(),method:'Actual browser rendered sample JSON compared against all35 independently indexed raw CSV fields',observations,maximum_artifact_bytes:(await stat(`${source}/run-2/samples.csv`)).size,limitations:['This verifies sample fidelity and maximum-row loading only, not browser RSS/p95/longtask or repeated-abort ownership budgets','Screenshots before issues52/53 narrow repairs; numerical rendering implementation unchanged by those repairs']};
await writeFile(`${evidence}/browser-fidelity.json`,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
