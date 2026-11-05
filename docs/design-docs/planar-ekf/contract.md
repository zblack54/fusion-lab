# Planar EKF executable contract (#24)

Draft for independent review. The filter state is planar x and y in metres and heading in radians. Prediction consumes two signed wheel travel observations in metres. Correction consumes a position fix in metres. Truth is available only to simulation and evaluation. The filter is synchronous and callers own ordering and timestamps.

## Public boundary

The `planar_ekf` module reuses `wheel_motion::{Pose, WheelTravel}`. It exposes:

- `motion(pose: Pose, track_width: f64, travel: WheelTravel) -> Result<Motion, EkfError>`. `Motion` has `pose: Pose`, `state_jacobian: [[f64; 3]; 3]`, and `wheel_jacobian: [[f64; 2]; 3]`.
- `PlanarEkf::new(pose: Pose, covariance: [[f64; 3]; 3]) -> Result<Self, EkfError>`, plus read-only `pose()` and `covariance()` returning copies.
- `predict(&mut self, track_width: f64, travel: WheelTravel, wheel_variance: [f64; 2]) -> Result<(), EkfError>`.
- `update_position(&mut self, measurement: [f64; 2], covariance: [[f64; 2]; 2]) -> Result<(), EkfError>`.

Coordinates and position fixes are finite within ±1e9 m. Input headings are in [−π,π] and stored headings in [−π,π). Track width is finite in [0.01,100] m. Wheel observations may exceed the simulator's 100 m ideal input bound. The filter accepts up to 128 m magnitude per wheel. Values beyond that are rejected rather than clipped. Wheel variances are independent per interval and in [0,100] m². Initial covariance and fix covariance must be exactly symmetric, have finite entries with magnitude ≤1e12 in their respective mixed units, and pass a Cholesky factorization. Symmetry compares numeric values, so signed zeros are equal. No diagonal jitter or covariance repair on input.

`EkfError` variants are Pose, TrackWidth, Travel, WheelVariance, StateCovariance, Measurement, MeasurementCovariance and NumericalFailure. Validate motion arguments in pose, width, travel order. Prediction validates and computes motion before wheel variance. A motion result outside the pose domain takes precedence over invalid wheel variance. Update validates measurement then measurement covariance. Invalid prior covariance at construction returns StateCovariance. A computed state outside its domain or a computed covariance outside its bounds or failing Cholesky returns NumericalFailure. Both mutating methods commit only after all candidate checks pass, preserving prior pose and covariance bitwise on failure.

## Mathematics

Let s=(r+l)/2, a=(r−l)/(2b), g=sin(a)/a and φ=h+a. With u=(cos φ,sin φ) and v=(−sin φ,cos φ), position advances by s g u. Normalize h+2a only after computing the local transition. At |a|≤1e−3 evaluate g=1−a²/6+a⁴/120−a⁶/5040 and g'=−a/3+a³/30−a⁵/840. Else use g=sin(a)/a and g'=(a cos(a)−sin(a))/a². The state Jacobian is identity except for the position/heading column s g v. Wheel column j has position derivative (g/2)u+s a_j(g'u+gv) and heading derivative 2a_j, with a_left=−1/(2b) and a_right=+1/(2b).

Predict P'=F P Fᵀ+G diag(wheel_variance) Gᵀ. Position H selects x and y. Use a Cholesky innovation solve and Joseph form for updated covariance. For computed covariance, average mirrored entries before validating and storing. Reject nonfinite outputs. Angle normalization uses signed remainder and range adjustment. Covariance is in local tangent coordinates and does not wrap.

## Verification and evaluation

Run cargo test --locked --test planar_ekf on Windows and Linux. The first acceptance slice covers exact straight mean and unequal-wheel covariance, finite-difference derivatives on curved and near-straight motion, heading wrap, stationary and coupled position updates, validation, failure atomicity, and repeated curved covariance checks. Absolute fixture tolerance is 1e−12 except quadrature reference means (2e−10 absolute) and finite differences (2e−7 absolute). Exact invariants use bit comparisons. The contract and executable tests are frozen before behavior.

Issue #24 also requires a held-out comparison before final feature acceptance. That evaluation has its own frozen protocol, identical wheel and fix data for the equal-information comparator, and a wheel-only information ablation. Report position and wrapped heading error and consistency, with weak-observability and simple-motion cases. No superiority claim follows from the filter contract tests.
