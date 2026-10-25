use fusion_experiment_service::{Service, ServiceOptions};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
#[test]
fn tst_acc_002_invalid_requests() {
    tst_neg_001_invalid_configuration_has_no_run();
    tst_bound_001_request_limit();
    tst_struct_001_non_loopback_rejected();
}
#[test]
fn tst_acc_003_lifecycle() {
    tst_state_001_cancel_is_terminal();
    tst_inv_001_single_active_admission();
    tst_perf_001_shutdown_bounded();
    tst_state_002_worker_failure_releases_slot();
}
#[test]
fn tst_acc_004_retained_artifacts() {
    tst_beh_001_retention_evicts_oldest();
    tst_anti_001_path_traversal_denied();
    tst_bound_002_insufficient_artifact_budget();
}
struct Fixture {
    service: Option<Service>,
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        Self::with_retention(2)
    }
    fn with_retention(retained_runs: usize) -> Self {
        Self::configured(retained_runs, 768 * 1024 * 1024, false)
    }
    fn controlled() -> Self {
        Self::configured(2, 768 * 1024 * 1024, true)
    }
    fn configured(retained_runs: usize, artifact_budget_bytes: u64, controlled: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "fusion-service-contract-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let worker = if controlled {
            PathBuf::from(env!("CARGO_BIN_EXE_controlled_worker"))
        } else {
            let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            let result = Command::new("cargo")
                .args(["build", "--locked", "--bin", "fusion-lab"])
                .current_dir(&repo)
                .status()
                .unwrap();
            assert!(result.success(), "real numerical CLI must build");
            repo.join("target/debug").join(if cfg!(windows) {
                "fusion-lab.exe"
            } else {
                "fusion-lab"
            })
        };
        let service = Service::start(ServiceOptions {
            bind: "127.0.0.1:0".parse().unwrap(),
            artifact_root: root.clone(),
            worker,
            retained_runs,
            artifact_budget_bytes,
        })
        .unwrap();
        Self {
            service: Some(service),
            root,
        }
    }
    fn ready_worker(&self) -> SocketAddr {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            for child in fs::read_dir(&self.root).unwrap() {
                let marker = child.unwrap().path().join("worker.ready");
                if let Ok(text) = fs::read_to_string(marker)
                    && let Ok(address) = text.parse::<SocketAddr>()
                {
                    assert!(TcpStream::connect_timeout(&address, Duration::from_secs(1)).is_ok());
                    return address;
                }
            }
            assert!(
                Instant::now() < deadline,
                "controlled child must be confirmed alive"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn address(&self) -> SocketAddr {
        self.service.as_ref().unwrap().address()
    }
    fn request(&self, method: &str, path: &str, body: &str) -> (u16, Vec<u8>) {
        request(self.address(), method, path, body)
    }
    fn submit(&self, config: Value) -> String {
        let (status, bytes) = self.request("POST", "/v1/runs", &config.to_string());
        assert_eq!(
            status,
            202,
            "valid run accepted: {}",
            String::from_utf8_lossy(&bytes)
        );
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["state"], "running");
        value["id"].as_str().unwrap().to_owned()
    }
    fn wait(&self, id: &str) -> Value {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let (status, bytes) = self.request("GET", &format!("/v1/runs/{id}"), "");
            assert_eq!(status, 200);
            let value: Value = serde_json::from_slice(&bytes).unwrap();
            if value["state"] != "running" {
                return value;
            }
            assert!(
                Instant::now() < deadline,
                "run must terminate within test deadline"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(service) = self.service.take() {
            let _ = service.shutdown();
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn request(address: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Vec<u8>) {
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(5)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(stream, "{method} {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    let split = bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let header = String::from_utf8_lossy(&bytes[..split]);
    let status = header.split_whitespace().nth(1).unwrap().parse().unwrap();
    let chunked = header
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked");
    if !path.ends_with("/events") {
        assert!(!chunked, "finite API replies use content-length");
    }
    let mut body = bytes[split + 4..].to_vec();
    if chunked {
        let mut decoded = Vec::new();
        let mut offset = 0;
        loop {
            let end = body[offset..]
                .windows(2)
                .position(|w| w == b"\r\n")
                .unwrap()
                + offset;
            let size = usize::from_str_radix(
                std::str::from_utf8(&body[offset..end])
                    .unwrap()
                    .split(';')
                    .next()
                    .unwrap(),
                16,
            )
            .unwrap();
            offset = end + 2;
            if size == 0 {
                break;
            }
            decoded.extend_from_slice(&body[offset..offset + size]);
            offset += size + 2;
        }
        body = decoded;
    }
    (status, body)
}
fn config(duration: u32) -> Value {
    json!({"seed":42,"duration_seconds":duration,"scenario":"straight","position_sigma":0.0,"velocity_sigma":0.0,"acceleration_density":0.1,"outage":null})
}

/// Incremental decoder keeps the SAME TCP/SSE subscription through a transition.
struct EventStream {
    reader: BufReader<TcpStream>,
    chunked: bool,
    remaining: usize,
    chunk_end: bool,
    ended: bool,
}
impl EventStream {
    fn open(address: SocketAddr, id: &str) -> Self {
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        write!(
            stream,
            "GET /v1/runs/{id}/events HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(line.split_whitespace().nth(1), Some("200"));
        let mut chunked = false;
        let mut event_type = false;
        loop {
            line.clear();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
            let lower = line.to_ascii_lowercase();
            chunked |= lower.starts_with("transfer-encoding:") && lower.contains("chunked");
            event_type |= lower.starts_with("content-type:") && lower.contains("text/event-stream");
        }
        assert!(event_type, "live subscription must use SSE content type");
        Self {
            reader,
            chunked,
            remaining: 0,
            chunk_end: false,
            ended: false,
        }
    }
    fn byte(&mut self) -> Option<u8> {
        if self.ended {
            return None;
        }
        if self.chunked && self.remaining == 0 {
            if self.chunk_end {
                let mut ending = [0u8; 2];
                self.reader.read_exact(&mut ending).unwrap();
                assert_eq!(&ending, b"\r\n");
            }
            let mut line = String::new();
            self.reader.read_line(&mut line).unwrap();
            self.remaining =
                usize::from_str_radix(line.trim().split(';').next().unwrap(), 16).unwrap();
            if self.remaining == 0 {
                self.ended = true;
                return None;
            }
            self.chunk_end = true;
        }
        let mut byte = [0u8; 1];
        match self.reader.read(&mut byte).unwrap() {
            0 => {
                self.ended = true;
                None
            }
            _ => {
                if self.chunked {
                    self.remaining -= 1;
                }
                Some(byte[0])
            }
        }
    }
    fn snapshot(&mut self) -> (u64, Value) {
        let mut line = String::new();
        let mut event = String::new();
        let mut id = None;
        let mut data = None;
        loop {
            let byte = self
                .byte()
                .expect("existing SSE subscription must deliver the next snapshot before closing");
            if byte != b'\n' {
                line.push(char::from(byte));
                continue;
            }
            let text = line.trim_end_matches('\r');
            if text.is_empty() {
                if event == "snapshot" {
                    return (
                        id.expect("snapshot has SSE id"),
                        data.expect("snapshot has JSON data"),
                    );
                }
                event.clear();
                id = None;
                data = None;
            } else if let Some(value) = text.strip_prefix("event:") {
                event = value.trim_start().to_owned();
            } else if let Some(value) = text.strip_prefix("id:") {
                id = Some(value.trim().parse().unwrap());
            } else if let Some(value) = text.strip_prefix("data:") {
                data = Some(serde_json::from_str(value.trim_start()).unwrap());
            }
            line.clear();
        }
    }
}

#[test]
fn tst_state_003_existing_stream_observes_cancellation() {
    let f = Fixture::controlled();
    let id = f.submit(config(3600));
    let child = f.ready_worker();
    let mut stream = EventStream::open(f.address(), &id);
    let (first_sequence, first) = stream.snapshot();
    assert_eq!(first["id"], id);
    assert_eq!(first["state"], "running");
    assert_eq!(first["sequence"], first_sequence);
    assert!(
        TcpStream::connect_timeout(&child, Duration::from_secs(1)).is_ok(),
        "child known live while subscription exists"
    );
    let start = Instant::now();
    assert_eq!(f.request("DELETE", &format!("/v1/runs/{id}"), "").0, 200);
    let (last_sequence, last) = stream.snapshot();
    assert_eq!(last["id"], id);
    assert_eq!(last["state"], "cancelled");
    assert!(
        last_sequence > first_sequence,
        "terminal event must advance the existing stream sequence"
    );
    assert_eq!(last["sequence"], last_sequence);
    assert_eq!(
        stream.byte(),
        None,
        "terminal event closes this existing stream"
    );
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "terminal notification and closure must be bounded"
    );
    assert!(TcpStream::connect_timeout(&child, Duration::from_secs(1)).is_err());
}

#[test]
fn tst_neg_002_future_event_id_rejected() {
    let f = Fixture::controlled();
    let id = f.submit(config(3600));
    f.ready_worker();
    let mut stream = TcpStream::connect(f.address()).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    write!(stream,"GET /v1/runs/{id}/events HTTP/1.1\r\nHost: {}\r\nLast-Event-ID: 18446744073709551615\r\nConnection: close\r\n\r\n",f.address()).unwrap();
    let mut bytes = String::new();
    stream.read_to_string(&mut bytes).unwrap();
    assert_eq!(bytes.split_whitespace().nth(1), Some("400"));
    assert!(bytes.contains("invalid_event_id"));
    assert_eq!(f.request("DELETE", &format!("/v1/runs/{id}"), "").0, 200);
}

#[test]
fn tst_acc_001_real_numerical_artifacts() {
    let f = Fixture::new();
    let id = f.submit(config(1));
    let done = f.wait(&id);
    assert_eq!(done["state"], "completed");
    assert_eq!(done["samples"], 101);
    let (status, csv) = f.request("GET", &format!("/v1/runs/{id}/artifacts/samples.csv"), "");
    assert_eq!(status, 200);
    let csv = String::from_utf8(csv).unwrap();
    let rows: Vec<_> = csv.lines().collect();
    assert_eq!(rows.len(), 102);
    assert!(rows.iter().all(|r| r.split(',').count() == 35));
    let final_row: Vec<_> = rows[101].split(',').collect();
    assert_eq!(final_row[0], "100");
    assert!((final_row[2].parse::<f64>().unwrap() - 1.0).abs() < 1e-10);
    let (status, report) = f.request("GET", &format!("/v1/runs/{id}/artifacts/report.txt"), "");
    assert_eq!(status, 200);
    assert!(
        String::from_utf8(report)
            .unwrap()
            .contains("Coverage is descriptive")
    );
    let oracle = f.root.with_extension("oracle");
    fs::create_dir(&oracle).unwrap();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let worker = repo.join("target/debug").join(if cfg!(windows) {
        "fusion-lab.exe"
    } else {
        "fusion-lab"
    });
    let result = Command::new(worker)
        .args([
            "--seed",
            "42",
            "--duration",
            "1",
            "--scenario",
            "straight",
            "--position-sigma",
            "0",
            "--velocity-sigma",
            "0",
            "--acceleration-density",
            "0.1",
            "--no-outage",
            "--output",
        ])
        .arg(oracle.join("result.csv"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "independent CLI oracle must execute"
    );
    let oracle_bytes = fs::read(oracle.join("result.csv")).unwrap();
    fs::remove_dir_all(oracle).unwrap();
    assert_eq!(
        csv.as_bytes(),
        oracle_bytes,
        "HTTP artifact must match direct offline numerical CLI byte for byte"
    );
}
#[test]
fn tst_neg_001_invalid_configuration_has_no_run() {
    let f = Fixture::new();
    for body in [
        config(0),
        config(3601),
        {
            let mut c = config(1);
            c["unknown"] = json!(true);
            c
        },
        {
            let mut c = config(1);
            c["position_sigma"] = json!(-1);
            c
        },
        {
            let mut c = config(1);
            c["outage"] = json!({"start":0,"end":2});
            c
        },
        {
            let mut c = config(1);
            c["outage"] = json!({"start":0,"end":1,"unknown":0});
            c
        },
        {
            let mut c = config(1);
            c["scenario"] = json!("invalid");
            c
        },
        json!({"duration_seconds":1,"unknown":true}),
        json!({"duration_seconds":1,"position_sigma":-1}),
    ] {
        let (s, b) = f.request("POST", "/v1/runs", &body.to_string());
        assert_eq!(s, 422);
        assert_eq!(
            serde_json::from_slice::<Value>(&b).unwrap()["error"]["code"],
            "invalid_config"
        );
    }
    assert_eq!(
        serde_json::from_slice::<Value>(&f.request("GET", "/v1/runs", "").1).unwrap(),
        json!([])
    );
    assert_eq!(fs::read_dir(&f.root).unwrap().count(), 0);
}
#[test]
fn tst_bound_001_request_limit() {
    let f = Fixture::new();
    assert_eq!(f.request("POST", "/v1/runs", &"x".repeat(4097)).0, 413);
    assert_eq!(f.request("POST", "/v1/runs", "{").0, 400);
}
#[test]
fn tst_state_001_cancel_is_terminal() {
    let f = Fixture::controlled();
    let id = f.submit(config(3600));
    let child_address = f.ready_worker();
    let start = Instant::now();
    let (s, _) = f.request("DELETE", &format!("/v1/runs/{id}"), "");
    assert_eq!(s, 200);
    assert!(start.elapsed() < Duration::from_secs(2));
    let terminal = f.wait(&id);
    assert_eq!(terminal["state"], "cancelled");
    assert!(
        TcpStream::connect_timeout(&child_address, Duration::from_secs(1)).is_err(),
        "cancel response must follow child termination"
    );
    assert_eq!(f.request("DELETE", &format!("/v1/runs/{id}"), "").0, 200);
    assert_eq!(
        f.request("GET", &format!("/v1/runs/{id}/artifacts/samples.csv"), "")
            .0,
        409
    );
    let next = f.submit(config(1));
    assert_ne!(next, id);
    f.ready_worker();
    assert_eq!(f.request("DELETE", &format!("/v1/runs/{next}"), "").0, 200);
}
#[test]
fn tst_inv_001_single_active_admission() {
    let f = Fixture::controlled();
    let id = f.submit(config(3600));
    let child_address = f.ready_worker();
    let (s, b) = f.request("POST", "/v1/runs", &config(3600).to_string());
    assert_eq!(s, 409);
    assert!(
        TcpStream::connect_timeout(&child_address, Duration::from_secs(1)).is_ok(),
        "first child remains active at rejected admission"
    );
    let list: Value = serde_json::from_slice(&f.request("GET", "/v1/runs", "").1).unwrap();
    assert_eq!(
        list.as_array().unwrap().len(),
        1,
        "rejection must not allocate a queued run"
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&b).unwrap()["error"]["code"],
        "busy"
    );
    assert_eq!(f.request("DELETE", &format!("/v1/runs/{id}"), "").0, 200);
}
#[test]
fn tst_beh_001_retention_evicts_oldest() {
    let f = Fixture::with_retention(1);
    let first = f.submit(config(1));
    f.wait(&first);
    let second = f.submit(config(1));
    f.wait(&second);
    assert_eq!(f.request("GET", &format!("/v1/runs/{first}"), "").0, 404);
    assert_eq!(f.request("GET", &format!("/v1/runs/{second}"), "").0, 200);
    let list: Value = serde_json::from_slice(&f.request("GET", "/v1/runs", "").1).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(fs::read_dir(&f.root).unwrap().count(), 1);
}
#[test]
fn tst_contract_001_status_and_errors() {
    let f = Fixture::new();
    let (s, b) = f.request("GET", "/v1/health", "");
    assert_eq!(s, 200);
    let v: Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(v["status"], "ok");
    assert_eq!(v["capacity"]["active_limit"], 1);
    assert_eq!(v["capacity"]["job_memory_reservation_bytes"], 268435456u64);
    let (s, b) = f.request("GET", "/v1/runs/missing", "");
    assert_eq!(s, 404);
    assert_eq!(
        serde_json::from_slice::<Value>(&b).unwrap()["error"]["code"],
        "not_found"
    );
}
#[test]
fn tst_anti_001_path_traversal_denied() {
    let f = Fixture::new();
    for path in [
        "/v1/runs/../artifacts/samples.csv",
        "/v1/runs/%2e%2e/artifacts/report.txt",
        "/v1/runs/x/artifacts/../../Cargo.toml",
    ] {
        assert!([400, 404].contains(&f.request("GET", path, "").0));
    }
}
#[test]
fn tst_struct_001_non_loopback_rejected() {
    let root = std::env::temp_dir().join(format!(
        "fusion-rejected-bind-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = Service::start(ServiceOptions {
        bind: "0.0.0.0:0".parse().unwrap(),
        artifact_root: root.clone(),
        worker: PathBuf::from(env!("CARGO_BIN_EXE_controlled_worker")),
        retained_runs: 2,
        artifact_budget_bytes: 768 * 1024 * 1024,
    });
    if let Err(error) = &result {
        assert!(
            !error
                .to_string()
                .contains("EXPERIMENT_SERVICE_NOT_IMPLEMENTED"),
            "{error}"
        );
    }
    assert!(result.is_err(), "non-loopback listener must be rejected");
    assert!(
        !root.exists(),
        "bind validation precedes filesystem effects"
    );
}
#[test]
fn tst_prop_001_seeded_repeatability() {
    let f = Fixture::new();
    let mut c = config(2);
    c["position_sigma"] = json!(1);
    let a = f.submit(c.clone());
    f.wait(&a);
    let bytes = f
        .request("GET", &format!("/v1/runs/{a}/artifacts/samples.csv"), "")
        .1;
    let b = f.submit(c.clone());
    f.wait(&b);
    assert_eq!(
        bytes,
        f.request("GET", &format!("/v1/runs/{b}/artifacts/samples.csv"), "")
            .1
    );
    c["seed"] = json!(43);
    let changed = f.submit(c);
    f.wait(&changed);
    assert_ne!(
        bytes,
        f.request(
            "GET",
            &format!("/v1/runs/{changed}/artifacts/samples.csv"),
            ""
        )
        .1,
        "seed changes must reach the real numerical worker"
    );
}
#[test]
fn tst_perf_001_shutdown_bounded() {
    let mut f = Fixture::controlled();
    let _id = f.submit(config(3600));
    let child_address = f.ready_worker();
    let address = f.address();
    let start = Instant::now();
    f.service.take().unwrap().shutdown().unwrap();
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "shutdown must cancel and reap promptly"
    );
    assert!(TcpStream::connect_timeout(&address, Duration::from_secs(1)).is_err());
    assert!(
        TcpStream::connect_timeout(&child_address, Duration::from_secs(1)).is_err(),
        "shutdown terminates child before returning"
    );
}
#[test]
fn tst_beh_002_events_include_terminal_snapshot() {
    let f = Fixture::new();
    let id = f.submit(config(1));
    let done = f.wait(&id);
    let (s, bytes) = f.request("GET", &format!("/v1/runs/{id}/events"), "");
    assert_eq!(s, 200);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("event: snapshot"));
    let data = text.lines().find_map(|l| l.strip_prefix("data: ")).unwrap();
    let v: Value = serde_json::from_str(data).unwrap();
    assert_eq!(v["id"], id);
    assert_eq!(v["state"], "completed");
    assert_eq!(v["sequence"], done["sequence"]);
}
#[test]
fn tst_bound_002_insufficient_artifact_budget() {
    let f = Fixture::new();
    let address = f.address();
    assert!(address.ip().is_loopback());
    let root = std::env::temp_dir().join(format!("fusion-service-budget-{}", std::process::id()));
    let result = Service::start(ServiceOptions {
        bind: "127.0.0.1:0".parse().unwrap(),
        artifact_root: root.clone(),
        worker: std::env::current_exe().unwrap(),
        retained_runs: 1,
        artifact_budget_bytes: 1,
    });
    assert!(
        result.is_err(),
        "startup refuses less than one artifact reservation"
    );
    assert!(!root.exists(), "invalid startup has no filesystem effects");
}
#[test]
fn tst_state_002_worker_failure_releases_slot() {
    let local = Fixture::controlled();
    let mut bad = config(1);
    bad["seed"] = json!(999);
    let id = local.submit(bad.clone());
    let done = local.wait(&id);
    assert_eq!(done["state"], "failed");
    assert_eq!(done["error"]["code"], "worker_failed");
    assert_eq!(
        local
            .request("GET", &format!("/v1/runs/{id}/artifacts/samples.csv"), "")
            .0,
        409
    );
    let next = local.submit(bad);
    assert_ne!(next, id);
    assert_eq!(local.wait(&next)["state"], "failed");
}
#[test]
fn tst_bound_003_sse_subscribers_bounded() {
    let f = Fixture::controlled();
    let id = f.submit(config(3600));
    f.ready_worker();
    let mut clients = Vec::new();
    for _ in 0..32 {
        let mut stream = TcpStream::connect(f.address()).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        write!(
            stream,
            "GET /v1/runs/{id}/events HTTP/1.1\r\nHost: {}\r\n\r\n",
            f.address()
        )
        .unwrap();
        let mut first = [0u8; 1024];
        let n = stream.read(&mut first).unwrap();
        let text = String::from_utf8_lossy(&first[..n]);
        assert!(text.starts_with("HTTP/1.1 200"));
        clients.push(stream);
    }
    assert_eq!(
        f.request("GET", &format!("/v1/runs/{id}/events"), "").0,
        429
    );
    assert_eq!(
        f.request("GET", "/v1/health", "").0,
        200,
        "slow clients must not consume ordinary request admission slots"
    );
    assert_eq!(
        f.request("DELETE", &format!("/v1/runs/{id}"), "").0,
        200,
        "slow streams must not block cancellation"
    );
    drop(clients);
}
#[test]
fn tst_bound_004_artifact_bytes_force_eviction() {
    let f = Fixture::configured(32, 256 * 1024 * 1024, false);
    let first = f.submit(config(1));
    assert_eq!(f.wait(&first)["state"], "completed");
    let second = f.submit(config(1));
    assert_eq!(f.wait(&second)["state"], "completed");
    assert_eq!(
        f.request("GET", &format!("/v1/runs/{first}"), "").0,
        404,
        "active reservation must evict retained bytes even below count limit"
    );
    assert_eq!(fs::read_dir(&f.root).unwrap().count(), 1);
}
