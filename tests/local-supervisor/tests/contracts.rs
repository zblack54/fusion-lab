use local_supervisor::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
fn verified(e: &mut Supervisor, id: u64, head: &str) {
    allow(e);
    submit(e, id, &[]);
    let s = call(e.apply(Event::Claim {
        owner: "a".into(),
        now: 1,
    }))
    .unwrap();
    call(e.apply(Event::Implemented {
        owner: "a".into(),
        generation: s.generation,
        head: head.into(),
    }))
    .unwrap();
    call(e.apply(Event::Verify {
        task: id,
        head: head.into(),
        passed: true,
    }))
    .unwrap();
    let intent = Intent {
        id: format!("existing-pr:{id}:{head}"),
        task: id,
        kind: IntentKind::CreatePr,
        expected_head: head.into(),
    };
    call(e.apply(Event::Intent(intent.clone()))).unwrap();
    call(e.apply(Event::Ack(Ack {
        id: intent.id,
        expected_head: head.into(),
        observed_head: head.into(),
        remote_url: "https://github.com/zblack54/fusion-lab/pull/1".into(),
        outcome: AckOutcome::Applied,
        evidence: Some(HostEvidence::Pr {
            head: head.into(),
            pr_url: "https://github.com/zblack54/fusion-lab/pull/1".into(),
            issue: 4,
            scope_digest: "abc".into(),
        }),
    })))
    .unwrap();
}
#[test]
fn tst_neg_009_applied_ack_requires_matching_evidence_kind() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    submit(&mut e, 1, &[]);
    let intent = Intent {
        id: "pr:1:h".into(),
        task: 1,
        kind: IntentKind::CreatePr,
        expected_head: "h".into(),
    };
    call(e.apply(Event::Intent(intent))).unwrap();
    let before = call(e.snapshot()).unwrap();
    let ack = Ack {
        id: "pr:1:h".into(),
        expected_head: "h".into(),
        observed_head: "h".into(),
        remote_url: "https://github.com/zblack54/fusion-lab/pull/1".into(),
        outcome: AckOutcome::Applied,
        evidence: None,
    };
    assert_eq!(
        call(e.apply(Event::Ack(ack.clone()))).unwrap_err(),
        Error::EvidenceMismatch
    );
    let mut wrong = ack;
    wrong.evidence = Some(HostEvidence::Issue {
        issue: 4,
        completion_ref: "done".into(),
    });
    assert_eq!(
        call(e.apply(Event::Ack(wrong))).unwrap_err(),
        Error::EvidenceMismatch
    );
    assert_eq!(call(e.snapshot()).unwrap(), before);
}
#[test]
fn tst_neg_010_denied_review_or_failed_checks_cannot_authorize_merge() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    verified(&mut e, 1, "h");
    let intent = Intent {
        id: "review:1:h".into(),
        task: 1,
        kind: IntentKind::RequestReview,
        expected_head: "h".into(),
    };
    call(e.apply(Event::Intent(intent))).unwrap();
    let before = call(e.snapshot()).unwrap();
    for (approved, checks_passed, checks_ref, review_ref) in [
        (false, true, "checks:h", "review:h"),
        (true, false, "checks:h", "review:h"),
        (true, true, "", "review:h"),
        (true, true, "checks:h", ""),
    ] {
        let ack = Ack {
            id: "review:1:h".into(),
            expected_head: "h".into(),
            observed_head: "h".into(),
            remote_url: "https://github.com/zblack54/fusion-lab/pull/1".into(),
            outcome: AckOutcome::Applied,
            evidence: Some(HostEvidence::Review {
                head: "h".into(),
                checks_ref: checks_ref.into(),
                review_ref: review_ref.into(),
                approved,
                checks_passed,
            }),
        };
        assert_eq!(
            call(e.apply(Event::Ack(ack))).unwrap_err(),
            Error::EvidenceMismatch
        );
        assert_eq!(call(e.snapshot()).unwrap(), before);
    }
    let merge = Intent {
        id: "merge:1:h".into(),
        task: 1,
        kind: IntentKind::Merge,
        expected_head: "h".into(),
    };
    assert_eq!(
        call(e.apply(Event::Intent(merge))).unwrap_err(),
        Error::InvalidTransition
    );
}
#[test]
fn tst_neg_011_merge_requires_same_approved_refs_and_merge_sha() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    verified(&mut e, 1, "h");
    let review = Intent {
        id: "review:1:h".into(),
        task: 1,
        kind: IntentKind::RequestReview,
        expected_head: "h".into(),
    };
    call(e.apply(Event::Intent(review))).unwrap();
    call(e.apply(Event::Ack(Ack {
        id: "review:1:h".into(),
        expected_head: "h".into(),
        observed_head: "h".into(),
        remote_url: "https://github.com/zblack54/fusion-lab/pull/1".into(),
        outcome: AckOutcome::Applied,
        evidence: Some(HostEvidence::Review {
            head: "h".into(),
            checks_ref: "checks:h".into(),
            review_ref: "review:h".into(),
            approved: true,
            checks_passed: true,
        }),
    })))
    .unwrap();
    let intent = Intent {
        id: "merge:1:h".into(),
        task: 1,
        kind: IntentKind::Merge,
        expected_head: "h".into(),
    };
    call(e.apply(Event::Intent(intent))).unwrap();
    let before = call(e.snapshot()).unwrap();
    for (merge_sha, checks_ref, review_ref) in [
        ("", "checks:h", "review:h"),
        ("not-a-merge-sha", "checks:h", "review:h"),
        (
            "0123456789012345678901234567890123456789",
            "checks:stale",
            "review:h",
        ),
        (
            "0123456789012345678901234567890123456789",
            "checks:h",
            "review:stale",
        ),
    ] {
        let ack = Ack {
            id: "merge:1:h".into(),
            expected_head: "h".into(),
            observed_head: "h".into(),
            remote_url: "https://github.com/zblack54/fusion-lab/pull/1".into(),
            outcome: AckOutcome::Applied,
            evidence: Some(HostEvidence::Merge {
                head: "h".into(),
                merge_sha: merge_sha.into(),
                checks_ref: checks_ref.into(),
                review_ref: review_ref.into(),
            }),
        };
        assert_eq!(
            call(e.apply(Event::Ack(ack))).unwrap_err(),
            Error::EvidenceMismatch
        );
        assert_eq!(call(e.snapshot()).unwrap(), before);
    }
    assert_ne!(row(&call(e.snapshot()).unwrap(), 1).state, State::Done);
}
#[test]
fn tst_neg_012_dead_root_unknown_descendants_keep_writer_fence() {
    for descendants in [DescendantStatus::Unknown, DescendantStatus::Alive] {
        let dir = Dir::new();
        let mut e = engine(&dir);
        allow(&mut e);
        submit(&mut e, 1, &[]);
        call(e.apply(Event::Claim {
            owner: "a".into(),
            now: 10,
        }))
        .unwrap();
        drop(e);
        let mut e = engine(&dir);
        let s = call(e.apply(Event::Reconcile {
            process_alive: false,
            process_identity_matches: false,
            descendants,
        }))
        .unwrap();
        assert_eq!(s.pause, Pause::AmbiguousProcess);
        assert_eq!(s.active, Some(1));
        assert_eq!(row(&s, 1).state, State::Running);
        assert_eq!(
            call(e.apply(Event::Claim {
                owner: "b".into(),
                now: 100000
            }))
            .unwrap_err(),
            Error::AmbiguousSurvivor
        );
        assert_eq!(row(&call(e.snapshot()).unwrap(), 1).attempts, 1);
    }
}

#[test]
fn tst_acc_008_cli_step_runs_real_child_and_exports_bridge() {
    let dir = Dir::new();
    let fake = runtime(&dir, "ok");
    let digest = materialize(&dir, &scope_document(&fake, 1, "workspace_write"));
    let checkout = fake.checkout.to_string_lossy().into_owned();
    let first = cli(
        &dir,
        &["enqueue", "--task", "1", "--issue", "4", "--scope", &digest],
    );
    assert_eq!(first.status.code(), Some(0));
    let executable = fake.codex_executable.to_str().unwrap();
    let env = format!("FAKE_CODEX_ROOT={}", dir.0.join("checkout/fake").display());
    let result = cli(
        &dir,
        &[
            "step",
            "--checkout",
            &checkout,
            "--codex",
            executable,
            "--model",
            "gpt-6-luna",
            "--child-env",
            &env,
        ],
    );
    assert_eq!(result.status.code(), Some(0));
    let report = String::from_utf8(result.stdout).unwrap();
    assert!(report.contains("\"task\":1"));
    assert!(report.contains("\"terminal\":\"awaiting_host\""));
    assert!(dir.0.join("checkout/fake/launch.txt").exists());
    assert!(dir.0.join("bridge/outbox").exists());
}

fn observed_head(checkout: &Path) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(checkout)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
fn scope_document(runtime: &RuntimeConfig, id: u64, mode: &str) -> Value {
    json!({"schema_version":1,"task_id":id,"issue":4,
        "canonical_checkout":std::fs::canonicalize(&runtime.checkout).unwrap().to_string_lossy(),
        "mode":mode,"instructions":format!("approved-fixture-task-{id}: perform only the declared controlled fixture work"),
        "non_goals":["no deployment","no paid APIs"],"verification":["controlled fixture checks"]})
}
fn scope_bytes(d: &Dir, bytes: &[u8]) -> String {
    assert_eq!(
        format!("{:x}", Sha256::digest(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let digest = format!("{:x}", Sha256::digest(bytes));
    std::fs::create_dir_all(d.0.join("approved-scopes")).unwrap();
    std::fs::write(
        d.0.join("approved-scopes").join(format!("{digest}.json")),
        bytes,
    )
    .unwrap();
    digest
}
fn materialize(d: &Dir, document: &Value) -> String {
    scope_bytes(d, &serde_json::to_vec(document).unwrap())
}
fn native_submit(
    e: &mut Supervisor,
    d: &Dir,
    runtime: &RuntimeConfig,
    id: u64,
    deps: &[u64],
    mode: &str,
) -> String {
    let digest = materialize(d, &scope_document(runtime, id, mode));
    call(e.apply(Event::Submit(Task {
        id,
        issue: 4,
        dependencies: deps.into(),
        scope_digest: digest.clone(),
    })))
    .unwrap();
    digest
}

fn runtime(d: &Dir, scenario: &str) -> RuntimeConfig {
    runtime_at(d, scenario, d.0.join("checkout"))
}
fn runtime_at(d: &Dir, scenario: &str, checkout: PathBuf) -> RuntimeConfig {
    std::fs::create_dir_all(checkout.clone()).unwrap();
    if !checkout.join(".git").exists() {
        for args in [
            vec!["init"],
            vec![
                "-c",
                "user.name=Spec",
                "-c",
                "user.email=spec@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--allow-empty",
                "-m",
                "initial fixture",
            ],
        ] {
            let output = Command::new("git")
                .current_dir(checkout.clone())
                .args(args)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "fixture git setup failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    let fake = checkout.join("fake");
    std::fs::create_dir_all(&fake).unwrap();
    std::fs::write(fake.join("scenario.txt"), scenario).unwrap();
    RuntimeConfig {
        codex_executable: PathBuf::from(env!("CARGO_BIN_EXE_fake-codex")),
        checkout: checkout.clone(),
        bridge: d.0.join("bridge"),
        model: "gpt-6-luna".into(),
        environment: vec![(
            "FAKE_CODEX_ROOT".into(),
            fake.to_string_lossy().into_owned(),
        )],
        worker_deadline_ms: 5000,
    }
}
fn finish_host(e: &mut Supervisor, id: u64, head: &str, scope_digest: &str) {
    let d = Dir::new();
    let bridge = d.0.join("bridge");
    std::fs::create_dir_all(bridge.join("inbox")).unwrap();
    for kind in [
        IntentKind::CreatePr,
        IntentKind::RequestReview,
        IntentKind::Merge,
    ] {
        let current = call(e.snapshot()).unwrap();
        let intent = if let Some(found) = current.pending_intents.iter().find(|i| i.kind == kind) {
            found.clone()
        } else {
            let intent = Intent {
                id: format!("{kind:?}:{id}:{head}"),
                task: id,
                kind,
                expected_head: head.into(),
            };
            call(e.apply(Event::Intent(intent.clone()))).unwrap();
            intent
        };
        call(e.write_bridge(&bridge)).unwrap();
        let evidence = match &intent.kind {
            IntentKind::CreatePr => format!(
                r#"{{"kind":"pr","head":"{head}","pr_url":"https://github.com/zblack54/fusion-lab/pull/1","issue":4,"scope_digest":"{scope_digest}"}}"#
            ),
            IntentKind::RequestReview => format!(
                r#"{{"kind":"review","head":"{head}","checks_ref":"checks:{head}","review_ref":"review:{head}","approved":true,"checks_passed":true}}"#
            ),
            IntentKind::Merge => format!(
                r#"{{"kind":"merge","head":"{head}","merge_sha":"0123456789012345678901234567890123456789","checks_ref":"checks:{head}","review_ref":"review:{head}"}}"#
            ),
            IntentKind::UpdateIssue => {
                r#"{"kind":"issue","issue":4,"completion_ref":"accepted:1"}"#.to_string()
            }
        };
        let ack = format!(
            r#"{{"schema_version":1,"id":"{}","expected_head":"{head}","observed_head":"{head}","remote_url":"https://github.com/zblack54/fusion-lab/pull/1","outcome":"applied","evidence":{evidence}}}"#,
            intent.id
        );
        std::fs::write(
            bridge
                .join("inbox")
                .join(format!("{}.json", intent.id.replace(':', "_"))),
            ack,
        )
        .unwrap();
        call(e.read_bridge(&bridge)).unwrap();
    }
}
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Dir(PathBuf);
impl Dir {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "supervisor-spec-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn call<T>(r: Result<T, Error>) -> Result<T, Error> {
    if matches!(&r, Err(Error::NotImplemented)) {
        panic!("SUPERVISOR_NOT_IMPLEMENTED: diagnostic compile seam reached")
    }
    r
}
fn engine(d: &Dir) -> Supervisor {
    call(Supervisor::open(&d.0)).unwrap()
}
fn task(id: u64, deps: &[u64]) -> Task {
    Task {
        id,
        issue: 4,
        dependencies: deps.into(),
        scope_digest: "abc".into(),
    }
}
fn submit(e: &mut Supervisor, id: u64, deps: &[u64]) {
    call(e.apply(Event::Submit(task(id, deps)))).unwrap();
}
fn row(s: &Snapshot, id: u64) -> &Row {
    s.tasks.iter().find(|r| r.id == id).unwrap()
}
fn quota(primary: u32, secondary: u32) -> Quota {
    Quota {
        primary: Some(Window {
            used_percent: primary,
            minutes: 300,
            resets_at: 100,
        }),
        secondary: Some(Window {
            used_percent: secondary,
            minutes: 10080,
            resets_at: 1000,
        }),
    }
}
fn allow(e: &mut Supervisor) {
    call(e.apply(Event::Quota {
        auth_type: "chatgpt".into(),
        quota: Some(quota(0, 0)),
        now: 1,
    }))
    .unwrap();
}
fn finish(e: &mut Supervisor, id: u64, head: &str) {
    let s = call(e.snapshot()).unwrap();
    call(e.apply(Event::Implemented {
        owner: "a".into(),
        generation: s.generation,
        head: head.into(),
    }))
    .unwrap();
    call(e.apply(Event::Verify {
        task: id,
        head: head.into(),
        passed: true,
    }))
    .unwrap();
    finish_host(e, id, head, "abc");
}
fn cli(d: &Dir, args: &[&str]) -> Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/local-supervisor");
    let build = Command::new("cargo")
        .args(["build", "--locked", "--offline", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "CLI build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let bin = root.join("target/debug/local-supervisor.exe");
    let out = Command::new(bin)
        .arg("--state-dir")
        .arg(&d.0)
        .args(args)
        .output()
        .unwrap();
    if out.status.code() == Some(78)
        && String::from_utf8_lossy(&out.stderr).contains("SUPERVISOR_NOT_IMPLEMENTED")
    {
        panic!("SUPERVISOR_NOT_IMPLEMENTED: executable diagnostic seam reached")
    }
    out
}

#[test]
fn tst_acc_001_runs_ready_dependency_then_continues() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    allow(&mut e);
    submit(&mut e, 1, &[]);
    submit(&mut e, 2, &[1]);
    let first = call(e.apply(Event::Claim {
        owner: "a".into(),
        now: 10,
    }))
    .unwrap();
    assert_eq!(first.active, Some(1));
    assert_eq!(row(&first, 2).state, State::Ready);
    finish(&mut e, 1, "h1");
    let next = call(e.apply(Event::Claim {
        owner: "a".into(),
        now: 11,
    }))
    .unwrap();
    assert_eq!(next.active, Some(2));
    assert_eq!(row(&next, 1).state, State::Done);
}

#[test]
fn tst_acc_002_restart_preserves_checkpoint_and_requeues() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    allow(&mut e);
    submit(&mut e, 1, &[]);
    let s = call(e.apply(Event::Claim {
        owner: "a".into(),
        now: 10,
    }))
    .unwrap();
    call(e.apply(Event::Checkpoint {
        owner: "a".into(),
        generation: s.generation,
        head: "abc".into(),
    }))
    .unwrap();
    drop(e);
    let mut e = engine(&dir);
    let s = call(e.apply(Event::Reconcile {
        process_alive: false,
        process_identity_matches: false,
        descendants: DescendantStatus::KnownStopped,
    }))
    .unwrap();
    assert_eq!(s.active, None);
    assert_eq!(row(&s, 1).head.as_deref(), Some("abc"));
    assert_eq!(row(&s, 1).state, State::Ready);
    assert_eq!(row(&s, 1).attempts, 1);
}

#[test]
fn tst_acc_003_quota_checkpoint_and_weekly_latch() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    allow(&mut e);
    submit(&mut e, 1, &[]);
    let q = quota(100, 100);
    let s = call(e.apply(Event::Quota {
        auth_type: "chatgpt".into(),
        quota: Some(q),
        now: 10,
    }))
    .unwrap();
    assert_eq!(s.pause, Pause::Quota);
    assert_eq!(s.cadence, Cadence::WeeklyLatched);
    assert_eq!(s.next_wake, Some(1000));
    let s = call(e.apply(Event::Quota {
        auth_type: "chatgpt".into(),
        quota: Some(quota(0, 0)),
        now: 1000,
    }))
    .unwrap();
    assert_eq!(s.cadence, Cadence::WeeklyLatched);
    assert_eq!(s.next_wake, Some(605800));
    assert_eq!(s.worker_launches, 0);
}

#[test]
fn tst_acc_004_host_ack_required_before_done() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    allow(&mut e);
    submit(&mut e, 1, &[]);
    let s = call(e.apply(Event::Claim {
        owner: "a".into(),
        now: 10,
    }))
    .unwrap();
    call(e.apply(Event::Implemented {
        owner: "a".into(),
        generation: s.generation,
        head: "h".into(),
    }))
    .unwrap();
    call(e.apply(Event::Verify {
        task: 1,
        head: "h".into(),
        passed: true,
    }))
    .unwrap();
    assert_eq!(
        call(e.apply(Event::Complete {
            task: 1,
            head: "h".into()
        }))
        .unwrap_err(),
        Error::InvalidTransition
    );
    finish_host(&mut e, 1, "h", "abc");
    let s = call(e.snapshot()).unwrap();
    assert_eq!(row(&s, 1).state, State::Done);
    assert!(s.pending_intents.is_empty());
}

#[test]
fn tst_acc_005_probe_triage_is_required() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    call(e.apply(Event::Suggest {
        key: "memory".into(),
        issue: 10,
    }))
    .unwrap();
    let s = call(e.snapshot()).unwrap();
    assert_eq!(s.suggestion_count, 1);
    assert!(s.tasks.is_empty());
    let s = call(e.apply(Event::AcceptSuggestion {
        key: "memory".into(),
        task: task(1, &[]),
    }))
    .unwrap();
    assert_eq!(s.tasks.len(), 1);
    assert_eq!(row(&s, 1).state, State::Ready);
}

#[test]
fn tst_beh_001_durable_enqueue_is_exact() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    submit(&mut e, 9, &[]);
    drop(e);
    let e = engine(&dir);
    let s = call(e.snapshot()).unwrap();
    assert_eq!(
        s.tasks,
        vec![Row {
            id: 9,
            state: State::Ready,
            attempts: 0,
            head: None
        }]
    );
    assert_eq!(s.active, None);
}

#[test]
fn tst_neg_001_duplicate_id_rejected_atomically() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    submit(&mut e, 1, &[]);
    let before = call(e.snapshot()).unwrap();
    assert_eq!(
        call(e.apply(Event::Submit(task(1, &[])))).unwrap_err(),
        Error::DuplicateTask
    );
    assert_eq!(call(e.snapshot()).unwrap(), before);
}

#[test]
fn tst_neg_002_unknown_dependency_and_cycle_rejected() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    assert_eq!(
        call(e.apply(Event::Submit(task(1, &[99])))).unwrap_err(),
        Error::InvalidInput
    );
    assert_eq!(
        call(e.apply(Event::Submit(task(1, &[1])))).unwrap_err(),
        Error::DependencyCycle
    );
    assert!(call(e.snapshot()).unwrap().tasks.is_empty());
}

#[test]
fn tst_bound_001_task_limit_has_no_partial_insert() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    for id in 1..=1024 {
        submit(&mut e, id, &[]);
    }
    assert_eq!(
        call(e.apply(Event::Submit(task(1025, &[])))).unwrap_err(),
        Error::Capacity
    );
    assert_eq!(call(e.snapshot()).unwrap().tasks.len(), 1024);
}

#[test]
fn tst_inv_001_second_owner_cannot_write() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    allow(&mut e);
    submit(&mut e, 1, &[]);
    let s = call(e.apply(Event::Claim {
        owner: "a".into(),
        now: 10,
    }))
    .unwrap();
    let mut other = engine(&dir);
    assert_eq!(
        call(other.apply(Event::Claim {
            owner: "b".into(),
            now: 11
        }))
        .unwrap_err(),
        Error::Busy
    );
    assert_eq!(
        call(other.apply(Event::Checkpoint {
            owner: "b".into(),
            generation: s.generation,
            head: "bad".into()
        }))
        .unwrap_err(),
        Error::StaleOwner
    );
    assert_eq!(row(&call(e.snapshot()).unwrap(), 1).head, None);
}

#[test]
fn tst_state_001_stale_generation_cannot_checkpoint() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    allow(&mut e);
    submit(&mut e, 1, &[]);
    let s = call(e.apply(Event::Claim {
        owner: "a".into(),
        now: 10,
    }))
    .unwrap();
    call(e.apply(Event::Release {
        owner: "a".into(),
        generation: s.generation,
    }))
    .unwrap();
    let next = call(e.apply(Event::Claim {
        owner: "b".into(),
        now: 11,
    }))
    .unwrap();
    assert!(next.generation > s.generation);
    assert_eq!(
        call(e.apply(Event::Checkpoint {
            owner: "a".into(),
            generation: s.generation,
            head: "bad".into()
        }))
        .unwrap_err(),
        Error::StaleOwner
    );
}

#[test]
fn tst_neg_003_ambiguous_survivor_blocks_requeue() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    allow(&mut e);
    submit(&mut e, 1, &[]);
    call(e.apply(Event::Claim {
        owner: "a".into(),
        now: 10,
    }))
    .unwrap();
    let s = call(e.apply(Event::Reconcile {
        process_alive: true,
        process_identity_matches: false,
        descendants: DescendantStatus::Unknown,
    }))
    .unwrap();
    assert_eq!(s.pause, Pause::AmbiguousProcess);
    assert_eq!(s.active, Some(1));
    assert_eq!(
        call(e.apply(Event::Claim {
            owner: "b".into(),
            now: 100000
        }))
        .unwrap_err(),
        Error::AmbiguousSurvivor
    );
}

#[test]
fn tst_state_002_disable_then_cancel_is_durable() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    allow(&mut e);
    submit(&mut e, 1, &[]);
    let s = call(e.apply(Event::Enable(false))).unwrap();
    assert_eq!(s.pause, Pause::Disabled);
    assert_eq!(
        call(e.apply(Event::Claim {
            owner: "a".into(),
            now: 10
        }))
        .unwrap_err(),
        Error::InvalidTransition
    );
    call(e.apply(Event::Cancel(1))).unwrap();
    drop(e);
    assert_eq!(
        row(&call(engine(&dir).snapshot()).unwrap(), 1).state,
        State::Cancelled
    );
}

#[test]
fn tst_bound_002_repair_cap_and_deadline_are_exact() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    submit(&mut e, 1, &[]);
    assert_eq!(
        call(e.apply(Event::Repair {
            task: 1,
            now: 20,
            deadline: 20
        }))
        .unwrap_err(),
        Error::Deadline
    );
    for _ in 0..2 {
        call(e.apply(Event::Repair {
            task: 1,
            now: 10,
            deadline: 20,
        }))
        .unwrap();
    }
    assert_eq!(
        call(e.apply(Event::Repair {
            task: 1,
            now: 10,
            deadline: 20
        }))
        .unwrap_err(),
        Error::RepairExhausted
    );
}

#[test]
fn tst_anti_001_api_auth_never_launches() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    assert_eq!(
        call(e.apply(Event::Quota {
            auth_type: "apiKey".into(),
            quota: Some(quota(0, 0)),
            now: 1
        }))
        .unwrap_err(),
        Error::UnsupportedAuth
    );
    let s = call(e.snapshot()).unwrap();
    assert_eq!(s.worker_launches, 0);
    assert_eq!(s.pause, Pause::Authentication);
}

#[test]
fn tst_neg_004_missing_quota_fails_closed() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    assert_eq!(
        call(e.apply(Event::Quota {
            auth_type: "chatgpt".into(),
            quota: None,
            now: 1
        }))
        .unwrap_err(),
        Error::MissingQuota
    );
    assert_eq!(call(e.snapshot()).unwrap().worker_launches, 0);
}

#[test]
fn tst_bound_003_five_hour_reset_is_reported_value() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    let s = call(e.apply(Event::Quota {
        auth_type: "chatgpt".into(),
        quota: Some(quota(100, 99)),
        now: 10,
    }))
    .unwrap();
    assert_eq!(s.cadence, Cadence::FiveHour);
    assert_eq!(s.next_wake, Some(100));
    assert_eq!(s.pause, Pause::Quota);
}

#[test]
fn tst_prop_001_quota_all_percentages_preserve_weekly_latch() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    call(e.apply(Event::Quota {
        auth_type: "chatgpt".into(),
        quota: Some(quota(100, 100)),
        now: 1,
    }))
    .unwrap();
    for percent in 0..=100 {
        let s = call(e.apply(Event::Quota {
            auth_type: "chatgpt".into(),
            quota: Some(quota(percent, percent)),
            now: 20,
        }))
        .unwrap();
        assert_eq!(s.cadence, Cadence::WeeklyLatched);
    }
    drop(e);
    assert_eq!(
        call(engine(&dir).snapshot()).unwrap().cadence,
        Cadence::WeeklyLatched
    );
}

#[test]
fn tst_contract_001_quota_wire_ignores_order_and_rejects_malformed() {
    let q=call(parse_quota(r#"{"rateLimitsByLimitId":{"codex":{"secondary":{"resetsAt":1000,"windowDurationMins":10080,"usedPercent":7},"primary":{"resetsAt":100,"windowDurationMins":300,"usedPercent":42}}}}"#)).unwrap();
    assert_eq!(q, quota(42, 7));
    assert_eq!(
        call(parse_quota(r#"{"rateLimits":{}}"#)).unwrap_err(),
        Error::MissingQuota
    );
    assert_eq!(call(parse_quota(r#"{"rateLimitsByLimitId":{"codex":{"primary":{"usedPercent":101,"windowDurationMins":300,"resetsAt":100}}}}"#)).unwrap_err(),Error::MalformedProtocol);
}

#[test]
fn tst_contract_002_account_wire_requires_managed_chatgpt() {
    assert_eq!(
        call(parse_account_type(
            r#"{"account":{"type":"chatgpt","planType":"plus"},"requiresOpenaiAuth":true}"#
        ))
        .unwrap(),
        "chatgpt"
    );
    assert_eq!(
        call(parse_account_type(r#"{"account":{"type":"apiKey"}}"#)).unwrap_err(),
        Error::UnsupportedAuth
    );
    assert_eq!(
        call(parse_account_type(
            r#"{"account":null,"requiresOpenaiAuth":true}"#
        ))
        .unwrap_err(),
        Error::AuthenticationRequired
    );
}

#[test]
fn tst_contract_003_worker_plan_enforces_subscription_and_model() {
    let dir = Dir::new();
    let e = engine(&dir);
    let plan = call(e.plan_worker(Path::new("C:/tools/codex.exe"), "gpt-6-luna")).unwrap();
    assert_eq!(plan.executable, PathBuf::from("C:/tools/codex.exe"));
    assert_eq!(plan.model, "gpt-6-luna");
    assert!(
        plan.argv
            .windows(2)
            .any(|x| x == ["--config", "forced_login_method=\"chatgpt\""])
    );
    assert!(
        plan.argv
            .windows(2)
            .any(|x| x == ["--config", "model_provider=\"openai\""])
    );
    for name in ["OPENAI_API_KEY", "CODEX_API_KEY"] {
        assert!(plan.removed_environment_names.iter().any(|n| n == name));
    }
    assert!(
        !plan
            .argv
            .iter()
            .any(|x| x == "--dangerously-bypass-approvals-and-sandbox")
    );
    assert_eq!(
        call(e.plan_worker(Path::new("C:/tools/codex.exe"), "unapproved")).unwrap_err(),
        Error::InvalidInput
    );
}

#[test]
fn tst_beh_002_actual_worker_exit_is_not_success() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    let runtime = runtime(&dir, "nonzero");
    let _scope1 = native_submit(&mut e, &dir, &runtime, 1, &[], "workspace_write");
    let out = call(e.drive_once(&runtime)).unwrap();
    assert_eq!(out.terminal, "failed");
    assert_eq!(out.exit_code, Some(7));
    assert_eq!(out.launches, 1);
    assert_eq!(out.task, Some(1));
    assert!(dir.0.join("checkout/fake/launch.txt").exists());
}

#[test]
fn tst_contract_004_bridge_writes_durable_typed_intent() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    submit(&mut e, 1, &[]);
    let intent = Intent {
        id: "pr:1:abc".into(),
        task: 1,
        kind: IntentKind::CreatePr,
        expected_head: "abc".into(),
    };
    call(e.apply(Event::Intent(intent.clone()))).unwrap();
    let exported = call(e.write_bridge(&dir.0.join("bridge"))).unwrap();
    assert_eq!(exported, vec![intent.clone()]);
    let wire = std::fs::read_to_string(dir.0.join("bridge/outbox/pr_1_abc.json")).unwrap();
    assert!(wire.contains("\"schema_version\":1"));
    assert!(wire.contains("\"id\":\"pr:1:abc\""));
    drop(e);
    let e = engine(&dir);
    assert_eq!(call(e.snapshot()).unwrap().pending_intents, vec![intent]);
}

#[test]
fn tst_neg_005_ack_head_mismatch_cannot_merge() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    submit(&mut e, 1, &[]);
    let intent = Intent {
        id: "pr:1:h".into(),
        task: 1,
        kind: IntentKind::CreatePr,
        expected_head: "h".into(),
    };
    call(e.apply(Event::Intent(intent))).unwrap();
    assert_eq!(
        call(e.apply(Event::Ack(Ack {
            id: "pr:1:h".into(),
            expected_head: "h".into(),
            observed_head: "other".into(),
            remote_url: "https://github.com/zblack54/fusion-lab/pull/1".into(),
            outcome: AckOutcome::Applied,
            evidence: Some(HostEvidence::Pr {
                head: "other".into(),
                pr_url: "https://github.com/zblack54/fusion-lab/pull/1".into(),
                issue: 4,
                scope_digest: "abc".into()
            }),
        })))
        .unwrap_err(),
        Error::EvidenceMismatch
    );
    assert_eq!(call(e.snapshot()).unwrap().pending_intents.len(), 1);
}

#[test]
fn tst_inv_002_uncertain_remote_effect_is_not_replayed() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    submit(&mut e, 1, &[]);
    let intent = Intent {
        id: "pr:1:h".into(),
        task: 1,
        kind: IntentKind::CreatePr,
        expected_head: "h".into(),
    };
    call(e.apply(Event::Intent(intent.clone()))).unwrap();
    call(e.apply(Event::Ack(Ack {
        id: "pr:1:h".into(),
        expected_head: "h".into(),
        observed_head: "h".into(),
        remote_url: "".into(),
        outcome: AckOutcome::Uncertain,
        evidence: None,
    })))
    .unwrap();
    assert_eq!(
        call(e.apply(Event::Intent(intent))).unwrap_err(),
        Error::UncertainEffect
    );
    assert_eq!(call(e.snapshot()).unwrap().pending_intents.len(), 1);
}

#[test]
fn tst_state_003_changed_head_invalidates_verification() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    allow(&mut e);
    submit(&mut e, 1, &[]);
    let s = call(e.apply(Event::Claim {
        owner: "a".into(),
        now: 1,
    }))
    .unwrap();
    call(e.apply(Event::Implemented {
        owner: "a".into(),
        generation: s.generation,
        head: "a".into(),
    }))
    .unwrap();
    call(e.apply(Event::Verify {
        task: 1,
        head: "a".into(),
        passed: true,
    }))
    .unwrap();
    let s = call(e.apply(Event::ChangeHead {
        task: 1,
        head: "b".into(),
    }))
    .unwrap();
    assert_eq!(row(&s, 1).state, State::ImplementedUnverified);
    assert_eq!(
        call(e.apply(Event::Complete {
            task: 1,
            head: "a".into()
        }))
        .unwrap_err(),
        Error::EvidenceMismatch
    );
}

#[test]
fn tst_perf_001_suggestion_work_is_bounded_and_deduplicated() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    for _ in 0..1000 {
        call(e.apply(Event::Suggest {
            key: "same".into(),
            issue: 10,
        }))
        .unwrap();
    }
    assert_eq!(call(e.snapshot()).unwrap().suggestion_count, 1);
    for id in 1..128 {
        call(e.apply(Event::Suggest {
            key: format!("k{id}"),
            issue: 10,
        }))
        .unwrap();
    }
    assert_eq!(
        call(e.apply(Event::Suggest {
            key: "overflow".into(),
            issue: 10
        }))
        .unwrap_err(),
        Error::Capacity
    );
    assert_eq!(call(e.snapshot()).unwrap().suggestion_count, 128);
}

#[test]
fn tst_beh_003_bridge_rejects_unknown_ack_atomically() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    let bridge = dir.0.join("bridge");
    std::fs::create_dir_all(bridge.join("inbox")).unwrap();
    std::fs::write(bridge.join("inbox/unknown.json"),r#"{"schema_version":1,"id":"missing","expected_head":"h","observed_head":"h","remote_url":"","outcome":"applied"}"#).unwrap();
    let before = call(e.snapshot()).unwrap();
    assert_eq!(call(e.read_bridge(&bridge)).unwrap_err(), Error::InvalidAck);
    assert_eq!(call(e.snapshot()).unwrap(), before);
}

#[test]
fn tst_struct_001_downstream_types_and_initial_state() {
    let dir = Dir::new();
    fn send<T: Send>() {}
    send::<Task>();
    send::<Quota>();
    send::<Intent>();
    let e = engine(&dir);
    let s = call(e.snapshot()).unwrap();
    assert!(s.tasks.is_empty());
    assert_eq!(s.active, None);
    assert_eq!(s.worker_launches, 0);
}

#[test]
fn tst_acc_006_cli_file_bridge_end_to_end() {
    let dir = Dir::new();
    let first = cli(
        &dir,
        &["enqueue", "--task", "1", "--issue", "4", "--scope", "abc"],
    );
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(first.stdout).unwrap().trim(),
        r#"{"accepted":1}"#
    );
    let status = cli(&dir, &["status"]);
    assert_eq!(status.status.code(), Some(0));
    let text = String::from_utf8(status.stdout).unwrap();
    assert!(text.contains("\"id\":1"));
    assert!(text.contains("\"state\":\"ready\""));
    let disabled = cli(&dir, &["disable"]);
    assert_eq!(disabled.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(disabled.stdout).unwrap().trim(),
        r#"{"enabled":false}"#
    );
}

#[test]
fn tst_neg_006_cli_invalid_command_does_not_mutate() {
    let dir = Dir::new();
    let output = cli(
        &dir,
        &["enqueue", "--task", "0", "--issue", "4", "--scope", "abc"],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(!dir.0.join("supervisor.sqlite3").exists());
}

#[test]
fn tst_anti_002_diagnostic_seam_is_absent_from_real_status() {
    let dir = Dir::new();
    let output = cli(&dir, &["status"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("SUPERVISOR_NOT_IMPLEMENTED"));
}
#[test]
fn tst_neg_008_gated_dispatch_never_launches_on_bad_account() {
    for scenario in ["unauth", "api", "zero", "missing"] {
        let dir = Dir::new();
        let mut e = engine(&dir);
        let runtime = runtime(&dir, scenario);
        let _scope1 = native_submit(&mut e, &dir, &runtime, 1, &[], "workspace_write");
        let result = call(e.drive_once(&runtime));
        let expected = match scenario {
            "unauth" => Error::AuthenticationRequired,
            "api" => Error::UnsupportedAuth,
            "zero" => Error::QuotaExhausted,
            _ => Error::MissingQuota,
        };
        assert_eq!(result.unwrap_err(), expected);
        assert!(!dir.0.join("checkout/fake/launch.txt").exists());
        assert_eq!(call(e.snapshot()).unwrap().worker_launches, 0);
    }
}
#[test]
fn tst_contract_005_actual_dispatch_uses_subscription_launch_policy() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    let mut runtime = runtime(&dir, "ok");
    let scope1 = native_submit(&mut e, &dir, &runtime, 1, &[], "workspace_write");
    runtime
        .environment
        .push(("CODEX_API_KEY".into(), "fixture-not-a-secret".into()));
    runtime
        .environment
        .push(("OPENAI_API_KEY".into(), "fixture-not-a-secret".into()));
    let initial_head = observed_head(&runtime.checkout);
    let report = call(e.drive_once(&runtime)).unwrap();
    assert_eq!(report.task, Some(1));
    assert_eq!(report.terminal, "awaiting_host");
    assert_eq!(report.checkpoint.as_deref(), Some("session-1"));
    let observed = std::fs::read_to_string(dir.0.join("checkout/fake/launch.txt")).unwrap();
    assert!(observed.contains("arg=forced_login_method=\"chatgpt\""));
    assert!(observed.contains("arg=model_provider=\"openai\""));
    assert!(observed.contains("arg=gpt-6-luna"));
    assert!(observed.contains("env_present_CODEX_API_KEY=false"));
    assert!(observed.contains("env_present_OPENAI_API_KEY=false"));
    assert!(observed.contains("approved_scope_present=true"));
    assert!(observed.contains(&format!("observed_expected_head={initial_head}")));
    assert!(observed.contains(&format!("scope_digest={scope1}")));
    assert!(
        observed.contains(&format!("observed_scope_sha256={scope1}")),
        "actual raw approved bytes must match independent file digest"
    );
    for option in [
        "sandbox_workspace_write.writable_roots=[]",
        "sandbox_workspace_write.exclude_tmpdir_env_var=true",
        "sandbox_workspace_write.exclude_slash_tmp=true",
    ] {
        assert!(observed.contains(option), "{observed}");
    }
}
#[test]
fn tst_bound_004_actual_child_deadline_keeps_task_unverified() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    let mut runtime = runtime(&dir, "slow");
    let _scope1 = native_submit(&mut e, &dir, &runtime, 1, &[], "workspace_write");
    runtime.worker_deadline_ms = 100;
    assert_eq!(call(e.drive_once(&runtime)).unwrap_err(), Error::Deadline);
    let s = call(e.snapshot()).unwrap();
    assert_ne!(row(&s, 1).state, State::Verified);
    assert_ne!(row(&s, 1).state, State::Done);
    assert!(dir.0.join("checkout/fake/launch.txt").exists());
}
#[test]
fn tst_acc_007_native_cycle_host_ack_then_next_ready() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    let runtime = runtime(&dir, "ok");
    let scope1 = native_submit(&mut e, &dir, &runtime, 1, &[], "workspace_write");
    native_submit(&mut e, &dir, &runtime, 2, &[1], "workspace_write");
    let first = call(e.drive_once(&runtime)).unwrap();
    assert_eq!(first.task, Some(1));
    assert_eq!(first.terminal, "awaiting_host");
    let state = call(e.snapshot()).unwrap();
    assert_eq!(row(&state, 1).state, State::ImplementedUnverified);
    assert_eq!(state.pending_intents.len(), 1);
    assert_eq!(state.pending_intents[0].kind, IntentKind::CreatePr);
    let path = dir.0.join("bridge");
    call(e.write_bridge(&path)).unwrap();
    assert!(path.join("outbox").exists());
    let head = row(&state, 1).head.clone().unwrap();
    call(e.apply(Event::Verify {
        task: 1,
        head: head.clone(),
        passed: true,
    }))
    .unwrap();
    finish_host(&mut e, 1, &head, &scope1);
    drop(e);
    let mut e = engine(&dir);
    let second = call(e.drive_once(&runtime)).unwrap();
    assert_eq!(second.task, Some(2));
    assert_eq!(second.checkpoint.as_deref(), Some("session-2"));
    let observed = std::fs::read_to_string(dir.0.join("checkout/fake/launch.txt")).unwrap();
    assert_eq!(observed.matches("task_id=1").count(), 1);
    assert_eq!(observed.matches("task_id=2").count(), 1);
}
#[test]
fn tst_state_004_active_cancel_requests_owned_child_shutdown() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    let runtime = runtime(&dir, "slow");
    let _scope1 = native_submit(&mut e, &dir, &runtime, 1, &[], "workspace_write");
    let state = dir.0.clone();
    let driver = std::thread::spawn(move || call(e.drive_once(&runtime)));
    let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !state.join("checkout/fake/launch.txt").exists() {
        assert!(
            std::time::Instant::now() < until,
            "fixture launch not observed"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let mut control = engine(&dir);
    call(control.apply(Event::Cancel(1))).unwrap();
    assert_eq!(driver.join().unwrap().unwrap_err(), Error::Cancelled);
    assert_eq!(
        row(&call(control.snapshot()).unwrap(), 1).state,
        State::Cancelled
    );
}

#[test]
fn tst_neg_007_failed_verification_cannot_complete() {
    let dir = Dir::new();
    let mut e = engine(&dir);
    allow(&mut e);
    submit(&mut e, 1, &[]);
    let s = call(e.apply(Event::Claim {
        owner: "a".into(),
        now: 1,
    }))
    .unwrap();
    call(e.apply(Event::Implemented {
        owner: "a".into(),
        generation: s.generation,
        head: "h".into(),
    }))
    .unwrap();
    let s = call(e.apply(Event::Verify {
        task: 1,
        head: "h".into(),
        passed: false,
    }))
    .unwrap();
    assert_eq!(row(&s, 1).state, State::ImplementedUnverified);
    assert_eq!(
        call(e.apply(Event::Complete {
            task: 1,
            head: "h".into()
        }))
        .unwrap_err(),
        Error::InvalidTransition
    );
}

const WORKER_POLICY_V1: &str = "Act only on the host-approved scope in approved_scope. Match task_id, issue and checkout; treat its instructions and non_goals as the work boundary. Read AGENTS.md and any referenced approved contract. Preserve locked tests and stop on a contract conflict. Use only managed ChatGPT subscription authentication. Do not use API keys, alternate providers, paid APIs, deployment, live trading or host GitHub mutations. Run declared verification and report actual failures. Return the final schema with observed Git head and current explicit session checkpoint.";
fn reject_scope(
    d: &Dir,
    runtime: &RuntimeConfig,
    bytes: Option<&[u8]>,
    digest_override: Option<&str>,
    expected: Error,
) {
    let digest = bytes
        .map(|bytes| scope_bytes(d, bytes))
        .unwrap_or_else(|| "0".repeat(64));
    let mut e = engine(d);
    call(e.apply(Event::Submit(Task {
        id: 1,
        issue: 4,
        dependencies: vec![],
        scope_digest: digest_override.unwrap_or(&digest).into(),
    })))
    .unwrap();
    let before = call(e.snapshot()).unwrap();
    assert_eq!(call(e.drive_once(runtime)).unwrap_err(), expected);
    assert_eq!(
        call(e.snapshot()).unwrap(),
        before,
        "scope rejection must preserve all committed task/control state"
    );
    assert_eq!(before.worker_launches, 0);
    assert!(before.pending_intents.is_empty());
    assert!(
        !runtime.checkout.join("fake/launch.txt").exists(),
        "invalid scope cannot invoke the real worker boundary"
    );
    assert!(
        !runtime
            .checkout
            .join(".git/fusion-supervisor-active.json")
            .exists(),
        "invalid scope cannot reserve a native attempt"
    );
}
#[test]
fn tst_neg_013_missing_approved_scope_cannot_launch() {
    let d = Dir::new();
    let r = runtime(&d, "ok");
    reject_scope(&d, &r, None, None, Error::InvalidInput);
}
#[test]
fn tst_neg_014_altered_scope_hash_cannot_launch() {
    let d = Dir::new();
    let r = runtime(&d, "ok");
    let original = scope_document(&r, 1, "workspace_write");
    let digest = materialize(&d, &original);
    let mut altered = original;
    altered["instructions"] = json!("unapproved alteration");
    std::fs::write(
        d.0.join("approved-scopes").join(format!("{digest}.json")),
        serde_json::to_vec(&altered).unwrap(),
    )
    .unwrap();
    let mut e = engine(&d);
    call(e.apply(Event::Submit(Task {
        id: 1,
        issue: 4,
        dependencies: vec![],
        scope_digest: digest,
    })))
    .unwrap();
    let before = call(e.snapshot()).unwrap();
    assert_eq!(call(e.drive_once(&r)).unwrap_err(), Error::EvidenceMismatch);
    assert_eq!(call(e.snapshot()).unwrap(), before);
    assert!(!r.checkout.join("fake/launch.txt").exists());
    assert_eq!(before.worker_launches, 0);
    assert!(before.pending_intents.is_empty());
}
#[test]
fn tst_neg_015_scope_identity_must_match_task_issue_checkout() {
    for field in ["task_id", "issue", "canonical_checkout"] {
        let d = Dir::new();
        let r = runtime(&d, "ok");
        let mut doc = scope_document(&r, 1, "workspace_write");
        doc[field] = match field {
            "task_id" => json!(2),
            "issue" => json!(99),
            _ => json!(std::fs::canonicalize(&d.0).unwrap().to_string_lossy()),
        };
        reject_scope(
            &d,
            &r,
            Some(&serde_json::to_vec(&doc).unwrap()),
            None,
            Error::EvidenceMismatch,
        );
    }
}
#[test]
fn tst_neg_016_scope_schema_is_strict_and_versioned() {
    for case in [
        "unknown",
        "missing",
        "empty",
        "wrong_type",
        "mode",
        "version",
        "list_type",
        "utf8",
        "malformed",
    ] {
        let d = Dir::new();
        let r = runtime(&d, "ok");
        let mut doc = scope_document(&r, 1, "workspace_write");
        match case {
            "unknown" => doc["unauthorized"] = json!(true),
            "missing" => {
                doc.as_object_mut().unwrap().remove("instructions");
            }
            "empty" => doc["instructions"] = json!(""),
            "wrong_type" => doc["task_id"] = json!("1"),
            "mode" => doc["mode"] = json!("danger_full_access"),
            "version" => doc["schema_version"] = json!(2),
            "list_type" => doc["non_goals"] = json!("not an array"),
            _ => {}
        }
        let bytes = match case {
            "utf8" => vec![255],
            "malformed" => b"{".to_vec(),
            _ => serde_json::to_vec(&doc).unwrap(),
        };
        reject_scope(&d, &r, Some(&bytes), None, Error::InvalidInput);
    }
}
#[test]
fn tst_bound_005_scope_file_is_bounded_before_launch() {
    let d = Dir::new();
    let r = runtime(&d, "ok");
    let mut doc = scope_document(&r, 1, "workspace_write");
    doc["instructions"] = json!("a".repeat(32768));
    let bytes = serde_json::to_vec(&doc).unwrap();
    assert!(bytes.len() > 32768);
    reject_scope(&d, &r, Some(&bytes), None, Error::Capacity);
}
#[test]
fn tst_bound_006_encoded_prompt_is_bounded_before_launch() {
    let d = Dir::new();
    let r = runtime(&d, "ok");
    let mut doc = scope_document(&r, 1, "workspace_write");
    doc["instructions"] = json!("");
    let overhead = serde_json::to_vec(&doc).unwrap().len();
    doc["instructions"] = json!("\\".repeat((32768 - overhead) / 2));
    let bytes = serde_json::to_vec(&doc).unwrap();
    assert!(bytes.len() <= 32768);
    let raw = std::str::from_utf8(&bytes).unwrap();
    let minimum_prompt = json!({"task_id":1,"issue":4,"scope_digest":"a".repeat(64),"expected_head":"b".repeat(40),"approved_scope":raw,"instructions":WORKER_POLICY_V1});
    assert!(
        serde_json::to_vec(&minimum_prompt).unwrap().len() > 65536,
        "the independent byte oracle must exercise the final prompt cap"
    );
    reject_scope(&d, &r, Some(&bytes), None, Error::Capacity);
}
#[test]
fn tst_anti_003_workspace_worker_cannot_write_state_control() {
    let outer = Dir::new();
    let r = runtime(&outer, "ok");
    let d = Dir(r.checkout.join("state"));
    std::fs::create_dir_all(&d.0).unwrap();
    let bytes = serde_json::to_vec(&scope_document(&r, 1, "workspace_write")).unwrap();
    reject_scope(&d, &r, Some(&bytes), None, Error::InvalidInput);
}
#[test]
fn tst_anti_004_workspace_worker_cannot_write_host_bridge() {
    let d = Dir::new();
    let mut r = runtime(&d, "ok");
    r.bridge = r.checkout.join("bridge");
    let bytes = serde_json::to_vec(&scope_document(&r, 1, "workspace_write")).unwrap();
    reject_scope(&d, &r, Some(&bytes), None, Error::InvalidInput);
}
#[test]
fn tst_anti_005_canonical_alias_cannot_hide_control_overlap() {
    let d = Dir::new();
    let mut r = runtime_at(&d, "ok", d.0.clone());
    std::fs::create_dir_all(d.0.join("alias")).unwrap();
    r.checkout = d.0.join("alias/..");
    let bytes = serde_json::to_vec(&scope_document(&r, 1, "workspace_write")).unwrap();
    reject_scope(&d, &r, Some(&bytes), None, Error::InvalidInput);
}
#[test]
fn tst_acc_009_readonly_scope_supports_legacy_same_path() {
    let d = Dir::new();
    let r = runtime_at(&d, "ok", d.0.clone());
    let mut e = engine(&d);
    let initial_head = observed_head(&r.checkout);
    let digest = native_submit(&mut e, &d, &r, 1, &[], "read_only");
    let result = call(e.drive_once(&r)).unwrap();
    assert_eq!(result.task, Some(1));
    assert_eq!(result.terminal, "awaiting_host");
    assert_eq!(result.launches, 1);
    let observed = std::fs::read_to_string(d.0.join("fake/launch.txt")).unwrap();
    assert!(
        observed.contains("arg=sandbox_mode=\"read-only\""),
        "{observed}"
    );
    assert!(observed.contains("approved_scope_present=true"));
    assert_eq!(
        observed_head(&r.checkout),
        initial_head,
        "read-only peer must leave Git HEAD unchanged"
    );
    assert!(observed.contains(&format!("observed_expected_head={initial_head}")));
    assert!(observed.contains(&format!("scope_digest={digest}")));
    assert!(
        observed.contains(&format!("observed_scope_sha256={digest}")),
        "actual raw approved bytes must match independent file digest"
    );
}

#[test]
fn tst_neg_017_cli_workspace_without_isolated_checkout_cannot_launch() {
    let d = Dir::new();
    let r = runtime_at(&d, "ok", d.0.clone());
    let digest = materialize(&d, &scope_document(&r, 1, "workspace_write"));
    assert_eq!(
        cli(
            &d,
            &["enqueue", "--task", "1", "--issue", "4", "--scope", &digest]
        )
        .status
        .code(),
        Some(0)
    );
    let before = String::from_utf8(cli(&d, &["status"]).stdout).unwrap();
    let env = format!("FAKE_CODEX_ROOT={}", d.0.join("fake").display());
    let result = cli(
        &d,
        &[
            "step",
            "--codex",
            r.codex_executable.to_str().unwrap(),
            "--model",
            "gpt-6-luna",
            "--child-env",
            &env,
        ],
    );
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(cli(&d, &["status"]).stdout).unwrap(),
        before
    );
    assert!(!d.0.join("fake/launch.txt").exists());
}
#[test]
fn tst_acc_010_cli_readonly_legacy_checkout_is_explicitly_sandboxed() {
    let d = Dir::new();
    let r = runtime_at(&d, "ok", d.0.clone());
    let initial_head = observed_head(&r.checkout);
    let digest = materialize(&d, &scope_document(&r, 1, "read_only"));
    assert_eq!(
        cli(
            &d,
            &["enqueue", "--task", "1", "--issue", "4", "--scope", &digest]
        )
        .status
        .code(),
        Some(0)
    );
    let env = format!("FAKE_CODEX_ROOT={}", d.0.join("fake").display());
    let result = cli(
        &d,
        &[
            "step",
            "--codex",
            r.codex_executable.to_str().unwrap(),
            "--model",
            "gpt-6-luna",
            "--child-env",
            &env,
        ],
    );
    assert_eq!(result.status.code(), Some(0));
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("\"terminal\":\"awaiting_host\"")
    );
    let observed = std::fs::read_to_string(d.0.join("fake/launch.txt")).unwrap();
    assert!(observed.contains("arg=sandbox_mode=\"read-only\""));
    assert!(observed.contains("approved_scope_present=true"));
    assert_eq!(
        observed_head(&r.checkout),
        initial_head,
        "read-only peer must leave Git HEAD unchanged"
    );
    assert!(observed.contains(&format!("observed_expected_head={initial_head}")));
    assert!(observed.contains(&format!("scope_digest={digest}")));
    assert!(
        observed.contains(&format!("observed_scope_sha256={digest}")),
        "actual raw approved bytes must match independent file digest"
    );
}

#[test]
fn tst_neg_018_readonly_peer_mutation_is_not_accepted_or_reverted() {
    let d = Dir::new();
    let r = runtime(&d, "readonly_mutates");
    let initial_head = observed_head(&r.checkout);
    let mut e = engine(&d);
    native_submit(&mut e, &d, &r, 1, &[], "read_only");
    assert_eq!(call(e.drive_once(&r)).unwrap_err(), Error::EvidenceMismatch);
    let changed_head = observed_head(&r.checkout);
    assert_ne!(
        changed_head, initial_head,
        "controlled peer must actually commit the forbidden mutation"
    );
    let state = call(e.snapshot()).unwrap();
    assert_eq!(
        state.worker_launches, 1,
        "postlaunch rejection is not a no-launch case"
    );
    assert_eq!(state.pause, Pause::EvidenceMismatch);
    assert_eq!(
        state.active, None,
        "known reaped worker is not a surviving-process ambiguity"
    );
    assert_eq!(row(&state, 1).state, State::Paused);
    assert_eq!(
        row(&state, 1).head.as_deref(),
        Some(changed_head.as_str()),
        "rejected mutation head must remain observable evidence, not an accepted implementation"
    );
    assert_ne!(row(&state, 1).state, State::ImplementedUnverified);
    assert_ne!(row(&state, 1).state, State::Verified);
    assert_ne!(row(&state, 1).state, State::Done);
    assert!(state.pending_intents.is_empty());
    assert!(
        r.checkout.join("fixture-task-1.txt").exists(),
        "mutation evidence must not be silently reverted"
    );
    drop(e);
    let mut reopened = engine(&d);
    assert_eq!(
        call(reopened.snapshot()).unwrap().pause,
        Pause::EvidenceMismatch
    );
    assert_eq!(
        observed_head(&r.checkout),
        changed_head,
        "restart must preserve observed mutation"
    );
    assert_eq!(
        call(reopened.drive_once(&r)).unwrap_err(),
        Error::EvidenceMismatch
    );
    assert_eq!(
        call(reopened.snapshot()).unwrap().worker_launches,
        1,
        "evidence pause must not launch another worker"
    );
    assert_eq!(
        call(reopened.apply(Event::Enable(true))).unwrap_err(),
        Error::EvidenceMismatch
    );
    assert_eq!(
        call(reopened.apply(Event::Tick { now: u64::MAX })).unwrap_err(),
        Error::EvidenceMismatch
    );
    assert_eq!(
        call(reopened.snapshot()).unwrap().pause,
        Pause::EvidenceMismatch
    );
    assert_eq!(observed_head(&r.checkout), changed_head);
}
