# Start here: a 15-minute learning path

Fusion Lab asks a concrete question: where is a moving robot when its position
beacon disappears, but velocity measurements keep arriving? The current v0.1
experiment is an offline Rust program. Truth is known to simulation and evaluation;
the estimator sees observations only. A service, browser UI, and delayed delivery
are planned work, not prerequisites for this walkthrough.

## Minutes 0–3: run one experiment

From the repository, with Rust 1.99+ and a native linker:

~~~sh
cargo run --locked -- --seed 42 --output results/run.csv
~~~

This runs 30 simulated seconds; it need not take 30 wall-clock seconds. Position
updates stop from 10 s through just before 20 s. Velocity updates continue.
Read the printed overall and outage-only errors. Files are results/run.csv and
results/run.txt; repeating the command overwrites them.

## Minutes 3–6: control the comparison

~~~sh
cargo run --locked -- --seed 42 --no-outage --output results/healthy.csv
~~~

The paired runs have identical truth and velocity observations; available
position measurements match outside the outage. Compare errors in their text
reports. Do not assume every noisy seed has greater error with the outage.
The [held-out study](../experiments/outage-study.md) records counterexamples.

Three estimators receive comparable available observations: the Kalman filter,
a hold of the latest position, and velocity integration with position resets.
The integration baseline is not dead reckoning from exact truth.

## Minutes 6–10: read the loop and one equation

Follow the [math and code tour](math-and-code.md), beginning with state and
prediction. Each tick is 0.01 s. Prediction moves estimated position using
estimated velocity; covariance represents uncertainty, not an error measured
against truth. At simultaneous sample times, position updates precede velocity.

Open the CSV as text or in a spreadsheet. Find tick 1000 (10 s): position fields
are empty while the outage flag is true. At tick 2000 (20 s), position resumes.
The position covariance block is p00, p01, p10, p11; its trace is p00+p11.
Follow one row across truth, measurements, estimate, and covariance. Avoid
confusing a blank measurement with the numerical value zero.

## Minutes 10–13: inspect evidence and a limitation

Read the [research guide](research.md). The repeated-seed outage study reports
paired effects, uncertainty, and the seeds where integration beat the filter.
An ellipse labelled 95% does not guarantee 95% empirical coverage. Changing
sensor sigma currently changes both generated noise and assumed covariance;
true/assumed noise mismatch and velocity bias are [future work](../roadmap.md).

## Minutes 13–15: test your understanding

Before running, predict the noiseless straight-motion position at 8 s from
initial velocity [1, 0.5] m/s. Then execute:

~~~sh
cargo run --locked -- --duration 8 --scenario straight --no-outage --position-sigma 0 --velocity-sigma 0 --acceleration-density 0 --output results/exercise.csv
~~~

Truth should be [8, 4] m at tick 800. The filter starts from a different prior;
its estimated result should be close, rather than assumed identical by design.
Explain why --no-outage is necessary for this duration: the default outage ends
at 20 s and must fit inside the run.

Continue with [running experiments](../experiments.md), the
[design contract](../design/v0.1.md), and the [roadmap](../roadmap.md).
When you change something, use the [iteration handoff](iteration-handoff.md)
to explain it to the next reader.
