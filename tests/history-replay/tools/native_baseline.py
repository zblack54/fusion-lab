"""Feature-specific native libtest evidence driver; not the private spec validator.

Builds/enumerates actual executables, verifies one exact outcome per catalog entry,
and preserves bounded raw command logs plus source/test/lock fingerprints.
"""
import argparse
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tempfile
import time
import tomllib

SUITE=Path(__file__).resolve().parents[1]
ROOT=SUITE.parents[1]
PACKAGE=ROOT/'docs/design-docs/history-replay'

def sha(path):
    value=hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda:source.read(65536),b''):value.update(chunk)
    return value.hexdigest()

def fingerprints():
    paths=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'rust-toolchain.toml',*sorted((ROOT/'src').glob('*.rs')),ROOT/'docs/design-docs/observation-delivery/implementation/resource_probe.py']
    paths += [p for p in sorted(SUITE.rglob('*')) if p.is_file() and not any(part in ('target','__pycache__','.git') for part in p.relative_to(SUITE).parts)]
    paths += [p for p in sorted(PACKAGE.rglob('*')) if p.is_file() and p.relative_to(PACKAGE).parts[0] not in ('implementation','validation')]
    return {str(p.relative_to(ROOT)).replace('\\','/'):sha(p) for p in paths}

OWNER=None
GUARD=None

def invoke(argv,env,timeout,cap,output):
    started=time.monotonic();child=None;failure=None
    record={'argv':argv,'cwd':str(ROOT),'output':output.name}
    with tempfile.TemporaryFile() as raw:
        try:
            child=subprocess.Popen(argv,cwd=ROOT,env=env,stdout=raw,stderr=subprocess.STDOUT)
            record['pid']=child.pid
            OWNER.owner_update(status='command_running',child_pid=child.pid,child_argv=argv)
            while child.poll() is None:
                if os.fstat(raw.fileno()).st_size>cap:raise RuntimeError('command output cap exceeded')
                if time.monotonic()-started>timeout:raise TimeoutError('owned command deadline exceeded')
                time.sleep(.02)
            record['reap_confirmed']=isinstance(child.wait(timeout=1),int)
        except BaseException as error:
            failure=repr(error);record['error']=failure
            if child is not None:
                # Same Popen remains owned; this call cannot return until actual reap.
                OWNER.reap_owned(child,record,output.with_suffix('.pending.json'))
                record['descendant_reconciliation_required']=True
                OWNER.owner_update(status='reconciliation_required',compiler_descendants_unverified=True)
            raw.seek(0);data=raw.read(cap+1);output.write_bytes(data)
            record.update(returncode=child.returncode if child else None,seconds=time.monotonic()-started,output_sha256=sha(output),output_bytes=len(data),excerpt=data[:3000].decode('utf-8','replace'))
            output.with_suffix('.failure.json').write_text(json.dumps(record,indent=2),encoding='utf-8')
            raise RuntimeError(json.dumps(record)) from error
        raw.seek(0);data=raw.read(cap+1)
    output.write_bytes(data)
    record.update(returncode=child.returncode,seconds=time.monotonic()-started,output_sha256=sha(output),output_bytes=len(data),excerpt=data[:3000].decode('utf-8','replace'))
    if len(data)>cap:raise RuntimeError(json.dumps(dict(record,failure='output limit')))
    return record,data.decode('utf-8','replace')

def result_counts(text):
    matches=re.findall(r'^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text,re.MULTILINE)
    if len(matches)!=1:raise ValueError('missing/ambiguous libtest summary')
    return tuple(map(int,matches[0]))

def main():
    global OWNER,GUARD
    parser=argparse.ArgumentParser()
    parser.add_argument('--toolchain',required=True,choices=['1.99.0','stable'])
    parser.add_argument('--mode',choices=['design','implementation'],default='design')
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if platform.system() not in ('Windows','Linux'):raise SystemExit('required native platform is Windows or Linux')
    output=args.output.resolve()
    if output==ROOT or ROOT in output.parents:raise SystemExit('native raw output must stay outside repository')
    output.mkdir(parents=True,exist_ok=True)
    if any(output.iterdir()):raise SystemExit('evidence output directory must be fresh and empty')
    sys.dont_write_bytecode=True
    helper=ROOT/'docs/design-docs/observation-delivery/implementation/resource_probe.py'
    if sha(helper)!='ab851ab12ec6f54ee52baec37b0b59b80f8db069b5a92179c3f12f23cc9b924c':raise SystemExit('accepted owner helper mismatch')
    spec=importlib.util.spec_from_file_location('history_native_owner',helper);OWNER=importlib.util.module_from_spec(spec);spec.loader.exec_module(OWNER)
    GUARD=PACKAGE/'implementation/.native-baseline-owner.json';OWNER.OWNER_GUARD=GUARD;OWNER.OWNER_TOKEN=os.urandom(16).hex()
    with GUARD.open('x') as file:json.dump(dict(token=OWNER.OWNER_TOKEN,owner_pid=os.getpid(),status='native_preparing',output=str(output)),file)
    env=os.environ.copy();env['RUSTUP_TOOLCHAIN']=args.toolchain;env['CARGO_TERM_COLOR']='never';env['RUST_BACKTRACE']='0';env['RUSTDOCFLAGS']='-D warnings'
    catalog=tomllib.loads((PACKAGE/'tests/tests.toml').read_text(encoding='utf-8'))['test']
    if not catalog or any(t['test_target']!='test:contracts' or not t.get('required',False) for t in catalog):raise SystemExit('unsupported/incomplete feature test catalog')
    names=[t['selector'] for t in catalog]
    if len(names)!=len(set(names)):raise SystemExit('duplicate catalog selectors')
    receipt={'schema':1,'mode':args.mode,'platform':platform.platform(),'machine':platform.machine(),'python':platform.python_version(),'toolchain':args.toolchain,'rustdocflags':env['RUSTDOCFLAGS'],'source_before':fingerprints(),'commands':[],'tests':[],'success':False}
    def command(argv,name,timeout=120,cap=4*1024*1024):
        record,text=invoke(argv,env,timeout,cap,output/name);receipt['commands'].append(record);return record,text
    try:
        record,text=command(['rustc','-Vv'],'rustc.txt')
        if record['returncode']:raise ValueError('compiler inspection failed')
        receipt['rustc']=text
        record,text=command(['git','-c','safe.directory='+str(ROOT),'rev-parse','HEAD'],'revision.txt')
        if record['returncode']:raise ValueError('revision inspection failed')
        receipt['revision']=text.strip()
        record,text=command(['cargo','test','--manifest-path',str(SUITE/'Cargo.toml'),'--locked','--no-run','--message-format=json'],'compile.jsonl',timeout=240)
        if record['returncode']:raise ValueError('harness compile failed')
        executables=[]
        for line in text.splitlines():
            try:item=json.loads(line)
            except json.JSONDecodeError:continue
            if item.get('reason')=='compiler-artifact' and item.get('target',{}).get('name')=='contracts' and 'test' in item.get('target',{}).get('kind',[]) and item.get('executable'):executables.append(item['executable'])
        if len(set(executables))!=1:raise ValueError('missing/duplicate compiled contracts executable')
        executable=executables[0]
        record,text=command([executable,'--list'],'enumeration.txt',timeout=60,cap=1024*1024)
        if record['returncode']:raise ValueError('libtest enumeration failed')
        actual=re.findall(r'^(.+): test\r?$',text,re.MULTILINE)
        if sorted(actual)!=sorted(names) or len(actual)!=len(names):raise ValueError('actual compiled selectors do not exactly match catalog')
        for test in catalog:
            record,text=command([executable,test['selector'],'--exact','--nocapture','--test-threads=1'],test['id']+'.txt',timeout=240 if test['kind']=='structural' else 60,cap=1024*1024)
            counts=result_counts(text)
            if counts[4]!=len(catalog)-1:raise ValueError('exact invocation filtered count mismatch')
            expected='pass' if args.mode=='implementation' else test['expected_pre_implementation']
            normal_pass=record['returncode']==0 and counts[:4]==(1,0,0,0)
            normal_fail=record['returncode']!=0 and counts[:4]==(0,1,0,0)
            if expected=='pass':valid=normal_pass
            elif expected=='fail':valid=normal_fail and bool(test.get('expected_failure_contains')) and test['expected_failure_contains'] in text
            else:raise ValueError('unsupported expected result')
            receipt['tests'].append({'id':test['id'],'selector':test['selector'],'expected':expected,'counts':counts,'valid':valid,'output':record['output'],'output_sha256':record['output_sha256']})
            if not valid:raise ValueError('unexpected exact test outcome: '+test['id'])
        receipt['source_after']=fingerprints()
        if receipt['source_before']!=receipt['source_after']:raise ValueError('source/test/lock changed during native baseline')
        receipt['expected_red']=sum(t['expected']=='fail' for t in receipt['tests']);receipt['passed']=sum(t['expected']=='pass' for t in receipt['tests'])
        receipt['success']=len(receipt['tests'])==len(catalog)
    except BaseException as error:
        receipt['error']=repr(error)
    finally:
        (output/'receipt.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8')
        # Expected-red tests are normal reaped exits, not cleanup failures.
        state=json.loads(GUARD.read_text())
        if not state.get('compiler_descendants_unverified'):
            OWNER.owner_update(status='finished_direct_children_reaped',child_pid=None,process_tree_cleanup_guaranteed=False);GUARD.unlink()
        else:
            receipt['guard_retained_for_manual_reconciliation']=True
            (output/'receipt.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8')
    print(json.dumps({key:receipt.get(key) for key in ('success','platform','toolchain','expected_red','passed','error')}))
    if not receipt['success']:raise SystemExit(1)

if __name__=='__main__':main()
