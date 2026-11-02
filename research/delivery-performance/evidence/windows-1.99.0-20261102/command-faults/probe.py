import importlib.util,pathlib,subprocess,sys,json,os,time,hashlib
root=pathlib.Path('C:/Users/Admin/Documents/Codex/2026-10-20/if-x20/work');repo=root/'fusion-lab'
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
source=repo/'research/delivery-performance/paired_probe.py'
pair=load('paired_probe',source);owner=load('accepted_owner',repo/'docs/design-docs/observation-delivery/implementation/resource_probe.py')
base=root/('accepted-update-command-faults-'+str(time.time_ns()));base.mkdir();original=subprocess.Popen;summary=[]
for mode in ['normal_metadata','transient_kill','transient_wait']:
 guard=base/(mode+'.guard.json');guard.write_text(json.dumps(dict(token=mode,owner_pid=os.getpid(),status='fixture_owned')))
 owner.OWNER_GUARD=guard;owner.OWNER_TOKEN=mode
 actual=[]
 class Proxy:
  def __init__(self,p):self.p=p;self.waits=0;self.kills=0
  def __getattr__(self,key):return getattr(self.p,key)
  def wait(self,timeout=None):
   self.waits+=1
   if mode!='normal_metadata' and self.waits==1:raise subprocess.TimeoutExpired(self.p.args,timeout,'injected command-watchdog failure')
   if mode=='transient_wait' and self.waits==2:raise OSError('injected transient reap operation failure')
   return self.p.wait(timeout=timeout)
  def kill(self):
   self.kills+=1
   if mode=='transient_kill' and self.kills==1:raise PermissionError('injected transient kill operation failure')
   return self.p.kill()
 def factory(*args,**kwargs):
  p=original(*args,**kwargs);actual.append(p);return Proxy(p)
 pair.subprocess.Popen=factory
 argv=[sys.executable,'-c','print("ordinary nonnumerical command")'] if mode=='normal_metadata' else [sys.executable,'-c','import time;time.sleep(120)']
 record={};receipt=base/(mode+'.json')
 pair.run_command(owner,argv,base,receipt,record=record,timeout=5)
 assert record['reap_confirmed'] and actual[0].poll() is not None and receipt.exists() and guard.exists()
 assert record['process_tree_cleanup_guaranteed'] is False
 if mode=='normal_metadata':assert record['success'] and 'ordinary nonnumerical command' in record['stdout']
 else:
  assert not record['success'] and 'TimeoutExpired' in record['error'] and record['descendant_reconciliation_required']
  assert any(e['operation']==('kill' if mode=='transient_kill' else 'wait_reap') for e in record['cleanup_errors'])
  assert json.loads(guard.read_text())['child_pid']==actual[0].pid
 summary.append(dict(mode=mode,actual_owned_pid=actual[0].pid,exit=actual[0].returncode,reap_confirmed=True,original_error=record.get('error'),guard_retained=True,descendant_reconciliation_required=record.get('descendant_reconciliation_required',False),command_receipt_sha256=hashlib.sha256(receipt.read_bytes()).hexdigest(),receipt=str(receipt)))
 # The fixture knows its Python child spawned no children. Reconcile only its
 # own fixture guard after actual direct reap; never clear a real build guard.
 guard.unlink();pair.subprocess.Popen=original
result=dict(status='NONNUMERICAL_REAL_DIRECT_CHILD_COMMAND_FAULT_PROOF_NOT_BENCHMARK',probe_sha256=hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),paired_source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),results=summary,all_direct_children_reaped=True,no_compiler_or_benchmark_started=True)
(base/'summary.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
