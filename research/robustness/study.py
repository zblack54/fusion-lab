"""Fixed inventory executor and independent standard-library verifier for issue21."""
import argparse
import csv
import hashlib
import itertools
import json
import math
import os
import platform
import sys
from pathlib import Path
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
THRESHOLD = 5.991464547107979
SCALES = (0.5, 1.0, 2.0)
BIASES = ((0.0, 0.0), (0.3, -0.15), (1.0, -0.5))
KEYS = list(itertools.product(range(1000, 1032), ('straight', 'maneuver'), range(3), range(3)))
COLUMNS = ('tick,time_s,true_x,true_y,true_vx,true_vy,estimated_x,estimated_y,estimated_vx,estimated_vy,position_x,position_y,velocity_x,velocity_y,hold_x,hold_y,integrated_x,integrated_y,position_outage,' + ','.join('p%d%d' % (r,c) for r in range(4) for c in range(4))).split(',')
EXTRA = ['source_covered', 'source_within_recovery']
SOURCE_PATHS = ['Cargo.toml','Cargo.lock','src/lib.rs','src/simulation.rs','src/sensors.rs','src/estimation.rs','src/evaluation.rs','src/main.rs','examples/robustness-run.rs','research/robustness/study.py','research/robustness/test_study.py','research/robustness/README.md','docs/design/robustness-protocol.md']
PROTOCOL_SHA = '05f4bb8553c7166a4707fa0f9e466f5857252ef5976b9a9c96d0a553c919ea2e'

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def key_text(key):
    return '-'.join(map(str, key))

def close(a,b,absolute=1e-8,relative=1e-8):
    return abs(a-b) <= absolute + relative * max(abs(a),abs(b))

def classified(row):
    dx,dy = row['estimated_x']-row['true_x'], row['estimated_y']-row['true_y']
    a,b,c,d = (row[p] for p in ('p00','p01','p10','p11'))
    if abs(b-c)>1e-10 or a<=0 or d<=0 or a*d-b*c<=0:
        raise ValueError('invalid position covariance')
    distance = (d*dx*dx-(b+c)*dx*dy+a*dy*dy)/(a*d-b*c)
    error = math.hypot(dx,dy)
    return distance <= THRESHOLD, error <= 1.0, distance, error

def recovery(within):
    # 101 snapshots: [start,start+100], candidates inclusive through2900.
    return next((start for start in range(2000,2901) if all(within[start:start+101])), None)

def metrics(rows):
    totals = [0.0,0.0,0.0]
    covered = 0
    for row in rows:
        for i,(x,y) in enumerate((('estimated_x','estimated_y'),('hold_x','hold_y'),('integrated_x','integrated_y'))):
            totals[i] += (row[x]-row['true_x'])**2+(row[y]-row['true_y'])**2
        covered += classified(row)[0]
    return {'samples':len(rows), 'filter_rmse':math.sqrt(totals[0]/len(rows)), 'hold_rmse':math.sqrt(totals[1]/len(rows)), 'integration_rmse':math.sqrt(totals[2]/len(rows)), 'position_coverage':covered/len(rows), 'covered':covered}

def read_rows(path, roundtrip=False):
    with path.open(newline='',encoding='utf-8') as stream:
        reader=csv.DictReader(stream)
        if reader.fieldnames != COLUMNS+(EXTRA if roundtrip else []):
            raise ValueError('wrong columns')
        rows=[]
        for tick,record in enumerate(reader):
            if None in record or any(value is None for value in record.values()):
                raise ValueError('wrong column count')
            row={}
            for field,value in record.items():
                if field in ('position_outage',*EXTRA):
                    if value not in ('true','false'): raise ValueError('invalid boolean')
                    row[field]=value=='true'
                elif value=='':
                    if field not in ('position_x','position_y','velocity_x','velocity_y'): raise ValueError('unexpected missing value')
                    row[field]=None
                else:
                    row[field]=float(value)
                    if not math.isfinite(row[field]): raise ValueError('nonfinite value')
            if row['tick']!=tick or tick>3000 or row['time_s']!=tick/100:
                raise ValueError('wrong row order/tick/time')
            if row['position_outage']!=(1000<=tick<2000): raise ValueError('outage mismatch')
            for fields,present in ((('position_x','position_y'),tick%20==0 and not row['position_outage']), (('velocity_x','velocity_y'),tick%5==0)):
                if any((row[field] is not None)!=present for field in fields): raise ValueError('schedule mismatch')
            rows.append(row)
    if len(rows)!=3001: raise ValueError('wrong row count')
    return rows

def read_summary(path,key):
    lines=[line.split('\t') for line in path.read_text().splitlines()]
    if len(lines)!=5 or lines[0]!=['key',*map(str,key)]: raise ValueError('summary key mismatch')
    expected=[30,2,.3,.1,10,20,2*SCALES[key[2]],.3*SCALES[key[2]],*BIASES[key[3]]]
    if lines[1][0]!='config' or list(map(float,lines[1][1:]))!=expected: raise ValueError('summary config mismatch')
    result={}
    for label,line in zip(('overall','outage'),lines[2:4]):
        if line[0]!=label or len(line)!=6: raise ValueError('summary metric shape')
        result[label]=dict(zip(('samples','filter_rmse','hold_rmse','integration_rmse','position_coverage'),map(float,line[1:])))
        if not all(math.isfinite(v) for v in result[label].values()): raise ValueError('nonfinite summary')
    if lines[4][0]!='recovery' or len(lines[4])!=2: raise ValueError('recovery summary shape')
    result['recovery_tick']=None if lines[4][1]=='censored' else int(lines[4][1])
    return result

def verify(stem,key):
    csv_path=Path(str(stem)+'.csv'); exact_path=Path(str(stem)+'.roundtrip.csv')
    rounded=read_rows(csv_path); exact=read_rows(exact_path,True)
    summary=read_summary(Path(str(stem)+'.summary.tsv'),key)
    boundary=[]
    for a,b in zip(rounded,exact):
        # Roundtrip values determine source classifications independently, not flags.
        for field in COLUMNS:
            if isinstance(a[field],float):
                if abs(a[field]-b[field])>1e-10: raise ValueError('CSV/roundtrip value mismatch')
            elif a[field]!=b[field]: raise ValueError('CSV/roundtrip token mismatch')
        ca,ra,qa,ea=classified(a); cb,rb,qb,eb=classified(b)
        if cb!=b['source_covered'] or rb!=b['source_within_recovery']:
            raise ValueError('source classification disagreement')
        near_q=abs(qa-THRESHOLD)<=1e-7
        near_e=abs(ea-1.0)<=1e-8
        if ca!=cb and not near_q or ra!=rb and not near_e:
            raise ValueError('unexplained rounded classification disagreement')
        if near_q or near_e:
            boundary.append({'tick':int(a['tick']),'rounded_coverage':ca,'roundtrip_coverage':cb,'source_coverage':b['source_covered'],'rounded_recovery':ra,'roundtrip_recovery':rb,'source_recovery':b['source_within_recovery'],'rounded_distance':qa,'roundtrip_distance':qb,'rounded_error':ea,'roundtrip_error':eb})
    for label,selection in (('overall',lambda r:True),('outage',lambda r:r['position_outage'])):
        m=metrics([r for r in rounded if selection(r)])
        unrounded=metrics([r for r in exact if selection(r)])
        expected=summary[label]
        if m['samples']!=expected['samples']: raise ValueError('sample count mismatch')
        for field in ('filter_rmse','hold_rmse','integration_rmse'):
            if not close(m[field],expected[field]): raise ValueError('CSV/summary RMSE mismatch')
            if not close(unrounded[field],expected[field]): raise ValueError('roundtrip/summary RMSE mismatch')
        if expected['position_coverage'] != unrounded['covered']/m['samples']:
            raise ValueError('summary coverage is not exact source count fraction')
        source_count=unrounded['covered']
        if unrounded['covered']!=source_count: raise ValueError('roundtrip/summary coverage mismatch')
        if m['covered']!=source_count and not boundary: raise ValueError('unresolved coverage mismatch')
    exact_recovery=recovery([classified(r)[1] for r in exact])
    csv_recovery=recovery([classified(r)[1] for r in rounded])
    if exact_recovery!=summary['recovery_tick']: raise ValueError('source recovery mismatch')
    if csv_recovery!=exact_recovery and not boundary: raise ValueError('unresolved recovery mismatch')
    result={'key':list(key),'summary':summary,'csv_sha256':digest(csv_path),'csv_bytes':csv_path.stat().st_size,'roundtrip_sha256':digest(exact_path),'rounding_boundary_rows':boundary,'rounded_recovery_tick':csv_recovery}
    return result,exact

def invariants(rows,reference,key):
    scale,bias=key[2:]
    if not reference:
        reference.extend({} for _ in rows)
    if len(reference)!=3001: raise ValueError('reference size')
    common=('tick','time_s','true_x','true_y','true_vx','true_vy','position_x','position_y','hold_x','hold_y','position_outage')
    covariance=tuple('p%d%d'%(r,c) for r in range(4) for c in range(4))
    velocity=('velocity_x','velocity_y','integrated_x','integrated_y')
    for row,ref in zip(rows,reference):
        for field in common:
            if field in ref and ref[field]!=row[field]: raise ValueError('common source invariant')
            ref[field]=row[field]
        values=[row[f] for f in covariance]
        if 'cov'+str(scale) in ref and ref['cov'+str(scale)]!=values: raise ValueError('bias changed covariance')
        ref['cov'+str(scale)]=values
        values=[row[f] for f in velocity]
        if 'velocity'+str(bias) in ref and ref['velocity'+str(bias)]!=values: raise ValueError('scale changed observations/integration')
        ref['velocity'+str(bias)]=values
        if row['velocity_x'] is not None:
            # Inventory order visits zero bias first at every scale.
            zero=ref['velocity0']
            for axis,field in enumerate(('velocity_x','velocity_y')):
                delta=row[field]-zero[axis]
                if not close(delta,BIASES[bias][axis],1e-12,1e-12): raise ValueError('bias offset invariant')

def statistics_for(values):
    return {'n':len(values),'mean':statistics.mean(values),'sample_sd':statistics.stdev(values),'min':min(values),'max':max(values)}

def aggregate(records):
    expected={key_text(k) for k in KEYS}
    if set(records)!=expected or len(records)!=576: raise ValueError('incomplete/unexpected aggregate inventory')
    result=[]
    for trajectory,scale,bias in itertools.product(('straight','maneuver'),range(3),range(3)):
        selected=[records[key_text((seed,trajectory,scale,bias))]['summary'] for seed in range(1000,1032)]
        entry={'trajectory':trajectory,'scale':SCALES[scale],'bias':BIASES[bias]}
        for label,field in (('outage','filter_rmse'),('outage','position_coverage'),('overall','filter_rmse'),('overall','position_coverage'),('outage','hold_rmse'),('outage','integration_rmse')):
            entry[label+'_'+field]=statistics_for([r[label][field] for r in selected])
        differences=[records[key_text((seed,trajectory,scale,bias))]['summary']['outage']['filter_rmse']-records[key_text((seed,trajectory,1,bias))]['summary']['outage']['filter_rmse'] for seed in range(1000,1032)]
        paired=statistics_for(differences);paired['monte_carlo_standard_error']=paired['sample_sd']/math.sqrt(32);entry['paired_outage_rmse_vs_scale1']=paired
        recovered=[(r['recovery_tick']-2000)/100 for r in selected if r['recovery_tick'] is not None]
        entry['recovery']={'total':32,'recovered':len(recovered),'censored':32-len(recovered),'recovered_latency_seconds':statistics_for(recovered) if len(recovered)>1 else {'values':recovered}}
        result.append(entry)
    return result

def fingerprints(binary):
    protocol=ROOT/'docs/design/robustness-protocol.md'
    if digest(protocol)!=PROTOCOL_SHA: raise ValueError('approved protocol changed')
    git=['git','-c','safe.directory='+str(ROOT)]
    revision=subprocess.check_output(git+['rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    tree=subprocess.check_output(git+['rev-parse','HEAD^{tree}'],cwd=ROOT,text=True).strip()
    return {'revision':revision,'tree':tree,'files':{p:digest(ROOT/p) for p in SOURCE_PATHS},'binary_sha256':digest(binary),'toolchain':subprocess.check_output(['rustc','-Vv'],text=True),'python':sys.version,'platform':platform.platform(),'machine':platform.machine(),'inventory':[list(k) for k in KEYS]}

def unique_object(pairs):
    result={}
    for key,value in pairs:
        if key in result: raise ValueError('duplicate JSON key')
        result[key]=value
    return result

def load_json(path):
    return json.loads(path.read_text(),object_pairs_hook=unique_object)

def check_inventory(records, complete=False):
    expected={key_text(k) for k in KEYS}
    if any(k not in expected for k in records): raise ValueError('unexpected resumed key')
    if complete and set(records)!=expected: raise ValueError('missing run keys')
    ordered=[key_text(k) for k in KEYS]
    if list(records)!=ordered[:len(records)]: raise ValueError('resumed inventory is not an ordered prefix')
    for index,(k,value) in enumerate(records.items()):
        if key_text(value['key'])!=k: raise ValueError('record key mismatch')
        if value.get('status') not in ('verified','failed'): raise ValueError('invalid recorded status')
        if value['status']=='failed' and index!=len(records)-1: raise ValueError('failed key must stop execution')
        if value['status']=='verified':
            for label,count in (('overall',3001),('outage',1000)):
                m=value['summary'][label]
                if m['samples']!=count or not all(math.isfinite(v) for v in m.values()): raise ValueError('invalid resumed metric')
                if any(m[f]<0 for f in ('filter_rmse','hold_rmse','integration_rmse')) or not 0<=m['position_coverage']<=1: raise ValueError('invalid metric range')
            tick=value['summary']['recovery_tick']
            if tick is not None and (not isinstance(tick,int) or not 2000<=tick<=2900): raise ValueError('invalid recovery tick')
            if len(value['csv_sha256'])!=64 or not 0<value['csv_bytes']<=32*1024*1024: raise ValueError('invalid recorded artifact')

def bounded_run(command,output,seconds,cap_bytes=32*1024*1024):
    stdout=output/'current.stdout.log';stderr=output/'current.stderr.log'
    start=time.monotonic()
    with stdout.open('wb') as out,stderr.open('wb') as err:
        process=subprocess.Popen(command,stdout=out,stderr=err)
        try:
            while process.poll() is None:
                if sum(p.stat().st_size for p in output.glob('current.*'))>cap_bytes:
                    raise ValueError('artifact cap exceeded')
                if time.monotonic()-start>=seconds:
                    raise subprocess.TimeoutExpired(command,seconds)
                time.sleep(.05)
        except BaseException:
            process.kill();process.wait();raise
    if process.returncode:
        raise subprocess.CalledProcessError(process.returncode,command)
    if sum(p.stat().st_size for p in output.glob('current.*'))>cap_bytes:
        raise ValueError('artifact cap exceeded')
    return stdout.read_text(),stderr.read_text()

class CheckpointReplaceError(RuntimeError):
    """Bounded journal promotion failed; old and pending states remain inspectable."""
    def __init__(self,checkpoint,temporary,errors):
        self.checkpoint=checkpoint
        self.temporary=temporary
        self.errors=tuple(errors)
        super().__init__('checkpoint promotion exhausted; reconcile preserved old/tmp before resume')

def checkpoint_event(path,event):
    with path.open('a',encoding='utf-8') as stream:
        stream.write(json.dumps(event,separators=(',',':'))+'\n')
        stream.flush();os.fsync(stream.fileno())

def commit_checkpoint(checkpoint,state):
    temporary=checkpoint.with_suffix('.tmp')
    # Never overwrite a previously unacknowledged journal on a fresh attempt.
    with temporary.open('x',encoding='utf-8') as stream:
        json.dump(state,stream,separators=(',',':'))
        stream.flush();os.fsync(stream.fileno())
    events=checkpoint.with_name('checkpoint-replace-events.jsonl')
    errors=[]
    for attempt in range(1,6):
        try:
            temporary.replace(checkpoint)
        except PermissionError as error:
            event={'operation':'checkpoint_replace','attempt':attempt,'error_type':type(error).__name__,'errno':error.errno,'winerror':getattr(error,'winerror',None),'message':str(error),'pending_sha256':digest(temporary)}
            errors.append(event)
            checkpoint_event(events,event)
            if attempt==5:
                raise CheckpointReplaceError(checkpoint,temporary,errors) from error
            time.sleep(.1)
        else:
            if errors:checkpoint_event(events,{'operation':'checkpoint_replace','status':'promoted','attempt':attempt,'checkpoint_sha256':digest(checkpoint)})
            return

def execute(args):
    output=args.output.resolve();output.mkdir(parents=True,exist_ok=True)
    if args.freeze and any(output.iterdir()):
        raise ValueError('freeze requires a fresh dedicated empty output directory')
    binary=args.binary.resolve()
    fingerprint=fingerprints(binary)
    frozen=output/'frozen.json'
    if args.freeze:
        if frozen.exists(): raise ValueError('freeze already exists')
        with frozen.open('x',encoding='utf-8') as stream:
            json.dump(fingerprint,stream,indent=2)
            stream.flush();os.fsync(stream.fileno())
        return
    if not frozen.exists() or load_json(frozen)!=fingerprint: raise ValueError('freeze mismatch; no execution')
    if not args.execute: raise ValueError('explicit --execute required')
    checkpoint=output/'checkpoint.json'
    if checkpoint.with_suffix('.tmp').exists():
        raise ValueError('pending checkpoint journal exists; independently reconcile before resume')
    state=load_json(checkpoint) if checkpoint.exists() else {'records':{},'reference':[],'group':None}
    records=state['records'];check_inventory(records)
    started=time.monotonic();reference=state['reference'];group=tuple(state['group']) if state['group'] else None
    expected_next=next((key_text(k) for k in KEYS if key_text(k) not in records or records[key_text(k)]['status']!='verified'),None)
    if args.retry_key and args.retry_key!=expected_next: raise ValueError('retry must name first outstanding key')
    for key in KEYS:
        remaining=4*3600-(time.monotonic()-started)
        if remaining<=0: break
        if key_text(key) in records and records[key_text(key)]['status']=='verified': continue
        if group!=key[:2]: reference=[];group=key[:2]
        previous=records.get(key_text(key));history=[] if previous is None else previous.get('attempts',[])
        if previous is not None:
            if args.retry_key!=key_text(key) or not previous.get('transient') or len(history)!=1:
                raise ValueError('failed key retained; only one explicitly named transient retry allowed')
        for artifact in output.glob('current.*'): artifact.unlink()
        stem=output/'current';command=[str(binary),*map(str,key),str(stem)]
        attempt={'key':list(key),'command':command,'status':'running'}
        executing=True
        try:
            stdout,stderr=bounded_run(command,output,min(60,remaining))
            executing=False
            artifacts=list(output.glob('current.*'))
            verified,rows=verify(stem,key)
            invariants(rows,reference,key)
            attempt.update(verified);attempt.update(status='verified',stdout=stdout,stderr=stderr)
            if verified['rounding_boundary_rows']:
                evidence=output/'rounding-boundary-evidence'/key_text(key)
                evidence.mkdir(parents=True,exist_ok=False)
                for artifact in artifacts: (evidence/artifact.name).write_bytes(artifact.read_bytes())
                attempt['retained_boundary_evidence']=str(evidence)
        except Exception as error:
            transient=executing and isinstance(error,(OSError,subprocess.TimeoutExpired))
            evidence=output/'failure-evidence'/key_text(key)/str(len(history)+1)
            evidence.mkdir(parents=True,exist_ok=False)
            for artifact in output.glob('current.*'): (evidence/artifact.name).write_bytes(artifact.read_bytes())
            attempt.update(status='failed',transient=transient,error=repr(error),artifact_hashes={p.name:digest(p) for p in evidence.iterdir()},retained_failure_evidence=str(evidence))
        history.append(dict(attempt));attempt['attempts']=history
        records[key_text(key)]=attempt
        state={'records':records,'group':list(group),'reference':reference}
        commit_checkpoint(checkpoint,state)
        if attempt['status']!='verified': raise ValueError('failed key checkpointed; interpretation stopped')
        # Compact checked metrics/hash records plus the bounded reference are durable.
        for artifact in output.glob('current.*'): artifact.unlink()
    if len(records)==576 and all(r['status']=='verified' for r in records.values()):
        check_inventory(records,True)
        (output/'records.json').write_text(json.dumps(records,indent=2))
        (output/'aggregates.json').write_text(json.dumps(aggregate(records),indent=2))
        print('complete: 576 verified keys')
    else: print('checkpoint: %d/576 verified keys'%sum(r['status']=='verified' for r in records.values()))

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--freeze',action='store_true')
    parser.add_argument('--execute',action='store_true')
    parser.add_argument('--retry-key',help='one explicitly recorded retry for first failed transient process/I/O key')
    args=parser.parse_args();execute(args)

if __name__=='__main__': main()
