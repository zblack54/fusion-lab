> Current decision: [accepted conformance](acceptance.md) for code79cb06a, including final native Linux/Windows proof. Earlier pending statements below are historical checkpoint records.

# Native implementation checkpoint

Status: IMPLEMENTED_UNVERIFIED

Externally approved revision1 digest: `3862f6d829e8a739580454f0edfbfb6aff696007286c51dbcfd61bd6f7529f1f`, read from host `approved-contracts.json`; source authority is the orchestrator's issue #5 approval. Trusted structural validation with `--require-lock --baseline-digest` passed with zero errors/warnings before implementation resumed.

The four slices are implemented within `crates/experiment-service/`; no locked catalog/test/configuration was changed by the implementation agent. The dependency-lock conflict was returned to the orchestrator for controlled revision. Exact behavior remains the approved HTTP protocol.

## Implemented boundary

* Strict typed configuration, complete mandatory fields, unknown-field rejection, domain/outage validation before admission or file creation, bounded body/read deadlines.
* Loopback-only service startup with private empty root validation and canonical executable/root paths. The standalone binary accepts bounded options and Ctrl+C shutdown; embedding retains the approved `ServiceOptions`/`Service` API.
* One serialized admission slot and no queued runs. Worker command arguments are constructed directly without a shell; existing synchronous numerical core/CLI are unchanged.
* Owned worker monitoring, successful-exit artifact validation, stable failed/cancelled terminal outcomes, kill then wait before successful cancellation, fallible shutdown and joined monitor/pipe threads.
* Each stdout/stderr reader retains at most65536bytes and drains continuously. Complete experiment samples never enter the service process; CSV validation uses bounded8192byte rows and artifact delivery streams from disk.
* Separate32 ordinary-request permits and32 SSE subscriber permits; sixteen-event channels; stale subscriber pruning; monotonic live notifications and terminal closure. Transport bounds96 accepted connections including header readers; five-second header/body/write-idle deadlines; four-second graceful connection drain.
* Count and byte-budget retention;256MiB active artifact reservation; completed-byte accounting; overflow monitoring and reap/fail; leased artifact streams keep retired disk bytes charged and close their file handle before deletion. Cleanup failures are bounded, visible and fail future admission closed.

## Actual evidence

`cargo test --manifest-path tests/experiment-service/Cargo.toml --locked --test http -- --test-threads=1` passed21/21 on native Windows in8.93seconds. Trusted `verify_test_suite.py ... --mode implementation --baseline-digest <external digest>` separately compiled/enumerated and executed all21 selectors and supplemental Clippy successfully; its authoritative record is `execution.json`.

The first post-restoration run accidentally reused a compiled diagnostic seam because source restoration preserved timestamps. Inspection confirmed real source was restored. A targeted `cargo clean --manifest-path tests/experiment-service/Cargo.toml -p fusion-experiment-service` forced a real rebuild; subsequent tests passed. Neither tests nor source behavior were weakened.

Production `cargo fmt -- --check`, `cargo check --locked --all-targets`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked --doc`, `cargo doc --locked --no-deps` passed using the production manifest. Four supplementary production ownership regression tests were subsequently added and pass; doctest targets still contain zero tests. Standalone `--help` executed successfully.

REQ-001..014 each have passing executable evidence through the unchanged catalog. This supports native behavioral implementation, not final product acceptance. CON-001..010 still require their reviewed non-test plans; no passing constants or narrow tests are presented as universal resource guarantees.

## Remaining acceptance work

* VER-001: independent exact-diff ownership/lifecycle, numerical separation, API/error and conformance inspection.
* VER-002: release maximum-duration memory/private-commit/disk/latency measurements and60second simultaneous slow-SSE/ordinary-request load;512MiB reservation is not an established measured aggregate bound yet.
* VER-003: native Windows/Linux with Rust1.99.0 and stable, all21 tests and required gates. Current evidence is native Windows stable1.99.0 only.
* VER-004: sentinel/symlink/reparse root, Origin, noisy pipes, artifact overflow, slow reader through eviction, permission/wait failures and task joins demonstrations. Implemented paths require real injected evidence and independent review.

The orchestrator coordinates these gates and publication. No commit, PR, final acceptance or Linux evidence is claimed by this checkpoint. `implementation/acceptance.md` remains NOT_ACCEPTED.

## Independent review repairs

The orchestrator found two concrete ownership defects in the initial implementation. Both were repaired without touching the approved21-test surface.

First, a lease count could outlive cancellation during async file-open or metadata operations. `ArtifactLease` now owns the count from the moment it is acquired under the registry lock. Open and metadata operations run as one blocking operation owning that guard and the ordinary-request permit; canceling the HTTP future cannot abandon an unowned count or prematurely free capacity for another pending operation. `PreparedArtifact` closes its synchronous file before releasing the guard on every error/drop path. The body is fed by a bounded one-chunk64KiB pipeline; its tracked reader owns the file/guard until client cancellation unblocks it. HTTP drain precedes joining those readers, preventing slow-reader shutdown deadlock.

Second, checking an empty artifact root did not exclusively claim it. Startup now atomically creates and holds a canonicalized sibling `.<root-name>.fusion-service.lock`. A second idle instance fails with AlreadyExists without writing into the first instance's root. Explicit shutdown releases the claim, permitting safe reuse. A crashed process can leave a stale claim; README documents deliberate inspection/recovery rather than automatic deletion. Root artifact-entry counts remain unchanged.

`cargo test --manifest-path crates/experiment-service/Cargo.toml --locked` executes four actual native Windows regressions: canceled queued open releases its guard; failed open and prepared-file destruction close/release correctly; dropped stream receiver unblocks and joins its reader before retired cleanup; competing idle startup is rejected and graceful release permits reuse. All four passed. The immutable21-test suite separately passed9.00seconds after these ownership changes; the final trusted implementation report is refreshed after request-permit lifetime hardening. Production fmt/check/Clippy/docs passed again.
