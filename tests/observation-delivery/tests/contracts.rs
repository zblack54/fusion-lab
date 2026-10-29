mod support;
use fusion_lab::*;
use support::*;

#[test]
fn tst_acc_001_zero_delay_matches_original_bits() {
    for config in [
        Config::default(),
        Config {
            duration_seconds: 2,
            scenario: Scenario::Straight,
            position_sigma: 0.0,
            velocity_sigma: 0.0,
            acceleration_density: 0.0,
            outage: None,
            ..Config::default()
        },
        Config {
            duration_seconds: 1,
            outage: None,
            velocity_bias: [0.2, -0.1],
            velocity_drift_rate: [0.4, -0.2],
            assumed_position_sigma: Some(3.0),
            assumed_velocity_sigma: Some(2.0),
            ..Config::default()
        },
    ] {
        let source = run(&config).unwrap();
        let plan = DeliveryPlan::new(0, 0, 128, None, None).unwrap();
        let input = Trace::from_samples(config.duration_seconds, &source.samples, plan).unwrap();
        let model = ReplayModel::new(
            config
                .assumed_position_sigma
                .unwrap_or(config.position_sigma),
            config
                .assumed_velocity_sigma
                .unwrap_or(config.velocity_sigma),
            config.acceleration_density,
        )
        .unwrap();
        let actual = replay_delivery(&model, &input).unwrap();
        assert_source_equal(&actual, &source);
        assert!(
            actual
                .ledger()
                .iter()
                .all(|entry| entry.outcome() == DeliveryOutcome::Accepted)
        );
    }
}

#[test]
fn tst_acc_002_record_import_replay_same_result() {
    let original = golden_input();
    let first = replay_delivery(&model(), &original).unwrap();
    let bytes = original.encode_tsv(&model()).unwrap();
    let (imported_model, imported) = Trace::decode_tsv(&bytes).unwrap();
    let again = replay_delivery(&imported_model, &imported).unwrap();
    assert_bitwise(&first, &again);
    assert_eq!(again.snapshots().len(), 101);
    assert_eq!(
        ledger(&again),
        vec![
            (Sensor::Position, 0, DeliveryOutcome::Accepted, 0, None),
            (Sensor::Velocity, 0, DeliveryOutcome::StaleDrop, 1, None),
            (Sensor::Velocity, 1, DeliveryOutcome::Accepted, 5, None)
        ]
    );
    assert_eq!(again.encode_ledger_tsv().unwrap(), golden_ledger());
}

#[test]
fn tst_acc_003_projection_copies_only_observations() {
    let source = run(&Config {
        duration_seconds: 1,
        outage: None,
        ..Config::default()
    })
    .unwrap();
    let plan = || DeliveryPlan::new(3, 5, 8, nz(3), nz(4)).unwrap();
    let expected = Trace::from_samples(1, &source.samples, plan())
        .unwrap()
        .encode_tsv(&model())
        .unwrap();
    let mut poisoned = source.samples.clone();
    for row in &mut poisoned {
        row.truth.position = [f64::NAN; 2];
        row.truth.velocity = [f64::NAN; 2];
        row.estimate.position = [f64::NAN; 2];
        row.estimate.velocity = [f64::NAN; 2];
        row.covariance = [[f64::NAN; 4]; 4];
        row.held_position = [f64::NAN; 2];
        row.integrated_position = [f64::NAN; 2];
        row.position_outage = !row.position_outage;
    }
    let owned = Trace::from_samples(1, &poisoned, plan()).unwrap();
    drop(poisoned);
    assert_eq!(owned.encode_tsv(&model()).unwrap(), expected);
}

#[test]
fn tst_acc_004_numerical_failure_has_no_successful_prefix() {
    let input = trace(vec![
        observation(Sensor::Position, 0, 0, 0, [f64::MAX; 2]),
        observation(Sensor::Velocity, 0, 0, 0, [f64::MAX; 2]),
    ]);
    let model = ReplayModel::new(0.0, 0.0, 0.0).unwrap();
    let before = input.encode_tsv(&model).unwrap();
    let error = replay_delivery(&model, &input).unwrap_err();
    match &error {
        DeliveryError::NumericalFailure {
            tick,
            observation,
            source,
        } => {
            assert_eq!(*tick, 1);
            assert!(observation.is_none());
            assert_eq!(*source, Error::NumericalFailure);
        }
        other => panic!("wrong failure: {other:?}"),
    };
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(input.encode_tsv(&model).unwrap(), before);
}

#[test]
fn tst_beh_001_readonly_observation_identity() {
    let value = observation(Sensor::Velocity, 1, 5, 9, [-0.0, 1.25]);
    assert_eq!(value.id().sensor(), Sensor::Velocity);
    assert_eq!(value.id().sequence(), 1);
    assert_eq!(value.sample_tick(), 5);
    assert_eq!(value.planned_arrival_tick(), 9);
    assert_eq!(value.value()[0].to_bits(), (-0.0_f64).to_bits());
    assert_eq!(value.value()[1], 1.25);
}

#[test]
fn tst_beh_002_raw_constructor_normalizes_ties() {
    let input = trace(vec![
        observation(Sensor::Velocity, 1, 5, 5, [1.0; 2]),
        observation(Sensor::Velocity, 0, 0, 5, [2.0; 2]),
        observation(Sensor::Position, 0, 0, 5, [3.0; 2]),
    ]);
    let identities = input
        .observations()
        .iter()
        .map(|o| (o.id().sensor(), o.id().sequence()))
        .collect::<Vec<_>>();
    assert_eq!(
        identities,
        vec![
            (Sensor::Position, 0),
            (Sensor::Velocity, 0),
            (Sensor::Velocity, 1)
        ]
    );
    let result = replay_delivery(&model(), &input).unwrap();
    assert_eq!(ids(result.arrival_order()), identities);
}

#[test]
fn tst_beh_003_stale_is_equivalent_to_omission() {
    let empty = replay_delivery(&model(), &trace(vec![])).unwrap();
    let late = replay_delivery(
        &model(),
        &trace(vec![observation(Sensor::Velocity, 0, 0, 1, [1.0, 0.5])]),
    )
    .unwrap();
    for (a, b) in empty.snapshots().iter().zip(late.snapshots()) {
        assert_eq!(a.estimate(), b.estimate());
        assert_eq!(a.covariance(), b.covariance());
        assert_eq!(a.held_position(), b.held_position());
        assert_eq!(a.integrated_position(), b.integrated_position());
    }
    let s = &late.snapshots()[1];
    assert_eq!(s.estimate().position, [0.0; 2]);
    close(s.covariance()[0][0], 25.0004, 1e-12);
    close(s.covariance()[0][2], 0.04, 1e-12);
    assert_eq!(s.covariance()[2][2], 4.0);
    assert_eq!(
        ledger(&late),
        vec![(Sensor::Velocity, 0, DeliveryOutcome::StaleDrop, 1, None)]
    );
}

#[test]
fn tst_beh_004_independent_cross_covariance_update() {
    let result = replay_delivery(
        &model(),
        &trace(vec![
            observation(Sensor::Position, 0, 0, 0, [10.0, 20.0]),
            observation(Sensor::Velocity, 1, 5, 5, [6.0, 3.0]),
        ]),
    )
    .unwrap();
    let zero = &result.snapshots()[0];
    assert_eq!(zero.estimate().position, [5.0, 10.0]);
    assert_eq!(zero.estimate().velocity, [0.0; 2]);
    close(zero.covariance()[0][0], 12.5, 1e-12);
    let fifth = &result.snapshots()[5];
    for (a, e) in fifth.estimate().position.into_iter().zip([5.15, 10.075]) {
        close(a, e, 1e-12);
    }
    for (a, e) in fifth.estimate().velocity.into_iter().zip([3.0, 1.5]) {
        close(a, e, 1e-12);
    }
    close(fifth.covariance()[0][0], 12.505, 1e-12);
    close(fifth.covariance()[0][2], 0.1, 1e-12);
    close(fifth.covariance()[2][2], 2.0, 1e-12);
    assert_eq!(fifth.held_position(), [10.0, 20.0]);
    assert_eq!(fifth.integrated_position(), [10.0, 20.0]);
    for (a, e) in result.snapshots()[6]
        .integrated_position()
        .into_iter()
        .zip([10.06, 20.03])
    {
        close(a, e, 1e-12);
    }
}

#[test]
fn tst_beh_005_loss_uses_scheduled_sequence_gaps() {
    let mut source = run(&Config {
        duration_seconds: 1,
        outage: None,
        ..Config::default()
    })
    .unwrap()
    .samples;
    source[20].position_measurement = None;
    source[80].position_measurement = None;
    source[100].position_measurement = None;
    let plan = DeliveryPlan::new(0, 0, 4, nz(3), None).unwrap();
    let input = Trace::from_samples(1, &source, plan).unwrap();
    let result = replay_delivery(&model(), &input).unwrap();
    let positions = result
        .ledger()
        .iter()
        .filter(|e| e.id().sensor() == Sensor::Position)
        .map(|e| (e.id().sequence(), e.outcome()))
        .collect::<Vec<_>>();
    assert_eq!(
        positions,
        vec![
            (0, DeliveryOutcome::Accepted),
            (2, DeliveryOutcome::ScriptedLoss),
            (3, DeliveryOutcome::Accepted)
        ]
    );
}

#[test]
fn tst_beh_006_fixed_prior_process_noise_and_floor() {
    let empty = trace(vec![]);
    let result = replay_delivery(&ReplayModel::new(0.0, 0.0, 3.0).unwrap(), &empty).unwrap();
    let zero = &result.snapshots()[0];
    assert_eq!(zero.estimate().position, [0.0; 2]);
    assert_eq!(
        *zero.covariance(),
        [
            [25.0, 0.0, 0.0, 0.0],
            [0.0, 25.0, 0.0, 0.0],
            [0.0, 0.0, 4.0, 0.0],
            [0.0, 0.0, 0.0, 4.0]
        ]
    );
    let final_s = &result.snapshots()[100];
    close(final_s.covariance()[0][0], 30.0, 1e-9);
    close(final_s.covariance()[0][2], 5.5, 1e-10);
    close(final_s.covariance()[2][2], 7.0, 1e-10);
    let exact = replay_delivery(
        &ReplayModel::new(0.0, 0.0, 0.0).unwrap(),
        &trace(vec![observation(Sensor::Velocity, 0, 0, 0, [1.0, 0.5])]),
    )
    .unwrap();
    close(
        exact.snapshots()[0].estimate().velocity[0],
        0.99999999999975,
        5e-16,
    );
    close(
        exact.snapshots()[0].covariance()[2][2],
        9.9999999999975e-13,
        2e-24,
    );
}

#[test]
fn tst_beh_007_model_preserves_requested_endpoints() {
    for values in [[0.0, 0.0, 0.0], [100.0, 100.0, 100.0], [-0.0, 0.0, 0.0]] {
        let model = ReplayModel::new(values[0], values[1], values[2]).unwrap();
        assert_eq!(
            model.assumed_position_sigma().to_bits(),
            values[0].to_bits()
        );
        assert_eq!(
            model.assumed_velocity_sigma().to_bits(),
            values[1].to_bits()
        );
        assert_eq!(model.acceleration_density().to_bits(), values[2].to_bits());
    }
}

#[test]
fn tst_bound_001_duration_and_capacity_edges() {
    for duration in [1, 3600] {
        for capacity in [1, 128] {
            let input = Trace::new(duration, capacity, [None, None], vec![]).unwrap();
            assert_eq!(input.duration_seconds(), duration);
            assert_eq!(input.pending_capacity(), capacity);
        }
    }
    for value in [0, 3601] {
        assert!(
            matches!(Trace::new(value,1,[None,None],vec![]),Err(DeliveryError::InvalidDuration{value:v})if v==value)
        );
    }
    for value in [0, 129] {
        assert!(
            matches!(Trace::new(1,value,[None,None],vec![]),Err(DeliveryError::InvalidCapacity{value:v})if v==value)
        );
        assert!(
            matches!(DeliveryPlan::new(0,0,value,None,None),Err(DeliveryError::InvalidCapacity{value:v})if v==value)
        );
    }
}

#[test]
fn tst_bound_002_delay_endpoints_and_plan() {
    for sensor in [Sensor::Position, Sensor::Velocity] {
        for delay in [0, 500] {
            let o = Observation::new(sensor, 0, 0, delay, [0.0; 2]).unwrap();
            assert_eq!(o.planned_arrival_tick() - o.sample_tick(), delay);
        }
    }
    let source = run(&Config {
        duration_seconds: 1,
        outage: None,
        ..Config::default()
    })
    .unwrap();
    for delays in [(0, 500), (500, 0)] {
        let plan = DeliveryPlan::new(delays.0, delays.1, 128, None, None).unwrap();
        let input = Trace::from_samples(1, &source.samples, plan).unwrap();
        for o in input.observations() {
            let expected = match o.id().sensor() {
                Sensor::Position => delays.0,
                Sensor::Velocity => delays.1,
            };
            assert_eq!(o.planned_arrival_tick() - o.sample_tick(), expected);
        }
    }
    let input = Trace::from_samples(
        1,
        &source.samples,
        DeliveryPlan::new(0, 0, 128, nz(3), nz(4)).unwrap(),
    )
    .unwrap();
    assert_eq!(input.drop_every(Sensor::Position), nz(3));
    assert_eq!(input.drop_every(Sensor::Velocity), nz(4));
}

#[test]
fn tst_bound_003_final_arrival_and_future_horizon() {
    let input = Trace::new(
        1,
        1,
        [None, None],
        vec![
            observation(Sensor::Velocity, 19, 95, 100, [2.0; 2]),
            observation(Sensor::Position, 5, 100, 101, [10.0; 2]),
            observation(Sensor::Velocity, 20, 100, 101, [3.0; 2]),
        ],
    )
    .unwrap();
    let result = replay_delivery(&model(), &input).unwrap();
    assert_eq!(result.snapshots().len(), 101);
    assert_eq!(result.snapshots().last().unwrap().tick(), 100);
    assert_eq!(
        ledger(&result),
        vec![
            (Sensor::Velocity, 19, DeliveryOutcome::StaleDrop, 100, None),
            (
                Sensor::Position,
                5,
                DeliveryOutcome::UndeliveredAtHorizon,
                100,
                None
            ),
            (
                Sensor::Velocity,
                20,
                DeliveryOutcome::QueueFullDrop,
                100,
                Some(1)
            )
        ]
    );
    assert_eq!(ids(result.arrival_order()), vec![(Sensor::Velocity, 19)]);
    let times = result
        .ledger()
        .iter()
        .map(|e| (e.sample_tick(), e.planned_arrival_tick(), e.decision_tick()))
        .collect::<Vec<_>>();
    assert_eq!(
        times,
        vec![(95, 100, 100), (100, 101, 100), (100, 101, 100)]
    );
}

#[test]
fn tst_bound_004_attainable_127_arrival_burst() {
    let mut offered = Vec::new();
    let mut expected = Vec::new();
    for tick in 0..=500 {
        if tick % 20 == 0 {
            offered.push(observation(
                Sensor::Position,
                tick / 20,
                tick,
                500,
                [0.0; 2],
            ));
            expected.push((Sensor::Position, tick / 20));
        }
        if tick % 5 == 0 {
            offered.push(observation(Sensor::Velocity, tick / 5, tick, 500, [0.0; 2]));
            expected.push((Sensor::Velocity, tick / 5));
        }
    }
    assert_eq!(offered.len(), 127);
    let input = Trace::new(5, 128, [None, None], offered).unwrap();
    let result = replay_delivery(&model(), &input).unwrap();
    assert_eq!(ids(result.arrival_order()), expected);
    assert_eq!(
        result
            .ledger()
            .iter()
            .filter(|e| e.outcome() == DeliveryOutcome::StaleDrop)
            .count(),
        125
    );
    assert_eq!(
        result
            .ledger()
            .iter()
            .filter(|e| e.outcome() == DeliveryOutcome::Accepted)
            .count(),
        2
    );
    assert!(
        result
            .ledger()
            .iter()
            .all(|e| e.queue_occupancy().is_none())
    );
}

#[test]
fn tst_bound_005_extreme_float_tokens_and_split_model_header() {
    let small = f64::from_bits(1);
    assert_eq!((-small).to_string().len(), 327);
    let model = ReplayModel::new(small, small, small).unwrap();
    let input = trace(vec![observation(
        Sensor::Position,
        0,
        0,
        0,
        [-small, f64::MAX],
    )]);
    let bytes = input.encode_tsv(&model).unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    assert!(text.lines().all(|line| line.len() <= 1024));
    assert!(text.contains(&format!("position_assumed_sigma\t{small}\nvelocity_assumed_sigma\t{small}\nacceleration_density\t{small}\n")));
    let (m, t) = Trace::decode_tsv(&bytes).unwrap();
    assert_eq!(m.assumed_position_sigma().to_bits(), small.to_bits());
    assert_eq!(m.assumed_velocity_sigma().to_bits(), small.to_bits());
    assert_eq!(m.acceleration_density().to_bits(), small.to_bits());
    assert_eq!(t.observations()[0].value()[0].to_bits(), (-small).to_bits());
    assert_eq!(t.observations()[0].value()[1].to_bits(), f64::MAX.to_bits());
}

#[test]
fn tst_bound_006_maximum_source_inventory() {
    let mut offered = Vec::new();
    for tick in 0..=360000 {
        if tick % 20 == 0 {
            offered.push(observation(
                Sensor::Position,
                tick / 20,
                tick,
                tick,
                [0.0; 2],
            ));
        }
        if tick % 5 == 0 {
            offered.push(observation(
                Sensor::Velocity,
                tick / 5,
                tick,
                tick,
                [0.0; 2],
            ));
        }
    }
    assert_eq!(offered.len(), 90002);
    let input = Trace::new(3600, 128, [None, None], offered).unwrap();
    assert_eq!(input.observations().len(), 90002);
    let last = input.observations().last().unwrap();
    assert_eq!(last.sample_tick(), 360000);
    assert_eq!(last.id().sensor(), Sensor::Velocity);
    assert_eq!(last.id().sequence(), 72000);
}

#[test]
fn tst_neg_001_model_rejects_nonfinite_and_outside_domain() {
    for field in [
        ModelField::PositionSigma,
        ModelField::VelocitySigma,
        ModelField::AccelerationDensity,
    ] {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 100.1] {
            let values = match field {
                ModelField::PositionSigma => [value, 1.0, 0.1],
                ModelField::VelocitySigma => [1.0, value, 0.1],
                ModelField::AccelerationDensity => [1.0, 1.0, value],
            };
            assert!(
                matches!(ReplayModel::new(values[0],values[1],values[2]),Err(DeliveryError::InvalidModel{field:f})if f==field)
            );
        }
    }
}

#[test]
fn tst_neg_002_payload_nonfinite_is_not_loss() {
    for axis in 0..2 {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut payload = [0.0; 2];
            payload[axis] = value;
            match Observation::new(Sensor::Velocity, 0, 0, 0, payload) {
                Err(DeliveryError::NonFinitePayload { id, axis: a }) => {
                    assert_eq!(a, axis);
                    assert_eq!(id.sensor(), Sensor::Velocity);
                    assert_eq!(id.sequence(), 0);
                }
                other => panic!("wrong payload outcome: {other:?}"),
            }
        }
    }
    for token in ["NaN", "inf", "-inf"] {
        let text = canonical_header(1) + &format!("1\t0\tvelocity\t0\t{token}\t0\n");
        match Trace::decode_tsv(text.as_bytes()) {
            Err(DeliveryError::NonFinitePayload { id, axis }) => {
                assert_eq!(axis, 0);
                assert_eq!(id.sensor(), Sensor::Velocity);
                assert_eq!(id.sequence(), 0);
            }
            other => panic!("invalid late payload was accepted: {other:?}"),
        }
    }
}

#[test]
fn tst_neg_003_sequence_schedule_and_horizon_are_checked() {
    for (sensor, sequence, sample) in [
        (Sensor::Position, 1, 5),
        (Sensor::Velocity, 2, 5),
        (Sensor::Velocity, u32::MAX, 0),
    ] {
        assert!(
            matches!(Observation::new(sensor,sequence,sample,sample,[0.0;2]),Err(DeliveryError::InvalidSequence{sample_tick,..})if sample_tick==sample)
        );
    }
    let extreme =
        Observation::new(Sensor::Velocity, u32::MAX / 5, u32::MAX, u32::MAX, [0.0; 2]).unwrap();
    assert!(matches!(
        Trace::new(3600, 128, [None, None], vec![extreme]),
        Err(DeliveryError::SampleBeyondHorizon { .. })
    ));
}

#[test]
fn tst_neg_004_causality_and_delay_excess() {
    assert!(matches!(
        Observation::new(Sensor::Velocity, 1, 5, 4, [0.0; 2]),
        Err(DeliveryError::ArrivalBeforeSample { .. })
    ));
    assert!(matches!(
        Observation::new(Sensor::Position, 0, 0, 501, [0.0; 2]),
        Err(DeliveryError::DelayExceeded {
            delay_ticks: 501,
            ..
        })
    ));
    for delays in [(501, 0), (0, 501)] {
        let expected = if delays.0 == 501 {
            Sensor::Position
        } else {
            Sensor::Velocity
        };
        assert!(
            matches!(DeliveryPlan::new(delays.0,delays.1,1,None,None),Err(DeliveryError::InvalidPlanDelay{sensor,delay_ticks:501})if sensor==expected)
        );
    }
}

#[test]
fn tst_neg_005_nonadjacent_duplicate_identity() {
    let offered = vec![
        observation(Sensor::Velocity, 0, 0, 0, [1.0; 2]),
        observation(Sensor::Velocity, 1, 5, 5, [2.0; 2]),
        observation(Sensor::Velocity, 0, 0, 500, [3.0; 2]),
    ];
    match Trace::new(5, 128, [None, None], offered) {
        Err(DeliveryError::DuplicateIdentity { id }) => {
            assert_eq!(id.sensor(), Sensor::Velocity);
            assert_eq!(id.sequence(), 0);
        }
        other => panic!("duplicate accepted: {other:?}"),
    }
    let text = canonical_header(3)
        + "0\t0\tvelocity\t0\t1\t1\n5\t5\tvelocity\t1\t2\t2\n500\t0\tvelocity\t0\t3\t3\n";
    assert!(matches!(
        Trace::decode_tsv(text.as_bytes()),
        Err(DeliveryError::DuplicateIdentity { .. })
    ));
}

#[test]
fn tst_neg_006_projection_layout_is_validated() {
    let source = run(&Config {
        duration_seconds: 1,
        outage: None,
        ..Config::default()
    })
    .unwrap()
    .samples;
    let plan = || DeliveryPlan::new(0, 0, 128, None, None).unwrap();
    assert!(matches!(
        Trace::from_samples(1, &source[..100], plan()),
        Err(DeliveryError::InvalidSourceSamples {
            index: 100,
            reason: SourceViolation::WrongCount
        })
    ));
    let mut changed = source.clone();
    changed[1].tick = 2;
    assert!(matches!(
        Trace::from_samples(1, &changed, plan()),
        Err(DeliveryError::InvalidSourceSamples {
            index: 1,
            reason: SourceViolation::UnexpectedTick
        })
    ));
    let mut changed = source;
    changed[1].velocity_measurement = Some([1.0; 2]);
    assert!(matches!(
        Trace::from_samples(1, &changed, plan()),
        Err(DeliveryError::InvalidSourceSamples {
            index: 1,
            reason: SourceViolation::OffScheduleMeasurement
        })
    ));
}

#[test]
fn tst_neg_007_encoding_and_newline_grammar() {
    let valid = golden_trace();
    for (bytes, reason) in [
        (vec![255], TsvViolation::InvalidEncoding),
        (
            [vec![239, 187, 191], valid.clone()].concat(),
            TsvViolation::NonLfOrBom,
        ),
        (
            String::from_utf8(valid.clone())
                .unwrap()
                .replace('\n', "\r\n")
                .into_bytes(),
            TsvViolation::NonLfOrBom,
        ),
        (
            valid[..valid.len() - 1].to_vec(),
            TsvViolation::MissingFinalNewline,
        ),
    ] {
        assert!(
            matches!(Trace::decode_tsv(&bytes),Err(DeliveryError::MalformedTsv{row:0,field:0,reason:r})if r==reason)
        );
    }
    let mut blank = canonical_header(0);
    blank.push('\n');
    assert!(matches!(
        Trace::decode_tsv(blank.as_bytes()),
        Err(DeliveryError::MalformedTsv {
            reason: TsvViolation::BlankLine,
            ..
        })
    ));
}

#[test]
fn tst_neg_008_schema_and_fixed_header_fields() {
    let valid = String::from_utf8(golden_trace()).unwrap();
    let version = valid.replacen(
        "fusion-lab-delivery-trace\t1",
        "fusion-lab-delivery-trace\t2",
        1,
    );
    assert!(matches!(
        Trace::decode_tsv(version.as_bytes()),
        Err(DeliveryError::UnsupportedSchema { version: 2 })
    ));
    for replacement in [
        valid.replacen("clock_ms\t10", "clock_ms\t20", 1),
        valid.replacen("policy\tOnTimeOnly", "policy\tApplyAtArrival", 1),
        valid.replacen(
            "position_assumed_sigma\t5",
            "position_assumed_sigma\t5\textra",
            1,
        ),
        valid.replacen("clock_kind\tvirtual_planned", "clock_kind\twall", 1),
    ] {
        assert!(matches!(
            Trace::decode_tsv(replacement.as_bytes()),
            Err(DeliveryError::MalformedTsv { .. })
        ));
    }
}

#[test]
fn tst_neg_009_noncanonical_numeric_tokens() {
    for (token, field, reason) in [
        ("00", 1, TsvViolation::NonCanonicalInteger),
        ("+0", 1, TsvViolation::NonCanonicalInteger),
        ("0.0", 1, TsvViolation::InvalidInteger),
        ("1.0", 5, TsvViolation::NonCanonicalFloat),
        ("+1", 5, TsvViolation::NonCanonicalFloat),
        ("1e0", 5, TsvViolation::NonCanonicalFloat),
        ("banana", 5, TsvViolation::InvalidFloat),
    ] {
        let mut row = ["0", "0", "velocity", "0", "1", "0.5"];
        row[field - 1] = token;
        let text = canonical_header(1) + &row.join("\t") + "\n";
        assert!(
            matches!(Trace::decode_tsv(text.as_bytes()),Err(DeliveryError::MalformedTsv{row:12,field:f,reason:r})if f==field && r==reason),
            "token={token}"
        );
    }
}

#[test]
fn tst_neg_010_row_count_and_column_count() {
    let row = "0\t0\tvelocity\t0\t1\t0.5\n";
    for text in [
        canonical_header(2) + row,
        canonical_header(0) + row,
        canonical_header(1) + "0\t0\tvelocity\t0\t1\n",
        canonical_header(1) + "0\t0\tvelocity\t0\t1\t0.5\textra\n",
    ] {
        assert!(matches!(
            Trace::decode_tsv(text.as_bytes()),
            Err(DeliveryError::MalformedTsv {
                reason: TsvViolation::WrongRowCount | TsvViolation::WrongFieldCount,
                ..
            })
        ));
    }
}

#[test]
fn tst_neg_011_import_rejects_order_inversion() {
    let text = canonical_header(2) + "5\t5\tvelocity\t1\t1\t1\n0\t0\tvelocity\t0\t2\t2\n";
    assert!(matches!(
        Trace::decode_tsv(text.as_bytes()),
        Err(DeliveryError::NonCanonicalOrder { row: 13 })
    ));
}

#[test]
fn tst_neg_012_input_byte_token_and_line_caps() {
    let oversized = vec![b'x'; 32 * 1024 * 1024 + 1];
    assert!(
        matches!(Trace::decode_tsv(&oversized),Err(DeliveryError::ArtifactTooLarge{kind:ArtifactKind::Trace,limit})if limit==32*1024*1024)
    );
    let text = canonical_header(1) + &format!("0\t0\tvelocity\t0\t{}\t0\n", "1".repeat(328));
    assert!(matches!(
        Trace::decode_tsv(text.as_bytes()),
        Err(DeliveryError::MalformedTsv {
            row: 12,
            field: 5,
            reason: TsvViolation::TokenTooLong
        })
    ));
    let line = "x".repeat(1025) + "\n";
    assert!(matches!(
        Trace::decode_tsv(line.as_bytes()),
        Err(DeliveryError::MalformedTsv {
            row: 1,
            field: 0,
            reason: TsvViolation::LineTooLong
        })
    ));
}

#[test]
fn tst_neg_013_cardinality_before_duplicate_allocation() {
    let item = observation(Sensor::Velocity, 0, 0, 0, [0.0; 2]);
    assert!(matches!(
        Trace::new(3600, 128, [None, None], vec![item; 90003]),
        Err(DeliveryError::TooManyObservations {
            count: 90003,
            limit: 90002
        })
    ));
}

#[test]
fn tst_prop_001_all_raw_permutations_are_equivalent() {
    let values = [
        observation(Sensor::Position, 0, 0, 10, [9.0; 2]),
        observation(Sensor::Velocity, 0, 0, 5, [1.0; 2]),
        observation(Sensor::Velocity, 1, 5, 5, [4.0, 2.0]),
        observation(Sensor::Velocity, 2, 10, 15, [8.0; 2]),
    ];
    let input = Trace::new(1, 1, [None, None], values.to_vec()).unwrap();
    let bytes = input.encode_tsv(&model()).unwrap();
    let baseline = replay_delivery(&model(), &input).unwrap();
    let mut count = 0;
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let mut indices = [a, b, c, d];
                    indices.sort_unstable();
                    if indices != [0, 1, 2, 3] {
                        continue;
                    }
                    let candidate = Trace::new(
                        1,
                        1,
                        [None, None],
                        vec![
                            values[a].clone(),
                            values[b].clone(),
                            values[c].clone(),
                            values[d].clone(),
                        ],
                    )
                    .unwrap();
                    assert_eq!(candidate.encode_tsv(&model()).unwrap(), bytes);
                    assert_bitwise(&replay_delivery(&model(), &candidate).unwrap(), &baseline);
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 24);
}

#[test]
fn tst_prop_002_finite_payload_roundtrip_preserves_bits() {
    let domain = [
        0.0,
        -0.0,
        f64::from_bits(1),
        -f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::MAX,
        f64::MIN,
        1.0,
        -1.0,
        0.1,
    ];
    let offered = domain
        .into_iter()
        .enumerate()
        .map(|(i, value)| {
            observation(
                Sensor::Velocity,
                i as u32,
                i as u32 * 5,
                i as u32 * 5,
                [value, -value],
            )
        })
        .collect();
    let input = trace(offered);
    let bytes = input.encode_tsv(&model()).unwrap();
    let (model2, again) = Trace::decode_tsv(&bytes).unwrap();
    assert_eq!(again.encode_tsv(&model2).unwrap(), bytes);
    for (a, b) in input.observations().iter().zip(again.observations()) {
        assert_eq!(a.value().map(f64::to_bits), b.value().map(f64::to_bits));
    }
}

#[test]
fn tst_prop_003_finite_extreme_late_inputs_have_no_numerical_effect() {
    let baseline = replay_delivery(
        &model(),
        &trace(vec![observation(Sensor::Position, 0, 0, 0, [10.0, 20.0])]),
    )
    .unwrap();
    let mut offered = vec![observation(Sensor::Position, 0, 0, 0, [10.0, 20.0])];
    for sequence in 0..=20 {
        let sample = sequence * 5;
        offered.push(observation(
            Sensor::Velocity,
            sequence,
            sample,
            sample + 1,
            [f64::MAX, f64::MIN],
        ));
    }
    let input = Trace::new(1, 1, [None, None], offered).unwrap();
    let result = replay_delivery(&model(), &input).unwrap();
    assert_eq!(result.snapshots().len(), baseline.snapshots().len());
    for (a, b) in result.snapshots().iter().zip(baseline.snapshots()) {
        assert_eq!(
            a.estimate().position.map(f64::to_bits),
            b.estimate().position.map(f64::to_bits)
        );
        assert_eq!(
            a.estimate().velocity.map(f64::to_bits),
            b.estimate().velocity.map(f64::to_bits)
        );
        assert_eq!(
            a.covariance().map(|row| row.map(f64::to_bits)),
            b.covariance().map(|row| row.map(f64::to_bits))
        );
        assert_eq!(
            a.held_position().map(f64::to_bits),
            b.held_position().map(f64::to_bits)
        );
        assert_eq!(
            a.integrated_position().map(f64::to_bits),
            b.integrated_position().map(f64::to_bits)
        );
    }
    assert!(
        result
            .ledger()
            .iter()
            .filter(|e| e.id().sensor() == Sensor::Velocity)
            .all(|e| e.outcome() != DeliveryOutcome::Accepted)
    );
}

#[test]
fn tst_prop_004_ledger_partitions_bounded_plan_grid() {
    let source = run(&Config {
        duration_seconds: 1,
        outage: None,
        ..Config::default()
    })
    .unwrap();
    for capacity in [1, 2, 128] {
        for delays in [(0, 0), (500, 1), (1, 500)] {
            for drops in [[None, None], [nz(1), None], [None, nz(1)], [nz(3), nz(4)]] {
                let input = Trace::from_samples(
                    1,
                    &source.samples,
                    DeliveryPlan::new(delays.0, delays.1, capacity, drops[0], drops[1]).unwrap(),
                )
                .unwrap();
                assert_eq!(input.observations().len(), 27);
                assert_eq!(input.drop_every(Sensor::Position), drops[0]);
                assert_eq!(input.drop_every(Sensor::Velocity), drops[1]);
                let result = replay_delivery(&model(), &input).unwrap();
                assert_eq!(result.ledger().len(), 27);
                let mut unique = Vec::new();
                let mut arrivals = Vec::new();
                for entry in result.ledger() {
                    let id = (entry.id().sensor(), entry.id().sequence());
                    assert!(!unique.contains(&id));
                    unique.push(id);
                    let schedule = match entry.id().sensor() {
                        Sensor::Position => drops[0],
                        Sensor::Velocity => drops[1],
                    };
                    let expected_scripted =
                        schedule.is_some_and(|n| (entry.id().sequence() + 1) % n.get() == 0);
                    assert_eq!(
                        entry.outcome() == DeliveryOutcome::ScriptedLoss,
                        expected_scripted,
                        "id={id:?}, drops={drops:?}"
                    );
                    match entry.outcome() {
                        DeliveryOutcome::QueueFullDrop => {
                            assert_eq!(entry.queue_occupancy(), Some(capacity as u16));
                            assert_eq!(entry.decision_tick(), entry.sample_tick());
                        }
                        DeliveryOutcome::ScriptedLoss => {
                            assert!(entry.queue_occupancy().is_none());
                            assert_eq!(entry.decision_tick(), entry.sample_tick());
                        }
                        DeliveryOutcome::Accepted | DeliveryOutcome::StaleDrop => {
                            assert!(entry.queue_occupancy().is_none());
                            assert_eq!(entry.decision_tick(), entry.planned_arrival_tick());
                            arrivals.push(id);
                        }
                        DeliveryOutcome::UndeliveredAtHorizon => {
                            assert!(entry.queue_occupancy().is_none());
                            assert_eq!(entry.decision_tick(), 100);
                        }
                    }
                }
                let actual = ids(result.arrival_order());
                assert_eq!(actual.len(), arrivals.len());
                for id in actual {
                    assert!(arrivals.contains(&id));
                }
            }
        }
    }
}

#[test]
fn tst_state_001_drain_bypass_and_source_arrival_ledgers() {
    let input = Trace::new(
        1,
        1,
        [None, None],
        vec![
            observation(Sensor::Position, 0, 0, 10, [10.0, 20.0]),
            observation(Sensor::Velocity, 0, 0, 5, [1.0; 2]),
            observation(Sensor::Velocity, 1, 5, 5, [4.0, 2.0]),
            observation(Sensor::Velocity, 2, 10, 15, [9.0; 2]),
        ],
    )
    .unwrap();
    let result = replay_delivery(&model(), &input).unwrap();
    assert_eq!(
        ledger(&result),
        vec![
            (Sensor::Position, 0, DeliveryOutcome::StaleDrop, 10, None),
            (
                Sensor::Velocity,
                0,
                DeliveryOutcome::QueueFullDrop,
                0,
                Some(1)
            ),
            (Sensor::Velocity, 1, DeliveryOutcome::Accepted, 5, None),
            (Sensor::Velocity, 2, DeliveryOutcome::StaleDrop, 15, None)
        ]
    );
    assert_eq!(
        ids(result.arrival_order()),
        vec![
            (Sensor::Velocity, 1),
            (Sensor::Position, 0),
            (Sensor::Velocity, 2)
        ]
    );
    let s = &result.snapshots()[15];
    for (a, e) in s.estimate().position.into_iter().zip([0.3, 0.15]) {
        close(a, e, 1e-12);
    }
    assert_eq!(s.estimate().velocity, [2.0, 1.0]);
    close(s.covariance()[0][0], 25.045, 1e-12);
    close(s.covariance()[0][2], 0.3, 1e-12);
    close(s.covariance()[2][2], 2.0, 1e-12);
    assert_eq!(s.held_position(), [0.0; 2]);
    for (a, e) in s.integrated_position().into_iter().zip([0.4, 0.2]) {
        close(a, e, 1e-12);
    }
    let times = result
        .ledger()
        .iter()
        .map(|e| (e.sample_tick(), e.planned_arrival_tick(), e.decision_tick()))
        .collect::<Vec<_>>();
    assert_eq!(times, vec![(0, 10, 10), (0, 5, 0), (5, 5, 5), (10, 15, 15)]);
}

#[test]
fn tst_state_002_scripted_loss_precedes_queue_and_horizon() {
    let input = Trace::new(
        1,
        1,
        [nz(6), None],
        vec![
            observation(Sensor::Velocity, 19, 95, 105, [1.0; 2]),
            observation(Sensor::Position, 5, 100, 101, [2.0; 2]),
            observation(Sensor::Velocity, 20, 100, 101, [3.0; 2]),
        ],
    )
    .unwrap();
    let result = replay_delivery(&model(), &input).unwrap();
    assert_eq!(
        ledger(&result),
        vec![
            (
                Sensor::Velocity,
                19,
                DeliveryOutcome::UndeliveredAtHorizon,
                100,
                None
            ),
            (
                Sensor::Position,
                5,
                DeliveryOutcome::ScriptedLoss,
                100,
                None
            ),
            (
                Sensor::Velocity,
                20,
                DeliveryOutcome::QueueFullDrop,
                100,
                Some(1)
            )
        ]
    );
    assert!(result.arrival_order().is_empty());
}

#[test]
fn tst_contract_001_exact_version_one_artifacts() {
    let input = golden_input();
    assert_eq!(input.encode_tsv(&model()).unwrap(), golden_trace());
    let (imported_model, imported) = Trace::decode_tsv(&golden_trace()).unwrap();
    assert_eq!(imported_model.assumed_position_sigma(), 5.0);
    assert_eq!(imported_model.assumed_velocity_sigma(), 2.0);
    assert_eq!(imported_model.acceleration_density(), 0.0);
    assert_eq!(
        imported.encode_tsv(&imported_model).unwrap(),
        golden_trace()
    );
    let result = replay_delivery(&imported_model, &imported).unwrap();
    assert_eq!(result.encode_ledger_tsv().unwrap(), golden_ledger());
    assert!(result.encode_ledger_tsv().unwrap().len() <= 16 * 1024 * 1024);
}

#[test]
fn tst_perf_001_valid_large_trace_fails_serialization_at_cap() {
    let mut offered = Vec::new();
    for tick in 0..=360000 {
        if tick % 20 == 0 {
            offered.push(observation(
                Sensor::Position,
                tick / 20,
                tick,
                tick,
                [f64::MAX; 2],
            ));
        }
        if tick % 5 == 0 {
            offered.push(observation(
                Sensor::Velocity,
                tick / 5,
                tick,
                tick,
                [f64::MAX; 2],
            ));
        }
    }
    let input = Trace::new(3600, 128, [None, None], offered).unwrap();
    assert_eq!(input.observations().len(), 90002);
    assert!(
        matches!(input.encode_tsv(&model()),Err(DeliveryError::ArtifactTooLarge{kind:ArtifactKind::Trace,limit})if limit==32*1024*1024)
    );
}

#[test]
fn tst_anti_001_named_seam_removed_before_implementation_acceptance() {
    let source = include_str!("../../../src/delivery.rs");
    assert!(
        !source.contains("OBSERVATION_DELIVERY_NOT_IMPLEMENTED"),
        "OBSERVATION_DELIVERY_NOT_IMPLEMENTED diagnostic seam must be removed"
    );
}

#[test]
fn tst_struct_001_readonly_surface_is_downstream_usable() {
    fn identity<T: Copy + Eq + std::hash::Hash>() {}
    identity::<ObservationId>();
    identity::<Sensor>();
    let _: fn(&Trace) -> &[Observation] = Trace::observations;
    let _: fn(&Trace, Sensor) -> Option<std::num::NonZeroU32> = Trace::drop_every;
    let _: fn(&ReplayResult) -> &[ReplaySnapshot] = ReplayResult::snapshots;
    let _: fn(&ReplayResult) -> &[LedgerEntry] = ReplayResult::ledger;
    let _: fn(&ReplaySnapshot) -> State = ReplaySnapshot::estimate;
    let _: fn(&ReplaySnapshot) -> &[[f64; 4]; 4] = ReplaySnapshot::covariance;
    let _: fn(&LedgerEntry) -> Option<u16> = LedgerEntry::queue_occupancy;
    type DecodeTrace = fn(&[u8]) -> Result<(ReplayModel, Trace), DeliveryError>;
    let _: DecodeTrace = Trace::decode_tsv;
    let _: fn(&ReplayModel, &Trace) -> Result<ReplayResult, DeliveryError> = replay_delivery;
    assert_ne!(Sensor::Position, Sensor::Velocity);
}

#[test]
fn tst_inv_001_replay_is_readonly_and_repeatable() {
    let input = golden_input();
    let model = model();
    let bytes = input.encode_tsv(&model).unwrap();
    let first = replay_delivery(&model, &input).unwrap();
    let second = replay_delivery(&model, &input).unwrap();
    assert_bitwise(&first, &second);
    assert_eq!(input.encode_tsv(&model).unwrap(), bytes);
    assert_eq!(model.assumed_position_sigma(), 5.0);
    assert_eq!(model.assumed_velocity_sigma(), 2.0);
    assert_eq!(model.acceleration_density(), 0.0);
}

#[test]
fn tst_neg_014_on_time_update_overflow_preserves_source_error() {
    let input = trace(vec![
        observation(Sensor::Velocity, 0, 0, 0, [f64::MAX, 0.0]),
        observation(Sensor::Velocity, 1, 5, 5, [f64::MIN, 0.0]),
    ]);
    let bytes = input.encode_tsv(&model()).unwrap();
    let error = replay_delivery(&model(), &input).unwrap_err();
    match &error {
        DeliveryError::NumericalFailure {
            tick,
            observation: Some(id),
            source,
        } => {
            assert_eq!(*tick, 5);
            assert_eq!(id.sensor(), Sensor::Velocity);
            assert_eq!(id.sequence(), 1);
            assert_eq!(*source, Error::NumericalFailure);
        }
        other => panic!("wrong update failure {other:?}"),
    };
    assert_eq!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<Error>(),
        Some(&Error::NumericalFailure)
    );
    assert_eq!(input.encode_tsv(&model()).unwrap(), bytes);
}

#[test]
fn tst_struct_002_external_record_update_is_rejected() {
    use std::{fs, process::Command};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let template = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("compile/consumer");
    let parent = std::env::temp_dir();
    let mut owned = None;
    for i in 0..100 {
        let candidate = parent.join(format!(
            "fusion-delivery-privacy-{}-{i}",
            std::process::id()
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => {
                owned = Some(candidate);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => panic!("temporary directory: {error}"),
        }
    }
    let owned = owned.expect("unique owned directory");
    fs::create_dir(owned.join("src")).unwrap();
    let manifest = fs::read_to_string(template.join("Cargo.toml")).unwrap();
    assert_eq!(manifest.matches("path = \"../../../..\"").count(), 1);
    let normalized = root.to_str().unwrap().replace(char::from(92), "/");
    let normalized = if let Some(unc) = normalized.strip_prefix("//?/UNC/") {
        format!("//{unc}")
    } else {
        normalized
            .strip_prefix("//?/")
            .unwrap_or(&normalized)
            .to_owned()
    };
    let canonical_root = normalized.replace('"', "\\\"");
    let manifest = manifest.replace(
        "path = \"../../../..\"",
        &format!("path = \"{canonical_root}\""),
    );
    fs::write(owned.join("Cargo.toml"), manifest).unwrap();
    let frozen_lock = fs::read(template.join("Cargo.lock")).unwrap();
    fs::write(owned.join("Cargo.lock"), &frozen_lock).unwrap();
    fs::copy(template.join("src/lib.rs"), owned.join("src/lib.rs")).unwrap();
    let toolchain = env!("RUSTUP_TOOLCHAIN");
    let build = || {
        let mut command = Command::new("cargo");
        command
            .env("RUSTUP_TOOLCHAIN", toolchain)
            .env("CARGO_TARGET_DIR", owned.join("target"))
            .current_dir(&owned)
            .args(["check", "--manifest-path"])
            .arg(owned.join("Cargo.toml"))
            .args(["--locked", "--offline"]);
        eprintln!("privacy current-root argv: {command:?}");
        command.output().unwrap()
    };
    let positive = build();
    let positive_error = String::from_utf8(positive.stderr).unwrap();
    eprintln!(
        "positive status={:?}\nstdout={}\nstderr={}",
        positive.status,
        String::from_utf8(positive.stdout).unwrap(),
        positive_error
    );
    if !positive.status.success() {
        let _ = fs::remove_dir_all(&owned);
        panic!("fresh current-root public API consumer failed: {positive_error}");
    }
    assert_eq!(fs::read(owned.join("Cargo.lock")).unwrap(), frozen_lock);
    let negative = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("compile/privacy.rs");
    fs::write(owned.join("src/lib.rs"), fs::read(negative).unwrap()).unwrap();
    let negative = build();
    let stderr = String::from_utf8(negative.stderr).unwrap();
    eprintln!(
        "negative status={:?}\nstdout={}\nstderr={}",
        negative.status,
        String::from_utf8(negative.stdout).unwrap(),
        stderr
    );
    assert_eq!(fs::read(owned.join("Cargo.lock")).unwrap(), frozen_lock);
    fs::remove_dir_all(&owned).unwrap();
    assert!(
        !negative.status.success(),
        "external record update unexpectedly compiled"
    );
    for name in [
        "ObservationId",
        "Observation",
        "ReplayModel",
        "DeliveryPlan",
        "Trace",
        "ReplayResult",
        "ReplaySnapshot",
        "LedgerEntry",
    ] {
        assert!(
            stderr.lines().any(|line| line.starts_with("error[E0451]:")
                && line
                    .split("of struct \x60")
                    .nth(1)
                    .and_then(|part| part.split('\x60').next())
                    .and_then(|part| part.rsplit("::").next())
                    == Some(name)
                && line.contains("private")),
            "missing exact private-field failure for {name}: {stderr}"
        );
    }
    assert_eq!(
        stderr
            .lines()
            .filter(|line| line.starts_with("error["))
            .count(),
        8,
        "unrelated compiler failure: {stderr}"
    );
}
