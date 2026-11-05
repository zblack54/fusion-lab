use fusion_lab::*;
fn model() -> ReplayModel {
    ReplayModel::new(5.0, 2.0, 0.0).unwrap()
}
fn obs(s: Sensor, n: u32, t: u32, a: u32, v: [f64; 2]) -> Observation {
    Observation::new(s, n, t, a, v).unwrap()
}
fn trace(d: u32, c: usize, v: Vec<Observation>) -> Trace {
    Trace::new(d, c, [None, None], v).unwrap()
}
fn history(t: &Trace, w: u32) -> HistoryResult {
    replay_history(&model(), t, &HistoryPlan::new(w).unwrap()).unwrap()
}
fn values(s: &HistorySnapshot) -> Vec<u64> {
    s.estimate()
        .position
        .into_iter()
        .chain(s.estimate().velocity)
        .chain(s.covariance().into_iter().flatten())
        .chain(s.held_position())
        .chain(s.integrated_position())
        .map(f64::to_bits)
        .collect()
}
fn legacy_values(s: &ReplaySnapshot) -> Vec<u64> {
    s.estimate()
        .position
        .into_iter()
        .chain(s.estimate().velocity)
        .chain(s.covariance().iter().flatten().copied())
        .chain(s.held_position())
        .chain(s.integrated_position())
        .map(f64::to_bits)
        .collect()
}
fn near(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 1e-12,
        "component {a} vs independent expected {b}"
    );
}
fn parity(t: &Trace, w: u32) {
    let h = history(t, w);
    let d = replay_delivery(&model(), t).unwrap();
    assert_eq!(h.arrival_order(), d.arrival_order());
    assert_eq!(h.ledger().len(), d.ledger().len());
    for (a, b) in h.ledger().iter().zip(d.ledger()) {
        assert_eq!(
            (
                a.id(),
                a.sample_tick(),
                a.planned_arrival_tick(),
                a.decision_tick(),
                a.queue_occupancy()
            ),
            (
                b.id(),
                b.sample_tick(),
                b.planned_arrival_tick(),
                b.decision_tick(),
                b.queue_occupancy()
            )
        );
        match b.outcome() {
            DeliveryOutcome::ScriptedLoss => assert_eq!(a.outcome(), HistoryOutcome::ScriptedLoss),
            DeliveryOutcome::QueueFullDrop => {
                assert_eq!(a.outcome(), HistoryOutcome::QueueFullDrop)
            }
            DeliveryOutcome::UndeliveredAtHorizon => {
                assert_eq!(a.outcome(), HistoryOutcome::UndeliveredAtHorizon)
            }
            DeliveryOutcome::Accepted => assert_eq!(a.outcome(), HistoryOutcome::AcceptedOnTime),
            DeliveryOutcome::StaleDrop => assert!(matches!(
                a.outcome(),
                HistoryOutcome::CorrectedLate | HistoryOutcome::TooOldDrop
            )),
        }
    }
}
fn same(a: &HistoryResult, b: &HistoryResult) {
    assert_eq!(a.snapshots().len(), b.snapshots().len());
    for (x, y) in a.snapshots().iter().zip(b.snapshots()) {
        assert_eq!(x.tick(), y.tick());
        assert_eq!(values(x), values(y));
    }
    assert_eq!(a.arrival_order(), b.arrival_order());
    assert_eq!(a.ledger().len(), b.ledger().len());
    assert_eq!(a.work().len(), b.work().len());
    for (x, y) in a.work().iter().zip(b.work()) {
        assert_eq!(
            (
                x.tick(),
                x.restored_sample_tick(),
                x.prediction_calls(),
                x.measurement_calls(),
                x.committed_checkpoints(),
                x.committed_observations(),
                x.peak_live_checkpoints(),
                x.peak_live_history_observations(),
                x.reserved_checkpoint_slots(),
                x.reserved_observation_slots()
            ),
            (
                y.tick(),
                y.restored_sample_tick(),
                y.prediction_calls(),
                y.measurement_calls(),
                y.committed_checkpoints(),
                y.committed_observations(),
                y.peak_live_checkpoints(),
                y.peak_live_history_observations(),
                y.reserved_checkpoint_slots(),
                y.reserved_observation_slots()
            )
        );
    }
    for (x, y) in a.ledger().iter().zip(b.ledger()) {
        assert_eq!(
            (x.id(), x.decision_tick(), x.outcome(), x.queue_occupancy()),
            (y.id(), y.decision_tick(), y.outcome(), y.queue_occupancy())
        );
    }
}

#[test]
fn tst_acc_001_late_velocity_rational_oracle() {
    let t = trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 1, [2., -2.])]);
    let h = history(&t, 1);
    let s = &h.snapshots()[1];
    for (a, b) in s.estimate().position.into_iter().zip([0.01, -0.01]) {
        near(a, b);
    }
    for (a, b) in s.estimate().velocity.into_iter().zip([1., -1.]) {
        near(a, b);
    }
    near(s.covariance()[0][0], 25.0002);
    near(s.covariance()[0][2], 0.02);
    near(s.covariance()[2][2], 2.);
    assert_eq!(s.integrated_position(), [0.02, -0.02]);
    assert_eq!(h.ledger()[0].outcome(), HistoryOutcome::CorrectedLate);
}

#[test]
fn tst_acc_002_zero_window_preserves_legacy_bits() {
    for delay in [0, 1, 5, 20, 500] {
        let t = trace(
            1,
            1,
            vec![
                obs(Sensor::Position, 0, 0, delay, [10., -5.]),
                obs(Sensor::Velocity, 0, 0, delay, [2., -2.]),
            ],
        );
        let h = history(&t, 0);
        let d = replay_delivery(&model(), &t).unwrap();
        assert_eq!(h.snapshots().len(), d.snapshots().len());
        for (a, b) in h.snapshots().iter().zip(d.snapshots()) {
            assert_eq!(values(a), legacy_values(b));
        }
        parity(&t, 0);
    }
}

#[test]
fn tst_acc_003_batch_late_and_current_once() {
    let late = trace(
        1,
        128,
        vec![
            obs(Sensor::Velocity, 0, 0, 5, [2., -2.]),
            obs(Sensor::Velocity, 1, 5, 5, [4., -4.]),
        ],
    );
    let ready = trace(
        1,
        128,
        vec![
            obs(Sensor::Velocity, 0, 0, 0, [2., -2.]),
            obs(Sensor::Velocity, 1, 5, 5, [4., -4.]),
        ],
    );
    let h = history(&late, 5);
    let d = replay_delivery(&model(), &ready).unwrap();
    assert_eq!(values(&h.snapshots()[5]), legacy_values(&d.snapshots()[5]));
    assert_eq!(h.work()[5].measurement_calls(), 2);
    assert_eq!(h.work()[5].prediction_calls(), 5);
}

#[test]
fn tst_beh_001_empty_trace_predicts_original_prior() {
    let t = trace(1, 128, vec![]);
    let h = history(&t, 20);
    let d = replay_delivery(&model(), &t).unwrap();
    assert_eq!(h.snapshots().len(), 101);
    assert_eq!(h.work().len(), 101);
    assert!(h.ledger().is_empty());
    assert_eq!(h.snapshots().len(), d.snapshots().len());
    for (a, b) in h.snapshots().iter().zip(d.snapshots()) {
        assert_eq!(values(a), legacy_values(b));
    }
}

#[test]
fn tst_bound_001_window_zero_and_maximum_valid() {
    for w in [0, 500] {
        let p = HistoryPlan::new(w).unwrap();
        assert_eq!(p.window_ticks(), w);
    }
}

#[test]
fn tst_neg_001_window_501_rejected() {
    assert!(matches!(
        HistoryPlan::new(501),
        Err(HistoryError::InvalidWindow { value: 501 })
    ));
}

#[test]
fn tst_neg_002_window_integer_max_rejected() {
    assert!(matches!(
        HistoryPlan::new(u32::MAX),
        Err(HistoryError::InvalidWindow { value: u32::MAX })
    ));
}

#[test]
fn tst_bound_002_inclusive_age_boundary_corrected() {
    let t = trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 20, [2., -2.])]);
    let h = history(&t, 20);
    assert_eq!(h.ledger()[0].outcome(), HistoryOutcome::CorrectedLate);
    assert_eq!(h.ledger()[0].decision_tick(), 20);
    near(h.snapshots()[20].estimate().position[0], 0.2);
}

#[test]
fn tst_bound_003_older_than_window_explicit_drop() {
    let t = trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 21, [2., -2.])]);
    let h = history(&t, 20);
    assert_eq!(h.ledger()[0].outcome(), HistoryOutcome::TooOldDrop);
    assert_eq!(h.ledger()[0].decision_tick(), 21);
    assert_eq!(h.snapshots()[21].estimate().velocity, [0., 0.]);
    assert_eq!(h.work()[21].restored_sample_tick(), None);
}

#[test]
fn tst_bound_004_max_age_initial_anchor() {
    let t = trace(6, 128, vec![obs(Sensor::Velocity, 0, 0, 500, [2., -2.])]);
    let h = history(&t, 500);
    let s = &h.snapshots()[500];
    near(s.estimate().position[0], 5.);
    near(s.covariance()[0][0], 75.);
    near(s.covariance()[0][2], 10.);
    near(s.covariance()[2][2], 2.);
    assert_eq!(h.work()[500].prediction_calls(), 500);
    assert_eq!(h.work()[500].measurement_calls(), 1);
}

#[test]
fn tst_bound_005_arrival_at_final_horizon_processed() {
    let t = trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 100, [2., -2.])]);
    let h = history(&t, 100);
    assert_eq!(h.ledger()[0].outcome(), HistoryOutcome::CorrectedLate);
    assert_eq!(h.arrival_order().len(), 1);
    near(h.snapshots()[100].estimate().position[0], 1.);
}

#[test]
fn tst_bound_006_beyond_horizon_retains_horizon_outcome() {
    let t = trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 101, [2., -2.])]);
    let h = history(&t, 500);
    assert_eq!(
        h.ledger()[0].outcome(),
        HistoryOutcome::UndeliveredAtHorizon
    );
    assert_eq!(h.ledger()[0].decision_tick(), 100);
    assert!(h.arrival_order().is_empty());
    assert_eq!(h.snapshots()[100].estimate().velocity, [0., 0.]);
}

#[test]
fn tst_beh_002_simultaneous_sensors_canonical() {
    let t = trace(
        1,
        128,
        vec![
            obs(Sensor::Velocity, 0, 0, 1, [2., -2.]),
            obs(Sensor::Position, 0, 0, 1, [10., -5.]),
        ],
    );
    let h = history(&t, 1);
    let s = &h.snapshots()[1];
    near(s.estimate().position[0], 5.01);
    near(s.estimate().position[1], -2.51);
    assert_eq!(s.held_position(), [10., -5.]);
    assert_eq!(s.integrated_position(), [10.02, -5.02]);
    assert_eq!(h.arrival_order()[0].sensor(), Sensor::Position);
    assert_eq!(h.work()[1].measurement_calls(), 2);
}

#[test]
fn tst_prop_001_input_permutation_irrelevant() {
    let a = vec![
        obs(Sensor::Velocity, 0, 0, 10, [2., -2.]),
        obs(Sensor::Position, 0, 0, 10, [10., -5.]),
        obs(Sensor::Velocity, 1, 5, 10, [4., -4.]),
    ];
    let mut b = a.clone();
    b.reverse();
    same(
        &history(&trace(1, 128, a), 10),
        &history(&trace(1, 128, b), 10),
    );
}

#[test]
fn tst_perf_001_exact_maximum_correction_batch() {
    let src = run(&Config {
        duration_seconds: 6,
        outage: None,
        ..Config::default()
    })
    .unwrap();
    let t = Trace::from_samples(
        6,
        &src.samples,
        DeliveryPlan::new(0, 0, 128, None, None).unwrap(),
    )
    .unwrap();
    let changed = t
        .observations()
        .iter()
        .map(|o| {
            obs(
                o.id().sensor(),
                o.id().sequence(),
                o.sample_tick(),
                if o.id().sensor() == Sensor::Position && o.sample_tick() == 20 {
                    520
                } else {
                    o.planned_arrival_tick()
                },
                o.value(),
            )
        })
        .collect();
    let h = history(&trace(6, 128, changed), 500);
    let d = replay_delivery(&model(), &t).unwrap();
    assert_eq!(h.work()[520].restored_sample_tick(), Some(20));
    assert_eq!(h.work()[520].prediction_calls(), 501);
    assert_eq!(h.work()[520].measurement_calls(), 127);
    assert_eq!(
        values(&h.snapshots()[520]),
        legacy_values(&d.snapshots()[520])
    );
    assert_ne!(
        values(&h.snapshots()[20]),
        legacy_values(&d.snapshots()[20])
    );
}

#[test]
fn tst_perf_002_actual_history_and_work_caps() {
    let src = run(&Config {
        duration_seconds: 6,
        outage: None,
        ..Config::default()
    })
    .unwrap();
    let t = Trace::from_samples(
        6,
        &src.samples,
        DeliveryPlan::new(499, 500, 128, None, None).unwrap(),
    )
    .unwrap();
    for window in [0, 1, 20, 499, 500] {
        let h = history(&t, window);
        for x in h.work() {
            assert!(x.prediction_calls() <= window as usize + 1);
            assert!(x.measurement_calls() <= 127);
            assert!(x.committed_checkpoints() <= window as usize + 2);
            assert!(x.committed_observations() <= 127);
            assert!(x.peak_live_checkpoints() <= 2 * (window as usize + 2));
            assert!(x.peak_live_history_observations() <= 254);
            assert!(x.reserved_checkpoint_slots() >= x.peak_live_checkpoints());
            assert!(x.reserved_observation_slots() >= x.peak_live_history_observations());
        }
    }
}

#[test]
fn tst_beh_003_normal_work_counts_actual_calls() {
    let t = trace(
        1,
        128,
        vec![
            obs(Sensor::Velocity, 0, 0, 0, [2., -2.]),
            obs(Sensor::Position, 0, 0, 0, [10., -5.]),
        ],
    );
    let h = history(&t, 500);
    assert_eq!(h.work()[0].prediction_calls(), 0);
    assert_eq!(h.work()[0].measurement_calls(), 2);
    assert_eq!(h.work()[1].prediction_calls(), 1);
    assert_eq!(h.work()[1].measurement_calls(), 0);
    assert_eq!(h.work()[1].restored_sample_tick(), None);
}

#[test]
fn tst_beh_004_correction_branch_no_wasted_normal_work() {
    let h = history(
        &trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 1, [2., -2.])]),
        1,
    );
    assert_eq!(h.work()[1].restored_sample_tick(), Some(0));
    assert_eq!(h.work()[1].prediction_calls(), 1);
    assert_eq!(h.work()[1].measurement_calls(), 1);
}

#[test]
fn tst_prop_002_sampled_future_prefix_causality() {
    let a = trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 5, [2., -2.])]);
    let b = trace(
        1,
        128,
        vec![
            obs(Sensor::Velocity, 0, 0, 5, [2., -2.]),
            obs(Sensor::Velocity, 3, 15, 15, [8., -8.]),
        ],
    );
    let x = history(&a, 20);
    let y = history(&b, 20);
    for i in 0..=10 {
        assert_eq!(values(&x.snapshots()[i]), values(&y.snapshots()[i]));
        assert_eq!(
            (
                x.work()[i].prediction_calls(),
                x.work()[i].measurement_calls()
            ),
            (
                y.work()[i].prediction_calls(),
                y.work()[i].measurement_calls()
            )
        );
    }
    assert_eq!(
        x.arrival_order()
            .iter()
            .filter(|id| id.sequence() == 0)
            .count(),
        1
    );
}

#[test]
fn tst_prop_003_unarrived_payload_not_numerical_input() {
    let x = history(
        &trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 50, [2., -2.])]),
        50,
    );
    let y = history(
        &trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 50, [200., -200.])]),
        50,
    );
    for i in 0..50 {
        assert_eq!(values(&x.snapshots()[i]), values(&y.snapshots()[i]));
    }
    assert_ne!(values(&x.snapshots()[50]), values(&y.snapshots()[50]));
}

#[test]
fn tst_contract_001_immediate_bypasses_full_waiting() {
    let t = trace(
        1,
        1,
        vec![
            obs(Sensor::Velocity, 0, 0, 30, [2., -2.]),
            obs(Sensor::Velocity, 1, 5, 35, [4., -4.]),
            obs(Sensor::Position, 1, 20, 20, [10., -5.]),
        ],
    );
    parity(&t, 30);
    let h = history(&t, 30);
    assert_eq!(h.ledger()[1].outcome(), HistoryOutcome::QueueFullDrop);
    assert_eq!(h.ledger()[1].queue_occupancy(), Some(1));
    assert_eq!(h.ledger()[2].outcome(), HistoryOutcome::AcceptedOnTime);
}

#[test]
fn tst_contract_002_drain_due_before_admit() {
    let t = trace(
        1,
        1,
        vec![
            obs(Sensor::Velocity, 0, 0, 5, [2., -2.]),
            obs(Sensor::Velocity, 1, 5, 10, [4., -4.]),
        ],
    );
    parity(&t, 5);
    let h = history(&t, 5);
    assert!(
        h.ledger()
            .iter()
            .all(|e| e.outcome() == HistoryOutcome::CorrectedLate)
    );
}

#[test]
fn tst_contract_003_scheduled_loss_missing_sequence() {
    let t = Trace::new(
        1,
        1,
        [None, std::num::NonZeroU32::new(2)],
        vec![
            obs(Sensor::Velocity, 1, 5, 5, [2., -2.]),
            obs(Sensor::Velocity, 2, 10, 10, [4., -4.]),
        ],
    )
    .unwrap();
    parity(&t, 20);
    let h = history(&t, 20);
    assert_eq!(h.ledger()[0].outcome(), HistoryOutcome::ScriptedLoss);
    assert_eq!(h.ledger()[0].decision_tick(), 5);
    assert_eq!(h.ledger()[1].outcome(), HistoryOutcome::AcceptedOnTime);
}

#[test]
fn tst_inv_001_input_bytes_preserved() {
    let t = trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 20, [2., -2.])]);
    let m = model();
    let bytes = t.encode_tsv(&m).unwrap();
    let h = replay_history(&m, &t, &HistoryPlan::new(20).unwrap()).unwrap();
    assert_eq!(h.ledger().len(), 1);
    assert_eq!(t.encode_tsv(&m).unwrap(), bytes);
    assert_eq!(m.assumed_velocity_sigma(), 2.);
}

#[test]
fn tst_beh_005_requested_model_assumptions_used() {
    let t = trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 1, [2., -2.])]);
    let m = ReplayModel::new(5., 0., 0.).unwrap();
    let h = replay_history(&m, &t, &HistoryPlan::new(1).unwrap()).unwrap();
    near(
        h.snapshots()[1].estimate().velocity[0],
        2. * 4. / (4. + 1e-12),
    );
}

#[test]
fn tst_state_001_repeated_correction_preserves_information() {
    let t = trace(
        1,
        128,
        vec![
            obs(Sensor::Velocity, 0, 0, 1, [2., -2.]),
            obs(Sensor::Position, 0, 0, 20, [10., -5.]),
        ],
    );
    let ready = trace(
        1,
        128,
        vec![
            obs(Sensor::Velocity, 0, 0, 0, [2., -2.]),
            obs(Sensor::Position, 0, 0, 0, [10., -5.]),
        ],
    );
    let h = history(&t, 20);
    let d = replay_delivery(&model(), &ready).unwrap();
    assert_eq!(
        values(&h.snapshots()[20]),
        legacy_values(&d.snapshots()[20])
    );
    assert_eq!(h.work()[1].measurement_calls(), 1);
    assert_eq!(h.work()[20].measurement_calls(), 2);
}

#[test]
fn tst_state_002_emitted_prefix_never_retroactively_rewritten() {
    let h = history(
        &trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 1, [2., -2.])]),
        1,
    );
    assert_eq!(h.snapshots()[0].estimate().velocity, [0., 0.]);
    assert_eq!(h.snapshots()[0].integrated_position(), [0., 0.]);
    near(h.snapshots()[1].estimate().velocity[0], 1.);
}

#[test]
fn tst_neg_003_overflow_preserves_typed_source() {
    let t = trace(
        1,
        128,
        vec![
            obs(Sensor::Position, 0, 0, 0, [f64::MAX, 0.]),
            obs(Sensor::Velocity, 0, 0, 0, [f64::MAX, 0.]),
        ],
    );
    let m = ReplayModel::new(0., 0., 0.).unwrap();
    let bytes = t.encode_tsv(&m).unwrap();
    let e = replay_history(&m, &t, &HistoryPlan::new(500).unwrap()).unwrap_err();
    match &e {
        HistoryError::NumericalFailure {
            arrival_tick,
            replay_tick,
            observation,
            source,
        } => {
            assert_eq!((*arrival_tick, *replay_tick, *observation), (1, 1, None));
            assert_eq!(*source, Error::NumericalFailure);
        }
        _ => panic!("expected exact numerical failure"),
    };
    assert!(
        std::error::Error::source(&e)
            .unwrap()
            .downcast_ref::<Error>()
            .is_some()
    );
    assert_eq!(t.encode_tsv(&m).unwrap(), bytes);
}

#[test]
fn tst_prop_004_repeat_deterministic_full_result() {
    let t = trace(
        1,
        3,
        vec![
            obs(Sensor::Velocity, 0, 0, 1, [2., -2.]),
            obs(Sensor::Position, 0, 0, 20, [10., -5.]),
        ],
    );
    let a = history(&t, 20);
    let b = history(&t, 20);
    same(&a, &b);
    for (x, y) in a.work().iter().zip(b.work()) {
        assert_eq!(
            (
                x.tick(),
                x.prediction_calls(),
                x.measurement_calls(),
                x.committed_checkpoints(),
                x.committed_observations(),
                x.peak_live_checkpoints(),
                x.peak_live_history_observations(),
                x.reserved_checkpoint_slots(),
                x.reserved_observation_slots()
            ),
            (
                y.tick(),
                y.prediction_calls(),
                y.measurement_calls(),
                y.committed_checkpoints(),
                y.committed_observations(),
                y.peak_live_checkpoints(),
                y.peak_live_history_observations(),
                y.reserved_checkpoint_slots(),
                y.reserved_observation_slots()
            )
        );
    }
}

#[test]
fn tst_beh_006_zero_window_signed_zero_legacy_bits() {
    let t = trace(
        1,
        128,
        vec![
            obs(Sensor::Velocity, 0, 0, 0, [-0., 0.]),
            obs(Sensor::Position, 0, 0, 0, [0., -0.]),
        ],
    );
    let h = history(&t, 0);
    let d = replay_delivery(&model(), &t).unwrap();
    assert_eq!(h.snapshots().len(), d.snapshots().len());
    for (a, b) in h.snapshots().iter().zip(d.snapshots()) {
        assert_eq!(values(a), legacy_values(b));
    }
}

#[test]
fn tst_struct_001_public_types_send_sync_signature() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<HistoryPlan>();
    send_sync::<HistoryResult>();
    send_sync::<HistoryWork>();
    let _: fn(&ReplayModel, &Trace, &HistoryPlan) -> Result<HistoryResult, HistoryError> =
        replay_history;
}

#[test]
fn tst_anti_001_private_plan_cannot_be_forged() {
    use std::{path::PathBuf, process::Command};
    let exe = std::env::current_exe().unwrap();
    let deps = exe.parent().unwrap();
    let rlib = std::fs::read_dir(deps)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("libfusion_lab-")
                && p.extension().is_some_and(|e| e == "rlib")
        })
        .max_by_key(|p| std::fs::metadata(p).unwrap().modified().unwrap())
        .unwrap();
    let root = std::env::temp_dir().join(format!(
        "history-privacy-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let source = root.join("private.rs");
    std::fs::write(
        &source,
        "use fusion_lab::HistoryPlan;fn main(){let _=HistoryPlan{window_ticks:1};}",
    )
    .unwrap();
    let result = Command::new(env!("HISTORY_TEST_RUSTC_PATH"))
        .arg("--edition=2024")
        .arg(&source)
        .arg("--crate-name=history_privacy")
        .arg("--extern")
        .arg(format!("fusion_lab={}", rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("--out-dir")
        .arg(&root)
        .output()
        .unwrap();
    let diagnostic = String::from_utf8_lossy(&result.stderr);
    assert!(
        !result.status.success() && diagnostic.contains("E0451"),
        "actual privacy compilation: {diagnostic}"
    );
    let _owned: PathBuf = root;
}

#[test]
fn tst_neg_004_rewind_overflow_preserves_arrival_and_replay_tick() {
    let t = trace(
        3,
        128,
        vec![
            obs(Sensor::Position, 0, 0, 0, [1e308, 1e308]),
            obs(Sensor::Velocity, 0, 0, 300, [1e308, 1e308]),
        ],
    );
    let m = ReplayModel::new(0., 0., 0.).unwrap();
    let before = t.encode_tsv(&m).unwrap();
    let stale = replay_delivery(&m, &t).unwrap();
    assert_eq!(stale.snapshots().len(), 301);
    assert!(stale.snapshots().iter().all(|s| {
        s.estimate()
            .position
            .into_iter()
            .chain(s.estimate().velocity)
            .chain(s.covariance().iter().flatten().copied())
            .chain(s.held_position())
            .chain(s.integrated_position())
            .all(f64::is_finite)
    }));
    let short = replay_history(&m, &t, &HistoryPlan::new(299).unwrap()).unwrap();
    assert_eq!(short.ledger()[1].outcome(), HistoryOutcome::TooOldDrop);
    assert_eq!(short.snapshots().len(), 301);
    for (a, b) in short.snapshots().iter().zip(stale.snapshots()) {
        assert_eq!(values(a), legacy_values(b));
    }
    let e = replay_history(&m, &t, &HistoryPlan::new(300).unwrap()).unwrap_err();
    match &e {
        HistoryError::NumericalFailure {
            arrival_tick,
            replay_tick,
            observation,
            source,
        } => {
            assert_eq!((*arrival_tick, *replay_tick, *observation), (300, 80, None));
            assert_eq!(*source, Error::NumericalFailure);
        }
        _ => panic!("expected rewind numerical failure"),
    };
    assert!(
        std::error::Error::source(&e)
            .unwrap()
            .downcast_ref::<Error>()
            .is_some()
    );
    assert_eq!(t.encode_tsv(&m).unwrap(), before);
}

#[test]
fn tst_beh_007_late_position_requested_sigma_oracle() {
    let t = trace(1, 128, vec![obs(Sensor::Position, 0, 0, 1, [10., -5.])]);
    let m = ReplayModel::new(10., 2., 0.).unwrap();
    let h = replay_history(&m, &t, &HistoryPlan::new(1).unwrap()).unwrap();
    let s = &h.snapshots()[1];
    near(s.estimate().position[0], 2.);
    near(s.estimate().position[1], -1.);
    near(s.covariance()[0][0], 20.0004);
    near(s.covariance()[0][2], 0.04);
    near(s.covariance()[2][2], 4.);
}

#[test]
fn tst_beh_008_nonzero_process_density_normal_and_rewind() {
    for (q, p, v) in [(0.1, 1., 0.15), (1., 10., 3.)] {
        let m = ReplayModel::new(p, v, q).unwrap();
        let ready = trace(
            1,
            128,
            vec![
                obs(Sensor::Position, 0, 0, 0, [10., -5.]),
                obs(Sensor::Velocity, 0, 0, 0, [2., -2.]),
            ],
        );
        let delayed = trace(
            1,
            128,
            vec![
                obs(Sensor::Position, 0, 0, 1, [10., -5.]),
                obs(Sensor::Velocity, 0, 0, 0, [2., -2.]),
            ],
        );
        let h = replay_history(&m, &delayed, &HistoryPlan::new(1).unwrap()).unwrap();
        let plain = replay_history(&m, &ready, &HistoryPlan::new(1).unwrap()).unwrap();
        let d = replay_delivery(&m, &ready).unwrap();
        assert_eq!(plain.snapshots().len(), d.snapshots().len());
        for (a, b) in plain.snapshots().iter().zip(d.snapshots()) {
            assert_eq!(values(a), legacy_values(b));
        }
        for i in 1..=100 {
            assert_eq!(values(&h.snapshots()[i]), legacy_values(&d.snapshots()[i]));
        }
    }
}

#[test]
fn tst_contract_004_scripted_loss_payload_never_consumed() {
    let loss = Trace::new(
        1,
        128,
        [std::num::NonZeroU32::new(1), None],
        vec![
            obs(Sensor::Position, 0, 0, 0, [1e308, 1e308]),
            obs(Sensor::Velocity, 0, 0, 0, [2., -2.]),
        ],
    )
    .unwrap();
    let without = trace(1, 128, vec![obs(Sensor::Velocity, 0, 0, 0, [2., -2.])]);
    let a = history(&loss, 500);
    let b = history(&without, 500);
    assert_eq!(a.ledger()[0].outcome(), HistoryOutcome::ScriptedLoss);
    assert_eq!(a.snapshots().len(), b.snapshots().len());
    for (x, y) in a.snapshots().iter().zip(b.snapshots()) {
        assert_eq!(values(x), values(y));
    }
}

#[test]
fn tst_contract_005_queue_full_payload_never_consumed() {
    let rejected = trace(
        1,
        1,
        vec![
            obs(Sensor::Position, 0, 0, 20, [10., -5.]),
            obs(Sensor::Velocity, 0, 0, 10, [999., -999.]),
        ],
    );
    let without = trace(1, 1, vec![obs(Sensor::Position, 0, 0, 20, [10., -5.])]);
    let a = history(&rejected, 500);
    let b = history(&without, 500);
    assert_eq!(a.ledger()[1].outcome(), HistoryOutcome::QueueFullDrop);
    assert_eq!(a.snapshots().len(), b.snapshots().len());
    for (x, y) in a.snapshots().iter().zip(b.snapshots()) {
        assert_eq!(values(x), values(y));
    }
}

#[test]
fn tst_inv_002_too_old_payload_absent_from_tail_and_rewind() {
    let with_old = trace(
        1,
        128,
        vec![
            obs(Sensor::Velocity, 0, 0, 21, [999., -999.]),
            obs(Sensor::Velocity, 1, 5, 25, [2., -2.]),
            obs(Sensor::Position, 1, 20, 40, [10., -5.]),
        ],
    );
    let without = trace(
        1,
        128,
        vec![
            obs(Sensor::Velocity, 1, 5, 25, [2., -2.]),
            obs(Sensor::Position, 1, 20, 40, [10., -5.]),
        ],
    );
    let a = history(&with_old, 20);
    let b = history(&without, 20);
    assert_eq!(a.ledger()[0].outcome(), HistoryOutcome::TooOldDrop);
    assert_eq!(a.snapshots().len(), b.snapshots().len());
    for (x, y) in a.snapshots().iter().zip(b.snapshots()) {
        assert_eq!(values(x), values(y));
    }
}
