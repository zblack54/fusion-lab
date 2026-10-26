//! Replaceable diagnostic compile seam for SPEC-SUPERVISOR; no supervisor is implemented.
#![forbid(unsafe_code)]
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pause {
    None,
    Disabled,
    Quota,
    Authentication,
    AmbiguousProcess,
    AwaitingHost,
    RepairExhausted,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cadence {
    FiveHour,
    WeeklyLatched,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: u64,
    pub issue: u64,
    pub dependencies: Vec<u64>,
    pub scope_digest: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: u64,
    pub state: State,
    pub attempts: u32,
    pub head: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub used_percent: u32,
    pub minutes: u32,
    pub resets_at: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quota {
    pub primary: Option<Window>,
    pub secondary: Option<Window>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentKind {
    CreatePr,
    RequestReview,
    Merge,
    UpdateIssue,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intent {
    pub id: String,
    pub task: u64,
    pub kind: IntentKind,
    pub expected_head: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ack {
    pub id: String,
    pub expected_head: String,
    pub observed_head: String,
    pub remote_url: String,
    pub outcome: AckOutcome,
    pub evidence: Option<HostEvidence>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescendantStatus {
    KnownStopped,
    Unknown,
    Alive,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AckOutcome {
    Applied,
    NotApplied,
    Uncertain,
}
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerPlan {
    pub executable: PathBuf,
    pub argv: Vec<String>,
    pub removed_environment_names: Vec<String>,
    pub model: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerOutcome {
    pub exit_code: i32,
    pub terminal: String,
    pub launches: u32,
    pub descendants_confirmed_stopped: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub codex_executable: PathBuf,
    pub checkout: PathBuf,
    pub bridge: PathBuf,
    pub model: String,
    pub environment: Vec<(String, String)>,
    pub worker_deadline_ms: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveReport {
    pub task: Option<u64>,
    pub terminal: String,
    pub launches: u32,
    pub next_wake: Option<u64>,
    pub checkpoint: Option<String>,
    pub exit_code: Option<i32>,
}
pub struct Supervisor {
    _state_root: PathBuf,
}
impl Supervisor {
    /// Diagnostic construction only; the compile seam creates no state or effects.
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            _state_root: path.to_path_buf(),
        })
    }
    pub fn apply(&mut self, _: Event) -> Result<Snapshot, Error> {
        Err(Error::NotImplemented)
    }
    pub fn snapshot(&self) -> Result<Snapshot, Error> {
        Err(Error::NotImplemented)
    }
    pub fn plan_worker(&self, _: &Path, _: &str) -> Result<WorkerPlan, Error> {
        Err(Error::NotImplemented)
    }
    pub fn drive_once(&mut self, _: &RuntimeConfig) -> Result<DriveReport, Error> {
        Err(Error::NotImplemented)
    }
    pub fn write_bridge(&self, _: &Path) -> Result<Vec<Intent>, Error> {
        Err(Error::NotImplemented)
    }
    pub fn read_bridge(&mut self, _: &Path) -> Result<Snapshot, Error> {
        Err(Error::NotImplemented)
    }
}
pub fn parse_quota(_: &str) -> Result<Quota, Error> {
    Err(Error::NotImplemented)
}
pub fn parse_account_type(_: &str) -> Result<String, Error> {
    Err(Error::NotImplemented)
}
