# ADR-001: isolate cancellable numerical work
Status: accepted

Context: the synchronous core returns a complete experiment and has no cooperative cancellation API. Maximum-duration measured child working set is 104128512 bytes, CSV 170623857 bytes (issue #10). Cancellation cannot rely on dropping a spawn_blocking join handle.

Options: (1) add cooperative cancellation throughout core, changing the offline contract; (2) run the existing CLI in an owned child, retain synchronous behavior and kill/reap on cancellation. Choose option 2 initially. Child paths and arguments are constructed, never shell commands. stdout and stderr are drained into bounded 64 KiB diagnostic tails; artifacts stream to private disk. Tokio supplies async process/network ownership; Axum supplies HTTP body limits and routing; serde supplies strict typed JSON. These dependencies have concrete adapter benefits, but their MSRV must be checked before adoption.

Positive: responsive OS cancellation, reusable CLI, no retained sample vector in service. Negative: process startup, CLI argument coupling, restart does not restore history; direct library transport can replace this under a later contract. Revisit when per-observation transport #6 requires a richer worker protocol, preserving this public API.

Affected: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012, REQ-013, REQ-014; CON-001, CON-002, CON-003, CON-004; MOD-001; TST-ACC-001, TST-NEG-001, TST-BOUND-001, TST-STATE-001, TST-INV-001, TST-BEH-001, TST-CONTRACT-001, TST-ANTI-001, TST-STRUCT-001, TST-PROP-001, TST-PERF-001, TST-BEH-002, TST-BOUND-002, TST-STATE-002.
