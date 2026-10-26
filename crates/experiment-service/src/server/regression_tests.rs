//! Ownership regressions supplementary to the immutable public HTTP contract.
use super::*;
use std::sync::atomic::AtomicU64;
static NEXT: AtomicU64 = AtomicU64::new(0);
fn root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "fusion-ownership-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ))
}
fn artifact() -> Arc<Artifact> {
    let path = root();
    fs::create_dir(&path).unwrap();
    fs::write(path.join("samples.csv"), vec![b'x'; 3 * 65536]).unwrap();
    Arc::new(Artifact {
        path,
        leases: AtomicUsize::new(0),
        retired: AtomicBool::new(false),
        bytes: AtomicU64::new(3 * 65536),
        errors: Arc::new(Mutex::new(Vec::new())),
    })
}
fn options(root: &Path) -> ServiceOptions {
    ServiceOptions {
        bind: "127.0.0.1:0".parse().unwrap(),
        artifact_root: root.to_path_buf(),
        worker: std::env::current_exe().unwrap(),
        retained_runs: 2,
        artifact_budget_bytes: 3 * RESERVATION,
    }
}

#[test]
fn same_empty_root_is_exclusively_claimed_and_released() {
    let path = root();
    let first = Service::start(options(&path)).unwrap();
    assert_eq!(fs::read_dir(&path).unwrap().count(), 0);
    let second = Service::start(options(&path));
    assert!(
        matches!(second,Err(ref error) if error.kind()==io::ErrorKind::AlreadyExists),
        "second instance must fail the atomic claim even while first has no runs"
    );
    assert_eq!(
        fs::read_dir(&path).unwrap().count(),
        0,
        "claim is outside the artifact root and rejected startup leaves the first root untouched"
    );
    first.shutdown().unwrap();
    let replacement = Service::start(options(&path)).unwrap();
    replacement.shutdown().unwrap();
    fs::remove_dir(path).unwrap();
}

#[test]
fn cancelled_pending_open_releases_its_owned_lease() {
    let artifact = artifact();
    let lease = ArtifactLease::acquire(artifact.clone());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let blocker = runtime.spawn_blocking(move || {
        started_tx.send(()).unwrap();
        release_rx.recv().unwrap();
    });
    started_rx.recv().unwrap();
    let pending = runtime.spawn_blocking(move || PreparedArtifact::open(lease, "samples.csv"));
    assert_eq!(
        artifact.leases.load(Ordering::SeqCst),
        1,
        "the queued operation owns its reservation"
    );
    artifact.retired.store(true, Ordering::SeqCst);
    pending.abort();
    release_tx.send(()).unwrap();
    runtime.block_on(async {
        blocker.await.unwrap();
        match pending.await {
            Err(error) => assert!(error.is_cancelled()),
            Ok(_) => panic!("blocked file operation must have been cancelled before starting"),
        }
    });
    assert_eq!(
        artifact.leases.load(Ordering::SeqCst),
        0,
        "cancelled async request cannot leave a phantom reservation"
    );
    assert!(!artifact.path.exists());
    assert!(artifact.errors.lock().unwrap().is_empty());
}

#[test]
fn failed_open_and_prepared_file_drop_release_without_handle_leaks() {
    let first = artifact();
    first.retired.store(true, Ordering::SeqCst);
    assert!(PreparedArtifact::open(ArtifactLease::acquire(first.clone()), "missing.csv").is_err());
    assert_eq!(first.leases.load(Ordering::SeqCst), 0);
    assert!(!first.path.exists());
    let second = artifact();
    let prepared =
        PreparedArtifact::open(ArtifactLease::acquire(second.clone()), "samples.csv").unwrap();
    second.retired.store(true, Ordering::SeqCst);
    drop(prepared);
    assert_eq!(second.leases.load(Ordering::SeqCst), 0);
    assert!(
        !second.path.exists(),
        "the open file must close before retired directory deletion, including Windows"
    );
    assert!(second.errors.lock().unwrap().is_empty());
}

#[test]
fn dropped_stream_receiver_unblocks_reader_and_releases_file_before_cleanup() {
    let artifact = artifact();
    let prepared =
        PreparedArtifact::open(ArtifactLease::acquire(artifact.clone()), "samples.csv").unwrap();
    let (sender, mut receiver) = mpsc::channel(1);
    let reader = thread::spawn(move || prepared.send(sender));
    assert_eq!(receiver.blocking_recv().unwrap().unwrap().len(), 65536);
    artifact.retired.store(true, Ordering::SeqCst);
    drop(receiver);
    reader.join().unwrap();
    assert_eq!(artifact.leases.load(Ordering::SeqCst), 0);
    assert!(!artifact.path.exists());
    assert!(artifact.errors.lock().unwrap().is_empty());
}

fn probe_worker() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let build = Command::new("cargo")
        .args([
            "build",
            "--locked",
            "--example",
            "probe_worker",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "fixture build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let metadata = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(metadata.status.success());
    let metadata: Value = serde_json::from_slice(&metadata.stdout).unwrap();
    PathBuf::from(metadata["target_directory"].as_str().unwrap())
        .join("debug/examples")
        .join(if cfg!(windows) {
            "probe_worker.exe"
        } else {
            "probe_worker"
        })
}

#[test]
fn noisy_child_stdout_and_stderr_are_drained_with_bounded_tails() {
    // Actual OS pipes and two MiB per pipe establish subprocess drain behavior.
    let mut child = Command::new(probe_worker())
        .args(["--seed", "9002", "--output"])
        .arg(std::env::temp_dir().join("unused-fusion-pipe-probe.csv"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let output = thread::spawn(move || drain(stdout));
    let errors = thread::spawn(move || drain(stderr));
    assert_eq!(child.wait().unwrap().code(), Some(7));
    let output = output.join().unwrap().unwrap();
    let errors = errors.join().unwrap().unwrap();
    assert_eq!(output.len(), 65536);
    assert!(output.iter().all(|byte| *byte == b'a'));
    assert_eq!(errors.len(), 65536);
    assert!(errors.iter().all(|byte| *byte == b'b'));
}

fn http(address: std::net::SocketAddr, method: &str, path: &str, body: &Value) -> (u16, Value) {
    use std::io::Write;
    let mut socket = std::net::TcpStream::connect(address).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let body = body.to_string();
    write!(socket,"{method} {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
    let mut response = String::new();
    socket.read_to_string(&mut response).unwrap();
    let (status, body) = response.split_once("\r\n\r\n").unwrap();
    (
        status.split_whitespace().nth(1).unwrap().parse().unwrap(),
        serde_json::from_str(body).unwrap(),
    )
}

#[test]
fn injected_wait_failure_is_visible_and_keeps_the_real_child_owned() {
    let root = root();
    let mut settings = options(&root);
    settings.worker = probe_worker();
    let service = Service::start(settings).unwrap();
    let configuration = json!({"seed":9001,"duration_seconds":3600,"scenario":"straight","position_sigma":0,"velocity_sigma":0,"acceleration_density":0.1,"outage":null});
    let (status, submitted) = http(service.address(), "POST", "/v1/runs", &configuration);
    assert_eq!(status, 202);
    let id = submitted["id"].as_str().unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let marker = root.join(id).join("worker.ready");
    let ready: Value = loop {
        if let Ok(text) = fs::read_to_string(&marker)
            && let Ok(value) = serde_json::from_str(&text)
        {
            break value;
        }
        assert!(std::time::Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    };
    let child_address: std::net::SocketAddr = ready["address"].as_str().unwrap().parse().unwrap();
    assert!(std::net::TcpStream::connect(child_address).is_ok());
    {
        let mut state = locked(&service.state);
        let run = state.runs.iter_mut().find(|run| run.id == id).unwrap();
        run.wait_fault = true;
    }
    loop {
        let (status, snapshot) = http(
            service.address(),
            "GET",
            &format!("/v1/runs/{id}"),
            &Value::Null,
        );
        assert_eq!(status, 200);
        if snapshot["error"]["code"] == "worker_wait_failed" {
            assert_eq!(snapshot["state"], "running");
            assert!(snapshot["sequence"].as_u64().unwrap() > 0);
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        http(service.address(), "POST", "/v1/runs", &configuration).0,
        409
    );
    {
        let state = locked(&service.state);
        assert_eq!(state.runs.len(), 1);
        assert!(
            state.runs[0].child.is_some(),
            "failed wait cannot discard the live Child owner"
        );
    }
    assert!(std::net::TcpStream::connect(child_address).is_ok());
    service.shutdown().unwrap();
    assert!(
        std::net::TcpStream::connect(child_address).is_err(),
        "shutdown still kills and reaps the retained real child"
    );
    fs::remove_dir(root).unwrap();
}
#[test]
fn worker_join_failure_still_observes_all_remaining_owned_threads() {
    let path = root();
    let service = Service::start(options(&path)).unwrap();
    let first = thread::spawn(|| panic!("injected first worker panic"));
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !first.is_finished() {
        assert!(std::time::Instant::now() < deadline);
        thread::yield_now();
    }
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let (finished_tx, finished_rx) = std::sync::mpsc::channel();
    let second = thread::spawn(move || {
        ready_tx.send(()).unwrap();
        release_rx.recv().unwrap();
        finished_tx.send(()).unwrap();
    });
    ready_rx.recv().unwrap();
    {
        let mut state = locked(&service.state);
        state.workers.extend([first, second]);
    }
    let shared = service.state.clone();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (result_tx, result_rx) = std::sync::mpsc::channel();
    let joiner = thread::spawn(move || {
        entered_tx.send(()).unwrap();
        result_tx.send(join_workers(&shared)).unwrap();
    });
    entered_rx.recv().unwrap();
    let premature = result_rx.recv_timeout(Duration::from_millis(100));
    // Clean up the deliberately stalled worker even when testing the broken
    // implementation: the failing regression must not leave its fixture alive.
    release_tx.send(()).unwrap();
    finished_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let result = match premature {
        Ok(result) => Some(result),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => None,
        Err(error) => panic!("join result channel failed: {error}"),
    };
    let returned_early = result.is_some();
    let result = result.unwrap_or_else(|| result_rx.recv_timeout(Duration::from_secs(5)).unwrap());
    joiner.join().unwrap();
    service.shutdown().unwrap();
    fs::remove_dir(path).unwrap();
    assert!(
        !returned_early,
        "join_workers returned after the first failure while another owned worker was still blocked"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("worker monitor panicked")
    );
}
