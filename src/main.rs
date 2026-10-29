use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};
use fusion_lab::{Config, Experiment, Metrics, Outage, Scenario, run};

#[derive(Clone, Copy, ValueEnum)]
enum Trajectory {
    Straight,
    Maneuver,
}

#[derive(Parser)]
#[command(version, about = "Run a reproducible 2D sensor fusion experiment")]
struct Args {
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Run length in whole seconds (1..=3600).
    #[arg(long, default_value_t = 30)]
    duration: u32,
    #[arg(long, value_enum, default_value_t = Trajectory::Maneuver)]
    scenario: Trajectory,
    /// True position noise sigma in metres; also assumed unless overridden.
    #[arg(long, default_value_t = 1.0)]
    position_sigma: f64,
    /// True velocity noise sigma in m/s; also assumed unless overridden.
    #[arg(long, default_value_t = 0.15)]
    velocity_sigma: f64,
    /// Filter-assumed position sigma in metres (omitted: follow true noise).
    #[arg(long)]
    assumed_position_sigma: Option<f64>,
    /// Filter-assumed velocity sigma in m/s (omitted: follow true noise).
    #[arg(long)]
    assumed_velocity_sigma: Option<f64>,
    /// Fixed world-frame x velocity sensor bias in m/s (-100..=100).
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    velocity_bias_x: f64,
    /// Fixed world-frame y velocity sensor bias in m/s (-100..=100).
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    velocity_bias_y: f64,
    /// Linear x velocity sensor drift rate in m/s^2 (-1..=1).
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    velocity_drift_rate_x: f64,
    /// Linear y velocity sensor drift rate in m/s^2 (-1..=1).
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    velocity_drift_rate_y: f64,
    /// Continuous white acceleration density in m^2/s^3.
    #[arg(long, default_value_t = 0.1)]
    acceleration_density: f64,
    /// First unavailable position sample time in whole seconds.
    #[arg(long, default_value_t = 10)]
    outage_start: u32,
    /// Position resumes at this whole-second timestamp.
    #[arg(long, default_value_t = 20)]
    outage_end: u32,
    /// Disable the position outage entirely.
    #[arg(long)]
    no_outage: bool,
    /// CSV destination; existing CSV and companion .txt report are overwritten.
    #[arg(long, default_value = "results/run.csv")]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Config {
        seed: args.seed,
        duration_seconds: args.duration,
        scenario: match args.scenario {
            Trajectory::Straight => Scenario::Straight,
            Trajectory::Maneuver => Scenario::Maneuver,
        },
        position_sigma: args.position_sigma,
        velocity_sigma: args.velocity_sigma,
        assumed_position_sigma: args.assumed_position_sigma,
        assumed_velocity_sigma: args.assumed_velocity_sigma,
        velocity_bias: [args.velocity_bias_x, args.velocity_bias_y],
        velocity_drift_rate: [args.velocity_drift_rate_x, args.velocity_drift_rate_y],
        acceleration_density: args.acceleration_density,
        outage: (!args.no_outage).then_some(Outage {
            start: args.outage_start,
            end: args.outage_end,
        }),
    };
    if args.output.extension().and_then(|s| s.to_str()) != Some("csv") {
        return Err("--output must have a .csv extension".into());
    }
    let experiment = run(&config)?;
    if let Some(parent) = args.output.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let mut csv = BufWriter::new(File::create(&args.output)?);
    write_csv(&experiment, &mut csv)?;
    csv.flush()?;
    let report = report(&experiment);
    std::fs::write(args.output.with_extension("txt"), &report)?;
    print!("{report}");
    println!("CSV: {}", args.output.display());
    Ok(())
}

fn write_csv(experiment: &Experiment, writer: &mut impl Write) -> std::io::Result<()> {
    write!(
        writer,
        "tick,time_s,true_x,true_y,true_vx,true_vy,estimated_x,estimated_y,estimated_vx,estimated_vy,position_x,position_y,velocity_x,velocity_y,hold_x,hold_y,integrated_x,integrated_y,position_outage"
    )?;
    for row in 0..4 {
        for col in 0..4 {
            write!(writer, ",p{row}{col}")?;
        }
    }
    writeln!(writer)?;
    for sample in &experiment.samples {
        write!(
            writer,
            "{},{:.2}",
            sample.tick,
            f64::from(sample.tick) / 100.0
        )?;
        for value in sample
            .truth
            .position
            .into_iter()
            .chain(sample.truth.velocity)
            .chain(sample.estimate.position)
            .chain(sample.estimate.velocity)
        {
            write!(writer, ",{value:.12}")?;
        }
        for measurement in [sample.position_measurement, sample.velocity_measurement] {
            if let Some(value) = measurement {
                write!(writer, ",{:.12},{:.12}", value[0], value[1])?;
            } else {
                write!(writer, ",,")?;
            }
        }
        for value in sample
            .held_position
            .into_iter()
            .chain(sample.integrated_position)
        {
            write!(writer, ",{value:.12}")?;
        }
        write!(writer, ",{}", sample.position_outage)?;
        for value in sample.covariance.into_iter().flatten() {
            write!(writer, ",{value:.12}")?;
        }
        writeln!(writer)?;
    }
    Ok(())
}

fn report(experiment: &Experiment) -> String {
    let mut report = format!(
        "Fusion Lab {}\nConfiguration: {:?}\n",
        env!("CARGO_PKG_VERSION"),
        experiment.config
    );
    let config = &experiment.config;
    let assumed_position = config
        .assumed_position_sigma
        .unwrap_or(config.position_sigma);
    let assumed_velocity = config
        .assumed_velocity_sigma
        .unwrap_or(config.velocity_sigma);
    let position_source = if config.assumed_position_sigma.is_some() {
        "override"
    } else {
        "follows true"
    };
    let velocity_source = if config.assumed_velocity_sigma.is_some() {
        "override"
    } else {
        "follows true"
    };
    report.push_str(&format!(
        "Measurement noise (per-axis sigma): true position={:.12} m; assumed position={assumed_position:.12} m ({position_source}); true velocity={:.12} m/s; assumed velocity={assumed_velocity:.12} m/s ({velocity_source})\n",
        config.position_sigma, config.velocity_sigma
    ));
    report.push_str(&format!(
        "Initial velocity sensor bias (world frame): x={:.12} m/s; y={:.12} m/s; deterministic offset, not Gaussian sigma or filter correction\n",
        config.velocity_bias[0], config.velocity_bias[1]
    ));
    let final_bias: [f64; 2] = std::array::from_fn(|axis| {
        if config.velocity_drift_rate[axis] == 0.0 {
            config.velocity_bias[axis]
        } else {
            config.velocity_bias[axis]
                + config.velocity_drift_rate[axis] * f64::from(config.duration_seconds)
        }
    });
    report.push_str(&format!(
        "Linear velocity sensor drift (world frame): rate x={:.12} m/s^2; y={:.12} m/s^2; duration={} s; final offset x={:.12} m/s; y={:.12} m/s; bias(t)=initial_bias+rate*t_seconds, not physical acceleration or filter correction\n",
        config.velocity_drift_rate[0], config.velocity_drift_rate[1],
        config.duration_seconds, final_bias[0], final_bias[1]
    ));
    let requested_position = assumed_position.powi(2);
    let requested_velocity = assumed_velocity.powi(2);
    report.push_str(&format!(
        "Measurement covariance R (per-axis): requested position={requested_position:.12e} m^2; effective position={:.12e} m^2; requested velocity={requested_velocity:.12e} (m/s)^2; effective velocity={:.12e} (m/s)^2 (variance floor=1e-12)\n",
        requested_position.max(1e-12), requested_velocity.max(1e-12)
    ));
    report.push_str(&metric_report("Overall", experiment.summary.overall));
    if let Some(metrics) = experiment.summary.outage {
        report.push_str(&metric_report("Position outage", metrics));
    }
    report
        .push_str("Coverage is descriptive; this run does not establish covariance calibration.\n");
    report
}

fn metric_report(label: &str, metrics: Metrics) -> String {
    format!(
        "{label} ({} samples):\n  position RMSE: filter={:.4} m, hold={:.4} m, integration={:.4} m\n  nominal 95% position ellipse coverage: {:.2}%\n",
        metrics.samples,
        metrics.filter_rmse,
        metrics.hold_rmse,
        metrics.integration_rmse,
        metrics.position_coverage * 100.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_has_full_covariance_and_empty_unsampled_measurements()
    -> Result<(), Box<dyn std::error::Error>> {
        let experiment = run(&Config {
            duration_seconds: 1,
            outage: None,
            ..Config::default()
        })?;
        let mut bytes = Vec::new();
        write_csv(&experiment, &mut bytes)?;
        let text = String::from_utf8(bytes)?;
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 102);
        assert!(lines[0].ends_with("p30,p31,p32,p33"));
        assert!(lines.iter().all(|line| line.split(',').count() == 35));
        assert_eq!(lines[2].split(',').nth(10), Some(""));
        assert_eq!(lines[2].split(',').nth(12), Some(""));
        assert_eq!(lines[2].split(',').nth(1), Some("0.01"));
        Ok(())
    }
}
