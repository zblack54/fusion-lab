//! One-second reproduction of virtual planned observation delivery.
use fusion_lab::{Config, DeliveryPlan, ReplayModel, Trace, replay_delivery, run};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

fn publish(directory: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let temporary = directory.join(format!("{name}.tmp"));
    let destination = directory.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(bytes)?;
    file.flush()?;
    drop(file);
    fs::rename(temporary, destination)
}

fn reproduce(directory: &Path) -> Result<(), Box<dyn Error>> {
    let config = Config {
        duration_seconds: 1,
        outage: None,
        ..Config::default()
    };
    let source = run(&config)?;
    let model = ReplayModel::new(1.0, 0.15, 0.1)?;
    let plan = DeliveryPlan::new(0, 5, 8, None, None)?;
    let trace = Trace::from_samples(1, &source.samples, plan)?;
    let result = replay_delivery(&model, &trace)?;
    // Finish both capped artifacts before creating any filesystem output.
    let trace_bytes = trace.encode_tsv(&model)?;
    let ledger_bytes = result.encode_ledger_tsv()?;
    // create_dir fails on a preexisting path without touching its contents.
    fs::create_dir(directory)?;
    if let Err(error) = publish(directory, "trace.tsv", &trace_bytes) {
        return Err(io::Error::new(error.kind(), format!("trace publication failed: {error}; owned directory {} may contain trace.tsv.tmp; no ledger was published", directory.display())).into());
    }
    if let Err(error) = publish(directory, "ledger.tsv", &ledger_bytes) {
        return Err(io::Error::new(error.kind(), format!("ledger publication failed: {error}; {} contains complete trace.tsv and may contain ledger.tsv.tmp", directory.display())).into());
    }
    println!(
        "Published complete trace.tsv ({} bytes) and ledger.tsv ({} bytes) in {}",
        trace_bytes.len(),
        ledger_bytes.len(),
        directory.display()
    );
    println!(
        "Virtual planned OnTimeOnly evidence: {} snapshots, {} offered observations; parent #6 async/channel/cancellation work remains incomplete.",
        result.snapshots().len(),
        result.ledger().len()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let directory = args
        .next()
        .ok_or("usage: observation_delivery NEW_OUTPUT_DIRECTORY")?;
    if args.next().is_some() {
        return Err("usage: observation_delivery NEW_OUTPUT_DIRECTORY".into());
    }
    reproduce(Path::new(&directory))
}
