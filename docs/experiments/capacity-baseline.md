# CLI capacity baseline (issue #10)

Status: measurement verified; proposed service budget awaits service implementation
and load verification. This investigation addresses the measurement prerequisite
in [issue #10](https://github.com/zblack54/fusion-lab/issues/10). Admission and
backpressure acceptance criteria remain work for service issue #5.

## Reproduce

Run from the repository on Windows, using Python 3.12+ and the pinned Rust toolchain:

```text
cargo build --release --locked
python scripts/capacity_probe.py --output C:/outside-repository/capacity-baseline.json
```

The actual isolated build used this equivalent offline command with verified
vendored dependencies:

```text
cargo build --release --locked --offline --config C:/Users/Admin/Documents/Codex/2026-10-20/if-x20/work/cargo-offline.toml
C:/Users/Admin/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe scripts/capacity_probe.py --output C:/Users/Admin/Documents/Codex/2026-10-20/if-x20/outputs/capacity-baseline.json
```

Both commands passed. The probe uses only the Python standard library. It executes
the Rust release CLI with maneuver trajectory, seed 42, default noise/process
parameters and the default position outage [10,20) seconds. For each duration it
runs one excluded warmup, then three sequential measured child processes. CSV and
report files are overwritten inside an OS temporary directory outside the
repository and removed afterward. CSV hashes matched across warmup and repeats
at each duration. Hashing happens outside the timed interval.

Elapsed time uses `perf_counter` from before process launch through child exit
and stdout/stderr collection. It includes startup, simulation, summary, CSV/text
formatting and writing, and shutdown. It is **not pure compute time**. Pure compute
and startup/I/O contributions were not separately instrumented; subtracting them
from these observations would be unsupported. Buffered writes do not measure
durable storage flush latency. The repeats reuse filesystem/cache state.

Memory is Windows `GetProcessMemoryInfo` `PeakWorkingSetSize`, queried using the
still-open subprocess handle after exit. The kernel's lifetime high-water value
avoids missing short peaks through polling. This is child process resident working
set, not committed/private allocation, Python harness memory, filesystem cache,
or an aggregate service process budget. The implementation is Windows-specific.

## Environment and observations

Recorded 2026-10-25 20:15 UTC at source revision
`7fc67d689e2b9cbddb18bedff40ef4746dcafaf6`; no numerical/CLI source changes.
Windows 10 build 19043, AMD Ryzen 7 5800X, 16 logical CPUs;
34,266,181,632 physical RAM bytes, 14,091,276,288 available at probe start.
Python 3.12.14; rustc 1.99.0 (`b940084d7`, LLVM 23.1.1),
`x86_64-pc-windows-msvc`; cargo 1.99.0 (`5f94df478`).

Cargo.lock SHA-256:
`1ff60b850d3f5928fc13476575700400765d31f7deb3268171f66176fb6cf6f3`.
Release executable SHA-256:
`acbc241a95ce30a076222c2c17d458036bedd170aadea91642725dc0940184f8`.

| Simulated seconds | Samples | Measured CLI seconds (three repeats) | Median seconds | Max measured peak bytes | CSV bytes |
|---:|---:|---|---:|---:|---:|
| 30 | 3,001 | 0.0138852, 0.0140159, 0.0147666 | 0.0140159 | 4,407,296 | 1,354,041 |
| 300 | 30,001 | 0.1001603, 0.0952527, 0.0978608 | 0.0978608 | 11,726,848 | 13,894,308 |
| 3600 | 360,001 | 1.1079700, 1.1131556, 1.1532376 | 1.1131556 | 104,128,512 | 170,623,857 |

Warmup times were 0.0158962, 0.0952346, and 1.1468892 seconds;
warmup peaks were 4,411,392, 11,718,656, and 104,120,320 bytes.
The external JSON evidence records all unrounded observations and artifact hashes.
The maximum-duration measured peak is approximately 99.30 MiB. These observations
support the concern that retained samples cost memory; they establish no speedup,
memory reduction, calibrated tail latency, or general performance threshold.

## Conservative service design input

Start #5 with **one active simulation job**, duration at most 3600 seconds, and a
**256 MiB reservation per active job**. This rounds the observed maximum peak up
with more than 2.5x headroom, as an engineering allocation rather than a proven
worst-case bound. Reserve a separate **256 MiB for service/runtime overhead** and
initially allocate **512 MiB total** to the service. This is a proposed operating
budget, not a claim that working set equals enforceable process commitment. Measure
private commit and aggregate resident/committed service peaks before selecting
an OS/container hard limit. Account separately for host headroom and disk.

Use immediate busy/rejection behavior when the single active slot is occupied;
do not create an unbounded waiting queue. Only retain the active experiment and
release it after artifacts/response generation. Completed jobs should retain
artifact paths/metadata, not full experiments. Reserve at least **256 MiB artifact
space per active job**, since the maximum CSV measured approximately 162.72 MiB;
bound completed artifact retention separately. These are recommendations for #5,
not implemented admission or cleanup behavior in the offline API.

Do not raise concurrency from CPU count or extrapolate throughput from these
serial runs. Before increasing it, measure the actual service at maximum duration,
simultaneous completions, serialization, retained results, and representative
background load. Verify aggregate memory, disk cleanup, rejection/backpressure,
and cancellation. A future admission rule can reserve job memory before launch
and cap jobs by both the configured slot count and remaining measured budget.
Keep bounded full retention for the first service design; summary-only or
streaming output needs a separate specification if later evidence warrants it.

Limitations: one machine, seed, trajectory, local storage/cache state and three
repeats; no concurrent service, transport buffers, prolonged soak, pressure test,
or pure-compute benchmark. The existing numerical API and test suite are unchanged.
