use nalgebra::{Matrix2, Vector2};

use crate::{Error, Sample};

/// Position error statistics over an explicitly counted collection of ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Number of post-update snapshots evaluated.
    pub samples: usize,
    /// sqrt(mean(dx² + dy²)), in metres.
    pub filter_rmse: f64,
    /// Same metric for last-position hold.
    pub hold_rmse: f64,
    /// Same metric for velocity integration with position resets.
    pub integration_rmse: f64,
    /// Fraction inside the filter's nominal 95% joint position ellipse.
    /// This is descriptive; a single run does not establish calibration.
    pub position_coverage: f64,
}

/// Overall and outage-only metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    /// All snapshots, including initialization.
    pub overall: Metrics,
    /// Unavailable position-beacon ticks, or None when no outage exists.
    pub outage: Option<Metrics>,
}

pub(crate) fn summarize(samples: &[Sample]) -> Result<Summary, Error> {
    let overall = metrics(samples.iter())?.ok_or(Error::NumericalFailure)?;
    let outage = metrics(samples.iter().filter(|s| s.position_outage))?;
    Ok(Summary { overall, outage })
}

fn metrics<'a>(samples: impl Iterator<Item = &'a Sample>) -> Result<Option<Metrics>, Error> {
    let mut count = 0;
    let mut squared = [0.0; 3];
    let mut covered = 0;
    for sample in samples {
        count += 1;
        for (i, position) in [
            sample.estimate.position,
            sample.held_position,
            sample.integrated_position,
        ]
        .into_iter()
        .enumerate()
        {
            squared[i] += (position[0] - sample.truth.position[0]).powi(2)
                + (position[1] - sample.truth.position[1]).powi(2);
        }
        let p = Matrix2::new(
            sample.covariance[0][0],
            sample.covariance[0][1],
            sample.covariance[1][0],
            sample.covariance[1][1],
        );
        let error = Vector2::new(
            sample.estimate.position[0] - sample.truth.position[0],
            sample.estimate.position[1] - sample.truth.position[1],
        );
        let factor = p.cholesky().ok_or(Error::NumericalFailure)?;
        let distance_squared = error.dot(&factor.solve(&error));
        // Chi-square 95th percentile with 2 degrees of freedom.
        if distance_squared <= 5.991_464_547_107_979 {
            covered += 1;
        }
    }
    Ok((count > 0).then(|| Metrics {
        samples: count,
        filter_rmse: (squared[0] / count as f64).sqrt(),
        hold_rmse: (squared[1] / count as f64).sqrt(),
        integration_rmse: (squared[2] / count as f64).sqrt(),
        position_coverage: covered as f64 / count as f64,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::State;

    #[test]
    fn metrics_use_euclidean_error_and_joint_ellipse() -> Result<(), Error> {
        let sample = Sample {
            tick: 0,
            truth: State {
                position: [0.0; 2],
                velocity: [0.0; 2],
            },
            estimate: State {
                position: [3.0, 4.0],
                velocity: [0.0; 2],
            },
            covariance: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            position_measurement: None,
            velocity_measurement: None,
            held_position: [0.0, 2.0],
            integrated_position: [1.0, 0.0],
            position_outage: false,
        };
        let summary = summarize(&[sample])?;
        assert_eq!(summary.overall.filter_rmse, 5.0);
        assert_eq!(summary.overall.hold_rmse, 2.0);
        assert_eq!(summary.overall.integration_rmse, 1.0);
        assert_eq!(summary.overall.position_coverage, 0.0);
        assert_eq!(summary.outage, None);
        Ok(())
    }
}
