# Fixed velocity sensor bias

Status: constant-bias slice implemented for library/CLI under [issue20](https://github.com/zblack54/fusion-lab/issues/20), pending review and exact-head CI. Drift remains planned; this slice does not close the issue.

## Meaning and ownership

Gaussian sigma describes random scatter. A fixed world-frame offset shifts every scheduled velocity observation by the same [bx,by] m/s. Config.velocity_bias defaults to [0,0]; components must be finite in [-100,100]. The seed determines noise, not the explicit offset. The filter has no bias state and receives only biased observations, without truth access or automatic covariance inflation. Linear-model covariance is unchanged by changing observation values, so confident error is possible.

src/simulation.rs validates the offset before allocation/sampling and adds it after the existing velocity draw. It skips each zero component, including -0, preserving arithmetic and draw counts. Truth, position observations, outages and held-position baseline stay identical. The integration baseline consumes the same biased velocity as the filter, so its trajectory intentionally changes; position observations still reset it. src/main.rs parses signed flags and reports units and offset separately from true/assumed noise. Complete external Config struct literals require the new field (approved 0.1 source impact); ..Config::default() retains zero bias.

## Independent first-tick reasoning

For noiseless straight motion, truth starts at [1,0.5]m/s. Bias[2,-1] yields observation[3,-0.5]. Prior velocity variance4 and assumed sigma2 give gain4/(4+4)=1/2, posterior velocity[1.5,-0.25] and variance2. The independent tests check these constants. At tick1, integration uses the observed velocity, giving position[.03,-.005]m; at tick20 it resets to the position observation. Covariance equality across paired runs confirms that shifting observations alone does not change the model's reported uncertainty.

## Reproduce

~~~sh
cargo run --locked -- --seed 42 --velocity-bias-x 0.5 --velocity-bias-y -0.25 --output results/bias.csv
~~~

The report identifies the fixed world-frame offset in m/s and distinguishes it from sigma or bias correction. CSV keeps35columns. Import this CLI artifact into the local dashboard; retain the companion report for bias provenance. Service JSON has no bias controls and service-created jobs stay zero-bias because they omit these CLI flags. No locked service/dashboard contracts change.

## Prespecified descriptive demonstration

Before executing the demonstration, fixed seeds7,42,314159 and paired biases[0,0]/[.5,-.25],30second maneuver,outage10–20, sigmas1m/.15m/s,q.1. No tuning or run selection. All six commands, reports, CSV hashes and source provenance are in [velocity-bias-demo.json](../experiments/velocity-bias-demo.json).

| Seed | Bias[m/s] | Overall filter RMSE[m] | Outage filter RMSE[m] | Overall nominal95% coverage[%] |
|---|---|---:|---:|---:|
|7|[0,0]|.2128|.0873|100.00|
|7|[.5,-.25]|5.1778|5.8113|8.90|
|42|[0,0]|.2667|.1769|100.00|
|42|[.5,-.25]|5.2089|5.8922|4.90|
|314159|[0,0]|.4147|.4473|65.14|
|314159|[.5,-.25]|4.9711|5.5338|9.33|

The selected biased runs all show larger filter error and lower observed coverage; their hold baseline is unchanged. This demonstrates this specified failure case, not universal estimator ranking or statistical calibration. The unbiased seed314159 already has imperfect coverage; bias is not the only model assumption. A larger held-out study belongs to issue21. Drift, bias-state estimation, correction, heading and EKF remain future work.

## Verification and limits

Tests check analytical tick0/tick1, reset semantics, signed bounds/nonfinite rejection, deterministic observations, covariance equality, schedule/truth/position/hold fairness and invalid CLI input before output-parent creation. Pre-change default and noiseless CSVs captured at0bbd098 match after adding explicit signed-zero flags byte-for-byte on the same Rust1.99 Windows MSVC reference. Source/lock/compiler hashes and both comparisons are in the demonstration JSON; this is not a cross-platform bitwise promise. Report text intentionally adds provenance. Actual gate evidence accompanies the PR; acceptance requires independent review and exact-head CI.

Exercise: with noiseless straight motion and assumed velocity sigma2, change bias to[-1,.5]. Derive tick0 observation/estimated velocity, then tick1 integration. Explain why covariance stays the same while estimation error changes.
