# Reading a paired sensor-robustness study

Status: all576prespecified runs completed and independently audited; PR65 product review and exact-head CI remain required. The frozen protocol remains authoritative; its original DRAFT header is preserved because main approval is recorded externally in [issue21comment6479640606](https://github.com/zblack54/fusion-lab/issues/21#issuecomment-6479640606). Main released reviewed execution in [issue21comment6479893719](https://github.com/zblack54/fusion-lab/issues/21#issuecomment-6479893719).

## What this asks

How do a fixed velocity-sensor offset and an incorrect assumed measurement covariance affect position error and reported uncertainty during a beacon outage? This evaluates a particular model, not an estimator ranking or recommendation to tune R. Noise mismatch and constant bias are accepted prerequisites in PR62/63. Drift and bias estimation are outside this study.

There are32held-out seeds1000–1031. Each seed runs two trajectories, three common assumed-noise scales and three fixed velocity offsets:32×2×3×3=576runs. Every30second run has3001snapshots and1000outage ticks during[10,20)s. True sigmas remain2m/.3m/s, q=.1m²/s³. The common assumed scale changes both channels together; it cannot isolate position-noise and velocity-noise effects independently.

A seed is the comparison unit. Adjacent100Hz samples are correlated through state prediction and updates; treating3001rows as independent trials would manufacture precision. Compare each scale with scale1 at the same seed, trajectory and bias. Across32paired differences, sample variance divides by31, and Monte Carlo standard error is sample_sd/sqrt(32). This describes variation from this simulation's random seeds, not an unqualified real-world population confidence interval.

## Metrics and mathematical checks

Position RMSE is sqrt(mean(dx²+dy²)). Nominal95% joint position coverage uses the full2×2position covariance, including cross-covariance. For P=[[a,b],[c,d]], the independent verifier evaluates (d dx²−(b+c)dxdy+a dy²)/(ad−bc), after checking symmetry and positive diagonals/determinant, against5.991464547107979. This is independent of the production Cholesky solve. A covariance ellipse is a model statement; incorrect assumptions can produce confident error.

Recovery has a separate operational definition: after20s, position error must stay≤1m for101consecutive snapshots spanning exactly1s. A100snapshot streak is insufficient. The latest candidate start is29s, ending at30s. Always report recovered count and unrecovered/censored count out of32; the average among recovered runs alone can conceal the harder failures. The1m threshold is a study convention, not a physical requirement.

CSV contains twelve-decimal values. It is not unrounded filter state. The Rust research runner also emits round-trip public Sample values and the unrounded Experiment.summary. The Python verifier recomputes metrics independently, checks fixed tolerances and explicitly records rows near coverage/recovery rounding boundaries. Exact source classifications resolve flagged differences; original evidence is retained rather than silently treating rounded disagreements as success.

## Ownership, fairness and reproduction

The research-only example uses the public Config/run API; it adds no production API or dependencies. Python owns one serial child at a time, bounded timeout/artifacts, fixed inventory, atomic checkpoint, independent verification and aggregation. The numerical loop remains synchronous. Neither transport nor replay influences numerical time.

A single3001row composite reference belongs to the current seed/trajectory. Truth, position observations, hold baseline, schedules and outage flags must be identical across all nine conditions. Changing only assumed noise must preserve velocity observations and integration. Changing bias must shift velocity observations by that fixed offset while preserving covariance at the same assumed scale. Integration intentionally changes because it consumes the same biased measurement as the filter.

Review [the frozen protocol](../design/robustness-protocol.md) and the research runner README before reproducing. Use the exact reviewed source, lockfile and compiler/target. The executor freezes source/protocol/binary hashes, toolchain, environment and all expected keys; a changed fingerprint prevents resume. Start in a fresh dedicated output directory. Completed keys are skipped; failed keys remain visible. Numerical/invariant failures are not retried. A single explicitly named retry is permitted only for execution-stage I/O failure or timeout, with the original failure retained.

~~~sh
cargo build --locked --release --example robustness-run
python research/robustness/study.py --binary target/release/examples/robustness-run.exe --output ../robustness-study --freeze
python research/robustness/study.py --binary target/release/examples/robustness-run.exe --output ../robustness-study --execute
~~~

On Linux omit.exe. These commands regenerate the entire fixed workload, not a selected favorable subset. Original raw artifacts may remain local because of volume; publication must state what is retained, what was discarded after verification, and the exact deterministic commands and hashes needed to regenerate it.

## Complete fixed-workload results

All576keys passed, totaling1,728,576snapshots. The following table reports all18condition groups, each with32seeds. RMSE is outage filter position error; SD is the across-seed sample SD. Delta and MCSE compare the same trajectory/bias with matched scale1, in metres. Coverage is mean outage joint-ellipse coverage. Full mean/SD/min/max and secondary overall/baseline outcomes are in [aggregates.json](../experiments/robustness/aggregates.json).

|Trajectory|Assumed scale|Bias [m/s]|Outage RMSE [m]|SD [m]|Coverage [%]|Paired delta [m]|MCSE [m]|Recovered|
|---|---:|---|---:|---:|---:|---:|---:|---:|
|straight|0.5|[0, 0]|0.4136|0.2200|56.42|0.000440|0.000129|32/32|
|straight|0.5|[0.3, -0.15]|3.4214|0.2929|0.00|-0.004070|0.000107|0/32|
|straight|0.5|[1, -0.5]|11.4579|0.2915|0.00|-0.013909|0.000105|0/32|
|straight|1|[0, 0]|0.4132|0.2195|95.13|0.000000|0.000000|32/32|
|straight|1|[0.3, -0.15]|3.4255|0.2925|0.00|0.000000|0.000000|0/32|
|straight|1|[1, -0.5]|11.4719|0.2912|0.00|0.000000|0.000000|0/32|
|straight|2|[0, 0]|0.4112|0.2177|100.00|-0.001968|0.000507|32/32|
|straight|2|[0.3, -0.15]|3.4383|0.2910|1.47|0.012797|0.000424|0/32|
|straight|2|[1, -0.5]|11.5152|0.2897|0.00|0.043390|0.000421|0/32|
|maneuver|0.5|[0, 0]|0.4139|0.2199|56.42|-0.000375|0.000509|32/32|
|maneuver|0.5|[0.3, -0.15]|3.4202|0.2929|0.00|-0.000476|0.000104|0/32|
|maneuver|0.5|[1, -0.5]|11.4567|0.2915|0.00|-0.010275|0.000104|0/32|
|maneuver|1|[0, 0]|0.4143|0.2192|95.13|0.000000|0.000000|32/32|
|maneuver|1|[0.3, -0.15]|3.4207|0.2925|0.00|0.000000|0.000000|0/32|
|maneuver|1|[1, -0.5]|11.4670|0.2912|0.00|0.000000|0.000000|0/32|
|maneuver|2|[0, 0]|0.4158|0.2165|100.00|0.001578|0.001996|32/32|
|maneuver|2|[0.3, -0.15]|3.4194|0.2910|1.68|-0.001279|0.000452|0/32|
|maneuver|2|[1, -0.5]|11.4962|0.2897|0.00|0.029150|0.000427|0/32|

Matched scale1 with zero bias gives mean outage RMSE0.4132m(straight)/0.4143m(maneuver), with95.13%coverage. At matched scale, offsets[.3,-.15] and[1,-.5] give about3.42m and11.47merror and zero outage coverage. At scale2 the smaller bias has1.47%/1.68%coverage, so the failure should not be summarized as zero coverage for every biased condition. Every zero-bias condition recovered32/32; every nonzero-bias condition recovered0/32under the stated operational criterion.

Larger assumed noise does not give a uniform accuracy improvement: scale2 slightly reduces zero-bias straight outage RMSE, while maneuver error slightly increases. Increasing reported uncertainty can improve coverage without correcting a systematic sensor offset. Do not choose a universal tuning from this table, treat descriptive MCSE as real-world uncertainty, or infer calibration merely from the matched case's near95%mean.

## Provenance, interruption and verification limits

Protocol SHA25605f4bb8553c7166a4707fa0f9e466f5857252ef5976b9a9c96d0a553c919ea2e was approved before data. Accepted production baseline was80c39e4f15b77d84b6d69c7e7f06ce05be49fed2. The exact executed research source is preserved in PR65 initial commit57a71f5a5e1abcd93bb5d9d7d326e88d9b3f5eb8; it was untracked locally during the run, with all13file hashes and binary/toolchain fingerprints independently checked in [frozen.json](../experiments/robustness/frozen.json). Later checkpoint hardening must not be represented as the executed version.

The first invocation stopped on Windows WinError5 while replacing checkpoint.tmp with checkpoint.json, coinciding with a host progress read. That is a likely sharing-handle cause, not a proven OS diagnosis. Durable journal141and pending142were preserved. Independent raw-artifact audit confirmed the142ndkey, all earlier records and the exact reference transition; main rechecked frozen fingerprints and promoted the verified pending journal. The resumed invocation skipped142completed keys. No numerical job was rerun, no seed was replaced, and no outcome was discarded. See [reconciliation audit](../experiments/robustness/checkpoint-reconciliation-review.json) and [issue64](https://github.com/zblack54/fusion-lab/issues/64).

The reviewed frozen verifier checked every run's CSV/roundtrip metrics, classification, schedule and fairness invariants before deleting ordinary raw files. It processed779,446,833CSV bytes serially. No numerical retry or rounding-boundary case occurred. Independent poststudy audit used retained per-run summaries and reproduced786aggregate comparisons; it did not re-read every deleted raw CSV. Only the142ndrun raw artifacts were separately re-read during journal recovery. [Independent audit evidence](../experiments/robustness/independent-review.json) makes this distinction explicit.

[records.json.gz](../experiments/robustness/records.json.gz) contains the exact complete records, not a selected subset; decompressed SHA256is111596fb8a24d2e7503d788207d079f4b95bcb0f15d6a0b8780c58e4822eb8e4. Raw CSVs are not published, and their hashes alone cannot re-establish their contents. Regenerate them with the fixed commands if independent row-level rechecking is required. On other compilers/targets, numerical roundoff and CSV hashes may differ; the protocol and meaningful numerical checks still apply. Run the portable independent audit against the published data and, optionally, the preserved executed source/binary to distinguish performed from skipped provenance checks. Main ran the portable auditor against the gzip records plus archived executed source/binary: both provenance checks passed; [portable audit evidence](../experiments/robustness/portable-audit.json) records the exact checks.

~~~sh
python research/robustness/audit_results.py --data-dir docs/experiments/robustness
~~~

This default command independently recomputes aggregates but reports source/binary checks as not requested. Use --executed-source-root and --executed-binary only for the original frozen artifacts, never the later hardened executor. The hardened checkpoint path adds bounded PermissionError-only retries with retained error evidence, preserves old/pending journals after exhaustion, rejects blind resume while a pending journal exists, and requires a fresh dedicated freeze directory. Fourteen synthetic checks include an actual Windows sharing-denial/release case. This hardening was verified after the study; the576runs used the preserved initial source.

These scripted trajectories, priors, true noise assumptions and three fixed offsets cannot establish general covariance calibration, real-world accuracy or best filter tuning. Drift remains untested. This study deliberately provides negative cases and a reproducible baseline for later work.

Exercise: derive the quadratic distance for error[2,1] and P=[[4,1],[1,2]]. Then explain why dropping covariance cross-terms changes the answer. For paired seed differences1through32, compute sample variance and Monte Carlo standard error; explain why repeating each seed's value3001times would not create additional independent evidence.
