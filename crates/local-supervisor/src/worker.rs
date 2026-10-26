//! One owned native cycle; host effects remain outside the inference process.
use crate::{native::Owned, storage, *};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, Read, Write},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn now() -> Result<u64, Error> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|t| t.as_secs())
        .map_err(|_| Error::Io)
}
fn token() -> Result<String, Error> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| Error::Io)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn pause(engine: &Supervisor) -> Result<(), Error> {
    storage::update(engine, "native_ownership_uncertain", |data| {
        data.pause = Pause::AmbiguousProcess;
        Ok(())
    })
}
fn report(
    engine: &Supervisor,
    task: Option<u64>,
    terminal: &str,
    checkpoint: Option<String>,
    exit_code: Option<i32>,
) -> Result<DriveReport, Error> {
    let s = engine.snapshot()?;
    Ok(DriveReport {
        task,
        terminal: terminal.into(),
        launches: s.worker_launches,
        next_wake: s.next_wake,
        checkpoint,
        exit_code,
    })
}
fn quarantine(engine: &mut Supervisor, owned: Owned, guard: Option<fs::File>) -> Result<(), Error> {
    engine.quarantine = Some((owned, guard));
    pause(engine)
}
fn finished(
    engine: &Supervisor,
    owner: &str,
    id: u64,
    state: State,
    checkpoint: Option<String>,
    head: Option<String>,
    exit: Option<i32>,
) -> Result<(), Error> {
    storage::update(
        engine,
        &json!({"event":"native_cleanup_confirmed","task":id,"exit_code":exit,"job_members":0})
            .to_string(),
        |data| {
            data.cleanup_owner = Some(owner.into());
            data.native_active = None;
            data.active = None;
            data.owner = None;
            data.cancel_requested.remove(&id);
            let task = data.tasks.get_mut(&id).ok_or(Error::InvalidInput)?;
            task.state = state;
            if let Some(head) = head {
                task.head = Some(head)
            }
            if let Some(checkpoint) = checkpoint {
                data.checkpoints.insert(id, checkpoint);
            }
            Ok(())
        },
    )
}
fn marker_path(checkout: &Path) -> Result<PathBuf, Error> {
    let git = checkout.join(".git");
    if !git.is_dir() {
        return Err(Error::InvalidInput);
    }
    Ok(git.join("fusion-supervisor-active.json"))
}
fn claim_checkout(
    engine: &Supervisor,
    runtime: &RuntimeConfig,
) -> Result<(fs::File, PathBuf), Error> {
    let checkout = fs::canonicalize(&runtime.checkout).map_err(|_| Error::Io)?;
    let marker = marker_path(&checkout)?;
    let guard = storage::acquire(&checkout.join(".git/fusion-supervisor-writer.lock"))?;
    if marker.exists() {
        let bytes = read_bounded(&marker, 65536).map_err(|_| Error::AmbiguousSurvivor)?;
        let prior: Value = serde_json::from_slice(&bytes).map_err(|_| Error::AmbiguousSurvivor)?;
        let data = storage::load(&storage::connection(&engine.root)?)?;
        if prior["state_root"].as_str() != engine.root.to_str()
            || data.cleanup_owner.as_deref() != prior["owner"].as_str()
        {
            pause(engine)?;
            return Err(Error::AmbiguousSurvivor);
        }
        fs::remove_file(&marker).map_err(|_| Error::Io)?;
    }
    Ok((guard, marker))
}
fn write_marker(engine: &Supervisor, marker: &Path, owner: &str) -> Result<(), Error> {
    crate::bridge::atomic(
        marker,
        &serde_json::to_vec(&json!({"state_root":engine.root,"owner":owner}))
            .map_err(|_| Error::Io)?,
    )
}
fn clear_marker(marker: &Path) -> Result<(), Error> {
    fs::remove_file(marker).map_err(|_| Error::Io)
}

/// Metadata Git is also an owned contained child with a deadline and bounded capture.
fn git_head(engine: &mut Supervisor, runtime: &RuntimeConfig) -> Result<String, Error> {
    let mut command = Command::new("git");
    command
        .current_dir(&runtime.checkout)
        .args(["rev-parse", "HEAD"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = match Owned::start(&mut command) {
        Ok(child) => child,
        Err(failure) => {
            if let Some(child) = failure.retained {
                quarantine(engine, child, None)?;
            }
            return Err(failure.error);
        }
    };
    let outcome = (|| -> Result<String, Error> {
        let mut pipe = child.child.stdout.take().ok_or(Error::Io)?;
        let (sender, receiver) = mpsc::sync_channel(1);
        child.readers.push(thread::spawn(move || {
            let mut out = Vec::new();
            let result = pipe.by_ref().take(1024).read_to_end(&mut out);
            let _ = std::io::copy(&mut pipe, &mut std::io::sink());
            let _ = sender.send(result.map(|_| out));
        }));
        let until = Instant::now() + Duration::from_secs(30);
        let status = loop {
            if let Some(status) = child.poll()? {
                break status;
            }
            if Instant::now() >= until {
                if child.stop(Duration::from_secs(5)).is_err() {
                    return Err(Error::AmbiguousSurvivor);
                }
                child.join_readers()?;
                return Err(Error::Deadline);
            }
            thread::sleep(Duration::from_millis(10));
        };
        if child.stop(Duration::from_secs(5)).is_err() {
            return Err(Error::AmbiguousSurvivor);
        }
        child.join_readers()?;
        let bytes = receiver
            .recv()
            .map_err(|_| Error::Io)?
            .map_err(|_| Error::Io)?;
        if !status.success() {
            return Err(Error::Io);
        }
        let head = std::str::from_utf8(&bytes)
            .map_err(|_| Error::MalformedProtocol)?
            .trim();
        if head.len() != 40 || !head.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::MalformedProtocol);
        }
        Ok(head.into())
    })();
    if outcome.is_err() {
        if child.stop(Duration::from_secs(5)).is_err() {
            quarantine(engine, child, None)?;
            return Err(Error::AmbiguousSurvivor);
        }
        child.join_readers()?;
    }
    outcome
}
struct StreamResult {
    session: Option<String>,
    completed: bool,
    failed: bool,
    malformed: bool,
    bytes: u64,
}
fn read_events(pipe: impl Read) -> StreamResult {
    let mut reader = std::io::BufReader::new(pipe);
    let mut result = StreamResult {
        session: None,
        completed: false,
        failed: false,
        malformed: false,
        bytes: 0,
    };
    loop {
        let mut line = Vec::new();
        match reader.by_ref().take(65537).read_until(b'\n', &mut line) {
            Ok(0) => break,
            Ok(size) => {
                result.bytes = result.bytes.saturating_add(size as u64);
                if size > 65536 || result.bytes > 16 * 1024 * 1024 {
                    result.malformed = true;
                    let _ = std::io::copy(&mut reader, &mut std::io::sink());
                    break;
                }
                match serde_json::from_slice::<Value>(&line) {
                    Ok(event) => match event["type"].as_str() {
                        Some("thread.started") => {
                            result.session = event["thread_id"].as_str().map(str::to_owned)
                        }
                        Some("turn.completed") => result.completed = true,
                        Some("turn.failed" | "error") => result.failed = true,
                        _ => {}
                    },
                    Err(_) => result.malformed = true,
                }
            }
            Err(_) => {
                result.malformed = true;
                break;
            }
        }
    }
    result
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Final {
    task_id: u64,
    status: String,
    head: String,
    checkpoint: String,
}

pub(super) fn drive(
    engine: &mut Supervisor,
    runtime: &RuntimeConfig,
) -> Result<DriveReport, Error> {
    crate::native::root()?;
    engine.plan_worker(&runtime.codex_executable, &runtime.model)?;
    if !(1..=3600000).contains(&runtime.worker_deadline_ms) {
        return Err(Error::InvalidInput);
    }
    let _scheduler = storage::acquire(&engine.root.join("scheduler.lock"))?;
    if engine.quarantine.is_some() {
        return Err(Error::AmbiguousSurvivor);
    }
    engine.read_bridge(&runtime.bridge)?;
    let data = storage::load(&storage::connection(&engine.root)?)?;
    if !data.enabled {
        return report(engine, None, "disabled", None, None);
    }
    if data.active.is_some()
        || data.native_active.is_some()
        || data.pause == Pause::AmbiguousProcess
    {
        pause(engine)?;
        return Err(Error::AmbiguousSurvivor);
    }
    if !data.intents.is_empty() {
        engine.write_bridge(&runtime.bridge)?;
        return report(engine, None, "awaiting_host", None, None);
    }
    if !data.tasks.values().any(|task| {
        task.state == State::Ready
            && task
                .task
                .dependencies
                .iter()
                .all(|id| data.tasks[id].state == State::Done)
    }) {
        return report(engine, None, "no_ready_scope", None, None);
    }
    let (checkout_guard, marker) = claim_checkout(engine, runtime)?;
    let owner = token()?;
    write_marker(engine, &marker, &owner)?;
    let quota = match crate::account::fetch(engine, runtime) {
        Ok(quota) => quota,
        Err(error) => {
            if let Some((child, _)) = engine.quarantine.take() {
                quarantine(engine, child, Some(checkout_guard))?;
                return Err(Error::AmbiguousSurvivor);
            }
            storage::update(engine, "control_cleanup_confirmed", |data| {
                data.cleanup_owner = Some(owner.clone());
                Ok(())
            })?;
            clear_marker(&marker)?;
            let auth = if matches!(
                error,
                Error::AuthenticationRequired | Error::UnsupportedAuth
            ) {
                "unavailable"
            } else {
                "chatgpt"
            };
            match engine.apply(Event::Quota {
                auth_type: auth.into(),
                quota: None,
                now: now()?,
            }) {
                Ok(_) | Err(Error::UnsupportedAuth | Error::MissingQuota) => {}
                Err(error) => return Err(error),
            }
            return Err(error);
        }
    };
    let state = engine.apply(Event::Quota {
        auth_type: "chatgpt".into(),
        quota: Some(quota),
        now: now()?,
    })?;
    if state.pause == Pause::Quota {
        storage::update(engine, "control_cleanup_confirmed", |data| {
            data.cleanup_owner = Some(owner.clone());
            Ok(())
        })?;
        clear_marker(&marker)?;
        return Err(Error::QuotaExhausted);
    }
    let initial_head = match git_head(engine, runtime) {
        Ok(head) => head,
        Err(error) => {
            if let Some((child, _)) = engine.quarantine.take() {
                quarantine(engine, child, Some(checkout_guard))?;
            }
            return Err(error);
        }
    };
    let claimed = engine.apply(Event::Claim {
        owner: owner.clone(),
        now: now()?,
    })?;
    let id = claimed.active.ok_or(Error::InvalidTransition)?;
    let data = storage::load(&storage::connection(&engine.root)?)?;
    let task = data.tasks[&id].task.clone();
    storage::update(engine, "native_attempt_reserved", |data| {
        data.native_active = Some(storage::NativeRecord {
            owner: owner.clone(),
            task: id,
            generation: data.generation,
            pid: None,
            model: runtime.model.clone(),
            reasoning: "low".into(),
        });
        Ok(())
    })?;
    let attempt = engine.root.join("attempts").join(&owner);
    fs::create_dir_all(&attempt).map_err(|_| Error::Io)?;
    let final_path = attempt.join("final.json");
    let schema = attempt.join("schema.json");
    crate::bridge::atomic(&schema,&serde_json::to_vec(&json!({"type":"object","properties":{"task_id":{"type":"integer"},"status":{"type":"string","enum":["implemented"]},"head":{"type":"string"},"checkpoint":{"type":"string"}},"required":["task_id","status","head","checkpoint"],"additionalProperties":false})).map_err(|_|Error::Io)?)?;
    let plan = engine.plan_worker(&runtime.codex_executable, &runtime.model)?;
    let mut command = crate::account::command(runtime);
    let mut arguments = plan.argv;
    if let Some(checkpoint) = data.checkpoints.get(&id) {
        arguments.insert(1, "resume".into());
        arguments.insert(2, checkpoint.clone());
    }
    command.args(&arguments);
    command
        .args([
            "--config",
            "model_reasoning_effort=\"low\"",
            "--output-schema",
        ])
        .arg(&schema)
        .arg("--output-last-message")
        .arg(&final_path);
    let prompt=json!({"task_id":id,"issue":task.issue,"scope_digest":task.scope_digest,"expected_head":initial_head,"instructions":"Implement only the approved issue scope. Read AGENTS.md and approved design before edits. Keep locked test contracts unchanged. Verify changes, commit locally, and return the requested JSON with the actual Git head and current session checkpoint. Do not use paid APIs, API-key auth, deployment, live trading, or host GitHub writes. Report contract conflicts rather than weakening tests."}).to_string();
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match Owned::start(&mut command) {
        Ok(child) => child,
        Err(failure) => {
            if let Some(child) = failure.retained {
                quarantine(engine, child, Some(checkout_guard))?;
                return Err(Error::AmbiguousSurvivor);
            }
            finished(engine, &owner, id, State::Paused, None, None, None)?;
            clear_marker(&marker)?;
            engine.writer.take();
            return Err(failure.error);
        }
    };
    let outcome = (|| -> Result<DriveReport, Error> {
        storage::update(engine, "native_worker_spawned", |data| {
            let record = data
                .native_active
                .as_mut()
                .ok_or(Error::InvalidTransition)?;
            record.pid = Some(child.child.id());
            data.launches = data.launches.checked_add(1).ok_or(Error::Capacity)?;
            Ok(())
        })?;
        let mut input = child.child.stdin.take().ok_or(Error::Io)?;
        let output = child.child.stdout.take().ok_or(Error::Io)?;
        let mut errors = child.child.stderr.take().ok_or(Error::Io)?;
        let pipe_failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stderr_failed = pipe_failed.clone();
        let stdin_failed = pipe_failed.clone();
        let (event_sender, event_receiver) = mpsc::sync_channel(1);
        child.readers.push(thread::spawn(move || {
            let result = read_events(output);
            let _ = event_sender.send(result);
        }));
        child.readers.push(thread::spawn(move || {
            let mut buffer = [0u8; 8192];
            loop {
                match errors.read(&mut buffer) {
                    Ok(0) => break,
                    Err(_) => {
                        stderr_failed.store(true, std::sync::atomic::Ordering::SeqCst);
                        break;
                    }
                    Ok(_) => {}
                }
            }
        }));
        child.readers.push(thread::spawn(move || {
            if input.write_all(prompt.as_bytes()).is_err() {
                stdin_failed.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            drop(input);
        }));
        let until = Instant::now() + Duration::from_millis(runtime.worker_deadline_ms);
        let mut interrupted = None;
        let status = loop {
            let state = storage::load(&storage::connection(&engine.root)?)?;
            if state.cancel_requested.contains(&id) || !state.enabled {
                interrupted = Some(Error::Cancelled);
                break None;
            }
            if Instant::now() >= until {
                interrupted = Some(Error::Deadline);
                break None;
            }
            match child.poll() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => {}
                Err(error) => {
                    interrupted = Some(error);
                    break None;
                }
            }
            thread::sleep(Duration::from_millis(10));
        };
        if child.stop(Duration::from_secs(5)).is_err() {
            return Err(Error::AmbiguousSurvivor);
        }
        child.join_readers()?;
        let mut events = event_receiver.recv().map_err(|_| Error::Io)?;
        events.malformed |= pipe_failed.load(std::sync::atomic::Ordering::SeqCst);
        storage::update(engine,&json!({"event":"native_stream_metadata","task":id,"stdout_bytes":events.bytes,"truncated":events.bytes>16*1024*1024,"stream_failed":events.malformed}).to_string(),|_|Ok(()))?;
        if let Some(error) = interrupted {
            finished(
                engine,
                &owner,
                id,
                if error == Error::Cancelled {
                    State::Cancelled
                } else {
                    State::Paused
                },
                events.session,
                None,
                None,
            )?;
            clear_marker(&marker)?;
            engine.writer.take();
            return Err(error);
        }
        let status = status.ok_or(Error::Io)?;
        let code = status.code();
        if !status.success() {
            finished(
                engine,
                &owner,
                id,
                State::Paused,
                events.session.clone(),
                None,
                code,
            )?;
            clear_marker(&marker)?;
            engine.writer.take();
            return report(engine, Some(id), "failed", events.session, code);
        }
        if !events.completed || events.failed || events.malformed {
            finished(
                engine,
                &owner,
                id,
                State::Paused,
                events.session,
                None,
                code,
            )?;
            clear_marker(&marker)?;
            engine.writer.take();
            return Err(Error::MalformedProtocol);
        }
        let bytes = read_bounded(&final_path, 65536)?;
        let result: Final = serde_json::from_slice(&bytes).map_err(|_| Error::MalformedProtocol)?;
        let session = events
            .session
            .filter(|s| {
                !s.is_empty()
                    && s.len() <= 256
                    && s.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            })
            .ok_or(Error::MalformedProtocol)?;
        if result.task_id != id || result.status != "implemented" || result.checkpoint.is_empty() {
            return Err(Error::MalformedProtocol);
        }
        let actual = git_head(engine, runtime)?;
        if result.head != actual {
            return Err(Error::EvidenceMismatch);
        }
        finished(
            engine,
            &owner,
            id,
            State::ImplementedUnverified,
            Some(session.clone()),
            Some(actual.clone()),
            code,
        )?;
        storage::update(engine, "implemented_awaiting_host", |data| {
            let intent = Intent {
                id: format!("pr:{id}:{actual}"),
                task: id,
                kind: IntentKind::CreatePr,
                expected_head: actual,
            };
            crate::bridge::intent(data, intent)?;
            data.pause = Pause::AwaitingHost;
            Ok(())
        })?;
        engine.write_bridge(&runtime.bridge)?;
        clear_marker(&marker)?;
        engine.writer.take();
        report(engine, Some(id), "awaiting_host", Some(session), code)
    })();
    if outcome.is_err() {
        if child.stop(Duration::from_secs(5)).is_err() {
            quarantine(engine, child, Some(checkout_guard))?;
            return Err(Error::AmbiguousSurvivor);
        }
        child.join_readers()?;
        if let Some((helper, _)) = engine.quarantine.take() {
            quarantine(engine, helper, Some(checkout_guard))?;
            return Err(Error::AmbiguousSurvivor);
        }
        let data = storage::load(&storage::connection(&engine.root)?)?;
        if data
            .native_active
            .as_ref()
            .is_some_and(|active| active.owner == owner)
        {
            finished(engine, &owner, id, State::Paused, None, None, None)?;
        }
        if marker.exists() {
            clear_marker(&marker)?;
        }
        engine.writer.take();
    }
    outcome
}

fn read_bounded(path: &Path, cap: u64) -> Result<Vec<u8>, Error> {
    let file = fs::File::open(path).map_err(|_| Error::Io)?;
    if file.metadata().map_err(|_| Error::Io)?.len() > cap {
        return Err(Error::Capacity);
    }
    let mut bytes = Vec::new();
    file.take(cap + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Io)?;
    if bytes.len() as u64 > cap {
        return Err(Error::Capacity);
    }
    Ok(bytes)
}
