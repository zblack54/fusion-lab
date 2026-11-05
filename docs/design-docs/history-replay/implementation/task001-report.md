# TASK-001 implementation evidence

State: awaiting main review. Local implementation is complete for this slice only.

`HistoryPlan::new` accepts 0 through 500 ticks and rejects larger values with `HistoryError::InvalidWindow` preserving the request. Construction allocates nothing and requires no observation, truth, Config or I/O authority. Existing private fields, read-only accessors, borrowed replay signature, and public exports remain unchanged. `replay_history` deliberately retains the named diagnostic until TASK-002.

Only `src/history_replay.rs` changed among the recorded production sources and root manifests. Its final SHA-256 is `a12ef02b15b480eea8d5a1bc8ee78e3a3d8807ce4485af26ab3811480000d893`. Before/after source inventories record unchanged delivery, estimator, evaluation, simulation, sensors, main, lib, Cargo.toml and Cargo.lock hashes. Unrelated existing WIP was preserved.

The five approved TASK-001 selectors each passed on `+1.99.0` and `+stable`. Both aliases actually resolve to rustc 1.99.0, commit b940084d7eb6a299eb4bfeb8e34901bc051e7ac4, native x86_64-pc-windows-msvc. This is local Windows stage evidence, not final native qualification. All six root gates plus the independent harness Clippy gate passed with `RUSTDOCFLAGS=-D warnings`. Exact argv, exits, raw output and output hashes are in `task001-final-evidence/`; compiler records and source inventories accompany them.

The trusted validator passed before and after against the externally supplied approved digest `8c57cf49bd99b927012ac7215ea44688bc8cde928ea6b233a2f37abc41f13095`, with 0 errors and 0 warnings. No locked tests, support, contracts, dependency closure or approved execution evidence changed.

The first formatting gate failed because the new error literal needed standard multiline formatting. That original failure and initial successful checks remain in `task001-evidence/`; the formatting correction was followed by a complete final rerun with all exits zero.

Full 38-selector runtime acceptance is not green or claimed: TASK-002 remains unstarted. Native resource proof, example, complete conformance and main acceptance remain outstanding under TASK-003. No core acceptance or merge is implied.
