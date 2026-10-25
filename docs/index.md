# Fusion Lab

Learn how a robot's estimated position and uncertainty evolve when a sensor goes
quiet. Fusion Lab pairs a small Rust implementation with reproducible experiments
and explanations that connect equations to code.

## Begin with one run

~~~sh
cargo run --locked -- --seed 42 --output results/run.csv
~~~

The current v0.1 offline CLI compares a linear Kalman filter, last-position hold,
and measured-velocity integration. It records truth, observations, estimates,
and covariance at 100 Hz. Truth is kept outside the estimator. The default
position-beacon outage covers [10,20) simulated seconds.

Follow the [15-minute walkthrough](learn/index.md), then the
[math and code tour](learn/math-and-code.md). Use the
[research guide](learn/research.md) to reproduce studies and interpret negative
outcomes. The [iteration handoff template](learn/iteration-handoff.md) makes each
future change teachable and reviewable.

## Find the evidence and next steps

- [Running experiments](experiments.md): commands, configuration, CSV interpretation.
- [Outage study](experiments/outage-study.md): paired held-out seeds, error,
  uncertainty, recovery, and limits.
- [Capacity baseline](experiments/capacity-baseline.md): measured CLI resources and
  provisional service budget.
- [Roadmap](roadmap.md): tracked milestones, dependencies, and implementation status.
- [Numerical contract](design/v0.1.md): models, ownership, time, and verification.
- [Development](development.md): setup, checks, local documentation preview.

This documentation portal is prepared for local preview. Public hosting has not
been activated. Service, dashboard, replay and delivery faults are roadmap work;
a draft contract or passing design validator does not establish implementation.
