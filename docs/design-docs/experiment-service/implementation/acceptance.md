# Experiment service conformance decision

Status: ACCEPTED

Accepted code: `79cb06a5f893e6bbe0a8090e14819fcb4ea3cf81` in [PR #32](https://github.com/zblack54/fusion-lab/pull/32). This decision accepts that verified code and its documented operating limits. The final evidence/documentation commit must pass its own checks before merge. No application deployment is included.

Approver: main orchestrator under the owner's delegated local engineering authority in AGENTS.md and explicit continuation request. A separate reviewer examined exact source, fault behavior, native artifacts and source hashes and found no remaining concrete conformance gap. Approved immutable contract: `3862f6d829e8a739580454f0edfbfb6aff696007286c51dbcfd61bd6f7529f1f`; trusted require-lock validation passes with zero errors and warnings. No requirement, test assertion or limit was waived.

## Evidence that determines acceptance

- [Service run40364583095](https://github.com/zblack54/fusion-lab/actions/runs/40364583095): Windows/Linux × Rust1.99.0/stable1.99.1, each executes all21 immutable tests and14 supplementary tests, zero ignored, plus adapter compiler/format/Clippy/doctest/rustdoc gates.
- [Root run40364582814](https://github.com/zblack54/fusion-lab/actions/runs/40364582814): all four numerical-core variants and strict documentation build passed. Roadmap validation run40364582731 also passed.
- `windows-probe-issue40.json`: rebuilt release warmup and three maximum3600-second runs, full60-second overload and native failure demonstrations, exact source/executable hashes. Baseline local HEAD is explicitly context only.
- `linux-probe-issue40.json`: native release workload/failures from artifact12669588042. Independent download ZIP SHA256 `2122dbd9d8013770014cbc36c4b20e712029e728c1414a318ecbfb1c9050e7d4` matches GitHub metadata; extracted JSON SHA256 `f7e0ed17308affad0db0522b22c69426fa21c3430c5c037d2224c02d801b446d` matches the repository evidence byte for byte.
- Separate reviewer recomputed all18 source hashes in each platform report against GitHub files at the accepted code head:18/18 Windows and18/18 Linux match. CI merge-checkout context is not mistaken for different code.

## Requirement decisions

Every row is accepted on the code and execution evidence above, with VER-001 source inspection in addition to the immutable test selectors in tests/tests.toml.

| Requirement | Accepted behavior/evidence |
|---|---|
| REQ-001 | Complete configuration launches the actual numerical CLI and publishes validated CSV/report artifacts. |
| REQ-002 | Missing/unknown/nested-invalid configuration rejects before admission, worker launch or output. |
| REQ-003 | Bounded body and read deadline reject oversized or malformed requests. |
| REQ-004 | Cancellation keeps the same Child owned until actual reap; real-child kill/reap fault cases preserve ownership and the first error. |
| REQ-005 | One active worker, no queue; concurrent admission returns409. |
| REQ-006 | Terminal count/byte retention evicts old identifiers and artifacts, respecting live readers. |
| REQ-007 | Stable lifecycle/error snapshots, including actual403/409/429/500/503/507 and persistent cancellation failure state. |
| REQ-008 | Confined private artifact paths and traversal rejection; no arbitrary file exposure. |
| REQ-009 | Loopback constructor/bind only; nonloopback rejects before effects. |
| REQ-010 | Root numerical implementation unchanged; real CLI oracle, repeated-seed equality and changed-seed variation pass. |
| REQ-011 | Healthy-host bounded shutdown; every connection/monitor/pipe handle observed, deadline aborts joined, cleanup/root release ordered after ownership barriers. Exceptional failure policy below is explicit. |
| REQ-012 |32 bounded SSE clients,16-event channels, monotonic sequence and terminal delivery/close; excess client429. |
| REQ-013 |256MiB active artifact reservation, count/byte eviction and deferred leased-file deletion; insufficient budget rejects. |
| REQ-014 | Worker exit/pipe/overflow failure is visible and never publishes partial successful artifacts. |

## Constraint decisions and verification methods

| Constraint | Accepted proof and boundary |
|---|---|
| CON-001 / VER-002 | Warmup+three maximum runs and60-second32SSE/33HTTP load on both native hosts. Windows sampled aggregateRSS109518848bytes/privatecommit103395328; LinuxRSS108834816/private resident103993344. RSS below512MiB reference budget. All387Windows/446Linux excess HTTP requests503; excessSSE429. Queue/capacity code inspected. |
| CON-002 / VER-001 | Synchronous numerical source and root Cargo.lock unchanged; transport owns no truth/filter loop. |
| CON-003 / VER-001 | Own Rust unsafe forbidden, bounded pipe tails and streamed file body; no retained complete numerical sample vector in service. |
| CON-004 / VER-001/004 | Actual direct-child completion/reaping precedes deletion/release. Kill acknowledgment or a closed socket alone never establishes reaping. |
| CON-005 / VER-003 | Native Windows/Linux × both declared toolchains execute21+14 tests and required gates at the accepted code. |
| CON-006 / VER-004 | Nonempty sentinels unchanged; Windows actual empty NTFS junction rejects without elevation; Linux symlink rejects. Earlier Windows symlink privilege blocker remains honestly recorded, covered by the actual reparse demonstration. Atomic root claim prevents two idle instances. |
| CON-007 / VER-004 | Malicious Origin403 plus inspected rejection before admission/launch, repeated natively. |
| CON-008 / VER-004 | Actual2MiB stdout and stderr drained without deadlock; real-pipe tests verify exact65536-byte retained tails. |
| CON-009 / VER-004 | Actual257MiB attempted artifact causes artifact_limit, terminated/reaped worker, no partial publication and private cleanup. |
| CON-010 / VER-001/004 | Real permission failures return500/507 and later explicit Err; all-worker/connection joins survive earlier panic. Persistent kill and all reap-confirmation faults retain Child/root ownership, bounded recovery of the same process, no new numerical run, original Err after recovery. |

## Failure and measurement limits are part of the accepted behavior

Healthy slow-reader cleanup remains strictly below5seconds: Windows3.9797252s and Linux3.998904693s, without client reads/close. The initial Linux5.001802873s failure is preserved; the fix reserved one second for cleanup by reducing writer idle time to4seconds. The assertion was not relaxed.

Under a persistently failing OS termination/reap operation, consuming shutdown can remain pending while its caller still owns Service, Child, tasks, files and root claim. Recovery retries the same child's lifecycle at shared50ms-to1s backoff, never reruns an experiment, and releases ownership only after confirmed reap/all joins. After recovery it returns the original failure. This policy was independently reviewed and approved in issue40 under REQ-011's healthy-host qualification. Controlled fault injection exercises the actual private termination boundary with real children; it does not claim every kernel fault was physically induced or infinite failures were mathematically tested.

Five-millisecond requested sampling can miss transient peaks; the512MiB allocation is an engineering reservation, not an OS hard limit. Linux private resident memory is explicitly different from Windows private commit; unavailable Linux commit is null. Measurements cover recorded hosts/workloads, not universal latency or performance guarantees. Artifacts are transient, POST retries create new runs, no remote authentication is provided, and crash recovery is outside this service version. These are documented scope boundaries, not undisclosed unfinished gates.

Historical attempts and repair notes remain in the neighboring evidence files. Their earlier pending statuses describe those checkpoints; this conformance decision and its exact source/run references are current.
