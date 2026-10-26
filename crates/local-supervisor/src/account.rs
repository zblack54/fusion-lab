use crate::*;
use serde_json::Value;
const MODELS: &[&str] = &[
    "gpt-6.1-sol",
    "gpt-6-astra",
    "gpt-6-sol",
    "gpt-6-luna",
    "gpt-5.6-sol",
    "gpt-5.6-terra",
    "gpt-5.6-luna",
];
pub(crate) const STRIPPED: &[&str] = &[
    "OPENAI_API_KEY",
    "CODEX_API_KEY",
    "OPENAI_ADMIN_KEY",
    "AZURE_OPENAI_API_KEY",
    "AZURE_API_KEY",
    "ANTHROPIC_API_KEY",
    "GOOGLE_API_KEY",
    "GEMINI_API_KEY",
    "OPENAI_BASE_URL",
    "AZURE_OPENAI_ENDPOINT",
];
fn json(text: &str) -> Result<Value, Error> {
    if text.len() > 65536 {
        return Err(Error::Capacity);
    }
    serde_json::from_str(text).map_err(|_| Error::MalformedProtocol)
}
pub(super) fn parse_account(text: &str) -> Result<String, Error> {
    let wire = json(text)?;
    let account = &wire["account"];
    if account.is_null() {
        return Err(Error::AuthenticationRequired);
    }
    if account["type"].as_str() != Some("chatgpt") {
        return Err(Error::UnsupportedAuth);
    }
    Ok("chatgpt".into())
}
pub(super) fn parse_limits(text: &str) -> Result<Quota, Error> {
    let wire = json(text)?;
    let bucket = &wire["rateLimitsByLimitId"]["codex"];
    if !bucket.is_object() {
        return Err(Error::MissingQuota);
    }
    let window = |key: &str, minutes: u64| -> Result<Window, Error> {
        let item = &bucket[key];
        if !item.is_object() {
            return Err(Error::MissingQuota);
        }
        let used = item["usedPercent"]
            .as_u64()
            .ok_or(Error::MalformedProtocol)?;
        let duration = item["windowDurationMins"]
            .as_u64()
            .ok_or(Error::MalformedProtocol)?;
        let reset = item["resetsAt"].as_u64().ok_or(Error::MalformedProtocol)?;
        if used > 100 || duration != minutes || reset == 0 {
            return Err(Error::MalformedProtocol);
        }
        Ok(Window {
            used_percent: used as u32,
            minutes: duration as u32,
            resets_at: reset,
        })
    };
    Ok(Quota {
        primary: Some(window("primary", 300)?),
        secondary: Some(window("secondary", 10080)?),
    })
}
pub(super) fn plan(executable: &Path, model: &str) -> Result<WorkerPlan, Error> {
    if !MODELS.contains(&model) {
        return Err(Error::InvalidInput);
    }
    Ok(WorkerPlan {
        executable: executable.to_path_buf(),
        model: model.into(),
        argv: vec![
            "exec".into(),
            "--config".into(),
            "forced_login_method=\"chatgpt\"".into(),
            "--config".into(),
            "model_provider=\"openai\"".into(),
            "--model".into(),
            model.into(),
            "--json".into(),
            "--config".into(),
            "sandbox_mode=\"workspace-write\"".into(),
        ],
        removed_environment_names: STRIPPED.iter().map(|x| (*x).into()).collect(),
    })
}

/// Apply the same credential policy to account control and inference processes.
#[cfg(windows)]
pub(crate) fn command(runtime: &RuntimeConfig) -> std::process::Command {
    let mut command = std::process::Command::new(&runtime.codex_executable);
    command
        .current_dir(&runtime.checkout)
        .envs(runtime.environment.iter().cloned());
    let names = std::env::vars_os()
        .map(|(k, _)| k)
        .chain(runtime.environment.iter().map(|(k, _)| k.into()));
    for key in names {
        let upper = key.to_string_lossy().to_ascii_uppercase();
        if STRIPPED.contains(&upper.as_str())
            || upper.ends_with("_API_KEY")
            || upper.ends_with("_ACCESS_TOKEN")
            || upper == "GH_TOKEN"
            || upper == "GITHUB_TOKEN"
        {
            command.env_remove(key);
        }
    }
    command
}
#[cfg(windows)]
struct Control {
    child: Option<crate::native::Owned>,
    input: Option<std::process::ChildStdin>,
    receiver: std::sync::mpsc::Receiver<Result<Value, Error>>,
    deadline: std::time::Instant,
}
#[cfg(windows)]
impl Control {
    fn request(&mut self, id: u64, method: &str, params: Value) -> Result<Value, Error> {
        use std::io::Write;
        let input = self.input.as_mut().ok_or(Error::Io)?;
        let wire =
            serde_json::to_vec(&serde_json::json!({"id":id,"method":method,"params":params}))
                .map_err(|_| Error::MalformedProtocol)?;
        input
            .write_all(&wire)
            .and_then(|()| input.write_all(b"\n"))
            .and_then(|()| input.flush())
            .map_err(|_| Error::Io)?;
        let deadline = self.deadline;
        for _ in 0..128 {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let message = self
                .receiver
                .recv_timeout(remaining)
                .map_err(|_| Error::Deadline)??;
            if message["id"].as_u64() != Some(id) {
                continue;
            }
            if message.get("error").is_some() {
                return Err(Error::MalformedProtocol);
            }
            return message
                .get("result")
                .cloned()
                .ok_or(Error::MalformedProtocol);
        }
        Err(Error::MalformedProtocol)
    }
    fn close(&mut self) -> Result<(), Error> {
        self.input.take();
        let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let child = self.child.as_mut().ok_or(Error::Io)?;
        let status = loop {
            if let Some(status) = child.poll()? {
                break Some(status);
            }
            if std::time::Instant::now() >= until {
                break None;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        child.stop(std::time::Duration::from_secs(5))?;
        let (_, empty) = std::sync::mpsc::channel();
        let receiver = std::mem::replace(&mut self.receiver, empty);
        drop(receiver);
        child.join_readers()?;
        if status.is_some_and(|status| status.success()) {
            Ok(())
        } else {
            Err(Error::Io)
        }
    }
}
#[cfg(windows)]
impl Drop for Control {
    fn drop(&mut self) {
        if self.child.is_some() {
            let _ = self.close();
        }
    }
}
/// Owned stdio initialize/account/quota exchange; only typed nonsecret results escape.
#[cfg(windows)]
pub(crate) fn fetch(engine: &mut Supervisor, runtime: &RuntimeConfig) -> Result<Quota, Error> {
    use std::io::{BufRead, Read, Write};
    use std::process::Stdio;
    let handshake_deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut command = command(runtime);
    command
        .args([
            "--config",
            "forced_login_method=\"chatgpt\"",
            "--config",
            "model_provider=\"openai\"",
            "app-server",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match crate::native::Owned::start(&mut command) {
        Ok(child) => child,
        Err(failure) => {
            if let Some(child) = failure.retained {
                engine.quarantine = Some((child, None));
            }
            return Err(failure.error);
        }
    };
    let input = child.child.stdin.take().ok_or(Error::Io)?;
    let stdout = child.child.stdout.take().ok_or(Error::Io)?;
    let mut stderr = child.child.stderr.take().ok_or(Error::Io)?;
    let (sender, receiver) = std::sync::mpsc::sync_channel(16);
    let output_reader = std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        loop {
            let mut line = Vec::new();
            let result = reader.by_ref().take(65537).read_until(b'\n', &mut line);
            let outcome = match result {
                Ok(0) => break,
                Ok(n) if n <= 65536 => {
                    serde_json::from_slice(&line).map_err(|_| Error::MalformedProtocol)
                }
                Ok(_) => Err(Error::Capacity),
                Err(_) => Err(Error::Io),
            };
            let failed = outcome.is_err();
            if sender.send(outcome).is_err() || failed {
                break;
            }
        }
    });
    let error_reader = std::thread::spawn(move || {
        let mut bytes = [0u8; 8192];
        loop {
            match stderr.read(&mut bytes) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
        }
    });
    child.readers = vec![output_reader, error_reader];
    let mut control = Control {
        child: Some(child),
        input: Some(input),
        receiver,
        deadline: handshake_deadline,
    };
    let outcome = (|| {
        control.request(1,"initialize",serde_json::json!({"clientInfo":{"name":"fusion-lab-supervisor","version":env!("CARGO_PKG_VERSION")}}))?;
        let input = control.input.as_mut().ok_or(Error::Io)?;
        writeln!(input, "{}", serde_json::json!({"method":"initialized"}))
            .map_err(|_| Error::Io)?;
        input.flush().map_err(|_| Error::Io)?;
        let account =
            control.request(2, "account/read", serde_json::json!({"refreshToken":false}))?;
        parse_account(&account.to_string())?;
        let limits = control.request(3, "account/rateLimits/read", serde_json::json!({}))?;
        parse_limits(&limits.to_string())
    })();
    let cleanup = control.close();
    if cleanup.is_err()
        && let Some(child) = control.child.take()
    {
        engine.quarantine = Some((child, None));
    }
    match (outcome, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(quota), Ok(())) => Ok(quota),
    }
}
