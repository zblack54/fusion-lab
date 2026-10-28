import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const directory='docs/design-docs/local-dashboard/implementation';
const privateDirectory='../ci-evidence/dashboard-native-private';
const pid=50516,parent=73556;
const phases=[];
for(const name of ['before-navigation','after-navigation','after-close']){
 const privateFile=`continuous-${name}.json`,raw=await readFile(`${privateDirectory}/${privateFile}`),data=JSON.parse(raw);
 const owned=data.processes.filter(row=>row.pid===pid);
 phases.push({phase:name,raw_private_file:privateFile,raw_sha256:createHash('sha256').update(raw).digest('hex'),sampled_utc_seconds:data.sampled_utc_seconds,candidate_count:data.processes.length,new_renderer_candidate_count:data.new_renderer_candidates?.length??null,owned_rows:owned,owned_new_dedicated_workers:data.new_dedicated_workers?.filter(row=>row.pid===pid)??[]});
}
assert.equal(phases[0].owned_rows.length,1,'blank/spare renderer existed before navigation');
assert.equal(phases[0].owned_rows[0].threads.named_threads.filter(thread=>thread.description==='DedicatedWorker thread').length,0);
assert.equal(phases[1].owned_rows[0].threads.named_threads.filter(thread=>thread.description==='DedicatedWorker thread').length,1);
assert.equal(phases[1].owned_new_dedicated_workers[0].tid,59728);
assert.equal(phases[2].owned_rows.length,0,'owned renderer must be absent after closing its tab');
const attribution={derived_filter_only:true,owned_renderer_pid:pid,context_parent_pid:parent,private_directory:privateDirectory,phases,note:'The blank/spare renderer already existed before navigation; attribution is the new dedicated worker59728, actual recording allocation, held process identity and PID exit on owned tab close. Unrelated process rows remain private. Browser/GPU shared services are excluded, never described as zero.'};
await writeFile(`${directory}/browser-continuous-attribution.json`,JSON.stringify(attribution,null,2)+'\n');
const json=async name=>JSON.parse(await readFile(`${directory}/${name}`,'utf8'));
const loads=await json('browser-continuous-loads.json'),cycles=await json('browser-continuous-cycles.json'),after=await json('browser-continuous-after-cycles.json'),final=await json('browser-continuous-final-diagnostics.json'),memory=await json('browser-continuous-memory.json');
assert.equal(loads.length,4);assert.equal(cycles.length,50);assert.equal(after.control_timing.samples_total,54);assert.equal(final.control_timing.samples_total,84);assert.equal(memory.pid,pid);assert.equal(memory.process_exited,true);
const controls=after.control_timing.recent_samples.slice(4);assert.equal(controls.length,50);
const whileLoading=controls.filter((_,index)=>cycles[index].prior_status.startsWith('Loading'));
const p95=values=>[...values].sort((a,b)=>a-b)[Math.ceil(values.length*.95)-1];
const summary={status:'measured_with_remaining_acceptance_obligations',method_revision:'continuous-worker-b26f8554',loads:loads.map(load=>({index:load.index,warmup:load.warmup,max_active_ms:load.diagnostics.worker.max_active_work_before_yield_ms,max_parsed_rows_between_yields:load.diagnostics.worker.max_rows_between_yields,allocated_numeric_bytes:load.diagnostics.numeric_buffers_bytes})),selection_reset_count:cycles.length,explicit_prior_loading_count:whileLoading.length,loading_control_p95_next_raf_proxy_ms:p95(whileLoading.map(control=>control.input_to_next_raf_proxy_ms)),loading_control_max_next_raf_proxy_ms:Math.max(...whileLoading.map(control=>control.input_to_next_raf_proxy_ms)),final_retained_control_p95_next_raf_proxy_ms:final.control_timing.p95_input_to_next_raf_proxy_ms,supported_main_thread_long_tasks:final.main_thread_long_tasks,lifetime_renderer_and_worker_rss_peak_bytes:memory.lifetime_rss_peak_bytes,native_samples:memory.samples,native_elapsed_seconds:memory.elapsed_seconds,native_max_sample_gap_seconds:memory.sampling_interval_max_seconds,native_sampling_errors:memory.errors,owned_renderer_exited:memory.process_exited,limitations:['NextRAF is a presentation proxy, not physical display proof.','Continuous timing subtracts only actual suspension of the load continuation; constant-size control callbacks are separately measured.','Prior snippet-only4.9ms evidence is preserved and cannot establish the whole-worker limit.','Native Windows/IAB reference observation is not a universal OS memory cap or Linux browser measurement.','Remaining lifecycle-failure/accessibility/browser and exact-head CI acceptance remains open.']};
assert.ok(summary.loads.every(load=>load.max_active_ms<=8&&load.max_parsed_rows_between_yields<=2048&&load.allocated_numeric_bytes===100800280));assert.ok(summary.lifetime_renderer_and_worker_rss_peak_bytes<512*1024*1024);assert.ok(summary.loading_control_p95_next_raf_proxy_ms<=250);
await writeFile(`${directory}/browser-continuous-summary.json`,JSON.stringify(summary,null,2)+'\n');console.log(JSON.stringify(summary,null,2));
