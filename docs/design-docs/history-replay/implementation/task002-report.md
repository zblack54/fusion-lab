# TASK-002 core implementation evidence

State: awaiting main conformance review. TASK-003 has not started.

The implementation now replaces the diagnostic with synchronous bounded history replay. Only `src/history_replay.rs` changed among production sources relative to the reviewed TASK-001 inventory. Final source SHA-256: `553925e3f3ff7f2c9601237de157eaee889031ec9d8fafc81f1aef0a46e63f18`.

## Admission and authority

Source indices are sorted once by sample/sensor/sequence metadata. A bounded min-heap drains due arrivals before each tick's source admission. Immediate records bypass full waiting capacity, scripted losses use scheduled sequence numbers, and waiting records retain horizon decisions until actual arrival. The ledger stays in source order; the arrival list records delivered identities, including too-old ones. No history payload is read for future, loss, queue-full, horizon-only or too-old records. `Observation::value()` appears only in the numerical application path after admitted, arrived, eligible indices enter candidate history. Replay borrows validated observational inputs and introduces no truth, Config, RNG, I/O, wall clock, unsafe or dependency authority.

## Candidate transaction and checkpoint phase

Each tick first prunes candidate history to samples at least `tick-W`, then inserts eligible arrivals. The earliest positive-age arrival selects the correction branch before prediction or baseline integration. Otherwise ordinary progress starts at the current tick. Restore uses end(sample-1), or the original pre-zero prior for sample zero. Candidate checkpoints retain the needed earlier prefix and rebuild through the arrival tick; candidate measurements are sorted by sample/position-before-velocity/sequence and consumed once. Only successful complete batches swap the two buffers and emit a new snapshot; previously emitted snapshots are never rewritten. Local ledger edits cannot escape as a successful partial result on error.

Prediction and measurement calls increment counters immediately before the actual unchanged `KalmanFilter` call. Filter failures preserve core Error, actual arrival/replay ticks and optional identity. Baseline integration retains the accepted `position += last_velocity * 0.01` arithmetic and sample-time position resets. No baseline finiteness rule or numerical retry was added. The natural rewind fixture passes with arrival 300, replay 80, no observation identity; W299 remains stale-comparator equivalent. W0 signed-zero bit parity also passes.

## Bounds and actual storage accounting

The two checkpoint vectors each reserve `W+2` slots; the two history-index vectors each reserve 127 slots. Checkpoints retain the original prior while zero is eligible, then end(t-W-1) through end(t). Candidate observations are pruned before insertion. Unique validated position/20 and velocity/5 identities bound an inclusive 500-tick interval to 127 records. Explicit insertion checks prevent capacity-domain overflow; candidate numerical calls are checked against W+1 predictions and 127 measurements.

Old buffers stay fixed throughout a batch while candidates grow monotonically, so the sum of old and final candidate lengths is the actual peak logical checkpoint/history-record count. The reported reservation is the sum of both vectors' actual capacities, including empty scratch storage. Current filter/baseline working state and complete source/index/scheduler/result storage are separate from these logical history figures; this is no RSS or process-time claim. Each ordinary tick scans only bounded history/checkpoints, not the full trace; each source index is admitted once.

## Actual verification

`task002_verify.py` records source inventories, exact argv, exits, raw output and hashes in `task002-evidence/`. Focused final-source checks passed for the rational oracle, exact 501-prediction/127-update batch, natural rewind overflow and W0 signed-zero parity.

The trusted `verify_test_suite.py` ran once with `--mode implementation --baseline-digest 8c57cf49bd99b927012ac7215ea44688bc8cde928ea6b233a2f37abc41f13095 --offline`. It compiled and enumerated each declared configuration, passed all 38 selectors on each alias (76 actual outcomes), and passed CMD-001 through CMD-007. `RUSTDOCFLAGS=-D warnings` was set. Complete command/compiler/raw selector and root-gate evidence is in mutable `implementation/execution.json`. Both aliases resolve to the installed native Windows Rust 1.99.0; local evidence does not establish the final four-native matrix.

External-lock validation immediately before and after full verification returned 0 errors and 0 warnings. The initial TASK-002 validation was run after the first code edit; this report does not claim a pre-edit validator timestamp. TASK-001's final inventory supplies the independently reviewed pre-TASK-002 production baseline. Source hashes were identical throughout final verification; comparison to that baseline shows only history_replay.rs changed. Protected delivery/estimation/evaluation/simulation/sensors/main/lib and root manifest/lock hashes are preserved in the summary. Locked design/test/support/fixture/command/approved execution files were not edited.

This core stage is functionally green locally. Main inspection, publication/CI and final native/resource/example obligations remain. No measured resource budgets, final native conformance, product acceptance or merge is claimed.
