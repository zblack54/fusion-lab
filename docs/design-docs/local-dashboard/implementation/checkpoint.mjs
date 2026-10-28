// Reproducible source provenance; this is evidence tooling, not app behavior.
import {readdir,readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import path from 'node:path';
const root=process.cwd();
const git=(...args)=>execFileSync('git',['-c',`safe.directory=${root.replaceAll('\\','/')}`,...args],{encoding:'utf8'}).trim();
const files=[];
async function inventory(directory){for(const entry of await readdir(directory,{withFileTypes:true})){const name=`${directory}/${entry.name}`;if(entry.isDirectory())await inventory(name);else if(entry.isFile())files.push(name);}}
for(const directory of ['crates/experiment-service/src','crates/experiment-service/web'])await inventory(directory);
files.push('crates/experiment-service/Cargo.toml','crates/experiment-service/Cargo.lock');
const hashes={};for(const file of files.sort())hashes[file]=createHash('sha256').update(await readFile(file)).digest('hex');
const report={captured_utc:new Date().toISOString(),platform:process.platform,node:process.version,revision:git('rev-parse','HEAD'),dirty_worktree:git('status','--porcelain').length!==0,source_sha256:hashes,acceptance:'source inventory only; final acceptance is separately recorded in acceptance.md'};
const output=path.join(root,'docs/design-docs/local-dashboard/implementation/source-checkpoint.json');await writeFile(output,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
