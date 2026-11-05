//! Differential-drive pose and wheel observation model.

/// Planar position in metres and heading in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub x: f64,
    pub y: f64,
    pub heading: f64,
}

/// Signed left and right wheel travel in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelTravel {
    pub left: f64,
    pub right: f64,
}

/// Wheel geometry, sampling schedule and noise configuration.
#[derive(Debug, Clone, Copy)]
pub struct WheelConfig {
    pub track_width: f64,
    pub start_tick: u32,
    pub interval_ticks: u32,
    pub seed: u64,
    pub sigma: [f64; 2],
}

/// Wheel observations over one interval, without the truth pose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelObservation {
    pub sequence: u32,
    pub start_tick: u32,
    pub end_tick: u32,
    pub travel: WheelTravel,
}

/// End poses and their separately recorded wheel observations.
#[derive(Debug, PartialEq)]
pub struct WheelRun {
    pub poses: Vec<Pose>,
    pub observations: Vec<WheelObservation>,
}

/// A wheel model input or result violates its supported domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelError {
    Pose,
    TrackWidth,
    Travel,
    Schedule,
    Noise,
}

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, StandardNormal};
use std::f64::consts::{PI, TAU};

fn validate_pose(pose: Pose) -> Result<(), WheelError> {
    if !(-1e9..=1e9).contains(&pose.x)
        || !(-1e9..=1e9).contains(&pose.y)
        || !(-PI..=PI).contains(&pose.heading)
    {
        return Err(WheelError::Pose);
    }
    Ok(())
}

fn validate_width(width: f64) -> Result<(), WheelError> {
    if !(0.01..=100.0).contains(&width) {
        return Err(WheelError::TrackWidth);
    }
    Ok(())
}

fn canonical_heading(angle: f64) -> f64 {
    let angle = angle % TAU;
    if angle >= PI {
        angle - TAU
    } else if angle < -PI {
        angle + TAU
    } else {
        angle
    }
}

/// Integrate one pair of signed wheel travels using constant body twist.
pub fn step(pose: Pose, track_width: f64, travel: WheelTravel) -> Result<Pose, WheelError> {
    validate_pose(pose)?;
    validate_width(track_width)?;
    if !(-100.0..=100.0).contains(&travel.left) || !(-100.0..=100.0).contains(&travel.right) {
        return Err(WheelError::Travel);
    }
    let center = (travel.right + travel.left) / 2.0;
    let turn = (travel.right - travel.left) / track_width;
    let half = turn / 2.0;
    let sinc = if half == 0.0 { 1.0 } else { half.sin() / half };
    let (sin, cos) = (pose.heading + half).sin_cos();
    let displacement = center * sinc;
    let end = Pose {
        x: pose.x + displacement * cos,
        y: pose.y + displacement * sin,
        heading: canonical_heading(pose.heading + turn),
    };
    validate_pose(end)?;
    Ok(end)
}

fn noisy(ideal: f64, sigma: f64, rng: &mut ChaCha8Rng) -> f64 {
    if sigma == 0.0 {
        return ideal;
    }
    let draw: f64 = StandardNormal.sample(rng);
    ideal + sigma * draw
}

/// Simulate ideal post-interval poses and separate noisy wheel observations.
pub fn simulate(
    config: WheelConfig,
    mut pose: Pose,
    inputs: &[WheelTravel],
) -> Result<WheelRun, WheelError> {
    if !(1..=10_000).contains(&config.interval_ticks)
        || inputs.len() > 10_000
        || config
            .start_tick
            .checked_add(config.interval_ticks * inputs.len() as u32)
            .is_none()
    {
        return Err(WheelError::Schedule);
    }
    if config
        .sigma
        .iter()
        .any(|sigma| !(0.0..=10.0).contains(sigma))
    {
        return Err(WheelError::Noise);
    }
    validate_pose(pose)?;
    validate_width(config.track_width)?;

    let mut left = ChaCha8Rng::seed_from_u64(config.seed);
    let mut right = left.clone();
    right.set_stream(1);
    let mut poses = Vec::with_capacity(inputs.len());
    let mut observations = Vec::with_capacity(inputs.len());
    let mut start_tick = config.start_tick;
    for (sequence, &ideal) in inputs.iter().enumerate() {
        pose = step(pose, config.track_width, ideal)?;
        let end_tick = start_tick + config.interval_ticks;
        poses.push(pose);
        observations.push(WheelObservation {
            sequence: sequence as u32,
            start_tick,
            end_tick,
            travel: WheelTravel {
                left: noisy(ideal.left, config.sigma[0], &mut left),
                right: noisy(ideal.right, config.sigma[1], &mut right),
            },
        });
        start_tick = end_tick;
    }
    Ok(WheelRun {
        poses,
        observations,
    })
}
