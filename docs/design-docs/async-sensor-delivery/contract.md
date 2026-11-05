# Bounded asynchronous observation delivery

Issue #6 shipped in [PR #82](https://github.com/zblack54/fusion-lab/pull/82) at merge revision `54c574321ad3a4ae2c1ac31c66904adca200822a`. Independent review cleared the published source, six controlled local tests passed, and all 30 hosted checks completed without failure. This slice is a single-producer adapter that runs an observation consumer on an owned worker thread using the Rust standard library. It takes validated Observation values and leaves estimator invocation and persistence to callers.

- Start with an explicit capacity from 1 through 1024. The capacity counts waiting observations and excludes the one observation the worker may be processing.
- Offering never waits for consumer progress. Return Accepted, Full, or Closed. Full and Closed leave ownership of the offered observation with the caller. Process accepted observations in offer order.
- Finish closes admission, drains accepted observations, and joins the worker. Cancellation closes admission and discards waiting observations, then joins after any in-flight call returns. Dropping the owner takes the cancellation path. A consumer must eventually return from each call. Rust cannot interrupt arbitrary synchronous callbacks.
- A consumer error or panic closes admission and discards pending observations. Preserve the first consumer failure at join. Define which transitions win at the queue lock so tests can control the relevant interleavings.
- Keep sample and planned arrival ticks intact. The worker does not consult the clock or alter a logical arrival tick. Saturation is a runtime decision, so record offers and outcomes at the caller boundary if replay is required. Replaying recorded accepted inputs invokes the same consumer in FIFO order. A failed or cancelled run requires recording the processed prefix as well. No new numerical late-data policy is introduced. Existing synchronous replay applies its configured policy.
- No runtime dependency, service integration, truth access, or changes to existing numerical APIs.

## Executable acceptance cases

1. Block the first callback at a barrier, fill the waiting queue exactly to capacity, and prove the next offer returns Full with the original observation. Release and finish. Verify FIFO order, original timestamps and values, and the expected number of callbacks.
2. Cancel with a blocked callback and a full queue. Once closure is observable, release the callback and verify no waiting value starts. Cancel an idle worker and verify it joins.
3. Inject a typed callback failure and a callback panic separately. Prove shutdown returns the first failure and that later pending callbacks are not invoked.
4. Feed the same recorded accepted observations to the consumer under different controlled pacing. Compare the full ordered observations, including timestamps and value bits. Validate illegal capacities before launching a worker.
5. Run the existing locked Rust and strict documentation gates. Review bounded storage, lifecycle linearization, and absence of dependency changes separately.

## State and replay invariants

A single mutex serializes admission, dequeue, and closure. Dequeue claims the in-flight slot before releasing the lock. Never call the consumer under this mutex. On cancellation, a previously claimed callback can finish and no later callback can be claimed. The callback failure owns the result even if shutdown was requested earlier. Graceful finish gives pending work to the consumer unless it fails. A waiting worker wakes on both admission and closure. The owning handle holds the only join handle and joins exactly once.

The caller can retain accepted observations in offer order. The completion report gives successful callback count and whether a failing callback was attempted. That distinguishes the successful prefix, the attempted failing item, and never-started observations. Replay reproduces the successful input prefix and can separately include the failing item to reproduce the failure with a deterministic consumer. The transport does not promise identical runtime admission decisions from a second run. Tests compare recorded input replay, not scheduler timing.

## API and race cases

Start returns InvalidCapacity with the requested capacity or Spawn with the original I/O error. Offer checks closure before capacity and returns the original observation in Full or Closed. Finish and cancel consume the owner and return a report with successful count and a terminal reason of Drained, Cancelled, ConsumerError carrying the original error, or ConsumerPanicked. Drop performs cancellation and synchronous join without returning a report.

Publish callback failure while holding the queue lock. An offer that wins the lock before failure publication may be accepted and then discarded. A dequeue claim that precedes cancellation may invoke its callback after closure. A callback that fails after cancellation still determines the terminal reason.

Use a private test rendezvous at closure and dequeue claim. Control phases with channels and barriers. Timeouts detect hangs only. Cover admission before failure publication, failure after cancellation, and claim before cancellation. Verify Closed takes precedence over Full after publication.

The replay fixture records each offer ordinal, original observation bits and admission result, plus the terminal report. FIFO can include repeated IDs and sample ticks out of order. This record is separate from the existing Trace format. Replaying the attempted failing item is opt-in and assumes a fresh deterministic consumer in equivalent initial state.

The owner is thread-affine. This prevents the callback from moving the owner onto its own worker and attempting a self-join. The caller owns any playback record.

Callbacks must return or unwind and the captured consumer must have a non-panicking destructor. A callback panic publishes closure before the worker disposes of the caught payload. If that payload destructor unwinds, the worker contains it and retains the original terminal classification.

Run the controlled tests with `cargo test --locked --offline async_delivery`. They cover saturation, drain, cancellation after claim, active owner drop, failure publication, panic payload disposal and bitwise FIFO playback under two schedules.
