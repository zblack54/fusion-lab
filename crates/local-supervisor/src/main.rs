//! Local control CLI. No recurring OS task is registered.
#![forbid(unsafe_code)]
use local_supervisor::*;
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf};

struct Args {
    root: PathBuf,
    command: String,
    values: BTreeMap<String, String>,
}
fn arguments() -> Result<Args, ()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 3 || args[0] != "--state-dir" || args[1].is_empty() {
        return Err(());
    }
    let command = args[2].clone();
    if ![
        "enqueue",
        "status",
        "disable",
        "enable",
        "cancel",
        "step",
        "run",
        "bridge-export",
        "bridge-import",
    ]
    .contains(&command.as_str())
    {
        return Err(());
    }
    let mut values = BTreeMap::new();
    for pair in args[3..].chunks(2) {
        if pair.len() != 2
            || !pair[0].starts_with("--")
            || values.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err(());
        }
    }
    let allowed = match command.as_str() {
        "enqueue" => vec!["--task", "--issue", "--scope", "--depends"],
        "cancel" => vec!["--task"],
        "step" | "run" => vec!["--codex", "--model", "--child-env"],
        _ => vec![],
    };
    if values.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err(());
    }
    if command == "enqueue" {
        positive(&values, "--task")?;
        positive(&values, "--issue")?;
        if values.get("--scope").is_none_or(|s| s.is_empty()) {
            return Err(());
        }
        dependencies(&values)?;
    }
    if command == "cancel" {
        positive(&values, "--task")?;
    }
    if let Some(env) = values.get("--child-env")
        && env.split_once('=').is_none_or(|(key, _)| key.is_empty())
    {
        return Err(());
    }
    Ok(Args {
        root: PathBuf::from(&args[1]),
        command,
        values,
    })
}
fn positive(values: &BTreeMap<String, String>, key: &str) -> Result<u64, ()> {
    values
        .get(key)
        .ok_or(())?
        .parse::<u64>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or(())
}
fn dependencies(values: &BTreeMap<String, String>) -> Result<Vec<u64>, ()> {
    values
        .get("--depends")
        .map(|s| {
            s.split(',')
                .map(|x| x.parse::<u64>().ok().filter(|n| *n > 0).ok_or(()))
                .collect()
        })
        .unwrap_or(Ok(vec![]))
}
fn execute(args: Args) -> Result<serde_json::Value, Error> {
    let mut engine = Supervisor::open(&args.root)?;
    match args.command.as_str() {
        "enqueue" => {
            let id = positive(&args.values, "--task").map_err(|_| Error::InvalidInput)?;
            engine.apply(Event::Submit(Task {
                id,
                issue: positive(&args.values, "--issue").map_err(|_| Error::InvalidInput)?,
                scope_digest: args.values["--scope"].clone(),
                dependencies: dependencies(&args.values).map_err(|_| Error::InvalidInput)?,
            }))?;
            Ok(json!({"accepted":id}))
        }
        "status" => serde_json::to_value(engine.snapshot()?).map_err(|_| Error::Io),
        "disable" | "enable" => {
            let enabled = args.command == "enable";
            engine.apply(Event::Enable(enabled))?;
            Ok(json!({"enabled":enabled}))
        }
        "cancel" => serde_json::to_value(engine.apply(Event::Cancel(
            positive(&args.values, "--task").map_err(|_| Error::InvalidInput)?,
        ))?)
        .map_err(|_| Error::Io),
        "bridge-export" => serde_json::to_value(engine.write_bridge(&args.root.join("bridge"))?)
            .map_err(|_| Error::Io),
        "bridge-import" => serde_json::to_value(engine.read_bridge(&args.root.join("bridge"))?)
            .map_err(|_| Error::Io),
        "step" | "run" => {
            let model = args
                .values
                .get("--model")
                .cloned()
                .unwrap_or_else(|| "gpt-6-luna".into());
            let executable = args
                .values
                .get("--codex")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("codex"));
            let environment = args
                .values
                .get("--child-env")
                .and_then(|v| v.split_once('='))
                .map(|(k, v)| vec![(k.into(), v.into())])
                .unwrap_or_default();
            let runtime = RuntimeConfig {
                codex_executable: executable,
                checkout: args.root.clone(),
                bridge: args.root.join("bridge"),
                model,
                environment,
                worker_deadline_ms: 1800000,
            };
            loop {
                let report = engine.drive_once(&runtime)?;
                if args.command == "step" || report.terminal != "continue" {
                    return serde_json::to_value(report).map_err(|_| Error::Io);
                }
                // Continue only when the durable cycle explicitly reports ready progress.
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        _ => Err(Error::InvalidInput),
    }
}
fn main() {
    let args = match arguments() {
        Ok(args) => args,
        Err(()) => {
            eprintln!("invalid command");
            std::process::exit(2)
        }
    };
    match execute(args) {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("{error:?}");
            std::process::exit(1)
        }
    }
}
