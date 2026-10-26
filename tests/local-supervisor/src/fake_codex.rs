//! Test-owned external Codex protocol peer. Never used as production behavior.
use std::{
    env, fs,
    io::{self, BufRead, Read, Write},
    path::PathBuf,
    thread,
    time::Duration,
};
fn number(s: &str, key: &str) -> u64 {
    s.split(&format!("\"{key}\":"))
        .nth(1)
        .unwrap_or("0")
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}
fn main() {
    let root = PathBuf::from(env::var("FAKE_CODEX_ROOT").expect("controlled fixture directory"));
    let scenario = fs::read_to_string(root.join("scenario.txt")).unwrap();
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|x| x == "app-server") {
        for line in io::stdin().lock().lines() {
            let line = line.unwrap();
            if !line.contains("\"id\"") {
                continue;
            }
            let id = number(&line, "id");
            let result = if line.contains("initialize") {
                r#"{"userAgent":"fake-codex"}"#.into()
            } else if line.contains("account/read") {
                if scenario == "unauth" {
                    r#"{"account":null,"requiresOpenaiAuth":true}"#.into()
                } else if scenario == "api" {
                    r#"{"account":{"type":"apiKey"},"requiresOpenaiAuth":true}"#.into()
                } else {
                    r#"{"account":{"type":"chatgpt","planType":"plus"},"requiresOpenaiAuth":true}"#
                        .into()
                }
            } else if line.contains("account/rateLimits/read") {
                if scenario == "missing" {
                    r#"{"rateLimits":null,"rateLimitsByLimitId":null}"#.into()
                } else {
                    let used = if scenario == "zero" { 100 } else { 0 };
                    format!(
                        r#"{{"rateLimitsByLimitId":{{"codex":{{"primary":{{"usedPercent":{used},"windowDurationMins":300,"resetsAt":2000000000}},"secondary":{{"usedPercent":0,"windowDurationMins":10080,"resetsAt":2000604800}}}}}}}}"#
                    )
                }
            } else if line.contains("model/list") {
                r#"{"data":[{"id":"gpt-6-luna","model":"gpt-6-luna"}],"nextCursor":null}"#.into()
            } else {
                r#"{}"#.into()
            };
            println!(r#"{{"id":{id},"result":{result}}}"#);
            io::stdout().flush().unwrap();
        }
        return;
    }
    let mut prompt = String::new();
    io::stdin().read_to_string(&mut prompt).unwrap();
    let id = number(&prompt, "task_id");
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("launch.txt"))
        .unwrap();
    writeln!(f, "task_id={id}").unwrap();
    for a in &args {
        writeln!(f, "arg={a}").unwrap();
    }
    for k in ["OPENAI_API_KEY", "CODEX_API_KEY"] {
        writeln!(f, "env_present_{k}={}", env::var_os(k).is_some()).unwrap();
    }
    f.sync_all().unwrap();
    if scenario == "slow" {
        thread::sleep(Duration::from_secs(30));
    }
    if scenario == "nonzero" {
        std::process::exit(7)
    }
    let checkout = root.parent().unwrap();
    let file = format!("fixture-task-{id}.txt");
    fs::write(checkout.join(&file), format!("task {id}")).unwrap();
    for args in [
        vec!["add".to_string(), file],
        vec![
            "-c".into(),
            "user.name=Spec".into(),
            "-c".into(),
            "user.email=spec@example.invalid".into(),
            "-c".into(),
            "commit.gpgsign=false".into(),
            "commit".into(),
            "-m".into(),
            format!("fixture task {id}"),
        ],
    ] {
        let result = std::process::Command::new("git")
            .current_dir(checkout)
            .args(args)
            .output()
            .unwrap();
        assert!(result.status.success(), "fixture git commit failed");
    }
    let observed = std::process::Command::new("git")
        .current_dir(checkout)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(observed.status.success());
    let head = String::from_utf8(observed.stdout).unwrap();
    let head = head.trim();
    let result = format!(
        r#"{{"task_id":{id},"status":"implemented","head":"{head}","checkpoint":"session-{id}"}}"#
    );
    if let Some(i) = args
        .iter()
        .position(|a| a == "--output-last-message" || a == "-o")
    {
        fs::write(&args[i + 1], &result).unwrap();
    }
    println!(r#"{{"type":"thread.started","thread_id":"session-{id}"}}"#);
    println!(
        r#"{{"type":"item.completed","item":{{"type":"agent_message","text":"fixture completed"}}}}"#
    );
    println!(r#"{{"type":"turn.completed","usage":{{"input_tokens":1,"output_tokens":1}}}}"#);
}
