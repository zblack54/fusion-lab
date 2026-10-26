//! Test-owned process fixture: never substitutes for real numerical acceptance.
use std::{
    fs,
    net::TcpListener,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let value = |flag: &str| {
        args.windows(2)
            .find(|v| v[0] == flag)
            .map(|v| v[1].clone())
            .unwrap()
    };
    if value("--seed") == "999" {
        std::process::exit(7);
    }
    let output = PathBuf::from(value("--output"));
    let parent = output.parent().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    fs::write(
        parent.join("worker.ready"),
        listener.local_addr().unwrap().to_string(),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(40);
    while Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    drop(listener);
    std::process::exit(9);
}
