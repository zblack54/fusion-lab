//! Validated observation-only inputs for deterministic delivery replay.
//! Replay and codec boundaries remain diagnostic until their ordered slices.
use crate::{Error, Sample, State};
use std::{collections::HashSet, num::NonZeroU32};

const MAX_OBSERVATIONS: usize = 90_002;
const MAX_DELAY_TICKS: u32 = 500;

fn not_implemented<T>() -> T {
    panic!("OBSERVATION_DELIVERY_NOT_IMPLEMENTED")
}

/// Sensor identity; simultaneous position observations precede velocity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sensor {
    Position,
    Velocity,
}
impl Sensor {
    fn period(self) -> u32 {
        match self {
            Self::Position => 20,
            Self::Velocity => 5,
        }
    }
    fn index(self) -> usize {
        match self {
            Self::Position => 0,
            Self::Velocity => 1,
        }
    }
}
/// Read-only observation identity, based on the scheduled sensor sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObservationId {
    sensor: Sensor,
    sequence: u32,
}
impl ObservationId {
    /// Sensor stream identity.
    pub fn sensor(self) -> Sensor {
        self.sensor
    }
    /// Scheduled per-sensor sequence, starting at zero.
    pub fn sequence(self) -> u32 {
        self.sequence
    }
}
/// A finite observation and its separate sample/planned-arrival timestamps.
#[derive(Debug, Clone)]
pub struct Observation {
    id: ObservationId,
    sample_tick: u32,
    planned_arrival_tick: u32,
    value: [f64; 2],
}
impl Observation {
    /// Validate payload, sequence, schedule, causality and maximum delay.
    pub fn new(
        sensor: Sensor,
        sequence: u32,
        sample_tick: u32,
        planned_arrival_tick: u32,
        value: [f64; 2],
    ) -> Result<Self, DeliveryError> {
        let id = ObservationId { sensor, sequence };
        for (axis, component) in value.iter().enumerate() {
            if !component.is_finite() {
                return Err(DeliveryError::NonFinitePayload { id, axis });
            }
        }
        if sequence.checked_mul(sensor.period()) != Some(sample_tick) {
            return Err(DeliveryError::InvalidSequence { id, sample_tick });
        }
        let delay_ticks = planned_arrival_tick
            .checked_sub(sample_tick)
            .ok_or(DeliveryError::ArrivalBeforeSample { id })?;
        if delay_ticks > MAX_DELAY_TICKS {
            return Err(DeliveryError::DelayExceeded { id, delay_ticks });
        }
        Ok(Self {
            id,
            sample_tick,
            planned_arrival_tick,
            value,
        })
    }
    /// Stable sensor/scheduled-sequence identity.
    pub fn id(&self) -> ObservationId {
        self.id
    }
    /// Sensor sampling time in 10 ms ticks.
    pub fn sample_tick(&self) -> u32 {
        self.sample_tick
    }
    /// Planned virtual availability time, not a claim of actual receipt.
    pub fn planned_arrival_tick(&self) -> u32 {
        self.planned_arrival_tick
    }
    /// Unmodified finite two-axis observation.
    pub fn value(&self) -> [f64; 2] {
        self.value
    }
}
/// Requested filter assumptions, without truth, seed or source configuration.
#[derive(Debug)]
pub struct ReplayModel {
    assumed_position_sigma: f64,
    assumed_velocity_sigma: f64,
    acceleration_density: f64,
}
impl ReplayModel {
    /// Validate requested sigmas and acceleration density in 0..=100.
    pub fn new(
        assumed_position_sigma: f64,
        assumed_velocity_sigma: f64,
        acceleration_density: f64,
    ) -> Result<Self, DeliveryError> {
        for (field, value) in [
            (ModelField::PositionSigma, assumed_position_sigma),
            (ModelField::VelocitySigma, assumed_velocity_sigma),
            (ModelField::AccelerationDensity, acceleration_density),
        ] {
            if !value.is_finite() || !(0.0..=100.0).contains(&value) {
                return Err(DeliveryError::InvalidModel { field });
            }
        }
        Ok(Self {
            assumed_position_sigma,
            assumed_velocity_sigma,
            acceleration_density,
        })
    }
    /// Requested position standard deviation in metres.
    pub fn assumed_position_sigma(&self) -> f64 {
        self.assumed_position_sigma
    }
    /// Requested velocity standard deviation in metres per second.
    pub fn assumed_velocity_sigma(&self) -> f64 {
        self.assumed_velocity_sigma
    }
    /// Continuous white acceleration density in m²/s³.
    pub fn acceleration_density(&self) -> f64 {
        self.acceleration_density
    }
}
fn validate_duration(duration_seconds: u32) -> Result<u32, DeliveryError> {
    if !(1..=3600).contains(&duration_seconds) {
        return Err(DeliveryError::InvalidDuration {
            value: duration_seconds,
        });
    }
    // The validated 3600-second bound makes this product representable.
    Ok(duration_seconds * 100)
}
fn validate_capacity(value: usize) -> Result<(), DeliveryError> {
    if !(1..=128).contains(&value) {
        return Err(DeliveryError::InvalidCapacity { value });
    }
    Ok(())
}
/// Fixed projection delays and scheduled-sequence loss, with bounded waiting.
#[derive(Debug)]
pub struct DeliveryPlan {
    delays: [u32; 2],
    pending_capacity: usize,
    drop_every: [Option<NonZeroU32>; 2],
}
impl DeliveryPlan {
    /// Validate delays <=500 ticks and waiting capacity 1..=128.
    pub fn new(
        position_delay_ticks: u32,
        velocity_delay_ticks: u32,
        pending_capacity: usize,
        position_drop_every: Option<NonZeroU32>,
        velocity_drop_every: Option<NonZeroU32>,
    ) -> Result<Self, DeliveryError> {
        for (sensor, delay_ticks) in [
            (Sensor::Position, position_delay_ticks),
            (Sensor::Velocity, velocity_delay_ticks),
        ] {
            if delay_ticks > MAX_DELAY_TICKS {
                return Err(DeliveryError::InvalidPlanDelay {
                    sensor,
                    delay_ticks,
                });
            }
        }
        validate_capacity(pending_capacity)?;
        Ok(Self {
            delays: [position_delay_ticks, velocity_delay_ticks],
            pending_capacity,
            drop_every: [position_drop_every, velocity_drop_every],
        })
    }
}
/// Owned, validated observation-only replay input.
#[derive(Debug)]
pub struct Trace {
    duration_seconds: u32,
    pending_capacity: usize,
    drop_every: [Option<NonZeroU32>; 2],
    observations: Vec<Observation>,
}
impl Trace {
    /// Validate a raw input and normalize its canonical arrival ordering.
    pub fn new(
        duration_seconds: u32,
        pending_capacity: usize,
        drop_every: [Option<NonZeroU32>; 2],
        mut observations: Vec<Observation>,
    ) -> Result<Self, DeliveryError> {
        let final_tick = validate_duration(duration_seconds)?;
        validate_capacity(pending_capacity)?;
        if observations.len() > MAX_OBSERVATIONS {
            return Err(DeliveryError::TooManyObservations {
                count: observations.len(),
                limit: MAX_OBSERVATIONS,
            });
        }
        let mut identities = HashSet::with_capacity(observations.len());
        for observation in &observations {
            if observation.sample_tick > final_tick {
                return Err(DeliveryError::SampleBeyondHorizon { id: observation.id });
            }
            if !identities.insert(observation.id) {
                return Err(DeliveryError::DuplicateIdentity { id: observation.id });
            }
        }
        observations.sort_unstable_by_key(|o| {
            (
                o.planned_arrival_tick,
                o.sample_tick,
                o.id.sensor.index(),
                o.id.sequence,
            )
        });
        Ok(Self {
            duration_seconds,
            pending_capacity,
            drop_every,
            observations,
        })
    }
    /// Copy only sample tick and present measurements, ending the source borrow.
    pub fn from_samples(
        duration_seconds: u32,
        source: &[Sample],
        plan: DeliveryPlan,
    ) -> Result<Self, DeliveryError> {
        let final_tick = validate_duration(duration_seconds)?;
        if source.len() != final_tick as usize + 1 {
            return Err(DeliveryError::InvalidSourceSamples {
                index: source.len(),
                reason: SourceViolation::WrongCount,
            });
        }
        // First validate the dense source layout before allocating its projection.
        for (index, sample) in source.iter().enumerate() {
            if sample.tick as usize != index {
                return Err(DeliveryError::InvalidSourceSamples {
                    index,
                    reason: SourceViolation::UnexpectedTick,
                });
            }
            if (sample.position_measurement.is_some() && sample.tick % 20 != 0)
                || (sample.velocity_measurement.is_some() && sample.tick % 5 != 0)
            {
                return Err(DeliveryError::InvalidSourceSamples {
                    index,
                    reason: SourceViolation::OffScheduleMeasurement,
                });
            }
        }
        let mut observations =
            Vec::with_capacity(final_tick as usize / 20 + final_tick as usize / 5 + 2);
        for sample in source {
            for (sensor, value) in [
                (Sensor::Position, sample.position_measurement),
                (Sensor::Velocity, sample.velocity_measurement),
            ] {
                if let Some(value) = value {
                    // A validated source tick <=360000 and delay <=500 cannot overflow.
                    observations.push(Observation::new(
                        sensor,
                        sample.tick / sensor.period(),
                        sample.tick,
                        sample.tick + plan.delays[sensor.index()],
                        value,
                    )?);
                }
            }
        }
        Self::new(
            duration_seconds,
            plan.pending_capacity,
            plan.drop_every,
            observations,
        )
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
        self.duration_seconds
    }
    /// Capacity of future observations waiting for arrival.
    pub fn pending_capacity(&self) -> usize {
        self.pending_capacity
    }
    /// Scripted loss interval in scheduled sequence units.
    pub fn drop_every(&self, sensor: Sensor) -> Option<NonZeroU32> {
        self.drop_every[sensor.index()]
    }
    /// Observations in canonical planned-arrival order.
    pub fn observations(&self) -> &[Observation] {
        &self.observations
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
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "observation delivery failed: {self:?}")
    }
}
impl std::error::Error for DeliveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NumericalFailure { source, .. } => Some(source),
            _ => None,
        }
    }
}
/// Synchronously replay observation-only input using the fixed version-one prior.
pub fn replay_delivery(
    _model: &ReplayModel,
    _trace: &Trace,
) -> Result<ReplayResult, DeliveryError> {
    not_implemented()
}
