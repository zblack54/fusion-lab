//! Standalone local service; Ctrl+C performs explicit fallible shutdown.
use fusion_experiment_service::{Service, ServiceOptions};
use std::{collections::HashSet, io, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = ServiceOptions {
        bind: "127.0.0.1:8080".parse()?,
        artifact_root: PathBuf::from("results/service"),
        worker: PathBuf::from(if cfg!(windows) {
            "target/debug/fusion-lab.exe"
        } else {
            "target/debug/fusion-lab"
        }),
        retained_runs: 2,
        artifact_budget_bytes: 768 * 1024 * 1024,
    };
    let mut args = std::env::args_os().skip(1);
    let mut seen = HashSet::new();
    while let Some(flag) = args.next() {
        let name = flag.to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "option name must be UTF-8")
        })?;
        if matches!(name, "--help" | "-h") {
            println!(
                "Fusion Lab local experiment service\n\nOptions:\n  --bind ADDRESS                 Loopback socket [127.0.0.1:8080]\n  --artifact-root PATH           Empty private root [results/service]\n  --worker PATH                  Numerical CLI [target/debug/fusion-lab]\n  --retained-runs COUNT          Terminal retention 1..32 [2]\n  --artifact-budget-bytes BYTES  At least 268435456 [805306368]\n\nBuild the numerical CLI first with cargo build --locked. Open GET / at the bound local origin for the dashboard, recording import and replay controls. POST a complete JSON configuration to /v1/runs. Press Ctrl+C to cancel/reap workers and stop. Artifacts are transient; an interrupted nonempty root requires inspection before reuse. POST retries create distinct runs."
            );
            return Ok(());
        }
        if !seen.insert(name.to_owned()) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "duplicate option").into());
        }
        let value = args
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "option needs a value"))?;
        match name {
            "--bind" => options.bind = value.to_string_lossy().parse()?,
            "--artifact-root" => options.artifact_root = value.into(),
            "--worker" => options.worker = value.into(),
            "--retained-runs" => options.retained_runs = value.to_string_lossy().parse()?,
            "--artifact-budget-bytes" => {
                options.artifact_budget_bytes = value.to_string_lossy().parse()?
            }
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "unknown option").into()),
        }
    }
    let service = Service::start(options)?;
    println!("Fusion Lab listening at http://{}", service.address());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(tokio::signal::ctrl_c())?;
    service.shutdown()?;
    Ok(())
}
