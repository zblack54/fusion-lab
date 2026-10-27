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


## Revision2 scope checkpoint (NOT_ACCEPTED)

Source scope59a9850783fdb6f6118551ef42a5263d024f230d7db411c4be87392d302ecf7f and worker7340be8bfe0300f96a3f2d943786bd59596ad9d06c34a052ddf6aec7438c85c1. Issue45 opened-handle FILE_FLAG_OPEN_REPARSE_POINT check rejects a regular-file-to-directory-junction swap. Actual file-symlink swap could not be created: Windows1314 required privilege absent; this remains an acceptance gap, not a claimed execution. Issue46 native junction ancestor is canonicalized before ParentDir; missing suffix ParentDir, inside-checkout overlap and existing non-directory rejection pass without creating control directories. File grows from2bytes to65538bytes after open metadata: bounded32769-byte read returns Capacity. Four production regressions pass; all10 production unit entries pass. The older crash demo first failed InvalidInput because its toy scopeabc correctly failed the new universal gate; migrated only that mutable production fixture to SHA-bound approved read_only content and retained every containment assertion. No immutable harness change. Real sandbox write-denial, actual authenticated subscription smoke, host GitHub integration, remaining VER003-009 acceptance pending.


## Original live probe failure and issue54

Preserved outputs/supervisor-scope-smoke/result.json and read_only-1793056481/private-state/attempts/d038695ebc22e28f004ac878ae54bde9/final.json. The actual ChatGPT/Luna worker reported head=unobserved because its ordinary shell failed Windows CET startup, and explicitly reported that no private decoy append attempts ran. Controller returned EvidenceMismatch against independently observed actual Git HEAD. All private decoys and initial HEAD remained unchanged. This is a tool-shell capability failure, NOT sandbox-denial evidence or a successful read-only smoke. Durable cleanup event records job_members0, active/native_active are null, checkout marker absent, process inventory shows no local-supervisor.exe. No retry or fallback happened at exhausted primary quota. After natural reset and issue54 recording, main approved a new materialized scope explicitly requesting exec_command shell=cmd.exe/login=false; sandbox/approval settings remain unchanged. Original failure evidence remains intact.


## Issue54 live results and issue55 probe-command correction

Two actual managed ChatGPT/gpt-6-luna runs completed through the approved universal gate (raw outputs/supervisor-scope-smoke/result-issue54.json). read_only scope8b9d4b71d3125df0d6fe07dbc57ca2f1429650064b9676977877c0ddaed91af8 preserved observed initial/final HEAD f40141a577923f270dd9cf22c3c1093335e3dea3. Workspace run created the allowed checkout probe-allowed.txt. Both private decoy sets remained unchanged, but workers explicitly reported Windows redirection syntax errors, so these cases DO NOT establish private sandbox write denial. Workspace git add/commit failed .git/index.lock: Permission denied; sentinel remains untracked and HEAD unchanged. This is an unresolved native Git metadata/sandbox integration blocker; no sandbox weakening or host-commit substitution is approved. Both controllers exited, active/native_active cleared and synthetic local PR intents remain unconsumed; no remote writes or acceptance. Main recorded issue55 before another deterministic-command smoke. Host-only unquoted no-space echo redirection preflight succeeded, retained redirection-preflight.txt. Original and issue54 artifacts remain unchanged.


## Issue55 deterministic native sandbox evidence

Actual managed ChatGPT/gpt-6-luna read_only scope fc68943542b02ce5518a5baaab061885d9d8d64ad1eabf1064ad51db89c90800 and workspace_write scope fc1714f66dab37267b01525dbfa084fdd3bca4a265f5b0d6b575074025cef21d ran through the universal gate with the unchanged enforced extra-root/temp exclusions. Host preflight proved the exact unquoted no-space command syntax. In EACH mode the three actual append tool calls to private state/ACK/scope decoys exited1 with Access is denied.; independent post-run bytes remained UNCHANGED. Read-only initial/final HEAD remained0534684034571db6e77925196ecdd404198590f3. Workspace allowed checkout write exited0 and type confirmed ALLOWED; host confirmed sentinel exists. This scope intentionally did not commit or modify Git metadata while the preceding native index.lock permission blocker remains unresolved.

Raw outputs: outputs/supervisor-scope-smoke/result-issue55.json; exact materialized scope files retained under each disposable private-state/approved-scopes. tool-evidence-issue55.json preserves actual tool-call inputs and outputs from only the two known probe sessions, with source-log SHA256 provenance; it excludes authentication and unrelated session data. Sessions are01a17db7-c2f9-7111-b2d3-61cec7487560 and01a17db8-110e-76b0-89bc-78af28bb7982. cleanup-issue55.json records active/native_active null, cleanup job_members0, markers absent and no local-supervisor.exe. No synthetic local PR intent was sent to GitHub. Both step processes exited0 within420s configured outer bounds.

These observations support native control-file write isolation for the tested pinned CLI, paths and ordinary commands, plus actual readonly/head and allowed workspace write. They do not establish adversarial OS-owner security, file-symlink race behavior (still Windows1314 capability gap), a working Git commit lifecycle, remaining crash/storage/resource obligations, or real exact-head GitHub integration. Product acceptance remains NOT_ACCEPTED. No scope/locked source changed for these probes; no fallback, escalation, paid API, credit redemption, deployment or scheduler occurred.
