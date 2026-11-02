use fusion_lab::{Config, DeliveryPlan, ReplayModel, ReplayResult, Trace, replay_delivery, run};
use std::{
    error::Error,
    hint::black_box,
    io::{self, Write},
};
fn equal(a: &ReplayResult, b: &ReplayResult) {
    assert_eq!(a.snapshots().len(), b.snapshots().len());
    for (a, b) in a.snapshots().iter().zip(b.snapshots()) {
        assert_eq!(a.tick(), b.tick());
        let values = |s: &fusion_lab::ReplaySnapshot| {
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
    assert_eq!(a.ledger().len(), b.ledger().len());
    for (a, b) in a.ledger().iter().zip(b.ledger()) {
        assert_eq!(
            (
                a.id(),
                a.sample_tick(),
                a.planned_arrival_tick(),
                a.decision_tick(),
                a.outcome(),
                a.queue_occupancy()
            ),
            (
                b.id(),
                b.sample_tick(),
                b.planned_arrival_tick(),
                b.decision_tick(),
                b.outcome(),
                b.queue_occupancy()
            )
        );
    }
    assert_eq!(a.arrival_order(), b.arrival_order());
}
fn main() -> Result<(), Box<dyn Error>> {
    let case = std::env::args()
        .nth(1)
        .ok_or("expected one approved profile")?;
    let (duration, delay) = match case.as_str() {
        "warmup-delayed" => (1, 500),
        "warmup-accepted" => (1, 0),
        "delayed" => (3600, 500),
        "accepted" => (3600, 0),
        _ => return Err("unknown approved profile".into()),
    };
    if std::env::args().len() != 2 {
        return Err("expected exactly one approved profile".into());
    }
    let config = Config {
        duration_seconds: duration,
        outage: None,
        ..Config::default()
    };
    let source = run(&config)?;
    let model = ReplayModel::new(1.0, 0.15, 0.1)?;
    let trace = Trace::from_samples(
        duration,
        &source.samples,
        DeliveryPlan::new(delay, delay, 128, None, None)?,
    )?;
    let result = replay_delivery(&model, &trace)?;
    let encoded = trace.encode_tsv(&model)?;
    let ledger = result.encode_ledger_tsv()?;
    let (decoded_model, decoded) = Trace::decode_tsv(&encoded)?;
    assert_eq!(trace.duration_seconds(), decoded.duration_seconds());
    assert_eq!(trace.pending_capacity(), decoded.pending_capacity());
    for sensor in [fusion_lab::Sensor::Position, fusion_lab::Sensor::Velocity] {
        assert_eq!(trace.drop_every(sensor), decoded.drop_every(sensor));
    }
    assert_eq!(trace.observations().len(), decoded.observations().len());
    for (a, b) in trace.observations().iter().zip(decoded.observations()) {
        assert_eq!(
            (a.id(), a.sample_tick(), a.planned_arrival_tick()),
            (b.id(), b.sample_tick(), b.planned_arrival_tick())
        );
        assert_eq!(a.value().map(f64::to_bits), b.value().map(f64::to_bits));
    }
    assert_eq!(
        [
            model.assumed_position_sigma(),
            model.assumed_velocity_sigma(),
            model.acceleration_density()
        ]
        .map(f64::to_bits),
        [
            decoded_model.assumed_position_sigma(),
            decoded_model.assumed_velocity_sigma(),
            decoded_model.acceleration_density()
        ]
        .map(f64::to_bits)
    );
    let regenerated = decoded.encode_tsv(&decoded_model)?;
    assert_eq!(encoded, regenerated);
    let replayed = replay_delivery(&decoded_model, &decoded)?;
    equal(&result, &replayed);
    let regenerated_ledger = replayed.encode_ledger_tsv()?;
    assert_eq!(ledger, regenerated_ledger);
    if duration == 3600 {
        assert_eq!(trace.observations().len(), 90002);
        assert_eq!(source.samples.len(), 360001);
    }
    let mut counts = [0usize; 5];
    for e in result.ledger() {
        let index = match e.outcome() {
            fusion_lab::DeliveryOutcome::Accepted => 0,
            fusion_lab::DeliveryOutcome::StaleDrop => 1,
            fusion_lab::DeliveryOutcome::UndeliveredAtHorizon => 2,
            fusion_lab::DeliveryOutcome::QueueFullDrop => 3,
            fusion_lab::DeliveryOutcome::ScriptedLoss => 4,
        };
        counts[index] += 1;
    }
    let expected = match case.as_str() {
        "warmup-delayed" => [0, 0, 27, 0, 0],
        "warmup-accepted" => [27, 0, 0, 0, 0],
        "delayed" => [0, 89877, 125, 0, 0],
        "accepted" => [90002, 0, 0, 0, 0],
        _ => return Err("unreachable profile".into()),
    };
    assert_eq!(counts, expected);
    assert_eq!(result.arrival_order().len(), counts[0] + counts[1]);
    println!(
        "COMPLETE\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        case,
        duration,
        trace.observations().len(),
        result.snapshots().len(),
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        counts[4],
        encoded.len(),
        ledger.len()
    );
    io::stdout().flush()?;
    // Every required allocation remains live across the owner's held-HWM read.
    black_box((
        &source,
        &trace,
        &result,
        &encoded,
        &ledger,
        &decoded,
        &replayed,
        &regenerated,
        &regenerated_ledger,
    ));
    let mut ack = String::new();
    io::stdin().read_line(&mut ack)?;
    if ack != "release\n" {
        return Err("missing owner release acknowledgement".into());
    }
    black_box((
        &source,
        &trace,
        &result,
        &encoded,
        &ledger,
        &decoded,
        &replayed,
        &regenerated,
        &regenerated_ledger,
    ));
    Ok(())
}
