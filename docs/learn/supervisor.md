# How the development supervisor earns permission to continue

Status: implementation under review in [PR #36](https://github.com/zblack54/fusion-lab/pull/36). This page describes that branch. It does not claim that a replacement scheduler has been activated.

## The graph and the loops

The task graph describes prerequisites. A task can become ready only after its dependencies are Done; completing code alone is insufficient. The execution loop advances one ready task through implementation, checks, independent review, merge evidence and issue updates. Separate repair loops have bounded attempts. Account pauses and host reconciliation are conditions to wait for, not reasons to restart a numerical job.

| Boundary | Owner | Evidence required to cross it |
|---|---|---|
| Ready → running | Supervisor | Hash-bound approved scope, private host-control isolation, durable claim, exclusive checkout ownership, enabled controls and fresh managed-subscription quota |
| Running → implemented | Owned worker adapter | Normal completed turn, valid output schema, matching task and actual Git head, confirmed process cleanup |
| Implemented → verified/reviewed | Main host | Actual checks and independent review tied to the exact head |
| Reviewed → done | Host bridge | Matching applied merge acknowledgment, check/review references and merge SHA |
| Suggestion → ready work | Main orchestrator | Deduplicated issue, explicit triage, approved scope and dependencies |

SQLite stores claims and effects before they are acknowledged. The file bridge exports stable operation IDs; a lost reply means reconcile the existing operation, not repeat a GitHub write. Changing the head invalidates older checks. This is why the task graph, execution loop and evidence ledger are separate concepts.

## Follow the code

`crates/local-supervisor/src/storage.rs` owns durable transactions and writer fencing. `bridge.rs` validates typed host evidence and exports/imports atomic files. `account.rs` reads managed ChatGPT account/quota through an owned app-server child. `scope.rs` validates exact approved file bytes, task/issue/checkout identity, bounded prompt and control-root isolation. `worker.rs` drives one native cycle. `native.rs` owns Windows Job Object membership and cleanup proof. `main.rs` exposes local controls; it does not register an operating-system schedule.

The current host chat still coordinates GitHub actions and recovery. A standalone worker has no assumed GitHub connector access. Subscription credentials remain managed by Codex; the supervisor does not read credential files and strips provider/API-key environment overrides. Model selection is explicit; quota exhaustion never triggers a paid fallback or credit purchase.

## Why a dead parent is insufficient

A grandchild can remain alive after its parent exits. Process-name matching, elapsed lease time and a process-tree snapshot cannot prove that the checkout has no surviving writer. The Windows adapter therefore creates a suspended child, enrolls it in a retained Job Object, then resumes it. A root kill-on-close job covers the spawn/enrollment crash window. Cleanup requires both direct-child completion and zero job membership. Uncertain ownership keeps the checkout fenced.

This required a controlled design revision. The previous approved contract and partially implemented source were preserved outside the candidate. The exact safe dependency and error paths were reviewed, the unchanged 43-test contract was rerun against diagnostic seams, and a new external digest was approved before implementation resumed. The revision did not relax a failing assertion.

## What has actually been tested

The initial state/bridge implementation passed 37 of 43 contract tests per configuration. After the containment revision, a full native 43-test run passed using a test-owned external Codex peer, followed by additional cap/deadline and ownership audit changes. Final approved-byte implementation verification and acceptance must be read from the current PR evidence; an earlier passing run is not a promise about later edits.

The peer is a real subprocess that records argv/environment, speaks controlled account/quota protocol and makes a local test commit. It is not a real model response or proof of account entitlement. Separate native demonstrations exercise an orphaned grandchild, suspended setup cleanup and supervisor crash containment. Real subscription inference and the main-host GitHub bridge require their own integration evidence before acceptance.

## Approval must contain the actual task

A digest alone does not tell a worker what work is authorized. The host materializes a strict, bounded scope document with task and issue identity, canonical checkout, sandbox mode, instructions, non-goals and verification references. SHA256 binds the exact UTF8 file bytes; the same raw text reaches the worker prompt. The scope gate runs before account work, host ACK intake or attempt reservation, and repeats if ACK intake makes a different task ready.

Workspace writes are confined to the checkout; host state, ACK bridge and approved scopes remain outside it. Extra writable roots and ambient temporary-directory write allowances are disabled. Actual Windows write-denial demonstrations remain necessary: configuration names and fake-peer argv assertions do not prove platform enforcement. These controls do not claim isolation from a malicious operating-system owner.

Read-only work also compares initial and final actual Git HEAD. If a controlled peer commits despite that mode and returns its true changed head, the supervisor preserves the change and durable EvidenceMismatch pause, rejects implementation/PR acceptance and blocks wake/enable from relaunching. Owner inspection, cancellation and a newly approved scope are required. It never silently resets Git; HEAD comparison does not detect every uncommitted edit.

The reviewed revision-two implementation has a first complete trusted run of 56 passing tests on each compiler, with six supplemental gates passing. Native path/reparse/growth probes and actual subscription/sandbox/host integration remain separate acceptance evidence. The external approved digest is recorded in issue #4; passing tests alone do not accept the product.

## Reproduce and question the evidence

From the PR checkout, on native Windows with Rust 1.99 or the supported stable toolchain:

```text
cargo test --manifest-path tests/local-supervisor/Cargo.toml --locked -- --test-threads=1
cargo test --manifest-path crates/local-supervisor/Cargo.toml --locked
cargo clippy --manifest-path crates/local-supervisor/Cargo.toml --locked --all-targets -- -D warnings
```

The current revision-two immutable harness has 56 contract tests; the original 43 remain, with 13 scope/isolation and read-only mutation cases added. Supplementary native fixture entrypoints must be distinguished from actual demonstration tests; a six-test summary does not mean six independent process experiments. External approval digests are recorded in [issue #4](https://github.com/zblack54/fusion-lab/issues/4), with current outcomes under `docs/design-docs/local-supervisor/implementation/`.

Exercise: explain why an ACK saying only “applied” is insufficient to mark a task Done. Then identify the evidence that must change after a new commit, and why a dead worker parent cannot release the checkout fence on its own.
