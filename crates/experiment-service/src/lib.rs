//! Bounded loopback experiment HTTP adapter around the synchronous Fusion Lab CLI.
mod server;
mod transport;
use std::{
    io,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
/// Process and filesystem ownership for a local service instance.
pub struct ServiceOptions {
    /// Loopback listener; port zero selects an unused local port.
    pub bind: SocketAddr,
    /// Empty private directory, or a nonexistent path to create.
    pub artifact_root: PathBuf,
    /// Existing numerical CLI executable, invoked without a shell.
    pub worker: PathBuf,
    /// Maximum terminal run retention, between one and 32.
    pub retained_runs: usize,
    /// Artifact accounting capacity including an active 256 MiB reservation.
    pub artifact_budget_bytes: u64,
}
/// Owns the HTTP runtime, child lifecycle and private artifacts.
pub struct Service {
    address: SocketAddr,
    state: Arc<Mutex<server::State>>,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<JoinHandle<io::Result<()>>>,
    claim: Option<server::RootClaim>,
}
impl Service {
    /// Validate options before creating files or listening, then start the adapter.
    pub fn start(options: ServiceOptions) -> io::Result<Self> {
        server::start(options)
    }
    /// Bound address, including any assigned ephemeral port.
    pub fn address(&self) -> SocketAddr {
        self.address
    }
    /// Stop admission, reap workers, close clients and report cleanup failures.
    pub fn shutdown(mut self) -> io::Result<()> {
        self.finish()
    }
    fn finish(&mut self) -> io::Result<()> {
        let cancellation = server::stop(&self.state);
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        let serving = match self.thread.take() {
            Some(thread) => match thread.join() {
                Ok(result) => result,
                Err(_) => Err(io::Error::other("HTTP runtime thread panicked")),
            },
            None => Ok(()),
        };
        let workers = server::join_workers(&self.state);
        let cleanup = server::cleanup(&self.state);
        let claim = match self.claim.as_mut() {
            Some(claim) => claim.release(),
            None => Ok(()),
        };
        cancellation
            .and(serving)
            .and(workers)
            .and(cleanup)
            .and(claim)
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

#[cfg(test)]
mod shutdown_tests {
    use super::{Service, ServiceOptions};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        thread,
    };
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    fn options(root: PathBuf) -> ServiceOptions {
        ServiceOptions {
            bind: "127.0.0.1:0".parse().unwrap(),
            artifact_root: root,
            worker: std::env::current_exe().unwrap(),
            retained_runs: 1,
            artifact_budget_bytes: 512 * 1024 * 1024,
        }
    }
    #[test]
    fn runtime_panic_does_not_skip_explicit_cleanup_or_root_release() {
        let root = std::env::temp_dir().join(format!(
            "fusion-runtime-panic-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::SeqCst)
        ));
        let mut service = Service::start(options(root.clone())).unwrap();
        service.stop.take().unwrap().send(()).unwrap();
        service.thread.take().unwrap().join().unwrap().unwrap();
        service.thread = Some(thread::spawn(|| panic!("injected HTTP runtime panic")));
        let error = service.finish().unwrap_err();
        assert!(error.to_string().contains("HTTP runtime thread panicked"));
        // Keep the original handle alive: its destructor must not be required
        // for explicit fallible completion to release the exclusive root claim.
        let replacement = Service::start(options(root.clone())).unwrap();
        replacement.shutdown().unwrap();
        drop(service);
        fs::remove_dir(root).unwrap();
    }
}
