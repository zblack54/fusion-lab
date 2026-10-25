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


## Autonomous development and review (issues #2–#4)

Before implementing a feature, identify or create its dedicated GitHub issue. Record problem, intended behavior, non-goals, dependencies, acceptance criteria, and verification. Bugs and independently actionable probe findings have separate issues. Link local implementation revisions and concise human review evidence; do not close an issue merely because code exists.

Use a lightweight design brief for bounded work. Large or difficult cross-component work requires the owner's rust-spec-design formal workflow: validate the executable contract, review design, record delegated approval authority, lock the approved test surface, implement dependency-ordered slices, and review final conformance including non-test obligations. Resolve contract conflicts through design revision; never weaken locked tests to make implementation pass. Keep approved digests and trusted validators outside implementation-owned candidates.

The owner delegated routine local development and within-scope design handoffs in the originating Codex chat on 2026-10-25. This permits technical approval after evidence review, not invention of unresolved product policy. Use subscription authentication only; no API-key fallback, paid model API, deployment, or live trading. GitHub issue updates, focused branches and PRs, delegated review, and merging verified within-scope changes are authorized by the owner's subsequent clarification. Platform access checks remain in force.

Maintain planned / implementing / implemented-unverified / verified / awaiting-human-review states with exact revisions and actual commands. Preserve failures and unrun checks. Operate on the dedicated local checkout, with one writer and bounded retries. The existing older untracked checkout must not be overwritten. Before each slice, inspect status and current progress; after each slice, checkpoint code and evidence, publish a focused PR, review its exact head and checks, merge when authorized gates pass, update its issue, and immediately take the next ready task. Continue until account limits, user stop, completed scope, or a real blocker prevents progress.

The health probe is read-only for production code. It files deduplicated suggestions for orchestrator triage; accepted work enters the normal issue/design/implementation/verification flow. Check performance, scalability, and maintainability after five verified slices or major architecture changes once the supervisor supports these triggers. Do not infer measurements from static inspection.

Account-limit recovery: check the actual five-hour and weekly usage windows; checkpoint and resume at reported five-hour reset, and switch to weekly only when weekly remaining is zero. Do not use paid fallback, automatically purchase credits, or redeem reset credits.
