# Validation report: SPEC-SUPERVISOR

Status: AI_READY

The main orchestrator approved this technical contract after independent review.
Metadata is approved/ai_ready; execution refresh and locking are being completed.
No supervisor behavior or final implementation acceptance is claimed.
Draft history below preserves preapproval evidence and limitations.

## Scope and environment
Target revision: b274c6f6deb3e86a9fdfbd31b8b92ec59396b5e8.
Initial application: native Windows x86_64-pc-windows-msvc.
Edition 2024, rust-version 1.99, resolver 3, default features.
CFG-001: Rust 1.99.0; CFG-002: stable. Observed rustc release was 1.99.0,
commit b940084d7eb6a299eb4bfeb8e34901bc051e7ac4, LLVM 23.1.1.
The numerical core's Windows/Linux promise is unchanged.

The owner's pinned schema-3 tools were read and invoked from:
C:/Users/Admin/Documents/Codex/2026-10-14/i-x20/work/fusion-lab/.agents/skills/rust/spec-design/scripts/.
These trusted tools are outside the implementation-owned candidate.
The parent's tool regression run reported 22/23 passing, with one Windows
symlink fixture failing WinError1314 due absent symlink privilege. No tool or
test was weakened; this remains an environment limitation, separate from the
successful package validator/runner evidence.

## Phase outcomes
P1 intent/acceptance, P2 atomic requirements, P3 ownership/modules, P4 constraints,
P5 executable suite, P6 ADR/implementation slices, and P7 validation are complete
for technical draft readiness. Independent parent semantic review is still required.
There are no open blocking product questions and no waivers. Live implementation
verification is not misrepresented as a design-stage completed activity.

## Catalog and coverage
Seven acceptance scenarios, 36 active requirements, six modules, seven constraints,
43 required executable tests, eight verification records and four ordered slices.
All eleven required test classes have actual cataloged tests:

| Class | Count |
|---|---:|
| acceptance | 8 |
| behavioral | 3 |
| boundary | 4 |
| negative | 12 |
| property | 1 |
| state_transition | 4 |
| contract | 5 |
| performance | 1 |
| anti_pattern | 2 |
| structural | 1 |
| invariant | 2 |

The seven scenarios include eight acceptance tests because the native cycle and
CLI step are independent process entry points for ACC-007.
Every REQ has reciprocal links in traceability.toml and a planned module and
implementation slice. REQ-031–034/036 have explicit adapter/host verification
obligations where unit tests cannot establish live integration.
Every normative CON has reciprocal tests and/or a verification plan; CON-006 is
a design-scope non-goal inspected in VER-002. There are no orphan obligations,
unapproved test-class exclusions, or ignored catalog tests.

| Obligation group | Evidence |
|---|---|
| REQ-001–013 durable state/dependency/ownership/recovery/control | ACC/BEH/BOUND/NEG/INV/STATE tests; VER-002/004/006/008 |
| REQ-014–020 subscription/quota/worker boundary | ANTI/NEG/BOUND/PROP/CONTRACT/BEH tests; VER-001/002/003/008 |
| REQ-021–026 host intents/head/replay/probes | CONTRACT/NEG/INV/STATE/PERF/BEH tests; VER-002/006/007 |
| REQ-027–030 native CLI/core/seam | STRUCT/ACC/NEG/ANTI tests; VER-002/005/008 |
| REQ-031–036 real adapters/model/probes/privacy/continuous host lifecycle | Native ACC-007/008 plus VER-001–008 as cataloged |
| CON-001 process ambiguity/fence | INV-001, NEG-003, VER-004 |
| CON-002 subscription-only | ANTI-001, CONTRACT-005, NEG-008, VER-003 |
| CON-003 no own unsafe | VER-005 and enforced crate forbids |
| CON-004 resource caps | BOUND-001, PERF-001, VER-006 |
| CON-005 uncertain effects | INV-002, VER-007 |
| CON-006 no scheduler during design | VER-002 inspection |
| CON-007 numerical core unchanged | STRUCT-001, VER-005 |

## Actual executable evidence
The parent executed the trusted design runner in its native Admin context:
python verify_test_suite.py docs/design-docs/local-supervisor --mode design --offline --timeout 60.
The generated validation/design-execution.json reports success=true, errors=[],
and exactly 43 expected_red observations for each of CFG-001 and CFG-002:
86 normal expected-red executions, zero unexpected failures and zero unexpected
passes. The runner compiled and enumerated actual libtest binaries and required
one execution per selector; no source-name matching or zero-test success is used.

All CMD-001–006 recorded pass: application formatting, harness formatting,
application Clippy, harness Clippy with all targets and -D warnings, application
doctests and application documentation. Independent local cargo test --no-run
and cargo clippy --all-targets -- -D warnings also passed.
The helper diagnostic recognizes only Error::NotImplemented or executable exit78
plus SUPERVISOR_NOT_IMPLEMENTED as the intended missing production behavior.
A setup/build failure, other panic, wrong diagnostic, timeout, absent selector or
ignored test is not accepted as expected-red.

Earlier author attempts failed toolchain inspection in the restricted launcher
(home directory resolution) before the parent reran successfully. Early scaffold
compile/lint diagnostics were corrected before the recorded successful baseline;
they did not require weakening any behavioral assertion.

After final phase metadata and verified-empty benches/fixtures/compile directory
cleanup, the author reran the same trusted runner in the now-native Admin context:
success=true, 43 expected-red results per configuration, errors=[], and all six
supplemental gates passed. Structural validation again returned VALID with zero
errors and only the intentionally-unlocked warning. The evidence is current for
the final draft bytes. Approval metadata changes require another refresh before
locking. This report is outside the locked contract; no candidate digest here is
an approval baseline.

## Efficacy, measurement and integration limits
VER-001 preserves actual parent-observed ChatGPT/Plus auth, account/quota/model
handshake and successful subscription-only Luna CLI smoke. That standalone
worker reported no GitHub connector tools. The formal adapter therefore uses
typed durable host intents/ACKs, not presumed connector inheritance or secrets.

VER-002 documents technical oracle/fixture review. The fake Codex is a real
external child supporting controlled stdio account/quota responses, observed
argv/env, failures/deadlines, valid output and actual temporary Git commits.
It implements no supervisor behavior. Tests specify no-launch sentinel checks,
actual subscription configuration stripping, bounded child control, native CLI
step, durable file ACK ingestion and next dependent launch.
Because the production seam is unimplemented, the successful design run proves
normal expected-red reachability and compilation, not those future runtime
properties. Selective implementation faults and crash cuts must be exercised
during VER-003–008; no production mutation success is claimed here.

PERF-001 establishes a deterministic resource contract (caps/dedup), not a latency
claim. No throughput, tail-latency or memory improvement is claimed. VER-006
requires characterization in the actual implementation environment.

Residual risks: safe descendant ownership after supervisor death, durable SQLite/
file rename semantics, account protocol evolution, quota races, untrusted local
ACK manipulation outside the intended private-user trust boundary, and actual
GitHub effect reconciliation. These are explicit implementation-stage gates,
not waivers. Ambiguous survivors fail closed. Issue #4 stays open until the
integrated main-host demonstration and final conformance review pass.

## Readiness and handoff
Accepted technical choices: ADR-001 durable fenced native ownership, ADR-002
managed-subscription workers with host-mediated GitHub lifecycle.
They are subject to package review and do not approve this draft.
No root manifest, numerical behavior or unrelated feature files were changed.
Only diagnostic production compile seams exist, with no actual scheduler run.

Next: parent independently reviews the exact package; resolves any semantic
findings; records approval authority and scope; sets approved/ai_ready metadata;
reruns design evidence; then locks with trusted tools and verifies the external
approved digest. Implementation must verify that digest before each slice and
must not edit test-owned files. Final acceptance covers all test and non-test
obligations and remains separate from runner success.

## Preapproval repair closure (issues #33 and #34)

The takeover author completed the existing repairs without implementing supervisor behavior.
Applied ACKs now require typed per-kind evidence. NEG-009 rejects absent/wrong-kind
evidence atomically; NEG-010 rejects denied review, failed checks and empty
check/review references without state changes; NEG-011 rejects empty/nonhex merge
SHA and stale check or review references atomically. Positive host lifecycle
helpers supply concrete PR/review/check/merge evidence. NEG-012 keeps the writer
fence for a dead parent with unknown or alive descendants; the positive restart
case explicitly requires KnownStopped. The actual Windows process/host bridge
demonstrations remain implementation acceptance obligations.

Pinned production dependencies and both lockfiles were regenerated before approval.
The offline dependency inventory reports 28 registry packages, complete license
metadata and no declared Rust floor above 1.99. Its full inventory and limitations
are in verification/dependency-review.md and dependency-inventory.json.

Final takeover execution: cargo fmt for the harness passed; the trusted
verify_test_suite.py design runner with --offline --timeout 60 returned success=true,
errors=[], exactly 43 expected-red results per CFG-001/CFG-002 (86 total), and
all six CMD-001–006 gates passed. The trusted structural validator returned
VALID, zero errors and only the deliberate unlocked warning. No approval, lock
or implementation acceptance is issued. Main must independently review these
final contract bytes before any approval/lock operation.

## Main technical approval, 2026-10-26

Main reviewed typed per-kind host evidence, atomic negative ACK cases, confirmed-stopped versus unknown/alive descendant recovery, dependency inventory and boundary ownership, and all remaining implementation-stage plans. No unresolved product decision is invented. Delegated technical approval is recorded in implementation-brief.md. Metadata is now approved/ai_ready; final execution refresh and locking follow immediately. Earlier draft-readiness statements above are historical and do not establish acceptance.

## Final approval lock

Main refreshed final approved bytes:43 expected-red tests per configuration,86 total, errors=[], all six supplemental gates passed. Trusted lock and require-lock validation passed with zero errors and zero warnings. Approved digest: `82ad2fc0e5791995c39e56fdafeaace7021b5462a98230cae2107a68812c7842`. Baseline is recorded outside implementation ownership in the host approval registry and issue #4. This proves contract preservation, not implementation acceptance.
