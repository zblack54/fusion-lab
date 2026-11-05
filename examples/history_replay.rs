//! One-second observational comparison: online snapshots stay immutable while
//! an eligible late velocity corrects the current estimate and baselines.
use fusion_lab::{
    HistoryOutcome, HistoryPlan, Observation, ReplayModel, Sensor, Trace, replay_delivery,
    replay_history,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = ReplayModel::new(5.0, 2.0, 0.0)?;
    let trace = Trace::new(
        1,
        128,
        [None, None],
        vec![
            Observation::new(Sensor::Position, 0, 0, 0, [10.0, -5.0])?,
            Observation::new(Sensor::Velocity, 0, 0, 1, [2.0, -2.0])?,
            Observation::new(Sensor::Velocity, 1, 5, 5, [4.0, -4.0])?,
        ],
    )?;
    let corrected = replay_history(&model, &trace, &HistoryPlan::new(1)?)?;
    let stale = replay_history(&model, &trace, &HistoryPlan::new(0)?)?;
    let legacy = replay_delivery(&model, &trace)?;
    let late = corrected
        .ledger()
        .iter()
        .filter(|entry| entry.outcome() == HistoryOutcome::CorrectedLate)
        .count();
    let on_time = corrected
        .ledger()
        .iter()
        .filter(|entry| entry.outcome() == HistoryOutcome::AcceptedOnTime)
        .count();
    assert_eq!((corrected.snapshots().len(), on_time, late), (101, 2, 1));
    assert_eq!(corrected.snapshots()[0].estimate().velocity, [0.0; 2]);
    assert_eq!(corrected.snapshots()[0].integrated_position(), [10.0, -5.0]);
    assert_eq!(stale.snapshots()[1].estimate().velocity, [0.0; 2]);
    assert_eq!(
        stale.snapshots()[1].estimate().velocity,
        legacy.snapshots()[1].estimate().velocity
    );
    println!("one second: 101 snapshots, 3 offered, {on_time} on time, {late} corrected late");
    println!(
        "immutable tick 0: velocity={:?}, integration={:?}",
        corrected.snapshots()[0].estimate().velocity,
        corrected.snapshots()[0].integrated_position()
    );
    println!(
        "tick 1, W1 correction: velocity={:?}, integration={:?}",
        corrected.snapshots()[1].estimate().velocity,
        corrected.snapshots()[1].integrated_position()
    );
    println!(
        "tick 1, W0 stale drop: velocity={:?}, integration={:?}",
        stale.snapshots()[1].estimate().velocity,
        stale.snapshots()[1].integrated_position()
    );
    println!("policy demonstration only; no accuracy or calibration claim");
    Ok(())
}
