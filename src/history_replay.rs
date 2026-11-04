//! Bounded observation-only correction with frozen virtual admission decisions.
use crate::{Error, ObservationId, Sensor, State, estimation::KalmanFilter};
use std::{cmp::Reverse, collections::BinaryHeap};
/// Validated bounded-history request.
#[derive(Debug)]
pub struct HistoryPlan {
    window_ticks: u32,
}
impl HistoryPlan {
    /// Validate a history window in 0..=500 ticks.
    pub fn new(window_ticks: u32) -> Result<Self, HistoryError> {
        if window_ticks > 500 {
            return Err(HistoryError::InvalidWindow {
                value: window_ticks,
            });
        }
        Ok(Self { window_ticks })
    }
    /// Requested inclusive age window in 10 ms ticks.
    pub fn window_ticks(&self) -> u32 {
        self.window_ticks
    }
}
/// Explicit per-observation final decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryOutcome {
    ScriptedLoss,
    QueueFullDrop,
    AcceptedOnTime,
    CorrectedLate,
    TooOldDrop,
    UndeliveredAtHorizon,
}
/// Defensive invariant violated by an implementation, not an overload drop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryInvariant {
    MissingAnchor,
    WorkBound,
    StorageBound,
}
/// Typed failure with no successful result prefix.
#[derive(Debug)]
pub enum HistoryError {
    /// Invalid requested window.
    InvalidWindow { value: u32 },
    /// Actual prediction/update failure, preserving the core source.
    NumericalFailure {
        arrival_tick: u32,
        replay_tick: u32,
        observation: Option<ObservationId>,
        source: Error,
    },
    /// A fixed work/history invariant failed.
    InvariantFailure { tick: u32, kind: HistoryInvariant },
}
impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "history replay failed: {self:?}")
    }
}
impl std::error::Error for HistoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NumericalFailure { source, .. } => Some(source),
            _ => None,
        }
    }
}
/// Immutable online state at its original arrival-time reporting tick.
#[derive(Debug)]
pub struct HistorySnapshot {
    tick: u32,
    estimate: State,
    covariance: [[f64; 4]; 4],
    held_position: [f64; 2],
    integrated_position: [f64; 2],
}
impl HistorySnapshot {
    /// Original reporting tick.
    pub fn tick(&self) -> u32 {
        self.tick
    }
    /// Corrected current estimate at that reporting tick.
    pub fn estimate(&self) -> State {
        self.estimate
    }
    /// Full state covariance.
    pub fn covariance(&self) -> [[f64; 4]; 4] {
        self.covariance
    }
    /// Sample-time corrected position hold baseline.
    pub fn held_position(&self) -> [f64; 2] {
        self.held_position
    }
    /// Sample-time corrected raw-velocity integration baseline.
    pub fn integrated_position(&self) -> [f64; 2] {
        self.integrated_position
    }
}
/// Source-order final decision and immutable original timestamps.
#[derive(Debug)]
pub struct HistoryLedgerEntry {
    id: ObservationId,
    sample_tick: u32,
    planned_arrival_tick: u32,
    decision_tick: u32,
    queue_occupancy: Option<u16>,
    outcome: HistoryOutcome,
}
impl HistoryLedgerEntry {
    /// Stable offered identity.
    pub fn id(&self) -> ObservationId {
        self.id
    }
    /// Original sample tick.
    pub fn sample_tick(&self) -> u32 {
        self.sample_tick
    }
    /// Original virtual availability tick.
    pub fn planned_arrival_tick(&self) -> u32 {
        self.planned_arrival_tick
    }
    /// Actual decision tick or final horizon.
    pub fn decision_tick(&self) -> u32 {
        self.decision_tick
    }
    /// Occupancy only for queue-full decisions.
    pub fn queue_occupancy(&self) -> Option<u16> {
        self.queue_occupancy
    }
    /// Explicit final policy disposition.
    pub fn outcome(&self) -> HistoryOutcome {
        self.outcome
    }
}
/// Actual per-tick numerical work and history storage diagnostics.
#[derive(Debug)]
pub struct HistoryWork {
    tick: u32,
    restored_sample_tick: Option<u32>,
    prediction_calls: usize,
    measurement_calls: usize,
    committed_checkpoints: usize,
    committed_observations: usize,
    peak_live_checkpoints: usize,
    peak_live_history_observations: usize,
    reserved_checkpoint_slots: usize,
    reserved_observation_slots: usize,
}
impl HistoryWork {
    /// Reporting tick.
    pub fn tick(&self) -> u32 {
        self.tick
    }
    /// Earliest replayed sample, or None for ordinary progress.
    pub fn restored_sample_tick(&self) -> Option<u32> {
        self.restored_sample_tick
    }
    /// Total actual prediction calls, including any discarded work.
    pub fn prediction_calls(&self) -> usize {
        self.prediction_calls
    }
    /// Total actual measurement calls, including any discarded work.
    pub fn measurement_calls(&self) -> usize {
        self.measurement_calls
    }
    /// Committed logical checkpoint objects.
    pub fn committed_checkpoints(&self) -> usize {
        self.committed_checkpoints
    }
    /// Committed logical measurement records.
    pub fn committed_observations(&self) -> usize {
        self.committed_observations
    }
    /// Peak original-plus-candidate logical checkpoint objects.
    pub fn peak_live_checkpoints(&self) -> usize {
        self.peak_live_checkpoints
    }
    /// Peak original-plus-candidate logical history observation records.
    pub fn peak_live_history_observations(&self) -> usize {
        self.peak_live_history_observations
    }
    /// Peak actual total allocated checkpoint slot capacity during this tick, separate from live objects.
    pub fn reserved_checkpoint_slots(&self) -> usize {
        self.reserved_checkpoint_slots
    }
    /// Peak actual total allocated history measurement slot capacity during this tick.
    pub fn reserved_observation_slots(&self) -> usize {
        self.reserved_observation_slots
    }
}
/// Complete immutable result; no partial result escapes on failure.
#[derive(Debug)]
pub struct HistoryResult {
    snapshots: Vec<HistorySnapshot>,
    ledger: Vec<HistoryLedgerEntry>,
    arrival_order: Vec<ObservationId>,
    work: Vec<HistoryWork>,
}
impl HistoryResult {
    /// Online snapshots, never retrospectively rewritten.
    pub fn snapshots(&self) -> &[HistorySnapshot] {
        &self.snapshots
    }
    /// One decision per offered observation in source order.
    pub fn ledger(&self) -> &[HistoryLedgerEntry] {
        &self.ledger
    }
    /// Actual delivered identity order, including too-old arrivals.
    pub fn arrival_order(&self) -> &[ObservationId] {
        &self.arrival_order
    }
    /// One actual-work record per reporting tick.
    pub fn work(&self) -> &[HistoryWork] {
        &self.work
    }
}
/// Replay sample-time corrections without truth, Config, I/O or future payload access.
pub fn replay_history(
    model: &crate::ReplayModel,
    trace: &crate::Trace,
    plan: &HistoryPlan,
) -> Result<HistoryResult, HistoryError> {
    let final_tick = trace.duration_seconds() * 100;
    let window = plan.window_ticks();
    let checkpoint_limit = window as usize + 2;
    let observations = trace.observations();
    // Sorting indices reads only scheduling metadata, never numerical payloads.
    let mut source_order: Vec<usize> = (0..observations.len()).collect();
    source_order.sort_unstable_by_key(|&index| sample_key(&observations[index]));
    let mut ledger: Vec<_> = source_order
        .iter()
        .map(|&index| {
            let o = &observations[index];
            HistoryLedgerEntry {
                id: o.id(),
                sample_tick: o.sample_tick(),
                planned_arrival_tick: o.planned_arrival_tick(),
                decision_tick: final_tick,
                queue_occupancy: None,
                outcome: HistoryOutcome::UndeliveredAtHorizon,
            }
        })
        .collect();
    let mut waiting = BinaryHeap::with_capacity(trace.pending_capacity());
    let mut snapshots = Vec::with_capacity(final_tick as usize + 1);
    let mut work = Vec::with_capacity(final_tick as usize + 1);
    let mut arrival_order = Vec::with_capacity(observations.len());
    // Both buffers have fixed domain capacities. Their sum is reported even
    // when the scratch buffer is empty; live objects are counted separately.
    let mut checkpoints = Vec::with_capacity(checkpoint_limit);
    let mut candidate_checkpoints = Vec::with_capacity(checkpoint_limit);
    let mut history: Vec<usize> = Vec::with_capacity(127);
    let mut candidate_history: Vec<usize> = Vec::with_capacity(127);
    checkpoints.push(Working::new(model).checkpoint(None));
    let mut next_source = 0;
    for tick in 0..=final_tick {
        candidate_checkpoints.clear();
        candidate_history.clear();
        let earliest_sample = tick.saturating_sub(window);
        // Prune before inserting this tick's arrived observations. No numerical
        // payload access occurs here, including for too-old or rejected records.
        candidate_history.extend(
            history
                .iter()
                .copied()
                .filter(|&index| observations[index].sample_tick() >= earliest_sample),
        );
        let mut restored_sample_tick = None;
        while let Some(&Reverse((arrival, _, _, _, ordinal))) = waiting.peek() {
            if arrival > tick {
                break;
            }
            waiting.pop();
            let index = source_order[ordinal];
            record_arrival(
                Arrival {
                    index,
                    ordinal,
                    tick,
                    window,
                },
                observations,
                &mut ledger,
                &mut arrival_order,
                &mut candidate_history,
                &mut restored_sample_tick,
            )?;
        }
        while next_source < source_order.len() {
            let index = source_order[next_source];
            let o = &observations[index];
            if o.sample_tick() > tick {
                break;
            }
            let id = o.id();
            if trace
                .drop_every(id.sensor())
                .is_some_and(|n| (id.sequence() + 1).is_multiple_of(n.get()))
            {
                ledger[next_source].decision_tick = tick;
                ledger[next_source].outcome = HistoryOutcome::ScriptedLoss;
            } else if o.planned_arrival_tick() == tick {
                record_arrival(
                    Arrival {
                        index,
                        ordinal: next_source,
                        tick,
                        window,
                    },
                    observations,
                    &mut ledger,
                    &mut arrival_order,
                    &mut candidate_history,
                    &mut restored_sample_tick,
                )?;
            } else if waiting.len() == trace.pending_capacity() {
                ledger[next_source].decision_tick = tick;
                ledger[next_source].outcome = HistoryOutcome::QueueFullDrop;
                ledger[next_source].queue_occupancy = Some(trace.pending_capacity() as u16);
            } else {
                let (sample, sensor, sequence) = sample_key(o);
                waiting.push(Reverse((
                    o.planned_arrival_tick(),
                    sample,
                    sensor,
                    sequence,
                    next_source,
                )));
            }
            next_source += 1;
        }
        candidate_history.sort_unstable_by_key(|&index| sample_key(&observations[index]));
        // Branch selection precedes every prediction, integration and update.
        let start = restored_sample_tick.unwrap_or(tick);
        let anchor_tick = start.checked_sub(1);
        let anchor = checkpoints.iter().find(|c| c.tick == anchor_tick).ok_or(
            HistoryError::InvariantFailure {
                tick,
                kind: HistoryInvariant::MissingAnchor,
            },
        )?;
        // Keep the original pre-zero prior until sample zero is no longer
        // eligible; otherwise the earliest retained object is end(t-W-1).
        candidate_checkpoints.extend(checkpoints.iter().copied().filter(|c| {
            c.tick < Some(start)
                && match tick.checked_sub(window + 1) {
                    None => true,
                    Some(minimum) => c.tick.is_some_and(|t| t >= minimum),
                }
        }));
        let mut current = Working::restore(model, anchor);
        let mut prediction_calls = 0;
        let mut measurement_calls = 0;
        let mut next_measurement =
            candidate_history.partition_point(|&index| observations[index].sample_tick() < start);
        for replay_tick in start..=tick {
            if replay_tick > 0 {
                prediction_calls += 1;
                current
                    .filter
                    .predict(0.01)
                    .map_err(|source| HistoryError::NumericalFailure {
                        arrival_tick: tick,
                        replay_tick,
                        observation: None,
                        source,
                    })?;
                for axis in 0..2 {
                    current.integrated_position[axis] += current.last_velocity[axis] * 0.01;
                }
            }
            while let Some(&index) = candidate_history.get(next_measurement) {
                let o = &observations[index];
                if o.sample_tick() > replay_tick {
                    break;
                }
                let velocity = o.id().sensor() == Sensor::Velocity;
                let sigma = if velocity {
                    model.assumed_velocity_sigma()
                } else {
                    model.assumed_position_sigma()
                };
                // Only arrived, admitted, eligible indices can enter history.
                let value = o.value();
                measurement_calls += 1;
                current
                    .filter
                    .update(value, sigma.powi(2), velocity)
                    .map_err(|source| HistoryError::NumericalFailure {
                        arrival_tick: tick,
                        replay_tick,
                        observation: Some(o.id()),
                        source,
                    })?;
                if velocity {
                    current.last_velocity = value;
                } else {
                    current.held_position = value;
                    current.integrated_position = value;
                }
                next_measurement += 1;
            }
            if candidate_checkpoints.len() == checkpoint_limit {
                return Err(HistoryError::InvariantFailure {
                    tick,
                    kind: HistoryInvariant::StorageBound,
                });
            }
            candidate_checkpoints.push(current.checkpoint(Some(replay_tick)));
        }
        if prediction_calls > window as usize + 1 || measurement_calls > 127 {
            return Err(HistoryError::InvariantFailure {
                tick,
                kind: HistoryInvariant::WorkBound,
            });
        }
        let record = HistoryWork {
            tick,
            restored_sample_tick,
            prediction_calls,
            measurement_calls,
            committed_checkpoints: candidate_checkpoints.len(),
            committed_observations: candidate_history.len(),
            peak_live_checkpoints: checkpoints.len() + candidate_checkpoints.len(),
            peak_live_history_observations: history.len() + candidate_history.len(),
            reserved_checkpoint_slots: checkpoints.capacity() + candidate_checkpoints.capacity(),
            reserved_observation_slots: history.capacity() + candidate_history.capacity(),
        };
        // Commit only after the entire numerical batch and bounds checks pass.
        std::mem::swap(&mut checkpoints, &mut candidate_checkpoints);
        std::mem::swap(&mut history, &mut candidate_history);
        candidate_checkpoints.clear();
        candidate_history.clear();
        snapshots.push(current.snapshot(tick));
        work.push(record);
    }
    Ok(HistoryResult {
        snapshots,
        ledger,
        arrival_order,
        work,
    })
}

fn sample_key(o: &crate::Observation) -> (u32, u8, u32) {
    (
        o.sample_tick(),
        match o.id().sensor() {
            Sensor::Position => 0,
            Sensor::Velocity => 1,
        },
        o.id().sequence(),
    )
}

struct Arrival {
    index: usize,
    ordinal: usize,
    tick: u32,
    window: u32,
}
// Scheduling decisions and numerical history insertion share one arrival path.
fn record_arrival(
    arrival: Arrival,
    observations: &[crate::Observation],
    ledger: &mut [HistoryLedgerEntry],
    arrival_order: &mut Vec<ObservationId>,
    history: &mut Vec<usize>,
    restored: &mut Option<u32>,
) -> Result<(), HistoryError> {
    let Arrival {
        index,
        ordinal,
        tick,
        window,
    } = arrival;
    let o = &observations[index];
    let entry = &mut ledger[ordinal];
    entry.decision_tick = tick;
    arrival_order.push(o.id());
    let age = tick - o.sample_tick();
    if age > window {
        entry.outcome = HistoryOutcome::TooOldDrop;
        return Ok(());
    }
    entry.outcome = if age == 0 {
        HistoryOutcome::AcceptedOnTime
    } else {
        HistoryOutcome::CorrectedLate
    };
    if age > 0 {
        *restored = Some(restored.map_or(o.sample_tick(), |old| old.min(o.sample_tick())));
    }
    if history.len() == 127 {
        return Err(HistoryError::InvariantFailure {
            tick,
            kind: HistoryInvariant::StorageBound,
        });
    }
    history.push(index);
    Ok(())
}

#[derive(Clone, Copy)]
struct Checkpoint {
    tick: Option<u32>,
    state: nalgebra::Vector4<f64>,
    covariance: nalgebra::Matrix4<f64>,
    held_position: [f64; 2],
    integrated_position: [f64; 2],
    last_velocity: [f64; 2],
}
struct Working {
    filter: KalmanFilter,
    held_position: [f64; 2],
    integrated_position: [f64; 2],
    last_velocity: [f64; 2],
}
impl Working {
    fn new(model: &crate::ReplayModel) -> Self {
        Self {
            filter: KalmanFilter::new(model.acceleration_density()),
            held_position: [0.0; 2],
            integrated_position: [0.0; 2],
            last_velocity: [0.0; 2],
        }
    }
    fn restore(model: &crate::ReplayModel, checkpoint: &Checkpoint) -> Self {
        let mut result = Self::new(model);
        result.filter.state = checkpoint.state;
        result.filter.covariance = checkpoint.covariance;
        result.held_position = checkpoint.held_position;
        result.integrated_position = checkpoint.integrated_position;
        result.last_velocity = checkpoint.last_velocity;
        result
    }
    fn checkpoint(&self, tick: Option<u32>) -> Checkpoint {
        Checkpoint {
            tick,
            state: self.filter.state,
            covariance: self.filter.covariance,
            held_position: self.held_position,
            integrated_position: self.integrated_position,
            last_velocity: self.last_velocity,
        }
    }
    fn snapshot(&self, tick: u32) -> HistorySnapshot {
        HistorySnapshot {
            tick,
            estimate: State {
                position: [self.filter.state[0], self.filter.state[1]],
                velocity: [self.filter.state[2], self.filter.state[3]],
            },
            covariance: std::array::from_fn(|row| {
                std::array::from_fn(|col| self.filter.covariance[(row, col)])
            }),
            held_position: self.held_position,
            integrated_position: self.integrated_position,
        }
    }
}
