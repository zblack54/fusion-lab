# Experiment service: ownership before concurrency

Status: verified local service in [PR #32](https://github.com/zblack54/fusion-lab/pull/32), awaiting the final proof commit's checks and merge. [Conformance decision](../design-docs/experiment-service/implementation/acceptance.md).

## What changed and why

The adapter exposes the existing deterministic experiment through loopback HTTP. It admits one worker and returns a run identifier, then offers lifecycle status, cancellation, bounded state events and completed CSV/report downloads. Simulation still owns truth, and the estimator receives observations only. HTTP timing cannot change the integer simulation clock.

A separate process keeps the complete numerical sample vector out of the service. One active worker is the starting capacity, informed by the [CLI capacity baseline](../experiments/capacity-baseline.md) and verified service measurements. The512MiB aggregate allocation is an engineering budget, not an operating-system memory limit.

## Follow the owners

`crates/experiment-service/src/server.rs` owns admission, worker completion, retention and artifact leases. `src/transport.rs` owns bounded connections and HTTP deadlines. `src/main.rs` supplies local controls and explicit shutdown. The [crate guide](https://github.com/zblack54/fusion-lab/blob/feature/experiment-service/crates/experiment-service/README.md) documents requests, artifacts and failure recovery.

A lease guard owns a retained artifact from acquisition through file close. Canceling a request cannot abandon a lease count. An atomic sibling claim owns the artifact root even while the service is idle; checking an empty directory alone could not exclude a second instance.

Shutdown is also an ownership operation. Review found a nested result where a timeout completed successfully while its inner task drain failed; checking only the outer result hid a task panic. One drain now owns the deadline, joins every connection and reports failures. Worker joins likewise observe every handle after an earlier panic, and explicit finish performs cleanup after a runtime-thread failure.

If the OS rejects termination or reap confirmation, the same Child and root fence remain owned. The caller-owned shutdown call can stay pending while that exceptional fault persists. Shared bounded recovery never restarts the numerical experiment. After the fault clears, actual reaping and all joins precede release, and the original error is returned. Real-child fault tests demonstrated persistent kill/reap rejection, transient recovery and failed-monitor cleanup. The healthy-host five-second guarantee does not promise a return deadline for a persistently failing kernel.

## Reproduce

From the PR checkout, run:

```text
cargo test --manifest-path tests/experiment-service/Cargo.toml --locked -- --test-threads=1
cargo test --manifest-path crates/experiment-service/Cargo.toml --locked
cargo build --locked
cargo run --manifest-path crates/experiment-service/Cargo.toml --locked -- --bind 127.0.0.1:8080
```

The first command runs21 approved protocol tests; the second runs14 supplemental ownership/failure tests at the accepted revision. Submit the complete JSON configuration from the crate guide. Ctrl+C invokes shutdown; embedding callers must observe its fallible result.

## Evidence and limits

All35 tests and quality gates passed on native Windows/Linux with Rust1.99.0 and stable1.99.1. Independent review checked both native release artifacts against exact GitHub source hashes. Maximum-run sampled aggregate memory was about104.4MiB on Windows and103.8MiB on Linux; both60-second overload probes passed. Strict slow-reader cleanup was3.980sWindows/3.999sLinux. An earlier Linux5.002s failure remains recorded; the fix reduced socket idle time to4seconds to reserve cleanup time, keeping the original `<5s` criterion.

Sampling can miss transient peaks, Linux private resident memory differs from Windows private commit, and these are recorded-host results rather than universal latency guarantees. Artifacts are transient, POST retries create new runs, and this version provides loopback access without remote authentication or automatic crash recovery. No numerical algorithm or offline experiment API changed.

## Check your understanding

Start one idle service with an empty artifact root, then try another instance using that root on a different port. The second must fail without changing the first instance's files. Graceful shutdown should release the claim. Explain why an empty-directory check cannot establish exclusivity, and why a successful kill request cannot establish that a child was reaped.
