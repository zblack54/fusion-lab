# Implementation handoff
Status: approved for implementation under the recorded delegated scope.

1. Make all required tests pass.
2. Do not modify any test-owned file, test catalog, fixture, snapshot, test dependency, test command, or test lock.
3. If a test conflicts with another test or an approved requirement, stop and report the exact IDs and evidence instead of skipping, weakening, quarantining, or special-casing it.

Permitted implementation root: crates/experiment-service. The named production start seam is replaceable; every behavioral test initially reaches EXPERIMENT_SERVICE_NOT_IMPLEMENTED. Root Cargo/CLI/core changes require main orchestrator integration review; no design-stage production implementation. Preserve existing numerical behavior. Contract tests build the real root CLI and use raw TCP, so HTTP and numerical artifacts cannot be substituted by in-memory happy-path mocks.

External production input inventory: root src, Cargo.toml, Cargo.lock; mutable adapter crate. No external test-support paths or ambient fixtures. Harness owns serde_json dependency, Cargo.lock, Rust assertions and all selectors. Root CLI build is performed inside test setup from repository source.

Approval authority: user delegation recorded in AGENTS.md and originating chat on 2026-10-25 permits routine within-scope development and design handoff. Main orchestrator must independently review this exact draft and record approval scope/digest before lock; design agent has not approved it.

Required entry gates: trusted validate_design_package.py with --require-lock --baseline-digest from external approval, then verify_test_suite.py --mode implementation with that digest. Run all AGENTS.md gates and adapter/harness fmt, check, clippy, tests, docs. Finish VER-001 and VER-002 and per-obligation acceptance in implementation/acceptance.md. Conflict report: IDs, exact revision, failing command/output, contract contradiction, proposed revision; stop affected slice.

## Main orchestrator approval — 2026-10-25
Approver: main orchestrator, acting under the owner delegation in AGENTS.md and the originating chat instruction to continue autonomous development through issue5 with technical auto-approvals. Approved scope: this exact14-requirement/10-constraint/21-test local HTTP service contract and its four implementation slices. Main independently reviewed architecture, catalogs, process/ownership limits, ADR001 alternatives and actual compiled expected-red execution. Independent peer review identified missing live SSE transition coverage; TST-STATE-003 and TST-NEG-002 resolve it, and main inspected those exact tests. Approval is design authority, not implementation acceptance. All VER001–004 remain required. The external approved digest will be recorded on GitHub issue5 and in host work/approved-contracts.json immediately after final-byte execution and lock.

## Contract revision1 — production dependency lock
The initial harness lock was generated against a zero-dependency compile seam and could not resolve acceptedADR dependencies under --locked after implementation began. Main preserved the previous approved contract and production source outside the candidate, returned the package to draft, regenerated only the harness dependency lock against the concrete accepted production dependencies, and used the same diagnostic seam to re-establish executable expected-red evidence. No selectors, public behavior or acceptance criteria changed. A discovered test-efficacy gap was strengthened: non-loopback rejection now explicitly rejects the diagnostic not-implemented error, so that missing implementation cannot masquerade as a valid rejection. Main approves this dependency-only revision under the same owner delegation; the prior digest is superseded after final validation and a new external approval record.
