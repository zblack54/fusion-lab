# Fixed held-out robustness executor (issue21)

Status: the original reviewed and frozen runner completed the fixed576-key workload after main release6479893719. Study interpretation/acceptance belongs to the independent audit and main review, not synthetic self-checks. Checkpoint hardening below is a later source revision under review; it has not rerun the study. Main approval6479640606 applies to the unchanged protocol SHA25605f4bb8553c7166a4707fa0f9e466f5857252ef5976b9a9c96d0a553c919ea2e. Constant-bias prerequisite is merged PR63.

## Files and ownership

- examples/robustness-run.rs: one key, publicConfig/run/Experiment.summary only; fixed576inventory inputs validated, no production API changes. Writes35-column12decimalCSV, round-trip per-row sidecar and unrounded public summary. Source classifications use Cholesky coverage and one consistent hypot recovery criterion.
- study.py: Python standardlibrary serial execution, inventory/source/protocol/toolchain/binary freeze, independent analytical2x2 quadratic form, CSV schema/order/config checks, round-trip invariant and rounding-boundary resolution,101snapshot recovery and seed-level statistics.
- test_study.py: independent golden values and synthetic data only. P[[4,1],[1,2]],error[2,1] gives quadratic8/7. Values1..32 give mean16.5, samplevariance88, MCSEsqrt(2.75). Recovery distinguishes100from101snapshots and2900from2901. No production study data used.

No dependencies, lockfile, service/dashboard contracts or production source changes. The reference is one3001row composite for the currentseed/trajectory: sharedtruth/position/hold; perbiasvelocity/integration; perscalecovariance. No priorseed/trajectory rowdata survives advancing. Serial currentCSV+roundtripCSV are checked and hashed before deletion. Compact perrun summaries persist. Original failure artifacts and any original rounded-boundary CSV/sidecar evidence are retained locally separately; public results must disclose regeneration/localrawdata limits.

## Review and execution commands

Before execution, independently review source and fixtures; main must verify frozen source hashes and accepted prerequisite revision. Do not execute these workload commands until released.

~~~sh
cargo test --locked --example robustness-run
python -m unittest discover -s research/robustness -p test_study.py -v
cargo build --locked --release --example robustness-run
python research/robustness/study.py --binary target/release/examples/robustness-run.exe --output ../robustness-study --freeze
python research/robustness/study.py --binary target/release/examples/robustness-run.exe --output ../robustness-study --execute
~~~

On Linux omit.exe. Use the samebinary/buildtarget/toolchain for freeze/resume. Thefreeze includes exactHEAD/tree, production/research/protocol/lockfileSHA256, rustc-Vv, Python/OS/machine provenance and complete expected run-keyinventory. A mismatch rejects before model work. Fingerprints must be reviewed; this script does not by itself certify a GitHub merge or independent approval. The full commands used for eachkey are stored in records. The runner positional args areSEED straight|maneuver SCALE_INDEX BIAS_INDEX OUTPUT_STEM; no silently defaulted study parameters.

Each child has atmost60seconds and32MiBcurrentartifact/log cap, polled every50ms; thePython owner kills and waits on failure/timeout. One overall4hour invocationbudget checkpoints without outcome-based selection. Resume skips completedkeys and requires an orderedprefix plus unchangedfingerprints. Atomic checkpoint.json contains compactrecords and the onecompositereference together, preventing split journal/reference acknowledgements. Never delete thecheckpoint to rerun failures silently.

The first failure stops interpretation and is retained with originalartifact hashes/logs. Only a positively identified execution-stage I/O failure or timeout permits one explicitly recordedretry using --retry-key KEY alongside --execute. Numerical, invariant, cap and malformeddata errors cannot use retry. All576verifiedkeys are required for records.json/aggregates.json. A partialcheckpoint is not a complete-studyclaim.

IndependentCSV coverage checks symmetry<=1e-10, positive diagonals/determinant, and fullcross-term. FixedRMSE tolerance1e-8absolute+1e-8relative. Boundaryrows within1e-8mofrecoverythreshold or1e-7ofcoveragequadratic threshold are explicitly listed with rounded/roundtrip/sourceclassifications, with originalartifactsretained. No aggregate mismatch is silently waived. Acrossseeds use samplesd denominator31 and pairedMCSEsd/sqrt32; always include recoverycensoringdenominator, no pooledwinner or tuningselection.

The protocol file retains its original DRAFT header because its approved bytes are frozen. Delegated main approval is recorded externally in issue21comment6479640606, not by editing the hashed protocol.

## Checkpoint sharing hardening after the executed study (issue64)

The completed576-key study used the original executor source preserved in PR65commit57a71f5 and the external executed-source archive. Its study.py SHA256 was4764f053ee17b060e0fba205fb7cc246713bc24aeb9f6ddc9e531b37e3171185. The executed freeze identifies the original source, binary and compiler. Do not claim the hardening below produced those observations, and do not edit an old freeze to accept changed source. Future regeneration uses a new dedicated output directory and new reviewed freeze.

Windows denied one atomic checkpoint replacement while a reader held the destination without delete sharing (WinError5). The old journal contained141verified keys; the complete pending journal and current artifacts described key142. Main and independent review checked the exact prefix, pending key, source/binary fingerprint and retained artifacts, promoted that exact pending journal, and resumed by skipping142acknowledged keys. This was journal acknowledgement recovery, not a numerical rerun, seed replacement or altered result. The original evidence is retained separately.

The hardened executor writes and fsyncs the pending journal once, then retries only PermissionError from atomic promotion at most5times with100ms between attempts (maximum4waits). Each original exception and pending-journal hash is appended and fsynced to checkpoint-replace-events.jsonl. A successful retry records the acknowledged journal hash. No model process or verifier is called again during these retries. Other I/O errors are not retried. On exhaustion both the old checkpoint and exact pending.tmp are preserved, with a typed error and the original exceptions. Resume refuses any pending journal until independent reconciliation; it cannot silently recompute already verified data. This does not claim crash-proof power-loss durability of the surrounding filesystem.

Freeze now rejects any nonempty output directory before fingerprint work or writes and creates frozen.json exclusively. This establishes a fresh dedicated staging directory before the executor owns current.* artifacts. It prevents accidentally adopting an existing directory containing unrelated current.* files.

Regression verification includes synthetic sharing-release and exhausted-sharing cases, non-PermissionError rejection, nonempty-directory preservation, and an actual Windows CreateFileW reader that omits delete sharing. The latter reproduced WinError5, released the owned handle, then observed successful promotion; on non-Windows it is an explicitly platform-specific fixture. These are checkpoint-only tests. The hardened executor was not used to rerun the completed held-out study.
