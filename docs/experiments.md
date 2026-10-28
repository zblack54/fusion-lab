# Running experiments

## Default fault scenario

```sh
cargo run --locked -- --seed 42 --output results/run.csv
```

The robot accelerates, turns during the position outage, and coasts. Position observations stop over `[10,20)` seconds; velocity observations continue. Inspect full covariance in the CSV to see uncertainty grow, then shrink as position measurements resume.

Compare healthy delivery using the same seed:

```sh
cargo run --locked -- --seed 42 --no-outage --output results/healthy.csv
```

Both runs share the same truth and velocity observations. Position observations match outside the outage. This controls random inputs when comparing a fault policy.

## Change model assumptions

```sh
cargo run --locked -- --acceleration-density 0.01 --output results/low-q.csv
cargo run --locked -- --position-sigma 3 --output results/noisy-position.csv
cargo run --locked -- --outage-start 5 --outage-end 25 --output results/long-outage.csv
```

Changing process density adjusts the filter's motion uncertainty, not the robot's acceleration. The existing sensor sigmas control actual noise and, when no assumed override is supplied, the matching filter covariance. Use `--assumed-position-sigma` and `--assumed-velocity-sigma` to change only estimator assumptions; each omitted override follows its own true sigma. For example, `--position-sigma 2 --assumed-position-sigma 1` generates 2 m noise but assumes 1 m. The [noise-mismatch lesson](learn/noise-mismatch.md) explains provenance, independent tests and the small fixed-seed demonstration. Service-created jobs retain matched noise; use CLI output/local CSV import for this initial feature.

Use `--no-outage` for short experiments whose duration is below the default outage end. All durations and outage endpoints are whole seconds. `--output` requires a `.csv` extension; existing CSV and companion `.txt` files are overwritten.

## Interpret results

RMSE is Euclidean position error in metres; smaller is better. Compare both overall and outage-only errors. The hold baseline stops moving between beacon updates. The integration baseline follows measured velocity and resets on beacon updates.

Coverage is the fraction of post-update errors inside the nominal joint 95% position ellipse. Temporal correlation, prior mismatch, and scripted acceleration mean one run cannot validate calibration. Future Monte Carlo experiments should include model mismatch and report statistical uncertainty.

CSV measurements are blank on unsampled or unavailable ticks. Covariance `pRC` uses zero-based row/column indices in state order `[px,py,vx,vy]`. The position submatrix is `p00,p01,p10,p11`.
