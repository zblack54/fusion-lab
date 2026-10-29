# Velocity sensor linear drift: design preflight

Status: approved for implementation by delegated main review in [issue20comment6480183228](https://github.com/zblack54/fusion-lab/issues/20#issuecomment-6480183228); implemented pending independent review and exact-head CI. Remaining scope of [issue20](https://github.com/zblack54/fusion-lab/issues/20). This approved slice changes the core/CLI and tests; no dependency or locked service/dashboard contracts change. The constant-bias prerequisite is accepted in [PR63](https://github.com/zblack54/fusion-lab/pull/63); its original design document is historical and its implementation status wording is not a new approval. The sections below record the approved model and verification contract.

## Smallest useful model

Propose a deterministic world-frame velocity offset for each axis:

~~~text
t_seconds = f64::from(integer_tick) / 100.0
bias(t) = initial_bias + drift_rate * t_seconds
velocity_observation = existing_noisy_velocity + bias(t)
~~~

The existing velocity_bias [m/s] is the offset at time zero. New velocity_drift_rate: [f64;2] has units m/s² (change in sensor velocity offset per second), default[0,0]. It is a sensor fault input, not physical acceleration, truth dynamics, extra filter state or correction. The seed controls the unchanged sensor noise streams; explicit drift does not consume additional random draws. This is not a claim to model every physical sensor's stochastic drift.

A fixed linear rate is reproducible, has an analytical finite-horizon bound, and introduces no additional RNG state. A random walk instead requires a separate diffusion parameter with units, increment cadence, seed/substream and draw-count policy, pathwise versus probabilistic bounds, and failure/clipping semantics. Those choices are unnecessary for this bounded first drift model. Do not quietly implement random walk, clipping, bias estimation, EKF or adaptive tuning.

## Proposed domain and validation

Retain initial velocity_bias components finite within[-100,100]m/s. Propose drift-rate components finite within inclusive[-1,1]m/s². This rate bound is a numerical scenario limit, not a physical realism assertion. Additionally require each combined bias to remain within[-100,100]m/s over the entire configured duration. Validate initial and final endpoints before sample allocation or sensor construction. With an affine function on nonnegative time, bounded endpoints establish the whole-horizon bound; duration remains1–3600whole seconds. Compute the final endpoint using the same explicit multiply-then-add order as the observation-time formula, with duration converted directly from its integer seconds, and require a finite result. No clamping or late partial failure is permitted. Return Error::InvalidConfig("velocity_drift_rate") for nonfinite/out-of-rate-domain input and Error::InvalidConfig("velocity_bias_horizon") for an invalid combined endpoint, preserving the existing velocity_bias validation for the initial offset.

The extra endpoint constraint matters: duration3600, initial0 and rate1 are individually bounded inputs but must fail before allocation because the final offset is3600m/s. A rate of±1/64m/s² with initial bias±43.75m/s over3600s reaches exactly±100m/s and should be accepted. Rate0.03125 with initial0 reaches100at3200s and100.03125at3201s, providing exact binary boundary fixtures.

Use the integer event tick as the only time source; drift is evaluated only when the existing20Hz velocity observation is sampled, including tick0 and the final tick. No accumulated floating-time or repeated bias integration, wall clock, additional sample event or randomness. Scheduled position noise is still drawn during outages. Preserve position-before-velocity update order.

For each axis with zero or signed-zero rate, retain the existing constant-bias arithmetic path exactly; do not calculate initial_bias+0*t. At tick0 use initial_bias directly, even for nonzero rate. Skip adding an effective offset that equals zero, as in the accepted bias implementation. This preserves existing constant-bias/default CSV bytes when drift is disabled on the captured reference target. No cross-platform bitwise guarantee is proposed. For nonzero-rate observations use an ordinary explicit multiplication followed by addition, not a fused multiply-add or incremental integration contract.

## Proposed public/reproduction surface

Add Config.velocity_drift_rate: [f64;2], retained in Experiment.config. Complete external Config literals require the new field (another explicit0.1source impact needing main approval); literals using ..Config::default() retain zero drift. Update the research-only fixed-study runner's complete Config literal explicitly to zero drift if required for compilation, without rerunning or relabeling its archived executed source/results. The already completed issue21 protocol excluded drift; its frozen history and evidence must remain unchanged.

Expose signed CLI --velocity-drift-rate-x and --velocity-drift-rate-y, with m/s² units and[-1,1]range in help. Reports state initial bias[m/s], rate[m/s²], configured duration[s], final effective bias[m/s] and the deterministic linear equation separately from Gaussian sigma/assumedR. Reports must distinguish sensor drift from physical acceleration_density, whose units are m²/s³. CSV retains35columns; its velocity columns record the changed observations, with the companion report carrying provenance. Negative separate arguments must parse as signed values rather than requiring undocumented syntax.

Keep service JSON and dashboard contracts unchanged. The service subprocess omits both bias and drift flags, so service-created jobs remain zero-bias/zero-drift. Users reproduce drift through the CLI and may import its CSV locally; retain the companion report. No UI control, route, persistence or estimator changes are included.

## Independent numerical expectations

Use a1second noiseless straight trajectory, no outage, true sigmas0, assumed position sigma3 and assumed velocity sigma2, q0. Truth velocity remains[1,.5]m/s. Choose initial_bias[.2,-.1]m/s and rate[.4,-.2]m/s²:

| Tick | Time[s] | Effective offset[m/s] | Velocity observation[m/s] |
|---|---:|---|---|
|0|0|[.2,-.1]|[1.2,.4]|
|5|.05|[.22,-.11]|[1.22,.39]|
|100|1|[.6,-.3]|[1.6,.2]|

At tick0, prior velocity variance4 and measurement variance4 give gain1/2: estimated velocity[.6,.2], diagonal velocity covariance2. The position posterior variance is225/34; velocity-position cross covariance is initially0. Before tick5, q0 and no additional measurement leave velocity variance2, while prediction produces position/velocity cross covariance.1. The tick5 velocity gain is2/(2+4)=1/3. Independent expected updated velocity is[121/150,79/300]m/s, velocity variance4/3, and estimated position[121/3000,79/6000]m. These rational constants detect missed/early drift sampling and accidental truth/filter changes; test them with fixed tight numerical tolerance instead of duplicating the implementation's drift helper.

Integration consumes the same observation as the filter. At tick1 its position is[.012,.004]m. At tick5, integration still used the previous observation, giving[.06,.02]m before the new velocity sample. At tick6 the new observation gives[.0722,.0239]m. Tick20resets integration to the noiseless position observation[.2,.1]m. These cases distinguish sampling-time drift from continuously integrating a physical acceleration. No integration-equality assertion across drift settings is valid.

## Fairness, invalid inputs and compatibility

Pair runs with the same seed/config except drift rate. Require equal lengths/ticks, exact truth, position observations, position outages, held-position baseline, measurement schedules and covariance at every tick. At fixed bias/rate, changing only assumed noise preserves all observations and both baselines. Between different drift rates, velocity observations shift by delta_rate*t within absolute1e-12m/s plus relative1e-12*max(abs(observations)); integration intentionally differs. No extra RNG draw is allowed. Compare repeated identical seeded runs exactly; compare deterministic sensor paths, not only aggregate metrics.

Verify each signed rate component independently, inclusive rate endpoints when the combined horizon is legal, NaN/±infinity/out-of-range rejection, combined-horizon boundary equality/violation and validation before CLI output-parent creation. Include sign reversal/crossing-zero cases and one axis with signed-zero rate while the other drifts. Capture pre-change default, nonzero constant-bias and noiseless CSVs with source/lock/compiler hashes. Default/explicit±zero drift must preserve those bytes on the same reference target; report text intentionally gains provenance. Existing constant-bias, noise-mismatch, service and dashboard regressions remain required. No locked tests or contracts are weakened.

## Proportional implementation sequence after approval

1. Main approves model/rate/horizon bounds, Config source impact, CLI/report names and scopes; record approval in issue20. Capture compatibility baselines before code.
2. Add validation and observation-boundary drift on the single synchronous run path, with zero-rate compatibility. Do not change sensor RNG, truth, estimator or service wire behavior.
3. Add independent analytical/validation/fairness/CLI tests and run required root gates plus unchanged service/dashboard contracts and strict documentation build.
4. Before demonstration execution, fix seeds7,42,314159,30second maneuver,outage10–20, initialbias[0,0], true sigmas1m/.15m/s,q.1, and paired rates[0,0]/[.01,-.005]m/s². Six descriptive runs only; no tuning or selection and no modification of the completed576-key constant-bias study. Publish all metrics and provenance, including negative results, in a focused reviewed PR and learning note. Issue20 closes only if main confirms that this separate drift contract and accepted constant-bias slice fulfill its remaining scope.
