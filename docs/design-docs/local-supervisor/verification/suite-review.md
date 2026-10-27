# Suite efficacy and inventory review (VER-002)
Technical author inspection; independent package review remains with the parent.
Preapproval findings #33/#34 are repaired in the contract: applied host ACKs
require the matching evidence variant with exact-head checks/review/merge refs,
and restart requires explicit KnownStopped descendants rather than dead root alone.
TST-NEG-009..011 reject absent/wrong-kind/denied/stale host evidence;
TST-NEG-012 preserves the writer fence for dead-root Unknown/Alive descendants.
The successful host helper now emits kind-specific evidence rather than bare ACKs.
Dependencies are reviewed in dependency-review.md; both lockfiles and the bounded
metadata collector/inventory are part of the preapproval candidate.
Each test calls the production-owned seam and compares independent expected
state, typed error, file contents, subprocess sentinel or observed argv/env.
No fake fixture performs supervisor transitions. The fake-codex binary is an
external protocol peer: app-server returns controlled auth/quota replies and exec
records actual launch config, writes final output and can fail/block. Tests require
forbidden auth/quota conditions to produce no sentinel, and actual dispatch to
carry forced subscription config with API variables removed. Native timeout/
cancel tests await a controlled process sentinel, not arbitrary sleeps as an
oracle; bounded fixture polling is limited to observing launch.

All eleven required test classes have executable records. PERF is deterministic
caps/deduplication, not a flaky latency threshold. PROP exhausts percent values
0..100 and compares against an independent weekly-latch rule. CLI tests execute
the standalone application binary; file tests export/import the real bridge.
The native cycle test proves ACK progression before next dependent launch.
Claims about fsync/crash cuts, process trees, inherited handles and actual GitHub
effects are intentionally separate implementation verification obligations.

The initial diagnostic seam returns NotImplemented and the CLI exits 78; helper
recognizes only that exact marker as expected-red. Other errors, build failure,
missing tests, timeout or unexpected success are not expected-red. Removing
enqueue durability, ignoring owner generations, permitting API auth, omitting
environment stripping, reissuing uncertain host writes, or skipping exact-head
checks would violate explicit assertions once the seam is implemented.
Before approval the runner must compile/enumerate every named test and record one
normal expected-red result per configuration. Production mutation testing cannot
be claimed against an unimplemented seam; selective fault injection belongs to
implementation verification, especially gating and replay.

No scheduler, OS recurring task or GitHub write was started during design.
There is no test-owned private credential input, external path support crate,
network test dependency or hidden root manifest change. Only owned production
crate files and harness/package files are used.

## Revision2 scope efficacy and fixture migration

This author technical review does not approve or lock the package. All prior43
selectors/assertions remain in force; only actual-dispatch fixture paths, bound
scope content/digests and host evidence parameters change. Pure domain tests
retain toy scopes because they never invoke a worker. The fake peer is still a
real subprocess, not a supervisor implementation: it observes argv/env and the
actual approved_scope stdin string, and respects the requested read-only mode
in its own controlled file-writing behavior. It grants no runtime exception.

Twelve added entrypoint tests vary missing scope, altered raw bytes, task/issue/
checkout identity, strict JSON/version/type/mode, raw-file size, final encoded
prompt size, canonical control isolation and CLI legacy/read-only behavior.
Rejections require exact errors, full committed snapshot preservation, zero
worker launches, absent sentinel/native reservation and no PR intent. Positive
workspace dispatch checks the actual approved instruction marker/digest and
all extra/temp sandbox settings. Positive read-only SDK/CLI use same-path legacy
control only with exact sandbox_mode=read-only. No assertion is skipped, retried,
quarantined or made conditional on peer identity.

The encoded-prompt boundary uses independently constructed maximum-size scope
bytes dominated by JSON backslashes: raw<=32768 yet the MINIMUM required prompt
encoding exceeds65536. Its oracle passed before the diagnostic seam was reached.
This distinguishes encoded-byte limits from input character counts. The fixture
SHA256 helper first checks the independently known abc vector, not SUT output.

Symlink/reparse privileges, concurrent file growth and actual Windows sandbox
write denial are concrete VER009 obligations; fake-peer flags and documentation
are not claimed as OS-isolation proof. Current design execution only establishes
compilation/enumeration and normal expected-red reachability. Selective omission
of a scope gate or private-root check would violate explicit downstream errors
and absence-of-effects assertions after implementation. No production mutation
result or live inference success is claimed during this design revision.


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
