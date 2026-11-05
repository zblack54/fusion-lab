use fusion_lab::wheel_motion::{Pose, WheelConfig, WheelError, WheelTravel, simulate, step};
use std::f64::consts::PI;

const ORIGIN: Pose = Pose {
    x: 0.0,
    y: 0.0,
    heading: 0.0,
};
const CONFIG: WheelConfig = WheelConfig {
    track_width: 2.0,
    start_tick: 5,
    interval_ticks: 3,
    seed: 42,
    sigma: [0.0, 0.0],
};
fn travel(left: f64, right: f64) -> WheelTravel {
    WheelTravel { left, right }
}
fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1e-12, "{actual} vs {expected}");
}

#[test]
fn analytic_arcs() {
    for (left, right, x, y, h) in [
        (3.0, 3.0, 3.0, 0.0, 0.0),
        (-3.0, -3.0, -3.0, 0.0, 0.0),
        (-PI / 2.0, PI / 2.0, 0.0, 0.0, PI / 2.0),
        (0.0, PI, 1.0, 1.0, PI / 2.0),
        (PI / 2.0, 3.0 * PI / 2.0, 2.0, 2.0, PI / 2.0),
    ] {
        let p = step(ORIGIN, 2.0, travel(left, right)).unwrap();
        near(p.x, x);
        near(p.y, y);
        near(p.heading, h);
    }
    let p = step(
        Pose {
            x: 10.0,
            y: -3.0,
            heading: PI / 2.0,
        },
        2.0,
        travel(PI / 2.0, 3.0 * PI / 2.0),
    )
    .unwrap();
    near(p.x, 8.0);
    near(p.y, -1.0);
    near(p.heading, -PI);
}

#[test]
fn tiny_turn_preserves_signed_lateral_motion() {
    let delta = 2.0_f64.powi(-27);
    for sign in [-1.0, 1.0] {
        let p = step(ORIGIN, 2.0, travel(1.0 - sign * delta, 1.0 + sign * delta)).unwrap();
        near(p.x, 1.0);
        near(p.heading, sign * delta);
        assert!((p.y / (sign * delta / 2.0) - 1.0).abs() < 1e-8);
    }
}

#[test]
fn heading_boundary_and_wrap() {
    let interior = PI.next_down();
    for (heading, expected) in [(PI, -PI), (-PI, -PI), (interior, interior)] {
        let p = step(Pose { heading, ..ORIGIN }, 2.0, travel(0.0, 0.0)).unwrap();
        assert_eq!(p.heading.to_bits(), expected.to_bits());
    }
    for sign in [-1.0, 1.0] {
        let p = step(
            Pose {
                heading: sign * 3.0 * PI / 4.0,
                ..ORIGIN
            },
            2.0,
            travel(-sign * PI / 2.0, sign * PI / 2.0),
        )
        .unwrap();
        near(p.heading, -sign * 3.0 * PI / 4.0);
    }
}

#[test]
fn invalid_inputs_and_cumulative_output_are_typed() {
    for w in [0.0, 0.009, 101.0, f64::INFINITY, f64::NAN] {
        assert_eq!(
            step(ORIGIN, w, travel(0.0, 0.0)),
            Err(WheelError::TrackWidth)
        );
    }
    assert_eq!(
        step(
            Pose {
                x: f64::NAN,
                ..ORIGIN
            },
            2.0,
            travel(0.0, 0.0)
        ),
        Err(WheelError::Pose)
    );
    assert_eq!(
        step(
            Pose {
                heading: PI.next_up(),
                ..ORIGIN
            },
            2.0,
            travel(0.0, 0.0)
        ),
        Err(WheelError::Pose)
    );
    assert_eq!(
        step(ORIGIN, 2.0, travel(101.0, 0.0)),
        Err(WheelError::Travel)
    );
    assert_eq!(
        step(Pose { x: 1e9, ..ORIGIN }, 2.0, travel(1.0, 1.0)),
        Err(WheelError::Pose)
    );
    for cfg in [
        WheelConfig {
            interval_ticks: 0,
            ..CONFIG
        },
        WheelConfig {
            interval_ticks: 10001,
            ..CONFIG
        },
        WheelConfig {
            start_tick: u32::MAX - 2,
            ..CONFIG
        },
    ] {
        assert_eq!(
            simulate(cfg, ORIGIN, &[travel(0.0, 0.0)]),
            Err(WheelError::Schedule)
        );
    }
    assert_eq!(
        simulate(CONFIG, ORIGIN, &vec![travel(0.0, 0.0); 10001]),
        Err(WheelError::Schedule)
    );
    assert_eq!(
        simulate(
            WheelConfig {
                sigma: [0.0, f64::NAN],
                ..CONFIG
            },
            ORIGIN,
            &[]
        ),
        Err(WheelError::Noise)
    );
    assert_eq!(
        simulate(
            WheelConfig {
                track_width: 0.0,
                ..CONFIG
            },
            ORIGIN,
            &[]
        ),
        Err(WheelError::TrackWidth)
    );
    assert!(
        simulate(CONFIG, ORIGIN, &[])
            .unwrap()
            .observations
            .is_empty()
    );
}

#[test]
fn observations_preserve_bits_and_cadence() {
    let inputs = [travel(-0.0, 1.0), travel(2.0, -3.0)];
    let result = simulate(CONFIG, ORIGIN, &inputs).unwrap();
    assert_eq!(result.poses.len(), 2);
    assert_eq!(result.observations.len(), 2);
    for (i, (obs, ideal)) in result.observations.iter().zip(inputs).enumerate() {
        let i = i as u32;
        assert_eq!(
            (obs.sequence, obs.start_tick, obs.end_tick),
            (i, 5 + 3 * i, 8 + 3 * i)
        );
        assert_eq!(obs.travel.left.to_bits(), ideal.left.to_bits());
        assert_eq!(obs.travel.right.to_bits(), ideal.right.to_bits());
    }
}

#[test]
fn seeded_noise_is_repeatable_and_separates_wheels_and_truth() {
    let inputs = [travel(1.0, 1.5); 8];
    let cfg = WheelConfig {
        sigma: [0.125, 0.25],
        ..CONFIG
    };
    let a = simulate(cfg, ORIGIN, &inputs).unwrap();
    let b = simulate(cfg, ORIGIN, &inputs).unwrap();
    let left = simulate(
        WheelConfig {
            sigma: [0.0, 0.25],
            ..cfg
        },
        ORIGIN,
        &inputs,
    )
    .unwrap();
    let ideal = simulate(CONFIG, ORIGIN, &inputs).unwrap();
    assert!(a.observations.iter().any(|o| o.travel.left != 1.0));
    for i in 0..inputs.len() {
        let bits = |p: Pose| [p.x, p.y, p.heading].map(f64::to_bits);
        assert_eq!(bits(a.poses[i]), bits(ideal.poses[i]));
        assert_eq!(bits(a.poses[i]), bits(left.poses[i]));
        assert_eq!(
            a.observations[i].travel.left.to_bits(),
            b.observations[i].travel.left.to_bits()
        );
        assert_eq!(
            a.observations[i].travel.right.to_bits(),
            b.observations[i].travel.right.to_bits()
        );
        assert_eq!(
            a.observations[i].travel.right.to_bits(),
            left.observations[i].travel.right.to_bits()
        );
    }
}

#[test]
fn cumulative_path_and_noise_draws_match_independent_oracles() {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use rand_distr::{Distribution, StandardNormal};
    let inputs = [travel(0.0, PI), travel(2.0, 2.0)];
    let cfg = WheelConfig {
        sigma: [0.125, 0.25],
        ..CONFIG
    };
    let run = simulate(cfg, ORIGIN, &inputs).unwrap();
    for (pose, y) in run.poses.iter().zip([1.0, 3.0]) {
        near(pose.x, 1.0);
        near(pose.y, y);
        near(pose.heading, PI / 2.0);
    }
    let other_seed = simulate(WheelConfig { seed: 43, ..cfg }, ORIGIN, &inputs).unwrap();
    let no_right = simulate(
        WheelConfig {
            sigma: [0.125, 0.0],
            ..cfg
        },
        ORIGIN,
        &inputs,
    )
    .unwrap();
    for axis in 0..2 {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        rng.set_stream(axis as u64);
        for (obs, ideal) in run.observations.iter().zip(inputs) {
            let draw: f64 = StandardNormal.sample(&mut rng);
            let expected = [ideal.left, ideal.right][axis] + cfg.sigma[axis] * draw;
            assert_eq!(
                [obs.travel.left, obs.travel.right][axis].to_bits(),
                expected.to_bits()
            );
        }
    }
    assert_ne!(run.observations, other_seed.observations);
    for (a, b) in run.observations.iter().zip(no_right.observations) {
        assert_eq!(a.travel.left.to_bits(), b.travel.left.to_bits());
    }
    assert_eq!(
        simulate(
            CONFIG,
            Pose {
                x: 1e9 - 1.0,
                ..ORIGIN
            },
            &[travel(1.0, 1.0); 2]
        ),
        Err(WheelError::Pose)
    );
}
