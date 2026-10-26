# Implementation handoff: approved technical contract

Status: AI_READY after final execution refresh and external lock validation.
Approver: main orchestrator /root, separate from both design authors.
Authority: owner delegation recorded in repository AGENTS.md (2026-10-25),
continued explicitly by owner request to autonomously build supervisor/service
and maintain issues/PRs/docs. Scope: issue #4, 36 requirements, seven constraints,
43 executable tests, four ordered implementation slices; repairs #33/#34 and
concrete pinned dependency closure reviewed before approval on 2026-10-26.
Approval permits implementation under the immutable contract; it does not accept
the product or authorize deployment, paid APIs, credential extraction or a second
scheduler. VER-003–008 remain mandatory acceptance obligations.

1. Make all required tests pass.
2. Do not modify any test-owned file, test catalog, fixture, snapshot, test dependency, test command, or test lock.
3. If a test conflicts with another test or an approved requirement, stop and report the exact IDs and evidence instead of skipping, weakening, quarantining, or special-casing it.

Implementation is authorized within the approved scope. Modify only
crates/local-supervisor and mutable progress/evidence under implementation/.
The library/main files are diagnostic compile seams, not behavior examples.
Remove NotImplemented/seam exit behavior through real implementation; never add
fixture-specific branches. The fake Codex executable belongs only to the locked
harness and may not be called by production except through explicitly configured
test executable injection. It neither implements storage nor simulates the
supervisor.

Preserve root manifest/core/tests and other feature packages. Inventory:
the standalone harness manifest/lock/source/tests/fake Codex binary source are
test-owned; crates/local-supervisor manifest/lock/source are mutable production;
root rust-toolchain.toml and .cargo configuration presence are auto-inventoried.
No external fixtures or test-support path dependencies exist. Only one local path
dependency points at the declared production crate. The harness transitively
locks the reviewed production closure: rusqlite/bundled, serde/derive, serde_json
and getrandom/std. See verification/dependency-review.md and the inventory.
Both lockfiles are generated before approval. A later production dependency or
feature change that alters the test-owned harness lock requires controlled
design revision, refreshed evidence and renewed approval before implementation.

Entry after approval: obtain the externally approved digest and trusted validator
path from the real approval record, validate --require-lock --baseline-digest,
then implement one ready TASK and run focused tests plus the locked full runner
--mode implementation --baseline-digest. Do not read the expected digest from the
candidate test-lock. Required supplemental commands are CMD-001–CMD-006; root
repository gates are separately coordinated by the parent. Finish VER-003–008
and record actual evidence/review for every obligation, especially host GitHub
bridge and descendant ambiguity. Update implementation/progress.toml only.

Conflict report includes package/digest, task/revision, exact REQ/CON/TST/VER IDs,
minimal reproducer, commands/output and proposed design correction. Do not
change the locked surface without returning to design and approval.
