# Proposed controlled revision2: approved scope materialization (issue #41)

Status: PROPOSED, not implemented or approved. No actual inference should be
started until this boundary is approved and implemented. The current executable
native evidence uses only the explicit test-owned protocol peer.

## Universal fail-closed boundary

Before inference spawn, load state-root/approved-scopes/<task.scope_digest>.json
with a metadata size check and bounded Take(cap+1) read. Proposed cap is 32768
bytes UTF-8; reject missing, oversized, malformed, unknown-field, symlink/reparse
or mismatched input with no worker launch. Require a lowercase64hex digest and
compute SHA-256 over the exact file bytes, matching the task digest. Do not hash
only an unbound instruction substring. Use a reviewed pinned SHA-256 dependency
(proposed RustCrypto sha2=0.10.9), update BOTH production and immutable harness
locks/inventories via controlled revision before approval. Do not implement a
novel cryptographic hash or invoke a shell to calculate it.

The hash-bound version1 JSON contains schema_version, task_id, issue,
canonical_checkout, mode (read_only/workspace_write), instructions, non_goals,
and verification references. Match task_id/issue and canonical checkout before
account/inference work; instructions must be nonempty. Resolve the file under a
configured private host-owned location. Materialization is an authorized host
approval action, not something generated or accepted from worker output. Its
hash must be recorded in the external approval/issue record before enqueue.

Build a bounded stdin prompt containing the actual approved document, issue,
task, digest and current locally observed Git head; reject an oversized final
prompt before spawn. The model must receive the task content, not just an
unresolvable digest or assumed GitHub connector. Enforce read_only through the
Codex sandbox setting, independently of prompt prose. Keep global subscription,
no-deployment/no-paid-API and locked-test rules authoritative. This is a private
local approval boundary, not a claim of resistance to a malicious OS owner.

## Executable contract adjustments

Update every actual-dispatch fixture to materialize an independently hashed
approved document for its task and checkout. Pure state-machine tests may retain
toy scope identifiers because no worker is launched. This requires a clean
fixture boundary: either pass the fixture directory into task/submit helpers or
add a read-only SDK state-directory accessor, then have the TEST OWNED helper
write the document and compute its digest using its reviewed hash dependency.
Do not infer fake-peer identity or waive the gate for tests. Explicit known
SHA-256 vectors plus altered-byte cases independently check the hash boundary.

Add negative tests for absent scope, oversized scope, malformed/unknown field,
wrong digest (including changed instructions), wrong task/issue/checkout and
invalid mode. Each asserts no worker sentinel, zero worker launches, preserved
prior task state and no PR intent. Add a positive peer assertion that observed
stdin includes the approved instructions and actual digest, and a read_only
argv assertion. Preserve all existing state/lifecycle/assertion contracts.

After main review/approval: refresh expected-red evidence against diagnostic
seams, lock the revised surface with a NEW external approved digest, restore
current implementation, then implement gate and rerun all contracts. Until then,
issue #41 remains an explicit unattended-execution acceptance blocker; passing
current43 tests or publishing native progress does not authorize real inference.

## Required control isolation and exact fixture migration

For workspace_write, canonical state, bridge and approved-scope control roots
must all lie outside the writable checkout; reject aliases/reparse paths that
resolve inside it. A state parent containing a separate checkout child is valid
only when sandbox write roots are restricted to that child. The existing CLI
conflates state directory and checkout: controlled revision2 should add optional
--checkout PATH for step/run, preserve legacy same-path only for an enforced
read_only scope, and reject workspace_write without isolation before dispatch.
Also explicitly exclude temp-directory write allowances and any extra writable
roots, using configuration keys verified against the pinned CLI schema. Temp
sibling control files otherwise remain writable even when outside the checkout.
Do not bypass approvals, hooks or execpolicy rules. This prevents functional
worker forgery of host ACKs/queue, not a malicious OS-owner attack.

Precise locked fixture migration without runtime exceptions:

1. Keep Dir.0 as state root. Add checkout() returning Dir.0/checkout; runtime()
   initializes Git there. Put fake peer root at checkout/fake, preserving its
   existing root.parent() Git identity convention. Update sentinel assertions
   to that path without weakening their conditions. Bridge/scopes stay siblings
   outside checkout.
2. Keep pure domain task()/submit() toy scopes unchanged because those tests
   launch no worker. For actual-dispatch tests, initialize runtime first, then
   use a test-owned native_submit helper that writes the bounded approved JSON,
   independently computes SHA256, and submits the Task using that digest. Both
   dependent tasks in ACC007 need separate task-bound documents. No production
   authorize-scope shortcut or fake-peer gate exemption is necessary.
3. Give finish_host an explicit scope_digest parameter: pure domain callers
   pass abc; native callers pass the independently computed fixture digest.
   Preserve all PR/review/merge evidence assertions and expected transitions.
4. CLI-step fixture computes its scope before enqueue, passes that digest to
   --scope and --checkout to step. Pure enqueue/status/disable CLI tests remain
   unchanged. Every auth/quota negative gets an otherwise valid materialization
   so it still isolates its original forbidden launch condition.
5. The locked fake peer records whether stdin includes the known approved
   fixture instruction marker. Positive assertions require that marker plus
   the expected digest, task and actual head; read_only cases require the
   enforced sandbox argv/config. No supervisor transition is implemented by
   this peer.
6. New negative cases use independently valid documents, altering exactly one
   boundary (missing/size/schema/hash/identity/checkout/mode/control isolation).
   Assert full snapshot preservation, zero launches, absent worker sentinel
   and no PR intent. Test an oversized file growing after metadata inspection
   through the universal bounded reader where deterministically feasible.

The current native progress may be published as NOT_ACCEPTED, but real inference
and unattended execution remain blocked until this controlled revision is
approved, locked and implemented.

Primary configuration reference (opened during proposal review):
[OpenAI Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
defines sandbox_workspace_write.writable_roots as additional writable roots,
plus sandbox_workspace_write.exclude_tmpdir_env_var and
sandbox_workspace_write.exclude_slash_tmp booleans. Proposed dispatch fixes
additional roots to [] and both exclusion values to true. Documentation confirms
configuration names, not entitlement or working native Windows isolation. The
pinned CLI must accept those keys and a real attempted control-file write must
be denied before any OS-isolation claim. No model API fallback is involved.
