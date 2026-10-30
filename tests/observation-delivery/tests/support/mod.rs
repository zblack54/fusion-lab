#![allow(dead_code)]
use fusion_lab::*;
use std::num::NonZeroU32;

pub fn observation(
    sensor: Sensor,
    sequence: u32,
    sample: u32,
    arrival: u32,
    value: [f64; 2],
) -> Observation {
    Observation::new(sensor, sequence, sample, arrival, value).unwrap()
}
pub fn model() -> ReplayModel {
    ReplayModel::new(5.0, 2.0, 0.0).unwrap()
}
pub fn trace(observations: Vec<Observation>) -> Trace {
    Trace::new(1, 128, [None, None], observations).unwrap()
}
pub fn nz(value: u32) -> Option<NonZeroU32> {
    Some(NonZeroU32::new(value).unwrap())
}
pub fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual={actual:?}, expected={expected:?}, tolerance={tolerance:?}"
    );
}
pub fn ids(values: &[ObservationId]) -> Vec<(Sensor, u32)> {
    values
        .iter()
        .map(|id| (id.sensor(), id.sequence()))
        .collect()
}
pub fn ledger(result: &ReplayResult) -> Vec<(Sensor, u32, DeliveryOutcome, u32, Option<u16>)> {
    result
        .ledger()
        .iter()
        .map(|e| {
            (
                e.id().sensor(),
                e.id().sequence(),
                e.outcome(),
                e.decision_tick(),
                e.queue_occupancy(),
            )
        })
        .collect()
}
pub fn assert_bitwise(left: &ReplayResult, right: &ReplayResult) {
    assert_eq!(left.snapshots().len(), right.snapshots().len());
    for (a, b) in left.snapshots().iter().zip(right.snapshots()) {
        assert_eq!(a.tick(), b.tick());
        let values = |s: &ReplaySnapshot| {
            s.estimate()
                .position
                .into_iter()
                .chain(s.estimate().velocity)
                .chain(s.covariance().iter().flatten().copied())
                .chain(s.held_position())
                .chain(s.integrated_position())
                .map(f64::to_bits)
                .collect::<Vec<_>>()
        };
        assert_eq!(values(a), values(b));
    }
    assert_eq!(ledger(left), ledger(right));
    assert_eq!(ids(left.arrival_order()), ids(right.arrival_order()));
    assert_eq!(
        left.encode_ledger_tsv().unwrap(),
        right.encode_ledger_tsv().unwrap()
    );
}
pub fn assert_source_equal(result: &ReplayResult, source: &Experiment) {
    assert_eq!(result.snapshots().len(), source.samples.len());
    for (actual, expected) in result.snapshots().iter().zip(&source.samples) {
        assert_eq!(actual.tick(), expected.tick);
        let a = actual
            .estimate()
            .position
            .into_iter()
            .chain(actual.estimate().velocity)
            .chain(actual.covariance().iter().flatten().copied())
            .chain(actual.held_position())
            .chain(actual.integrated_position())
            .map(f64::to_bits)
            .collect::<Vec<_>>();
        let b = expected
            .estimate
            .position
            .into_iter()
            .chain(expected.estimate.velocity)
            .chain(expected.covariance.iter().flatten().copied())
            .chain(expected.held_position)
            .chain(expected.integrated_position)
            .map(f64::to_bits)
            .collect::<Vec<_>>();
        assert_eq!(a, b, "tick{}", actual.tick());
    }
}
pub fn canonical_header(count: usize) -> String {
    format!(
        "fusion-lab-delivery-trace\t1\nclock_ms\t10\nclock_kind\tvirtual_planned\npolicy\tOnTimeOnly\nduration_seconds\t1\nposition_assumed_sigma\t5\nvelocity_assumed_sigma\t2\nacceleration_density\t0\nbuffer\tcapacity\t128\tposition_drop_every\t0\tvelocity_drop_every\t0\nobservations\t{count}\nplanned_arrival_tick\tsample_tick\tsensor\tsequence\tx\ty\n"
    )
}
pub fn golden_trace() -> Vec<u8> {
    (canonical_header(3)
        + "0\t0\tposition\t0\t10\t20\n1\t0\tvelocity\t0\t1\t0.5\n5\t5\tvelocity\t1\t6\t3\n")
        .into_bytes()
}
pub fn golden_ledger() -> Vec<u8> {
    b"fusion-lab-delivery-ledger\t1\nclock_ms\t10\npolicy\tOnTimeOnly\nduration_seconds\t1\noffered\t3\ncounts\taccepted\t2\tstale_drop\t1\tscripted_loss\t0\tqueue_full_drop\t0\tundelivered_at_horizon\t0\nsource_rows\t3\nsensor\tsequence\tsample_tick\tplanned_arrival_tick\tdecision_tick\toutcome\tqueue_occupancy\nposition\t0\t0\t0\t0\taccepted\t0\nvelocity\t0\t0\t1\t1\tstale_drop\t0\nvelocity\t1\t5\t5\t5\taccepted\t0\narrivals\t3\nsensor\tsequence\nposition\t0\nvelocity\t0\nvelocity\t1\n".to_vec()
}
pub fn golden_input() -> Trace {
    trace(vec![
        observation(Sensor::Position, 0, 0, 0, [10.0, 20.0]),
        observation(Sensor::Velocity, 0, 0, 1, [1.0, 0.5]),
        observation(Sensor::Velocity, 1, 5, 5, [6.0, 3.0]),
    ])
}
