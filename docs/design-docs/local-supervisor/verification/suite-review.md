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
