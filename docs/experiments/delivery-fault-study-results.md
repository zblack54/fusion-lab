# Delivery faults and bounded history correction

Issue #8 compared stale-drop (W0) with a 500-tick history window (W500) on the same observations in healthy, delay-only, loss-only, and combined arms. The [frozen protocol](delivery-fault-study-protocol.md) specifies the 100 paired seeds and analysis. This run produced all 800 required records on Windows with Rust 1.99.0. The driver verified transport parity between policies in every arm and bitwise snapshot parity in the healthy and loss-only arms.

| Scenario | Combined-arm RMSE difference, W500 − W0 | 95% paired bootstrap interval |
| --- | ---: | ---: |
| Straight | −0.415 m | [−0.473, −0.361] m |
| Maneuver | −1.551 m | [−1.636, −1.470] m |
| Equal-weight mean | −0.983 m | [−1.034, −0.933] m |

During the combined disturbance, joint position ellipse coverage averaged 99.38% for W0 and 96.31% for W500. After the disturbance window it was 95.23% and 95.75%, respectively. Higher coverage alone can reflect wider uncertainty and does not establish better calibration. These are seed-level summaries for the specified workload.

All delay-only runs met the recovery criterion by the horizon under both policies. Mean confirmation was 127.88 ticks after the fixed boundary for W0 and 99 for W500. The paired difference was −28.88 ticks (95% interval [−43.58, −16.64]). Tick 99 is the earliest possible confirmation. Recovery here means 100 consecutive reporting ticks within 0.5 m of the corresponding healthy run's error. Loss continues in the loss arms, so recovery is reported only for delay-only.

Reproduce the reserved fixture using `cargo run --release --locked --offline --example delivery_fault_study -- fixture fixture.tsv` in a fresh output location. The research directory contains the held-out rows, full bootstrap analysis, fixture outputs, exact input and binary hashes, and launch receipt. Recompute the published intervals with Python 3.12 using `python research/delivery-fault-study/analyze.py heldout research/delivery-fault-study/records.tsv analysis.json`.

The results cover the fixed synthetic scenarios and deterministic fault schedules. They do not measure wall-clock queue overload or generalize to other delay, loss, model, or noise settings.
