# A changing sensor offset without changing truth

Status: deterministic linear drift implemented for library/CLI under [issue20approval](https://github.com/zblack54/fusion-lab/issues/20#issuecomment-6480183228), pending independent review and exact-head CI. This extends the accepted constant-bias slice; no random walk or bias estimator is added.

## What changes

The observed velocity now supports an explicit offset b(t)=b0+r*t, where b0 is the existing velocity_bias[m/s], r is velocity_drift_rate[m/s²], and t comes directly from the integer10ms event tick. This describes how sensor error changes, not how truth accelerates. True Gaussian sigma still determines random scatter; assumed R still determines how strongly the filter trusts the observation. Neither corrects bias automatically.

Each rate component must be finite in[-1,1]m/s². Initial and final combined offsets must lie in[-100,100]m/s for the configured1–3600s run. Linear functions attain their extrema at endpoints, so validating both bounds the whole horizon before sampling/allocation. There is no clipping. Invalid rates or whole-horizon offsets produce typed InvalidConfig errors before CLI output-parent creation. The rate bound is a numerical scenario limit, not a claim about a particular physical sensor.

src/simulation.rs owns validation and observation-boundary arithmetic. For zero or signed-zero rate it keeps the accepted constant-bias path exactly; tick0 uses b0 directly. It evaluates drift only when the existing20Hz velocity sample is produced, with no extra draw or event. src/main.rs exposes signed flags and reports initial/rate/duration/final offset with units. Truth dynamics, position samples, hold baseline, outage schedule and covariance at the same model assumptions remain identical. Integration intentionally changes because it consumes the same measured velocity as the filter, then resets on position observations.

Config gains velocity_drift_rate:[f64;2]. Complete external Rust struct literals must supply it; ..Config::default() retains zero drift. The current research example supplies zero explicitly for compilation. Its original576-key study was executed without drift; the archived source/protocol/binary/results remain unchanged and are not relabeled as new-source or drift evidence.

## Independent first and later updates

Take noiseless straight truth[1,.5]m/s, initial bias[.2,-.1], rate[.4,-.2], q0, assumed position sigma3 and velocity sigma2. At tick0 the observation is[1.2,.4], prior velocity variance4 gives gain1/2, and estimated velocity is[.6,.2] with variance2.

At tick5(.05s) the observation is[1.22,.39]. No position sample arrives between0and5. Predicted velocity variance remains2, so gain is1/3; velocity becomes[121/150,79/300]m/s with variance4/3. Predicted position/velocity cross covariance is.1, giving updated position[121/3000,79/6000]m. Tests check these independently derived rational values rather than copying the drift helper.

Integration uses the previous sampled velocity: tick1position[.012,.004], tick5[.06,.02], then tick6[.0722,.0239] after the new velocity sample. At tick20 it resets to the raw position observation[.2,.1]. This differs from continuously integrating a physical acceleration, even though the rate units are m/s².

## Reproduce and inspect

~~~sh
cargo run --locked -- --seed 42 --velocity-drift-rate-x 0.01 --velocity-drift-rate-y -0.005 --output results/drift.csv
~~~

The companion report states initial offset, rate, duration, final offset and the equation separately from Gaussian sigma and acceleration density. CSV retains35columns; import it into the local dashboard and keep the report for provenance. Service JSON/dashboard controls are unchanged. Service-created jobs omit bias/drift flags and remain zero-bias/zero-drift.

Before code, captured actual default, nonzero constant-bias and noiseless CSVs at80c39e4 with source/lock/Rust1.99WindowsMSVC provenance. All nine comparisons—implicit zero, negative-x zero and negative-y zero for each case—preserved CSV bytes exactly after this change. This is a same-reference-target compatibility check, not a cross-platform bitwise promise. Report text intentionally gains provenance.

## Prespecified descriptive demonstration

Before execution, fixed seeds7,42,314159,30s maneuver,outage[10,20),initialbias[0,0],true sigmas1m/.15m/s,q.1 and paired rates[0,0]/[.01,-.005]m/s². The nonzero rate reaches final offset[.3,-.15]m/s. All six commands/reports, source/CSV hashes and compatibility comparisons are in [velocity-drift-demo.json](../experiments/velocity-drift-demo.json).

| Seed | Rate[m/s²] | Overall filter RMSE[m] | Outage filter RMSE[m] | Overall nominal95% coverage[%] |
|---|---|---:|---:|---:|
|7|[0,0]|.2128|.0873|100.00|
|7|[.01,-.005]|1.4702|1.2745|31.52|
|42|[0,0]|.2667|.1769|100.00|
|42|[.01,-.005]|1.4687|1.3470|20.03|
|314159|[0,0]|.4147|.4473|65.14|
|314159|[.01,-.005]|1.3135|1.0585|26.66|

These selected drifting runs have larger error and lower observed coverage; the hold baseline is unchanged. All three drifting outage coverage values are0%, but these trajectories do not establish general calibration or an estimator ranking. The zero-rate seed314159 already has imperfect coverage. No tuning, outcome-based selection or extension of the completed576-key no-drift study occurred. The existing filter cannot estimate or remove this changing offset.

## Verification and limits

Tests cover tick0/tick5 rational filter values, tick1/5/6integration/reset20, exact long-horizon binary bounds, invalid/nonfinite components, whole-horizon rejection before output creation, signed-zero and one-axis-zero bit preservation, a crossing-zero offset, unchanged truth/position/schedule/covariance and seeded repeatability. Changing assumed noise preserves all drifting observations and both baselines. The PR records actual root, unchanged service/dashboard and documentation gates; local implementation evidence alone is not final acceptance.

Exercise: with b0[-.25,.125]m/s and r[.5,-.25]m/s², find the time when both offsets cross zero. Predict the noiseless velocity observation then. Explain why reported covariance can stay unchanged even though error before and after that crossing changes.
