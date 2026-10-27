# Recorded dashboard: preserve evidence while changing the view

Status: implemented and under review in [PR #44](https://github.com/zblack54/fusion-lab/pull/44). Known-row browser screenshots and comparisons are available; total browser resource acceptance remains pending. The dashboard has not merged as a shipped feature.

## Why recorded replay is separate from live status

The Rust service runs the deterministic CLI and publishes lifecycle status. Its SSE stream reports queued/running/completed/cancelled states; it is not a stream of every numerical observation. The dashboard will load completed CSV artifacts, retain their recorded rows and move a playback cursor over them. Changing playback speed changes the view clock, never the numerical 10 ms clock or filter result. Pausing or resetting the view does not cancel an experiment.

## A complete row and a bounded plot

A recording has 35 columns: tick/time, truth, observations, filter estimate, hold/integration baselines, outage and all16covariance entries. Empty position or velocity pairs mean absent measurements; an actual [0, 0] pair remains a measurement. The parser rejects partial pairs, reordered headers, invalid UTF8, nonfinite required values and gaps in the tick sequence. The row inspector can select any recorded row, even one omitted from the plot.

Plots retain at most 4096 actual rows, including endpoints. They never synthesize sensor observations through interpolation. This display reduction can hide short features; point counts and that limitation must be visible. The strict JSON boundary preserves unsigned 64-bit seeds and sequence values as exact decimal tokens, avoiding JavaScript Number rounding above 2^53.

## Storage and ownership

At the 3600 second limit, 360001 rows times 35 columns times 8 bytes require 100800280 numeric bytes, about 96.13 MiB. A dedicated worker incrementally parses the input into typed chunks; the last chunk allocates only the remaining rows. Whole response text and full arrays of row objects would add avoidable copies. The byte budget does not include the browser runtime, canvases or transient network buffers, so it cannot establish total resident memory by itself.

One worker owns the recording. A run switch aborts the old reader and acknowledges storage reset before loading the latest pending selection. Generation identifiers prevent stale worker or SSE messages from replacing the current view. Browser measurements must still demonstrate the proposed 512 MiB reference budget, control responsiveness and repeated cancellation behavior. Node tests alone do not prove browser resource use.

## What the ellipse means

The position ellipse uses the symmetrized 2 by 2 position covariance block, its eigenvectors and chi-square factor 5.991464547107979 for two dimensions. For eigenvalues 9 and 1, the semiaxes are sqrt(5.991464547107979*9) and sqrt(5.991464547107979), not the eigenvalues themselves. CSV rounding can yield tiny negative eigenvalues; the contract allows only a scale-relative tiny clamp and omits invalid covariance. A 95 percent label describes the Gaussian model construction; it is not empirical calibration evidence.

## Review and reproduction

The approved contract is under docs/design-docs/local-dashboard; the separate Rust/Node harness is tests/local-dashboard. Its 17 selectors run against actual production exports and a real service artifact. The initial design checkpoint has 16 expected-red cases and one passing service characterization per compiler. Implementation must make all 17 pass without modifying the locked surface.

After implementation, the required command is:

~~~sh
cargo test --manifest-path tests/local-dashboard/Cargo.toml --locked -- --test-threads=1
~~~

Native Windows/Linux and Rust 1.99/stable, unchanged service tests, actual browser interaction, known-row screenshots, a maximum real artifact and independent source review remain acceptance obligations. Follow [issue #7](https://github.com/zblack54/fusion-lab/issues/7) for actual current evidence.

## Current observed evidence

The current implementation passed all 17 locked tests on each local compiler, six supplementary regressions and the unchanged service tests. Published head `7fa29649` also passed native Windows/Linux with Rust 1.99/stable, plus service/root/docs checks. These checks apply to that exact head; later diagnostic changes need fresh review.

A real 170624208 byte maximum artifact loaded 360001 rows, with 4096 plotted samples and exact cursor row 7777. Browser DOM values at rows 0, 1500, 2100 and 7777 were compared across all 35 fields with recorded CSV. [Actual screenshots and comparisons](https://github.com/zblack54/fusion-lab/tree/7fa29649b15a06e73b037eac9cd1bdc892733493/docs/design-docs/local-dashboard/implementation) are immutable review evidence. They establish inspected values and interactions, not total memory or broad calibration.

[Issue #57](https://github.com/zblack54/fusion-lab/issues/57) tracks bounded diagnostics and actual owned-renderer measurements for maximum imports, repeated aborts and control responsiveness. Warmup plus three runs, 50 selection/abort cycles, p95 control response and long-task evidence remain acceptance work. Unsupported measurements must be reported as unavailable; zero is a measured result only when observation actually worked.

## Exercise

Generate a healthy run and an outage run with the same seed. Before viewing either, identify which CSV columns can be blank and predict what should happen to the position ellipse during the outage. Inspect a row that is absent from the 4096 point plot selection. Explain why a larger ellipse on one noisy trajectory does not establish 95 percent coverage across experiments.
