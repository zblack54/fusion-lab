# Planar EKF implementation

The filter predicts position and heading from signed wheel travel, then corrects position using the full fix covariance. Its motion Jacobians map pose uncertainty and independent wheel travel variance into the next tangent covariance. For small turns the sinc derivative uses a series, which avoids subtracting nearly equal trigonometric values.

The correction solves the innovation system with Cholesky and uses the Joseph covariance form. The state changes only after the candidate pose and covariance pass validation. A stationary position fix with zero heading cross-covariance leaves heading unchanged. With cross-covariance, the same position residual can also change heading.

Run `cargo test --locked --offline --test planar_ekf` to reproduce the seven acceptance cases. Before implementation all seven reached the diagnostic seam. All seven passed on Windows with the frozen tests after implementation. Hosted qualification is pending. The held-out evaluation remains separate and requires a prespecified equal-information comparator.

Try swapping the left and right variances in the straight prediction fixture. The sign of the position-heading cross-term contributed by wheel noise changes. The filter does not estimate slip or wheel bias.

Clippy flagged index-only iteration in the symmetry test. The [independently reviewed editorial revision](https://github.com/zblack54/fusion-lab/issues/24#issuecomment-6669322607) preserves all nine ordered bit comparisons. The original approved test is retained outside the candidate.
