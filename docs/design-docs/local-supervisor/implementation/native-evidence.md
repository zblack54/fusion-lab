# Native supervisor implementation handoff

Status: implemented-unverified / NOT_ACCEPTED. Approved revision1 digest is
1eb471a28c9cc8c21c818202de7d6202d16ff39006a2c1bebffb3e86c0e33499.
Issue #41 approved-scope materialization and live VER003–008 evidence remain
acceptance obligations. No actual inference worker was used in this checkpoint;
Codex integration runs exercised only the explicitly configured test-owned peer.

## What changed and why

The diagnostic seam now has transactional SQLite state, fenced checkout ownership,
subscription account/quota exchange, a contained native child cycle, typed exact-head
host intents/ACKs and CLI control. A dead direct child is not proof of absent
writers. Windows jobs track membership through orphaned grandchildren; a zero
membership count plus direct Child completion is required before ownership release.

native.rs retains JobObject and Child through suspended creation, assignment and
resume, and uses bounded terminate/poll cleanup. Root kill-on-close containment
is installed before child creation. account.rs uses it for the app-server and a
shared30second handshake deadline; worker.rs also contains Git metadata helpers.
The driver holds a canonical checkout lock and durable dirty marker. After crash,
an unnamed old job cannot be reopened, so a surviving marker/attempt pauses
AmbiguousProcess instead of guessing cleanup from a PID or lease timeout.

All post-spawn worker/metadata failure paths perform retained cleanup or transfer
the owner into quarantine. Stream readers/writer are owned and joined after
membership0. JSON/frame/file reads have prechecks and Take(cap+1) race-safe bounds;
no raw account response or worker text is persisted in logs/bridge evidence.
Final worker identity, normal terminal event, exit status, schema and actual Git
head are checked before ImplementedUnverified. Real GitHub effects remain host
mediated and independently reviewed; worker claims never imply verification.

## Reproduce and actual evidence

From repository root on native Windows Rust1.99.0/stable:

~~~text
cargo test --manifest-path crates/local-supervisor/Cargo.toml --locked --offline --lib native::tests -- --test-threads=1 --nocapture
cargo clippy --manifest-path crates/local-supervisor/Cargo.toml --locked --all-targets -- -D warnings
~~~

The trusted implementation runner used external baseline1eb471a...33499,
--offline --timeout60:43/43 PASS for each CFG001/CFG002, all six supplemental
checks PASS, success=true and errors=[]. require-lock validator:0errors/0warnings.
See execution.json for full commands, binaries, revisions and outputs. Main
coordinates ordinary root gates and publishes/reviews the exact PR head.

Three supplementary native demonstrations ran actual child processes: orphan
leaf survives directparent completion until job termination; suspended children
are cleaned both before and after enrollment without executing their body;
killing an owned driver closes its root-job handle and terminates the orphaned
leaf, while restart preserves the durable ambiguity fence. Three additional
libtest entrypoints are only fixture process roles; their ordinary no-env returns
are not containment evidence. No fixture branch exists in production dispatch.

The first orphan demo's immediate TCP-close assertion failed AFTER job membership
was already0: endpoint teardown can lag process accounting. The revised observation
waits up to5seconds for that fixture endpoint to close, while retaining mandatory
zero-membership/direct-completion assertions. This is supplementary demonstration
correction, not a weakened locked contract. The final native demo run passed.

## Limits and next exercise

Do not run unattended real inference before issue #41's universal approved-scope
gate. Current Task supplies a digest, not the actual approved instructions; the
proposed controlled revision binds a private bounded scope document to that digest.
No fake-peer exception is proposed. Real subscription smoke, host effect crash
reconciliation, storage fault injection, full resource characterization and
read-only probe integration still require independent evidence. Passing the43
tests is not final conformance acceptance. No paid API, deployment, extra OS
scheduler, numerical core change or locked-contract edit occurred.

Exercise: run the orphan demo and explain why job.process_count()>0 after the
parent exits is the observation that a simple try_wait-based cleanup would miss.
Then inspect the crash demo: why does a newly opened supervisor still pause even
though the test externally observed the old tree terminate? It cannot reopen the
old unnamed job to independently establish that fact from persisted state.
