use fusion_lab::{Config, Error, Outage, Scenario, run};
use nalgebra::Matrix4;

#[test]
fn seeded_runs_are_identical_and_other_seeds_change_observations() -> Result<(), Error> {
    let config = Config::default();
    let first = run(&config)?;
    assert_eq!(first, run(&config)?);
    let second = run(&Config { seed: 43, ..config })?;
    assert_ne!(
        first.samples[0].position_measurement,
        second.samples[0].position_measurement
    );
    assert_eq!(first.samples[100].truth, second.samples[100].truth);
    Ok(())
}

#[test]
fn sample_rates_and_outage_boundaries_are_exact() -> Result<(), Error> {
    let result = run(&Config::default())?;
    assert_eq!(result.samples.len(), 3001);
    assert_eq!(
        result
            .samples
            .iter()
            .filter(|s| s.position_measurement.is_some())
            .count(),
        101
    );
    assert_eq!(
        result
            .samples
            .iter()
            .filter(|s| s.velocity_measurement.is_some())
            .count(),
        601
    );
    assert_eq!(
        result.summary.outage.expect("configured outage").samples,
        1000
    );
    assert!(!result.samples[999].position_outage);
    assert!(result.samples[1000].position_outage);
    assert_eq!(result.samples[1000].position_measurement, None);
    assert!(result.samples[1999].position_outage);
    assert!(!result.samples[2000].position_outage);
    assert!(result.samples[2000].position_measurement.is_some());
    Ok(())
}

#[test]
fn noiseless_straight_motion_matches_analytical_solution() -> Result<(), Error> {
    let result = run(&Config {
        scenario: Scenario::Straight,
        position_sigma: 0.0,
        velocity_sigma: 0.0,
        acceleration_density: 0.0,
        outage: None,
        ..Config::default()
    })?;
    for sample in &result.samples {
        let time = f64::from(sample.tick) / 100.0;
        assert!((sample.truth.position[0] - time).abs() < 1e-9);
        assert!((sample.truth.position[1] - 0.5 * time).abs() < 1e-9);
        assert!((sample.estimate.position[0] - time).abs() < 1e-8);
        assert!((sample.estimate.velocity[0] - 1.0).abs() < 1e-8);
    }
    assert!(result.summary.overall.filter_rmse < 1e-8);
    Ok(())
}

#[test]
fn maneuver_truth_integrates_scripted_acceleration() -> Result<(), Error> {
    let result = run(&Config::default())?;
    // At 8 s: x=5 + 1*3 + (0.4/2)*3^2 = 9.8 m; vx=2.2 m/s.
    assert!((result.samples[800].truth.position[0] - 9.8).abs() < 1e-9);
    assert!((result.samples[800].truth.velocity[0] - 2.2).abs() < 1e-9);
    assert!((result.samples[1800].truth.velocity[1] - 1.7).abs() < 1e-9);
    Ok(())
}

#[test]
fn outages_do_not_perturb_sensor_random_streams() -> Result<(), Error> {
    let faulty = run(&Config::default())?;
    let healthy = run(&Config {
        outage: None,
        ..Config::default()
    })?;
    for (a, b) in faulty.samples.iter().zip(&healthy.samples) {
        assert_eq!(a.velocity_measurement, b.velocity_measurement);
        if !a.position_outage {
            assert_eq!(a.position_measurement, b.position_measurement);
        }
    }
    Ok(())
}

#[test]
fn covariance_stays_symmetric_positive_definite_and_grows_during_outage() -> Result<(), Error> {
    let result = run(&Config::default())?;
    for sample in &result.samples {
        let p = Matrix4::from_fn(|r, c| sample.covariance[r][c]);
        assert!((p - p.transpose()).norm() < 1e-12);
        assert!(p.cholesky().is_some());
    }
    assert!(result.samples[1999].covariance[0][0] > result.samples[1000].covariance[0][0]);
    assert!(result.samples[2000].covariance[0][0] < result.samples[1999].covariance[0][0]);
    Ok(())
}

#[test]
fn fusion_beats_baselines_across_fixed_seed_suite() -> Result<(), Error> {
    let mut errors = [0.0; 3];
    for seed in 0..8 {
        let result = run(&Config {
            seed,
            ..Config::default()
        })?;
        let metrics = result.summary.outage.expect("configured outage");
        errors[0] += metrics.filter_rmse;
        errors[1] += metrics.hold_rmse;
        errors[2] += metrics.integration_rmse;
    }
    assert!(errors[0] < errors[1], "filter vs hold: {errors:?}");
    assert!(errors[0] < errors[2], "filter vs integration: {errors:?}");
    Ok(())
}

#[test]
fn invalid_configuration_is_rejected_before_a_run() {
    for config in [
        Config {
            duration_seconds: 0,
            ..Config::default()
        },
        Config {
            duration_seconds: 3601,
            ..Config::default()
        },
        Config {
            position_sigma: f64::NAN,
            ..Config::default()
        },
        Config {
            velocity_sigma: -0.1,
            ..Config::default()
        },
        Config {
            acceleration_density: f64::INFINITY,
            ..Config::default()
        },
        Config {
            outage: Some(Outage { start: 20, end: 10 }),
            ..Config::default()
        },
        Config {
            outage: Some(Outage { start: 10, end: 31 }),
            ..Config::default()
        },
    ] {
        assert!(matches!(run(&config), Err(Error::InvalidConfig(_))));
    }
}
