import {readFile} from 'node:fs/promises';
const data=JSON.parse(await readFile(process.argv[2],'utf8'));
if(process.argv.includes('--new-workers')){console.log(JSON.stringify(data.new_dedicated_workers));process.exit(0);}
for(const row of data.processes)if(row.name.toLowerCase()==='chatgpt.exe'&&row.type==='renderer')console.log(JSON.stringify({pid:row.pid,client:row.renderer_client_id,rss:row.rss_bytes,peak:row.peak_rss_bytes,workers:row.threads?.named_threads.filter(thread=>!thread.description.startsWith('ThreadPool')&&/worker/i.test(thread.description)),thread_query_failures:row.threads?.query_failures}));
