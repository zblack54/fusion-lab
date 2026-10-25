"""Windows CLI capacity baseline, using only Python's standard library.

Build the release binary first. Outputs are temporary and outside the repository;
only the requested JSON evidence persists. Timings include process startup, the
simulation, summary computation, CSV/report writing, and process shutdown.
"""

import argparse
import ctypes
from ctypes import wintypes
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile
import time


class ProcessMemoryCounters(ctypes.Structure):
    _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [
        (name, ctypes.c_size_t) for name in (
            "PeakWorkingSetSize", "WorkingSetSize", "QuotaPeakPagedPoolUsage",
            "QuotaPagedPoolUsage", "QuotaPeakNonPagedPoolUsage",
            "QuotaNonPagedPoolUsage", "PagefileUsage", "PeakPagefileUsage")
    ]


class MemoryStatus(ctypes.Structure):
    _fields_ = [("length", wintypes.DWORD), ("load", wintypes.DWORD)] + [
        (name, ctypes.c_ulonglong) for name in (
            "total_physical", "available_physical", "total_pagefile",
            "available_pagefile", "total_virtual", "available_virtual", "extended")
    ]


def measure(binary, duration, seed, output):
    command = [str(binary), "--duration", str(duration), "--seed", str(seed),
               "--scenario", "maneuver", "--output", str(output)]
    start = time.perf_counter()
    with subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
        stdout, stderr = process.communicate()
        elapsed = time.perf_counter() - start
        counters = ProcessMemoryCounters()
        counters.cb = ctypes.sizeof(counters)
        psapi = ctypes.WinDLL("psapi", use_last_error=True)
        get_memory = psapi.GetProcessMemoryInfo
        get_memory.argtypes = [wintypes.HANDLE, ctypes.POINTER(ProcessMemoryCounters), wintypes.DWORD]
        get_memory.restype = wintypes.BOOL
        if not get_memory(wintypes.HANDLE(int(process._handle)), ctypes.byref(counters), counters.cb):
            raise ctypes.WinError(ctypes.get_last_error())
        if process.returncode:
            raise RuntimeError(stderr.decode(errors="replace"))
    with output.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"duration_seconds": duration, "seed": seed, "elapsed_seconds": elapsed,
            "peak_working_set_bytes": counters.PeakWorkingSetSize,
            "csv_bytes": output.stat().st_size, "csv_sha256": digest,
            "samples": duration * 100 + 1}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/release/fusion-lab.exe"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repeats", type=int, default=3)
    args = parser.parse_args()
    if os.name != "nt":
        parser.error("This measurement implementation requires Windows")
    if args.repeats < 2:
        parser.error("At least two measured repeats are required")
    binary = args.binary.resolve(strict=True)
    root = Path(__file__).resolve().parent.parent
    if args.output.resolve().is_relative_to(root):
        parser.error("Evidence output must be outside the repository")
    import winreg
    with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r"HARDWARE\DESCRIPTION\System\CentralProcessor\0") as key:
        cpu = winreg.QueryValueEx(key, "ProcessorNameString")[0].strip()
    memory = MemoryStatus()
    memory.length = ctypes.sizeof(memory)
    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
    if not kernel32.GlobalMemoryStatusEx(ctypes.byref(memory)):
        raise ctypes.WinError(ctypes.get_last_error())
    metadata = {"recorded_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                "platform": platform.platform(), "cpu": cpu, "logical_cpus": os.cpu_count(),
                "physical_memory_bytes": memory.total_physical,
                "available_physical_memory_bytes_at_start": memory.available_physical,
                "python": platform.python_version(), "binary": str(binary),
                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                "cargo_lock_sha256": hashlib.sha256((root / "Cargo.lock").read_bytes()).hexdigest()}
    for command in (["rustc", "-Vv"], ["cargo", "-V"], ["git", "rev-parse", "HEAD"]):
        metadata[command[0]] = subprocess.check_output(command, cwd=root, text=True).strip()
    rows = []
    warmups = []
    with tempfile.TemporaryDirectory(prefix="fusion-capacity-") as temp:
        output = Path(temp) / "run.csv"
        for duration in (30, 300, 3600):
            warmups.append(measure(binary, duration, 42, output))
            for repeat in range(args.repeats):
                row = measure(binary, duration, 42, output)
                row["repeat"] = repeat + 1
                rows.append(row)
            group = [r for r in rows if r["duration_seconds"] == duration]
            if len({r["csv_sha256"] for r in group + [warmups[-1]]}) != 1:
                raise RuntimeError("Identical-seed artifacts differ")
            print(f"{duration}s: median={statistics.median(r['elapsed_seconds'] for r in group):.6f}s; "
                  f"peak={max(r['peak_working_set_bytes'] for r in group)} bytes", flush=True)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps({"metadata": metadata, "warmups": warmups, "runs": rows}, indent=2) + "\n")


if __name__ == "__main__":
    main()
