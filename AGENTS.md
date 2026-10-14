# Fusion Lab development

When the owner's optional local skill suite is present, read `.agents/skills/rust/SKILL.md` and its shared engineering contract before Rust work. Route focused tasks to its architecture, testing, quality, documentation, or spec-design child as appropriate. Keep relative references intact. Otherwise follow the project contract and gates below.

The optional local Rust suite comes from the owner's private `zblack54/copilot-skills` repository, revision `8f461b9d6e19aa5fbb54ee7bc9d1dccdc55a2771`. It is ignored by Git and is not a build dependency. Update it deliberately from upstream.

Read `docs/design/v0.1.md` before changing behavior. Keep numerical and simulation code synchronous; transport and async orchestration are future adapters. Truth belongs to simulation and evaluation; estimators receive only observations. Time is integer 10 ms ticks. Preserve deterministic sensor streams and event ordering.

Required gates:

```
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --doc
cargo doc --locked --no-deps
```

Keep public API documentation, CLI help, and the design contract consistent. Derive numerical tests from independently computed expectations; do not weaken them to fit implementation. Never claim covariance calibration based on one trajectory.

Use plain ownership and concrete implementations first. Validate configuration before allocation or output. Return typed library errors, preserve filter state on numerical failure, and forbid unsafe code. Add dependencies only for a concrete benefit. Report checks that passed and limitations separately; preserve the intended test contract.

