//! Research-only one-key runner. The Python executor freezes and bounds the study.
use fusion_lab::{Config, Experiment, Metrics, Outage, Scenario, run};
use nalgebra::{Matrix2, Vector2};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

const THRESHOLD: f64 = 5.991_464_547_107_979;

fn recovery_start(within: &[bool]) -> Option<usize> {
    (2000..=2900).find(|&start| within[start..=start + 100].iter().all(|&v| v))
}

fn write_samples(experiment: &Experiment, path: &Path, rounded: bool) -> std::io::Result<()> {
    let mut out = BufWriter::new(File::create(path)?);
    write!(
        out,
        "tick,time_s,true_x,true_y,true_vx,true_vy,estimated_x,estimated_y,estimated_vx,estimated_vy,position_x,position_y,velocity_x,velocity_y,hold_x,hold_y,integrated_x,integrated_y,position_outage"
    )?;
    for r in 0..4 {
        for c in 0..4 {
            write!(out, ",p{r}{c}")?;
        }
    }
    if !rounded {
        write!(out, ",source_covered,source_within_recovery")?;
    }
    writeln!(out)?;
    for s in &experiment.samples {
        write!(out, "{},{}", s.tick, f64::from(s.tick) / 100.0)?;
        let mut number = |value: f64| -> std::io::Result<()> {
            if rounded {
                write!(out, ",{value:.12}")
            } else {
                write!(out, ",{value}")
            }
        };
        for value in s
            .truth
            .position
            .into_iter()
            .chain(s.truth.velocity)
            .chain(s.estimate.position)
            .chain(s.estimate.velocity)
        {
            number(value)?;
        }
        // End the mutable borrow before writing absent observations directly.
        for measurement in [s.position_measurement, s.velocity_measurement] {
            if let Some(values) = measurement {
                for value in values {
                    if rounded {
                        write!(out, ",{value:.12}")?;
                    } else {
                        write!(out, ",{value}")?;
                    }
                }
            } else {
                write!(out, ",,")?;
            }
        }
        for value in s.held_position.into_iter().chain(s.integrated_position) {
            if rounded {
                write!(out, ",{value:.12}")?;
            } else {
                write!(out, ",{value}")?;
            }
        }
        write!(out, ",{}", s.position_outage)?;
        for value in s.covariance.into_iter().flatten() {
            if rounded {
                write!(out, ",{value:.12}")?;
            } else {
                write!(out, ",{value}")?;
            }
        }
        if !rounded {
            let e = Vector2::new(
                s.estimate.position[0] - s.truth.position[0],
                s.estimate.position[1] - s.truth.position[1],
            );
            let p = Matrix2::new(
                s.covariance[0][0],
                s.covariance[0][1],
                s.covariance[1][0],
                s.covariance[1][1],
            );
            let factor = p
                .cholesky()
                .ok_or_else(|| std::io::Error::other("invalid covariance"))?;
            write!(
                out,
                ",{},{}",
                e.dot(&factor.solve(&e)) <= THRESHOLD,
                e[0].hypot(e[1]) <= 1.0
            )?;
        }
        writeln!(out)?;
    }
    out.flush()
}

fn write_metrics(out: &mut impl Write, label: &str, m: Metrics) -> std::io::Result<()> {
    writeln!(
        out,
        "{label}\t{}\t{}\t{}\t{}\t{}",
        m.samples, m.filter_rmse, m.hold_rmse, m.integration_rmse, m.position_coverage
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 5 {
        return Err(
            "usage: robustness-run SEED straight|maneuver SCALE_INDEX BIAS_INDEX OUTPUT_STEM"
                .into(),
        );
    }
    let seed: u64 = args[0].parse()?;
    if !(1000..=1031).contains(&seed) {
        return Err("unexpected seed".into());
    }
    let scenario = match args[1].as_str() {
        "straight" => Scenario::Straight,
        "maneuver" => Scenario::Maneuver,
        _ => return Err("unexpected trajectory".into()),
    };
    let scale_index: usize = args[2].parse()?;
    let bias_index: usize = args[3].parse()?;
    let scale = *[0.5, 1.0, 2.0]
        .get(scale_index)
        .ok_or("unexpected scale index")?;
    let bias = *[[0.0, 0.0], [0.3, -0.15], [1.0, -0.5]]
        .get(bias_index)
        .ok_or("unexpected bias index")?;
    let config = Config {
        seed,
        scenario,
        duration_seconds: 30,
        position_sigma: 2.0,
        velocity_sigma: 0.3,
        assumed_position_sigma: Some(2.0 * scale),
        assumed_velocity_sigma: Some(0.3 * scale),
        velocity_bias: bias,
        acceleration_density: 0.1,
        outage: Some(Outage { start: 10, end: 20 }),
    };
    let experiment = run(&config)?;
    write_samples(&experiment, Path::new(&format!("{}.csv", args[4])), true)?;
    write_samples(
        &experiment,
        Path::new(&format!("{}.roundtrip.csv", args[4])),
        false,
    )?;
    let within: Vec<_> = experiment
        .samples
        .iter()
        .map(|s| {
            (s.estimate.position[0] - s.truth.position[0])
                .hypot(s.estimate.position[1] - s.truth.position[1])
                <= 1.0
        })
        .collect();
    let mut out = BufWriter::new(File::create(format!("{}.summary.tsv", args[4]))?);
    writeln!(out, "key\t{seed}\t{}\t{scale_index}\t{bias_index}", args[1])?;
    writeln!(
        out,
        "config\t30\t2\t0.3\t0.1\t10\t20\t{}\t{}\t{}\t{}",
        2.0 * scale,
        0.3 * scale,
        bias[0],
        bias[1]
    )?;
    write_metrics(&mut out, "overall", experiment.summary.overall)?;
    write_metrics(
        &mut out,
        "outage",
        experiment.summary.outage.ok_or("missing outage metrics")?,
    )?;
    match recovery_start(&within) {
        Some(tick) => writeln!(out, "recovery\t{tick}")?,
        None => writeln!(out, "recovery\tcensored")?,
    }
    out.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::recovery_start;
    #[test]
    fn recovery_requires_101_inclusive_samples_and_respects_last_start() {
        let mut values = vec![false; 3001];
        values[2000..2100].fill(true);
        assert_eq!(recovery_start(&values), None);
        values[2100] = true;
        assert_eq!(recovery_start(&values), Some(2000));
        values.fill(false);
        values[2900..=3000].fill(true);
        assert_eq!(recovery_start(&values), Some(2900));
        values[2900] = false;
        assert_eq!(recovery_start(&values), None);
    }
}
