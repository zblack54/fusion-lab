// Preserve original inventories privately; publish only explicit derived scope.
import {readFile,writeFile,readdir,mkdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
const directory='docs/design-docs/local-dashboard/implementation';
const ownedPid=Number(process.argv[2]??94864),parentPid=Number(process.argv[3]??73556),prefix=process.argv[4]??'browser-';
const privateDirectory=path.resolve('../ci-evidence/dashboard-native-private');await mkdir(privateDirectory,{recursive:true});
let provenance=[];try{provenance=JSON.parse(await readFile(path.join(directory,'process-evidence-provenance.json'),'utf8')).files;}catch{}
for(const name of await readdir(directory)){
 if(!name.startsWith(prefix)||!name.endsWith('.json'))continue;
 const file=path.join(directory,name),bytes=await readFile(file);let original;try{original=JSON.parse(bytes);}catch{continue;}
 if(!Array.isArray(original.processes))continue;
 const sha256=createHash('sha256').update(bytes).digest('hex'),privateName=`${sha256}-${name}`;
 await writeFile(path.join(privateDirectory,privateName),bytes);
 const scoped={derived_filter_only:true,original_sha256:sha256,original_private_file:privateName,sampled_utc_seconds:original.sampled_utc_seconds,original_candidate_count:original.processes.length,original_new_renderer_count:original.new_renderer_candidates?.length??null,owned_renderer_pid:ownedPid,context_parent_pid:parentPid,owned_renderer_rows:original.processes.filter(row=>row.pid===ownedPid),context_parent:original.processes.filter(row=>row.pid===parentPid).map(row=>({pid:row.pid,parent:row.parent,name:row.name,type:row.type})),owned_new_dedicated_workers:original.new_dedicated_workers?.filter(row=>row.pid===ownedPid)??[],scope_note:'Only the attributed final renderer is retained. Parent is identity context, not memory charged. Early snapshots can show this PID as a spare before it acquires the dedicated worker; the final create/load/close chain is required. Full originals remain privately available by SHA. No sampler measurement is modified.'};
 await writeFile(file,JSON.stringify(scoped,null,2)+'\n');if(!provenance.some(item=>item.original_sha256===sha256&&item.file===name))provenance.push({file:name,original_sha256:sha256,private_file:privateName});
}
await writeFile(path.join(directory,'process-evidence-provenance.json'),JSON.stringify({private_directory:privateDirectory,files:provenance},null,2)+'\n');console.log(JSON.stringify({filtered_files:provenance.length,private_directory:privateDirectory}));
