//! Frozen issue #8 delivery fault experiment.
use fusion_lab::{
    Config, HistoryOutcome, HistoryPlan, HistorySnapshot, Observation, ReplayModel, Sample,
    Scenario, Sensor, Trace, replay_history, run,
};
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    num::NonZeroU32,
};

fn normalized(outcome: HistoryOutcome) -> usize {
    match outcome {
        HistoryOutcome::AcceptedOnTime => 0,
        HistoryOutcome::CorrectedLate | HistoryOutcome::TooOldDrop => 1,
        HistoryOutcome::ScriptedLoss => 2,
        HistoryOutcome::QueueFullDrop | HistoryOutcome::UndeliveredAtHorizon => 3,
    }
}

fn snapshot_bits(s: &HistorySnapshot) -> impl Iterator<Item = u64> {
    let state = s.estimate();
    state
        .position
        .into_iter()
        .chain(state.velocity)
        .chain(s.covariance().into_iter().flatten())
        .chain(s.held_position())
        .chain(s.integrated_position())
        .map(f64::to_bits)
}

fn error_and_coverage(s: &HistorySnapshot, sample: &Sample) -> (f64, bool) {
    assert_eq!(s.tick(), sample.tick);
    let [x, y] = s.estimate().position;
    let [tx, ty] = sample.truth.position;
    let (dx, dy) = (x - tx, y - ty);
    let covariance = s.covariance();
    let (a, b, c, d) = (
        covariance[0][0],
        covariance[0][1],
        covariance[1][0],
        covariance[1][1],
    );
    assert!(a.is_finite() && b.is_finite() && c.is_finite() && d.is_finite());
    assert_eq!(b, c, "position block must be symmetric");
    assert!(a > 0.0);
    let lower = b / a.sqrt();
    let remainder = d - lower * lower;
    assert!(remainder > 0.0 && remainder.is_finite());
    let u = dx / a.sqrt();
    let v = (dy - lower * u) / remainder.sqrt();
    let squared = dx * dx + dy * dy;
    let distance = u * u + v * v;
    assert!(squared.is_finite() && distance.is_finite());
    (squared, distance <= 5.991464547107979)
}

fn summary(records: &[(f64, bool)]) -> (f64, f64) {
    let n = records.len() as f64;
    let rmse = (records.iter().map(|r| r.0).sum::<f64>() / n).sqrt();
    let coverage = records.iter().filter(|r| r.1).count() as f64 / n;
    (rmse, coverage)
}

fn recovery(delayed: &[(f64, bool)], healthy: &[(f64, bool)]) -> (bool, usize) {
    let mut streak = 0;
    for tick in 2120..=3000 {
        if delayed[tick].0.sqrt() <= healthy[tick].0.sqrt() + 0.5 {
            streak += 1;
            if streak == 100 {
                return (true, tick - 2120);
            }
        } else {
            streak = 0;
        }
    }
    (false, 880)
}

fn study_seed(
    out: &mut impl Write,
    family: &str,
    scenario: Scenario,
    seed: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = run(&Config {
        duration_seconds: 30,
        seed,
        scenario,
        outage: None,
        position_sigma: 1.0,
        velocity_sigma: 0.15,
        assumed_position_sigma: Some(1.0),
        assumed_velocity_sigma: Some(0.15),
        acceleration_density: 0.1,
        velocity_bias: [0.0; 2],
        velocity_drift_rate: [0.0; 2],
    })?;
    let model = ReplayModel::new(1.0, 0.15, 0.1)?;
    let mut healthy = [Vec::new(), Vec::new()];
    for (arm, delay, loss, expected) in [
        ("H", false, false, [752, 0, 0, 0]),
        ("D", true, false, [502, 250, 0, 0]),
        ("L", false, true, [637, 0, 115, 0]),
        ("DL", true, true, [426, 211, 115, 0]),
    ] {
        let mut observations = Vec::with_capacity(752);
        for s in &source.samples {
            for (sensor, period, lag, value) in [
                (Sensor::Position, 20, 120, s.position_measurement),
                (Sensor::Velocity, 5, 40, s.velocity_measurement),
            ] {
                if let Some(value) = value {
                    let extra = if delay && (1000..2000).contains(&s.tick) {
                        lag
                    } else {
                        0
                    };
                    observations.push(Observation::new(
                        sensor,
                        s.tick / period,
                        s.tick,
                        s.tick + extra,
                        value,
                    )?);
                }
            }
        }
        assert_eq!(observations.len(), 752);
        let drops = if loss {
            [NonZeroU32::new(5), NonZeroU32::new(7)]
        } else {
            [None; 2]
        };
        let trace = Trace::new(30, 128, drops, observations)?;
        let zero = replay_history(&model, &trace, &HistoryPlan::new(0)?)?;
        let full = replay_history(&model, &trace, &HistoryPlan::new(500)?)?;
        assert_eq!(zero.arrival_order(), full.arrival_order());
        assert_eq!(zero.ledger().len(), 752);
        assert_eq!(full.ledger().len(), 752);
        let mut counts = [0; 4];
        for (a, b) in zero.ledger().iter().zip(full.ledger()) {
            assert_eq!(
                (
                    a.id(),
                    a.sample_tick(),
                    a.planned_arrival_tick(),
                    a.decision_tick(),
                    a.queue_occupancy(),
                    normalized(a.outcome())
                ),
                (
                    b.id(),
                    b.sample_tick(),
                    b.planned_arrival_tick(),
                    b.decision_tick(),
                    b.queue_occupancy(),
                    normalized(b.outcome())
                ),
            );
            counts[normalized(a.outcome())] += 1;
        }
        assert_eq!(counts, expected, "{family}/{seed}/{arm}");
        let last_late = full
            .ledger()
            .iter()
            .filter(|r| r.outcome() == HistoryOutcome::CorrectedLate)
            .map(|r| r.decision_tick())
            .max()
            .unwrap_or(0);
        assert_eq!(
            last_late,
            match arm {
                "D" => 2100,
                "DL" => 2080,
                _ => 0,
            }
        );
        assert_eq!(zero.snapshots().len(), 3001);
        assert_eq!(full.snapshots().len(), 3001);
        if !delay {
            assert!(
                zero.snapshots()
                    .iter()
                    .zip(full.snapshots())
                    .all(|(a, b)| a.tick() == b.tick() && snapshot_bits(a).eq(snapshot_bits(b)))
            );
        }
        for (index, (window, result)) in [(0, &zero), (500, &full)].into_iter().enumerate() {
            let metrics: Vec<_> = result
                .snapshots()
                .iter()
                .zip(&source.samples)
                .map(|(snapshot, sample)| error_and_coverage(snapshot, sample))
                .collect();
            let (rmse, coverage) = summary(&metrics[1000..=2119]);
            let (post_rmse, post_coverage) = summary(&metrics[2120..=3000]);
            let (event, elapsed) = if arm == "D" {
                let (event, elapsed) = recovery(&metrics, &healthy[index]);
                (usize::from(event).to_string(), elapsed.to_string())
            } else {
                (String::new(), String::new())
            };
            writeln!(
                out,
                "{family}\t{seed}\t{arm}\t{window}\t{rmse}\t{post_rmse}\t{coverage}\t{post_coverage}\t{event}\t{elapsed}\t{}\t{}\t{}\t{last_late}",
                counts[0], counts[1], counts[2]
            )?;
            if arm == "H" {
                healthy[index] = metrics;
            }
        }
    }
    out.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let (straight, maneuver, count) = match args.get(1).map(String::as_str) {
        Some("fixture") => (30000, 40000, 4),
        Some("heldout") => (31000, 41000, 50),
        _ => return Err("usage: delivery_fault_study fixture|heldout OUTPUT".into()),
    };
    if args.len() != 3 {
        return Err("expected exactly one output path".into());
    }
    let mut out = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[2])?,
    );
    writeln!(
        out,
        "family\tseed\tarm\twindow\trmse\tpost_rmse\tcoverage\tpost_coverage\tevent\telapsed\tontime\tlate\tlost\tlast_late"
    )?;
    for (family, scenario, start) in [
        ("straight", Scenario::Straight, straight),
        ("maneuver", Scenario::Maneuver, maneuver),
    ] {
        for seed in start..start + count {
            study_seed(&mut out, family, scenario, seed)?;
        }
    }
    Ok(())
}
