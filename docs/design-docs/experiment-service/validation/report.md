# Validation report
Status: AI_READY

Target revision: b274c6f6deb3e86a9fdfbd31b8b92ec59396b5e8. This extends issue #5 with admission budgets measured under issue #10; it does not implement production behavior.

All P1–P7 design artifacts are complete for independent technical review. Counts: 4 acceptance scenarios, 14 requirements, 10 constraints, 1 production ownership module, 21 required executable tests, 4 implementation-stage non-test verification plans, 4 dependency-ordered slices, 1 proposed ADR, no unresolved product questions or granted waivers.

Test classes: acceptance4, behavioral2, boundary4, negative2, property1, state-transition3, contract1, performance1, anti-pattern1, structural1, invariant1. No test class is waived. Performance test is a bounded shutdown contract; resource characterization has separately specified workload, reference budget, repeats, host evidence and failure policy (VER-002), not an invented microbenchmark claim.

Native environment: Windows x86_64-pc-windows-msvc, stable Rust1.99.0. Harness Cargo.lock generated and retained; serde_json1.0.154 and its locked registry inputs reviewed for JSON parsing only. Production compile seam has zero dependencies. No machine-specific Cargo override or external fixture environment variable is required. CLI source, root manifest/lock and adapter are classified production inputs; every assertion and controlled worker source is test-owned. Linux and pinned-channel execution are not claimed: VER-003 explicitly requires the unchanged21 tests and gates on both OSes with1.99.0/stable before implementation acceptance.

Commands from repository root:

* `cargo generate-lockfile --manifest-path tests/experiment-service/Cargo.toml`: passed.
* `cargo fmt --manifest-path tests/experiment-service/Cargo.toml`: passed.
* `cargo fmt --manifest-path crates/experiment-service/Cargo.toml -- --check`: passed.
* Trusted `validate_design_package.py docs/design-docs/experiment-service`: VALID, zero errors; intentional warning that no test lock exists.
* Trusted `verify_test_suite.py docs/design-docs/experiment-service --mode design`: success; actual compiled enumeration of21 tests,21 expected-red results, zero unexpected failures. Supplemental harness Clippy with `-D warnings` passed. Final execution JSON is authoritative for exact commands, compiler, outputs, inputs and fingerprints.

Expected red is a normal libtest failure at the one production start seam, `EXPERIMENT_SERVICE_NOT_IMPLEMENTED`, never compilation failure, timeout or missing test. The real CLI is built in numerical-fixture setup. Acceptance compares complete HTTP CSV against a separately executed offline CLI oracle and analytical final truth; repeatability also checks seed variation. Test-owned controlled worker provides a known-live TCP listener and readiness marker, so admission cannot pass by waiting for the original run to complete and cancellation/shutdown cannot pass by accepting an already-completed result. It exits unsuccessfully for a distinct seed to exercise failure release. Slow SSE tests hold32 streams, reject33rd and preserve health/cancel capacity. Byte-budget eviction is separate from count retention.

Suite quality review removed two weak alternatives (accepting a second202 and accepting a cancellation409), added strict unknown/nested-field validation and independent seed variation, and made raw HTTP parsing accept SSE chunk framing without permitting chunked finite API responses. No production mutation results are claimed while implementation is absent. Non-test VER-001/002/003/004 cover exact-diff lifecycle, numerical separation, memory/load evidence, compiler/platform matrix, root/origin confinement, diagnostic pipe bounds, artifact overflow and Windows open-file cleanup. Their results remain implementation obligations, not preemptive successful reviews.

Residual risks: abrupt-process crash recovery is explicitly outside this feature and startup preserves a nonempty root for inspection; 512MiB is an engineering reservation until measured against the real service; OS hard memory enforcement is not claimed; native Linux execution and privilege-sensitive symlink/failure injection remain required acceptance evidence. Frontend and observation transport belong to #7/#6. No lock/digest or production acceptance is issued.

Independent review identified an SSE efficacy gap: terminal-only subscriptions could pass without publishing live changes. Added TST-STATE-003 with a controlled known-live child and an incremental HTTP/chunk/SSE decoder; the SAME existing connection must first deliver running, then a higher-sequence cancelled snapshot after a separate DELETE, then close within five seconds. Initial-snapshot-only implementations fail this test. TST-NEG-002 checks a Last-Event-ID above current sequence returns400 invalid_event_id without opening a stream. Both are linked reciprocally to REQ-012; the transition also covers REQ-004 and TASK-003. No architecture behavior was weakened.

Approval authority exists through AGENTS.md owner delegation, but main orchestrator must independently review and approve this exact draft before changing status, accepting ADR, refreshing evidence and locking. Keep spec.toml draft and ai_ready=false. The next implementation entry is trusted validator/runner with an externally recorded approved digest, as specified in implementation-brief.md.

Main independent review approved the exact scope after correcting live SSE test efficacy. Final metadata execution and lock verification are being performed; implementation must use the externally recorded digest, not its candidate lock. Empty optional harness directories and unused local .cargo directory were removed before inventory so an ordinary Git checkout preserves ownership markers.

Final approved-byte design execution passed21 expected-red tests and supplemental Clippy. Lock36entries; digest ce6a9ca67f2654188c0b52e6a9d24edd0262de9ac36aa3dadcd07466b4d667e6. Final validator --require-lock: VALID0errors0warnings. Approval baseline recorded externally on GitHub issue5 and host work/approved-contracts.json. Implementation acceptance remains NOT_ACCEPTED.

Dependency-lock revision1: previous externally approved digest ce6a9ca67f2654188c0b52e6a9d24edd0262de9ac36aa3dadcd07466b4d667e6 preserved in host contract history. New harness lock includes57 resolved packages compatible with Rust1.99 for accepted production dependencies. No behavioral test or normative requirement changed. Exact diagnostic-seam execution and new lock are required before production resumes.

Revision review also found that reconstructing a diagnostic Err made the non-loopback negative test pass without establishing rejection behavior. The test now explicitly fails on the named unimplemented error; selectors and normative behavior are unchanged, no assertion weakened. Original test bytes retained in host contract history.

Revision1 final validation passed21 expected-red and Clippy; lock digest 3862f6d829e8a739580454f0edfbfb6aff696007286c51dbcfd61bd6f7529f1f. Require-lock validator0errors0warnings. Production source restored after lock; implementation may resume against the new external approval baseline.
