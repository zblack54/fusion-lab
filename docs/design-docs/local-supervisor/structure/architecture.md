# Architecture and executable contract

## Boundaries
One standalone crate owns the Windows application. The root Cargo manifest and
numerical code are outside its implementation roots. The seam defines the public
consumer types in lib.rs; it has no persistence or worker implementation.
Supervisor::open constructs a handle; apply performs one durable domain command;
snapshot observes committed state. plan_worker is inspectable configuration;
drive_once is the actual gated process adapter and is the only worker-dispatch
entry point. There is no public arbitrary run_worker bypass.

RuntimeConfig supplies the trusted Codex executable, checkout, bridge directory,
approved model, bounded worker deadline and extra child environment. It does not
grant authority to change authentication or provider. All API/provider credential
variables are removed after merging extra environment. Tests inject a real
test-owned fake Codex binary at this external executable boundary; production
does not inspect fixture flags or implement fixture cases.

The state module owns SQLite transactions and append-only attempt/effect evidence.
The account adapter owns a separate stdio app-server child, initialize handshake,
request IDs, framed response bounds, account type and quota reads. The worker
adapter owns the CLI child and completion/cancellation/cleanup. The bridge owns
atomic outbox/inbox serialization and ACK validation. The scheduler orchestrates
these boundaries; the CLI validates commands and presents committed state.

## Durable state and ownership
Use SQLite with foreign keys, integrity checks and synchronous durable commit
through pinned rusqlite 0.40.1 with bundled SQLite and default features disabled.
Pinned serde 1.0.228/derive and serde_json 1.0.149 own bounded wire encoding;
getrandom 0.3.4/std supplies OS entropy for owner tokens. Both production and
harness lockfiles include the reviewed dependency closure before approval.
See verification/dependency-review.md and dependency-inventory.json.
Record task, dependency, attempt, checkpoint, verification, suggestion, effect,
ACK, quota observation, scheduler control and cadence rows. No durable claim is
acknowledged before commit. Open/schema migration and corruption failures return
typed I/O/validation errors without dispatch. Store code/session evidence without
credentials. Retained task cap is 1024, suggestion cap 128, each event 64 KiB,
each ACK 1 MiB, and each worker log 16 MiB. Reject overflow before allocation or
truncate logs with an explicit truncation record; do not silently discard errors.

Use a native exclusive file lock plus generation and random owner token. The
lock belongs to the current scheduler/active attempt, not idle observation
handles; a second process cannot claim a live checkout. Canonicalize the checkout
identity and reject unapproved path alias/overlap. PID alone never proves process
identity. A persisted lease deadline alone never permits takeover. Reconcile
alive/creation identity and descendant state before requeue. If descendant
absence cannot be established after crash/timeout, keep AmbiguousProcess pause
and deny claim even if the root process is gone. Reconcile explicitly carries
DescendantStatus::KnownStopped, Unknown or Alive from the trusted native process
adapter; only a dead root plus KnownStopped permits requeue. Unknown and Alive
retain the active task/generation and reject a second claim as AmbiguousSurvivor.
This is a trusted adapter fact, never a worker-provided JSON assertion. Native
ownership evidence is reviewed in VER-004; a root-death boolean alone is not proof.
No unsafe application code is
permitted. Standard safe file locking is preferred. A safe Job Object wrapper
may strengthen ownership after dependency review; unproven tree cleanup must
remain fail closed rather than claimed as automatic recovery.

Submit validates positive IDs, issue/scope, already-known dependencies and no
self edge before commit. Choose the smallest ready ID with all dependencies
Done. Claim increments attempt count and generation and persists owner identity
before dispatch. Release changes Running back to Ready only after owned worker
completion/cleanup is known. Checkpoint/Implemented require matching owner and
generation. Head changes and failed verification invalidate earlier checks.
Repair has at most two additional attempts; now >= deadline rejects. Quota/auth
pauses do not consume repair budget.

## Gated native cycle
drive_once first establishes scheduler ownership, ingests validated host ACKs,
checks enabled/cancel intent, reconciles unfinished attempts, then reads fresh
managed ChatGPT account/quota using its owned app-server. It uses initialize
clientInfo name/version, initialized notify, account/read refreshToken false
and account/rateLimits/read. It rejects null/API/external-token/provider auth,
missing quota, exhausted allowance, unapproved model and malformed frames before
launch. Model catalog discovery is not entitlement proof.

A selected worker uses a direct argv vector, trusted checkout, stdin prompt
containing task_id/issue/scope/expected head, forced_login_method="chatgpt",
model_provider="openai", approved model/reasoning, --json, output schema and
--output-last-message. It preserves saved managed authentication without reading
credential files. Sandbox defaults workspace-write and platform approval gates;
no dangerous bypass flag. Permission/auth failures pause, not escalate.
Checkpoint session IDs are explicit; resume never uses --last.

Read JSONL events with a bounded frame/log budget, require a normal completed
turn plus exit 0 and valid final schema: task_id, status="implemented", head and
checkpoint. Match task identity; persist session/checkpoint. Nonzero exit,
missing/failed/interrupted terminal, malformed output or mismatched task cannot
be verified. Retain structured failure and exact status. The worker's claimed
head is checked against local Git before host PR creation; it is not trusted
verification evidence. Runtime model and effective reasoning selection are
persisted for every attempt, with no substitution. Default bounded worker
deadline is 30 minutes, configurable 1..3600000 ms. Account/process reads have
30-second deadlines. Graceful shutdown gets at most 5 seconds, then owned root
termination at most 5 seconds; uncertain descendants cause pause.

After implementation output the task remains ImplementedUnverified and exports
CreatePr intent, with no automatic check/merge claim. The host performs required
repository checks and independent review against the exact head before applied
RequestReview. Verify may record actual checks separately, but cannot synthesize
host review. Merge intent requires passed exact-head checks and applied approved
review. Applied Merge with matching head/URL/merge evidence makes the task Done;
UpdateIssue is then an independently reconciled effect. A failed check leaves
ImplementedUnverified. Cancel persists intent, signals the active driver, and
returns Cancelled when root completion is observed; ambiguous descendants retain
the checkout fence. Snapshot does not itself prove cleanup.

## CLI and host file bridge
CLI grammar is --state-dir PATH followed by:
enqueue --task U64 --issue U64 --scope DIGEST [--depends comma-separated IDs],
status, disable, enable, cancel --task U64, step, run, bridge-export, bridge-import.
step/run accept --codex PATH --model APPROVED --child-env NAME=VALUE; API
environment names remain stripped. These options configure trusted operator
input, not untrusted queue payloads. run repeats the same native cycle and
checks control intent between operations, with at most one active task.
It sleeps on host/quota/control conditions with a bounded wait (at most 1 second
for local control, due-time scheduling for quota). It exits on disabled,
cancellation, auth/quota pause, no ready scope, or unresolved survivor/host
condition; an external owner wake may invoke run again. It never registers OS
jobs itself. The current goal remains authoritative until approved migration.

Enqueue emits exactly {"accepted":ID}; disable emits {"enabled":false}.
status emits canonical JSON including tasks with lowercase state and IDs.
step emits canonical JSON with task (number/null), terminal, launches,
next_wake and checkpoint. Invalid commands exit 2 before state creation,
operational failures exit 1, successful control/status/handled awaiting_host
exit 0. Status and errors omit secret identifiers. Diagnostic seam exit 78 is
design-only and prohibited after implementation.

Bridge outbox is bridge/outbox/ID_WITH_COLONS_REPLACED_BY_UNDERSCORES.json.
Write temp in the same directory, flush, atomic rename, durable intent remains
authoritative if export fails. JSON schema version 1 includes id, task, kind,
expected_head, issue, approved scope/evidence references and bounded operation
payload. Required payload varies by kind: create_pr has branch/base/title/body
references; request_review has PR URL/check evidence and requested exact head;
merge has PR URL/approved review/check references; update_issue has issue and
accepted outcome references. Stable IDs are deduplicated; path collisions from
colon normalization are rejected, never overwrite another intent. No credential
material enters payloads. Untrusted ACKs are read from bridge/inbox/*.json,
bounded and validated before transactional ingestion.

ACK v1 requires schema_version,id,expected_head,observed_head,remote_url,outcome
and an evidence field. Evidence may be null only for uncertain/not_applied ACKs;
applied ACKs require the matching HostEvidence variant. Its JSON kind tag is
pr, review, merge or issue. Pr includes head, pr_url, issue and scope_digest,
which must match task/intent and remote_url. Review includes head, nonempty
checks_ref/review_ref and approved/checks_passed, both true. Merge includes head,
a 40-hex merge_sha and the identical checks_ref/review_ref previously accepted
for the current head. Issue includes the task's issue and nonempty completion_ref
after merge. Missing/wrong-kind/denied/stale evidence rejects with EvidenceMismatch
and no task or pending-intent mutation. Unknown ACK IDs return InvalidAck.
Host trust
comes from the configured private local bridge location and human-authorized
host chat; this is not a cryptographic service boundary. Applied RequestReview
means the host attests approved independent review and actual required checks,
not simply request delivery. Applied Merge attests checked head merged and
records merge evidence. Both host and consumer validate ID/head/kind and expected
stage. Import consumes/archive files only after commit; duplicate identical ACK
is idempotent, conflicting/unknown/malformed ACK rejects without task mutation.
Uncertain records pause AwaitingHost until read-only host reconciliation yields
applied/not_applied. Reexporting the same ID is a read/reconciliation request,
not authority for a second write. The host always fetches current PR/head/state
before retrying uncertain effects.

## Probe and cadence
Suggestions are stable-key deduplicated read-only findings with issue references;
they never become ready work until accepted with scope/dependencies. Count
verified slice events durably; each fifth or explicit major architecture event
creates one read-only probe intent keyed by event sequence. This dispatch cannot
bypass single writer, subscription, quota, deadline or host policy.
Preserve the weekly latch across reopens and allowance resets. There is no
automatic credit purchase or reset redemption. Restored allowance does not
guarantee future quota because other account work can consume it.

## Verification separation
Controlled tests exercise external executable/file boundaries and state contracts.
VER-003/004/006/007/008 cover actual Windows ownership, crash cuts, auth/CLI,
resource characterization and main-chat GitHub integration. No mocked success
is acceptance for those obligations. The real GitHub host bridge must be
demonstrated before issue #4 completion. A standalone authenticated GitHub client
is a future alternative, not a prerequisite or assumed current capability.
