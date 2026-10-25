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
