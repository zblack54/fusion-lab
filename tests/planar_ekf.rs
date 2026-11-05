use fusion_lab::{
    planar_ekf::{EkfError, PlanarEkf, motion},
    wheel_motion::{Pose, WheelTravel},
};
use std::f64::consts::{PI, TAU};

const ORIGIN: Pose = Pose {
    x: 0.0,
    y: 0.0,
    heading: 0.0,
};
const PRIOR: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]];
const FIX: [[f64; 2]; 2] = [[1.0, 0.0], [0.0, 1.0]];

fn travel(left: f64, right: f64) -> WheelTravel {
    WheelTravel { left, right }
}

fn close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} vs {b}, tolerance {tol}");
}

fn angle_delta(a: f64, b: f64) -> f64 {
    (a - b + PI).rem_euclid(TAU) - PI
}

// Integrate the instantaneous world direction with Simpson quadrature.
// This oracle neither uses sinc nor calls the production wheel model.
fn reference(p: [f64; 3], w: [f64; 2], width: f64) -> [f64; 3] {
    let distance = (w[0] + w[1]) * 0.5;
    let turn = (w[1] - w[0]) / width;
    let mut total = [0.0; 2];
    for i in 0..=128 {
        let a = p[2] + turn * f64::from(i) / 128.0;
        let weight = if i == 0 || i == 128 {
            1.0
        } else if i % 2 == 0 {
            2.0
        } else {
            4.0
        };
        total[0] += weight * a.cos();
        total[1] += weight * a.sin();
    }
    [
        p[0] + distance * total[0] / 384.0,
        p[1] + distance * total[1] / 384.0,
        p[2] + turn,
    ]
}

#[test]
fn unequal_wheels_propagate_hand_computed_covariance() {
    let mut filter = PlanarEkf::new(ORIGIN, PRIOR).unwrap();
    let m = motion(ORIGIN, 2.0, travel(2.0, 2.0)).unwrap();
    assert_eq!(
        m.state_jacobian,
        [[1.0, 0.0, 0.0], [0.0, 1.0, 2.0], [0.0, 0.0, 1.0]]
    );
    assert_eq!(m.wheel_jacobian, [[0.5, 0.5], [-0.5, 0.5], [-0.5, 0.5]]);
    filter.predict(2.0, travel(2.0, 2.0), [1.0, 4.0]).unwrap();
    assert_eq!(filter.pose(), Pose { x: 2.0, ..ORIGIN });
    assert_eq!(
        filter.covariance(),
        [[2.25, 0.75, 0.75], [0.75, 15.25, 7.25], [0.75, 7.25, 4.25]]
    );
}

#[test]
fn motion_jacobians_match_independent_quadrature_near_straight_and_wrap() {
    let eps = 1e-5;
    for (p, w) in [
        ([2.0, -1.0, 0.3], [0.5, 1.9]),
        ([0.0, 0.0, -0.4], [-2.0, -2.0 + 1e-7]),
        ([1.0, 2.0, 3.0 * PI / 4.0], [0.0, PI / 2.0]),
    ] {
        let m = motion(
            Pose {
                x: p[0],
                y: p[1],
                heading: p[2],
            },
            2.0,
            travel(w[0], w[1]),
        )
        .unwrap();
        assert!((-PI..PI).contains(&m.pose.heading));
        let expected = reference(p, w, 2.0);
        close(m.pose.x, expected[0], 2e-10);
        close(m.pose.y, expected[1], 2e-10);
        close(angle_delta(m.pose.heading, expected[2]), 0.0, 1e-12);
        for j in 0..3 {
            let mut low = p;
            let mut high = p;
            low[j] -= eps;
            high[j] += eps;
            let a = reference(high, w, 2.0);
            let b = reference(low, w, 2.0);
            for i in 0..3 {
                close(
                    m.state_jacobian[i][j],
                    (if i == 2 {
                        angle_delta(a[i], b[i])
                    } else {
                        a[i] - b[i]
                    }) / (2.0 * eps),
                    2e-7,
                );
            }
        }
        for j in 0..2 {
            let mut low = w;
            let mut high = w;
            low[j] -= eps;
            high[j] += eps;
            let a = reference(p, high, 2.0);
            let b = reference(p, low, 2.0);
            for i in 0..3 {
                close(
                    m.wheel_jacobian[i][j],
                    (if i == 2 {
                        angle_delta(a[i], b[i])
                    } else {
                        a[i] - b[i]
                    }) / (2.0 * eps),
                    2e-7,
                );
            }
        }
    }
}

#[test]
fn position_fix_updates_heading_only_through_cross_covariance() {
    let mut stationary = PlanarEkf::new(ORIGIN, PRIOR).unwrap();
    stationary
        .update_position([2.0, 4.0], [[1.0, 0.0], [0.0, 2.0]])
        .unwrap();
    close(stationary.pose().x, 1.0, 1e-12);
    close(stationary.pose().y, 2.0, 1e-12);
    assert_eq!(stationary.pose().heading.to_bits(), 0.0_f64.to_bits());
    close(stationary.covariance()[0][0], 0.5, 1e-12);
    close(stationary.covariance()[1][1], 1.0, 1e-12);
    close(stationary.covariance()[2][2], 3.0, 1e-12);
    let mut coupled =
        PlanarEkf::new(ORIGIN, [[2.0, 0.0, 1.0], [0.0, 2.0, 0.0], [1.0, 0.0, 1.0]]).unwrap();
    coupled.update_position([3.0, 0.0], FIX).unwrap();
    close(coupled.pose().x, 2.0, 1e-12);
    close(coupled.pose().heading, 1.0, 1e-12);
    close(coupled.covariance()[0][2], 1.0 / 3.0, 1e-12);
    close(coupled.covariance()[2][2], 2.0 / 3.0, 1e-12);
}

fn bits(filter: &PlanarEkf) -> Vec<u64> {
    let p = filter.pose();
    [p.x, p.y, p.heading]
        .into_iter()
        .chain(filter.covariance().into_iter().flatten())
        .map(f64::to_bits)
        .collect()
}

#[test]
fn validation_and_failed_operations_preserve_bits() {
    assert!(matches!(
        PlanarEkf::new(ORIGIN, [[0.0; 3]; 3]),
        Err(EkfError::StateCovariance)
    ));
    let mut filter = PlanarEkf::new(
        Pose {
            x: 1e9,
            heading: -0.0,
            ..ORIGIN
        },
        PRIOR,
    )
    .unwrap();
    let before = bits(&filter);
    assert_eq!(
        filter.predict(2.0, travel(1.0, 1.0), [0.0; 2]),
        Err(EkfError::NumericalFailure)
    );
    assert_eq!(
        filter.predict(2.0, travel(0.0, 0.0), [-1.0, 0.0]),
        Err(EkfError::WheelVariance)
    );
    assert_eq!(
        filter.update_position([f64::NAN, 0.0], FIX),
        Err(EkfError::Measurement)
    );
    assert_eq!(
        filter.update_position([0.0, 0.0], [[1.0, 1.0], [0.0, 1.0]]),
        Err(EkfError::MeasurementCovariance)
    );
    assert_eq!(bits(&filter), before);
    assert!(motion(ORIGIN, 2.0, travel(128.0, -128.0)).is_ok());
    assert!(matches!(
        motion(ORIGIN, 2.0, travel(128.1, 0.0)),
        Err(EkfError::Travel)
    ));
    assert!(matches!(
        motion(ORIGIN, 0.0, travel(0.0, 0.0)),
        Err(EkfError::TrackWidth)
    ));
}

#[test]
fn repeated_curved_updates_keep_covariance_symmetric_positive() {
    let mut filter = PlanarEkf::new(ORIGIN, PRIOR).unwrap();
    for _ in 0..128 {
        filter
            .predict(0.8, travel(0.08, 0.1), [0.001, 0.002])
            .unwrap();
        let p = filter.pose();
        filter
            .update_position([p.x + 0.001, p.y - 0.001], FIX)
            .unwrap();
        let c = filter.covariance();
        for (i, row) in c.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                assert_eq!(value.to_bits(), c[j][i].to_bits());
            }
        }
        let [a, b, d] = c[0];
        let e = c[1][1];
        let f = c[1][2];
        let g = c[2][2];
        assert!(a > 0.0 && a * e - b * b > 0.0);
        assert!(a * (e * g - f * f) - b * (b * g - f * d) + d * (b * f - e * d) > 0.0);
        assert!((-PI..PI).contains(&filter.pose().heading));
    }
}

#[test]
fn correlated_fix_and_postsolve_failure_are_observable() {
    let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let mut filter = PlanarEkf::new(ORIGIN, identity).unwrap();
    filter
        .update_position([3.0, 0.0], [[2.0, 1.0], [1.0, 2.0]])
        .unwrap();
    close(filter.pose().x, 9.0 / 8.0, 1e-12);
    close(filter.pose().y, -3.0 / 8.0, 1e-12);
    close(filter.covariance()[0][0], 5.0 / 8.0, 1e-12);
    close(filter.covariance()[0][1], 1.0 / 8.0, 1e-12);
    close(filter.covariance()[1][1], 5.0 / 8.0, 1e-12);
    let mut edge = PlanarEkf::new(
        Pose { y: 1e9, ..ORIGIN },
        [[2.0, 1.0, 0.0], [1.0, 2.0, 0.0], [0.0, 0.0, 1.0]],
    )
    .unwrap();
    let before = bits(&edge);
    assert_eq!(
        edge.update_position([1.0, 1e9], FIX),
        Err(EkfError::NumericalFailure)
    );
    assert_eq!(bits(&edge), before);
}

#[test]
fn exact_heading_boundary_is_canonical_without_rounding_interior() {
    let filter = PlanarEkf::new(
        Pose {
            heading: PI,
            ..ORIGIN
        },
        PRIOR,
    )
    .unwrap();
    assert_eq!(filter.pose().heading.to_bits(), (-PI).to_bits());
    let h = PI.next_down();
    let m = motion(
        Pose {
            heading: h,
            ..ORIGIN
        },
        2.0,
        travel(0.0, 0.0),
    )
    .unwrap();
    assert_eq!(m.pose.heading.to_bits(), h.to_bits());
}
