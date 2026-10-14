# Fusion Lab

Fusion Lab estimates a simulated robot's position from noisy sensors and measures behavior during a position-beacon outage.

Run the offline experiment:

```sh
cargo run --locked -- --seed 42 --output results/run.csv
```

Start with the [experiment guide](experiments.md). The [design contract](design/v0.1.md) documents models, ownership, timing, and verification. The [development guide](development.md) describes checks and documentation tooling.
