# Ownership and boundaries

The source experiment/evaluator owns Config, truth, RNGs and prior source estimates. Projection borrows Sample rows only long enough to copy tick and present observations into an owned validated Trace. It ignores all other Sample fields, including position_outage. ReplayModel owns only requested assumed sigmas and acceleration density; it is not a Config alias.

One synchronous replay call owns a fresh filter, hold/integration baselines, bounded future waiting entries, output snapshots, source ledger and arrival order. No thread/runtime/channel/clock enters this owner. At each tick it predicts/integrates (except tick0), drains due arrivals before future admission, handles current source decisions and on-time bypass, then records. Loss precedes admission, preserve-old/drop-newest handles a full future buffer, and horizon finalizes remaining future observations without applying them.

Logical modules MOD001validation/projection, MOD002replay and MOD003codec may remain cohesive units inside src/delivery.rs; no crate split is required. MOD004is a later separate example. Existing estimation.rs keeps the same fixed filter model and failure commit semantics; its public exposure is not widened.

The pure library accepts borrowed values/bytes and returns complete owned values or structured errors. It performs no filesystem/network I/O. The separate example owns a fresh directory and complete temporary artifact writes. Multiple files are not promised one filesystem transaction; failures disclose earlier complete artifacts/temporaries and never report successful partial replay.

See api-and-format.md for exact public/error/byte contracts; verification/resource-workload.md for the complete release workload; implementation-brief.md for the reserved approval boundary.
