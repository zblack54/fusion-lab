# Experiment service: ownership before concurrency

Status: implemented, under acceptance verification in [PR #32](https://github.com/zblack54/fusion-lab/pull/32). This page describes that branch; it does not claim the service has shipped on main.

## What changed and why

The adapter exposes the existing deterministic experiment through loopback HTTP. It admits one worker and returns a run identifier, then offers lifecycle status, cancellation, bounded state events and completed CSV/report downloads. The numerical algorithm is unchanged: simulation still owns truth, and the estimator receives observations only. HTTP timing cannot change the integer simulation clock.

A separate process keeps the complete numerical sample vector out of the service. One active worker is a deliberate starting capacity, informed by the [CLI capacity baseline](../experiments/capacity-baseline.md). The proposed 512 MiB aggregate allocation is an engineering budget, not an operating-system memory limit; service-level measurements remain an acceptance gate.

## Follow the owners

`crates/experiment-service/src/server.rs` owns admission, worker completion, retention and artifact leases. `src/transport.rs` owns bounded connections and HTTP deadlines. `src/main.rs` supplies the local command line and explicit shutdown. The [crate guide](https://github.com/zblack54/fusion-lab/blob/feature/experiment-service/crates/experiment-service/README.md) documents requests and artifact paths.

Two review findings illustrate why passing protocol tests is insufficient. An artifact lease acquired before an asynchronous open could leak when the request was canceled. A guard now owns the lease immediately and travels with the file operation, closing the file before releasing retention ownership. Separately, checking an empty artifact root did not exclude a second idle service. Atomic sibling-file creation now claims that root for the instance. Crashes can leave a stale claim; automatic recovery is outside this service version.

## Reproduce the verified boundary

Check out the PR branch in a separate checkout, then run:

```text
cargo test --manifest-path tests/experiment-service/Cargo.toml --locked -- --test-threads=1
cargo test --manifest-path crates/experiment-service/Cargo.toml --locked
cargo build --locked
cargo run --manifest-path crates/experiment-service/Cargo.toml --locked -- --bind 127.0.0.1:8080
```

The first command runs the 21 approved protocol tests; the second includes four additional ownership regressions at the initial PR revision. Use the complete JSON configuration in the crate guide when submitting a run. Ctrl+C invokes shutdown; embedding callers must observe the fallible shutdown result.

## Evidence and limits

Native Windows tests and quality gates passed before publication. The PR adds actual Windows/Linux × Rust 1.99/stable CI, whose exact-head results must be inspected rather than inferred from local success. Release workload memory/latency measurements, sustained load and injected failure demonstrations remain acceptance work. See the PR's `implementation/evidence.md` and `implementation/acceptance.md` for the current record. Artifacts are transient, POST retries are not idempotent, and no remote service or authentication is provided.

## Check your understanding

Start one idle service with an empty artifact root, then try another instance using the same root and a different port. The second must fail without changing the first instance's files. After graceful shutdown, the root claim should be released. Explain why an empty-directory check alone cannot establish exclusive ownership.
