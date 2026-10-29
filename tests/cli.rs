use std::process::Command;

#[test]
fn invalid_output_is_rejected_before_simulation_validation() {
    // Invalid duration would fail inside run(). Seeing the output error instead
    // establishes that destination validation happens before entering run().
    let output = Command::new(env!("CARGO_BIN_EXE_fusion-lab"))
        .args(["--duration", "0", "--no-outage", "--output", "invalid.txt"])
        .output()
        .expect("launch CLI");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 error");
    assert!(
        stderr.contains("--output must have a .csv extension"),
        "{stderr}"
    );
    assert!(!stderr.contains("InvalidConfig"), "{stderr}");
}

struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let name = format!(
            "fusion-noise-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn invalid_assumed_noise_creates_no_output_parent() {
    let root = Temp::new();
    for field in ["assumed-position-sigma", "assumed-velocity-sigma"] {
        for value in ["NaN", "inf", "-1", "101"] {
            let output_path = root.0.join(field).join(value).join("run.csv");
            let output = Command::new(env!("CARGO_BIN_EXE_fusion-lab"))
                .args([
                    "--duration",
                    "1",
                    "--no-outage",
                    &format!("--{field}={value}"),
                    "--output",
                ])
                .arg(&output_path)
                .output()
                .unwrap();
            assert!(!output.status.success());
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains("InvalidConfig"), "{stderr}");
            assert!(!output_path.parent().unwrap().exists());
        }
    }
}

#[test]
fn reports_distinguish_true_assumed_and_effective_zero_variance() {
    let root = Temp::new();
    for (name, flags, expected) in [
        (
            "position",
            vec!["--assumed-position-sigma", "1"],
            vec![
                "true position=2.000000000000 m",
                "assumed position=1.000000000000 m (override)",
                "assumed velocity=0.300000000000 m/s (follows true)",
            ],
        ),
        (
            "velocity",
            vec!["--assumed-velocity-sigma", "0.6"],
            vec![
                "assumed position=2.000000000000 m (follows true)",
                "assumed velocity=0.600000000000 m/s (override)",
            ],
        ),
        (
            "zero",
            vec![
                "--assumed-position-sigma",
                "0",
                "--assumed-velocity-sigma",
                "0",
            ],
            vec![
                "requested position=0.000000000000e0 m^2; effective position=1.000000000000e-12 m^2",
                "requested velocity=0.000000000000e0 (m/s)^2; effective velocity=1.000000000000e-12 (m/s)^2",
            ],
        ),
    ] {
        let path = root.0.join(format!("{name}.csv"));
        let output = Command::new(env!("CARGO_BIN_EXE_fusion-lab"))
            .args([
                "--duration",
                "1",
                "--no-outage",
                "--position-sigma",
                "2",
                "--velocity-sigma",
                "0.3",
            ])
            .args(flags)
            .arg("--output")
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report = std::fs::read_to_string(path.with_extension("txt")).unwrap();
        for fragment in expected {
            assert!(report.contains(fragment), "missing {fragment}: {report}");
        }
        assert_eq!(
            std::fs::read_to_string(path)
                .unwrap()
                .lines()
                .next()
                .unwrap()
                .split(',')
                .count(),
            35
        );
    }
}

#[test]
fn invalid_velocity_bias_creates_no_output_parent() {
    let root = Temp::new();
    for field in ["velocity-bias-x", "velocity-bias-y"] {
        for value in ["NaN", "inf", "-inf", "-100.1", "100.1"] {
            let destination = root.0.join(field).join(value).join("run.csv");
            let output = Command::new(env!("CARGO_BIN_EXE_fusion-lab"))
                .args([
                    "--duration",
                    "1",
                    "--no-outage",
                    &format!("--{field}"),
                    value,
                    "--output",
                ])
                .arg(&destination)
                .output()
                .unwrap();
            assert!(!output.status.success());
            assert!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .contains("InvalidConfig")
            );
            assert!(!destination.parent().unwrap().exists());
        }
    }
}

#[test]
fn signed_bias_cli_reports_units_and_preserves_csv_shape() {
    let root = Temp::new();
    let destination = root.0.join("bias.csv");
    let output = Command::new(env!("CARGO_BIN_EXE_fusion-lab"))
        .args([
            "--duration",
            "1",
            "--no-outage",
            "--velocity-bias-x",
            "2",
            "--velocity-bias-y",
            "-1",
            "--output",
        ])
        .arg(&destination)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = std::fs::read_to_string(destination.with_extension("txt")).unwrap();
    assert!(report.contains("x=2.000000000000 m/s; y=-1.000000000000 m/s"));
    assert!(report.contains("deterministic offset, not Gaussian sigma or filter correction"));
    assert!(
        std::fs::read_to_string(destination)
            .unwrap()
            .lines()
            .all(|line| line.split(',').count() == 35)
    );
}

#[test]
fn invalid_drift_rates_and_whole_horizons_create_no_output_parent() {
    let root = Temp::new();
    for field in ["velocity-drift-rate-x", "velocity-drift-rate-y"] {
        for value in ["NaN", "inf", "-inf", "-1.01", "1.01"] {
            let destination = root.0.join(field).join(value).join("run.csv");
            let output = Command::new(env!("CARGO_BIN_EXE_fusion-lab"))
                .args([
                    "--duration",
                    "1",
                    "--no-outage",
                    &format!("--{field}"),
                    value,
                    "--output",
                ])
                .arg(&destination)
                .output()
                .unwrap();
            assert!(!output.status.success());
            assert!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .contains("velocity_drift_rate")
            );
            assert!(!destination.parent().unwrap().exists());
        }
    }
    let destination = root.0.join("whole-horizon").join("run.csv");
    let output = Command::new(env!("CARGO_BIN_EXE_fusion-lab"))
        .args([
            "--duration",
            "3600",
            "--no-outage",
            "--velocity-drift-rate-x",
            "1",
            "--output",
        ])
        .arg(&destination)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("velocity_bias_horizon")
    );
    assert!(!destination.parent().unwrap().exists());
}

#[test]
fn signed_drift_cli_reports_initial_rate_duration_and_final_offset() {
    let root = Temp::new();
    let destination = root.0.join("drift.csv");
    let output = Command::new(env!("CARGO_BIN_EXE_fusion-lab"))
        .args([
            "--duration",
            "1",
            "--no-outage",
            "--velocity-bias-x",
            "0.2",
            "--velocity-bias-y",
            "-0.1",
            "--velocity-drift-rate-x",
            "0.4",
            "--velocity-drift-rate-y",
            "-0.2",
            "--output",
        ])
        .arg(&destination)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = std::fs::read_to_string(destination.with_extension("txt")).unwrap();
    for fragment in [
        "Initial velocity sensor bias (world frame): x=0.200000000000 m/s; y=-0.100000000000 m/s",
        "rate x=0.400000000000 m/s^2; y=-0.200000000000 m/s^2; duration=1 s",
        "final offset x=0.600000000000 m/s; y=-0.300000000000 m/s",
        "bias(t)=initial_bias+rate*t_seconds, not physical acceleration or filter correction",
    ] {
        assert!(report.contains(fragment), "missing {fragment}: {report}");
    }
    assert!(
        std::fs::read_to_string(destination)
            .unwrap()
            .lines()
            .all(|line| line.split(',').count() == 35)
    );
}
