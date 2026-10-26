# Native resource and failure demonstrations

Status: NATIVE_WINDOWS_EVIDENCE_RECORDED; FINAL_ACCEPTANCE_PENDING

Latest repair checkpoint: issue38 subsequently exposed a strict Linux cleanup failure at5.001802873seconds. The failure is preserved, not waived. `issue38-write-cleanup.md` records the four-second writer budget with one-second cleanup reserve and rebuilt native Windows rerun at3.974633seconds, unchanged `<5` assertion, full gates and source hashes. Earlier native results below remain historical observations; Linux repair execution is pending.

This records implementation-stage VER-002/VER-004 evidence. It does not replace independent VER-001 review, final OS/toolchain evidence or the per-obligation acceptance audit. The approved contract digest remains `3862f6d829e8a739580454f0edfbfb6aff696007286c51dbcfd61bd6f7529f1f`.

## Reproduction and provenance

From the repository root, using Python3.12+:

```text
python crates/experiment-service/scripts/service_resource_probe.py --build --output docs/design-docs/experiment-service/implementation/windows-probe-final.json
python crates/experiment-service/scripts/windows_reparse_probe.py --output docs/design-docs/experiment-service/implementation/windows-reparse-probe.json
cargo test --manifest-path crates/experiment-service/Cargo.toml --locked --lib
```

The first command is portable to native Linux; use an output named `linux-probe.json`. It defaults to60seconds of load. `--load-seconds 1` is a short development check and is never presented as satisfying the60second criterion. `probe_host` embeds the real service and exposes only its address plus explicit stdin-triggered fallible shutdown. The real root release CLI supplies all numerical measurements. `probe_worker` separately supplies known-live, noisy and overflowing process fixtures; it is not numerical acceptance evidence.

The final Windows probe rebuilt release binaries after the shutdown repairs and captured source SHA-256 for production sources/manifests/locks, examples, the probe itself and the numerical sources, plus measured executable hashes. Its `environment.revision` is the local baseline HEAD, **not** a claim that uncommitted service code existed at that revision. `source_worktree_dirty=true`, exact status and `revision_meaning` make that distinction explicit. Match `source_sha256` and binary hashes when relating this evidence to a published commit. Subsequent independent CI should bind Linux results to its exact commit.

`windows-probe.json` and `windows-probe-current.json` are earlier observations; `windows-probe-permissions.json` is a short-load permission-path development check. `windows-probe-attempt-1.json` preserves a failed initial fixture attempt: the fixture's1MiB stack buffer exceeded the Windows process stack; it was moved to a heap buffer. This was a probe-fixture defect, not a service-performance result. Only `windows-probe-final.json` supplies the final rebuilt60second resource result below.

## VER-002: actual Windows observations

Reference host: Windows10 build19043, AMD64 family25/model33,16 logical CPUs, Python3.12.14, Rust1.99.0 stable, x86_64-pc-windows-msvc. One excluded warmup then three measured sequential3600second maneuver experiments, seed42, sigma1.0/0.15, process density0.1, outage[10,20). Each yields360001 samples and170623857 CSV bytes; all four CSV hashes match. The measured end-to-end interval includes HTTP admission, child startup, numerical compute, formatting, file I/O, artifact validation and final status polling; it is not pure numerical compute.

| Measurement | Observed result |
|---|---:|
| Excluded warmup |1.3664335s|
| Three measured runs |1.2961985 /1.2986255 /1.3132114s|
| Measured median |1.2986255s|
| Largest sampled aggregate service+worker working set |109486080bytes (~104.41MiB)|
| Largest sampled aggregate private commit |103378944bytes (~98.59MiB)|
| Artifact disk peak including retained runs |511873323bytes|
| Largest measured status-request latency |0.2003623s|
| Controlled cancellation through kill/reap response |0.0013837s|
| Real child listener closed after cancellation |yes|
| Numerical host explicit shutdown |0.0014831s|

Resident observations are below the512MiB engineering reservation on this host. Windows `GetProcessMemoryInfo` samples aggregate current working set and `PrivateUsage`, retaining child handles for lifetime `PeakWorkingSetSize` and `PeakPagefileUsage` evidence. Requested sampling interval is5ms, not a guaranteed real-time sample period or a proof that no transient aggregate spike occurred. This is not an OS-enforced memory cap. Disk and per-process metrics are raw in the JSON. Retention and cleanup left all successful-host roots empty.

The60.172second controlled load held32 SSE clients without reading, repeatedly attempted a33rd (429), and used32 incomplete ordinary request bodies plus a33rd request. All389 excess ordinary requests returned503; maximum rejection latency was0.0254759s. One worker remained active with zero queued jobs; the earlier conflicting POST returned409. Controlled aggregate resident peak was10440704bytes and private commit5386240bytes. Sixteen-event channel capacity and bounded request/connection ownership are additionally established by exact-code inspection; this state-only protocol does not manufacture enough state changes to fill sixteen events.

Linux collection is implemented using `/proc/<pid>/status` VmRSS/VmHWM and `smaps_rollup` Private_Clean+Private_Dirty, with writable virtual data/stack reported separately. Linux has no equivalent exact per-process Windows-private-commit counter in this collector, so that field is explicitly null rather than relabeled. Linux native probe execution/results remain pending in this checkpoint; no Windows result is presented as Linux evidence.

## VER-004: native failure and ownership evidence

| Obligation | Evidence and result |
|---|---|
| Nonempty root | Actual sentinel preserved; startup exits1 with InvalidInput before claiming/modifying it. |
| Symlink/reparse root | Normal symlink creation lacks Windows privilege1314; retained honestly in the main JSON. Separate `windows-reparse-probe.json` uses a real empty NTFS junction without elevation: root is a junction, startup rejects it, target remains empty. This covers a Windows reparse point rather than waiving it. |
| Malicious Origin | Actual mutation returns403 invalid_origin. The implementation's early return is before body validation/admission/worker launch; independent inspection supplies the no-launch argument. |
| Noisy stdout/stderr | Real fixture emits2097152bytes per pipe and exits7; service records failed worker_failed in0.0166773s without deadlock or retained directory. Supplementary real-OS-pipe unit test runs that fixture through the production drain function and verifies each retained tail is exactly65536bytes with expected contents. |
| Artifact overflow | Fixture attempts269484032bytes (257MiB); service records failed artifact_limit in0.0727339s, removes its private directory and releases admission after termination/reap. |
| Slow reader through eviction | Existing large CSV response remains unread with1024byte receive buffer; subsequent runs evict its id. Status is immediately404 while directory is retained. Without client reads or client close, write-idle cleanup removes the directory4.9708242s after eviction. |
| Real cleanup failure | Windows CreateFile denies delete sharing on a run-owned marker. Cancellation returns500, status records cancelled/cleanup_failed, new admission returns507 cleanup_failed, and the real child listener is already closed. Directory persists while locked. After releasing the handle, explicit shutdown cleans the directory but exits1 reporting the recorded OS32 cleanup failure. |
| Wait-error ownership | Supplementary test injects a typed try_wait error at a private, test-only process boundary while a real stalled child is alive. Status exposes worker_wait_failed with advancing sequence, second POST remains409, Child ownership remains present, and shutdown kills/reaps the original real child. This models the monitor error path; it does **not** force an actual kernel Child::wait failure after kill or prove all OS failures. Those stronger claims are not made. |
| Task joins/shutdown | Independent review of the orchestrator's lib/transport repairs found no actionable changed-path issue: shared drain observes task panics, aborts and joins on deadline, and explicit finish performs worker joins/cleanup/claim release even after runtime panic. Three focused production regressions pass. Final acceptance still belongs to the orchestrator's exact-diff review. |

Actual native production test count is now9: four ownership regressions, noisy-pipe and injected-wait tests, plus the orchestrator's three shutdown regressions. All9 passed with the production manifest. The trusted implementation runner separately re-executed all21 unchanged approved HTTP tests plus supplemental Clippy successfully after the final server error-publication change. No locked file was edited.

## Remaining acceptance boundaries

Publish these inputs and run native Linux resource/failure demonstrations against that exact source. Linux symlink and permission behavior must be observed there; root-run permission bypass, if encountered, must be recorded rather than claimed as a pass. The simulated wait error is partial failure-path evidence, not an actual kernel wait failure. The original four-way test matrix predates the newest supplementary regressions and shutdown fixes; final CI must verify current source. Sampling and one-host limits remain as above. This checkpoint does not mark the package ACCEPTED or close issue #5.
