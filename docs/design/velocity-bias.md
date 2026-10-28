# Controlled constant velocity bias: design preflight

Status: constant-bias slice approved by delegated main review in [issue20comment6479496819](https://github.com/zblack54/fusion-lab/issues/20#issuecomment-6479496819); implemented pending independent review and exact-head CI. Existing issue [#20](https://github.com/zblack54/fusion-lab/issues/20) also requests a separately specified drift model; this brief proposes only a bounded first constant-bias slice. Drift remains unimplemented and needs a later reviewed contract. No dependency or locked service/dashboard contract changes are part of this slice.

## Proposed numerical contract

Represent an explicit fixed world-frame velocity offset [bx, by] in m/s, default [0, 0]. It is a deterministic scenario input, not sampled from the seed and not an estimated state. Require finite components in inclusive [-100, 100] m/s, validated before allocation or sensor draws with a typed InvalidConfig field. A sensor observation is the existing noisy velocity plus the fixed offset. Truth remains unchanged. The filter receives only that observation and the existing assumed R, without a bias correction, covariance inflation, or truth access. Zero-mean Gaussian noise sigma and fixed bias have different meanings; report both explicitly.

Keep the current independent ChaCha8 position/velocity streams and exact draw count/order, including position draws during outages and zero-sigma draws. For exact same-target zero-bias compatibility, skip offset arithmetic when a component equals zero rather than assuming adding signed zero preserves every bit. Capture pre-change CSV, source/lock/toolchain provenance before implementation. Require default and explicit zero runs to preserve baseline CSV bytes on that reference target; make no cross-platform bitwise promise.

## Approved integration scope

Config.velocity_bias is [f64;2], default [0,0], retained in Experiment.config. Complete external Config literals need the new field (explicitly approved 0.1 source impact); default-update literals retain zero bias. CLI --velocity-bias-x and --velocity-bias-y accept signed components in m/s. Reports explicitly distinguish fixed world-frame offset from Gaussian sigma and filter correction. CSV keeps all35columns; companion report carries bias provenance. Service JSON/dashboard locked contracts remain unchanged; the service launches the CLI without bias flags, so its jobs remain zero-bias. No service Config constructor exists. No drift, bias estimation, extra filter state or tuning is added.

## Independent verification

Use a noiseless straight trajectory, no outage, true sigmas zero, assumed velocity sigma 2, and q=0. At tick zero the position update has zero velocity cross-covariance; prior velocity covariance is 4, so K=4/(4+4)=1/2. For bias [2, -1], observation is [3, -0.5] m/s and post-update velocity must be [1.5, -0.25] m/s, with each velocity variance 2. Derive these constants independently; use a tight numerical tolerance rather than copying the implementation formula into an oracle. Truth velocity stays [1, 0.5]. Separately test signed components, finite endpoints, NaN/infinities/out-of-range rejection before output, and default-zero compatibility.

Across nonzero-bias and zero-bias runs with identical seed/config, assert equal sample lengths and every tick's truth, position observations, held-position baseline, outage flags, and measurement schedule. Velocity observations differ by the configured offset within floating-point tolerance; the Gaussian stream itself must be identical. Integrated measured velocity is intentionally affected by bias, because both integration baseline and filter must consume the same biased observations. Do not assert integration equality. Position resets retain their existing semantics. Covariance should remain unchanged when only observation values change in this linear model; error/coverage may change and do not prove calibration.

## Ordered implementation after approval

1. Approve exact scope, input representation/bounds, report/reproduction surface, and separate drift follow-up; capture pre-change artifacts.
2. Add validated constant bias at the velocity observation boundary while preserving streams, default arithmetic, truth, filter model, and baseline data fairness.
3. Add independent first-tick, default, validation, deterministic-stream, schedule, and baseline tests; run actual root checks and unchanged service/dashboard contracts.
4. Record a small prespecified demonstration with bias units, source/config provenance, descriptive error/coverage, and limitations; obtain independent review and exact-head CI in a focused PR. No bias estimation, drift, EKF, tuning, or general performance claim.
