//! Diagnostic compile seam for the unapproved observation-delivery contract.
//! No delivery behavior is implemented here. Every callable boundary reaches the
//! same named diagnostic; implementation may replace it only after approval.
use crate::{Error, Sample, State};
use std::num::NonZeroU32;

fn not_implemented<T>() -> T {
    panic!("OBSERVATION_DELIVERY_NOT_IMPLEMENTED")
}

/// Sensor identity; simultaneous position observations precede velocity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sensor {
    Position,
    Velocity,
}
/// Read-only observation identity, based on the scheduled sensor sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservationId {
    _not_implemented: (),
}
impl ObservationId {
    /// Sensor stream identity.
    pub fn sensor(self) -> Sensor {
        not_implemented()
    }
    /// Scheduled per-sensor sequence, starting at zero.
    pub fn sequence(self) -> u32 {
        not_implemented()
    }
}
/// A finite observation and its separate sample/planned-arrival timestamps.
#[derive(Debug, Clone)]
pub struct Observation {
    _not_implemented: (),
}
impl Observation {
    /// Validate payload, sequence, schedule, causality and maximum delay.
    pub fn new(
        _sensor: Sensor,
        _sequence: u32,
        _sample_tick: u32,
        _planned_arrival_tick: u32,
        _value: [f64; 2],
    ) -> Result<Self, DeliveryError> {
        not_implemented()
    }
    /// Stable sensor/scheduled-sequence identity.
    pub fn id(&self) -> ObservationId {
        not_implemented()
    }
    /// Sensor sampling time in 10 ms ticks.
    pub fn sample_tick(&self) -> u32 {
        not_implemented()
    }
    /// Planned virtual availability time, not a claim of actual receipt.
    pub fn planned_arrival_tick(&self) -> u32 {
        not_implemented()
    }
    /// Unmodified finite two-axis observation.
    pub fn value(&self) -> [f64; 2] {
        not_implemented()
    }
}
/// Requested filter assumptions, without truth, seed or source configuration.
#[derive(Debug)]
pub struct ReplayModel {
    _not_implemented: (),
}
impl ReplayModel {
    /// Validate requested sigmas and acceleration density in 0..=100.
    pub fn new(
        _assumed_position_sigma: f64,
        _assumed_velocity_sigma: f64,
        _acceleration_density: f64,
    ) -> Result<Self, DeliveryError> {
        not_implemented()
    }
    /// Requested position standard deviation in metres.
    pub fn assumed_position_sigma(&self) -> f64 {
        not_implemented()
    }
    /// Requested velocity standard deviation in metres per second.
    pub fn assumed_velocity_sigma(&self) -> f64 {
        not_implemented()
    }
    /// Continuous white acceleration density in m²/s³.
    pub fn acceleration_density(&self) -> f64 {
        not_implemented()
    }
}
/// Fixed projection delays and scheduled-sequence loss, with bounded waiting.
#[derive(Debug)]
pub struct DeliveryPlan {
    _not_implemented: (),
}
impl DeliveryPlan {
    /// Validate delays <=500 ticks and waiting capacity 1..=128.
    pub fn new(
        _position_delay_ticks: u32,
        _velocity_delay_ticks: u32,
        _pending_capacity: usize,
        _position_drop_every: Option<NonZeroU32>,
        _velocity_drop_every: Option<NonZeroU32>,
    ) -> Result<Self, DeliveryError> {
        not_implemented()
    }
}
/// Owned, validated observation-only replay input.
#[derive(Debug)]
pub struct Trace {
    _not_implemented: (),
}
impl Trace {
    /// Validate a raw input and normalize its canonical arrival ordering.
    pub fn new(
        _duration_seconds: u32,
        _pending_capacity: usize,
        _drop_every: [Option<NonZeroU32>; 2],
        _observations: Vec<Observation>,
    ) -> Result<Self, DeliveryError> {
        not_implemented()
    }
    /// Copy only sample tick and present measurements, ending the source borrow.
    pub fn from_samples(
        _duration_seconds: u32,
        _source: &[Sample],
        _plan: DeliveryPlan,
    ) -> Result<Self, DeliveryError> {
        not_implemented()
    }
    /// Decode the strict version-one canonical TSV within its byte/token caps.
    pub fn decode_tsv(_bytes: &[u8]) -> Result<(ReplayModel, Self), DeliveryError> {
        not_implemented()
    }
    /// Encode a complete canonical trace or return a typed resource error.
    pub fn encode_tsv(&self, _model: &ReplayModel) -> Result<Vec<u8>, DeliveryError> {
        not_implemented()
    }
    /// Duration in whole seconds.
    pub fn duration_seconds(&self) -> u32 {
        not_implemented()
    }
    /// Capacity of future observations waiting for arrival.
    pub fn pending_capacity(&self) -> usize {
        not_implemented()
    }
    /// Scripted loss interval in scheduled sequence units.
    pub fn drop_every(&self, _sensor: Sensor) -> Option<NonZeroU32> {
        not_implemented()
    }
    /// Observations in canonical planned-arrival order.
    pub fn observations(&self) -> &[Observation] {
        not_implemented()
    }
}
/// One final disposition per offered observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryOutcome {
    ScriptedLoss,
    QueueFullDrop,
    Accepted,
    StaleDrop,
    UndeliveredAtHorizon,
}
/// Read-only source-order decision, with planned time distinguished from receipt.
#[derive(Debug)]
pub struct LedgerEntry {
    _not_implemented: (),
}
impl LedgerEntry {
    /// Offered observation identity.
    pub fn id(&self) -> ObservationId {
        not_implemented()
    }
    /// Original sample tick.
    pub fn sample_tick(&self) -> u32 {
        not_implemented()
    }
    /// Original planned arrival tick.
    pub fn planned_arrival_tick(&self) -> u32 {
        not_implemented()
    }
    /// Source, arrival or horizon tick at which the outcome is decided.
    pub fn decision_tick(&self) -> u32 {
        not_implemented()
    }
    /// Waiting occupancy only for a queue-full decision; otherwise absent.
    pub fn queue_occupancy(&self) -> Option<u16> {
        not_implemented()
    }
    /// Closed terminal decision.
    pub fn outcome(&self) -> DeliveryOutcome {
        not_implemented()
    }
}
/// Post-update numerical state at one virtual tick, without truth fields.
#[derive(Debug)]
pub struct ReplaySnapshot {
    _not_implemented: (),
}
impl ReplaySnapshot {
    /// Numerical tick, starting at zero.
    pub fn tick(&self) -> u32 {
        not_implemented()
    }
    /// Estimated position and velocity only.
    pub fn estimate(&self) -> State {
        not_implemented()
    }
    /// Full `[px, py, vx, vy]` covariance.
    pub fn covariance(&self) -> &[[f64; 4]; 4] {
        not_implemented()
    }
    /// Last accepted position observation, initially the origin.
    pub fn held_position(&self) -> [f64; 2] {
        not_implemented()
    }
    /// Integrated accepted velocity, reset by accepted position observations.
    pub fn integrated_position(&self) -> [f64; 2] {
        not_implemented()
    }
}
/// Complete owned replay; no successful partial result exists.
#[derive(Debug)]
pub struct ReplayResult {
    _not_implemented: (),
}
impl ReplayResult {
    /// All ticks including zero and the final tick.
    pub fn snapshots(&self) -> &[ReplaySnapshot] {
        not_implemented()
    }
    /// One decision per offered observation in canonical source order.
    pub fn ledger(&self) -> &[LedgerEntry] {
        not_implemented()
    }
    /// Only observations actually considered as arrivals, in consumption order.
    pub fn arrival_order(&self) -> &[ObservationId] {
        not_implemented()
    }
    /// Encode the separate capped version-one source/arrival ledger.
    pub fn encode_ledger_tsv(&self) -> Result<Vec<u8>, DeliveryError> {
        not_implemented()
    }
}
/// Model input whose requested value is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelField {
    PositionSigma,
    VelocitySigma,
    AccelerationDensity,
}
/// Source projection layout failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceViolation {
    WrongCount,
    UnexpectedTick,
    OffScheduleMeasurement,
}
/// Strict TSV token/record violation, with a bounded diagnostic location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsvViolation {
    WrongHeader,
    WrongFieldCount,
    InvalidInteger,
    NonCanonicalInteger,
    InvalidFloat,
    NonCanonicalFloat,
    InvalidSensor,
    WrongRowCount,
    InvalidEncoding,
    NonLfOrBom,
    MissingFinalNewline,
    BlankLine,
    TokenTooLong,
    LineTooLong,
}
/// Encoded artifact whose fixed capacity would be exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Trace,
    Ledger,
}
/// Typed invalid-input, resource or numerical failure; never a successful drop.
#[derive(Debug)]
pub enum DeliveryError {
    InvalidDuration {
        value: u32,
    },
    InvalidPlanDelay {
        sensor: Sensor,
        delay_ticks: u32,
    },
    InvalidCapacity {
        value: usize,
    },
    InvalidModel {
        field: ModelField,
    },
    TooManyObservations {
        count: usize,
        limit: usize,
    },
    InvalidSourceSamples {
        index: usize,
        reason: SourceViolation,
    },
    SampleBeyondHorizon {
        id: ObservationId,
    },
    InvalidSequence {
        id: ObservationId,
        sample_tick: u32,
    },
    ArrivalBeforeSample {
        id: ObservationId,
    },
    DelayExceeded {
        id: ObservationId,
        delay_ticks: u32,
    },
    NonFinitePayload {
        id: ObservationId,
        axis: usize,
    },
    DuplicateIdentity {
        id: ObservationId,
    },
    NonCanonicalOrder {
        row: usize,
    },
    MalformedTsv {
        row: usize,
        field: usize,
        reason: TsvViolation,
    },
    UnsupportedSchema {
        version: u32,
    },
    ArtifactTooLarge {
        kind: ArtifactKind,
        limit: usize,
    },
    NumericalFailure {
        tick: u32,
        observation: Option<ObservationId>,
        source: Error,
    },
}
impl std::fmt::Display for DeliveryError {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        not_implemented()
    }
}
impl std::error::Error for DeliveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        not_implemented()
    }
}
/// Synchronously replay observation-only input using the fixed version-one prior.
pub fn replay_delivery(
    _model: &ReplayModel,
    _trace: &Trace,
) -> Result<ReplayResult, DeliveryError> {
    not_implemented()
}
