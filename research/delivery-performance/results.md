# Accepted-update companion measurement

Measured research candidate; publication review and exact-head CI remain required. Issues [#75](https://github.com/zblack54/fusion-lab/issues/75) and [#77](https://github.com/zblack54/fusion-lab/issues/77). The frozen [protocol](protocol.md) records the pre-run checkpoint; its historical no-execution statements describe that checkpoint. Main authorized the sealed series in [comment6567116231](https://github.com/zblack54/fusion-lab/issues/75#issuecomment-6567116231).

One Windows x86_64-pc-windows-msvc Rust1.99.0 release binary on AMD64 Family25 Model33 Stepping0, 16 logical CPUs, 34,266,181,632 bytes physical RAM. Binary SHA256 `85e334ee81b9f2d0cfd227c61378c931d52cc87f87d43dd6a550b6eef2ea6aa9`. A delayed and accepted one-second warmup preceded exactly D/A, A/D, D/A. All eight processes exited0 and were reaped. No measurement retry occurred.

| Run | Profile | Spawn-to-complete seconds | Lifetime peak bytes | Accepted per replay |
|---|---|---:|---:|---:|
| 1 | delayed | 0.422 | 286461952 | 0 |
| 2 | accepted | 0.437 | 286244864 | 90002 |
| 3 | accepted | 0.453 | 286228480 | 90002 |
| 4 | delayed | 0.422 | 286441472 | 0 |
| 5 | delayed | 0.422 | 286523392 | 0 |
| 6 | accepted | 0.438 | 286298112 | 90002 |

Every maximum process offered90,002 observations and produced360,001 snapshots. Delayed replay had89,877 stale drops and125 undelivered at horizon; accepted replay had90,002 accepted updates and no stale/horizon drops. Both had zero queue-full/scripted loss. The common source simulation already performs90,002 updates; two replay passes add0 versus180,004 accepted updates. All observation/model/snapshot bits, ledger/arrival values and canonical trace/ledger buffers matched across regeneration. Required allocations remained live across the owner ACK.

Paired accepted-minus-delayed complete-work time was15ms,31ms and16ms. These three pairs are descriptive, without a statistical or isolated kernel-cost claim. Work includes simulation, queue policy, codecs, allocations and equality checks. Memory differences also reflect different encoded ledger lengths. Spawn-to-complete excludes the later held-memory read and teardown; each raw record retains total owner time separately.

The [raw receipt](evidence/windows-1.99.0-20261102/receipt.json), individual process/command records, exact generated input bytes, and failure receipts are retained losslessly. Source, research, generated inputs and binary fingerprints matched before/after. Windows lifetime PeakWorkingSetSize uses an owned handle, read while allocations are held and again after exit; it is not sampled RSS. Accepted helper SHA256 ab851ab remains unchanged.

Issue77 records a static pre-run build cleanup gap, repaired before any build or measurement. Real Python-child normal completion, injected kill failure and injected wait failure prove direct-child reap and original-error retention. Failed compiler commands retain the guard for manual descendant reconciliation; compiler process-tree containment is not claimed. A60second owned workload watchdog is an execution bound. This companion adds no30second/512MiB product criterion and does not replace issue69 evidence.

Reproduce in a fresh external output directory after inspecting protocol and code:

```text
python research/delivery-performance/paired_probe.py --repo <absolute-repository> --output <new-external-directory> --run-approved-series
```

A new run is a new disclosed series, never a replacement for these observations. Initial evidence covers one Windows machine and compiler only; no accuracy, calibration, universal performance, Linux/stable or asynchronous transport claim follows.

Exercise: explain why the delayed profile still performs source-estimator work and why the accepted-minus-delayed difference is not the cost of one Kalman update. Check the raw counts and divide by neither90,002 nor180,004 without accounting for all changed work.
