# Fusion Lab

Reproducible Rust experiments that estimate a moving robot's position from noisy position and velocity sensors, then measure what happens when its position beacon goes offline.

The v0.1 CLI compares a linear Kalman filter with last-position hold and measured-velocity integration. It records ground truth, observations, estimates, and full covariance at 100 Hz. A browser dashboard and asynchronous sensor delivery are planned for v0.2 and v0.3.

## Quick start

Install Rust 1.99 or newer with a native linker (Visual Studio C++ Build Tools on Windows).

```sh
git clone https://github.com/zblack54/fusion-lab.git
cd fusion-lab
cargo run --locked -- --seed 42 --output results/run.csv
```

The default experiment runs 30 simulated seconds, with a position outage from 10 through just before 20 seconds. It writes `results/run.csv` and `results/run.txt`, and prints overall and outage-only position RMSE for all three estimators. Existing output files are overwritten.

```sh
# Compare against healthy sensors using the same observations outside the outage.
cargo run --locked -- --seed 42 --no-outage --output results/healthy.csv

# Exact observations for a straight-motion numerical sanity check.
cargo run --locked -- --scenario straight --no-outage --position-sigma 0 --velocity-sigma 0 --acceleration-density 0 --output results/noiseless.csv

cargo run --locked -- --help
```

## Model and scope

- World-frame state `[px, py, vx, vy]`, in metres and metres per second.
- 100 Hz truth and prediction, 5 Hz position observations, 20 Hz velocity observations.
- Independent seeded Gaussian sensor noise and an explicit integer virtual clock.
- Constant-velocity Kalman model with continuous white acceleration process uncertainty, Cholesky solves, and Joseph covariance updates.
- Scripted straight or accelerating/turning motion. No path planning or collision model.

These sensors are idealized world-frame observations; the velocity sensor is not a physical wheel-encoder model. Reported confidence-ellipse coverage is descriptive, not proof of covariance calibration. Repeated runs are deterministic under the same lockfile, toolchain, target, and configuration; bitwise agreement across platforms or dependency changes is not promised.

## Documentation and development

Read the [v0.1 design contract](docs/design/v0.1.md), [experiment guide](docs/experiments.md), and [development guide](docs/development.md). Generate the public API reference with `cargo doc --locked --no-deps --open`.

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --doc
cargo doc --locked --no-deps
```

Optional: install `cargo-make` and run `cargo make pr-check`. To build the MkDocs site, run `python scripts/docs.py build`; use `serve` for local preview. Python 3.10+ and network access are required on its first run.

Contributor and agent instructions are in [AGENTS.md](AGENTS.md). The owner can additionally use their private Rust skill suite locally; it is not required to build or contribute.

## License

MIT. See [LICENSE](LICENSE).
