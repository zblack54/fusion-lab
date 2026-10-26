# Local dashboard implementation checkpoint

Status: implemented-unverified. Approved digest d33ff5c8c1828d503d0cecfb144df72ab96621fa0f2e8dd8096c7f98ba0ddd95 remains unchanged. Native Windows proof below is separate from final browser acceptance and Linux CI.

The app adds explicit embedded static routes to the existing loopback Rust service. Pure `data.mjs` owns CSV chunks, strict wire validation, row snapshots, errors/ellipse and replay. One parsing worker owns the recording and replay; the main adapter owns one selected lifecycle EventSource and bounded plot data. No dependency or numerical-core changes.

Actual commands from repository root:

- Trusted dashboard `validate_design_package.py --require-lock --baseline-digest d33ff...`: zero errors/warnings before entry.
- `cargo test --manifest-path tests/local-dashboard/Cargo.toml --locked --test contracts`: all17passed, including real HTTP artifact fetch/parser and static routes.
- Trusted dashboard implementation runner: initial34executions passed across stable/1.99.0. Final runner refreshed after issue51; actual final-byte result is `execution.json`.
- `node --test crates/experiment-service/web/regressions.test.mjs`:4passed, no skips, covering paused ACK, forged numeric objects, fractional replay speed/pause and anisotropic/extreme/tiny covariance.
- Service production `cargo test --manifest-path crates/experiment-service/Cargo.toml --locked --all-targets`:14passed.
- Trusted existing-service implementation runner with approved3862... digest:21passed.
- Service Clippy all-targets warnings denied:passed. Initial fmt check caught one new tuple-formatting difference; `cargo fmt` applied it. Final style check must remain green.
- Root fmt/check/clippy/test/doc-tests/doc:passed. Root behavior unchanged.

The first direct Node-only run passed14pure cases and failed its integration case because it requires the real HTTP fixture supplied by the Rust wrapper. This is preserved as an entry-point limitation, not misreported as production failure or accepted red. The subsequent complete Rust fixture passed all17.

Initial native IAB browser launch on127.0.0.1:8087 created a real30second maneuver run with10–20second position outage, loaded3001rows and showed curves, uncertainty and actual sample0. It exposed issue47(stale running label) and48(paused seek lost behind pending frame ACK), tracked before narrow repair. Independent data review found49numeric-token impersonation,50fractional replay loss and51anisotropic minor-axis cancellation. These were tracked before fixes, repaired within approved contracts and covered by supplementary production regressions. The final reviewer independently cleared data e6ef2a8c..., worker fe3ab1eb..., dashboard5cfec0f1...; exact hashes and dirty-checkout provenance are in `source-checkpoint.json`.

Initial browser observations used a pre-repair embedded binary and are NOT postrepair acceptance. Rebuild/relaunch and actual screenshots, known-row fidelity, keyboard/resize/cancel/reconnect/import and maximum recording/resource demonstrations remain required. The Node max-row RSS contract passing does not establish renderer/worker RSS, main-thread long tasks or control latency. No native Linux/browser claims are made here.

Reproduction starts with `cargo build --locked` and `cargo build --manifest-path crates/experiment-service/Cargo.toml --locked`. Launch `crates/experiment-service/target/debug/fusion-experiment-service.exe --bind 127.0.0.1:8087 --artifact-root results/dashboard-review-20261026 --worker target/debug/fusion-lab.exe --retained-runs 8` using an empty private artifact root, then open that local URL. Ctrl+C performs service shutdown. An interrupted nonempty root requires inspection before reuse. A fresh private root is needed for a separate parallel inspection.

Checkable exercise after rebuilding: create the30second outage run, inspect ticks0/1500/2100, pause and seek exact nonplotted tick7777 on a longer recording, then compare the complete sample panel to the recorded CSV. Changing view/playback must not create or cancel a numerical run. This remains an acceptance exercise until observed and recorded.
