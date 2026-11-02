#!/usr/bin/env python3
"""Issue75 fixed paired research series; no new product resource budget."""
import argparse,hashlib,importlib.util,json,os,pathlib,queue,subprocess,sys,threading,time,tomllib
AUTHORITY='https://github.com/zblack54/fusion-lab/issues/75#issuecomment-6534442577'
OWNER_SHA='ab851ab12ec6f54ee52baec37b0b59b80f8db069b5a92179c3f12f23cc9b924c'
ORDER=('delayed','accepted','accepted','delayed','delayed','accepted')
def digest(data): return hashlib.sha256(data).hexdigest()
def run_case(owner,binary,case,output):
    record=dict(argv=[str(binary),case],profile=case,watchdog_seconds=60,no_product_time_or_memory_pass_budget=True)
    start=time.monotonic();p=None;close=lambda:None;errors=[];lines=queue.Queue()
    try:
        p=subprocess.Popen(record['argv'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        record['pid']=p.pid;owner.owner_update(status='child_running',child_pid=p.pid,child_argv=record['argv'])
        read,close=owner.memory_reader(p.pid)
        def stdout_reader(): lines.put(p.stdout.readline(4097))
        def stderr_reader():
            while True:
                block=p.stderr.read(4096)
                if not block: break
                if sum(map(len,errors))+len(block)>65536: errors.append(b'[stderr capped]');break
                errors.append(block)
        a=threading.Thread(target=stdout_reader,daemon=True);b=threading.Thread(target=stderr_reader,daemon=True);a.start();b.start()
        line=lines.get(timeout=max(.001,60-(time.monotonic()-start)))
        record['seconds_spawn_to_complete_work']=time.monotonic()-start
        record['complete_line']=line.decode('utf8',errors='replace')
        fields=line.decode('ascii').rstrip('\n').split('\t')
        if len(line)>4096 or len(fields)!=12 or fields[:2]!=['COMPLETE',case]: raise RuntimeError('missing/malformed profile equality/count proof')
        values=list(map(int,fields[2:]));duration,offered,snapshots,accepted,stale,horizon,full,lost,trace_bytes,ledger_bytes=values
        expected={'warmup-delayed':(1,27,101,0,0,27,0,0),'warmup-accepted':(1,27,101,27,0,0,0,0),'delayed':(3600,90002,360001,0,89877,125,0,0),'accepted':(3600,90002,360001,90002,0,0,0,0)}[case]
        if tuple(values[:8])!=expected: raise RuntimeError('actual profile inventory/dispositions disagree with predeclared protocol')
        record['actual_counts']=dict(duration_seconds=duration,offered=offered,snapshots=snapshots,accepted=accepted,stale=stale,horizon=horizon,queue_full=full,scripted_loss=lost,trace_bytes=trace_bytes,ledger_bytes=ledger_bytes)
        if p.poll() is not None: raise RuntimeError('child exited before retained-work observation')
        record['held_complete_work_memory']=read()
        record['retention']='source Experiment, both traces/results and encoded/regenerated buffers live across owner acknowledgement'
        p.stdin.write(b'release\n');p.stdin.flush();p.stdin.close()
        record['exit']=p.wait(timeout=max(.001,60-(time.monotonic()-start)));record['reap_confirmed']=True
        a.join(timeout=1);b.join(timeout=1)
        record['post_exit_retained_handle_memory']=read()
        record['lifetime_peak_bytes']=max(record['held_complete_work_memory']['lifetime_peak_bytes'],record['post_exit_retained_handle_memory']['lifetime_peak_bytes'])
        if record['exit']!=0: raise RuntimeError('driver failed after owner release')
        record['success']=True
    except BaseException as error:
        record['success']=False;record['error']=repr(error)
        if p is not None: owner.reap_owned(p,record,output)
    finally:
        close();record['stderr']=b''.join(errors).decode('utf8',errors='replace');record['seconds_owner_total_including_hold_and_teardown']=time.monotonic()-start
        output.write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8',newline='\n')
    return record

def save_record(path,record):
    temporary=path.with_name(path.name+'.tmp')
    with temporary.open('w',encoding='utf-8',newline='\n') as file:
        json.dump(record,file,indent=2);file.write('\n');file.flush();os.fsync(file.fileno())
    os.replace(temporary,path)

def run_command(owner,argv,cwd,output,record=None,timeout=240):
    """Own/reap the direct command child; do not assert descendant-tree cleanup."""
    if record is None: record={}
    record.update(argv=list(argv),cwd=str(cwd),timeout_seconds=timeout,scope='direct Popen child only; compiler descendants are not process-tree contained',process_tree_cleanup_guaranteed=False)
    start=time.monotonic();p=None
    stdout_path=output.with_name(output.stem+'.stdout.log');stderr_path=output.with_name(output.stem+'.stderr.log')
    try:
        with stdout_path.open('wb') as stdout,stderr_path.open('wb') as stderr:
            p=subprocess.Popen(argv,cwd=cwd,stdout=stdout,stderr=stderr)
            record['pid']=p.pid;record['status']='owned_command_running'
            owner.owner_update(status='command_running',child_pid=p.pid,child_argv=list(argv),command_receipt=str(output))
            save_record(output,record)
            record['exit']=p.wait(timeout=timeout);record['reap_confirmed']=True
            if record['exit']!=0: raise RuntimeError('command returned nonzero exit')
            record['success']=True;record['status']='direct_command_reaped_success'
    except BaseException as error:
        record['success']=False;record['error']=repr(error);record['status']='command_failed_stop_before_series'
        if p is not None:
            # Same child and guard remain owned until actual reap; never return
            # from exceptional cleanup on merely attempted kill/wait.
            owner.reap_owned(p,record,output)
            record['descendant_reconciliation_required']=True
            record['descendant_limit']='Direct child reaped, but compiler descendants are not contained. Keep guard and manually reconcile before any new build or numerical series.'
        else:
            record['spawn_failed_before_child']=True;record['reap_confirmed']=True
    finally:
        record['seconds_total']=time.monotonic()-start
        for name,path in [('stdout',stdout_path),('stderr',stderr_path)]:
            if path.exists():
                with path.open('rb') as file: prefix=file.read(65537)
                record[name]=prefix[:65536].decode('utf8',errors='replace');record[name+'_prefix_truncated']=len(prefix)>65536
                record[name+'_path']=str(path);record[name+'_observed_bytes']=path.stat().st_size
        save_record(output,record)
    return record


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo',type=pathlib.Path,default=pathlib.Path(__file__).resolve().parents[2])
    parser.add_argument('--output',type=pathlib.Path,required=True)
    parser.add_argument('--run-approved-series',action='store_true',help='main must independently review this code before authorizing the fixed six-run series')
    args=parser.parse_args()
    if not args.run_approved_series: raise ValueError('code checkpoint only; explicit main authorization/run flag required')
    if os.name!='nt': raise RuntimeError('first approved scope is native Windows Rust1.99.0 only')
    repo=args.repo.resolve();output=args.output.resolve();research=repo/'research/delivery-performance'
    if output==repo or repo in output.parents: raise ValueError('owned output/project must be external to repository')
    helper=repo/'docs/design-docs/observation-delivery/implementation/resource_probe.py'
    if digest(helper.read_bytes())!=OWNER_SHA: raise RuntimeError('reviewed accepted ownership helper bytes differ')
    spec=importlib.util.spec_from_file_location('accepted_delivery_owner',helper);owner=importlib.util.module_from_spec(spec);spec.loader.exec_module(owner)
    source_before=owner.inventory(repo)
    research_before={name:digest((research/name).read_bytes()) for name in ('protocol.md','paired_probe.py','driver.rs')}
    output.mkdir(parents=True,exist_ok=False);project=output/'project';(project/'src').mkdir(parents=True)
    manifest=(repo/'tests/observation-delivery/Cargo.toml').read_text().replace('observation-delivery-spec-tests','delivery-performance-paired-probe').replace('path = "../.."','path = '+json.dumps(repo.as_posix()))
    lock=(repo/'tests/observation-delivery/Cargo.lock').read_bytes().replace(b'observation-delivery-spec-tests',b'delivery-performance-paired-probe')
    root_registry=[p for p in tomllib.loads((repo/'Cargo.lock').read_text())['package'] if 'source' in p]
    generated_registry=[p for p in tomllib.loads(lock.decode())['package'] if 'source' in p]
    if root_registry!=generated_registry: raise RuntimeError('exact root registry dependency closure mismatch')
    (project/'Cargo.toml').write_text(manifest,encoding='utf-8',newline='\n');(project/'Cargo.lock').write_bytes(lock);(project/'src/main.rs').write_bytes((research/'driver.rs').read_bytes())
    def generated(): return {name:dict(sha256=digest((project/name).read_bytes()),bytes=(project/name).stat().st_size) for name in ('Cargo.toml','Cargo.lock','src/main.rs')}
    generated_before=generated()
    result=dict(status='RESEARCH_CANDIDATE_NOT_ACCEPTED',authority=AUTHORITY,protocol_sha256=research_before['protocol.md'],research_before=research_before,source_before=source_before,owner_helper_sha256=OWNER_SHA,environment=owner.environment(),generated_before_build=generated_before,order=list(ORDER),commands=[],warmups=[],runs=[],budgets='No new30s/512MiB pass criterion;60s owned watchdog only')
    owner.OWNER_GUARD=repo/'docs/design-docs/observation-delivery/implementation/.resource-probe-owner.json';owner.OWNER_TOKEN=os.urandom(16).hex()
    with owner.OWNER_GUARD.open('x') as file: json.dump(dict(token=owner.OWNER_TOKEN,owner_pid=os.getpid(),status='research_owned_preparing',output=str(output),authority=AUTHORITY),file)
    try:
        for index,argv in enumerate((['rustc','+1.99.0','-Vv'],['cargo','+1.99.0','build','--release','--locked','--offline','--manifest-path',str(project/'Cargo.toml')])):
            command={};result['commands'].append(command)
            run_command(owner,argv,repo,output/f'command-{index+1}.json',record=command)
            if not command['success']: raise RuntimeError('fixed toolchain/build failed; no numerical series authorized after command failure')
        result['generated_after_build']=generated()
        if result['generated_after_build']!=generated_before: raise RuntimeError('generated actual input bytes changed during build')
        binary=project/'target/release/delivery-performance-paired-probe.exe';result['binary_sha256']=digest(binary.read_bytes())
        for case in ('warmup-delayed','warmup-accepted'):
            record=run_case(owner,binary,case,output/(case+'.json'));result['warmups'].append(record)
            if not record['success']: raise RuntimeError('warmup failed; stop without retry')
        for i,case in enumerate(ORDER):
            record=run_case(owner,binary,case,output/f'run-{i+1}-{case}.json');result['runs'].append(record)
            if not record['success']: raise RuntimeError('paired process failed; stop without retry/omission')
        result['pairs']=[dict(pair=i+1,run_indices=[2*i+1,2*i+2],profiles=list(ORDER[2*i:2*i+2])) for i in range(3)]
        result['generated_after_series']=generated();result['source_after']=owner.inventory(repo);result['research_after']={name:digest((research/name).read_bytes()) for name in research_before}
        if result['generated_after_series']!=generated_before or result['source_after']!=source_before or result['research_after']!=research_before: raise RuntimeError('frozen generated/root/research bytes changed')
        if digest(binary.read_bytes())!=result['binary_sha256']: raise RuntimeError('binary changed during paired series')
        result['success']=True
    except BaseException as error: result['success']=False;result['error']=repr(error)
    finally:
        (output/'receipt.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8',newline='\n')
        direct_children_reaped=all(c.get('reap_confirmed') for c in result['commands']) and all(r.get('reap_confirmed') for r in result['warmups']+result['runs'])
        descendants_unverified=any(c.get('descendant_reconciliation_required') for c in result['commands'])
        if direct_children_reaped and not descendants_unverified:
            owner.owner_update(status='finished_direct_children_reaped',child_pid=None,process_tree_cleanup_guaranteed=False);owner.OWNER_GUARD.unlink()
        else:
            owner.owner_update(status='reconciliation_required_no_next_series',direct_children_reaped=direct_children_reaped,compiler_descendants_unverified=descendants_unverified)
            result['guard_retained_for_reconciliation']=True
            (output/'receipt.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8',newline='\n')
    print(json.dumps(dict(success=result['success'],status=result['status'],receipt=str(output/'receipt.json'))))
    return 0 if result['success'] else 1
if __name__=='__main__': raise SystemExit(main())
