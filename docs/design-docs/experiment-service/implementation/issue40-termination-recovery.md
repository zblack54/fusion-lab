> Current decision: [accepted conformance](acceptance.md) for code79cb06a, including final native Linux/Windows proof. Earlier pending statements below are historical checkpoint records.

# Issue40: retain ownership through termination failure

Status: NATIVE_FAULT_CONTRACT_AND_RESOURCE_PASS; CURRENT_PLATFORM_ACCEPTANCE_PENDING

The main orchestrator approved the exact policy in issue40 comment6415871166 under the existing healthy-host shutdown qualification and CON-004/CON-010. A separate reviewer agreed that this is a compatible technical repair, not a public API/state change or a numerical-job retry. External digest remains `3862f6d829e8a739580454f0edfbfb6aff696007286c51dbcfd61bd6f7529f1f`; no locked input was changed.

## Original-red evidence

Test-only faults were added at the actual private termination boundary around a real stalled child, without first repairing stop/monitor recovery. The injected persistent-reap mode performs the real kill but prevents both termination and ordinary polling from confirming reaping while enabled. The normal polling-only fault test from earlier work is not substituted for this evidence.

```text
cargo test --manifest-path crates/experiment-service/Cargo.toml --locked --lib termination_failure_ -- --nocapture
```

Before the repair, all three tests executed and failed exit101:

* One rejected kill left ordinary monitor polling running forever after the fault was gone.
* A persistent kill rejection did not recover the same shutdown call after clearing the fault.
* Persistent reap-confirmation rejection remained in ordinary `worker_wait_failed` polling rather than the requested `cancellation_failed` recovery path.

Each fixture clears its fault, manually finishes the real child if the broken implementation does not recover, and joins the caller-owned shutdown thread **before** asserting failure. The original-red run therefore does not rely on a leaked process or an unobserved hanging fixture.

## Implemented lifecycle

Run keeps private cancellation intent, acknowledged-kill state, first termination error, shared retry deadline and capped backoff. A successful kill is not sufficient to remove Child: actual try_wait/wait confirmation is required. Normal public running/terminal states and endpoints remain unchanged.

The initial stop closes admission monotonically, attempts all required cancellation cleanup and retains the first error. While cancellation is pending or the service is stopping, the monitor performs lifecycle recovery on the same owned Child instead of ordinary completion polling. Shared per-run deadlines start at50ms and back off to a1second cap, so monitor and caller do not independently hammer the kernel. Identical errors do not grow logs: the first is retained/logged once and stable failure state is published when it changes. No experiment is restarted.

Pipe readers are registered as top-level owned worker handles rather than nested handles held only by the monitor. After HTTP drain, the caller-held Service executes a child-ownership recovery barrier **before** potentially blocking reader joins. Thus a monitor panic cannot leave a live child keeping those pipes open forever without an owning recovery path. Issue39 joins every reader/monitor even after a panic. A final lease barrier precedes filesystem cleanup and root-claim release.

If kill/reap errors persist, explicit shutdown remains pending with the Service, Child, directory and root fence owned by its caller's thread. When the external failure clears, the same call recovers, confirms reaping, joins owned tasks, finishes cleanup and returns the originally recorded error. It does not report false success or transfer work to a hidden thread/registry. A persistently failing or blocked kernel is outside the healthy-host five-second return bound, as explicitly approved and documented in rustdoc/README.

## Current native fault matrix

| Case | Actual observed evidence |
|---|---|
| Persistent kill failure | Shutdown stays pending while the real child remains alive/owned and root claim exists; attempts remain rate bounded. Clear fault -> automatic recovery without another shutdown call, child listener closes, first kill error returned, claim released. |
| Persistent reap-confirmation failure | Real kill closes listener, but owned Child/running state/private directory and claim remain until fault is cleared. No samples/artifacts are published as completed; recovery confirms reaping and returns the original reap error. |
| One transient kill failure | The existing shutdown call recovers automatically and preserves its first error, rather than waiting forever in ordinary completion polling. |
| Owning monitor panic | A real monitor panics while termination is faulted; shutdown remains fenced/owned. Clearing the fault lets the caller's ownership barrier recover the real child and observe every top-level reader/monitor handle before release. |

These use controlled fault injection and real subprocesses; they do not claim to induce every real kernel error or mathematically prove an infinitely persistent failure. Bounded test observation ends by clearing the fault and joining all fixtures safely.

## Actual gates and scope

All14 supplementary production tests pass natively, including the four new cases, issue39 join-all, real noisy-pipe tails, RAII cancellation, root ownership and shutdown-panic/deadline regressions. The unchanged trusted implementation runner separately executes all21 approved HTTP tests and supplemental Clippy successfully. Production formatting, all-target compiler/Clippy with `-D warnings`, doctest target and documentation pass.

Read-only issue40 review reported no actionable finding in the shutdown/cancellation scope, tied to these source hashes: server `745a0642f9cddace18694f67149ac700c0abccf6a5755014c2d0ce27e36a89f7`, lib `20069057922a409990c680dff31542cb59da7da406f5b81ee64dc69a0dfb3e7a`, transport `9c337edd1b628e6f6c82427f823977b1c1cf4de89da64a0f4eb230b484ebc39c`, regressions `036edb9c195adb72f8f86e923399815fcf1af49ec1cc859fc5f408acf6552ed1`. Source writes are held on those bytes while release resource execution finishes.

The rebuilt Windows resource probe completed successfully in `windows-probe-issue40.json`, with actual source/probe/manifests and executable SHA-256 inventory. It retained the strict five-second cleanup assertion: observed deferred deletion was3.9797252seconds with immediate404 while the client remained unread/open. One excluded3600second warmup was1.3371796seconds; three measured runs were1.3383163/1.3863194/1.3499846seconds. Sampled aggregate resident peak was109518848bytes and private commit103395328bytes, below the512MiB engineering reservation; collector/sampling limitations are unchanged. The60.031second load completed387cycles with excess SSE429 and all excess ordinary requests503 (maximum0.0257515seconds). Controlled cancellation completed in0.001613seconds. Noisy pipes, overflow, malicious Origin, nonempty root, slow-reader and real permission-failure semantics all passed with recorded gaps/limits. No persistent-kernel-failure bounded-return claim is made.

The orchestrator published the source as PR #32 head `79cb06a5f893e6bbe0a8090e14819fcb4ea3cf81`; native Windows/Linux Rust1.99/stable and Linux resource reruns for that source remain required acceptance work. Earlier issue38 Linux proof is preserved, but it does not prove execution of this newer lifecycle code. Keep final Status: NOT_ACCEPTED.
