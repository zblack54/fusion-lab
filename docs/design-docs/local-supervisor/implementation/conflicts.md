# Native descendant ownership proof boundary

Status: affected native dispatch slice stopped pending main disposition.
Approved contract digest: 82ad2fc0e5791995c39e56fdafeaace7021b5462a98230cae2107a68812c7842.

CON-001 and REQ-009/011 require no second checkout writer while an old worker
or descendant may survive. Architecture permits conservative AmbiguousProcess
pause when absence cannot be established, and forbids claiming cleanup merely
from a dead parent. Immutable TST-ACC-007 requires a successful native worker,
matching host ACKs, then an automatic next dependent native launch.

The pinned dependency closure has no safe Windows Job Object containment wrapper.
Own unsafe Rust is forbidden. A post-exit process inventory cannot prove absence
of orphaned grandchildren when their intermediate parent exited before sampling;
PID alone also does not establish creation identity. taskkill /T after a root
has exited is not descendant absence proof. No process-name/fixture branch,
unverified ancestry claim, or bypass is implemented to obtain a green test.

Current implementation has durable state/fences, validated host evidence, CLI
control and owned bounded subscription account/quota exchange. It deliberately
stops before positive inference-worker launch until the ownership proof mechanism
is resolved. This is incomplete implementation, not a waiver or acceptance.

Proposed resolution: main reviews a concrete safe Windows process-containment
boundary/dependency, including spawn/enrollment race and supervisor-crash behavior;
if it changes harness Cargo.lock, use a controlled design revision and renewed
external approval before editing that locked closure. Alternatively provide a
reviewed existing native capability that establishes the same guarantees without
changing dependencies. Conservative pause must remain the fallback when proof
is unavailable. The locked contract and tests have not been modified.
