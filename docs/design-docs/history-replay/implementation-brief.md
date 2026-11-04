# History replay implementation handoff

Status: DRAFT; do not implement, self-approve or lock. Main scope/API selection issue22comment6636494428 follows repository delegated review authority; final independently reviewed contract and externally published digest are still required.

1. Make all required tests pass.
2. Do not modify any test-owned file, test catalog, fixture, snapshot, test dependency, test command, or test lock.
3. If a test conflicts with another test or an approved requirement, stop and report exact IDs and evidence instead of skipping, weakening, quarantining, or special-casing it.

Permitted implementation roots only src/history_replay.rs, the necessary lib export, examples/history_replay.rs and mutable progress/evidence plus nonproduction implementation/resource_probe.py, implementation/cleanup_probe.py and implementation/example_probe.py. These scripts may reuse the exact inventoried accepted owner helper unchanged; they may not edit or replace accepted helpers, test-owned adapters/fixtures, catalogs, gates or contracts. Other documentation changes remain main-owned, not blanket implementation permission. Preserve every other production file, root Cargo.toml/Cargo.lock and accepted contract/harness. Rootmanifest classifies the mutable package for Cargo metadata; that does not grant implementation permission to edit it. Existing root closure is the only allowed dependency closure. All test-owned scripts/build.rs/compiler fixture support are immutable after approval. The public accepted resource owner is inventoried externally; no private skill/checker source is copied into public tests. No machine-specific Cargo configuration.

Initial executable expectation:36named HISTORY_REPLAY_NOT_IMPLEMENTED runtime failures and2actual structural/privacy passes per declared configuration; execution evidence, not this description, establishes outcomes. Follow TASK-001..003 with externally sourced digest verification. Full38green/native matrix/non-test/resource/example acceptance remains mandatory. No new codec, real async, deployment or research-accuracy series.

Conflicts: report IDs, exact revision, input, expected/actual values, original rawfailure and required design interpretation. Preserve the old approval; main independently revises the contract before any changed behavior/test.
