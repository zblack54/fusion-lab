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
