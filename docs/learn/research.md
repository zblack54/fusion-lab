# Reproduce a result, then question it

The goal is to understand what the evidence supports. Start with the
[experiment commands](../experiments.md) and the
[15-minute walkthrough](index.md); then read a prespecified study and its limits.
Raw tool transcripts are supplementary evidence, not the learning path.

## Position-beacon outage baseline

Read the [protocol](../experiments/outage-study-protocol.md) before the
[results](../experiments/outage-study.md). The question compares a ten-second
position outage with paired healthy runs. It also compares the existing filter
with hold and velocity-integration baselines receiving the same available data.
There is no tuning or algorithm change. Delivery delay/loss are not tested.

The protocol fixes 50 distinct seeds for straight motion and 50 for maneuvering,
30-second runs, noise/process parameters, windows, recovery criterion, stopping
rule, and seed-level bootstrap before results are examined. Reproduce from the
repository with Python 3.12+:

~~~sh
cargo build --release --locked
python scripts/outage_study.py --self-check
python scripts/outage_study.py --output /absolute/path/outside-repository/outage-study.json
~~~

On Windows the default binary is target/release/fusion-lab.exe. On Linux pass
--binary target/release/fusion-lab. The recorded execution was on Windows; this
command is a reproduction instruction, not a claim that Linux was evaluated.
Use an absolute Windows output path instead of the POSIX example where needed.
The driver rejects evidence output inside the repository, uses sequential child
processes with timeouts, and removes temporary CSV/report pairs. It retains
per-seed statistics, artifact hashes, exact argv, and provenance in external JSON.

Read the primary paired outage-period RMSE effect before secondary windows.
The reported average error increase is about 0.0244 m for straight motion and
0.0312 m for maneuvering under these conditions. Individual seeds can reverse
that direction: an average does not describe every realization. Integration
outperformed the filter during the outage for seeds 10008 and 20033. Follow
those negative outcomes in the results rather than hiding them behind a mean.

Recovery is an explicitly chosen operational measure: the first post-outage
100-sample window whose filter RMSE is within paired healthy RMSE +0.1 m.
A zero latency under that tolerant criterion does not mean instantaneous or
permanent recovery. Covariance and mean post-outage error can remain elevated.
An empirical ellipse coverage fraction near or above 95% does not prove a
correct uncertainty model.

Exercise: recompute the mean paired outage-minus-healthy filter RMSE from the
50 per-seed records in one family. Do not pool correlated ticks or bootstrap
the two conditions independently. Explain why resampling complete seed pairs
preserves the comparison's shared random inputs.

## Resource baseline

The [capacity study](../experiments/capacity-baseline.md) measures complete CLI
runtime and child peak working set at 30, 300, and 3600 simulated seconds.
It includes process startup and file output; it is not pure compute time.
Its proposed service budget is a design input, not implemented enforcement or a
worst-case guarantee. The probe is Windows-specific. Reproduce only with the
platform and prerequisites described in that report.

## Measurement mismatch and remaining work

The CLI/library now support separate true and assumed measurement noise through
optional assumed-sigma overrides. Omission preserves matched noise. Read the
[noise-mismatch lesson](noise-mismatch.md): its three fixed seeds demonstrate
controlled observations and report counterexamples, not calibration or tuning.
The service JSON is unchanged and still creates matched-noise jobs. Constant
velocity-sensor bias remains planned; it is not a relabelled sigma or q value.
Delay/loss and late-observation policy need their own implementation.
Track these through the [roadmap](../roadmap.md), with separate tuning and held-out
evaluation seeds and a protocol written before comparisons.

## Design decisions and change handoffs

Start with the [numerical contract](../design/v0.1.md). The
[service design record](https://github.com/zblack54/fusion-lab/issues/5)
and [implementation tracking](https://github.com/zblack54/fusion-lab/issues/5)
explain transport ownership; read their status before treating them as
accepted or implemented. Formal draft tests that fail at an explicit seam are
design evidence, not a working service. Follow the
[iteration handoff template](iteration-handoff.md) for each future change.
