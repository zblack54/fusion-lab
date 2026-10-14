use nalgebra::{Matrix2, Matrix4, SMatrix, Vector2, Vector4};

use crate::Error;

/// Owns only estimated state and covariance; it has no truth reference.
pub(crate) struct KalmanFilter {
    pub(crate) state: Vector4<f64>,
    pub(crate) covariance: Matrix4<f64>,
    acceleration_density: f64,
}

impl KalmanFilter {
    pub(crate) fn new(acceleration_density: f64) -> Self {
        Self {
            state: Vector4::zeros(),
            covariance: Matrix4::from_diagonal(&Vector4::new(25.0, 25.0, 4.0, 4.0)),
            acceleration_density,
        }
    }

    pub(crate) fn predict(&mut self, dt: f64) -> Result<(), Error> {
        let f = Matrix4::new(
            1.0, 0.0, dt, 0.0, 0.0, 1.0, 0.0, dt, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );
        // Continuous white acceleration: Q = q * integral G(t) G(t)^T dt.
        let q = self.acceleration_density
            * Matrix4::new(
                dt.powi(3) / 3.0,
                0.0,
                dt * dt / 2.0,
                0.0,
                0.0,
                dt.powi(3) / 3.0,
                0.0,
                dt * dt / 2.0,
                dt * dt / 2.0,
                0.0,
                dt,
                0.0,
                0.0,
                dt * dt / 2.0,
                0.0,
                dt,
            );
        self.commit(f * self.state, f * self.covariance * f.transpose() + q)
    }

    pub(crate) fn update(
        &mut self,
        value: [f64; 2],
        variance: f64,
        velocity: bool,
    ) -> Result<(), Error> {
        let h = if velocity {
            SMatrix::<f64, 2, 4>::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0)
        } else {
            SMatrix::<f64, 2, 4>::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0)
        };
        // A small positive floor permits exact synthetic observations without a singular R.
        let r = Matrix2::identity() * variance.max(1e-12);
        let innovation = Vector2::from(value) - h * self.state;
        let cross = self.covariance * h.transpose();
        let s = h * cross + r;
        let factor = s.cholesky().ok_or(Error::NumericalFailure)?;
        let gain = factor.solve(&cross.transpose()).transpose();
        let state = self.state + gain * innovation;
        let a = Matrix4::identity() - gain * h;
        // Joseph form preserves covariance better than (I - KH)P.
        let covariance = a * self.covariance * a.transpose() + gain * r * gain.transpose();
        self.commit(state, covariance)
    }

    fn commit(&mut self, state: Vector4<f64>, covariance: Matrix4<f64>) -> Result<(), Error> {
        let covariance = (covariance + covariance.transpose()) * 0.5;
        if !state.iter().chain(covariance.iter()).all(|v| v.is_finite())
            || covariance.cholesky().is_none()
        {
            return Err(Error::NumericalFailure);
        }
        self.state = state;
        self.covariance = covariance;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_scalar_measurement_update() -> Result<(), Error> {
        let mut filter = KalmanFilter::new(0.0);
        filter.update([10.0, -5.0], 25.0, false)?;
        // Prior P=25, R=25 => gain=1/2, posterior variance=12.5.
        assert!((filter.state[0] - 5.0).abs() < 1e-12);
        assert!((filter.state[1] + 2.5).abs() < 1e-12);
        assert!((filter.covariance[(0, 0)] - 12.5).abs() < 1e-12);
        assert_eq!(filter.state[2], 0.0);
        Ok(())
    }

    #[test]
    fn prediction_transports_velocity_and_covariance() -> Result<(), Error> {
        let mut filter = KalmanFilter::new(0.0);
        filter.state = Vector4::new(1.0, 2.0, 3.0, -4.0);
        filter.predict(2.0)?;
        assert_eq!(filter.state, Vector4::new(7.0, -6.0, 3.0, -4.0));
        assert_eq!(filter.covariance[(0, 0)], 41.0);
        assert_eq!(filter.covariance[(0, 2)], 8.0);
        Ok(())
    }

    #[test]
    fn white_acceleration_covariance_has_correct_units() -> Result<(), Error> {
        let mut filter = KalmanFilter::new(3.0);
        filter.predict(2.0)?;
        assert_eq!(filter.covariance[(0, 0)], 49.0);
        assert_eq!(filter.covariance[(0, 2)], 14.0);
        assert_eq!(filter.covariance[(2, 2)], 10.0);
        Ok(())
    }

    #[test]
    fn failed_update_preserves_filter_state() {
        let mut filter = KalmanFilter::new(0.1);
        let prior = filter.covariance;
        assert_eq!(
            filter.update([f64::NAN, 0.0], 1.0, false),
            Err(Error::NumericalFailure)
        );
        assert_eq!(filter.state, Vector4::zeros());
        assert_eq!(filter.covariance, prior);
    }
}
