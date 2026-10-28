use crate::{Error, Summary, estimation::KalmanFilter, evaluation, sensors::Sensors};

const DT: f64 = 0.01;
const TICKS_PER_SECOND: u32 = 100;

/// Scripted truth dynamics. Both start at the origin with velocity [1, 0.5] m/s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    /// Fixed velocity throughout the experiment.
    Straight,
    /// Accelerate between 5–8 s, turn between 15–18 s, then coast.
    Maneuver,
}

/// Position beacon outage in seconds, with a half-open interval [start, end).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outage {
    /// First unavailable sample time, in whole seconds.
    pub start: u32,
    /// First available sample time after the outage, in whole seconds.
    pub end: u32,
}

/// Experiment inputs, validated by [`run`] before allocating or sampling.
///
/// The optional assumed-noise fields default to following true sensor noise.
/// In 0.1, complete external struct literals must include these new fields;
/// literals using `..Config::default()` retain the matched-noise defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Run length: 1–3600 whole seconds, inclusive.
    pub duration_seconds: u32,
    /// Seed for independent ChaCha8 position and velocity streams.
    pub seed: u64,
    /// Scripted trajectory.
    pub scenario: Scenario,
    /// True per-axis position noise standard deviation, 0–100 m.
    pub position_sigma: f64,
    /// True per-axis velocity noise standard deviation, 0–100 m/s.
    pub velocity_sigma: f64,
    /// Filter-assumed position sigma, 0–100 m; `None` follows true position noise.
    pub assumed_position_sigma: Option<f64>,
    /// Filter-assumed velocity sigma, 0–100 m/s; `None` follows true velocity noise.
    pub assumed_velocity_sigma: Option<f64>,
    /// Filter's continuous white acceleration spectral density, 0–100 m²/s³.
    pub acceleration_density: f64,
    /// Optional outage; must satisfy 0 <= start < end <= duration.
    pub outage: Option<Outage>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            duration_seconds: 30,
            seed: 42,
            scenario: Scenario::Maneuver,
            position_sigma: 1.0,
            velocity_sigma: 0.15,
            assumed_position_sigma: None,
            assumed_velocity_sigma: None,
            acceleration_density: 0.1,
            outage: Some(Outage { start: 10, end: 20 }),
        }
    }
}

impl Config {
    fn validate(&self) -> Result<(), Error> {
        if !(1..=3600).contains(&self.duration_seconds) {
            return Err(Error::InvalidConfig("duration_seconds (1..=3600)"));
        }
        for (name, value) in [
            ("position_sigma", self.position_sigma),
            ("velocity_sigma", self.velocity_sigma),
            ("acceleration_density", self.acceleration_density),
        ] {
            if !value.is_finite() || !(0.0..=100.0).contains(&value) {
                return Err(Error::InvalidConfig(name));
            }
        }
        for (name, value) in [
            ("assumed_position_sigma", self.assumed_position_sigma),
            ("assumed_velocity_sigma", self.assumed_velocity_sigma),
        ] {
            if let Some(value) = value
                && (!value.is_finite() || !(0.0..=100.0).contains(&value))
            {
                return Err(Error::InvalidConfig(name));
            }
        }
        if let Some(outage) = self.outage
            && (outage.start >= outage.end || outage.end > self.duration_seconds)
        {
            return Err(Error::InvalidConfig("outage"));
        }
        Ok(())
    }

    fn is_outage(&self, tick: u32) -> bool {
        self.outage
            .is_some_and(|o| tick >= o.start * TICKS_PER_SECOND && tick < o.end * TICKS_PER_SECOND)
    }
}

/// World-frame Cartesian position in metres and velocity in metres per second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    /// [x, y] position.
    pub position: [f64; 2],
    /// [vx, vy] velocity.
    pub velocity: [f64; 2],
}

/// One post-update snapshot at an integer 10 ms tick.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// Virtual time in 10 ms ticks, starting at zero.
    pub tick: u32,
    /// Ground truth, supplied only to evaluation and recording.
    pub truth: State,
    /// Filter state after all measurements at this tick.
    pub estimate: State,
    /// Full covariance ordered [px, py, vx, vy]; first index is row.
    pub covariance: [[f64; 4]; 4],
    /// Present only on position sampling ticks outside the outage.
    pub position_measurement: Option<[f64; 2]>,
    /// Present only on velocity sampling ticks.
    pub velocity_measurement: Option<[f64; 2]>,
    /// Last observed position, or the origin before any position observation.
    pub held_position: [f64; 2],
    /// Integrated measured velocity, reset on every position observation.
    pub integrated_position: [f64; 2],
    /// Whether the position beacon is unavailable at this tick.
    pub position_outage: bool,
}

/// Complete bounded in-memory run and its evaluation report.
#[derive(Debug, Clone, PartialEq)]
pub struct Experiment {
    /// Inputs retained for experiment provenance.
    pub config: Config,
    /// Snapshots at 100 Hz, including time zero and the final time.
    pub samples: Vec<Sample>,
    /// Overall and outage-only evaluation.
    pub summary: Summary,
}

/// Run an offline experiment without clocks, threads, or external I/O.
///
/// Position samples arrive at 5 Hz and velocity samples at 20 Hz, with zero
/// delivery delay. At simultaneous times position updates precede velocity
/// updates. No partial experiment is returned on failure.
///
/// # Errors
/// Returns [`Error::InvalidConfig`] for invalid inputs or
/// [`Error::NumericalFailure`] if a covariance cannot be factored.
pub fn run(config: &Config) -> Result<Experiment, Error> {
    config.validate()?;
    let assumed_position_sigma = config
        .assumed_position_sigma
        .unwrap_or(config.position_sigma);
    let assumed_velocity_sigma = config
        .assumed_velocity_sigma
        .unwrap_or(config.velocity_sigma);
    let last_tick = config.duration_seconds * TICKS_PER_SECOND;
    let mut samples = Vec::with_capacity(last_tick as usize + 1);
    let mut truth = State {
        position: [0.0; 2],
        velocity: [1.0, 0.5],
    };
    let mut filter = KalmanFilter::new(config.acceleration_density);
    let mut sensors = Sensors::new(config.seed);
    let mut held_position = [0.0; 2];
    let mut integrated_position = [0.0; 2];
    let mut last_velocity = [0.0; 2];

    for tick in 0..=last_tick {
        if tick > 0 {
            advance_truth(&mut truth, config.scenario, tick - 1);
            filter.predict(DT)?;
            for axis in 0..2 {
                integrated_position[axis] += last_velocity[axis] * DT;
            }
        }
        let position_outage = config.is_outage(tick);
        let position_measurement = if tick % 20 == 0 {
            // Draw even during outages: a fault cannot perturb future noise.
            let measurement = sensors.position(truth.position, config.position_sigma);
            (!position_outage).then_some(measurement)
        } else {
            None
        };
        let velocity_measurement = if tick % 5 == 0 {
            Some(sensors.velocity(truth.velocity, config.velocity_sigma))
        } else {
            None
        };

        if let Some(value) = position_measurement {
            filter.update(value, assumed_position_sigma.powi(2), false)?;
            held_position = value;
            integrated_position = value;
        }
        if let Some(value) = velocity_measurement {
            filter.update(value, assumed_velocity_sigma.powi(2), true)?;
            last_velocity = value;
        }
        samples.push(Sample {
            tick,
            truth,
            estimate: State {
                position: [filter.state[0], filter.state[1]],
                velocity: [filter.state[2], filter.state[3]],
            },
            covariance: std::array::from_fn(|row| {
                std::array::from_fn(|col| filter.covariance[(row, col)])
            }),
            position_measurement,
            velocity_measurement,
            held_position,
            integrated_position,
            position_outage,
        });
    }
    let summary = evaluation::summarize(&samples)?;
    Ok(Experiment {
        config: config.clone(),
        samples,
        summary,
    })
}

fn advance_truth(truth: &mut State, scenario: Scenario, interval_tick: u32) {
    let acceleration = match (scenario, interval_tick) {
        (Scenario::Maneuver, 500..800) => [0.4, 0.0],
        (Scenario::Maneuver, 1500..1800) => [-0.3, 0.4],
        _ => [0.0, 0.0],
    };
    for (axis, a) in acceleration.into_iter().enumerate() {
        truth.position[axis] += truth.velocity[axis] * DT + 0.5 * a * DT * DT;
        truth.velocity[axis] += a * DT;
    }
}
