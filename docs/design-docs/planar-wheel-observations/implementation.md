# Planar wheel model implementation

The pose step uses midpoint heading and sinc to preserve small lateral motion near straight travel. With a 2 m track and wheel travels 0 and π m, the center follows a 1 m radius quarter circle and ends at (1,1), facing π/2. Two metres of equal wheel travel then move the pose to (1,3).

The simulation returns one end pose and one observation per interval. Truth uses ideal wheel travel. Each noisy wheel uses its own deterministic stream. The zero-noise path preserves input bits, including negative zero. A wheel record carries only sequence, interval and signed wheel travel.

Run `cargo test --locked --offline --test wheel_motion`. Seven tests compiled and failed at the named diagnostic seam before implementation. All seven passed after implementation on Windows. The approved contract and tests are unchanged. PR #86 merged at `7c1990d3158ef0fecccd9f9904e10b79ccdb957f` after all 30 hosted checks passed. All seven wheel tests passed on Windows and Linux with Rust 1.99.0 and stable. [Hosted Rust matrix](https://github.com/zblack54/fusion-lab/actions/runs/42106224329). The [issue approval](https://github.com/zblack54/fusion-lab/issues/23#issuecomment-6668853054) records external hashes.

A useful check is to swap the wheel travels in the pivot fixture. The turn and lateral motion reverse, while the forward displacement is unchanged. Wheel observations alone cannot identify global pose, slip or track calibration.
