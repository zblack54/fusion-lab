# From equations to the Rust loop

This tour explains the existing linear filter. It is not an EKF, a heading model,
a wheel-encoder model, or a learned correction. The authoritative numerical
contract is [v0.1](../design/v0.1.md); source links below point to repository main.
For historical evidence, use the revision recorded by the relevant study or PR.

## 1. Separate truth, observation, and estimate

The state is x = [px, py, vx, vy]^T. Positions are in metres and velocities in m/s.
The simulator knows truth; the filter stores only an estimated state and a 4x4
covariance P. A sensor adds independent Gaussian noise per world-frame axis.
Velocity observations here are already world-frame velocities, not wheel ticks.

Read [simulation.rs](https://github.com/zblack54/fusion-lab/blob/main/src/simulation.rs):
Config validates duration/noise/outage before allocation, run owns the tick loop,
and advance_truth applies the scripted acceleration. Read
[sensors.rs](https://github.com/zblack54/fusion-lab/blob/main/src/sensors.rs):
Sensors owns independent ChaCha8 streams. Scheduled position noise is drawn even
when delivery is suppressed, preserving paired comparisons after the outage.

Truth starts at position [0,0], velocity [1,0.5]. The filter starts with zero
state and diagonal covariance [25,25,4,4]. Estimation therefore is not initialized
from truth. For a constant acceleration over one interval, truth integration is:

~~~text
p_next = p + v*dt + 0.5*a*dt^2
v_next = v + a*dt
~~~

Exercise: during the maneuver, x-acceleration is 0.4 m/s^2 over [5,8) s.
Starting from x=5 m and vx=1 m/s at 5 s, compute x and vx at 8 s.
The independent expectations are 9.8 m and 2.2 m/s; compare
maneuver_truth_integrates_scripted_acceleration in
[experiments.rs](https://github.com/zblack54/fusion-lab/blob/main/tests/experiments.rs).

## 2. Predict the estimated motion and uncertainty

In [estimation.rs](https://github.com/zblack54/fusion-lab/blob/main/src/estimation.rs),
KalmanFilter::predict uses the constant-velocity transition:

~~~text
    [1 0 dt 0 ]
F = [0 1 0  dt]
    [0 0 1  0 ]
    [0 0 0  1 ]
x_prior = F*x_previous
P_prior = F*P_previous*F^T + Q
~~~

Why add Q? Estimated velocity cannot predict unknown acceleration exactly.
Continuous white acceleration density q has units m^2/s^3. A velocity impulse
accumulated over the remaining interval u affects position by u and velocity by
1. Integrating the outer product [u,1]^T[u,1] from 0 to dt gives each axis block:

~~~text
Q_axis = q * integral_0^dt [u^2 u; u 1] du
       = q * [dt^3/3  dt^2/2]
             [dt^2/2 dt    ]
~~~

Place that block at indices (px,vx) and (py,vy) in the four-state matrix.
Its position diagonal has units m^2; velocity diagonal has units m^2/s^2;
the cross term has units m^2/s. q is not the robot's scripted acceleration.
The filter keeps the same motion model during turns; that mismatch matters.

Hand check: with prior position variance 25, velocity variance 4, zero cross
covariance, dt=2 s and q=3, predicted position variance is
25 + 4*2^2 + 3*2^3/3 = 49. Position/velocity covariance is
4*2 + 3*2^2/2 = 14; velocity variance is 4 + 3*2 = 10.
The white_acceleration_covariance_has_correct_units test uses these values.
Normal simulation steps use dt=0.01, not 2 s.

## 3. Correct using the observation

Position observation matrix H selects px,py; velocity H selects vx,vy.
Measurement covariance R = sigma^2 I. The implementation floors the variance at
1e-12 in its squared units for exact synthetic observations.

~~~text
y = observation - H*x_prior       (innovation)
S = H*P_prior*H^T + R             (innovation covariance)
K = P_prior*H^T*S^-1              (gain, expressed mathematically)
x_posterior = x_prior + K*y
A = I - K*H
P_posterior = A*P_prior*A^T + K*R*K^T
~~~

The code computes K through a Cholesky solve, without explicitly constructing
S^-1. The last equation is Joseph form. KalmanFilter::commit symmetrizes the
candidate covariance, checks finite state/covariance and positive definiteness,
then commits both together. A failed numerical update preserves prior state.

Scalar oracle: P=25, R=25 gives gain 1/2. A zero estimate with position observation
10 becomes 5, and posterior variance becomes 12.5. This independently explains
known_scalar_measurement_update. It is easier to assess than an expected value
copied from the implementation's output.

## 4. Locate time and ownership

run processes ticks 0..duration*100 inclusive. Except at zero, it advances truth
and predicts, then applies a position measurement if due, then velocity, then
records and evaluates. Position arrives every 20 ticks (5 Hz), velocity every
5 ticks (20 Hz). The outage is half-open: [start,end). No wall clock, network,
or async scheduler influences this loop.

[main.rs](https://github.com/zblack54/fusion-lab/blob/main/src/main.rs) converts CLI
arguments to Config, validates the .csv extension before run, and writes CSV
and a companion report. Existing files are overwritten. Filesystem failure can
leave partial output; this is not atomic publication.
[lib.rs](https://github.com/zblack54/fusion-lab/blob/main/src/lib.rs) exposes Config,
run, and typed errors; generate its API reference with cargo doc --locked --no-deps.

## 5. Evaluate without feeding truth back

[evaluation.rs](https://github.com/zblack54/fusion-lab/blob/main/src/evaluation.rs)
computes Euclidean position RMSE: sqrt(mean(dx^2+dy^2)), including tick zero.
Hold uses the latest position observation; integration uses the previous tick's
measured velocity and resets on position observations. Neither uses truth.

For error e and the 2x2 position block Ppos, nominal ellipse inclusion is
 e^T Ppos^-1 e <= 5.991464547107979. A Cholesky solve evaluates this expression.
Coverage is the fraction included, not a calibration certificate. Temporally
correlated ticks are not independent trials; the
[study protocol](../experiments/outage-study-protocol.md) resamples seeds instead.

Exercise: a single position error [3,4] has Euclidean magnitude 5 m, not
sqrt((3^2+4^2)/2). With identity position covariance it lies outside the nominal
95% joint ellipse because its squared Mahalanobis distance is 25. Compare
metrics_use_euclidean_error_and_joint_ellipse before changing any evaluator.
