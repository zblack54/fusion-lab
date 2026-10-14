# v0.1 validation evidence

Local validation performed on Windows x86_64 using Rust 1.99.0, Cargo 1.99.0, and Python 3.12. GitHub CI separately checks Linux and Windows on Rust 1.99.0 and stable; those remote jobs have not yet been observed.

## Numerical and integration evidence

Fourteen unit/integration tests and one runnable documentation example passed. Tests cover independently calculated filter updates and process covariance, exact noiseless motion, scripted acceleration, deterministic runs, independent noise streams, sample rates, half-open outage boundaries, covariance validity, known metric values, invalid inputs, and CSV schema. An eight-seed regression compares outage RMSE against both baselines.

## Seed 42 demonstration

Command:

```sh
cargo run --locked -- --seed 42 --output results/run.csv
```

| Window | Samples | Filter RMSE (m) | Hold RMSE (m) | Integration RMSE (m) |
|---|---:|---:|---:|---:|
| Entire run | 3001 | 0.2667 | 7.7399 | 1.2877 |
| Position outage `[10,20)` s | 1000 | 0.1769 | 13.2392 | 0.8181 |

Nominal 95% position-ellipse coverage is 100% in this particular run. This suggests conservative uncertainty on this trajectory; it does not establish calibration or universal superiority. The truth trajectory is scripted, while the filter models uncertain acceleration.

## Tooling boundary

The execution environment's native Cargo HTTPS client encountered a Windows Schannel credential error. Dependencies were fetched using a workspace-only loopback registry adapter whose Python client verifies upstream HTTPS; Cargo still verifies crate checksums. The adapter is outside this repository. Cargo.lock retains canonical crates.io sources. No TLS verification was disabled.

## Gate results

All local gates passed: formatting check, compilation of all targets, Clippy with warnings denied, 14 unit/integration tests, one doctest, Rust API documentation, and a strict MkDocs build (MkDocs 1.6.1 / Material 9.7.7). The documented healthy-sensor and noiseless straight-motion CLI runs also succeeded. The cargo-make aggregate itself was not executed; its individual Cargo gates were.

The local Cargo commands used `--offline` after verified dependency downloads. No Windows TLS adapter or machine-specific Cargo configuration is committed to the project. Remote CI and non-Windows runtime behavior remain to be confirmed by the PR checks.
