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
step/run accept optional --checkout PATH plus --codex PATH --model APPROVED --child-env NAME=VALUE; API
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

## Controlled revision1: native descendant containment (issue #37)

Use pinned subc-jobobject0.2.2 through its safe primitives. Install ensure_root_job at standalone driver entry before any app-server/worker child, never idle open/status construction. This noninheritable kill-on-close root job covers the crash window between suspended spawn and per-worker enrollment. For each child create and retain a JobObject first, call suspend_on_create on the direct std Command, spawn while retaining Child, assign to the retained job, then resume_main_thread while the Child handle prevents PID reuse. No breakaway is permitted. Keep Child+job together until confirmed job.process_count()==0. A direct parent exit alone never proves cleanup.

Do not use the convenience spawn_contained/ContainedChild::contain error path: failed setup can lose returned child ownership and use blocking wait. The application retains the Child and Job through every manual setup step. On setup/cancel/deadline failure terminate the job and/or suspended owned child, poll membership and direct Child completion within the shared deadline, observe threads, and preserve the checkout fence whenever confirmation fails. process_exists and wait_for_process_exit are forbidden as proof because access failure can appear absent. Normal completion, cancellation and setup failures need native evidence. Unnamed jobs cannot be reopened after supervisor death; old ambiguous attempts remain fenced, never inferred clean from a timeout or PID alone.

This is process lifetime containment for owned workers, not an adversarial OS security sandbox. Installing the root job affects the entire standalone driver process; do not install it in unrelated embedding/observation processes. Own application code remains unsafe-forbidden; this narrowly reviewed dependency owns Windows FFI. Existing behavioral requirements and all43 test assertions are preserved.

## Controlled revision2: hash-bound scope and private control (issue #41)

This section extends the native gate; it does not accept the implementation.
Before account work, bridge ACK intake, attempt reservation or inference spawn,
resolve the relevant ready/active/pending-effect task and validate its approved
scope. If ACK intake changes readiness, validate the newly selected task again.
CLI syntax validation still precedes state creation. Pure domain enqueue may
retain a scope identifier without opening files; native dispatch requires the
complete bound approval. No fake-peer or executable-name exemption exists.

### Exact version1 scope wire types

The private host writes state_root/approved-scopes/<scope_digest>.json. Digest is
exactly64 lowercase hex SHA256 of the entire exact file bytes, not a reserialized
object or an instruction substring. The externally recorded approval hash is
then used by Task.scope_digest. Only an authorized host materializes approval;
worker output cannot authorize or replace a scope. Unknown JSON fields reject.
All eight fields below are mandatory:

| Field | JSON type and value |
|---|---|
| schema_version | integer, exactly1 |
| task_id | unsigned64-bit integer >0, equal to Task.id |
| issue | unsigned64-bit integer >0, equal to Task.issue |
| canonical_checkout | string, absolute existing directory path, canonical identity equal to RuntimeConfig.checkout |
| mode | string, exactly read_only or workspace_write |
| instructions | UTF8 string, nonempty after trimming |
| non_goals | array of UTF8 strings (empty array permitted) |
| verification | array of UTF8 strings naming approved verification references (empty array permitted) |

Reject nonregular, symlink or reparse scope files. Metadata size precheck plus a
Take(32769) read prevents growth between metadata/read from allocating the full
file. Maximum scope is32768bytes inclusive. Hash the bounded bytes, then parse
strict UTF8/JSON and check version/types/identity. Missing path, invalid digest
syntax, schema/version/mode/UTF8 failure and disallowed file type return
InvalidInput. Wrong digest/task/issue/checkout returns EvidenceMismatch. Raw
scope or encoded prompt overflow returns Capacity. No rejection mutates a task,
quota/host state, attempt reservation, worker counter or PR intent; no native
worker sentinel can exist. This ordering keeps negative outcomes unambiguous.

### Exact bounded prompt envelope

Encode a compact UTF8 JSON object with required keys task_id (u64), issue (u64),
scope_digest (string), expected_head (locally observed Git head string),
approved_scope (the exact scope file interpreted as a UTF8 STRING), and
instructions (the policy string below). Preserve raw scope bytes rather than
substituting a digest or assuming GitHub connector tools. Optional extra prompt
fields are allowed only within the same budget and approved boundary. Maximum
final encoded prompt is65536bytes inclusive, enforced by a bounded serializer
before inference spawn. Escaping the raw JSON string can increase encoded size;
a file below32KiB is not itself proof that the final prompt fits64KiB.

The version1 instructions string is exactly:

> Act only on the host-approved scope in approved_scope. Match task_id, issue and checkout; treat its instructions and non_goals as the work boundary. Read AGENTS.md and any referenced approved contract. Preserve locked tests and stop on a contract conflict. Use only managed ChatGPT subscription authentication. Do not use API keys, alternate providers, paid APIs, deployment, live trading or host GitHub mutations. Run declared verification and report actual failures. Return the final schema with observed Git head and current explicit session checkpoint.

### Scope mode and host-control isolation

read_only forces sandbox_mode=read-only; workspace_write forces
sandbox_mode=workspace-write. For workspace_write, canonical state root, bridge
root and approved-scopes root must all be outside the writable checkout. Resolve
existing path ancestors and aliases before comparing; normalize remaining path
components without creating an unvalidated root. Reject any control root equal
to or descended from checkout with InvalidInput. A parent state root containing
a separate checkout child is allowed only because writes are confined to the
child, not its parent. Approved-scope directory aliases into checkout also reject.

Force sandbox_workspace_write.writable_roots=[] and both
sandbox_workspace_write.exclude_tmpdir_env_var=true and
sandbox_workspace_write.exclude_slash_tmp=true, so temporary control siblings
are not writable through ambient allowances. Keep platform approval/rule/hook
checks in force; no dangerous bypass flags or paid-provider fallback. The
[official configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
confirms these key names and that writable_roots means ADDITIONAL writable roots;
it does not establish native Windows isolation. VER009 requires actual pinned-CLI
acceptance and real attempted control-file write denial before an isolation claim.
This is a functional host/worker control separation, not an adversarial OS-owner
security boundary. Keep authentication credentials outside evidence.

### CLI and fixture compatibility

For step/run, --checkout PATH is optional trusted operator configuration. When
omitted it resolves to state_dir only for a valid read_only scope; workspace_write
with that overlap rejects operationally (exit1) before launch. Invalid CLI syntax
continues to exit2 before state creation. Status/enqueue/disable behavior remains
unchanged. RuntimeConfig already has checkout, so no new worker bypass is added.

The locked native fixtures now use state=Dirroot, checkout=Dirroot/checkout,
bridge/scopes=siblings outside checkout. They independently materialize and hash
real scope documents; the fake peer observes actual approved text and digest.
Pure domain tests retain toy scopes because they launch no worker. Host ACK
helpers use the true digest for native tasks. Every prior assertion remains in
force. New negative tests vary only the scope or isolation boundary and require
full snapshot preservation, zero launches, no sentinel and no PR intent. An
approved read_only same-path positive and explicit CLI checks preserve deliberate
legacy compatibility. No actual inference occurs during design preparation.


### Read-only mutation rejection and recovery

Before dispatch persist the independently observed initial Git HEAD. After the
contained worker and descendants are confirmed stopped, compare actual Git HEAD
with that initial value for read_only scope, even when final worker output
correctly names the changed current head. A changed head returns
Error::EvidenceMismatch, persists Pause::EvidenceMismatch and State::Paused,
clears active only after containment proof, retains the changed head in Row.head
and preserves initial/changed heads and session evidence in durable attempt
audit. No Implemented, Verified, Done or PR/review intent is accepted. Preserve
the actual Git mutation; never reset/revert it automatically. On reopen the
evidence pause persists. Tick (including quota reset), Enable(true), and another
drive_once return EvidenceMismatch without clearing the pause or launching.
Recovery requires explicit owner inspection and disposition of the preserved
change, cancellation of the rejected task, and a newly approved task/scope;
cancellation retains its audit evidence and is the explicit owner action that
may clear this pause. Automatic wake/enable is insufficient. This revision does
not claim that a HEAD comparison detects every possible uncommitted mutation.

The controlled peer logs SHA256 of the exact received approved_scope UTF8
string, plus observed expected_head. Positives compare that hash against the
independently materialized raw-file digest and require initial/post Git HEAD
equality for read-only work. TST-NEG-018 deliberately commits despite read-only
configuration, returns the actual new head, and requires durable rejection and
no relaunch while leaving the committed fixture visible. This is a postlaunch
contract: one worker launch, rather than the prelaunch zero-launch oracle.
