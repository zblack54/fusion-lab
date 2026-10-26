# Final Conformance Review

Status: NOT_ACCEPTED

Issue38 repair checkpoint: original clean-source Linux resource job failed at5.001802873seconds and is preserved as `linux-probe-issue38-failed.json`. The implementation now reserves one second for cleanup by starting socket write-idle expiry after four seconds. Rebuilt Windows proof in `windows-probe-issue38.json` passes strict cleanup at3.974633seconds plus the60second load and other native demonstrations. No approved criterion changed. Current-source Linux rerun and final platform conformance remain pending; see issue38-write-cleanup.md.

Approved external revision1 digest: `3862f6d829e8a739580454f0edfbfb6aff696007286c51dbcfd61bd6f7529f1f`. Authority is the owner's delegated orchestrator, as recorded in GitHub issue #5 and host approved-contracts.json. The implementation agent records evidence; final acceptance remains with that authority after independent exact-source review.

Current evidence spans a local working tree whose HEAD remains baseline `b274c6f6deb3e86a9fdfbd31b8b92ec59396b5e8` and the main orchestrator's published PR #32 snapshots. **HEAD alone does not identify the measured service.** `windows-probe-final.json` inventories actual production/probe source SHA-256 and release executable hashes and explicitly records dirty/untracked state. The orchestrator must match those bytes to its final published head. Latest reported publication is `bf856232cdc5fb49472507c1dd13d574b4bfffed`; current-source native CI and Linux resource probe are pending here. Earlier platform evidence is preserved separately in platform-evidence.md.

## Requirement audit

Every named test below passed in the trusted native Windows implementation runner; exact commands, selectors and fingerprints are in execution.json. Each requirement also requires VER-001 independent inspection, so these rows are **behavior proved natively; final conformance review pending**, rather than unconditional acceptance.

| Requirement | Executable evidence | Additional scope/evidence |
|---|---|---|
| REQ-001 accepted local runs | TST-ACC-001 | Real root CLI oracle, complete CSV and report. |
| REQ-002 validation before effects | TST-NEG-001, TST-ACC-002 | Missing, unknown and nested fields; no run/artifact side effects. |
| REQ-003 bounded request body | TST-BOUND-001, TST-ACC-002 |4097byte rejection and malformed JSON. |
| REQ-004 cancel and reap | TST-STATE-001, TST-ACC-003, TST-STATE-003 | Known-live child listener closes; retained ownership on injected poll failure; actual permission error surfaced. |
| REQ-005 single active reservation | TST-INV-001, TST-ACC-003 | Known-live overlap rejected; controlled load returns409 rather than queueing work. |
| REQ-006 terminal retention | TST-BEH-001, TST-ACC-004 | Old id/path eviction; real maximum-size artifacts retained within budget. |
| REQ-007 status and stable errors | TST-CONTRACT-001 | Actual failure snapshots and HTTP403/409/429/500/503/507 in native raw evidence. |
| REQ-008 artifact confinement | TST-ANTI-001, TST-ACC-004 | Traversal rejection and private run-owned files; exact-path review pending. |
| REQ-009 loopback | TST-STRUCT-001, TST-ACC-002 | Public nonloopback constructor rejected before root changes. |
| REQ-010 numerical compatibility | TST-PROP-001, TST-ACC-001 | Repeated seeds match, changed seed differs, all four maximum-run CSV hashes match. |
| REQ-011 bounded shutdown | TST-PERF-001, TST-ACC-003 | Native host shutdown milliseconds; transport task panic/deadline and runtime panic regressions pass. |
| REQ-012 bounded SSE | TST-BEH-002, TST-BOUND-003, TST-STATE-003, TST-NEG-002 | Same existing stream receives higher-sequence terminal state and closes;60second32+1 subscriber load. |
| REQ-013 artifact reservation | TST-BOUND-002, TST-BOUND-004, TST-ACC-004 | Count-independent byte eviction, real retained disk peak and deferred open-reader lease. |
| REQ-014 worker failure | TST-STATE-002, TST-ACC-003 | Actual noisy child exit and overflow paths fail without exposing partial artifacts. |

## Constraint audit

| Constraint | Evidence | Current outcome / remaining limit |
|---|---|---|
| CON-001 memory/queue bounds | VER-002 plan; windows-probe-final.json; resource-verification.md; immutable admission/SSE tests | Native measured peak109486080bytes resident,103378944private commit;389cycles/60.172seconds. Channel/queue construction inspected, not universal workload proof. Linux current-source probe pending. |
| CON-002 synchronous separation | Numerical source SHA inventory and unchanged root sources; VER-001 | No async/time/transport added to numerical loop. Independent exact-head conformance review pending. |
| CON-003 safe bounded ownership | Crate unsafe_code=forbid; worker adapter and streamed file ownership; VER-001 | No full numerical Experiment vector in service; bounded pipes/stream pipeline. Final diff inspection pending. |
| CON-004 reap before deletion | Known-live cancellation/shutdown tests, RAII/file-drop regressions, overflow evidence; VER-001 | Native actual child ownership demonstrated; full failure-path inspection pending. |
| CON-005 compiler/platform support | platform-evidence.md; VER-003 | Earlier four-way native test evidence exists. Current nine-regression/source matrix pending; do not substitute older head. |
| CON-006 exclusive root | Nonempty sentinel; Windows empty-junction JSON; exclusive-idle-root regression; VER-004 | Native Windows reparse and preserved-content rejection demonstrated. Linux symlink demonstration pending. |
| CON-007 browser origin | Actual403 plus early-return-before-admission source inspection; VER-004 | Native origin rejection demonstrated; independent no-launch review and Linux repetition pending. |
| CON-008 bounded pipes | Actual2MiB-per-pipe worker; direct real-OS-pipe production drain regression; VER-004 | Native no-deadlock and exact65536byte tails pass; current Linux execution pending. |
| CON-009 overflow termination | Actual257MiB attempted artifact -> artifact_limit; private directory removed; VER-004 | Native failure recorded after termination/reap code path; final reviewer must inspect ordering and Linux repetition. |
| CON-010 cleanup/task ownership | Windows deny-delete error demonstration; RAII and shutdown fault tests; injected monitor-wait error retaining real child; VER-004 | Native cleanup failure surfaced and new admission blocked. The wait test models try_wait failure; it does not induce a real kernel Child::wait failure after kill. Do not claim that stronger evidence. Independent acceptance judgment and Linux failure demonstrations pending. |

## Required gates and unresolved acceptance

Trusted structural validation with externally sourced digest passes zero errors/warnings. Trusted implementation execution passes all21 immutable tests and CMD-001 Clippy after the final server state-publication change. Native crate format/check/Clippy/docs pass; nine supplementary tests pass. These are actual execution results, not source-selector counts.

Resource/failure methodology, exact commands and all remaining limits are in resource-verification.md. Raw earlier/failed attempts remain preserved rather than overwritten as successful evidence. Linux private commit has no exact collector equivalent and must remain explicitly unavailable; Linux private resident/writable virtual observations are separate metrics.

Final acceptance requires independent VER-001 exact-byte audit, current-head VER-003 matrix, native Linux VER-002/004 results, and explicit treatment of partial wait-failure evidence. No requirement, constraint, platform or failure obligation is waived here. Keep Status: NOT_ACCEPTED until the orchestrator resolves those items against authoritative evidence and records its acceptance decision.
