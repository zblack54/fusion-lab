//! Reproducible local probe host: JSON address on stdout, explicit shutdown on stdin.
use fusion_experiment_service::{Service, ServiceOptions};
use std::{io, path::PathBuf, time::Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "probe_host ROOT WORKER").into());
    }
    let service = Service::start(ServiceOptions {
        bind: "127.0.0.1:0".parse()?,
        artifact_root: PathBuf::from(&args[0]),
        worker: PathBuf::from(&args[1]),
        retained_runs: 2,
        artifact_budget_bytes: 768 * 1024 * 1024,
    })?;
    println!(
        "{{\"address\":\"{}\",\"pid\":{}}}",
        service.address(),
        std::process::id()
    );
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let start = Instant::now();
    service.shutdown()?;
    println!("{{\"shutdown_seconds\":{}}}", start.elapsed().as_secs_f64());
    Ok(())
}
