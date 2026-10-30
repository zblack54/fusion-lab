import argparse,importlib.util,pathlib,subprocess,sys,json,threading,time,os,hashlib
parser=argparse.ArgumentParser(description='Actual owned-child lifecycle fault qualification; no numerical workload')
parser.add_argument('--output',type=pathlib.Path,required=True)
args=parser.parse_args()
source=pathlib.Path(__file__).resolve().with_name('resource_probe.py')
spec=importlib.util.spec_from_file_location('resource_probe',source);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
original=subprocess.Popen
out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
results=[]
for mode in ['transient_kill','transient_wait','persistent_pending']:
 gate=out/(mode+'.gate');gate.write_text('hold')
 guard=out/(mode+'.owner.json');guard.write_text(json.dumps(dict(token=mode,owner_pid=os.getpid(),status='fixture_owned')))
 m.OWNER_GUARD=guard;m.OWNER_TOKEN=mode
 actual=[];pending=[]
 class Proxy:
  def __init__(self,p):self.p=p;self.kills=0;self.waits=0
  def __getattr__(self,key):return getattr(self.p,key)
  def kill(self):
   self.kills+=1
   if (mode=='transient_kill' and self.kills==1) or (mode=='persistent_pending' and gate.exists()):raise PermissionError('injected owned-child kill failure')
   return self.p.kill()
  def wait(self,timeout=None):
   self.waits+=1
   if (mode=='transient_wait' and self.waits==1) or (mode=='persistent_pending' and gate.exists()):raise TimeoutError('injected owned-child wait failure')
   return self.p.wait(timeout=timeout)
 def factory(*a,**kw):
  p=original([sys.executable,'-c','import sys,time;print("FAULT",flush=True);time.sleep(120)'],**kw);actual.append(p);return Proxy(p)
 m.subprocess.Popen=factory
 def release_pending():
  marker=out/(mode+'.pending.json')
  deadline=time.monotonic()+20
  while time.monotonic()<deadline:
   if marker.exists():
    try:r=json.loads(marker.read_text())
    except (json.JSONDecodeError,PermissionError):time.sleep(.02);continue
    if r.get('cleanup_status')=='owned_pending':
     assert actual[0].poll() is None
     assert guard.exists()
     try:guard.open('x');raise AssertionError('owner guard bypass')
     except FileExistsError:pass
     pending.append(r);gate.unlink();return
   time.sleep(.02)
  # Release on fixture failure so its actual child still gets cleaned up.
  gate.unlink(missing_ok=True)
 if mode=='persistent_pending':
  controller=threading.Thread(target=release_pending);controller.start()
 receipt=out/(mode+'.json')
 r=m.measured(pathlib.Path('synthetic-fault-fixture-no-numerical-work'),'maximum',receipt)
 if mode=='persistent_pending':controller.join(timeout=30);assert pending
 assert not r['success'] and r['reap_confirmed'] and actual[0].poll() is not None
 assert 'missing/malformed complete-work proof' in r['error']
 assert any(e['operation']==('wait_reap' if mode=='transient_wait' else 'kill') for e in r['cleanup_errors'])
 results.append(dict(mode=mode,actual_owned_pid=actual[0].pid,exit=actual[0].returncode,reap_confirmed=r['reap_confirmed'],cleanup_error_count=r['cleanup_error_count'],persistent_pending_observed=bool(pending),original_error=r['error'],receipt=str(receipt)))
 guard.unlink();gate.unlink(missing_ok=True)
 m.subprocess.Popen=original
summary=dict(helper_sha256=hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),kind='actual owned Python children with deterministic injected OS-operation faults; NO numerical workload',sampler_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),results=results,all_reaped=True)
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
