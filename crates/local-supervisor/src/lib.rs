//! Durable, fenced local orchestration with subscription-only worker boundaries.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
mod account;
mod bridge;
mod storage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    NotImplemented,
    InvalidInput,
    DependencyCycle,
    DuplicateTask,
    Busy,
    StaleOwner,
    AmbiguousSurvivor,
    AuthenticationRequired,
    UnsupportedAuth,
    MissingQuota,
    QuotaExhausted,
    Capacity,
    InvalidTransition,
    EvidenceMismatch,
    UncertainEffect,
    InvalidAck,
    Deadline,
    RepairExhausted,
    Cancelled,
    MalformedProtocol,
    Io,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Ready,
    Running,
    ImplementedUnverified,
    Verified,
    AwaitingHumanReview,
    Done,
    Cancelled,
    Paused,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pause {
    None,
    Disabled,
    Quota,
    Authentication,
    AmbiguousProcess,
    AwaitingHost,
    RepairExhausted,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cadence {
    FiveHour,
    WeeklyLatched,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub issue: u64,
    pub dependencies: Vec<u64>,
    pub scope_digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    pub id: u64,
    pub state: State,
    pub attempts: u32,
    pub head: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub tasks: Vec<Row>,
    pub active: Option<u64>,
    pub generation: u64,
    pub pause: Pause,
    pub cadence: Cadence,
    pub next_wake: Option<u64>,
    pub suggestion_count: usize,
    pub pending_intents: Vec<Intent>,
    pub worker_launches: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Window {
    pub used_percent: u32,
    pub minutes: u32,
    pub resets_at: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quota {
    pub primary: Option<Window>,
    pub secondary: Option<Window>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentKind {
    CreatePr,
    RequestReview,
    Merge,
    UpdateIssue,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intent {
    pub id: String,
    pub task: u64,
    pub kind: IntentKind,
    pub expected_head: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ack {
    pub id: String,
    pub expected_head: String,
    pub observed_head: String,
    pub remote_url: String,
    pub outcome: AckOutcome,
    pub evidence: Option<HostEvidence>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HostEvidence {
    Pr {
        head: String,
        pr_url: String,
        issue: u64,
        scope_digest: String,
    },
    Review {
        head: String,
        checks_ref: String,
        review_ref: String,
        approved: bool,
        checks_passed: bool,
    },
    Merge {
        head: String,
        merge_sha: String,
        checks_ref: String,
        review_ref: String,
    },
    Issue {
        issue: u64,
        completion_ref: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DescendantStatus {
    KnownStopped,
    Unknown,
    Alive,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AckOutcome {
    Applied,
    NotApplied,
    Uncertain,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Submit(Task),
    Claim {
        owner: String,
        now: u64,
    },
    Release {
        owner: String,
        generation: u64,
    },
    Checkpoint {
        owner: String,
        generation: u64,
        head: String,
    },
    Implemented {
        owner: String,
        generation: u64,
        head: String,
    },
    Verify {
        task: u64,
        head: String,
        passed: bool,
    },
    Complete {
        task: u64,
        head: String,
    },
    Quota {
        auth_type: String,
        quota: Option<Quota>,
        now: u64,
    },
    Reconcile {
        process_alive: bool,
        process_identity_matches: bool,
        descendants: DescendantStatus,
    },
    Enable(bool),
    Cancel(u64),
    Repair {
        task: u64,
        now: u64,
        deadline: u64,
    },
    Suggest {
        key: String,
        issue: u64,
    },
    AcceptSuggestion {
        key: String,
        task: Task,
    },
    Intent(Intent),
    Ack(Ack),
    Tick {
        now: u64,
    },
    ChangeHead {
        task: u64,
        head: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerPlan {
    pub executable: PathBuf,
    pub argv: Vec<String>,
    pub removed_environment_names: Vec<String>,
    pub model: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerOutcome {
    pub exit_code: i32,
    pub terminal: String,
    pub launches: u32,
    pub descendants_confirmed_stopped: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub codex_executable: PathBuf,
    pub checkout: PathBuf,
    pub bridge: PathBuf,
    pub model: String,
    pub environment: Vec<(String, String)>,
    pub worker_deadline_ms: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriveReport {
    pub task: Option<u64>,
    pub terminal: String,
    pub launches: u32,
    pub next_wake: Option<u64>,
    pub checkpoint: Option<String>,
    pub exit_code: Option<i32>,
}
pub struct Supervisor {
    root: PathBuf,
    writer: Option<std::fs::File>,
}
impl Supervisor {
    /// Open durable state without claiming the checkout writer.
    pub fn open(path: &Path) -> Result<Self, Error> {
        storage::open(path)
    }
    /// Apply one validated durable domain command.
    pub fn apply(&mut self, event: Event) -> Result<Snapshot, Error> {
        storage::apply(self, event)
    }
    /// Observe only committed state.
    pub fn snapshot(&self) -> Result<Snapshot, Error> {
        storage::snapshot(self)
    }
    pub fn plan_worker(&self, executable: &Path, model: &str) -> Result<WorkerPlan, Error> {
        account::plan(executable, model)
    }
    pub fn drive_once(&mut self, runtime: &RuntimeConfig) -> Result<DriveReport, Error> {
        self.plan_worker(&runtime.codex_executable, &runtime.model)?;
        if !(1..=3600000).contains(&runtime.worker_deadline_ms) {
            return Err(Error::InvalidInput);
        }
        let _scheduler = storage::acquire(&self.root.join("scheduler.lock"))?;
        self.read_bridge(&runtime.bridge)?;
        let state = self.snapshot()?;
        if state.pause == Pause::Disabled {
            return Err(Error::InvalidTransition);
        }
        let quota = match account::fetch(runtime) {
            Ok(quota) => quota,
            Err(error) => {
                let auth_type = if matches!(
                    error,
                    Error::AuthenticationRequired | Error::UnsupportedAuth
                ) {
                    "unavailable"
                } else {
                    "chatgpt"
                };
                match self.apply(Event::Quota {
                    auth_type: auth_type.into(),
                    quota: None,
                    now: 0,
                }) {
                    Ok(_) | Err(Error::UnsupportedAuth | Error::MissingQuota) => {}
                    Err(persistence_error) => return Err(persistence_error),
                }
                return Err(error);
            }
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::Io)?
            .as_secs();
        let state = self.apply(Event::Quota {
            auth_type: "chatgpt".into(),
            quota: Some(quota),
            now,
        })?;
        if state.pause == Pause::Quota {
            return Err(Error::QuotaExhausted);
        }
        // Native dispatch remains gated until descendant containment has an approved proof boundary.
        Err(Error::NotImplemented)
    }
    pub fn write_bridge(&self, path: &Path) -> Result<Vec<Intent>, Error> {
        bridge::export(self, path)
    }
    pub fn read_bridge(&mut self, path: &Path) -> Result<Snapshot, Error> {
        bridge::import(self, path)
    }
}
pub fn parse_quota(text: &str) -> Result<Quota, Error> {
    account::parse_limits(text)
}
pub fn parse_account_type(text: &str) -> Result<String, Error> {
    account::parse_account(text)
}
