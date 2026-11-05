# Delivery fault study protocol (#8)

Frozen before any held-out evaluation.

Question: how do deterministic delay and scripted loss change position error, uncertainty coverage and recovery, and how does bounded history correction compare with stale-drop on the same delivered observations?

Use the accepted observation source and history replay from PRs #71 and #80. Generate each scenario once per seed. Construct healthy, delay-only, loss-only and delay-plus-loss inputs from the same source values. Each policy sees exactly the same offered observations, transport capacity and scripted drop configuration within each arm. Verify equality of transport dispositions across policies after normalizing only their numerical late-data decisions.

Compare W=0 and W=500. Use straight and maneuver families, 30-second runs, cap 128, explicit no outage, true and assumed sigmas 1 m and 0.15 m/s, acceleration density 0.1, zero bias/drift, and fixed one-based scheduled loss every fifth position and seventh velocity observation. In delayed arms, delay position by 120 ticks and velocity by 40 ticks during sample ticks 1000 through 1999. Stop delaying new samples at tick 2000. Record the last delayed arrival and any horizon drop. Confirm that the independent observation constructor accepts these schedules before freezing the protocol.

Reserve seeds 30000–30003 and 40000–40003 for fixture development. Held-out seeds are 31000–31049 for straight and 41000–41049 for maneuver. Freeze seed ranges, window, workloads and metrics before opening holdout results. Run every assigned pair once and preserve failures. Do not tune from the held-out set.

Primary measure: paired difference in position RMSE during the delivery disturbance. Secondary measures: post-disturbance RMSE, fraction of ticks inside the 95% joint position ellipse, and time after the last delayed arrival until position error stays within the prespecified healthy-relative tolerance for one second. Treat unrecovered runs as censored. Define exact metric formulas, tie behavior, and recovery tolerance in final protocol. Summarize uncertainty across seeds rather than treating correlated ticks as independent. Report workload, actual dispositions, compiler, platform, exact source and limitations.

Keep this first study deterministic. The worker from #82 owns asynchronous execution, but wall-clock scheduling is excluded from experimental treatments. Runtime overload is a separate experiment.

## Prespecified analysis

Use all 3,001 ticks per run. Primary endpoint is position RMSE on ticks 1000–2119 inclusive in the combined arm, W500 minus W0, paired by scenario and seed. Negative favors correction. Report each family separately and the equal-weight mean over the 100 independent seed pairs. Disturbance-end tick 2119 is a fixed design bound, even when a seed's last delayed arrival comes earlier. Report per-arm per-policy error summaries and fault-minus-healthy contrasts as secondary endpoints. Use ticks 2120–3000 inclusive for post-disturbance RMSE.

Position RMSE is sqrt(mean(dx²+dy²)). Compute joint ellipse inclusion with the position-position covariance block and chi-square(2) cutoff 5.991464547107979 inclusive. Use Cholesky solve and fail a run on a nonfinite error or a nonpositive-definite block. Summarize within-run coverage first, then aggregate across seeds. Do not treat ticks as independent trials.

Scripted loss runs throughout the trace, so recovery time is defined only in the delay-only arm. Starting at tick 2120, find the first start of 100 consecutive ticks where Euclidean error does not exceed the corresponding healthy run error plus 0.5 m. Report elapsed ticks from 2120 to the end of that first qualifying window. A run with no completed window is right-censored at tick 3000. Report censored counts and recovery fraction by the horizon. For combined and loss-only arms, report post-disturbance error and coverage instead of implying that loss stopped.

For uncertainty, report a prespecified paired seed bootstrap with 10,000 resamples, resampling 50 pairs within each scenario family with replacement. Use Python 3.12 random.Random(80122). For each replicate, draw 50 indices with randrange(50) for straight, then 50 for maneuver. Reuse the draw across all metrics and contrasts. Sort 10,000 replicate values and linearly interpolate the 0.025 and 0.975 quantiles at rank p × 9999. Intervals are descriptive for this workload and seed design. Preserve per-run records, exact program versions and failures. No data-dependent stopping or metric substitutions.

Preflight invariants for every no-outage seed: 752 offers. Healthy has 752 on-time observations. Delay has 502 on-time and 250 late. Loss has 637 on-time and 115 scripted losses. Combined has 426 on-time, 211 late and 115 scripted losses. No queue or horizon drops. Actual last late arrivals are 2100 in delay-only and 2080 in combined. Use 2119 as the conservative fixed analysis boundary.

Coverage is reported separately for ticks 1000–2119 and 2120–3000. Require finite symmetric positive-definite position covariance. Within each arm verify normalized ledger and arrival order across policies, and require full snapshot bit equality across policies in healthy and loss-only arms. Compare D−H, L−H and DL−H separately within each policy. Report pointwise descriptive intervals. For delay-only recovery compare paired event fractions and mean restricted confirmation time, with censored runs contributing 880 elapsed ticks and a separate event flag. The window is 100 reporting ticks, so earliest confirmation is at elapsed tick 99. A missing or invalid required run leaves qualification incomplete.
