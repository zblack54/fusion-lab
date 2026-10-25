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
