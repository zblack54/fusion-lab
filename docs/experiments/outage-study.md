# Paired position-beacon outage study (issue #8)

Status: partial numerical baseline executed and independently reviewed.
Delivery delay/loss, late-observation policy, and asynchronous service evaluation
remain pending #6. This report does not complete issue #8.

Across the prespecified 50 held-out seeds in each trajectory family, a position
outage increased mean per-seed filter RMSE during [10,20) seconds by about
0.0244 m (straight) and 0.0312 m (maneuver). The filter had lower mean outage RMSE
than both existing baselines, but lost to velocity integration for one seed in
each family. These results describe this workload; they establish no general
superiority or uncertainty calibration.

## Protocol, provenance, and actual execution

The protocol in [outage-study-protocol.md](outage-study-protocol.md) was written,
reviewed, and accepted before comparisons. Its SHA-256 is
`80f1f191a53218dc15b2b7d10fc1600247178dc018845fc4e1dca8c71a6af2bb`.
No workload, metric, seed, threshold, or stopping-rule deviations occurred.
The driver received independent review before comparisons. Two pre-run review
fixes supplied per-command Git safe.directory and initialized failure context;
neither changed scientific methodology.

Executed from the repository with Python 3.12.14, Windows 10 build 19043,
Rust 1.99.0 / x86_64-pc-windows-msvc:

```text
C:/Users/Admin/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe scripts/outage_study.py --self-check
cargo build --release --locked --offline --config C:/Users/Admin/Documents/Codex/2026-10-20/if-x20/work/cargo-offline.toml
C:/Users/Admin/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe scripts/outage_study.py --output C:/Users/Admin/Documents/Codex/2026-10-20/if-x20/outputs/outage-study.json
```

All three commands returned zero. Analytical self-checks covered known Euclidean
errors (5, 2, 1 m), ellipse membership, immediate/censored recovery, a hand-derived
bootstrap/quantile oracle, and fixed seed/pair counts. The driver executed all
200 runs, validated schema, 3001 ticks per run, finite values, measurement/outage
schedules, positive-definite symmetric position covariance, and matching paired
truth/available observation streams. No seed failed or was omitted. The JSON
failure field is null. Temporary CSV/reports were removed; hashes and exact argv
are recorded per run. Rounded CSV is the numerical analysis input.

Recorded at 2026-10-25T20:34:16.194594+00:00; source revision
`b274c6f6deb3e86a9fdfbd31b8b92ec59396b5e8`. Source/lock files were unchanged;
untracked drafts for other work and this study were present. Full Git status
and individual src/*.rs SHA-256 values are in the external JSON.
Release executable SHA-256:
`acbc241a95ce30a076222c2c17d458036bedd170aadea91642725dc0940184f8`.
Cargo.lock SHA-256:
`1ff60b850d3f5928fc13476575700400765d31f7deb3268171f66176fb6cf6f3`.
Driver SHA-256:
`ffc827090c76d8d2becfe1b8736e1e2275469c6461aee06a1c4f8ec3baa84a38`.
External evidence: `C:/Users/Admin/Documents/Codex/2026-10-20/if-x20/outputs/outage-study.json`.
This machine-local evidence path is not a portable public artifact; the script,
protocol, seeds, hashes, and reported aggregates permit reproduction.

## Fixed workload and summaries

Straight uses seeds 10000..10049; maneuver uses 20000..20049. Every seed pairs
healthy sensors with position outage [10,20) s, using the same observation draws
where available. Duration is 30 s; position sigma 1.0 m, velocity sigma 0.15 m/s,
acceleration density 0.1 m^2/s^3. No parameter tuning occurred.

Overall includes ticks 0..3000 (3001 samples); outage-period includes 1000..1999
(1000) even in the healthy condition; post includes 2000..3000 (1001). Values
below are arithmetic means of 50 per-seed statistics, not pooled-tick RMSE.
Brackets give pointwise 95% percentile bootstrap intervals, 10000 resamples of
complete seed pairs with replacement, random seed 20261025. Ticks were not
resampled. Intervals are descriptive Monte Carlo uncertainty over these seed
families, without multiple-comparison adjustment or trajectory-general coverage.
All predefined windows are reported, regardless of direction of effect.

| Family | Condition | Window | Filter RMSE m [interval] | Hold RMSE m [interval] | Integration RMSE m [interval] |
|---|---|---|---:|---:|---:|
| straight | healthy | overall | 0.2791 [0.2615, 0.2972] | 1.4205 [1.4051, 1.4356] | 1.4156 [1.4004, 1.4306] |
| straight | healthy | outage | 0.2017 [0.1795, 0.2243] | 1.4140 [1.3860, 1.4409] | 1.4099 [1.3822, 1.4366] |
| straight | healthy | post | 0.1730 [0.1538, 0.1927] | 1.4267 [1.4016, 1.4511] | 1.4218 [1.3965, 1.4465] |
| straight | outage | overall | 0.2940 [0.2745, 0.3145] | 3.9755 [3.8497, 4.1051] | 1.3889 [1.3405, 1.4415] |
| straight | outage | outage | 0.2260 [0.1982, 0.2549] | 6.5804 [6.3515, 6.8178] | 1.2305 [1.0687, 1.3958] |
| straight | outage | post | 0.2103 [0.1905, 0.2305] | 1.4267 [1.4016, 1.4511] | 1.4218 [1.3965, 1.4465] |
| maneuver | healthy | overall | 0.2616 [0.2446, 0.2794] | 1.4199 [1.4026, 1.4378] | 1.4025 [1.3856, 1.4201] |
| maneuver | healthy | outage | 0.1710 [0.1549, 0.1881] | 1.4030 [1.3736, 1.4315] | 1.3836 [1.3553, 1.4117] |
| maneuver | healthy | post | 0.1527 [0.1389, 0.1666] | 1.4489 [1.4167, 1.4815] | 1.4263 [1.3942, 1.4585] |
| maneuver | outage | overall | 0.2783 [0.2611, 0.2971] | 7.4603 [7.2941, 7.6251] | 1.4393 [1.3654, 1.5183] |
| maneuver | outage | outage | 0.2022 [0.1791, 0.2254] | 12.7639 [12.4717, 13.0517] | 1.3709 [1.1678, 1.5854] |
| maneuver | outage | post | 0.1877 [0.1708, 0.2050] | 1.4489 [1.4167, 1.4815] | 1.4263 [1.3942, 1.4585] |

Paired differences are outage minus healthy; positive RMSE means greater error.
Coverage differences are percentage points. The primary endpoint is the
outage-period RMSE difference, not the overall or post window.

| Family | Window | Paired filter RMSE change m [interval] | Coverage change pp [interval] |
|---|---|---:|---:|
| straight | overall | 0.0148 [0.0074, 0.0224] | 2.2033 [-0.2346, 4.7358] |
| straight | outage | 0.0244 [0.0063, 0.0427] | 3.5240 [-1.4000, 8.4861] |
| straight | post | 0.0373 [0.0221, 0.0520] | 3.0849 [-0.6893, 7.6444] |
| maneuver | overall | 0.0168 [0.0094, 0.0247] | 0.1879 [-0.8551, 1.1690] |
| maneuver | outage | 0.0312 [0.0151, 0.0472] | 0.5140 [-1.2800, 2.3060] |
| maneuver | post | 0.0349 [0.0185, 0.0512] | 0.0500 [-2.3438, 2.0500] |

## Baselines and negative results

Differences below are filter minus baseline in the outage condition. Negative
values favor the filter. Wins/ties/losses count individual seed RMSE comparisons;
these are not tick-level counts or a universal guarantee.

| Family | Window | Filter minus hold m [interval] | W/T/L | Filter minus integration m [interval] | W/T/L |
|---|---|---:|---|---:|---|
| straight | overall | -3.6815 [-3.8139, -3.5556] | 50/0/0 | -1.0949 [-1.1507, -1.0424] | 50/0/0 |
| straight | outage | -6.3544 [-6.5951, -6.1226] | 50/0/0 | -1.0045 [-1.1722, -0.8397] | 49/0/1 |
| straight | post | -1.2164 [-1.2456, -1.1870] | 50/0/0 | -1.2115 [-1.2408, -1.1818] | 50/0/0 |
| maneuver | overall | -7.1819 [-7.3480, -7.0165] | 50/0/0 | -1.1610 [-1.2423, -1.0839] | 50/0/0 |
| maneuver | outage | -12.5617 [-12.8515, -12.2699] | 50/0/0 | -1.1687 [-1.3861, -0.9639] | 49/0/1 |
| maneuver | post | -1.2612 [-1.3016, -1.2205] | 50/0/0 | -1.2386 [-1.2781, -1.1987] | 50/0/0 |

In straight, 18/50 seeds had lower outage-period filter RMSE with the outage
than without it despite the positive average change. Integration outperformed
the filter during the outage for seed 10008 (filter 0.4313 m; integration 0.1635 m).

In maneuver, 16/50 seeds had lower outage-period filter RMSE with the outage
than without it despite the positive average change. Integration outperformed
the filter during the outage for seed 20033 (filter 0.3261 m; integration 0.3189 m).

## Reported uncertainty and recovery

Coverage is empirical nominal 95% joint position-ellipse inclusion, not proof of
calibration. The intervals include between-seed variation; the same scripted
dynamics, measurement model, q, and covariance recursion are reused throughout.

| Family | Condition | Window | Coverage % [interval] | Mean position trace m^2 [interval] |
|---|---|---|---:|---:|
| straight | healthy | overall | 92.3505 [88.8110, 95.4522] | 0.0837 [0.0837, 0.0837] |
| straight | healthy | outage | 90.7780 [84.4339, 96.1061] | 0.0381 [0.0381, 0.0381] |
| straight | healthy | post | 93.0290 [87.6642, 97.2988] | 0.0318 [0.0318, 0.0318] |
| straight | outage | overall | 94.5538 [91.3195, 97.2229] | 0.0956 [0.0956, 0.0956] |
| straight | outage | outage | 94.3020 [88.7699, 98.7000] | 0.0590 [0.0590, 0.0590] |
| straight | outage | post | 96.1139 [93.5405, 98.2738] | 0.0465 [0.0465, 0.0465] |
| maneuver | healthy | overall | 97.4442 [95.8487, 98.7071] | 0.0837 [0.0837, 0.0837] |
| maneuver | healthy | outage | 97.7500 [95.7039, 99.3641] | 0.0381 [0.0381, 0.0381] |
| maneuver | healthy | post | 98.5455 [97.1548, 99.5504] | 0.0318 [0.0318, 0.0318] |
| maneuver | outage | overall | 97.6321 [95.7133, 99.1316] | 0.0956 [0.0956, 0.0956] |
| maneuver | outage | outage | 98.2640 [96.0937, 99.9940] | 0.0590 [0.0590, 0.0590] |
| maneuver | outage | post | 98.5954 [96.3976, 99.9041] | 0.0465 [0.0465, 0.0465] |

Covariance traces and ratios were identical across seeds and both families
(the covariance recursion depends on the observation schedule and configured
noise, not observed values). No bootstrap uncertainty is attached to these
deterministic ratios.

| Tick (time s) | Healthy position trace m^2 | Outage position trace m^2 | Outage/healthy ratio |
|---|---:|---:|---:|
| 999 (9.99) | 0.047580 | 0.047580 | 1.000000 |
| 1999 (19.99) | 0.033600 | 0.070497 | 2.098143 |
| 2000 (20.00) | 0.032984 | 0.068038 | 2.062745 |
| 3000 (30.00) | 0.030635 | 0.035621 | 1.162758 |

Trace grew from the pre-outage tick to the final outage tick, then contracted
at the first resumed beacon (tick 2000), but remained above the healthy run even
at tick 3000. These scheduled observations support the covariance-growth
expectation; larger ellipses and higher coverage do not establish better
calibration. Coverage paired intervals include zero in all reported windows.

Operational recovery is the first start tick at/after t=20 s whose next 100
samples have filter RMSE within paired healthy RMSE +0.1 m. It is a forward
window with 0.99 s endpoint span, not permanent recovery. A zero latency means
the first window qualifies, not that every post-outage tick matches healthy.

| Family | Recovered / 50 | Not recovered by horizon | Median among recovered s | Range among recovered s |
|---|---:|---:|---:|---:|
| straight | 50 | 0 | 0.00 | 0.00–5.46 |
| maneuver | 50 | 0 | 0.00 | 0.00–8.11 |

The median is zero under this deliberately tolerant criterion even though mean
post-outage RMSE remains higher and covariance has not fully returned to healthy
values by 30 s. No censored seed was assigned a numeric latency.

## Boundaries and review

This is one platform/executable, two scripted trajectories, 50 seeds per family,
fixed q/sigma, one outage time/location/duration, correlated ticks, and
twelve-decimal CSV input. The filter's acceleration model is not identical to
scripted truth. Bootstrap intervals describe seed variation within these
conditions, not general real-world uncertainty. No pure-compute performance,
service concurrency/load, cancellation, late delivery, random loss, reordering,
or algorithm improvement was evaluated. No model or numerical source was changed.

Reproduce using the fixed protocol and driver; inspect per-seed JSON and
independently recompute the paired differences and bootstrap endpoints before
acceptance. Issue #8 remains open for its untested delivery/fault portions.

Main orchestrator independently recomputed the primary per-seed paired means, prespecified 10000-resample percentile endpoints, and outage baseline win/tie/loss counts from the external per-seed JSON. All matched within 1e-12. Evidence SHA-256: a5ee663f49e5cfeb28dfa77ec0df417031f438ae646cefbebf4c43d09d0df852. Command: bundled Python work/review_outage_results.py (outside candidate repository); exit0. This verifies the stated aggregates, not the unimplemented delay/loss scenarios.
