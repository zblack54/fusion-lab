# Issue38: reserve time for retired artifact cleanup

Status: NATIVE_WINDOWS_REPAIR_VERIFIED; LINUX_RERUN_PENDING

## Preserved failure

Linux resource job121442559870 in workflow run40283756555 failed the existing strict end-to-end cleanup assertion at **5.001802873000003seconds**. The source was clean merge checkout `4db4ff08501eaf855323ea127bd90d6f709effc0`; this was a failed resource run, not a pass or an accepted tolerance.

The orchestrator preserved artifact12632644613 as `work/ci-evidence/service-linux-failed-issue38.zip` (SHA-256 `441a8ec9a7c3df6615d370db8bf56dc78fc0f54f387fa62e059fabce63e847c8`). Its raw JSON is copied unchanged to `implementation/linux-probe-issue38-failed.json` (SHA-256 `e3f16e281a5d541f19388566469214385a58a74ed87ce4e2410d0e054d1ec082`). The assertion remains `<5`; no test, locked contract, probe threshold or validator was loosened.

## Change

The socket write-idle timer previously consumed the entire five-second end-to-end budget before reader termination, file close and retired-directory removal could finish. `transport.rs` now uses one four-second `WRITE_IDLE_BUDGET` for initial timer and successful-write resets, reserving the remaining second for those cleanup steps. Header/body deadlines remain five seconds; the separate graceful connection drain remains four seconds. README documents this allocation. This is a tighter implementation within the approved five-second upper bound, not a new guarantee about arbitrary scheduler or filesystem stalls.

No other production behavior changed in this repair. The existing slow-reader resource demonstration remains the regression test: the client does not read or close after receiving headers; its run becomes404 immediately on eviction, the leased directory remains temporarily, and automatic cleanup must complete within the unchanged strict bound.

## Actual repaired native evidence

Command from repository root:

```text
python crates/experiment-service/scripts/service_resource_probe.py --build --output docs/design-docs/experiment-service/implementation/windows-probe-issue38.json
```

The probe rebuilt release executables and recorded actual source/manifests/locks/examples/probe SHA-256 and binary hashes. Local baseline HEAD is context only and `source_worktree_dirty=true`; source hashes bind the measured candidate. Repaired transport source SHA-256 is `9c337edd1b628e6f6c82427f823977b1c1cf4de89da64a0f4eb230b484ebc39c`.

| Measurement | Repaired Windows observation |
|---|---:|
| Strict deferred deletion after eviction |**3.9746330s**, immediate404, client never resumed reading or closed|
| Excluded maximum-duration warmup |1.4562572s|
| Three measured3600second runs |1.3311340 /1.3435287 /1.3671770s|
| Sampled combined resident peak |110047232bytes (~104.95MiB)|
| Sampled combined private commit peak |103411712bytes|
| Retained artifact disk peak |511873323bytes|
| Controlled kill/reap response |0.0013733s, child listener closed|
| Load interval |60.093seconds,396cycles|
| Excess ordinary request results |396responses503; maximum0.0257940s|
| Excess SSE subscriber result |429 while32existing clients remain open|
| Cleanup/Origin/pipe/overflow demos |Pass with the same documented semantics and limitations|

Reference host and collector limitations remain in the JSON and resource-verification.md. The root verification commands were finishing around the early release warmup; timings are end-to-end local observations with uncontrolled background work, not a comparative speedup/regression claim. Aggregate sampling is not a hard OS memory limit or proof that every transient was sampled. Resident measurements remain below the512MiB operating reservation.

## Verification

* Service formatting, all-target compiler check, Clippy with `-D warnings`, nine supplementary tests, doctest target and documentation passed with the production manifest.
* Trusted implementation runner executed all21 unchanged cataloged tests and supplemental Clippy successfully against external digest `3862f6d829e8a739580454f0edfbfb6aff696007286c51dbcfd61bd6f7529f1f`.
* All root AGENTS.md gates passed: format, all-target check/Clippy,15numerical/CLI/integration tests, one doctest, and documentation.
* Final required-lock validation remains zero errors/warnings.

The original Linux failure is preserved and remains unresolved until the orchestrator's current-source Linux resource rerun passes. A Windows pass does not satisfy that platform obligation. Final acceptance remains NOT_ACCEPTED and the external approved digest is unchanged.
