# True versus assumed measurement noise — issue #19

Status: approved for core/CLI-only implementation under main issue19comment6472048747. Approver: /root under owner delegation and AGENTS.md;2026-10-28. Main selected the two optional Config fields and one run/config path, explicitly accepting complete external struct-literal source impact in0.1. Service/dashboard JSON, locked contracts, dependencies, bias/EKF remain outside scope. This brief targets the existing numerical core and CLI after dashboard delivery; it does not expand into sensor bias, EKF or asynchronous delivery.

## Problem and bounded behavior

Today Config.position_sigma and velocity_sigma are used twice: Sensors scales actual Gaussian draws, then run passes the same sigma squared to the Kalman update. An operator cannot hold observations fixed while changing the filter's assumed measurement covariance.

Recommended initial scope: retain those existing fields/CLI flags as the true per-axis sensor standard deviations; add optional assumed_position_sigma and assumed_velocity_sigma overrides. Omitted overrides resolve independently to the corresponding true sigma. This preserves existing defaults, matching-noise behavior and existing CLI invocations. The new flags would be --assumed-position-sigma and --assumed-velocity-sigma. Position values are metres, velocity values metres/second; filter R uses their squared units. Each explicit override must be finite and within 0..=100, exactly like the existing sigmas. Zero remains valid and the estimator's existing variance floor 1e-12 is unchanged.

Example: --position-sigma 2 --assumed-position-sigma 1 creates 2 m actual position noise while the filter uses 1 m assumed sigma. Changing only an assumed override must leave every truth state, measurement, outage flag and baseline value unchanged for a fixed seed/true configuration. It changes only the estimator's state/covariance and resulting evaluation. Neither better nor worse RMSE on every seed is promised.

## Invariants and provenance

* Validate both optional values before sample allocation, sensor creation or CLI output-directory/file creation. Typed InvalidConfig identifies the invalid assumed field. Do not silently clamp, square before validation or replace invalid values with a fallback.
* Keep Sensors unchanged: two ChaCha8 streams, two StandardNormal draws per scheduled observation, including position draws suppressed during outages and draws multiplied by zero. Do not add draws, reseed, change stream numbers or sample ordering. Position update still precedes velocity at coincident ticks. Truth remains unavailable to the estimator.
* Resolve both assumptions once after validation, and use only those resolved variances in filter.update; only true sigmas enter Sensors.position/velocity. Baselines consume the same actual available observations.
* Preserve the 35-column CSV header, row ordering, tick count and 12-decimal data format. Default sample/CSV bytes must remain identical on the same locked toolchain/target. Companion report text may add an explicit noise-provenance line recording all four resolved sigmas, units and whether each assumption followed true noise or was overridden; unchanged report bytes are not promised. Experiment.config retains the requested optional inputs, so the report must show resolved values rather than only None.

## Compatibility decision for main

The smallest implementation adds two Option<f64> fields to Config with None defaults. This is a Rust source compatibility change for external complete struct literals, including src/main.rs's current literal; callers using ..Config::default() continue compiling. The repository is 0.1, but that is not permission to silently claim an additive field is universally source compatible. Main should explicitly accept/document this bounded API evolution before implementation.

Alternative: a separate run_with_assumptions/config wrapper API preserves Config's exact shape but creates parallel configuration/run paths and still needs a provenance ownership decision. Do not implement both interfaces. Recommended choice is the two optional fields with documented source impact, no new dependency, edition/MSRV/target change or estimator algorithm change.

The initial service wire stays unchanged. Its strict Configuration currently forwards the two true-noise flags to the numerical CLI; omitted new flags preserve its current matched-noise semantics. The dashboard still exposes every field in that existing service API and can import the unchanged CSV produced by a mismatch CLI run. Do not silently add JSON fields or dashboard controls under this numerical slice: exposing assumed-noise overrides through the service requires a separately reviewed service/dashboard contract revision, current strict payload/provenance rules and UI feedback. Document that the first delivery is library/CLI capable while service-created jobs remain matched-noise.

## Independent verification

Keep all current scalar-update, prediction, numerical-failure, CLI failure-before-output, seeded stream, outage comparability and baseline tests. Add only tests that distinguish true noise from estimator assumptions:

1. Default/explicit-follow regression: same locked seed/config with omitted assumptions and explicit assumptions equal to true sigmas produces identical samples/summary/CSV. For backward behavior, record a small pre-change CLI fixture/hash before implementation and compare its CSV after the change; a post-change self-comparison alone is insufficient.
2. Observation fairness: two nonzero-noise runs differing only in assumed sigmas have exactly equal truth, position/velocity measurement options, outage flags, held and integrated baselines at every tick. Also preserve healthy/outage stream comparability. Estimator covariance must differ at a known update; do not use a statistical RMSE ordering assertion.
3. Independent tick-zero oracle: straight motion, no outage, true sigmas 0, assumed position 3 and velocity 2, q 0. Prior diagonal is [25,25,4,4]; observation is position [0,0], velocity [1,0.5]. With R_position=9 and R_velocity=4, gain for velocity is 1/2, estimated velocity [0.5,0.25], position posterior variance 225/34 and velocity posterior variance 2; cross terms remain 0. Compare using justified floating tolerances. These rational values do not reuse the implementation's matrix calculation.
4. Validate each override independently for NaN, infinity, negative, >100 and inclusive0/100; verify an omitted other-channel assumption still resolves to that channel's true sigma. CLI invalid overrides must leave the output parent/files absent.
5. CLI/report regression: both override flags work together and separately; report records requested/resolved values and units; CSV schema remains 35 columns. Existing service contract and actual HTTP artifact characterization still pass with old JSON.

Required repository commands remain cargo fmt --all -- --check; cargo check --locked --all-targets; cargo clippy --locked --all-targets -- -D warnings; cargo test --locked; cargo test --locked --doc; cargo doc --locked --no-deps. Run the unchanged service/dashboard locked contracts after the core slice because they invoke the CLI. No commands above were run for this design-only preflight.

## Small ordered delivery

1. Before code: main selects the Config compatibility choice and records a small legacy CSV baseline; agree a short fixed-seed mismatch demonstration protocol.
2. Implement validated optional assumptions and exact analytic/fairness tests in the core, preserving RNG code and defaults.
3. Add CLI flags and explicit report provenance; meaningful CLI regressions and public API/help documentation.
4. Run all required checks and compatibility contracts; independently review the small numerical diff, publish one focused PR linked to #19, and merge only verified exact-head changes. Add a learning note explaining true noise versus assumed R.
5. Perform the issue #19 demonstration with predeclared fixed true noise and under/matched/over assumptions on paired seeds. Record negative outcomes and descriptive coverage, not calibration or general superiority. The larger held-out robustness matrix belongs to issue #21; no bias/EKF or tuning expansion.

Lightweight depth is appropriate for the recommended core/CLI-only slice: established synchronous ownership, two optional validated inputs and unchanged algorithm/CSV/service wire. Escalate the specific integration portion to formal revision if service JSON or dashboard contracts are changed. Main's scope/API choice and independent final review remain approval gates.

## Approved demonstration protocol

Before code, preserve legacy CSVs and exact source/lock/toolchain provenance outside the implementation candidate. Then use seeds7,42,314159,30seconds maneuver with outage10–20s, true position2m/velocity0.3m/s, q0.1 and assumed scales0.5/1/2. Do not tune or make a general ranking/calibration claim. Record all results, including counterexamples. For zero assumed sigma, report requested R=0 separately from unchanged effective variance floor1e-12.

## Actual local implementation evidence

The three pre-change CSV cases (default/nondefault/noiseless) remain byte-identical. All22root tests plus1doctest pass, including independent scalar/fairness/channel/invalid/CLI provenance cases. Required6root gates pass, unchanged dashboard34executions and service21executions pass against their external digests, and strict docs build passes. The9predeclared demonstration runs are recorded without tuning or general ranking claims. Independent review strengthened the every-tick fairness test with explicit equal/expected sample counts. Exact source/gate metadata is in ../experiments/noise-mismatch-verification.json; main final review and published-head CI remain acceptance gates.
