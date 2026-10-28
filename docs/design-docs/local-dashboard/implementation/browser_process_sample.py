"""Read-only native process observations; never drives the browser UI.

Lists Chromium renderer candidates for manual attribution; aggregate candidates
are not automatically represented as the selected tab's memory. No command line
or user profile path is persisted, only process type and renderer client ID.
"""
import ctypes
import ctypes.wintypes as wt
import argparse
import json
import re
import sys
import time

class Process(ctypes.Structure):
    _fields_=[('size',wt.DWORD),('usage',wt.DWORD),('pid',wt.DWORD),('heap',ctypes.c_size_t),('module',wt.DWORD),('threads',wt.DWORD),('parent',wt.DWORD),('priority',wt.LONG),('flags',wt.DWORD),('name',wt.WCHAR*260)]
class Memory(ctypes.Structure):
    _fields_=[('size',wt.DWORD),('faults',wt.DWORD)]+[(name,ctypes.c_size_t) for name in ['peak_rss','rss','peak_paged','paged','peak_nonpaged','nonpaged','pagefile','peak_pagefile','private']]
class Unicode(ctypes.Structure):
    _fields_=[('length',wt.USHORT),('maximum',wt.USHORT),('buffer',ctypes.c_void_p)]
K=ctypes.WinDLL('kernel32',use_last_error=True)
P=ctypes.WinDLL('psapi',use_last_error=True)
N=ctypes.WinDLL('ntdll',use_last_error=True)
K.CreateToolhelp32Snapshot.argtypes=[wt.DWORD,wt.DWORD];K.CreateToolhelp32Snapshot.restype=wt.HANDLE
K.Process32FirstW.argtypes=[wt.HANDLE,ctypes.POINTER(Process)];K.Process32NextW.argtypes=[wt.HANDLE,ctypes.POINTER(Process)]
K.OpenProcess.argtypes=[wt.DWORD,wt.BOOL,wt.DWORD];K.OpenProcess.restype=wt.HANDLE
K.CloseHandle.argtypes=[wt.HANDLE]
P.GetProcessMemoryInfo.argtypes=[wt.HANDLE,ctypes.POINTER(Memory),wt.DWORD]
N.NtQueryInformationProcess.argtypes=[wt.HANDLE,wt.ULONG,ctypes.c_void_p,wt.ULONG,ctypes.POINTER(wt.ULONG)];N.NtQueryInformationProcess.restype=wt.LONG
class Thread(ctypes.Structure):
    _fields_=[('size',wt.DWORD),('usage',wt.DWORD),('tid',wt.DWORD),('pid',wt.DWORD),('priority',wt.LONG),('delta',wt.LONG),('flags',wt.DWORD)]
K.Thread32First.argtypes=[wt.HANDLE,ctypes.POINTER(Thread)];K.Thread32Next.argtypes=[wt.HANDLE,ctypes.POINTER(Thread)]
K.OpenThread.argtypes=[wt.DWORD,wt.BOOL,wt.DWORD];K.OpenThread.restype=wt.HANDLE
K.GetExitCodeProcess.argtypes=[wt.HANDLE,ctypes.POINTER(wt.DWORD)]
K.GetProcessTimes.argtypes=[wt.HANDLE,ctypes.POINTER(wt.FILETIME),ctypes.POINTER(wt.FILETIME),ctypes.POINTER(wt.FILETIME),ctypes.POINTER(wt.FILETIME)]
if hasattr(K,'GetThreadDescription'):
    K.GetThreadDescription.argtypes=[wt.HANDLE,ctypes.POINTER(ctypes.c_void_p)];K.GetThreadDescription.restype=wt.LONG
K.LocalFree.argtypes=[ctypes.c_void_p];K.LocalFree.restype=ctypes.c_void_p

def threads(pid):
    if not hasattr(K,'GetThreadDescription'):return None
    snapshot=K.CreateToolhelp32Snapshot(4,0);entry=Thread();entry.size=ctypes.sizeof(entry);found={'total_threads':0,'query_successes':0,'query_failures':0,'named_threads':[],'query_error_codes':{}}
    def failure(code):
        found['query_failures']+=1
        if len(found['query_error_codes'])<8 or code in found['query_error_codes']:found['query_error_codes'][code]=found['query_error_codes'].get(code,0)+1
    try:
        more=K.Thread32First(snapshot,ctypes.byref(entry))
        while more:
            if entry.pid==pid:
                found['total_threads']+=1
                handle=K.OpenThread(0x800,False,entry.tid)
                if handle:
                    pointer=ctypes.c_void_p()
                    try:
                        status=K.GetThreadDescription(handle,ctypes.byref(pointer))
                        if status>=0 and pointer.value:
                            found['query_successes']+=1
                            name=ctypes.wstring_at(pointer.value)
                            if name and len(found['named_threads'])<128:found['named_threads'].append({'tid':entry.tid,'description':name[:128]})
                        else:failure(f'hresult:{status}')
                    finally:
                        if pointer.value:K.LocalFree(pointer)
                        K.CloseHandle(handle)
                else:failure(f'open:{ctypes.get_last_error()}')
            more=K.Thread32Next(snapshot,ctypes.byref(entry))
    finally:K.CloseHandle(snapshot)
    return found

def command(handle):
    length=wt.ULONG()
    N.NtQueryInformationProcess(handle,60,None,0,ctypes.byref(length))
    if not 16<=length.value<=131072:return ''
    data=ctypes.create_string_buffer(length.value)
    if N.NtQueryInformationProcess(handle,60,data,length,ctypes.byref(length))!=0:return ''
    text=Unicode.from_buffer(data)
    if text.buffer is None or text.buffer<ctypes.addressof(data) or text.buffer+text.length>ctypes.addressof(data)+len(data):return ''
    return ctypes.wstring_at(text.buffer,text.length//2)

def sample():
    snapshot=K.CreateToolhelp32Snapshot(2,0);entry=Process();entry.size=ctypes.sizeof(entry);result=[]
    try:
        more=K.Process32FirstW(snapshot,ctypes.byref(entry))
        while more:
            if any(name in entry.name.lower() for name in ['codex','chrome','msedge','chatgpt','electron','openai','browser','webview','cef','webkit','qtweb']):
                handle=K.OpenProcess(0x410,False,entry.pid)
                if handle:
                    try:
                        text=command(handle);kind=re.search(r'--type=([^ ]+)',text);client=re.search(r'--renderer-client-id=(\d+)',text)
                        memory=Memory();memory.size=ctypes.sizeof(memory)
                        if P.GetProcessMemoryInfo(handle,ctypes.byref(memory),ctypes.sizeof(memory)):
                            row={'pid':entry.pid,'parent':entry.parent,'name':entry.name,'type':kind.group(1) if kind else 'browser-or-other','renderer_client_id':client.group(1) if client else None,'rss_bytes':memory.rss,'peak_rss_bytes':memory.peak_rss,'private_bytes':memory.private}
                            if entry.name.lower()=='chatgpt.exe' and row['type']=='renderer':row['threads']=threads(entry.pid)
                            result.append(row)
                    finally:K.CloseHandle(handle)
            more=K.Process32NextW(snapshot,ctypes.byref(entry))
    finally:K.CloseHandle(snapshot)
    return {'sampled_utc_seconds':time.time(),'processes':result,'attribution':'candidate inventory only; selected tab PID not yet independently established'}
def monitor(pid,seconds,output):
    if not 1<=seconds<=600:raise ValueError('duration must be1..600seconds')
    handle=K.OpenProcess(0x410,False,pid)
    if not handle:raise ctypes.WinError(ctypes.get_last_error())
    created=wt.FILETIME();exited=wt.FILETIME();kernel=wt.FILETIME();user=wt.FILETIME()
    if not K.GetProcessTimes(handle,ctypes.byref(created),ctypes.byref(exited),ctypes.byref(kernel),ctypes.byref(user)):
        error=ctypes.WinError(ctypes.get_last_error());K.CloseHandle(handle);raise error
    report={'pid':pid,'creation_filetime':(created.dwHighDateTime<<32)|created.dwLowDateTime,'requested_seconds':seconds,'requested_interval_seconds':.005,'started_utc_seconds':time.time(),'method':'Held native process handle prevents PID-reuse confusion; lifetime RSS peaks plus periodic working set; no browser UI automation','ownership_threads_at_start':threads(pid),'samples':0,'sampled_rss_peak_bytes':0,'lifetime_rss_peak_bytes':0,'sampled_private_peak_bytes':0,'sampling_interval_max_seconds':0,'errors':[],'process_exited':False,'attribution':'Requires fresh-tab create/close and named dedicated-worker/main-renderer thread evidence; no shared browser/GPU charge claimed'}
    started=time.monotonic();previous=started
    try:
        while time.monotonic()-started<seconds:
            tick=time.monotonic();report['sampling_interval_max_seconds']=max(report['sampling_interval_max_seconds'],tick-previous);previous=tick
            code=wt.DWORD()
            if not K.GetExitCodeProcess(handle,ctypes.byref(code)):raise ctypes.WinError(ctypes.get_last_error())
            if code.value!=259:report['process_exited']=True;break
            counters=Memory();counters.size=ctypes.sizeof(counters)
            if not P.GetProcessMemoryInfo(handle,ctypes.byref(counters),ctypes.sizeof(counters)):raise ctypes.WinError(ctypes.get_last_error())
            report['samples']+=1;report['sampled_rss_peak_bytes']=max(report['sampled_rss_peak_bytes'],counters.rss);report['lifetime_rss_peak_bytes']=max(report['lifetime_rss_peak_bytes'],counters.peak_rss);report['sampled_private_peak_bytes']=max(report['sampled_private_peak_bytes'],counters.private)
            if report['samples']%200==0:
                with open(output,'w',encoding='utf8') as destination:json.dump(report,destination,indent=2)
            time.sleep(max(0,.005-(time.monotonic()-tick)))
    except Exception as error:
        report['errors'].append(repr(error));raise
    finally:
        report['elapsed_seconds']=time.monotonic()-started;report['ownership_threads_at_end']=threads(pid)
        K.CloseHandle(handle)
        with open(output,'w',encoding='utf8') as destination:json.dump(report,destination,indent=2)
    return report

if __name__=='__main__' and '--monitor' in sys.argv:
    parser=argparse.ArgumentParser();parser.add_argument('--monitor',type=int,required=True);parser.add_argument('--seconds',type=int,default=600);parser.add_argument('--output',required=True);args=parser.parse_args()
    print(json.dumps(monitor(args.monitor,args.seconds,args.output),indent=2))
elif __name__=='__main__':
    report=sample()
    if len(sys.argv)>2:
        with open(sys.argv[2],encoding='utf8') as source:previous=json.load(source)
        before={row['pid'] for row in previous['processes']}
        report['new_renderer_candidates']=[row for row in report['processes'] if row['type']=='renderer' and row['pid'] not in before]
        for row in report['new_renderer_candidates']:row['named_worker_renderer_threads']=threads(row['pid'])
        old_threads={thread['tid'] for row in previous['processes'] for thread in (row.get('threads') or {}).get('named_threads',[])}
        report['new_dedicated_workers']=[{'pid':row['pid'],'client':row['renderer_client_id'],'tid':thread['tid'],'description':thread['description']} for row in report['processes'] for thread in (row.get('threads') or {}).get('named_threads',[]) if thread['description']=='DedicatedWorker thread' and thread['tid'] not in old_threads]
    if len(sys.argv)>1:
        with open(sys.argv[1],'w',encoding='utf8') as output:json.dump(report,output,indent=2)
    print(json.dumps(report.get('new_renderer_candidates',{'output':sys.argv[1],'candidate_processes':len(report['processes'])} if len(sys.argv)>1 else report),indent=2))
