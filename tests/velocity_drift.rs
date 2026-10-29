use fusion_lab::{Config, Error, Scenario, run};

#[test]
fn drift_observations_filter_and_integration_match_independent_rationals() -> Result<(), Error> {
    let result = run(&Config {
        duration_seconds: 1,
        scenario: Scenario::Straight,
        position_sigma: 0.0,
        velocity_sigma: 0.0,
        assumed_position_sigma: Some(3.0),
        assumed_velocity_sigma: Some(2.0),
        velocity_bias: [0.2, -0.1],
        velocity_drift_rate: [0.4, -0.2],
        acceleration_density: 0.0,
        outage: None,
        ..Config::default()
    })?;
    let first = &result.samples[0];
    assert_eq!(first.truth.velocity, [1.0, 0.5]);
    assert_eq!(first.velocity_measurement, Some([1.2, 0.4]));
    for (actual, expected) in first.estimate.velocity.into_iter().zip([0.6, 0.2]) {
        assert!((actual - expected).abs() < 1e-12);
    }
    let fifth = &result.samples[5];
    for (actual, expected) in fifth
        .velocity_measurement
        .unwrap()
        .into_iter()
        .zip([1.22, 0.39])
    {
        assert!((actual - expected).abs() < 1e-12);
    }
    for (actual, expected) in fifth
        .estimate
        .velocity
        .into_iter()
        .zip([121.0 / 150.0, 79.0 / 300.0])
    {
        assert!((actual - expected).abs() < 1e-12);
    }
    for (actual, expected) in fifth
        .estimate
        .position
        .into_iter()
        .zip([121.0 / 3000.0, 79.0 / 6000.0])
    {
        assert!((actual - expected).abs() < 1e-12);
    }
    assert!((fifth.covariance[2][2] - 4.0 / 3.0).abs() < 1e-12);
    assert!((fifth.covariance[3][3] - 4.0 / 3.0).abs() < 1e-12);
    for (tick, expected) in [
        (1, [0.012, 0.004]),
        (5, [0.06, 0.02]),
        (6, [0.0722, 0.0239]),
    ] {
        for (actual, expected) in result.samples[tick]
            .integrated_position
            .into_iter()
            .zip(expected)
        {
            assert!((actual - expected).abs() < 1e-12);
        }
    }
    for (actual, expected) in result.samples[20]
        .integrated_position
        .into_iter()
        .zip([0.2, 0.1])
    {
        assert!((actual - expected).abs() < 1e-12);
    }
    for (actual, expected) in result.samples[100]
        .velocity_measurement
        .unwrap()
        .into_iter()
        .zip([1.6, 0.2])
    {
        assert!((actual - expected).abs() < 1e-12);
    }
    Ok(())
}

#[test]
fn drift_preserves_truth_position_schedule_covariance_and_noise_streams() -> Result<(), Error> {
    let config = Config::default();
    let ordinary = run(&config)?;
    let drifting = run(&Config {
        velocity_drift_rate: [0.01, -0.005],
        ..config
    })?;
    assert_eq!(ordinary.samples.len(), 3001);
    assert_eq!(ordinary.samples.len(), drifting.samples.len());
    for (a, b) in ordinary.samples.iter().zip(&drifting.samples) {
        assert_eq!(a.tick, b.tick);
        assert_eq!(a.truth, b.truth);
        assert_eq!(a.position_measurement, b.position_measurement);
        assert_eq!(a.position_outage, b.position_outage);
        assert_eq!(a.held_position, b.held_position);
        assert_eq!(a.covariance, b.covariance);
        assert_eq!(
            a.velocity_measurement.is_some(),
            b.velocity_measurement.is_some()
        );
        if let (Some(x), Some(y)) = (a.velocity_measurement, b.velocity_measurement) {
            for (axis, rate) in [0.01, -0.005].into_iter().enumerate() {
                let expected = rate * f64::from(a.tick) / 100.0;
                let tolerance = 1e-12 + 1e-12 * x[axis].abs().max(y[axis].abs());
                assert!((y[axis] - x[axis] - expected).abs() <= tolerance);
            }
        }
    }
    assert_ne!(
        ordinary.samples[1001].integrated_position,
        drifting.samples[1001].integrated_position
    );
    assert_eq!(drifting, run(&drifting.config)?);
    let altered_assumptions = run(&Config {
        assumed_position_sigma: Some(3.0),
        assumed_velocity_sigma: Some(0.6),
        ..drifting.config.clone()
    })?;
    assert_eq!(altered_assumptions.samples.len(), drifting.samples.len());
    for (a, b) in drifting.samples.iter().zip(&altered_assumptions.samples) {
        assert_eq!(a.position_measurement, b.position_measurement);
        assert_eq!(a.velocity_measurement, b.velocity_measurement);
        assert_eq!(a.held_position, b.held_position);
        assert_eq!(a.integrated_position, b.integrated_position);
    }
    Ok(())
}

#[test]
fn signed_zero_rate_axes_and_zero_crossing_preserve_constant_path() -> Result<(), Error> {
    let config = Config {
        duration_seconds: 1,
        outage: None,
        velocity_bias: [0.5, -0.25],
        ..Config::default()
    };
    let constant = run(&config)?;
    for rates in [[0.0, 0.0], [-0.0, 0.0], [0.0, -0.0], [-0.0, -0.0]] {
        assert_eq!(
            constant.samples,
            run(&Config {
                velocity_drift_rate: rates,
                ..config.clone()
            })?
            .samples
        );
    }
    for rates in [[-0.0, 0.02], [0.02, -0.0]] {
        let result = run(&Config {
            velocity_drift_rate: rates,
            ..config.clone()
        })?;
        let unchanged_axis = if rates[0] == 0.0 { 0 } else { 1 };
        for (a, b) in constant.samples.iter().zip(&result.samples) {
            if let (Some(x), Some(y)) = (a.velocity_measurement, b.velocity_measurement) {
                assert_eq!(x[unchanged_axis].to_bits(), y[unchanged_axis].to_bits());
            }
        }
    }
    let crossing = run(&Config {
        duration_seconds: 1,
        scenario: Scenario::Straight,
        position_sigma: 0.0,
        velocity_sigma: 0.0,
        velocity_bias: [-0.25, 0.125],
        velocity_drift_rate: [0.5, -0.25],
        outage: None,
        ..Config::default()
    })?;
    assert_eq!(crossing.samples[50].velocity_measurement, Some([1.0, 0.5]));
    Ok(())
}

#[test]
fn rate_and_whole_horizon_validation_are_finite_and_inclusive() -> Result<(), Error> {
    for axis in 0..2 {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.01, 1.01] {
            let mut config = Config {
                duration_seconds: 1,
                outage: None,
                ..Config::default()
            };
            config.velocity_drift_rate[axis] = value;
            assert_eq!(
                run(&config),
                Err(Error::InvalidConfig("velocity_drift_rate"))
            );
        }
        for value in [-1.0, 1.0] {
            let mut config = Config {
                duration_seconds: 1,
                outage: None,
                ..Config::default()
            };
            config.velocity_drift_rate[axis] = value;
            assert_eq!(run(&config)?.samples.len(), 101);
            config.velocity_bias[axis] = 99.0 * value;
            let final_observation = run(&config)?.samples[100].velocity_measurement.unwrap();
            assert!(final_observation[axis].is_finite());
            config.velocity_bias[axis] = 100.0 * value;
            assert_eq!(
                run(&config),
                Err(Error::InvalidConfig("velocity_bias_horizon"))
            );
        }
    }
    for config in [
        Config {
            duration_seconds: 3600,
            velocity_drift_rate: [1.0, 0.0],
            ..Config::default()
        },
        Config {
            duration_seconds: 3201,
            velocity_drift_rate: [0.03125, 0.0],
            ..Config::default()
        },
    ] {
        assert_eq!(
            run(&config),
            Err(Error::InvalidConfig("velocity_bias_horizon"))
        );
    }
    Ok(())
}
