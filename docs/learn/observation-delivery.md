# Timestamped observation delivery and replay

Status: implemented and independently reviewed through the codec slice; final example, resource and native conformance are in progress. Tracking: [issue #69](https://github.com/zblack54/fusion-lab/issues/69), [draft PR #71](https://github.com/zblack54/fusion-lab/pull/71), v0.3 delivery milestone. No deployment or completed asynchronous adapter is claimed.

A measurement has two clocks: when the sensor sampled it and when it becomes available. Previously the simulation consumed observations at their sample tick. The new observation-only API records sample time, planned arrival time and scheduled identity separately, so a delayed measurement cannot silently become a new measurement of the present.

One tick is 10 ms. Position samples occur every 20 ticks and velocity samples every five. An identity is the sensor plus its scheduled sequence number; absent measurements leave gaps. A one-second source includes both endpoints: six position and 21 velocity slots, or 27 observations. A 3600-second source has 90002 slots.

The first policy is deliberately conservative: OnTimeOnly accepts an observation only when its sample and arrival ticks match. A late arrival is logged and does not update the filter, position hold or velocity integration. This is a reproducible baseline, not a claim that dropping delayed data is the best estimator. [History correction #22](https://github.com/zblack54/fusion-lab/issues/22) remains separate.

At each tick, due observations drain before newly sampled observations are admitted. Scripted loss is decided first; zero-delay data bypasses the waiting slots. A full queue drops the newest offered future observation. Source-order decisions and arrival-order consumption are separate records. Data still waiting at the final tick receives an explicit horizon outcome. The fixed capacity bounds this virtual queue; it is not an operating-system memory quota or a real asynchronous channel.

The numerical model is unchanged: state is position and velocity, prediction uses 10 ms steps and white-acceleration process noise, and simultaneous position updates precede velocity. Both simple baselines receive the same accepted subset as the filter. Replay owns no truth, source configuration or seed: projection copies only timestamps and present measurements, then ends the source borrow.

For a scalar illustration, a zero prior mean with variance 25 and a position observation 10 with variance 25 has gain 25/(25+25) = 1/2, giving mathematical posterior mean 5. The unchanged Cholesky computation returns about 4.999999999999999 on the observed target. [Issue #72](https://github.com/zblack54/fusion-lab/issues/72) preserves this counterexample: two exact-rational assertions were corrected through a reviewed contract revision, while the rational expected values and all bitwise legacy/signed-zero checks stayed intact. Rounding production to satisfy the test would have changed the reference behavior.

Follow the data through src/delivery.rs: Observation and ReplayModel validate inputs; Trace owns and orders observations; replay_delivery owns queue decisions and numerical state; the codecs produce capped canonical TSV. Fields are private with read-only accessors. The [API and format contract](../design-docs/observation-delivery/structure/api-and-format.md) defines error variants, byte limits and ordering. Numerical failures return the original error and no successful replay prefix.

From a checkout with Rust 1.99+ and a native linker:

~~~sh
cargo run --locked --release --example observation_delivery -- delivery-lesson-output
cargo test --locked --manifest-path tests/observation-delivery/Cargo.toml -- --test-threads=1
~~~

The example requires a new output directory. It finishes both bounded in-memory artifacts first, then writes and flushes owned temporary files and publishes trace.tsv followed by ledger.tsv. Repeating the same command rejects the existing directory and preserves its files. This is not an atomic two-file transaction: a ledger failure can leave a complete trace and an owned temporary, which the error message reports. The native fault demonstrations and raw receipts are supplementary evidence under the package implementation directory.

At published core revision d36e705, all 43 tests pass on four hosted Windows/Linux and Rust 1.99.0/stable combinations; main independently verified archive, source and raw-log fingerprints. Local aliases both resolve to 1.99.0; hosted stable was 1.99.1. Required root, service and dashboard checks also passed. The immutable revision-two [external authority](https://github.com/zblack54/fusion-lab/issues/72#issuecomment-6527689826) fixes 37 requirements, eight constraints and 43 tests. Passing tests alone does not complete measured resource, example-failure or final conformance obligations. Those remain pending in this checkpoint.

The maximum release workload retains the source, trace, both replay results and original/regenerated artifacts through an owned measurement handshake. Acceptance requires a warmup then three fresh processes per native combination, each within 30 seconds and 512 MiB. A diagnostic is not that full series. [Issue #73](https://github.com/zblack54/fusion-lab/issues/73) tracks measurement-owner cleanup: an uncertain child termination or reap must retain ownership and block the next workload, not silently detach or retry numerical work.

[Issue #74](https://github.com/zblack54/fusion-lab/issues/74) distinguishes logical LF source hashes from actual generated Windows CRLF bytes. The original measured series and binary were preserved with supplemental byte provenance; future probes record actual materialized input hashes. This metadata correction did not rerun or relabel the original numerical work.

Exercise: the example gives position zero delay, velocity five ticks of delay, capacity eight and no scripted loss. Predict the 27 ledger outcomes before opening ledger.tsv. Checkable answer: six accepted positions, 20 stale velocity arrivals and one velocity observation still waiting at the horizon. Explain why the last observation is undelivered rather than stale, and why this says nothing about actual socket, channel or cancellation behavior. [Parent #6](https://github.com/zblack54/fusion-lab/issues/6) still owns those asynchronous questions.
