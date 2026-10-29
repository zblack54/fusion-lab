# True sensor noise and assumed measurement covariance

Status: accepted and merged in [PR62](https://github.com/zblack54/fusion-lab/pull/62). Core/library and CLI support the mismatch; the service/dashboard JSON remains unchanged and service-created jobs retain matched-noise behavior. No bias state, EKF or tuning study is added.

## What changed and why

The existing position_sigma and velocity_sigma remain actual per-axis sensor noise. New optional assumed_position_sigma and assumed_velocity_sigma control only the Kalman measurement covariance. None follows the corresponding true sigma independently. The same Config/run path validates all values before sampling or allocating.

A smaller assumed R tells the filter to trust each observation more. In one scalar update, gain=P/(P+R). With prior position variance 25 and assumed sigma 3 m, R=9m², gain 25/34 and posterior variance 225/34. At tick zero the noiseless position observation is0, while velocity observation [1,0.5] with prior variance 4 and assumed sigma 2 m/s gives gain 1/2, estimated velocity [0.5,0.25] and variance 2. These independent rational values are tested through the public run API.

Sensors/RNG code is unchanged: actual draws still use separate ChaCha8 streams, draw during outages and at zero sigma, and preserve position-before-velocity ordering. Changing only assumptions leaves truth, measurements, outage flags and both baselines identical. Config gains two public fields: complete external Rust struct literals must supply them; callers using ..Config::default() retain defaults. This explicit 0.1 source impact was approved by main in [issue #19 approval](https://github.com/zblack54/fusion-lab/issues/19#issuecomment-6472048747).

## Reproduce

~~~sh
cargo run --locked -- --seed 42 --position-sigma 2 --velocity-sigma 0.3 --assumed-position-sigma 1 --assumed-velocity-sigma 0.15 --output results/mismatch.csv
~~~

CSV keeps its 35 columns and records the changed filter state/covariance. The companion report states true/resolved assumed sigmas with units, fallback versus override, requested R and effective R. A zero assumed sigma requests R=0 but the existing positive variance floor 1e-12 still applies; zero is not reported as the effective covariance. Legacy default/nondefault/noiseless CSVs were captured before code at main 080adb5, with lock/toolchain/source hashes preserved outside the candidate. Byte identity was observed on the same locked Rust 1.99.0 Windows x86_64-pc-windows-msvc reference; it is not a cross-platform bitwise guarantee or a Linux golden-file requirement. Their after-change CSV bytes are identical; noise-mismatch-demo.json records all three hash comparisons. Report text adds explicit provenance and is intentionally not byte-identical.

The current service sends no new flags and retains its existing strict JSON schema. To inspect mismatch in the dashboard, generate a CLI CSV then import it locally; its companion report carries noise provenance. Exposing new assumed-noise controls through the service/UI requires its own reviewed contract integration.

## Predeclared demonstration

Main fixed seeds 7, 42, 314159 before code: 30 second maneuver, position outage 10–20 s, true sigmas 2 m/0.3 m/s, q 0.1 m²/s³, assumed scales 0.5/1/2 for both channels. No tuning or selection was performed. All nine runs used identical truth/observations/baselines within each seed. Raw commands, reports, hashes and source provenance are in [noise-mismatch-demo.json](../experiments/noise-mismatch-demo.json). Metrics below are printed precision: RMSE 4 decimals, coverage 2 decimals.

| Seed | Assumed / true sigma | Overall filter RMSE(m) | Outage filter RMSE(m) | Overall nominal95% coverage(%) |
|---|---:|---:|---:|---:|
| 7 | 0.5 | 0.4258 | 0.1751 | 88.07 |
| 7 | 1 | 0.4078 | 0.1740 | 100.00 |
| 7 | 2 | 0.3634 | 0.1710 | 100.00 |
| 42 | 0.5 | 0.5337 | 0.3547 | 57.61 |
| 42 | 1 | 0.5196 | 0.3528 | 100.00 |
| 42 | 2 | 0.4842 | 0.3449 | 100.00 |
| 314159 | 0.5 | 0.8294 | 0.8946 | 19.86 |
| 314159 | 1 | 0.8164 | 0.8932 | 64.81 |
| 314159 | 2 | 0.7815 | 0.8882 | 100.00 |

All three selected seeds have lower RMSE at scale 2 in this demonstration; that is not evidence to inflate R generally. The matched case for seed 314159 has only 64.81% observed coverage, so matching measurement sigmas does not establish calibration of the complete experiment. Other model/prior assumptions remain; this small demonstration does not isolate or tune them. The larger held-out robustness protocol belongs to issue #21.

## Code and verification

src/simulation.rs validates/resolves assumptions and supplies their squares only to filter.update. src/sensors.rs and src/estimation.rs are unchanged. src/main.rs parses optional flags and reports resolved provenance. Tests cover independent tick zero values, channel fallback, invalid/boundary values, exact observation/baseline fairness and no output creation on invalid input. Existing regression assertions remain. Final command evidence and source hashes accompany the PR; product acceptance is main review plus exact-head CI, not this note.

Exercise: hold true noise fixed and double assumed velocity sigma from 2 to 4. Before running, derive the tick zero velocity gain, estimated velocity and posterior variance. Then verify the output without comparing only RMSE. Explain why changing true sigma while omitting its assumed override changes two things at once.
