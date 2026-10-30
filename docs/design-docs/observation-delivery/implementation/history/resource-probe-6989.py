#!/usr/bin/env python3
"""Owned native process measurements. Nonproduction evidence; no OS quota claim."""
import argparse,ctypes,hashlib,json,os,pathlib,platform,queue,subprocess,sys,threading,time,tomllib
DRIVER = "use fusion_lab::{Config, DeliveryPlan, ReplayModel, ReplayResult, Trace, replay_delivery, run};\nuse std::{error::Error, hint::black_box, io::{self, Write}};\nfn equal(a: &ReplayResult, b: &ReplayResult) {\n    assert_eq!(a.snapshots().len(), b.snapshots().len());\n    for (a,b) in a.snapshots().iter().zip(b.snapshots()) {\n        assert_eq!(a.tick(),b.tick());\n        let values=|s: &fusion_lab::ReplaySnapshot| s.estimate().position.into_iter().chain(s.estimate().velocity).chain(s.covariance().iter().flatten().copied()).chain(s.held_position()).chain(s.integrated_position()).map(f64::to_bits).collect::<Vec<_>>();\n        assert_eq!(values(a),values(b));\n    }\n    assert_eq!(a.ledger().len(), b.ledger().len());\n    for (a,b) in a.ledger().iter().zip(b.ledger()) {\n        assert_eq!((a.id(),a.sample_tick(),a.planned_arrival_tick(),a.decision_tick(),a.outcome(),a.queue_occupancy()),(b.id(),b.sample_tick(),b.planned_arrival_tick(),b.decision_tick(),b.outcome(),b.queue_occupancy()));\n    }\n    assert_eq!(a.arrival_order(),b.arrival_order());\n}\nfn main() -> Result<(),Box<dyn Error>> {\n    let duration=match std::env::args().nth(1).as_deref() { Some(\"warmup\")=>1,Some(\"maximum\")=>3600,_=>return Err(\"expected warmup or maximum\".into()) };\n    let config=Config { duration_seconds:duration, outage:None, ..Config::default() };\n    let source=run(&config)?;\n    let model=ReplayModel::new(1.0,0.15,0.1)?;\n    let trace=Trace::from_samples(duration,&source.samples,DeliveryPlan::new(500,500,128,None,None)?)?;\n    let result=replay_delivery(&model,&trace)?;\n    let encoded=trace.encode_tsv(&model)?;\n    let ledger=result.encode_ledger_tsv()?;\n    let (decoded_model,decoded)=Trace::decode_tsv(&encoded)?;\n    assert_eq!(trace.duration_seconds(),decoded.duration_seconds());\n    assert_eq!(trace.pending_capacity(),decoded.pending_capacity());\n    for sensor in [fusion_lab::Sensor::Position,fusion_lab::Sensor::Velocity] { assert_eq!(trace.drop_every(sensor),decoded.drop_every(sensor)); }\n    assert_eq!(trace.observations().len(),decoded.observations().len());\n    for (a,b) in trace.observations().iter().zip(decoded.observations()) {\n        assert_eq!((a.id(),a.sample_tick(),a.planned_arrival_tick()),(b.id(),b.sample_tick(),b.planned_arrival_tick()));\n        assert_eq!(a.value().map(f64::to_bits),b.value().map(f64::to_bits));\n    }\n    assert_eq!([model.assumed_position_sigma(),model.assumed_velocity_sigma(),model.acceleration_density()].map(f64::to_bits),[decoded_model.assumed_position_sigma(),decoded_model.assumed_velocity_sigma(),decoded_model.acceleration_density()].map(f64::to_bits));\n    let regenerated=decoded.encode_tsv(&decoded_model)?;\n    assert_eq!(encoded,regenerated);\n    let replayed=replay_delivery(&decoded_model,&decoded)?;\n    equal(&result,&replayed);\n    let regenerated_ledger=replayed.encode_ledger_tsv()?;\n    assert_eq!(ledger,regenerated_ledger);\n    if duration==3600 { assert_eq!(trace.observations().len(),90002); assert_eq!(source.samples.len(),360001); }\n    println!(\"COMPLETE\\t{}\\t{}\\t{}\\t{}\\t{}\",duration,trace.observations().len(),result.snapshots().len(),encoded.len(),ledger.len());\n    io::stdout().flush()?;\n    // Every required allocation remains live across the owner's held-HWM read.\n    black_box((&source,&trace,&result,&encoded,&ledger,&decoded,&replayed,&regenerated,&regenerated_ledger));\n    let mut ack=String::new();\n    io::stdin().read_line(&mut ack)?;\n    if ack!=\"release\\n\" { return Err(\"missing owner release acknowledgement\".into()); }\n    black_box((&source,&trace,&result,&encoded,&ledger,&decoded,&replayed,&regenerated,&regenerated_ledger));\n    Ok(())\n}\n"
BASELINE='e0d3a23f4335193f6442fa7d1b244621b93bbf3f08ee9ba9c4b860c2eb15911b'
MIB=1024*1024

def digest(data): return hashlib.sha256(data).hexdigest()
def inventory(repo):
    paths=[repo/'Cargo.toml',repo/'Cargo.lock',repo/'rust-toolchain.toml']
    paths+=sorted((repo/'src').glob('*.rs'))
    paths+=sorted(p for p in (repo/'tests/observation-delivery').rglob('*') if p.is_file() and not any(x in p.parts for x in ('target','__pycache__','.git')))
    paths += [repo/'docs/design-docs/observation-delivery/validation/test-lock.json']
    return {p.relative_to(repo).as_posix():digest(p.read_bytes()) for p in paths}

def memory_reader(pid):
    if os.name=='nt':
        from ctypes import wintypes as w
        class Counters(ctypes.Structure):
            _fields_=[('cb',w.DWORD),('PageFaultCount',w.DWORD)]+[(n,ctypes.c_size_t) for n in ['PeakWorkingSetSize','WorkingSetSize','QuotaPeakPagedPoolUsage','QuotaPagedPoolUsage','QuotaPeakNonPagedPoolUsage','QuotaNonPagedPoolUsage','PagefileUsage','PeakPagefileUsage']]
        kernel=ctypes.WinDLL('kernel32',use_last_error=True)
        psapi=ctypes.WinDLL('psapi',use_last_error=True)
        kernel.OpenProcess.argtypes=[w.DWORD,w.BOOL,w.DWORD];kernel.OpenProcess.restype=w.HANDLE
        kernel.CloseHandle.argtypes=[w.HANDLE];kernel.CloseHandle.restype=w.BOOL
        psapi.GetProcessMemoryInfo.argtypes=[w.HANDLE,ctypes.POINTER(Counters),w.DWORD];psapi.GetProcessMemoryInfo.restype=w.BOOL
        handle=kernel.OpenProcess(0x410,False,pid)
        if not handle: raise ctypes.WinError(ctypes.get_last_error())
        def read():
            counter=Counters();counter.cb=ctypes.sizeof(counter)
            if not psapi.GetProcessMemoryInfo(handle,ctypes.byref(counter),counter.cb): raise ctypes.WinError(ctypes.get_last_error())
            return dict(lifetime_peak_bytes=counter.PeakWorkingSetSize,current_rss_bytes=counter.WorkingSetSize,method='Windows GetProcessMemoryInfo.PeakWorkingSetSize',units='bytes',owned_pid=pid)
        return read,lambda:kernel.CloseHandle(handle)
    if sys.platform.startswith('linux'):
        def read():
            fields={line.split(':',1)[0]:line.split(':',1)[1].strip() for line in pathlib.Path(f'/proc/{pid}/status').read_text().splitlines() if ':' in line}
            def kb(name):
                value,unit=fields[name].split()
                if unit!='kB': raise ValueError('unexpected Linux memory units')
                return int(value)*1024
            return dict(lifetime_peak_bytes=kb('VmHWM'),current_rss_bytes=kb('VmRSS'),method='Linux /proc/owned-PID/status VmHWM',units='kB multiplied by1024',owned_pid=pid)
        return read,lambda:None
    raise RuntimeError('Only native Windows/Linux memory evidence supported')

def environment():
    ram=None
    if os.name=='nt':
        class Status(ctypes.Structure):
            _fields_=[('length',ctypes.c_ulong),('load',ctypes.c_ulong)]+[(n,ctypes.c_ulonglong) for n in ['total','available','page_total','page_available','virtual_total','virtual_available','extended']]
        status=Status();status.length=ctypes.sizeof(status)
        if not ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status)): raise ctypes.WinError()
        ram=status.total
    elif sys.platform.startswith('linux'): ram=os.sysconf('SC_PHYS_PAGES')*os.sysconf('SC_PAGE_SIZE')
    return dict(os=platform.platform(),machine=platform.machine(),cpu=platform.processor() or os.environ.get('PROCESSOR_IDENTIFIER','unknown'),logical_cpus=os.cpu_count(),physical_ram_bytes=ram,python=sys.version)

OWNER_GUARD=None
OWNER_TOKEN=None

def owner_update(**changes):
    if OWNER_GUARD is None: return
    state=json.loads(OWNER_GUARD.read_text())
    if state.get('token')!=OWNER_TOKEN: raise RuntimeError('sampler ownership guard changed')
    state.update(changes)
    temporary=OWNER_GUARD.with_suffix('.tmp')
    temporary.write_text(json.dumps(state,indent=2)+'\n')
    os.replace(temporary,OWNER_GUARD)

def cleanup_error(record,operation,error):
    record['cleanup_error_count']=record.get('cleanup_error_count',0)+1
    entry=dict(operation=operation,error=repr(error))
    if len(record.setdefault('cleanup_errors',[]))<64: record['cleanup_errors'].append(entry)
    record['latest_cleanup_error']=entry

def reap_owned(p,record,output):
    """Never return or release owned handles while actual child reap is uncertain."""
    marker=output.with_name(output.stem+'.pending.json')
    record['cleanup_owned_pid']=p.pid
    record['reap_confirmed']=False
    cycle=0
    while not record['reap_confirmed']:
        cycle+=1;record['cleanup_cycle']=cycle
        for attempt in range(3):
            record['cleanup_attempt']=attempt+1
            try:
                code=p.poll()
                if isinstance(code,int): record['exit']=code;record['reap_confirmed']=True;break
            except BaseException as error: cleanup_error(record,'poll',error)
            record['kill_attempted']=True
            try: p.kill()
            except BaseException as error: cleanup_error(record,'kill',error)
            try:
                code=p.wait(timeout=1)
                if not isinstance(code,int): raise RuntimeError('wait did not confirm a process exit code')
                record['exit']=code;record['reap_confirmed']=True;break
            except BaseException as error: cleanup_error(record,'wait_reap',error)
        record['cleanup_status']='reaped' if record['reap_confirmed'] else 'owned_pending'
        record['manual_reconciliation_required']=not record['reap_confirmed']
        record['same_child_and_memory_handle_retained']=True
        record['original_error_preserved']=record.get('error')
        record['no_next_workload_until_reaped']=True
        try:
            marker.write_text(json.dumps(record,indent=2)+'\n')
            output.write_text(json.dumps(record,indent=2)+'\n')
            owner_update(status=record['cleanup_status'],child_pid=p.pid,child_argv=record['argv'],pending_receipt=str(marker))
        except BaseException as error:
            cleanup_error(record,'durable_pending_marker',error)
            # The already-exclusive owner guard remains in place; never detach.
        if not record['reap_confirmed']:
            try: time.sleep(5)
            except BaseException as error: cleanup_error(record,'pending_wait_interrupted',error)
    return


def measured(binary,mode,output):
    record=dict(argv=[str(binary),mode],workload=mode,timeout_seconds=60)
    start=time.monotonic();p=None;close=lambda:None;errors=[];line_queue=queue.Queue()
    try:
        p=subprocess.Popen(record['argv'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        record['pid']=p.pid
        owner_update(status='child_running',child_pid=p.pid,child_argv=record['argv'])
        read,close=memory_reader(p.pid)
        def first_line():
            line_queue.put(p.stdout.readline(4097))
        def stderr_reader():
            while True:
                block=p.stderr.read(4096)
                if not block: break
                if sum(map(len,errors))+len(block)>65536: errors.append(b'[stderr capped]');break
                errors.append(block)
        stdout_thread=threading.Thread(target=first_line,daemon=True);stdout_thread.start()
        stderr_thread=threading.Thread(target=stderr_reader,daemon=True);stderr_thread.start()
        line=line_queue.get(timeout=max(0.001,60-(time.monotonic()-start)))
        record['seconds_spawn_to_complete_work']=time.monotonic()-start
        record['complete_line']=line.decode('utf8',errors='replace')
        tokens=line.decode('ascii').rstrip('\n').split('\t')
        if len(line)>4096 or len(tokens)!=6 or tokens[0]!='COMPLETE': raise RuntimeError('missing/malformed complete-work proof')
        values=list(map(int,tokens[1:]));expected_duration=1 if mode=='warmup' else 3600
        if values[0]!=expected_duration or values[2]!=expected_duration*100+1: raise RuntimeError('wrong duration/snapshot inventory')
        if mode=='maximum' and values[1]!=90002: raise RuntimeError('wrong maximum observation inventory')
        if p.poll() is not None: raise RuntimeError('child exited before held-work measurement')
        record['held_complete_work_memory']=read()
        record['retained_phase']='all source/trace/results/encoded+regenerated buffers retained; child waits for owner release'
        p.stdin.write(b'release\n');p.stdin.flush();p.stdin.close()
        record['exit']=p.wait(timeout=max(0.001,60-(time.monotonic()-start)))
        record['reap_confirmed']=True
        stdout_thread.join(timeout=1);stderr_thread.join(timeout=1)
        record['stderr']=b''.join(errors).decode('utf8',errors='replace')
        if os.name=='nt': record['post_exit_retained_handle_memory']=read()
        record['lifetime_peak_bytes']=max(record['held_complete_work_memory']['lifetime_peak_bytes'],record.get('post_exit_retained_handle_memory',{}).get('lifetime_peak_bytes',0))
        record['seconds_total_including_hold_and_teardown']=time.monotonic()-start
        if record['exit']!=0: raise RuntimeError('measurement driver failed after complete-work release')
        record['time_within_30_seconds']=record['seconds_spawn_to_complete_work']<=30
        record['memory_within_512_MiB']=record['lifetime_peak_bytes']<=512*MIB
        record['success']=record['time_within_30_seconds'] and record['memory_within_512_MiB']
    except BaseException as error:
        record['success']=False;record['error']=repr(error)
        if p is not None: reap_owned(p,record,output)
        record['stderr']=b''.join(errors).decode('utf8',errors='replace')
    finally:
        close()
        record['seconds_owner_total']=time.monotonic()-start
        output.write_text(json.dumps(record,indent=2)+'\n')
    return record

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo',type=pathlib.Path,default=pathlib.Path(__file__).resolve().parents[4])
    parser.add_argument('--output',type=pathlib.Path,required=True)
    parser.add_argument('--toolchain',choices=['1.99.0','stable'],required=True)
    parser.add_argument('--diagnostic',action='store_true',help='one maximum run after warmup; explicitly incomplete acceptance')
    args=parser.parse_args();repo=args.repo.resolve();output=args.output.resolve()
    if output==repo or repo in output.parents: raise ValueError('measurement project/output must be external to production repository')
    global OWNER_GUARD,OWNER_TOKEN
    OWNER_GUARD=repo/'docs/design-docs/observation-delivery/implementation/.resource-probe-owner.json'
    OWNER_TOKEN=os.urandom(16).hex()
    # A crashed owner leaves this exclusive marker; never guess ownership away.
    with OWNER_GUARD.open('x') as file:
        json.dump(dict(token=OWNER_TOKEN,owner_pid=os.getpid(),status='owned_preparing',output=str(output),baseline_digest=BASELINE),file)
    output.mkdir(parents=True,exist_ok=False);project=output/'project';(project/'src').mkdir(parents=True)
    before=inventory(repo)
    manifest=(repo/'tests/observation-delivery/Cargo.toml').read_text().replace('observation-delivery-spec-tests','observation-delivery-resource-probe').replace('path = "../.."','path = '+json.dumps(repo.as_posix()))
    lock=(repo/'tests/observation-delivery/Cargo.lock').read_bytes().replace(b'observation-delivery-spec-tests',b'observation-delivery-resource-probe')
    root_entries=[p for p in tomllib.loads((repo/'Cargo.lock').read_text())['package'] if 'source' in p]
    project_entries=[p for p in tomllib.loads(lock.decode())['package'] if 'source' in p]
    if root_entries!=project_entries: raise ValueError('registry closure differs from exact root lock')
    (project/'Cargo.toml').write_text(manifest);(project/'Cargo.lock').write_bytes(lock);(project/'src/main.rs').write_text(DRIVER)
    result=dict(status='DIAGNOSTIC_NOT_ACCEPTANCE' if args.diagnostic else 'MEASUREMENT_CANDIDATE_NOT_ACCEPTED',baseline_authority='issue72comment6527689826',baseline_digest=BASELINE,environment=environment(),source_before=before,sampler_sha256=digest(pathlib.Path(__file__).read_bytes()),driver_sha256=digest(DRIVER.encode()),generated_manifest_sha256=digest(manifest.encode()),generated_lock_sha256=digest(lock),commands=[],runs=[])
    try:
        for argv in [['rustc','+'+args.toolchain,'-Vv'],['cargo','+'+args.toolchain,'build','--release','--locked','--offline','--manifest-path',str(project/'Cargo.toml')]]:
            p=subprocess.run(argv,cwd=repo,capture_output=True,text=True,timeout=240)
            result['commands'].append(dict(argv=argv,exit=p.returncode,stdout=p.stdout,stderr=p.stderr))
            if p.returncode: raise RuntimeError('toolchain/build failed')
        if (project/'Cargo.lock').read_bytes()!=lock: raise RuntimeError('generated exact lock changed')
        binary=project/'target/release'/('observation-delivery-resource-probe.exe' if os.name=='nt' else 'observation-delivery-resource-probe')
        result['binary_sha256']=digest(binary.read_bytes())
        warmup=measured(binary,'warmup',output/'warmup.json');result['warmup']=warmup
        if not warmup['success']: raise RuntimeError('warmup failed')
        for i in range(1 if args.diagnostic else 3):
            run=measured(binary,'maximum',output/f'maximum-{i+1}.json');result['runs'].append(run)
            if not run['success']: raise RuntimeError('maximum run failed; no rerun or omitted failure')
        result['source_after']=inventory(repo)
        if result['source_after']!=before: raise RuntimeError('source changed during measurement')
        result['success']=True
    except BaseException as error:
        result['success']=False;result['error']=repr(error)
    finally:
        (output/'receipt.json').write_text(json.dumps(result,indent=2)+'\n')
        owner_update(status='finished_no_live_child',child_pid=None)
        OWNER_GUARD.unlink()
    print(json.dumps(dict(success=result['success'],status=result['status'],receipt=str(output/'receipt.json'))))
    return 0 if result['success'] else 1
if __name__=='__main__': raise SystemExit(main())
