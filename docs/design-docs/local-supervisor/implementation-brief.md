# Implementation handoff: controlled revision2 reviewable draft

Status: approved / ai_ready=true; final-byte execution and new lock required.
Revision2 approver: main orchestrator /root,2026-10-26, after independent /root/capacity_baseline review and main exact-oracle/architecture review. Authority: owner delegation2026-10-25 and repository AGENTS.md. Approved44requirements/10constraints/56tests/5slices. Historical revision1 digest cannot authorize this changed surface.
Preparation authority: main/user delegation and issue41 comment6422273739 permit
design preparation; main separately approves this final reviewed scope. No actual inference before implemented gate or deployment is authorized.
Current scope:44requirements,10constraints,56executable tests,5ordered slices.
Native published baseline:9204bdbfeaa24c0923aee0ac3c33ebc293e1570d; service main
advanced independently without changing the preserved supervisor baseline.

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

Revision2 adds exactly pinned sha2=0.10.9 to production and independent harness fixtures, with serde_json1.0.149 direct harness use already in the production closure. Original assertions are preserved; native paths/scope helpers and new rejection/read-only cases are reviewed changes, not fixture exceptions. TASK005 scope gate precedes new native dispatch. Do not run real inference before this revision is independently approved, locked and implemented; live VER009 write-denial remains required before unattended acceptance.

Interface handoff: Pause::EvidenceMismatch is a new approved-surface enum variant
in the diagnostic lib. When restoring preserved full native revision-one lib,
reapply this variant before compiling; do not overwrite the new contract types.
Implement its durable fail-closed owner-disposition policy from architecture
and TST-NEG-018. Original native backup stays untouched.
