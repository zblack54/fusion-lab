use super::{Service, ServiceOptions};
use axum::{
    Router,
    body::Body,
    extract::{Request, State as HttpState},
    http::{StatusCode, header},
    response::{IntoResponse, Response, Sse, sse::Event},
    routing::any,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    convert::Infallible,
    fs,
    io::{self, BufRead, BufReader, Read},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};
use tokio::sync::{Semaphore, mpsc};
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
const RESERVATION: u64 = 256 * 1024 * 1024;
type Shared = Arc<Mutex<State>>;
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    seed: u64,
    duration_seconds: u32,
    scenario: String,
    position_sigma: f64,
    velocity_sigma: f64,
    acceleration_density: f64,
    outage: Option<Outage>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Outage {
    start: u32,
    end: u32,
}
impl Configuration {
    fn valid(&self) -> bool {
        (1..=3600).contains(&self.duration_seconds)
            && matches!(self.scenario.as_str(), "straight" | "maneuver")
            && [
                self.position_sigma,
                self.velocity_sigma,
                self.acceleration_density,
            ]
            .iter()
            .all(|v| v.is_finite() && (0.0..=100.0).contains(v))
            && self
                .outage
                .as_ref()
                .is_none_or(|o| o.start < o.end && o.end <= self.duration_seconds)
    }
}
struct Artifact {
    path: PathBuf,
    leases: AtomicUsize,
    retired: AtomicBool,
    bytes: AtomicU64,
    errors: Arc<Mutex<Vec<String>>>,
}
struct Run {
    id: String,
    config: Configuration,
    state: &'static str,
    sequence: u64,
    samples: Option<u64>,
    error: Option<Value>,
    child: Option<Child>,
    artifact: Arc<Artifact>,
    bytes: u64,
    subscribers: Vec<mpsc::Sender<Value>>,
}
impl Run {
    fn snapshot(&self) -> Value {
        json!({"id":self.id,"config":self.config,"state":self.state,"sequence":self.sequence,"samples":self.samples,"error":self.error,"artifacts":if self.state=="completed" {vec![format!("/v1/runs/{}/artifacts/samples.csv",self.id),format!("/v1/runs/{}/artifacts/report.txt",self.id)]}else{vec![]}})
    }
    fn publish(&mut self) {
        let value = self.snapshot();
        self.subscribers
            .retain(|s| s.try_send(value.clone()).is_ok());
        if self.state != "running" {
            self.subscribers.clear();
        }
    }
}
pub(super) struct State {
    options: ServiceOptions,
    runs: VecDeque<Run>,
    next: u64,
    stopping: bool,
    workers: Vec<thread::JoinHandle<()>>,
    cleanup_errors: Arc<Mutex<Vec<String>>>,
    retired: Vec<Arc<Artifact>>,
    request_slots: Arc<Semaphore>,
    event_slots: Arc<Semaphore>,
    origin: String,
}
fn locked(state: &Shared) -> std::sync::MutexGuard<'_, State> {
    state.lock().unwrap_or_else(|e| e.into_inner())
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
pub(super) fn start(mut options: ServiceOptions) -> io::Result<Service> {
    if !options.bind.ip().is_loopback()
        || !(1..=32).contains(&options.retained_runs)
        || options.artifact_budget_bytes < RESERVATION
    {
        return Err(invalid("invalid local service options"));
    }
    if !options.worker.is_file() {
        return Err(invalid("worker must be an existing executable file"));
    }
    if let Ok(meta) = fs::symlink_metadata(&options.artifact_root)
        && (!meta.is_dir()
            || meta.file_type().is_symlink()
            || reparse(&meta)
            || fs::read_dir(&options.artifact_root)?.next().is_some())
    {
        return Err(invalid("artifact root must be empty and not a link"));
    }
    let worker = fs::canonicalize(&options.worker)?;
    let root_absolute = if options.artifact_root.is_absolute() {
        options.artifact_root.clone()
    } else {
        std::env::current_dir()?.join(&options.artifact_root)
    };
    if worker.starts_with(&root_absolute) {
        return Err(invalid("worker cannot be inside artifact root"));
    }
    let listener = TcpListener::bind(options.bind)?;
    listener.set_nonblocking(true)?;
    let address = listener.local_addr()?;
    let claim = RootClaim::acquire(&options.artifact_root)?;
    fs::create_dir_all(&options.artifact_root)?;
    options.worker = worker;
    options.artifact_root = fs::canonicalize(&options.artifact_root)?;
    let state = Arc::new(Mutex::new(State {
        options,
        runs: VecDeque::new(),
        next: 0,
        stopping: false,
        workers: Vec::new(),
        cleanup_errors: Arc::new(Mutex::new(Vec::new())),
        retired: Vec::new(),
        request_slots: Arc::new(Semaphore::new(32)),
        event_slots: Arc::new(Semaphore::new(32)),
        origin: format!("http://{address}"),
    }));
    let (stop, receive) = tokio::sync::oneshot::channel();
    let shared = state.clone();
    let runtime_thread = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()?;
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener)?;
            let router = Router::new().fallback(any(handle)).with_state(shared);
            crate::transport::serve(listener, router, receive).await
        })
    });
    Ok(Service {
        address,
        state,
        stop: Some(stop),
        thread: Some(runtime_thread),
        claim: Some(claim),
    })
}
/// Atomic sibling marker keeps even an idle, empty root exclusive across processes.
pub(super) struct RootClaim {
    file: Option<fs::File>,
    path: PathBuf,
}
impl RootClaim {
    fn acquire(root: &Path) -> io::Result<Self> {
        let absolute = if root.exists() {
            fs::canonicalize(root)?
        } else if root.is_absolute() {
            root.to_path_buf()
        } else {
            std::env::current_dir()?.join(root)
        };
        let parent = absolute
            .parent()
            .ok_or_else(|| invalid("artifact root needs a parent"))?;
        fs::create_dir_all(parent)?;
        let mut name = std::ffi::OsString::from(".");
        name.push(
            absolute
                .file_name()
                .ok_or_else(|| invalid("artifact root needs a directory name"))?,
        );
        name.push(".fusion-service.lock");
        let path = fs::canonicalize(parent)?.join(name);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("artifact root claim {}: {error}", path.display()),
                )
            })?;
        use std::io::Write;
        if let Err(error) = writeln!(
            file,
            "Fusion Lab process {}; remove only after inspecting the stopped instance",
            std::process::id()
        ) {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        Ok(Self {
            file: Some(file),
            path,
        })
    }
    pub(super) fn release(&mut self) -> io::Result<()> {
        drop(self.file.take());
        if self.path.as_os_str().is_empty() {
            return Ok(());
        }
        fs::remove_file(&self.path)?;
        self.path = PathBuf::new();
        Ok(())
    }
}
impl Drop for RootClaim {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
#[cfg(windows)]
fn reparse(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn reparse(_: &fs::Metadata) -> bool {
    false
}
fn response(status: u16, value: Value) -> Response {
    let bytes = value.to_string();
    (
        StatusCode::from_u16(status).unwrap(),
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CONTENT_LENGTH, &bytes.len().to_string()),
        ],
        bytes,
    )
        .into_response()
}
fn error(status: u16, code: &str) -> Response {
    response(status, json!({"error":{"code":code,"message":code}}))
}
async fn handle(HttpState(state): HttpState<Shared>, request: Request) -> Response {
    let slots = locked(&state).request_slots.clone();
    let Ok(permit) = slots.try_acquire_owned() else {
        return error(503, "capacity");
    };
    let method = request.method().as_str().to_owned();
    let path = request.uri().path().to_owned();
    let headers = request.headers().clone();
    if matches!(method.as_str(), "POST" | "DELETE")
        && let Some(origin) = headers.get(header::ORIGIN)
        && origin.to_str().ok() != Some(locked(&state).origin.as_str())
    {
        return error(403, "invalid_origin");
    }
    let body = match tokio::time::timeout(
        Duration::from_secs(5),
        axum::body::to_bytes(request.into_body(), 4096),
    )
    .await
    {
        Ok(Ok(body)) => body,
        Ok(Err(_)) => return error(413, "body_too_large"),
        Err(_) => return error(408, "request_timeout"),
    };
    if path == "/v1/health" && method == "GET" {
        let s = locked(&state);
        return response(
            200,
            json!({"status":"ok","capacity":{"active_limit":1,"active":s.runs.iter().filter(|r|r.state=="running").count(),"job_memory_reservation_bytes":RESERVATION,"service_memory_reservation_bytes":RESERVATION,"artifact_reservation_bytes":RESERVATION},"shutting_down":s.stopping}),
        );
    }
    if path == "/v1/health" {
        return error(405, "method_not_allowed");
    }
    if path == "/v1/runs" {
        return match method.as_str() {
            "GET" => response(
                200,
                json!(
                    locked(&state)
                        .runs
                        .iter()
                        .map(Run::snapshot)
                        .collect::<Vec<_>>()
                ),
            ),
            "POST" => {
                let raw: Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => return error(400, "invalid_json"),
                };
                if !raw.as_object().is_some_and(|v| v.contains_key("outage")) {
                    return error(422, "invalid_config");
                }
                let config: Configuration = match serde_json::from_value(raw) {
                    Ok(c) => c,
                    Err(_) => return error(422, "invalid_config"),
                };
                if !config.valid() {
                    return error(422, "invalid_config");
                }
                launch(&state, config)
            }
            _ => error(405, "method_not_allowed"),
        };
    }
    let segments: Vec<_> = path.split('/').collect();
    if segments.len() < 4
        || segments[1] != "v1"
        || segments[2] != "runs"
        || !segments[3]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return error(404, "not_found");
    }
    let id = segments[3];
    if segments.len() == 4 {
        let mut s = locked(&state);
        let Some(run) = s.runs.iter_mut().find(|r| r.id == id) else {
            return error(404, "not_found");
        };
        return match method.as_str() {
            "GET" => response(200, run.snapshot()),
            "DELETE" => {
                if run.state == "cancelled" {
                    response(200, run.snapshot())
                } else if run.state != "running" {
                    error(409, "terminal_state")
                } else {
                    match cancel(run) {
                        Ok(()) => response(200, run.snapshot()),
                        Err(_) => error(500, "cancellation_failed"),
                    }
                }
            }
            _ => error(405, "method_not_allowed"),
        };
    }
    if segments.len() == 5 && segments[4] == "events" && method == "GET" {
        let (receiver, event_permit) = {
            let mut s = locked(&state);
            let event_slots = s.event_slots.clone();
            let Some(run) = s.runs.iter_mut().find(|r| r.id == id) else {
                return error(404, "not_found");
            };
            if let Some(cursor) = headers.get("last-event-id")
                && cursor
                    .to_str()
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .is_none_or(|v| v > run.sequence)
            {
                return error(400, "invalid_event_id");
            }
            let Ok(event_permit) = event_slots.try_acquire_owned() else {
                return error(429, "capacity");
            };
            let (tx, rx) = mpsc::channel(16);
            let _ = tx.try_send(run.snapshot());
            if run.state == "running" {
                run.subscribers.retain(|sender| !sender.is_closed());
                run.subscribers.push(tx);
            }
            (rx, event_permit)
        };
        drop(permit);
        let stream = ReceiverStream::new(receiver).map(move |v| {
            let _lease = &event_permit;
            Ok::<_, Infallible>(
                Event::default()
                    .event("snapshot")
                    .id(v["sequence"].as_u64().unwrap().to_string())
                    .data(v.to_string()),
            )
        });
        return Sse::new(stream)
            .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
            .into_response();
    }
    if segments.len() == 6
        && segments[4] == "artifacts"
        && method == "GET"
        && matches!(segments[5], "samples.csv" | "report.txt")
    {
        let lease = {
            let s = locked(&state);
            let Some(run) = s.runs.iter().find(|r| r.id == id) else {
                return error(404, "not_found");
            };
            if run.state != "completed" {
                return error(409, "artifact_unavailable");
            }
            ArtifactLease::acquire(run.artifact.clone())
        };
        let name = segments[5].to_owned();
        // The blocking operation owns the guard, including if this request is dropped.
        // Returning a prepared file to a canceled JoinHandle drops it and its guard.
        let (prepared, permit) = match tokio::task::spawn_blocking(move || -> io::Result<_> {
            Ok((PreparedArtifact::open(lease, &name)?, permit))
        })
        .await
        {
            Ok(Ok(prepared)) => prepared,
            _ => return error(500, "artifact_failed"),
        };
        let length = prepared.length;
        let (sender, receiver) = mpsc::channel(1);
        let reader = thread::spawn(move || prepared.send(sender));
        locked(&state).workers.push(reader);
        let stream = ReceiverStream::new(receiver).map(move |chunk| {
            let _permit = &permit;
            chunk
        });
        let mut reply = Response::new(Body::from_stream(stream));
        reply
            .headers_mut()
            .insert(header::CONTENT_LENGTH, length.into());
        reply.headers_mut().insert(
            header::CONTENT_TYPE,
            if segments[5] == "samples.csv" {
                "text/csv"
            } else {
                "text/plain"
            }
            .parse()
            .unwrap(),
        );
        return reply;
    }
    if (segments.len() == 5 && segments[4] == "events")
        || (segments.len() == 6
            && segments[4] == "artifacts"
            && matches!(segments[5], "samples.csv" | "report.txt"))
    {
        return error(405, "method_not_allowed");
    }
    error(404, "not_found")
}
fn launch(state: &Shared, config: Configuration) -> Response {
    let mut s = locked(state);
    collect_retired(&mut s);
    if !s
        .cleanup_errors
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .is_empty()
    {
        return error(507, "cleanup_failed");
    }
    if s.stopping {
        return error(503, "shutting_down");
    }
    if s.runs.iter().any(|r| r.state == "running") {
        let mut reply = error(409, "busy");
        reply
            .headers_mut()
            .insert(header::RETRY_AFTER, "1".parse().unwrap());
        return reply;
    }
    for worker in std::mem::take(&mut s.workers) {
        if worker.is_finished() {
            if worker.join().is_err() {
                record(&s.cleanup_errors, "worker monitor panicked".to_owned());
            }
        } else {
            s.workers.push(worker);
        }
    }
    while s
        .runs
        .iter()
        .map(|r| r.bytes)
        .sum::<u64>()
        .saturating_add(
            s.retired
                .iter()
                .map(|a| a.bytes.load(Ordering::SeqCst))
                .sum::<u64>(),
        )
        .saturating_add(RESERVATION)
        > s.options.artifact_budget_bytes
    {
        if !evict(&mut s) {
            return error(507, "insufficient_storage");
        }
    }
    s.next += 1;
    let id = format!("run-{}", s.next);
    let path = s.options.artifact_root.join(&id);
    if fs::create_dir(&path).is_err() {
        return error(507, "insufficient_storage");
    }
    let output = path.join("samples.csv");
    let mut command = Command::new(&s.options.worker);
    command
        .args([
            "--seed",
            &config.seed.to_string(),
            "--duration",
            &config.duration_seconds.to_string(),
            "--scenario",
            &config.scenario,
            "--position-sigma",
            &config.position_sigma.to_string(),
            "--velocity-sigma",
            &config.velocity_sigma.to_string(),
            "--acceleration-density",
            &config.acceleration_density.to_string(),
            "--output",
        ])
        .arg(&output);
    if let Some(outage) = &config.outage {
        command.args([
            "--outage-start",
            &outage.start.to_string(),
            "--outage-end",
            &outage.end.to_string(),
        ]);
    } else {
        command.arg("--no-outage");
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(_) => {
            if let Err(e) = fs::remove_dir_all(&path) {
                record(&s.cleanup_errors, e.to_string());
            }
            return error(500, "worker_start_failed");
        }
    };
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let pipes = [
        thread::spawn(move || drain(stdout)),
        thread::spawn(move || drain(stderr)),
    ];
    let run = Run {
        id: id.clone(),
        config,
        state: "running",
        sequence: 0,
        samples: None,
        error: None,
        child: Some(child),
        artifact: Arc::new(Artifact {
            path,
            leases: AtomicUsize::new(0),
            retired: AtomicBool::new(false),
            bytes: AtomicU64::new(0),
            errors: s.cleanup_errors.clone(),
        }),
        bytes: 0,
        subscribers: Vec::new(),
    };
    let snapshot = run.snapshot();
    s.runs.push_back(run);
    let shared = state.clone();
    s.workers
        .push(thread::spawn(move || monitor(shared, id, pipes)));
    response(202, snapshot)
}
fn drain(mut pipe: impl Read) -> io::Result<Vec<u8>> {
    let mut tail = VecDeque::with_capacity(65536);
    let mut bytes = [0; 8192];
    loop {
        let n = pipe.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        for b in &bytes[..n] {
            if tail.len() == 65536 {
                tail.pop_front();
            }
            tail.push_back(*b);
        }
    }
    Ok(tail.into())
}
fn directory_bytes(path: &Path) -> io::Result<u64> {
    fs::read_dir(path)?.try_fold(0u64, |total, e| {
        Ok(total.saturating_add(e?.metadata()?.len()))
    })
}
fn monitor(state: Shared, id: String, pipes: [thread::JoinHandle<io::Result<Vec<u8>>>; 2]) {
    loop {
        let done = {
            let mut s = locked(&state);
            let mut done = true;
            if let Some(run) = s.runs.iter_mut().find(|r| r.id == id)
                && run.state == "running"
            {
                match run.child.as_mut().unwrap().try_wait() {
                    Ok(Some(status)) => {
                        run.child.take();
                        if directory_bytes(&run.artifact.path)
                            .is_ok_and(|bytes| bytes > RESERVATION)
                        {
                            fail(run, "artifact_limit");
                        } else if status.success() {
                            match validate_artifacts(run) {
                                Ok(samples) => {
                                    run.samples = Some(samples);
                                    run.state = "completed";
                                    run.bytes =
                                        directory_bytes(&run.artifact.path).unwrap_or(RESERVATION);
                                    run.artifact.bytes.store(run.bytes, Ordering::SeqCst);
                                }
                                Err(_) => fail(run, "artifact_failed"),
                            }
                        } else {
                            fail(run, "worker_failed");
                        }
                        run.sequence += 1;
                        run.publish();
                    }
                    Ok(None) => {
                        let disk = directory_bytes(&run.artifact.path);
                        if !disk.as_ref().is_ok_and(|bytes| *bytes <= RESERVATION) {
                            if let Err(error) = terminate(run) {
                                run.error = Some(
                                    json!({"code":"cancellation_failed","message":"cancellation_failed"}),
                                );
                                record(&run.artifact.errors, error.to_string());
                                done = false;
                            } else {
                                fail(
                                    run,
                                    if disk.is_err() {
                                        "artifact_failed"
                                    } else {
                                        "artifact_limit"
                                    },
                                );
                                run.sequence += 1;
                                run.publish();
                            }
                        } else {
                            done = false;
                        }
                    }
                    Err(_) => {
                        run.error = Some(
                            json!({"code":"worker_wait_failed","message":"worker_wait_failed"}),
                        );
                        done = false;
                    }
                }
            }
            let count = s.runs.iter().filter(|r| r.state != "running").count();
            if count > s.options.retained_runs {
                evict(&mut s);
            }
            done
        };
        if done {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    for pipe in pipes {
        match pipe.join() {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => record(&locked(&state).cleanup_errors, e.to_string()),
            Err(_) => record(
                &locked(&state).cleanup_errors,
                "worker pipe reader panicked".to_owned(),
            ),
        }
    }
}
fn validate_artifacts(run: &Run) -> io::Result<u64> {
    let file = fs::File::open(run.artifact.path.join("samples.csv"))?;
    let mut reader = BufReader::new(file);
    let header = bounded_line(&mut reader)?.ok_or_else(|| invalid("missing CSV header"))?;
    if !header.starts_with("tick,time_s,") || header.split(',').count() != 35 {
        return Err(invalid("invalid CSV header"));
    }
    let mut count = 0;
    while let Some(row) = bounded_line(&mut reader)? {
        if row.split(',').count() != 35 {
            return Err(invalid("invalid CSV row"));
        }
        count += 1;
    }
    if count != u64::from(run.config.duration_seconds) * 100 + 1 {
        return Err(invalid("invalid sample count"));
    }
    let report = run.artifact.path.join("samples.txt");
    fs::rename(report, run.artifact.path.join("report.txt"))?;
    let mut report = fs::File::open(run.artifact.path.join("report.txt"))?;
    let mut first = [0u8; 1];
    if report.read(&mut first)? == 0 {
        return Err(invalid("empty report"));
    }
    if fs::metadata(run.artifact.path.join("report.txt"))?.len() == 0
        || directory_bytes(&run.artifact.path)? > RESERVATION
    {
        return Err(invalid("invalid report or artifact size"));
    }
    Ok(count)
}
fn bounded_line(reader: &mut BufReader<fs::File>) -> io::Result<Option<String>> {
    let mut line = String::new();
    let size = reader.by_ref().take(8193).read_line(&mut line)?;
    if size > 8192 {
        return Err(invalid("oversized CSV row"));
    }
    Ok((size != 0).then_some(line))
}
fn fail(run: &mut Run, code: &str) {
    run.state = "failed";
    run.error = Some(json!({"code":code,"message":code}));
    if let Err(e) = fs::remove_dir_all(&run.artifact.path) {
        record(&run.artifact.errors, e.to_string());
        run.error = Some(json!({"code":code,"message":code,"cleanup_error":"cleanup_failed"}));
    }
}
fn cancel(run: &mut Run) -> io::Result<()> {
    terminate(run)?;
    run.state = "cancelled";
    run.sequence += 1;
    if let Err(e) = fs::remove_dir_all(&run.artifact.path) {
        record(&run.artifact.errors, e.to_string());
        run.error = Some(json!({"code":"cleanup_failed","message":"cleanup_failed"}));
        run.publish();
        return Err(e);
    }
    run.publish();
    Ok(())
}
fn terminate(run: &mut Run) -> io::Result<()> {
    if let Some(child) = run.child.as_mut() {
        match child.try_wait()? {
            Some(_) => {}
            None => {
                child.kill()?;
                child.wait()?;
            }
        }
        run.child.take();
    }
    Ok(())
}
fn evict(state: &mut State) -> bool {
    let Some(index) = state.runs.iter().position(|r| r.state != "running") else {
        return false;
    };
    let run = state.runs.remove(index).unwrap();
    run.artifact.retired.store(true, Ordering::SeqCst);
    if run.artifact.leases.load(Ordering::SeqCst) == 0
        && run.artifact.path.exists()
        && let Err(e) = fs::remove_dir_all(&run.artifact.path)
    {
        record(&state.cleanup_errors, e.to_string());
    }
    if run.artifact.path.exists() || run.artifact.leases.load(Ordering::SeqCst) > 0 {
        state.retired.push(run.artifact);
    }
    true
}
pub(super) fn stop(state: &Shared) -> io::Result<()> {
    let mut s = locked(state);
    s.stopping = true;
    for run in &mut s.runs {
        if run.state == "running" {
            cancel(run)?;
        }
        run.subscribers.clear();
    }
    Ok(())
}
pub(super) fn join_workers(state: &Shared) -> io::Result<()> {
    let workers = std::mem::take(&mut locked(state).workers);
    for worker in workers {
        worker
            .join()
            .map_err(|_| io::Error::other("worker monitor panicked"))?;
    }
    Ok(())
}
pub(super) fn cleanup(state: &Shared) -> io::Result<()> {
    let mut s = locked(state);
    while evict(&mut s) {}
    collect_retired(&mut s);
    let errors = s.cleanup_errors.lock().unwrap_or_else(|e| e.into_inner());
    if errors.is_empty() {
        Ok(())
    } else {
        Err(io::Error::other(errors.join("; ")))
    }
}
/// Held before any async boundary; releasing the last retired guard owns cleanup.
struct ArtifactLease {
    artifact: Arc<Artifact>,
}
impl ArtifactLease {
    fn acquire(artifact: Arc<Artifact>) -> Self {
        artifact.leases.fetch_add(1, Ordering::SeqCst);
        Self { artifact }
    }
}
impl Drop for ArtifactLease {
    fn drop(&mut self) {
        if self.artifact.leases.fetch_sub(1, Ordering::SeqCst) == 1
            && self.artifact.retired.load(Ordering::SeqCst)
            && self.artifact.path.exists()
            && let Err(error) = fs::remove_dir_all(&self.artifact.path)
        {
            record(&self.artifact.errors, error.to_string());
        }
    }
}
/// File is closed explicitly before releasing the lease on every return/drop path.
struct PreparedArtifact {
    file: Option<fs::File>,
    lease: ArtifactLease,
    length: u64,
}
impl PreparedArtifact {
    fn open(lease: ArtifactLease, name: &str) -> io::Result<Self> {
        let mut prepared = Self {
            file: None,
            lease,
            length: 0,
        };
        prepared.file = Some(fs::File::open(prepared.lease.artifact.path.join(name))?);
        prepared.length = prepared
            .file
            .as_ref()
            .expect("file assigned above")
            .metadata()?
            .len();
        Ok(prepared)
    }
    fn send(mut self, sender: mpsc::Sender<Result<axum::body::Bytes, io::Error>>) {
        let mut bytes = [0u8; 65536];
        loop {
            match self
                .file
                .as_mut()
                .expect("prepared artifact owns an open file")
                .read(&mut bytes)
            {
                Ok(0) => break,
                Ok(size) => {
                    if sender
                        .blocking_send(Ok(axum::body::Bytes::copy_from_slice(&bytes[..size])))
                        .is_err()
                    {
                        break;
                    }
                }
                Err(error) => {
                    record(&self.lease.artifact.errors, error.to_string());
                    let _ = sender.blocking_send(Err(io::Error::other("artifact read failed")));
                    break;
                }
            }
        }
    }
}
impl Drop for PreparedArtifact {
    fn drop(&mut self) {
        drop(self.file.take());
    }
}
fn record(errors: &Arc<Mutex<Vec<String>>>, message: String) {
    let mut errors = errors.lock().unwrap_or_else(|e| e.into_inner());
    if errors.len() < 32 {
        errors.push(message);
    }
}
fn collect_retired(state: &mut State) {
    state
        .retired
        .retain(|artifact| artifact.path.exists() || artifact.leases.load(Ordering::SeqCst) > 0);
}

#[cfg(test)]
mod regression_tests;
