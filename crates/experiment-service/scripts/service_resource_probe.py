"""Native Windows/Linux evidence; local code only, no external service.

Run from repository root with bundled Python:
python crates/experiment-service/scripts/service_resource_probe.py --output docs/design-docs/experiment-service/implementation/native-probe.json
Build release prerequisites first with --build (or record the equivalent build commands).
"""
from __future__ import annotations
import argparse, contextlib, ctypes, ctypes.wintypes as wt, hashlib, http.client, json, os, pathlib, platform, queue, signal, socket, subprocess, sys, tempfile, threading, time

MIB=1024*1024
class MEMORY(ctypes.Structure):
    _fields_=[("cb",wt.DWORD),("PageFaultCount",wt.DWORD)]+[(name,ctypes.c_size_t) for name in ("PeakWorkingSetSize","WorkingSetSize","QuotaPeakPagedPoolUsage","QuotaPagedPoolUsage","QuotaPeakNonPagedPoolUsage","QuotaNonPagedPoolUsage","PagefileUsage","PeakPagefileUsage","PrivateUsage")]
class PROCESS(ctypes.Structure):
    _fields_=[("dwSize",wt.DWORD),("cntUsage",wt.DWORD),("th32ProcessID",wt.DWORD),("th32DefaultHeapID",ctypes.c_size_t),("th32ModuleID",wt.DWORD),("cntThreads",wt.DWORD),("th32ParentProcessID",wt.DWORD),("pcPriClassBase",wt.LONG),("dwFlags",wt.DWORD),("szExeFile",wt.WCHAR*260)]
K=ctypes.WinDLL("kernel32",use_last_error=True) if os.name=="nt" else None
P=ctypes.WinDLL("psapi",use_last_error=True) if os.name=="nt" else None
if K:
    K.CreateToolhelp32Snapshot.argtypes=[wt.DWORD,wt.DWORD];K.CreateToolhelp32Snapshot.restype=wt.HANDLE
    K.Process32FirstW.argtypes=[wt.HANDLE,ctypes.POINTER(PROCESS)];K.Process32NextW.argtypes=[wt.HANDLE,ctypes.POINTER(PROCESS)]
    K.OpenProcess.argtypes=[wt.DWORD,wt.BOOL,wt.DWORD];K.OpenProcess.restype=wt.HANDLE;K.CloseHandle.argtypes=[wt.HANDLE]
    P.GetProcessMemoryInfo.argtypes=[wt.HANDLE,ctypes.POINTER(MEMORY),wt.DWORD];P.GetProcessMemoryInfo.restype=wt.BOOL
def processes():
    if os.name!="nt":
        found={}
        for directory in pathlib.Path("/proc").iterdir():
            if directory.name.isdigit():
                try:
                    fields=dict(line.split(":",1) for line in (directory/"status").read_text().splitlines());found[int(directory.name)]=int(fields["PPid"].strip())
                except (OSError,ValueError,KeyError):pass
        return found
    handle=K.CreateToolhelp32Snapshot(2,0);entry=PROCESS();entry.dwSize=ctypes.sizeof(entry);found={}
    try:
        more=K.Process32FirstW(handle,ctypes.byref(entry))
        while more:
            found[entry.th32ProcessID]=entry.th32ParentProcessID;more=K.Process32NextW(handle,ctypes.byref(entry))
    finally:K.CloseHandle(handle)
    return found
def disk_bytes(root):
    total=0
    for path in root.rglob("*"):
        try:
            if path.is_file():total+=path.stat().st_size
        except FileNotFoundError:pass
    return total
class Monitor:
    def __init__(self,pid,root):
        self.pid=pid;self.root=root;self.stop=threading.Event();self.handles={};self.peaks={};self.aggregate_working=0;self.aggregate_commit=0;self.disk=0;self.samples=0;self.errors=[];self.thread=threading.Thread(target=self.loop,daemon=True);self.thread.start()
    def sample(self):
        table=processes();family={self.pid}
        while True:
            next_family=family|{pid for pid,parent in table.items() if parent in family}
            if next_family==family:break
            family=next_family
        for pid in family:
            if pid not in self.handles:
                handle=K.OpenProcess(0x410,False,pid) if K else pid
                if handle:self.handles[pid]=handle
        working=commit=0
        for pid,handle in self.handles.items():
            if not K:
                try:
                    fields=dict(line.split(":",1) for line in pathlib.Path(f"/proc/{pid}/status").read_text().splitlines())
                    number=lambda key:int(fields.get(key,"0 kB").split()[0])*1024
                    rollup=dict(line.split(":",1) for line in pathlib.Path(f"/proc/{pid}/smaps_rollup").read_text().splitlines() if ":" in line)
                    private=sum(int(rollup.get(key,"0 kB").split()[0])*1024 for key in ("Private_Clean","Private_Dirty"))
                    self.peaks[pid]={"peak_working_set_bytes":number("VmHWM"),"peak_virtual_size_bytes":number("VmPeak"),"private_resident_bytes_last":private,"private_writable_virtual_bytes_last":number("VmData")+number("VmStk"),"private_commit_bytes":None}
                    working+=number("VmRSS");commit+=private
                except (OSError,ValueError,KeyError):pass
                continue
            counters=MEMORY();counters.cb=ctypes.sizeof(counters)
            if P.GetProcessMemoryInfo(handle,ctypes.byref(counters),ctypes.sizeof(counters)):
                self.peaks[pid]={"peak_working_set_bytes":counters.PeakWorkingSetSize,"peak_private_commit_bytes":counters.PeakPagefileUsage,"private_commit_bytes_last":counters.PrivateUsage}
                if pid in table:working+=counters.WorkingSetSize;commit+=counters.PrivateUsage
        self.aggregate_working=max(self.aggregate_working,working);self.aggregate_commit=max(self.aggregate_commit,commit);self.disk=max(self.disk,disk_bytes(self.root));self.samples+=1
    def loop(self):
        while not self.stop.is_set():
            try:self.sample()
            except Exception as error:self.errors.append(repr(error))
            self.stop.wait(.005)
    def finish(self):
        self.stop.set();self.thread.join();self.sample()
        if K:
            for handle in self.handles.values():K.CloseHandle(handle)
        return {"sampled_aggregate_working_set_peak_bytes":self.aggregate_working,"sampled_aggregate_private_commit_peak_bytes":self.aggregate_commit if K else None,"sampled_aggregate_private_resident_peak_bytes":None if K else self.aggregate_commit,"process_lifetime_peaks":self.peaks,"disk_peak_bytes":self.disk,"samples":self.samples,"sampling_errors":self.errors,"requested_interval_seconds":.005,"collector":"Windows GetProcessMemoryInfo with held lifetime handles" if K else "Linux /proc/status VmRSS/VmHWM and smaps_rollup Private_Clean+Private_Dirty; no exact private-commit counter"}
class Host:
    instances=[]
    def __init__(self,host,root,worker):
        self.root=root;self.process=subprocess.Popen([str(host),str(root),str(worker)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,start_new_session=os.name!="nt")
        lines=queue.Queue();threading.Thread(target=lambda:lines.put(self.process.stdout.readline()),daemon=True).start()
        line=lines.get(timeout=15)
        if not line:raise RuntimeError("host failed: "+self.process.stderr.read())
        self.initial=json.loads(line);self.address=self.initial["address"];self.monitor=Monitor(self.process.pid,root);self.stopped=None;Host.instances.append(self)
    def request(self,method,path,body=None,headers=None):
        connection=http.client.HTTPConnection(self.address,timeout=10);start=time.perf_counter()
        try:
            data=None if body is None else json.dumps(body);connection.request(method,path,data,headers or {"Content-Type":"application/json"});response=connection.getresponse();raw=response.read();elapsed=time.perf_counter()-start
            try:value=json.loads(raw)
            except (ValueError,UnicodeDecodeError):value=raw.decode(errors="replace")
            return response.status,value,elapsed
        finally:connection.close()
    def submit(self,body):
        status,value,elapsed=self.request("POST","/v1/runs",body);assert status==202,(status,value);return value["id"],elapsed
    def wait(self,run,timeout=30):
        deadline=time.monotonic()+timeout;latencies=[]
        while time.monotonic()<deadline:
            status,value,elapsed=self.request("GET","/v1/runs/"+run);assert status==200,(status,value);latencies.append(elapsed)
            if value["state"]!="running":return value,latencies
            time.sleep(.005)
        raise TimeoutError(run)
    def stop(self,expect_success=True):
        if self.stopped is not None:return self.stopped
        start=time.perf_counter();self.process.stdin.write("shutdown\n");self.process.stdin.flush()
        out,err=self.process.communicate(timeout=10);metrics=self.monitor.finish()
        self.stopped={"shutdown":json.loads(out) if out else None,"outer_shutdown_seconds":time.perf_counter()-start,"metrics":metrics,"root_remaining":list(path.name for path in self.root.iterdir()) if self.root.exists() else None,"stderr":err,"returncode":self.process.returncode}
        if expect_success:assert self.process.returncode==0,(out,err)
        return self.stopped
@contextlib.contextmanager
def probe_directory():
    with tempfile.TemporaryDirectory(prefix="fusion-service-probe-") as temporary:
        try:yield temporary
        finally:
            for host in Host.instances:
                if host.stopped is None:
                    try:host.stop()
                    except Exception:
                        kill_host(host)
def kill_host(host):
    if os.name=="nt":subprocess.run(["taskkill","/PID",str(host.process.pid),"/T","/F"],capture_output=True)
    else:
        try:os.killpg(host.process.pid,signal.SIGKILL)
        except ProcessLookupError:pass
def config(duration=3600,seed=42):return {"seed":seed,"duration_seconds":duration,"scenario":"maneuver","position_sigma":1.0,"velocity_sigma":.15,"acceleration_density":.1,"outage":{"start":10,"end":20} if duration>=20 else None}
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def await_ready(root,run):
    marker=root/run/"worker.ready";deadline=time.monotonic()+5
    while time.monotonic()<deadline:
        try:return json.loads(marker.read_text())
        except (OSError,ValueError):time.sleep(.005)
    raise TimeoutError("controlled worker readiness")
def connect(address):return socket.create_connection(tuple([address.rsplit(":",1)[0],int(address.rsplit(":",1)[1])]),timeout=5)
def start_sse(address,run):
    sock=connect(address);sock.sendall(f"GET /v1/runs/{run}/events HTTP/1.1\r\nHost: {address}\r\n\r\n".encode());raw=b""
    while b"\r\n\r\n" not in raw:raw+=sock.recv(4096)
    return sock,int(raw.split()[1])
def slow_reader_eviction(host,run):
    address,port=host.address.rsplit(":",1);sock=socket.socket();sock.setsockopt(socket.SOL_SOCKET,socket.SO_RCVBUF,1024);sock.settimeout(5);sock.connect((address,int(port)))
    sock.sendall(f"GET /v1/runs/{run}/artifacts/samples.csv HTTP/1.1\r\nHost: {host.address}\r\nConnection: close\r\n\r\n".encode());raw=b""
    while b"\r\n\r\n" not in raw:raw+=sock.recv(512)
    assert int(raw.split()[1])==200
    try:
        for _ in range(2):
            new,_=host.submit(config(duration=1));done,_=host.wait(new);assert done["state"]=="completed"
        started=time.perf_counter();status,_,_=host.request("GET","/v1/runs/"+run);assert status==404
        deferred=(host.root/run).exists();assert deferred,"open slow-reader lease must defer Windows-safe deletion"
        deadline=time.monotonic()+7
        while (host.root/run).exists() and time.monotonic()<deadline:time.sleep(.005)
        elapsed=time.perf_counter()-started;removed=not (host.root/run).exists();assert removed,"write-idle deadline must close body and release retired artifact"
        assert elapsed<5,("retired cleanup exceeded5second idle/write bound",elapsed)
        return {"status_after_eviction":status,"directory_initially_retained":deferred,"deferred_deletion_seconds":elapsed,"directory_removed_without_client_reading_or_closing":removed,"socket_receive_buffer_bytes":sock.getsockopt(socket.SOL_SOCKET,socket.SO_RCVBUF),"initial_response_bytes_read":len(raw)}
    finally:sock.close()
def cleanup_permission_demo(host_binary,worker_binary,root):
    if os.name!="nt" and os.geteuid()==0:return {"status":"not_run_privileged_root_bypasses_chmod"}
    host=Host(host_binary,root,worker_binary);run,_=host.submit(config(seed=9001));ready=await_ready(root,run);directory=root/run;handle=None
    try:
        if K:
            K.CreateFileW.argtypes=[wt.LPCWSTR,wt.DWORD,wt.DWORD,ctypes.c_void_p,wt.DWORD,wt.DWORD,wt.HANDLE];K.CreateFileW.restype=wt.HANDLE
            handle=K.CreateFileW(str(directory/"worker.ready"),0x80000000,1,None,3,0x80,None)
            if handle==ctypes.c_void_p(-1).value:raise ctypes.WinError(ctypes.get_last_error())
        else:directory.chmod(0o555)
        status,value,_=host.request("DELETE","/v1/runs/"+run);assert status==500 and value["error"]["code"]=="cancellation_failed",(status,value)
        _,snapshot,_=host.request("GET","/v1/runs/"+run);assert snapshot["state"]=="cancelled" and snapshot["error"]["code"]=="cleanup_failed",snapshot
        status,next_value,_=host.request("POST","/v1/runs",config(seed=9001));assert status==507 and next_value["error"]["code"]=="cleanup_failed",(status,next_value)
        try:sock=connect(ready["address"])
        except OSError:closed=True
        else:sock.close();closed=False
        assert closed
        evidence={"status":"demonstrated","injection":"Windows open file denies DELETE sharing" if K else "POSIX run directory chmod0555","cancel_http_status":500,"cancel_snapshot":snapshot,"new_admission_http_status":status,"new_admission_error":next_value,"child_listener_closed_before_new_admission":closed,"directory_preserved_on_error":directory.exists()}
    finally:
        if handle is not None:K.CloseHandle(handle)
        elif directory.exists():directory.chmod(0o755)
    evidence["shutdown"]=host.stop(expect_success=False);assert evidence["shutdown"]["returncode"]!=0 and evidence["shutdown"]["stderr"],"historical cleanup failure must surface from explicit fallible shutdown"
    assert evidence["shutdown"]["root_remaining"]==[]
    return evidence
def main():
    parser=argparse.ArgumentParser();parser.add_argument("--output",type=pathlib.Path,required=True);parser.add_argument("--build",action="store_true");parser.add_argument("--load-seconds",type=int,default=60);args=parser.parse_args()
    repo=pathlib.Path(__file__).resolve().parents[3];crate=repo/"crates/experiment-service";output=args.output.resolve();output.parent.mkdir(parents=True,exist_ok=True)
    commands=[];build_commands=[["cargo","build","--release","--locked"],["cargo","build","--manifest-path",str(crate/"Cargo.toml"),"--release","--locked","--examples"]]
    if args.build:
        for command in build_commands:subprocess.run(command,cwd=repo,check=True);commands.append(command)
    suffix=".exe" if os.name=="nt" else "";host_binary=crate/("target/release/examples/probe_host"+suffix);worker_binary=crate/("target/release/examples/probe_worker"+suffix);numerical=repo/("target/release/fusion-lab"+suffix)
    result={"environment":{"platform":platform.platform(),"processor":platform.processor(),"logical_cpu_count":os.cpu_count(),"python":sys.version,"rustc":subprocess.check_output(["rustc","-Vv"],text=True),"revision":subprocess.check_output(["git","-c","safe.directory="+str(repo),"rev-parse","HEAD"],cwd=repo,text=True).strip()},"commands":commands,"binaries":{str(p.relative_to(repo)):sha(p) for p in [host_binary,worker_binary,numerical]},"limitations":["5ms requested sampling is not a hard bound on transient aggregate peaks. Windows retains per-process kernel lifetime handles; Linux VmHWM is read while each process remains alive.","Windows PrivateUsage/PeakPagefileUsage report commit; Linux reports private resident pages and writable virtual allocation separately, not an exact per-process private-commit equivalent.","One native host per report; background activity uncontrolled; native release-build end-to-end times include process startup, HTTP, artifact validation and I/O."]}
    inputs=[pathlib.Path(__file__).resolve(),crate/"Cargo.toml",crate/"Cargo.lock",repo/"Cargo.toml",repo/"Cargo.lock",*sorted((crate/"src").rglob("*.rs")),*sorted((crate/"examples").glob("*.rs")),*sorted((repo/"src").glob("*.rs"))]
    result["source_sha256"]={str(path.relative_to(repo)):sha(path) for path in inputs};result["reproduction_build_commands"]=build_commands
    dirty=subprocess.check_output(["git","-c","safe.directory="+str(repo),"status","--porcelain","--untracked-files=all","--","src","Cargo.toml","Cargo.lock","crates/experiment-service"],cwd=repo,text=True)
    result["environment"]["source_worktree_dirty"]=bool(dirty.strip());result["environment"]["source_worktree_status"]=dirty
    result["environment"]["revision_meaning"]="HEAD is context only; source_sha256 inventories the actual working-tree build inputs and binaries records the measured executables."
    try:
      with probe_directory() as temporary:
        base=pathlib.Path(temporary);root=base/"numerical";host=Host(host_binary,root,numerical);runs=[]
        for index in range(4):
            started=time.perf_counter();run,submit_latency=host.submit(config());done,polls=host.wait(run);assert done["state"]=="completed",done
            runs.append({"warmup":index==0,"id":run,"duration_seconds":3600,"seed":42,"elapsed_seconds":time.perf_counter()-started,"submit_latency_seconds":submit_latency,"status_latency_max_seconds":max(polls),"samples":done["samples"],"csv_bytes":(root/run/"samples.csv").stat().st_size,"csv_sha256":sha(root/run/"samples.csv")})
        result["slow_reader_eviction"]=slow_reader_eviction(host,runs[-1]["id"])
        result["maximum_duration"]={"runs":runs,"host":host.stop()};assert len({r["csv_sha256"] for r in runs})==1
        assert result["maximum_duration"]["host"]["metrics"]["sampled_aggregate_working_set_peak_bytes"]<512*MIB
        root=base/"controlled";host=Host(host_binary,root,worker_binary)
        run,_=host.submit(config(seed=9001));ready=await_ready(root,run);status,value,latency=host.request("POST","/v1/runs",config(seed=9001));assert status==409
        started=time.perf_counter();status,cancelled,_=host.request("DELETE","/v1/runs/"+run);assert status==200 and cancelled["state"]=="cancelled";cancel_seconds=time.perf_counter()-started
        try:s=connect(ready["address"])
        except OSError:child_closed=True
        else:s.close();child_closed=False
        assert child_closed
        result["cancellation"]={"seconds":cancel_seconds,"child_listener_closed":child_closed,"busy_status":409}
        status,value,_=host.request("POST","/v1/runs",config(seed=9001),{"Content-Type":"application/json","Origin":"https://untrusted.invalid"});assert status==403
        result["origin"]={"status":status,"value":value}
        for seed,title in [(9002,"noisy_pipes"),(9003,"artifact_overflow")]:
            started=time.perf_counter();run,_=host.submit(config(seed=seed));done,polls=host.wait(run);expected="worker_failed" if seed==9002 else "artifact_limit";assert done["state"]=="failed" and done["error"]["code"]==expected,done
            result[title]={"elapsed_seconds":time.perf_counter()-started,"status":done,"run_directory_exists":(root/run).exists(),"fixture_stdout_bytes":2*MIB if seed==9002 else 0,"fixture_stderr_bytes":2*MIB if seed==9002 else 0,"fixture_artifact_bytes_attempted":257*MIB if seed==9003 else 0};assert not (root/run).exists()
        run,_=host.submit(config(seed=9001));await_ready(root,run)
        clients=[]
        for _ in range(32):sock,status=start_sse(host.address,run);assert status==200;clients.append(sock)
        excess,status=start_sse(host.address,run);excess.close();assert status==429
        load_started=time.monotonic();cycles=0;rejections=[];ordinary_latencies=[]
        while time.monotonic()-load_started<args.load_seconds:
            excess,status=start_sse(host.address,run);excess.close();assert status==429
            blockers=[]
            for _ in range(32):
                sock=connect(host.address);sock.sendall(f"POST /v1/runs HTTP/1.1\r\nHost: {host.address}\r\nContent-Type: application/json\r\nContent-Length: 4096\r\n\r\n{{".encode());blockers.append(sock)
            time.sleep(.1)
            status,value,elapsed=host.request("GET","/v1/health");ordinary_latencies.append(elapsed);rejections.append(status);assert status==503,(status,value)
            for sock in blockers:sock.close()
            cycles+=1;time.sleep(.03)
        for sock in clients:sock.close()
        result["load"]={"requested_seconds":args.load_seconds,"elapsed_seconds":time.monotonic()-load_started,"slow_sse_clients":32,"excess_sse_status":429,"ordinary_concurrent_requests":33,"cycles":cycles,"ordinary_rejections":{str(code):rejections.count(code) for code in set(rejections)},"ordinary_latency_max_seconds":max(ordinary_latencies)}
        result["controlled_host"]=host.stop();assert result["controlled_host"]["metrics"]["sampled_aggregate_working_set_peak_bytes"]<512*MIB
        result["cleanup_permission_failure"]=cleanup_permission_demo(host_binary,worker_binary,base/"permission")
        sentinel_root=base/"sentinel";sentinel_root.mkdir();sentinel=sentinel_root/"keep.txt";sentinel.write_text("preserve this sentinel");rejected=subprocess.run([str(host_binary),str(sentinel_root),str(worker_binary)],input="shutdown\n",capture_output=True,text=True,timeout=10);assert rejected.returncode!=0 and sentinel.read_text()=="preserve this sentinel";result["nonempty_root"]={"returncode":rejected.returncode,"stderr":rejected.stderr,"sentinel_preserved":True}
        link=base/"linked";destination=base/"link-target";destination.mkdir()
        try:
            link.symlink_to(destination,target_is_directory=True);rejected=subprocess.run([str(host_binary),str(link),str(worker_binary)],input="shutdown\n",capture_output=True,text=True,timeout=10);assert rejected.returncode!=0;result["symlink_root"]={"status":"rejected","stderr":rejected.stderr}
        except OSError as error:result["symlink_root"]={"status":"not_run_privilege_blocker","error":repr(error)}
        result["status"]="native_demonstrations_passed_with_recorded_gaps"
    except Exception as error:
        result["status"]="failed";result["error"]=repr(error);raise
    finally:
        cleanup=[]
        for host in Host.instances:
            if host.stopped is None:
                try:cleanup.append(host.stop())
                except Exception as error:
                    cleanup.append({"error":repr(error),"pid":host.process.pid})
                    kill_host(host)
        if cleanup:result["emergency_cleanup"]=cleanup
        output.write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps({"status":result["status"],"output":str(output),"aggregate_peak_bytes":result["maximum_duration"]["host"]["metrics"]["sampled_aggregate_working_set_peak_bytes"]}))
if __name__=="__main__":main()
