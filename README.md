# Fusion Lab

Reproducible Rust experiments that estimate a moving robot's position from noisy position and velocity sensors, then measure what happens when its position beacon goes offline.

The v0.1 CLI compares a linear Kalman filter with last-position hold and measured-velocity integration. It records ground truth, observations, estimates, and full covariance at 100 Hz. The local application adds recorded-result inspection and replay; asynchronous sensor delivery remains planned for v0.3.

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

## Local browser application

Build the numerical CLI first, then start the loopback service from the repository root:

```sh
cargo build --locked
cargo run --manifest-path crates/experiment-service/Cargo.toml --locked -- --bind 127.0.0.1:8080
```

Open [the local dashboard](http://127.0.0.1:8080/). Create a healthy or outage experiment, select a completed run, or import a local CSV without uploading it. Playback controls move over recorded samples; they do not pause or restart the numerical job. Cancel is a separate explicit action. Position observations appear on the XY plot; exact velocity observations and sensor status are available in the sample panel.

The service requires an empty artifact root and serves transient results. Ctrl+C cancels/reaps owned work and shuts it down. Inspect leftovers after a crash or choose a new empty `--artifact-root`; do not delete an unknown directory to make startup pass. Read the [service guide](crates/experiment-service/README.md) and [dashboard lesson](docs/learn/dashboard.md) for limits, evidence and local review instructions.

## Model and scope

- World-frame state `[px, py, vx, vy]`, in metres and metres per second.
- 100 Hz truth and prediction, 5 Hz position observations, 20 Hz velocity observations.
- Independent seeded Gaussian sensor noise and an explicit integer virtual clock.
- Constant-velocity Kalman model with continuous white acceleration process uncertainty, Cholesky solves, and Joseph covariance updates.
- Scripted straight or accelerating/turning motion. No path planning or collision model.

These sensors are idealized world-frame observations; the velocity sensor is not a physical wheel-encoder model. Reported confidence-ellipse coverage is descriptive, not proof of covariance calibration. Repeated runs are deterministic under the same lockfile, toolchain, target, and configuration; bitwise agreement across platforms or dependency changes is not promised.

## Documentation and development

Start with the [15-minute learning path](docs/learn/index.md), [math and code tour](docs/learn/math-and-code.md), and [tracked roadmap](docs/roadmap.md). The [research guide](docs/learn/research.md) connects reproducible studies to their results and limitations. The documentation portal is available for local preview; the owner manages public hosting.

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
