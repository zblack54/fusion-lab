# Roadmap and execution order

This is the repository's evolving release plan, tracked by
[roadmap issue #17](https://github.com/zblack54/fusion-lab/issues/17).
Version names below express scope and dependency order, not delivery dates or
promises that every feature currently exists. Issues and their milestone records
carry current work status; a roadmap entry is not implementation evidence.
The tracked milestone groups are listed in [GitHub milestones](https://github.com/zblack54/fusion-lab/milestones); the repository manifest is [.github/roadmap.json](https://github.com/zblack54/fusion-lab/blob/main/.github/roadmap.json). Neither a milestone name nor an open issue means the work has shipped.

The [learning portal #18](https://github.com/zblack54/fusion-lab/issues/18) explains
completed iterations so the project remains understandable as it grows.

## Release map

| Stage | Intended scope | Current evidence and boundary | Tracking |
|---|---|---|---|
| v0.1 numerical core | Seeded synchronous 2D experiment; linear Kalman filter; position/velocity observations; hold/integration baselines; outage; CSV/report | Existing committed CLI/core. Read the numerical contract and validation; no cross-platform bitwise guarantee. | [contract](design/v0.1.md), [validation](validation.md) |
| v0.1b Sensor realism and robustness | Separate true/assumed noise; constant velocity-sensor bias; repeated held-out evaluation | Noise mismatch is merged in PR62. Constant velocity bias is accepted in PR63. Deterministic linear drift is accepted in PR67 for CLI/library. The 576-run constant-bias study is accepted in PR65; it did not evaluate drift. Service JSON remains matched-noise and zero-bias. No tuning or bias state silently added. | [noise mismatch #19](https://github.com/zblack54/fusion-lab/issues/19), [velocity bias #20](https://github.com/zblack54/fusion-lab/issues/20), [robustness study #21](https://github.com/zblack54/fusion-lab/issues/21) |
| v0.2 local application | Local service with lifecycle/cancellation/artifacts; browser dashboard and replay | Local service verified in [PR #32](https://github.com/zblack54/fusion-lab/pull/32):21locked+14supplementary tests on four native variants, measured resource/load and ownership-failure evidence. Dashboard/replay is merged and accepted in [PR #44](https://github.com/zblack54/fusion-lab/pull/44), with measured resource/response and native UI evidence. | [service #5](https://github.com/zblack54/fusion-lab/issues/5), [dashboard #7](https://github.com/zblack54/fusion-lab/issues/7), [capacity #10](https://github.com/zblack54/fusion-lab/issues/10) |
| v0.3 delivery faults | Delay, loss, queues and explicit late-observation policy; evaluate effects | Planned transport adapter. Current position outage is numerical observation suppression, not an async delivery implementation. | [delivery #6](https://github.com/zblack54/fusion-lab/issues/6), [research #8](https://github.com/zblack54/fusion-lab/issues/8), [late-policy comparison #22](https://github.com/zblack54/fusion-lab/issues/22) |
| v0.4 richer estimation | Heading, wheel encoders, bias state, EKF | Proposed scope requiring its own state/observation model, design and independent tests. Current world-frame velocity observations are not wheel encoders. | [heading/wheels #23](https://github.com/zblack54/fusion-lab/issues/23), [EKF #24](https://github.com/zblack54/fusion-lab/issues/24), [bias state #25](https://github.com/zblack54/fusion-lab/issues/25) |
| Beyond initial scope (optional) | Learned correction; local inference serving | Optional research direction, not committed release scope. Requires a separately tracked question, baseline, resource/privacy assumptions, and acceptance criteria before work. | [optional local correction #26](https://github.com/zblack54/fusion-lab/issues/26) |

Milestone names: **v0.1 Numerical baseline**, **v0.1b Sensor realism and robustness**,
**v0.2 Interactive local demo**, **v0.3 Unreliable delivery and replay**,
**v0.4 Richer estimation**, **Engineering and knowledge transfer**, and
**Exploratory research candidates**. The last group is unscheduled and optional.

## Dependency order

1. Preserve the committed numerical baseline and its regression gates. Record
   research protocols before comparisons; use distinct tuning/evaluation seeds.
2. Implement separately tracked noise mismatch (#19) and constant velocity bias
   (#20), then run the prespecified robustness study (#21) depending on both.
   Investigate their effects as bounded research
   questions. Separate observations from model assumptions; do not claim the
   true-noise sigma alone provides mismatch without an assumed override. Any algorithm change needs
   its own feature design and tracked acceptance criteria.
3. Complete and verify the local service contract, including lifecycle,
   cancellation, resource ownership and native platform checks. Use measured
   capacity evidence before selecting concurrency. Keep numerical state separate
   from transport.
4. Build dashboard and replay against explicit service/artifact interfaces.
   Keep replay timing distinct from the numerical clock; expose truth for
   evaluation without making it available to the estimator.
5. Implement delivery faults with bounded queues and a declared late-observation
   policy, then extend repeated research to delay/loss. Preserve sensor-stream
   comparability and event-order evidence.
6. Derive and verify the richer v0.4 model before implementing it. Heading,
   wheel-encoder measurements, bias estimation and nonlinear linearization are
   explicit changes, not labels for the existing four-state filter. Heading/wheel
   modeling (#23) precedes EKF (#24); bias-state work (#25) depends on the bias
   experiment/configuration (#20). Late-policy comparison (#22) depends on
   delivery (#6). The optional correction candidate (#26) is outside this order.

The service and numerical robustness work are independent and may proceed in parallel;
the numbered list expresses a planning preference, while explicit issue dependencies
are the actual prerequisites. Independent documentation and research preparation may proceed while a formal
contract is under review. Dependencies determine the next ready implementation
slice; version labels alone do not justify bypassing a required contract or gate.
No step requires a paid remote service or deployment.

## Research evidence already available

The [outage study](experiments/outage-study.md) is a partial #8 baseline:
paired healthy/outage runs, two scripted trajectory families, held-out seeds,
RMSE/coverage/recovery and negative outcomes. It does not evaluate delay/loss,
noise mismatch or velocity bias. The [capacity study](experiments/capacity-baseline.md)
measures complete CLI cost and informs a proposed service budget; it does not
implement admission, backpressure, cleanup or hard memory enforcement.

## Supporting automation and knowledge track

[Engineering flow #2](https://github.com/zblack54/fusion-lab/issues/2),
[health review #3](https://github.com/zblack54/fusion-lab/issues/3), and
[supervisor #4](https://github.com/zblack54/fusion-lab/issues/4) support execution;
they are separate from the product's v0.1–v0.4 scope. A documented policy or
expected-red formal suite does not prove a running supervisor. Keep ownership,
quota recovery, evidence and user-control requirements explicit.

For every completed slice, provide a
[knowledge handoff](learn/iteration-handoff.md): what changed, why, math, code,
reproduction, evidence, limits and an exercise. Update this roadmap when the
actual state changes, link its issue/PR and milestone, and preserve remaining
gaps. The portal can be previewed locally with python scripts/docs.py serve.
Public publication is controlled by the owner through the opt-in Pages workflow.

## Current engineering checkpoint

The local service is merged and accepted in [PR #32](https://github.com/zblack54/fusion-lab/pull/32), including 21 immutable and 14 supplementary tests on four native OS/compiler variants, with measured resource/failure evidence.

[Supervisor PR #36](https://github.com/zblack54/fusion-lab/pull/36) has an approved revision-two scope gate: 56 tests per compiler pass, real private-control writes are denied, and process cleanup is observed. Actual worker Git commits are blocked by protected Git metadata. [Issue #56](https://github.com/zblack54/fusion-lab/issues/56) is preparing a controlled trusted-host commit design; no replacement scheduler is active and the supervisor is not accepted. Read the [supervisor lesson](learn/supervisor.md) for graph/loop ownership and the failure evidence.

[Dashboard PR #44](https://github.com/zblack54/fusion-lab/pull/44) is merged and accepted after independent review, four native OS/compiler variants and measured browser/resource evidence. Read the [dashboard lesson](learn/dashboard.md) for storage arithmetic, timing and covariance limits.

Noise mismatch is merged in [PR62](https://github.com/zblack54/fusion-lab/pull/62). The [constant velocity bias lesson](learn/velocity-bias.md) documents the accepted first slice of issue #20 in [PR63](https://github.com/zblack54/fusion-lab/pull/63). The [linear drift lesson](learn/velocity-drift.md) documents the accepted remaining slice in [PR67](https://github.com/zblack54/fusion-lab/pull/67). The [held-out robustness study](learn/robustness-study.md) and all 576 runs are accepted in [PR65](https://github.com/zblack54/fusion-lab/pull/65). Original Windows journal sharing failure and verified no-rerun reconciliation are tracked in #64.

The engineering health probe identified concurrent research-output ownership in [issue #68](https://github.com/zblack54/fusion-lab/issues/68). The research-only exclusive execution lease is merged in [PR #70](https://github.com/zblack54/fusion-lab/pull/70); the completed study used one owner and its historical evidence is unchanged.


[Observation delivery #69](https://github.com/zblack54/fusion-lab/issues/69) is being implemented in [draft PR #71](https://github.com/zblack54/fusion-lab/pull/71). Its synchronous replay design is approved and locked; the validated observation domain and source projection have independent stage verification. Replay and codecs remain incomplete. [Issue #72](https://github.com/zblack54/fusion-lab/issues/72) tracks an exact-rational floating assertion that conflicts with unchanged legacy arithmetic; implementation paused for independent confirmation and a controlled contract revision. No full feature, resource or actual async acceptance is claimed.
