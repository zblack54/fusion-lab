use crate::*;
use rusqlite::{Connection, TransactionBehavior, params};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    time::Duration,
};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct StoredTask {
    pub task: Task,
    pub state: State,
    pub attempts: u32,
    pub head: Option<String>,
    pub repairs: u32,
    pub deadline: Option<u64>,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct NativeRecord {
    pub owner: String,
    pub task: u64,
    pub generation: u64,
    pub pid: Option<u32>,
    pub model: String,
    pub reasoning: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Data {
    #[serde(skip)]
    pub tasks: BTreeMap<u64, StoredTask>,
    pub active: Option<u64>,
    pub generation: u64,
    pub owner: Option<String>,
    pub enabled: bool,
    pub pause: Pause,
    pub cadence: Cadence,
    pub next_wake: Option<u64>,
    pub quota_allowed: bool,
    pub suggestions: BTreeMap<String, u64>,
    pub intents: BTreeMap<String, Intent>,
    #[serde(default)]
    pub effects: BTreeMap<String, Intent>,
    #[serde(default)]
    pub invalidated: BTreeSet<String>,
    pub acknowledgements: BTreeMap<String, Ack>,
    pub launches: u32,
    pub checkpoints: BTreeMap<u64, String>,
    #[serde(default)]
    pub native_active: Option<NativeRecord>,
    #[serde(default)]
    pub cleanup_owner: Option<String>,
    #[serde(default)]
    pub evidence_task: Option<u64>,
    #[serde(default)]
    pub cancel_requested: BTreeSet<u64>,
    #[serde(default)]
    pub verified_heads: BTreeSet<String>,
    #[serde(default)]
    pub probe_events: BTreeSet<String>,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            tasks: BTreeMap::new(),
            active: None,
            generation: 0,
            owner: None,
            enabled: true,
            pause: Pause::None,
            cadence: Cadence::FiveHour,
            next_wake: None,
            quota_allowed: false,
            suggestions: BTreeMap::new(),
            intents: BTreeMap::new(),
            effects: BTreeMap::new(),
            invalidated: BTreeSet::new(),
            acknowledgements: BTreeMap::new(),
            launches: 0,
            checkpoints: BTreeMap::new(),
            native_active: None,
            cleanup_owner: None,
            evidence_task: None,
            cancel_requested: BTreeSet::new(),
            verified_heads: BTreeSet::new(),
            probe_events: BTreeSet::new(),
        }
    }
}
impl Data {
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            tasks: self
                .tasks
                .values()
                .map(|t| Row {
                    id: t.task.id,
                    state: t.state.clone(),
                    attempts: t.attempts,
                    head: t.head.clone(),
                })
                .collect(),
            active: self.active,
            generation: self.generation,
            pause: self.pause.clone(),
            cadence: self.cadence.clone(),
            next_wake: self.next_wake,
            suggestion_count: self.suggestions.len(),
            pending_intents: self.intents.values().cloned().collect(),
            worker_launches: self.launches,
        }
    }
}
pub(crate) fn connection(root: &Path) -> Result<Connection, Error> {
    let conn = Connection::open(root.join("supervisor.sqlite3")).map_err(|_| Error::Io)?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|_| Error::Io)?;
    conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS control(id INTEGER PRIMARY KEY CHECK(id=1), data TEXT NOT NULL); CREATE TABLE IF NOT EXISTS tasks(id TEXT PRIMARY KEY,data TEXT NOT NULL); CREATE TABLE IF NOT EXISTS events(sequence INTEGER PRIMARY KEY AUTOINCREMENT,data TEXT NOT NULL);") .map_err(|_|Error::Io)?;
    let integrity: String = conn
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .map_err(|_| Error::Io)?;
    if integrity != "ok" {
        return Err(Error::Io);
    }
    Ok(conn)
}
pub(crate) fn load(conn: &Connection) -> Result<Data, Error> {
    let text = conn.query_row("SELECT data FROM control WHERE id=1", [], |r| {
        r.get::<_, String>(0)
    });
    let mut data = match text {
        Ok(text) => serde_json::from_str(&text).map_err(|_| Error::Io)?,
        Err(rusqlite::Error::QueryReturnedNoRows) => Data::default(),
        Err(_) => return Err(Error::Io),
    };
    let mut stmt = conn
        .prepare("SELECT data FROM tasks ORDER BY id")
        .map_err(|_| Error::Io)?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|_| Error::Io)?;
    for row in rows {
        let task: StoredTask =
            serde_json::from_str(&row.map_err(|_| Error::Io)?).map_err(|_| Error::Io)?;
        data.tasks.insert(task.task.id, task);
    }
    if data.tasks.len() > 1024 || data.suggestions.len() > 128 {
        return Err(Error::Io);
    }
    Ok(data)
}
pub(crate) fn save(conn: &Connection, data: &Data, prior: &Data, event: &str) -> Result<(), Error> {
    if event.len() > 65536 {
        return Err(Error::Capacity);
    }
    let control = serde_json::to_string(data).map_err(|_| Error::Io)?;
    conn.execute(
        "INSERT INTO control(id,data)VALUES(1,?1)ON CONFLICT(id)DO UPDATE SET data=excluded.data",
        [control],
    )
    .map_err(|_| Error::Io)?;
    for task in data.tasks.values() {
        if prior.tasks.get(&task.task.id) == Some(task) {
            continue;
        }
        conn.execute("INSERT INTO tasks(id,data)VALUES(?1,?2)ON CONFLICT(id)DO UPDATE SET data=excluded.data",params![task.task.id.to_string(),serde_json::to_string(task).map_err(|_|Error::Io)?]).map_err(|_|Error::Io)?;
    }
    conn.execute("INSERT INTO events(data)VALUES(?1)", [event])
        .map_err(|_| Error::Io)?;
    Ok(())
}
pub(super) fn open(path: &Path) -> Result<Supervisor, Error> {
    fs::create_dir_all(path).map_err(|_| Error::Io)?;
    let root = fs::canonicalize(path).map_err(|_| Error::Io)?;
    connection(&root)?;
    Ok(Supervisor {
        root,
        writer: None,
        #[cfg(windows)]
        quarantine: None,
    })
}
pub(crate) fn acquire(path: &Path) -> Result<File, Error> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|_| Error::Io)?;
    file.try_lock().map_err(|_| Error::Busy)?;
    Ok(file)
}
pub(super) fn snapshot(engine: &Supervisor) -> Result<Snapshot, Error> {
    Ok(load(&connection(&engine.root)?)?.snapshot())
}
pub(crate) fn owner(data: &Data, token: &str, generation: u64) -> Result<u64, Error> {
    if data.owner.as_deref() != Some(token) || generation != data.generation {
        return Err(Error::StaleOwner);
    }
    data.active.ok_or(Error::InvalidTransition)
}
fn submit(data: &mut Data, task: Task) -> Result<(), Error> {
    if task.id == 0 || task.issue == 0 || task.scope_digest.is_empty() {
        return Err(Error::InvalidInput);
    }
    if data.tasks.contains_key(&task.id) {
        return Err(Error::DuplicateTask);
    }
    if task.dependencies.contains(&task.id) {
        return Err(Error::DependencyCycle);
    }
    if task
        .dependencies
        .iter()
        .any(|id| !data.tasks.contains_key(id))
    {
        return Err(Error::InvalidInput);
    }
    if data.tasks.len() >= 1024 {
        return Err(Error::Capacity);
    }
    data.tasks.insert(
        task.id,
        StoredTask {
            task,
            state: State::Ready,
            attempts: 0,
            head: None,
            repairs: 0,
            deadline: None,
        },
    );
    Ok(())
}
pub(crate) fn transition(data: &mut Data, event: Event) -> Result<(), Error> {
    if data.pause == Pause::EvidenceMismatch
        && !matches!(
            &event,
            Event::Cancel(_) | Event::Submit(_) | Event::Suggest { .. }
        )
    {
        return Err(Error::EvidenceMismatch);
    }
    match event {
        Event::Submit(task) => submit(data, task)?,
        Event::Claim { owner, now: _ } => {
            if data.pause == Pause::AmbiguousProcess {
                return Err(Error::AmbiguousSurvivor);
            }
            if data.active.is_some() {
                return Err(Error::Busy);
            }
            if !data.enabled || !data.quota_allowed || data.pause == Pause::AwaitingHost {
                return Err(Error::InvalidTransition);
            }
            if owner.is_empty() {
                return Err(Error::InvalidInput);
            }
            let id = data
                .tasks
                .iter()
                .find(|(_, t)| {
                    t.state == State::Ready
                        && t.task
                            .dependencies
                            .iter()
                            .all(|dep| data.tasks[dep].state == State::Done)
                })
                .map(|(id, _)| *id)
                .ok_or(Error::InvalidTransition)?;
            let t = data.tasks.get_mut(&id).ok_or(Error::InvalidInput)?;
            t.attempts = t.attempts.checked_add(1).ok_or(Error::Capacity)?;
            t.state = State::Running;
            data.active = Some(id);
            data.owner = Some(owner);
            data.generation = data.generation.checked_add(1).ok_or(Error::Capacity)?;
        }
        Event::Checkpoint {
            owner: token,
            generation,
            head,
        } => {
            let id = owner(data, &token, generation)?;
            data.tasks.get_mut(&id).ok_or(Error::InvalidInput)?.head = Some(head);
        }
        Event::Implemented {
            owner: token,
            generation,
            head,
        } => {
            let id = owner(data, &token, generation)?;
            let task = data.tasks.get_mut(&id).ok_or(Error::InvalidInput)?;
            task.head = Some(head);
            task.state = State::ImplementedUnverified;
            data.active = None;
            data.owner = None;
        }
        Event::Release {
            owner: token,
            generation,
        } => {
            if data.native_active.is_some() {
                return Err(Error::Busy);
            }
            let id = owner(data, &token, generation)?;
            data.tasks.get_mut(&id).ok_or(Error::InvalidInput)?.state = State::Ready;
            data.active = None;
            data.owner = None;
        }
        Event::Reconcile {
            process_alive,
            process_identity_matches: _,
            descendants,
        } => {
            if data.active.is_some() {
                if process_alive || descendants != DescendantStatus::KnownStopped {
                    data.pause = Pause::AmbiguousProcess;
                } else {
                    let id = data.active.take().ok_or(Error::InvalidTransition)?;
                    data.tasks.get_mut(&id).ok_or(Error::InvalidInput)?.state = State::Ready;
                    data.owner = None;
                    data.pause = Pause::None;
                }
            }
        }
        Event::Quota {
            auth_type,
            quota,
            now,
        } => {
            if auth_type != "chatgpt" {
                data.quota_allowed = false;
                data.pause = Pause::Authentication;
                return Err(Error::UnsupportedAuth);
            }
            let Some(q) = quota else {
                data.quota_allowed = false;
                data.pause = Pause::Quota;
                return Err(Error::MissingQuota);
            };
            let (Some(p), Some(w)) = (q.primary, q.secondary) else {
                data.quota_allowed = false;
                return Err(Error::MissingQuota);
            };
            if p.used_percent > 100
                || w.used_percent > 100
                || p.minutes != 300
                || w.minutes != 10080
            {
                return Err(Error::MalformedProtocol);
            }
            data.quota_allowed = p.used_percent < 100 && w.used_percent < 100;
            if w.used_percent == 100 {
                data.cadence = Cadence::WeeklyLatched;
                data.next_wake = Some(w.resets_at)
            } else if data.cadence == Cadence::WeeklyLatched {
                data.next_wake = Some(now.saturating_add(604800))
            } else if p.used_percent == 100 {
                data.next_wake = Some(p.resets_at)
            } else {
                data.next_wake = None
            }
            if !data.quota_allowed {
                data.pause = Pause::Quota;
            } else if data.pause == Pause::Quota || data.pause == Pause::Authentication {
                data.pause = Pause::None;
            }
        }
        Event::Enable(enabled) => {
            data.enabled = enabled;
            if !enabled {
                data.pause = Pause::Disabled
            } else if data.pause == Pause::Disabled {
                data.pause = Pause::None
            }
        }
        Event::Cancel(id) => {
            if !data.tasks.contains_key(&id) {
                return Err(Error::InvalidInput);
            }
            if data
                .native_active
                .as_ref()
                .is_some_and(|record| record.task == id)
            {
                data.cancel_requested.insert(id);
            } else {
                let rejected =
                    data.pause == Pause::EvidenceMismatch && data.evidence_task == Some(id);
                data.tasks.get_mut(&id).ok_or(Error::InvalidInput)?.state = State::Cancelled;
                if rejected {
                    data.evidence_task = None;
                    data.pause = Pause::None;
                }
                if data.active == Some(id) {
                    data.active = None;
                    data.owner = None;
                }
            }
        }
        Event::Repair {
            task,
            now,
            deadline,
        } => {
            let t = data.tasks.get_mut(&task).ok_or(Error::InvalidInput)?;
            let deadline = *t.deadline.get_or_insert(deadline);
            if now >= deadline {
                return Err(Error::Deadline);
            }
            if t.repairs >= 2 {
                return Err(Error::RepairExhausted);
            }
            t.repairs += 1;
            t.state = State::Ready;
        }
        Event::Suggest { key, issue } => {
            if key.is_empty() || issue == 0 {
                return Err(Error::InvalidInput);
            }
            if !data.suggestions.contains_key(&key) && data.suggestions.len() >= 128 {
                return Err(Error::Capacity);
            }
            data.suggestions.entry(key).or_insert(issue);
        }
        Event::AcceptSuggestion { key, task } => {
            if !data.suggestions.contains_key(&key) {
                return Err(Error::InvalidInput);
            }
            submit(data, task)?;
            data.suggestions.remove(&key);
        }
        Event::Tick { now: _ } => {}
        Event::Verify { task, head, passed } => {
            let t = data.tasks.get_mut(&task).ok_or(Error::InvalidInput)?;
            if t.head.as_deref() != Some(&head) {
                return Err(Error::EvidenceMismatch);
            }
            if !matches!(
                t.state,
                State::ImplementedUnverified | State::Verified | State::AwaitingHumanReview
            ) {
                return Err(Error::InvalidTransition);
            }
            if !passed {
                for (id, effect) in &data.effects {
                    if effect.task == task {
                        data.invalidated.insert(id.clone());
                    }
                }
                data.intents.retain(|_, effect| {
                    effect.task != task || !matches!(effect.kind, IntentKind::Merge)
                });
            }
            if passed {
                let event = format!("{task}:{head}");
                if data.verified_heads.insert(event) && data.verified_heads.len().is_multiple_of(5)
                {
                    probe(data, format!("verified:{}", data.verified_heads.len()))?;
                }
            }
            let t = data.tasks.get_mut(&task).ok_or(Error::InvalidInput)?;
            t.state = if passed {
                State::Verified
            } else {
                State::ImplementedUnverified
            };
        }
        Event::ChangeHead { task, head } => {
            let t = data.tasks.get_mut(&task).ok_or(Error::InvalidInput)?;
            if matches!(t.state, State::Done | State::Cancelled) {
                return Err(Error::InvalidTransition);
            }
            t.head = Some(head);
            t.state = State::ImplementedUnverified;
            data.intents.retain(|_, i| i.task != task);
            for (id, effect) in &data.effects {
                if effect.task == task {
                    data.invalidated.insert(id.clone());
                }
            }
        }
        Event::Complete { task, head } => {
            let t = data.tasks.get(&task).ok_or(Error::InvalidInput)?;
            if t.head.as_deref() != Some(&head) {
                return Err(Error::EvidenceMismatch);
            }
            if t.state != State::Done {
                return Err(Error::InvalidTransition);
            }
        }
        Event::Intent(intent) => crate::bridge::intent(data, intent)?,
        Event::Ack(ack) => crate::bridge::acknowledge(data, ack)?,
    }
    Ok(())
}
pub(super) fn apply(engine: &mut Supervisor, event: Event) -> Result<Snapshot, Error> {
    let event_json = serde_json::to_string(&event).map_err(|_| Error::InvalidInput)?;
    if event_json.len() > 65536 {
        return Err(Error::Capacity);
    }
    let is_claim = matches!(event, Event::Claim { .. });
    let guard = if is_claim && engine.writer.is_none() {
        Some(acquire(&engine.root.join("writer.lock"))?)
    } else {
        None
    };
    let mut conn = connection(&engine.root)?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| Error::Io)?;
    let mut data = load(&tx)?;
    let prior = data.clone();
    let result = transition(&mut data, event);
    if result.is_ok() || matches!(result, Err(Error::UnsupportedAuth | Error::MissingQuota)) {
        save(&tx, &data, &prior, &event_json)?;
        tx.commit().map_err(|_| Error::Io)?;
    }
    result?;
    if is_claim && let Some(guard) = guard {
        engine.writer = Some(guard);
    }
    if data.active.is_none() {
        engine.writer.take();
    }
    Ok(data.snapshot())
}

/// Internal adapters commit evidence through the same immediate transaction boundary.
pub(crate) fn update<T>(
    engine: &Supervisor,
    label: &str,
    change: impl FnOnce(&mut Data) -> Result<T, Error>,
) -> Result<T, Error> {
    let mut connection = connection(&engine.root)?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| Error::Io)?;
    let mut data = load(&transaction)?;
    let prior = data.clone();
    let result = change(&mut data)?;
    save(&transaction, &data, &prior, label)?;
    transaction.commit().map_err(|_| Error::Io)?;
    Ok(result)
}

pub(crate) fn probe(data: &mut Data, key: String) -> Result<(), Error> {
    if data.probe_events.contains(&key) {
        return Ok(());
    }
    if data.suggestions.len() >= 128 {
        return Err(Error::Capacity);
    }
    let suggestion = format!("probe:{key}");
    data.suggestions.insert(suggestion, 3);
    data.probe_events.insert(key);
    Ok(())
}
