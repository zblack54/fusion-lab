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
            Some(thread) => thread
                .join()
                .map_err(|_| io::Error::other("HTTP runtime thread panicked"))?,
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
