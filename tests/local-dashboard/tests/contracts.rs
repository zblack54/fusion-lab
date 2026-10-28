use fusion_experiment_service::{Service, ServiceOptions};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Host {
    service: Option<Service>,
    root: PathBuf,
}
impl Host {
    fn start() -> Self {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let build = Command::new("cargo")
            .args(["build", "--locked", "--bin", "fusion-lab"])
            .current_dir(&repo)
            .status()
            .unwrap();
        assert!(build.success(), "numerical CLI must build");
        let root = std::env::temp_dir().join(format!(
            "fusion-dashboard-contract-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let worker = repo.join("target/debug").join(if cfg!(windows) {
            "fusion-lab.exe"
        } else {
            "fusion-lab"
        });
        let service = Service::start(ServiceOptions {
            bind: "127.0.0.1:0".parse().unwrap(),
            artifact_root: root.clone(),
            worker,
            retained_runs: 2,
            artifact_budget_bytes: 768 * 1024 * 1024,
        })
        .unwrap();
        Self {
            service: Some(service),
            root,
        }
    }
    fn request(&self, method: &str, path: &str, body: &str) -> (u16, String, Vec<u8>) {
        let address = self.service.as_ref().unwrap().address();
        let mut socket = TcpStream::connect(address).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        write!(socket,"{method} {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        let mut bytes = Vec::new();
        socket.read_to_end(&mut bytes).unwrap();
        let split = bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let header = String::from_utf8(bytes[..split].to_vec()).unwrap();
        let status = header.split_whitespace().nth(1).unwrap().parse().unwrap();
        (status, header, bytes[split + 4..].to_vec())
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        if let Some(service) = self.service.take() {
            let _ = service.shutdown();
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn node_case(name: &str) {
    node_case_at(name, None);
}
fn node_case_at(name: &str, base: Option<String>) {
    let version = Command::new("node")
        .arg("--version")
        .output()
        .expect("Node24 is required");
    assert!(version.status.success());
    let major = std::str::from_utf8(&version.stdout)
        .unwrap()
        .trim()
        .trim_start_matches('v')
        .split('.')
        .next()
        .unwrap()
        .parse::<u32>()
        .unwrap();
    assert!(major >= 24, "Node24 or newer is required");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("node/data.test.mjs");
    let mut command = Command::new("node");
    command
        .args([
            "--test",
            "--test-reporter=tap",
            &format!("--test-name-pattern=^{name}$"),
        ])
        .arg(script);
    if let Some(base) = base {
        command.env("FUSION_DASHBOARD_BASE", base);
    }
    let output = command.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Node contract failed: {stdout}\n{stderr}"
    );
    assert!(
        stdout.lines().any(|line| line == "# pass 1")
            && stdout.lines().any(|line| line == "# fail 0"),
        "exactly one selected Node test must actually execute: {stdout}"
    );
}
macro_rules! node_contract {
    ($name:ident) => {
        #[test]
        fn $name() {
            node_case(stringify!($name));
        }
    };
}
node_contract!(tst_acc_001_stream_known_recording);
node_contract!(tst_beh_001_independent_errors_and_ellipse);
node_contract!(tst_state_001_replay_pause_seek_reset);
node_contract!(tst_contract_001_full_u64_wire_fidelity);
node_contract!(tst_neg_001_schema_and_numeric_rejections);
node_contract!(tst_bound_001_row_byte_and_line_caps);
node_contract!(tst_prop_001_chunk_partition_invariance);
node_contract!(tst_inv_001_measurement_absence_is_not_zero);
node_contract!(tst_struct_001_downsample_keeps_exact_cursor);
node_contract!(tst_perf_001_maximum_rows_have_bounded_storage);
node_contract!(tst_anti_001_row_snapshots_do_not_mutate_store);
node_contract!(tst_neg_002_invalid_utf8_and_partial_recording);
node_contract!(tst_bound_002_rounded_covariance_is_explicit);
node_contract!(tst_neg_003_unsigned_integer_tokens_are_strict);
#[test]
fn tst_contract_003_parser_consumes_real_http_artifact() {
    let host = Host::start();
    node_case_at(
        "tst_contract_003_parser_consumes_real_http_artifact",
        Some(format!(
            "http://{}",
            host.service.as_ref().unwrap().address()
        )),
    );
}

#[test]
fn tst_acc_002_real_http_dashboard_and_assets() {
    let host = Host::start();
    for (path, mime) in [
        ("/", "text/html"),
        ("/assets/dashboard.mjs", "javascript"),
        ("/assets/data.mjs", "javascript"),
        ("/assets/parser-worker.mjs", "javascript"),
        ("/assets/dashboard.css", "text/css"),
    ] {
        let (status, headers, body) = host.request("GET", path, "");
        assert_eq!(status, 200, "dashboard route unavailable: {path}");
        assert!(!body.is_empty());
        assert!(headers.to_ascii_lowercase().contains(mime));
    }
    let (_, headers, _) = host.request("GET", "/", "");
    assert!(
        headers
            .to_ascii_lowercase()
            .contains("content-security-policy:"),
        "dashboard CSP is required"
    );
}

#[test]
fn tst_contract_002_existing_service_artifact_schema() {
    let host = Host::start();
    let config = json!({"seed":18446744073709551615u64,"duration_seconds":1,"scenario":"straight","position_sigma":0,"velocity_sigma":0,"acceleration_density":0.1,"outage":null});
    let (status, _, body) = host.request("POST", "/v1/runs", &config.to_string());
    assert_eq!(status, 202);
    let submitted: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(submitted["config"]["seed"], json!(18446744073709551615u64));
    let id = submitted["id"].as_str().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let (status, _, body) = host.request("GET", &format!("/v1/runs/{id}"), "");
        assert_eq!(status, 200);
        let snapshot: Value = serde_json::from_slice(&body).unwrap();
        if snapshot["state"] == "completed" {
            assert_eq!(snapshot["samples"], 101);
            break;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    }
    let (status, _, body) =
        host.request("GET", &format!("/v1/runs/{id}/artifacts/samples.csv"), "");
    assert_eq!(status, 200);
    let csv = String::from_utf8(body).unwrap();
    let rows: Vec<_> = csv.lines().collect();
    assert_eq!(rows.len(), 102);
    assert_eq!(rows[0].split(',').count(), 35);
    let last: Vec<_> = rows[101].split(',').collect();
    assert_eq!(last[0], "100");
    assert!((last[2].parse::<f64>().unwrap() - 1.).abs() < 1e-12);
}
