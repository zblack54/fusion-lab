# Local experiment service

From the repository root:

```text
cargo build --locked
cargo run --manifest-path crates/experiment-service/Cargo.toml --locked -- --bind 127.0.0.1:8080
```

The service invokes the existing numerical CLI without a shell. Configure a different executable or empty artifact directory with `--worker` and `--artifact-root`. Use `--help` for bounded retention and capacity options. Ctrl+C stops admission, cancels and reaps active work, closes clients and removes service-owned transient artifacts. The embedding API exposes explicit fallible `Service::shutdown` for the same operation.

Submit a complete configuration to `POST /v1/runs`:

```json
{"seed":42,"duration_seconds":30,"scenario":"maneuver","position_sigma":1.0,"velocity_sigma":0.15,"acceleration_density":0.1,"outage":{"start":10,"end":20}}
```

`GET /v1/runs/{id}` returns lifecycle status; `DELETE` cancels. Completed runs expose `/artifacts/samples.csv` and `/artifacts/report.txt`. `/events` is a bounded server-sent state stream with an immediate snapshot, monotonic sequence and terminal closure. It does not transport per-tick observations. `/v1/health` reports operating reservations.

Only one job is admitted; overlapping requests return 409. Each POST is a new run: retrying after a lost reply can create a second run, so inspect the list first. Completed history is bounded by count and byte reservations. HTTP streams do not retain complete numerical experiments in process memory. Memory reservations are conservative engineering allocations, not OS-enforced limits. No authentication or remote deployment is provided; binding is restricted to loopback and browser mutations require a matching Origin.

Artifacts are local and transient. Startup refuses a nonempty or linked root and never deletes preexisting content. An atomic sibling file named `.<root-name>.fusion-service.lock` claims the root exclusively for the instance, including while it is idle. Graceful shutdown releases it. An abrupt process crash can leave that claim: inspect the process and leftover files before deliberately removing a stale claim or supplying a new empty root. This version does not promise automatic crash recovery.
