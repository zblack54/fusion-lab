# Issue40: exceptional shutdown ownership proposal

Status: PROPOSED_FOR_ORCHESTRATOR_REVIEW; NOT_IMPLEMENTED; NO_CONTRACT_LOCK_CHANGED

Approved digest remains `3862f6d829e8a739580454f0edfbfb6aff696007286c51dbcfd61bd6f7529f1f`. This is an implementation/failure-policy proposal, not an approved schema revision or authorization to change tests. Issue39's separate join-all repair is implemented and evidenced separately.

## Observed defect

`stop` sets the service stopping flag and calls cancel; a termination error returns early. `Service::finish` then signals HTTP shutdown and joins workers. The active monitor only polls normal completion; it does not retry the requested termination. A real child that remains alive therefore keeps the monitor Running forever even after a transient termination fault clears. Returning early or dropping the remaining handles would falsely release ownership; immediately releasing the root claim would also be wrong. Retrying the same unchanged consuming shutdown method in Drop does not solve a join that never reaches completion.

## Recommended policy

Keep the approved public API and state vocabulary. Explicit shutdown is **fallible completion of ownership**, not permission to abandon a child after an OS error:

1. Close admission monotonically. New runs cannot be accepted once stopping begins.
2. Attempt termination/reap of every owned child and record each failure without skipping remaining cleanup. Keep the first error for the final returned Result; subsequent errors can be aggregated without replacing it.
3. While stopping, the existing owning monitor performs termination recovery instead of only ordinary completion polling. A kill/reap failure preserves Child, running state, private directory, task handles and root claim. Record a stable cancellation/wait failure, increment sequence on a changed observable snapshot, and publish before closing subscribers. Do not publish a terminal state until exit/reap is confirmed.
4. Recovery uses the same concrete Child owner and does not launch another process. It retries lifecycle operations at a bounded cadence (initial50ms; backoff capped1second). This is termination recovery, not retrying a failed numerical job, repeating output creation or restarting the worker. No generic job retry is added.
5. Successful kill is not reaping. Track whether termination was requested and require an observed `try_wait -> Some` or successful `wait` before removing Child. If a reap operation returns Err, preserve the owner and retry confirmation; do not infer success from a kill acknowledgment or a closed listener.
6. Drain HTTP clients, then join all owned workers using issue39's aggregate repair. A monitor that faults cannot be allowed to bypass child cleanup: before terminal cleanup/root release, the shutdown owner must check an explicit ownership barrier. If any child remains unconfirmed, that owner continues the same recovery responsibility rather than discarding state. No background quarantine, hidden registry, detached thread or abandoned Child is introduced.
7. Only after all children are confirmed reaped and every task/file lease is observed may cleanup and claim release finish. Return the originally recorded failure even if recovery eventually succeeds; recovery must not turn a real lifecycle failure into Ok.

**Persistent OS failure means shutdown can remain pending while the caller's shutdown thread still owns the Service.** The root claim stays held and no competing writer is allowed. This is observable waiting for an exceptional resource condition, not a claim of bounded success on a broken kernel. Log the first pending lifecycle failure once and retain structured state; do not produce an unbounded stream of repeated errors. A stalled kernel call itself cannot be made interruptible by wrapping it in another thread without transferring ownership to that thread, so no universal kernel-return deadline is claimed.

The consuming `shutdown(self) -> io::Result<()>` remains coherent under this policy: it cannot return while an unreaped Child would be dropped. Once recovery is complete it returns Err containing the original failure. Calling it on a dedicated caller-owned thread is an explicit application decision; the service does not detach one on behalf of the caller. Dropping Service has the same fail-closed ownership requirement.

## Relationship to the approved contract

REQ-011 explicitly limits the five-second guarantee to a **healthy local host**. The healthy path still requires all existing tests, the strict resource cleanup bound, and native platform gates; the proposal does not extend that budget or change successful behavior.

CON-010 requires failures to be surfaced while ownership is retained until children, leases and tasks terminate. Keeping explicit completion pending under persistent termination/reap errors, and later returning the first error, satisfies that obligation more faithfully than an early return or detachment. CON-004 still requires reap before removal. Public running -> cancelled/failed/completed transitions and terminal immutability remain unchanged; no cancelling state, new endpoint or restart policy is proposed.

The architecture's prohibition on retrying process/numerical failures is retained: no failed experiment is rerun. The approver must explicitly accept that lifecycle recovery of the same owned process is distinct from job retry, and that the healthy-host qualifier allows exceptional pending shutdown. If that interpretation is accepted, this can be implemented as a reviewed repair with the existing digest and supplementary fault evidence. If the desired product policy is instead a guaranteed bounded error return, a public recoverable ownership/error handle or explicit quarantine manager would be necessary and would require a controlled formal contract/API revision. That alternative is not silently substituted here.

## Deterministic fault evidence before implementation

Use test-only injection at the actual private termination/reap boundary while keeping a real stalled child and real process ownership. A normal polling-only injection is insufficient for issue40.

* **Persistent kill rejection:** force the termination attempt to return a typed error before kill. Start consuming shutdown on a retained test-owned thread. Confirm the first failure is recorded, the same child remains alive, Run/Child ownership stays present, directory and root claim remain, and shutdown has neither returned early nor reported Ok. Clear the injection; automatic recovery must kill and reap without another shutdown call, join the original shutdown thread, clean files/release claim, and return the original Err.
* **Persistent reap-confirmation rejection:** allow the real kill, then force all relevant reap-confirmation attempts to return Err until cleared. A dead child/listener is not accepted as confirmed reaping. Verify the owned Child and root fence remain and no terminal snapshot/artifact is published; clear the fault and require actual reap confirmation, joins, cleanup and original Err return. Injection must cover any alternative try_wait/wait route so the test cannot accidentally bypass the intended persistent fault.
* **Transient termination error:** reject once, then allow the next attempt. Prove recovery is automatic and does not restart the numerical process or allocate a second run.
* **Worker panic plus outstanding child:** issue39 proves remaining handles are observed. Add ownership-barrier evidence that a failed monitor cannot permit cleanup/root release while an unreaped Child remains; the explicit shutdown owner must take responsibility.
* **Persistent-fault observation:** the test waits for an explicit fault-attempt notification, then verifies shutdown stays pending over a bounded test interval. It finally clears the fault and joins every fixture even when an assertion would fail. This proves no premature completion for that interval, not the mathematical behavior of an infinitely persistent kernel failure.
* **Healthy compatibility:** unchanged21-test suite, supplementary regressions, all native Windows/Linux Rust1.99/stable gates, strict slow-reader cleanup and release resource probe remain required. No ignored tests, altered fixtures, relaxed bound or fake successful cancellation.

Test harnesses retain their shutdown JoinHandle, completion channels, live-child evidence and fault controls. A finally/RAII fixture recovery path clears injected faults so an original-red run cannot strand a real process. Rate-bound recovery is inspected and attempt counters observed; repeated identical failures must not grow state/logs without bound.

## Evidence already available, not acceptance

The orchestrator reports native Windows/Linux Rust1.99/stable21approved tests plus9supplementary tests and root gates passed before issue39. Linux resource rerun after issue38 passed strict cleanup at3.99225949seconds and60second428cycle load; its raw proof is `implementation/linux-probe-issue38-passed.json`. Independently reviewed artifact12648896951/run40319744745 and all18source hashes matched GitHub source `d89c809` as reported by the capacity reviewer. These establish the healthy baseline, not issue40 fault behavior or final acceptance. The proposal does not overwrite that evidence or mark the service accepted.

## Review decision required

The orchestrator should approve or reject this exact retained-pending completion policy and its fault matrix before issue40 production edits. If approved, record authority/scope here or in the actual issue approval and implement in dependency order: test-only termination-boundary injection seam and deterministic original-red fault harness without repairing behavior; termination owner/stage model and stop/monitor recovery; final ownership barrier/error preservation; green fault regressions; full unchanged verification and exact-source conformance review. No locked artifact has been changed by this proposal.
