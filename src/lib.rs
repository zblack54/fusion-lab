//! Reproducible sensor fusion experiments with an explicit virtual clock.
//!
//! ```
//! use fusion_lab::{Config, run};
//! let experiment = run(&Config::default())?;
//! assert_eq!(experiment.samples.len(), 3001);
//! # Ok::<(), fusion_lab::Error>(())
//! ```

mod delivery;
mod estimation;
mod evaluation;
mod sensors;
mod simulation;

pub use delivery::{
    ArtifactKind, DeliveryError, DeliveryOutcome, DeliveryPlan, LedgerEntry, ModelField,
    Observation, ObservationId, ReplayModel, ReplayResult, ReplaySnapshot, Sensor, SourceViolation,
    Trace, TsvViolation, replay_delivery,
};
pub use evaluation::{Metrics, Summary};
pub use simulation::{Config, Experiment, Outage, Sample, Scenario, State, run};

/// Invalid configuration or a numerical failure. Runs return no partial result.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A named configuration field violates the documented input limits.
    InvalidConfig(&'static str),
    /// A covariance cannot be factored, or the calculation became non-finite.
    NumericalFailure,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfig(field) => write!(f, "invalid configuration: {field}"),
            Self::NumericalFailure => {
                write!(f, "non-finite state or non-positive-definite covariance")
            }
        }
    }
}

impl std::error::Error for Error {}
