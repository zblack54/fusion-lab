//! Diagnostic compile seam for the separately reviewed history-replay design.
//! No correction behavior exists until the approved contract is implemented.
use crate::{Error, ObservationId, State};
fn not_implemented() -> ! {
    panic!("HISTORY_REPLAY_NOT_IMPLEMENTED")
}
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
    _model: &crate::ReplayModel,
    _trace: &crate::Trace,
    _plan: &HistoryPlan,
) -> Result<HistoryResult, HistoryError> {
    not_implemented()
}
