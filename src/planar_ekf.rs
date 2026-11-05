//! Planar extended Kalman filter for wheel travel and position fixes.
use crate::wheel_motion::{Pose, WheelTravel};
use nalgebra::{SMatrix, SVector};
use std::f64::consts::{PI, TAU};

type Mat3 = SMatrix<f64, 3, 3>;
type Mat2 = SMatrix<f64, 2, 2>;

/// Planar prediction mean and local Jacobians.
pub struct Motion {
    pub pose: Pose,
    pub state_jacobian: [[f64; 3]; 3],
    pub wheel_jacobian: [[f64; 2]; 3],
}

/// EKF input or numerical failure.
#[derive(Debug, PartialEq, Eq)]
pub enum EkfError {
    Pose,
    TrackWidth,
    Travel,
    WheelVariance,
    StateCovariance,
    Measurement,
    MeasurementCovariance,
    NumericalFailure,
}

fn bounded(value: f64, limit: f64) -> bool {
    (-limit..=limit).contains(&value)
}

fn valid_pose(p: Pose) -> bool {
    bounded(p.x, 1e9) && bounded(p.y, 1e9) && bounded(p.heading, PI)
}

fn heading(angle: f64) -> f64 {
    let angle = angle % TAU;
    if angle >= PI {
        angle - TAU
    } else if angle < -PI {
        angle + TAU
    } else {
        angle
    }
}

fn sinc_pair(a: f64) -> (f64, f64) {
    let z = a * a;
    if a.abs() <= 1e-3 {
        (
            1.0 + z * (-1.0 / 6.0 + z * (1.0 / 120.0 - z / 5040.0)),
            a * (-1.0 / 3.0 + z * (1.0 / 30.0 - z / 840.0)),
        )
    } else {
        let (sin, cos) = a.sin_cos();
        (sin / a, (a * cos - sin) / z)
    }
}

/// Integrate wheel observations and return derivatives in local tangent coordinates.
pub fn motion(pose: Pose, track_width: f64, travel: WheelTravel) -> Result<Motion, EkfError> {
    if !valid_pose(pose) {
        return Err(EkfError::Pose);
    }
    if !(0.01..=100.0).contains(&track_width) {
        return Err(EkfError::TrackWidth);
    }
    if !bounded(travel.left, 128.0) || !bounded(travel.right, 128.0) {
        return Err(EkfError::Travel);
    }
    let s = (travel.right + travel.left) / 2.0;
    let a = (travel.right - travel.left) / (2.0 * track_width);
    let (g, derivative) = sinc_pair(a);
    let (sin, cos) = (pose.heading + a).sin_cos();
    let distance = s * g;
    let end = Pose {
        x: pose.x + distance * cos,
        y: pose.y + distance * sin,
        heading: heading(pose.heading + 2.0 * a),
    };
    if !valid_pose(end) {
        return Err(EkfError::NumericalFailure);
    }
    let wheel_column = |da: f64| {
        [
            g * cos / 2.0 + s * da * (derivative * cos - g * sin),
            g * sin / 2.0 + s * da * (derivative * sin + g * cos),
            2.0 * da,
        ]
    };
    let left = wheel_column(-1.0 / (2.0 * track_width));
    let right = wheel_column(1.0 / (2.0 * track_width));
    Ok(Motion {
        pose: end,
        state_jacobian: [
            [1.0, 0.0, -distance * sin],
            [0.0, 1.0, distance * cos],
            [0.0, 0.0, 1.0],
        ],
        wheel_jacobian: std::array::from_fn(|i| [left[i], right[i]]),
    })
}

fn valid_entries<const N: usize>(matrix: &[[f64; N]; N]) -> bool {
    (0..N).all(|i| (0..N).all(|j| bounded(matrix[i][j], 1e12) && matrix[i][j] == matrix[j][i]))
}

fn rows(matrix: &Mat3) -> [[f64; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|j| matrix[(i, j)]))
}

/// Owns the planar mean and positive-definite tangent covariance.
pub struct PlanarEkf {
    pose: Pose,
    covariance: Mat3,
}

impl PlanarEkf {
    /// Validate the initial pose and covariance.
    pub fn new(mut pose: Pose, covariance: [[f64; 3]; 3]) -> Result<Self, EkfError> {
        if !valid_pose(pose) {
            return Err(EkfError::Pose);
        }
        let matrix = Mat3::from_fn(|i, j| covariance[i][j]);
        if !valid_entries(&covariance) || matrix.cholesky().is_none() {
            return Err(EkfError::StateCovariance);
        }
        pose.heading = heading(pose.heading);
        Ok(Self {
            pose,
            covariance: matrix,
        })
    }

    /// Current mean.
    pub fn pose(&self) -> Pose {
        self.pose
    }

    /// Current tangent covariance as row-major values.
    pub fn covariance(&self) -> [[f64; 3]; 3] {
        rows(&self.covariance)
    }

    /// Predict with independent per-interval wheel travel variances.
    pub fn predict(
        &mut self,
        track_width: f64,
        travel: WheelTravel,
        wheel_variance: [f64; 2],
    ) -> Result<(), EkfError> {
        let m = motion(self.pose, track_width, travel)?;
        if wheel_variance.iter().any(|v| !(0.0..=100.0).contains(v)) {
            return Err(EkfError::WheelVariance);
        }
        let f = Mat3::from_fn(|i, j| m.state_jacobian[i][j]);
        let g = SMatrix::<f64, 3, 2>::from_fn(|i, j| m.wheel_jacobian[i][j]);
        let noise = Mat2::from_diagonal(&SVector::from(wheel_variance));
        self.commit(
            m.pose,
            f * self.covariance * f.transpose() + g * noise * g.transpose(),
        )
    }

    /// Correct the mean with a position fix and its full covariance.
    pub fn update_position(
        &mut self,
        measurement: [f64; 2],
        covariance: [[f64; 2]; 2],
    ) -> Result<(), EkfError> {
        if measurement.iter().any(|&v| !bounded(v, 1e9)) {
            return Err(EkfError::Measurement);
        }
        let r = Mat2::from_fn(|i, j| covariance[i][j]);
        if !valid_entries(&covariance) || r.cholesky().is_none() {
            return Err(EkfError::MeasurementCovariance);
        }
        let h = SMatrix::<f64, 2, 3>::from_row_slice(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        let hp = h * self.covariance;
        let innovation = hp * h.transpose() + r;
        let gain = innovation
            .cholesky()
            .ok_or(EkfError::NumericalFailure)?
            .solve(&hp)
            .transpose();
        let correction = gain
            * SVector::<f64, 2>::new(measurement[0] - self.pose.x, measurement[1] - self.pose.y);
        let end = Pose {
            x: self.pose.x + correction[0],
            y: self.pose.y + correction[1],
            heading: heading(self.pose.heading + correction[2]),
        };
        let residual = Mat3::identity() - gain * h;
        self.commit(
            end,
            residual * self.covariance * residual.transpose() + gain * r * gain.transpose(),
        )
    }

    fn commit(&mut self, pose: Pose, candidate: Mat3) -> Result<(), EkfError> {
        let covariance = (candidate + candidate.transpose()) * 0.5;
        if !valid_pose(pose)
            || !valid_entries(&rows(&covariance))
            || covariance.cholesky().is_none()
        {
            return Err(EkfError::NumericalFailure);
        }
        self.pose = pose;
        self.covariance = covariance;
        Ok(())
    }
}
