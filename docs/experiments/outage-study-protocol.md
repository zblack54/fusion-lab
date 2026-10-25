# Prespecified position-outage study (issue #8)

Status: proposed protocol; comparisons not run. This is a partial numerical
baseline for issue #8, not completion of delivery delay/loss or service research.
The protocol is frozen before examining these evaluation runs. Changes after
execution require a dated deviation record, not silent replacement.

## Question and hypotheses

How does a position-beacon outage change position error and reported uncertainty
relative to a paired healthy run, and how does the existing filter compare with
existing hold/integration baselines receiving the same available observations?
Primary hypothesis: mean per-seed filter RMSE during [10,20) seconds is higher
with an outage. Secondary expectations: position covariance grows during the
outage and contracts at the first resumed beacon; filter outage RMSE is lower
than each existing baseline on average. Negative or mixed outcomes are retained.
There is no tuning, algorithm selection, or implementation change in this study.

## Fixed workload and stopping rule

Use the existing release CLI, 30-second duration, two trajectory families:
straight seeds 10000..10049 inclusive; maneuver seeds 20000..20049 inclusive.
These 100 distinct evaluation seeds are held out from existing seeds 0..7 and
42/43; no tuning runs use them. Each seed has a healthy run (--no-outage) and an
outage run (--outage-start 10 --outage-end 20): exactly 200 child processes.
Explicitly set position sigma 1.0 m, velocity sigma 0.15 m/s, and acceleration
density 0.1 m^2/s^3. Position samples occur every 20 ticks, velocity every 5;
all post-update ticks 0..3000 are retained. Do not increase sample size or alter
thresholds after results. Abort on child failure, timeout (10 seconds per child),
nonfinite values, malformed CSV, invalid covariance, or mismatched paired truth /
available observation streams. No failed seed is dropped or substituted.

Execute sequentially, with temporary CSV/report artifacts removed after each
pair. Persist per-seed metrics, SHA-256 hashes, aggregate results, environment,
source revision and dirty status, binary/lock/protocol/driver hashes, exact argv,
and actual failures in external JSON. CSV files use twelve-decimal values, so
computed metrics have that rounding limitation. No benchmark timing claims.

## Metrics fixed before comparison

For each run and estimator (filter, last-position hold, velocity integration with
position resets), compute Euclidean position RMSE = sqrt(mean(dx^2+dy^2)).
Windows are overall ticks 0..3000 (3001), outage-period ticks 1000..1999 (1000)
in both conditions, and post-outage ticks 2000..3000 (1001). Baselines receive
exactly the observation information provided to the filter in that condition.

For the filter, compute descriptive nominal 95% position ellipse coverage in each
window: e^T P_position^-1 e <= 5.991464547107979, using a 2x2 Cholesky solve.
Validate positive definiteness and symmetry of that block. Also record mean
position covariance trace per window and trace at ticks 999, 1999, 2000, 3000;
record outage/healthy trace ratios at 1999, 2000, 3000. Trace is in m^2, coverage
is a fraction (report percentage points for differences). Neither establishes
calibration under these two scripted trajectories.

Operational error recovery: for each paired seed find the first start tick s in
2000..2901 inclusive for which the outage filter RMSE over ticks s..s+99 is at
most paired healthy filter RMSE over those same 100 ticks plus 0.1 m. Recovery
latency is (s-2000)/100 seconds. This is a forward 100-sample window (0.99-second
endpoint span at 100 Hz), not permanent recovery or a statistical stopping test.
If none qualifies, record null and a not-recovered-by-horizon flag. Report the
recovered count and median/range among recovered seeds without substituting a
latency for censored seeds or presenting it as an unconditional mean.

## Aggregation and uncertainty

Keep trajectory families separate. Report arithmetic means of per-seed metrics
(not pooled-tick RMSE), mean paired outage-minus-healthy differences for filter
RMSE and coverage in each window, and mean within-outage filter-minus-baseline
RMSE differences for each baseline/window. Primary endpoint is mean paired
outage-period filter RMSE difference. Report baseline win counts, ties included
separately; no universal superiority claim.

For each reported mean/difference, use a 10000-replicate percentile bootstrap of
50 complete seed records with replacement within its family. A seed, including
both paired conditions, is the resampling unit; never resample correlated ticks.
Use Python random.Random(20261025) reset for each statistic, sorted replicate
means, and linear interpolation at 2.5%/97.5% with index (n-1)*p. Intervals are
pointwise descriptive Monte Carlo uncertainty across the prespecified seed
population, not simultaneous/multiplicity-corrected confidence statements or
uncertainty over trajectory families. Recovery summaries and deterministic
covariance trace ratios receive no bootstrap interval. No significance tests.

## Verification and interpretation boundary

Before results, independently review this protocol and driver. Verify metric
arithmetic with analytical small inputs, check output counts/windows and matched
streams in every pair, and independently recompute primary aggregates from raw
per-seed evidence. Record actual commands and outcomes. Report all predefined
windows and both families even if primary hypothesis fails. Include failures
and deviations. No changing q/sigma, seed sets, recovery threshold, or estimator.

Limitations: one executable/platform; only two scripted trajectories, fixed noise
and process model, one outage duration/location, 50 seeds per family, rounded CSV,
correlated ticks, and a deliberately descriptive ellipse measure. No delivery
delay, random loss, reordering, late-observation policy, asynchronous service,
load, cancellation, calibration guarantee, or deployed-product result is tested.
Those portions of #8 depend on #6 and remain pending. Any proposed algorithm
change requires a separate feature issue and acceptance criteria.
