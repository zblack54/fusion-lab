use crate::{storage::Data, *};
use rusqlite::TransactionBehavior;
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
};

fn matching_review<'a>(data: &'a Data, task: u64, head: &str) -> Option<(&'a str, &'a str)> {
    data.acknowledgements.iter().find_map(|(id, ack)| {
        if data
            .effects
            .get(id)
            .is_none_or(|effect| effect.task != task)
        {
            return None;
        }
        if ack.expected_head != head || ack.outcome != AckOutcome::Applied {
            return None;
        }
        if let Some(HostEvidence::Review {
            head: review_head,
            checks_ref,
            review_ref,
            approved: true,
            checks_passed: true,
        }) = &ack.evidence
            && review_head == head
            && data.tasks.contains_key(&task)
        {
            return Some((checks_ref.as_str(), review_ref.as_str()));
        }
        None
    })
}
pub(crate) fn intent(data: &mut Data, intent: Intent) -> Result<(), Error> {
    if intent.id.is_empty()
        || intent.id.len() > 256
        || intent.expected_head.is_empty()
        || intent
            .id
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && !matches!(c, ':' | '_' | '-'))
    {
        return Err(Error::InvalidInput);
    }
    let task = data.tasks.get(&intent.task).ok_or(Error::InvalidInput)?;
    if let Some(ack) = data.acknowledgements.get(&intent.id)
        && ack.outcome == AckOutcome::Uncertain
    {
        return Err(Error::UncertainEffect);
    }
    if let Some(existing) = data.intents.get(&intent.id) {
        return if existing == &intent {
            Ok(())
        } else {
            Err(Error::InvalidInput)
        };
    }
    if data.acknowledgements.contains_key(&intent.id) {
        return Err(Error::InvalidTransition);
    }
    if data
        .intents
        .keys()
        .chain(data.acknowledgements.keys())
        .any(|id| id.replace(':', "_") == intent.id.replace(':', "_"))
    {
        return Err(Error::InvalidInput);
    }
    match intent.kind {
        IntentKind::Merge => {
            if task.state != State::Verified
                || task.head.as_deref() != Some(&intent.expected_head)
                || matching_review(data, intent.task, &intent.expected_head).is_none()
            {
                return Err(Error::InvalidTransition);
            }
        }
        IntentKind::RequestReview => {
            if task.state != State::Verified || task.head.as_deref() != Some(&intent.expected_head)
            {
                return Err(Error::InvalidTransition);
            }
        }
        IntentKind::UpdateIssue => {
            if task.state != State::Done {
                return Err(Error::InvalidTransition);
            }
        }
        IntentKind::CreatePr => {}
    }
    data.effects.insert(intent.id.clone(), intent.clone());
    data.intents.insert(intent.id.clone(), intent);
    Ok(())
}
pub(crate) fn acknowledge(data: &mut Data, ack: Ack) -> Result<(), Error> {
    if let Some(previous) = data.acknowledgements.get(&ack.id) {
        if previous == &ack {
            return Ok(());
        }
        if previous.outcome != AckOutcome::Uncertain {
            return Err(Error::InvalidAck);
        }
    }
    let intent = data
        .intents
        .get(&ack.id)
        .cloned()
        .ok_or(Error::InvalidAck)?;
    if ack.expected_head != intent.expected_head || ack.observed_head != intent.expected_head {
        return Err(Error::EvidenceMismatch);
    }
    if ack.outcome == AckOutcome::Uncertain {
        data.acknowledgements.insert(ack.id.clone(), ack);
        data.pause = Pause::AwaitingHost;
        return Ok(());
    }
    if ack.outcome == AckOutcome::NotApplied {
        data.acknowledgements.insert(ack.id.clone(), ack);
        data.intents.remove(&intent.id);
        return Ok(());
    }
    if !ack.remote_url.starts_with("https://github.com/") {
        return Err(Error::EvidenceMismatch);
    }
    let task = data.tasks.get(&intent.task).ok_or(Error::InvalidInput)?;
    match (&intent.kind, &ack.evidence) {
        (
            IntentKind::CreatePr,
            Some(HostEvidence::Pr {
                head,
                pr_url,
                issue,
                scope_digest,
            }),
        ) if head == &intent.expected_head
            && pr_url == &ack.remote_url
            && *issue == task.task.issue
            && scope_digest == &task.task.scope_digest => {}
        (
            IntentKind::RequestReview,
            Some(HostEvidence::Review {
                head,
                checks_ref,
                review_ref,
                approved: true,
                checks_passed: true,
            }),
        ) if head == &intent.expected_head
            && !checks_ref.trim().is_empty()
            && !review_ref.trim().is_empty()
            && task.state == State::Verified
            && task.head.as_deref() == Some(head) => {}
        (
            IntentKind::Merge,
            Some(HostEvidence::Merge {
                head,
                merge_sha,
                checks_ref,
                review_ref,
            }),
        ) if head == &intent.expected_head
            && merge_sha.len() == 40
            && merge_sha.bytes().all(|b| b.is_ascii_hexdigit())
            && matching_review(data, intent.task, head)
                == Some((checks_ref.as_str(), review_ref.as_str()))
            && task.state == State::Verified
            && task.head.as_deref() == Some(head) => {}
        (
            IntentKind::UpdateIssue,
            Some(HostEvidence::Issue {
                issue,
                completion_ref,
            }),
        ) if *issue == task.task.issue
            && !completion_ref.trim().is_empty()
            && task.state == State::Done => {}
        _ => return Err(Error::EvidenceMismatch),
    }
    let merged = intent.kind == IntentKind::Merge;
    data.acknowledgements.insert(ack.id.clone(), ack);
    data.intents.remove(&intent.id);
    if merged {
        data.tasks
            .get_mut(&intent.task)
            .ok_or(Error::InvalidInput)?
            .state = State::Done;
    }
    if data.intents.is_empty() && data.pause == Pause::AwaitingHost {
        data.pause = Pause::None;
    }
    Ok(())
}
pub(crate) fn atomic(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let parent = path.parent().ok_or(Error::InvalidInput)?;
    fs::create_dir_all(parent).map_err(|_| Error::Io)?;
    let mut nonce = [0u8; 16];
    getrandom::fill(&mut nonce).map_err(|_| Error::Io)?;
    let name = nonce.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let temp = parent.join(format!(".{name}.tmp"));
    let outcome = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|_| Error::Io)?;
        file.write_all(bytes).map_err(|_| Error::Io)?;
        file.sync_all().map_err(|_| Error::Io)?;
        drop(file);
        fs::rename(&temp, path).map_err(|_| Error::Io)
    })();
    if outcome.is_err() {
        let _ = fs::remove_file(temp);
    }
    outcome
}
pub(super) fn export(engine: &Supervisor, bridge: &Path) -> Result<Vec<Intent>, Error> {
    let data = crate::storage::load(&crate::storage::connection(&engine.root)?)?;
    let intents = data.intents.values().cloned().collect::<Vec<_>>();
    for intent in &intents {
        let task = &data.tasks[&intent.task];
        let kind = serde_json::to_value(&intent.kind).map_err(|_| Error::Io)?;
        let payload = match intent.kind {
            IntentKind::CreatePr => {
                json!({"branch_ref":format!("task:{}:branch",intent.task),"base_ref":"main","title_ref":format!("issue:{}:title",task.task.issue),"body_ref":format!("scope:{}",task.task.scope_digest)})
            }
            IntentKind::RequestReview | IntentKind::Merge => {
                json!({"pr_ref":format!("task:{}:pr",intent.task),"requested_head":intent.expected_head,"evidence_ref":format!("task:{}:head:{}",intent.task,intent.expected_head)})
            }
            IntentKind::UpdateIssue => {
                json!({"issue":task.task.issue,"accepted_outcome_ref":format!("task:{}:merge",intent.task)})
            }
        };
        let wire = json!({"schema_version":1,"id":intent.id,"task":intent.task,"kind":kind,"expected_head":intent.expected_head,"issue":task.task.issue,"scope_digest":task.task.scope_digest,"payload":payload,"reconcile_only":data.acknowledgements.get(&intent.id).is_some_and(|a|a.outcome==AckOutcome::Uncertain)});
        let path = bridge
            .join("outbox")
            .join(format!("{}.json", intent.id.replace(':', "_")));
        if path.exists() {
            let old: Value =
                serde_json::from_slice(&read_bounded(&path)?).map_err(|_| Error::InvalidInput)?;
            if old["id"] != intent.id {
                return Err(Error::InvalidInput);
            }
        }
        atomic(&path, &serde_json::to_vec(&wire).map_err(|_| Error::Io)?)?;
    }
    Ok(intents)
}
fn read_bounded(path: &Path) -> Result<Vec<u8>, Error> {
    let file = fs::File::open(path).map_err(|_| Error::Io)?;
    if file.metadata().map_err(|_| Error::Io)?.len() > 1048576 {
        return Err(Error::Capacity);
    }
    let mut bytes = Vec::new();
    file.take(1048577)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::Io)?;
    if bytes.len() > 1048576 {
        return Err(Error::Capacity);
    }
    Ok(bytes)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireAck {
    schema_version: u64,
    id: String,
    expected_head: String,
    observed_head: String,
    remote_url: String,
    outcome: AckOutcome,
    evidence: Option<HostEvidence>,
}
pub(super) fn import(engine: &mut Supervisor, bridge: &Path) -> Result<Snapshot, Error> {
    let inbox = bridge.join("inbox");
    if !inbox.exists() {
        return engine.snapshot();
    }
    let mut paths = fs::read_dir(&inbox)
        .map_err(|_| Error::Io)?
        .map(|entry| entry.map(|e| e.path()).map_err(|_| Error::Io))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    if paths.len() > 1024 {
        return Err(Error::Capacity);
    }
    let mut conn = crate::storage::connection(&engine.root)?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| Error::Io)?;
    let mut data = crate::storage::load(&tx)?;
    let prior = data.clone();
    for path in paths
        .iter()
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
    {
        let wire: WireAck =
            serde_json::from_slice(&read_bounded(path)?).map_err(|_| Error::InvalidAck)?;
        if wire.schema_version != 1 {
            return Err(Error::InvalidAck);
        }
        acknowledge(
            &mut data,
            Ack {
                id: wire.id,
                expected_head: wire.expected_head,
                observed_head: wire.observed_head,
                remote_url: wire.remote_url,
                outcome: wire.outcome,
                evidence: wire.evidence,
            },
        )?;
    }
    crate::storage::save(&tx, &data, &prior, "host_ack_import")?;
    tx.commit().map_err(|_| Error::Io)?;
    let archive = bridge.join("processed");
    fs::create_dir_all(&archive).map_err(|_| Error::Io)?;
    for path in paths
        .iter()
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
    {
        fs::rename(
            path,
            archive.join(path.file_name().ok_or(Error::InvalidInput)?),
        )
        .map_err(|_| Error::Io)?;
    }
    Ok(data.snapshot())
}
