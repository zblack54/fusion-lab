//! Windows membership-based containment. No PID liveness heuristic is used.
use crate::Error;
use std::{
    process::{Child, Command, ExitStatus},
    thread,
    time::{Duration, Instant},
};
use subc_jobobject::JobObject;

pub(crate) fn root() -> Result<(), Error> {
    subc_jobobject::ensure_root_job().map_err(|_| Error::Io)
}
pub(crate) struct Owned {
    pub child: Child,
    pub job: JobObject,
    pub readers: Vec<std::thread::JoinHandle<()>>,
}
pub(crate) struct StartError {
    pub error: Error,
    pub retained: Option<Owned>,
}
impl Owned {
    /// Retain both handles through assignment and resume; no convenience cleanup path.
    pub fn start(command: &mut Command) -> Result<Self, StartError> {
        root().map_err(|error| StartError {
            error,
            retained: None,
        })?;
        let job = JobObject::new().map_err(|_| StartError {
            error: Error::Io,
            retained: None,
        })?;
        subc_jobobject::suspend_on_create(command);
        let child = command.spawn().map_err(|_| StartError {
            error: Error::Io,
            retained: None,
        })?;
        let mut owned = Self {
            child,
            job,
            readers: vec![],
        };
        let setup = owned
            .job
            .assign(&owned.child)
            .and_then(|()| subc_jobobject::resume_main_thread(owned.child.id()));
        if setup.is_err() {
            if owned.stop(Duration::from_secs(5)).is_err() {
                return Err(StartError {
                    error: Error::AmbiguousSurvivor,
                    retained: Some(owned),
                });
            }
            return Err(StartError {
                error: Error::Io,
                retained: None,
            });
        }
        Ok(owned)
    }
    pub fn join_readers(&mut self) -> Result<(), Error> {
        let mut failed = false;
        for reader in self.readers.drain(..) {
            if reader.join().is_err() {
                failed = true;
            }
        }
        if failed { Err(Error::Io) } else { Ok(()) }
    }
    pub fn poll(&mut self) -> Result<Option<ExitStatus>, Error> {
        self.child.try_wait().map_err(|_| Error::Io)
    }
    /// Root reaping AND zero job membership are mandatory before releasing ownership.
    pub fn clean(&mut self, deadline: Duration) -> Result<ExitStatus, Error> {
        let until = Instant::now() + deadline;
        loop {
            let status = self.poll()?;
            let count = self
                .job
                .process_count()
                .map_err(|_| Error::AmbiguousSurvivor)?;
            if let Some(status) = status
                && count == 0
            {
                return Ok(status);
            }
            if Instant::now() >= until {
                return Err(Error::AmbiguousSurvivor);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn stop(&mut self, deadline: Duration) -> Result<ExitStatus, Error> {
        // An assignment failure can leave the suspended root outside the narrower job.
        // The outer root job still owns it; retain the direct Child while terminating.
        let job_result = self.job.terminate();
        if self.poll()?.is_none() && self.child.kill().is_err() && self.poll()?.is_none() {
            return Err(Error::AmbiguousSurvivor);
        }
        let status = self.clean(deadline)?;
        if job_result.is_err()
            && self
                .job
                .process_count()
                .map_err(|_| Error::AmbiguousSurvivor)?
                != 0
        {
            return Err(Error::AmbiguousSurvivor);
        }
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::{
        fs,
        net::{SocketAddr, TcpListener, TcpStream},
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fn dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "fusion-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
    fn fixture(role: &str, path: &Path) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                format!("native::tests::fixture_{role}"),
                "--exact".into(),
                "--nocapture".into(),
            ])
            .env("FUSION_NATIVE_FIXTURE", role)
            .env("FUSION_NATIVE_DIR", path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        command
    }
    fn ready(path: &Path) -> SocketAddr {
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(text) = fs::read_to_string(path.join("leaf.ready"))
                && let Ok(address) = text.parse()
            {
                return address;
            }
            assert!(Instant::now() < until, "native leaf readiness timed out");
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn socket_closed(address: SocketAddr) {
        let until = Instant::now() + Duration::from_secs(5);
        while TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok() {
            assert!(
                Instant::now() < until,
                "terminated fixture endpoint must close within deadline"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    // These three fixture entrypoints are selected in actual separate test processes.
    // Their normal no-env invocation is not counted as containment evidence.
    #[test]
    fn fixture_leaf() {
        if std::env::var("FUSION_NATIVE_FIXTURE").ok().as_deref() != Some("leaf") {
            return;
        }
        let path = PathBuf::from(std::env::var_os("FUSION_NATIVE_DIR").unwrap());
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        fs::write(
            path.join("leaf.ready"),
            listener.local_addr().unwrap().to_string(),
        )
        .unwrap();
        thread::sleep(Duration::from_secs(30));
        drop(listener);
    }
    #[test]
    fn fixture_parent() {
        if std::env::var("FUSION_NATIVE_FIXTURE").ok().as_deref() != Some("parent") {
            return;
        }
        let path = PathBuf::from(std::env::var_os("FUSION_NATIVE_DIR").unwrap());
        let _leaf = fixture("leaf", &path).spawn().unwrap();
        ready(&path);
        std::process::exit(0);
    }
    #[test]
    fn fixture_driver() {
        if std::env::var("FUSION_NATIVE_FIXTURE").ok().as_deref() != Some("driver") {
            return;
        }
        let path = PathBuf::from(std::env::var_os("FUSION_NATIVE_DIR").unwrap());
        root().unwrap();
        fs::create_dir_all(path.join(".git")).unwrap();
        let state = path.join("state");
        let mut engine = Supervisor::open(&state).unwrap();
        engine
            .apply(Event::Quota {
                auth_type: "chatgpt".into(),
                quota: Some(Quota {
                    primary: Some(Window {
                        used_percent: 0,
                        minutes: 300,
                        resets_at: 2_000_000_000,
                    }),
                    secondary: Some(Window {
                        used_percent: 0,
                        minutes: 10080,
                        resets_at: 2_000_604_800,
                    }),
                }),
                now: 1,
            })
            .unwrap();
        // Native recovery still validates scope before inspecting ownership.
        use sha2::{Digest, Sha256};
        let approved=serde_json::to_vec(&serde_json::json!({"schema_version":1,"task_id":1,"issue":4,"canonical_checkout":fs::canonicalize(&path).unwrap(),"mode":"read_only","instructions":"Controlled native containment crash demonstration only","non_goals":[],"verification":[]})).unwrap();
        let digest = format!("{:x}", Sha256::digest(&approved));
        fs::create_dir_all(state.join("approved-scopes")).unwrap();
        fs::write(
            state.join("approved-scopes").join(format!("{digest}.json")),
            approved,
        )
        .unwrap();
        engine
            .apply(Event::Submit(Task {
                id: 1,
                issue: 4,
                dependencies: vec![],
                scope_digest: digest,
            }))
            .unwrap();
        engine
            .apply(Event::Claim {
                owner: "native-fixture-owner".into(),
                now: 1,
            })
            .unwrap();
        storage::update(&engine, "fixture_native_reserved", |data| {
            data.native_active = Some(storage::NativeRecord {
                owner: "native-fixture-owner".into(),
                task: 1,
                generation: data.generation,
                pid: None,
                model: "gpt-6-luna".into(),
                reasoning: "low".into(),
            });
            Ok(())
        })
        .unwrap();
        crate::bridge::atomic(&path.join(".git/fusion-supervisor-active.json"),&serde_json::to_vec(&serde_json::json!({"owner":"native-fixture-owner","state_root":fs::canonicalize(state).unwrap()})).unwrap()).unwrap();
        let child = Owned::start(&mut fixture("parent", &path))
            .unwrap_or_else(|_| panic!("contained fixture spawn failed"));
        ready(&path);
        fs::write(path.join("driver.ready"), b"owned").unwrap();
        thread::sleep(Duration::from_secs(30));
        drop(child);
    }

    #[test]
    fn job_membership_proves_orphaned_grandchild_cleanup() {
        let path = dir();
        let mut child = Owned::start(&mut fixture("parent", &path))
            .unwrap_or_else(|_| panic!("contained fixture spawn failed"));
        let address = ready(&path);
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            if child.poll().unwrap().is_some() {
                break;
            }
            assert!(Instant::now() < until);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            child.job.process_count().unwrap() > 0,
            "reaped parent must not imply absent grandchild"
        );
        assert!(TcpStream::connect_timeout(&address, Duration::from_secs(1)).is_ok());
        child.stop(Duration::from_secs(5)).unwrap();
        assert_eq!(child.job.process_count().unwrap(), 0);
        assert!(child.poll().unwrap().is_some());
        socket_closed(address);
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn suspended_setup_failure_cleanup_covers_before_and_after_assignment() {
        root().unwrap();
        for assigned in [false, true] {
            let path = dir();
            let job = JobObject::new().unwrap();
            let mut command = fixture("leaf", &path);
            subc_jobobject::suspend_on_create(&mut command);
            let child = command.spawn().unwrap();
            let mut owner = Owned {
                child,
                job,
                readers: vec![],
            };
            if assigned {
                owner.job.assign(&owner.child).unwrap();
            }
            assert!(
                !path.join("leaf.ready").exists(),
                "suspended process must not execute fixture body"
            );
            owner.stop(Duration::from_secs(5)).unwrap();
            assert_eq!(owner.job.process_count().unwrap(), 0);
            assert!(owner.poll().unwrap().is_some());
            assert!(!path.join("leaf.ready").exists());
            fs::remove_dir_all(path).unwrap();
        }
    }
    #[test]
    fn crashed_driver_root_job_kills_descendants_and_restart_stays_fenced() {
        let path = dir();
        let mut driver = Owned::start(&mut fixture("driver", &path))
            .unwrap_or_else(|_| panic!("contained fixture spawn failed"));
        let address = ready(&path);
        let until = Instant::now() + Duration::from_secs(5);
        while !path.join("driver.ready").exists() {
            assert!(Instant::now() < until);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(driver.job.process_count().unwrap() >= 2);
        driver.child.kill().unwrap();
        driver.clean(Duration::from_secs(5)).unwrap();
        assert_eq!(driver.job.process_count().unwrap(), 0);
        socket_closed(address);
        let mut engine = Supervisor::open(&path.join("state")).unwrap();
        let runtime = RuntimeConfig {
            codex_executable: std::env::current_exe().unwrap(),
            checkout: path.clone(),
            bridge: path.join("bridge"),
            model: "gpt-6-luna".into(),
            environment: vec![],
            worker_deadline_ms: 1000,
        };
        assert_eq!(
            engine.drive_once(&runtime).unwrap_err(),
            Error::AmbiguousSurvivor,
            "unnamed old job cannot be reopened, so restart cannot infer cleanup"
        );
        assert_eq!(engine.snapshot().unwrap().pause, Pause::AmbiguousProcess);
        assert_eq!(engine.snapshot().unwrap().active, Some(1));
        assert!(path.join(".git/fusion-supervisor-active.json").exists());
        drop(engine);
        fs::remove_dir_all(path).unwrap();
    }
}
