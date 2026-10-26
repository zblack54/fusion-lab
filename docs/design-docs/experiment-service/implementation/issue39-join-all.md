# Issue39: observe all worker handles after a join failure

Status: NATIVE_REPAIR_VERIFIED; CURRENT_SOURCE_PLATFORM_REVIEW_PENDING

The old `join_workers` removed the owned handle vector from State, then returned at the first panic through `?`. Dropping the remaining JoinHandles detached still-running readers/monitors. The repair joins every handle and aggregates indexed panic errors before returning Err; it does not suppress the first error or claim success.

Original-red command:

```text
cargo test --manifest-path crates/experiment-service/Cargo.toml --locked --lib worker_join_failure_still_observes_all_remaining_owned_threads -- --nocapture
```

Before the repair, this executed one test and failed exit101 with `join_workers returned after the first failure while another owned worker was still blocked`. It uses an already-panicked first worker and a separately gated real second thread. The test deliberately releases and observes the second fixture even on the broken path, avoiding a leak in its own failing setup. After the repair the same unchanged regression passed in0.11seconds.

All10supplementary production tests and all-target Clippy with `-D warnings` passed. The trusted21-test implementation runner is refreshed separately against the unchanged external digest. This fixes premature handle abandonment, but it does not solve persistent termination failure in issue40; that distinct policy is proposed in design-revision2-proposal.md and is not implemented by this repair.
