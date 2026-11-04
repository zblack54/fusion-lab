# Issue22 P1–P4/API review checkpoint

Status: DRAFT CONTRACT; approved-API diagnostic seam and independent harness now exist. No correction behavior, lock or final approval exists. Authority read: issue22comment6636278825. Baseline cargo check --locked --all-targets passed locally. Accepted69 source and public types inspected; root1.99/Edition2024/resolver3/nooptionalfeatures preserved. Parent6 async remains open.

## Boundary and proposed public interface

Main API review issue22comment6636494428 authorized the minimal named NOT_IMPLEMENTED seam in src/history_replay.rs plus root module/export additions. Existing delivery.rs, estimator equations, simulation/Config/RNG, codecs, oldtests and locks stay byte-identical. Reuse validated public Trace/Observation/ReplayModel getters. Use crate-private KalmanFilter without changing it: reconstruct checkpoint filter via new(q), then assign its crate-visible state/covariance. No Clone/unsafe/new dependency is needed.

Selected API names (all result fields private/read-only):

- HistoryPlan::new(window_ticks:u32)->Result<HistoryPlan,HistoryError>, window_ticks() getter, valid0..=500.
- replay_history(model:&ReplayModel,trace:&Trace,plan:&HistoryPlan)->Result<HistoryResult,HistoryError>.
- HistoryResult::snapshots()->&[HistorySnapshot], ledger()->&[HistoryLedgerEntry], arrival_order()->&[ObservationId], work()->&[HistoryWork].
- HistorySnapshot::tick(), estimate()->State, covariance()->[[f64;4];4], held_position()->[f64;2], integrated_position()->[f64;2]. Own new snapshot type because accepted ReplaySnapshot fields/constructor are private and must remain unchanged.
- HistoryLedgerEntry exposes id/sample_tick/planned_arrival_tick/decision_tick/queue_occupancy/outcome, matching source-order existing inventory. New HistoryOutcome variants: ScriptedLoss, QueueFullDrop, AcceptedOnTime, CorrectedLate, TooOldDrop, UndeliveredAtHorizon. Do not add variants to accepted DeliveryOutcome or call the old TSV OnTimeOnly codec for these results.
- HistoryWork per arrival tick: tick, restored_sample_tick(Option<u32>), prediction_calls, measurement_calls, committed_checkpoints, committed_observations, peak_live_checkpoints, peak_live_history_observations, reserved_checkpoint_slots, reserved_observation_slots. Counters describe actually performed work/storage, not formula-only promises; source/output/scheduler allocations reported separately by non-test resource proof.
- HistoryError::InvalidWindow{value}; NumericalFailure{arrival_tick,replay_tick,observation:Option<ObservationId>,source:Error}; InvariantFailure{tick,kind:HistoryInvariant}, where kinds are MissingAnchor/WorkBound/StorageBound; no caller budget or fault hook.

A pure function returns complete result or typed error, with no successful prefix and no externally mutable history object. All previously emitted snapshot values remain unchanged when current state is corrected. Truth and Config never cross into the numerical API.

## Transport and timing

Derive a private frozen admission schedule from Trace before numerical application using exactly accepted transport semantics: scripted loss first; drain all due waiting before source admission; immediate on-time bypasses waiting cap; source-order admission; only future waiters occupy capacity; horizon dispositions retain final tick. Sort arrived usable observations canonically by sample_tick, position before velocity, sequence. Do not obtain admissions by calling replay_delivery: a numerical failure there must not redefine transport. Tests compare complete admission/source identity/arrivals/queue occupancy against accepted successful comparator cases, including capacity boundary, losses and missing scheduled sequences.

At arrival tick t, admit the transport batch once and select correction-versus-normal branch BEFORE prediction, baseline integration or numerical updates. No ordinary work is performed then discarded before correction; the501/127 ceiling is the actual total numerical work of a correction tick. Partition arrivals: sample=t on-time; 0<t-s<=W eligible correction; age>W TooOldDrop. If any late eligible arrival exists, restore checkpoint immediately BEFORE earliest sample in batch; replay through t using only admitted/arrived information known at t. Include current-tick on-time records exactly once in this batch and canonical sample order. If no late eligible arrival, normal one-tick predict/integrate then current on-time updates. At tick0 no prediction; a distinct original prior anchor is the checkpoint before sample0. End-tick checkpoint t means all observations known at the current arrival tick have been canonically applied through sample tick t. Commit corrected current state/history and append online snapshot only after the whole batch succeeds.

Anchor for the retained window at t is end-tick(t-W-1), or original prior when that index is negative. Retain checkpoints from anchor through t, at mostW+2. Prune candidate observation scratch before inserting the newly arrived eligible batch; otherwise atomic duplication can transiently exceed the intended bound.

## Constructive work/storage obligations

Committed history: <=W+2 checkpoints and <=floor(W/20)+1+floor(W/5)+1<=127 observations (inclusive sample interval). Replay work per correction: <=W+1 predictions (<=501), <=127 measurement updates; initial-prior sample0 uses no prediction for tick0. Normal progress is separately counted (<=1 prediction and2updates). W0 must reproduce accepted OnTimeOnly state/covariance/baseline bits and dispositions mapped onto TooOldDrop/AcceptedOnTime.

Atomic rollback may keep original plus candidate history: physical logical slots count <=2*(W+2) checkpoint objects and <=254 retained observation slots after candidate pruning, plus constant current filter/baseline and bounded arrival-index scratch. These are not total RSS claims. Input Trace, frozen schedule, complete snapshots, ledger, work diagnostics, comparator output, equality buffers and temporary arrays must be inventoried in resource verification. Main/reviewer must confirm exact scratch construction and counters before locking; do not label committed logical caps as process peak.

## Independent oracles / acceptance sketches

1. Velocity sample0 [2,-2], sigma2, q0 arriving tick1, no other observations. Prior Pvv4/R4 -> gain1/2, velocity[1,-1], current position[.01,-.01], Ppp25.0002/Ppv.02/Pvv2. Stale-drop current statezero, Ppp25.0004/Ppv.04/Pvv4. Raw velocity baseline integrated position[.02,-.02]. Use explicit1e-12 component checks, not false rational==floating equality.
2. Same data on-time vs one-tick late: online prefix at tick0 differs and remains immutable; corrected tick1 equals a manually constructed sorted sample-time filter result within declared tolerance. No noncausal early update allowed.
3. Eligibility ageW inclusive, W+1 rejected, W0 bit-equivalent stale policy, missing sequence does not renumber loss/admission; window500 and initial anchor edge.
4. Late+on-time batch, reversed arrival insertion order, simultaneous sensors, repeated corrections to one historical sample range, boundary eviction and exact once application.
5. Causality: append only observations SAMPLED AFTER the online-prefix cutoff and require that prefix identical. Merely later planned arrival is insufficient: an earlier-sampled future waiter legitimately changes capacity/loss. Separately hold all metadata fixed, vary finite unarrived payloads and require earlier prefix unchanged. The scheduler may inspect metadata for source/admission, while the numerical branch cannot read payloads before actual arrival; inspect this separation and exercise it metamorphically.
6. Finite large values causing an actual replay overflow return typed error at current arrival/replay tick, preserve original numerical cause and expose no partial result. No artificial production fault hook. Exact reproducible fixture still to qualify against current unchanged filter.
7. Read-only diagnostics test actual per-tick work/committed/physical scratch caps. Canonical stale comparison shares transport outcomes; different numerical admission policy is explicit, not an estimator-only advantage.

## Main selections

Issue22comment6636494428 selects the separate API/no codec; typed MissingAnchor/WorkBound/StorageBound invariant errors without caller budgets/fault hooks; total actual counters; pure library+small deterministic example. Resource profiles and targets are specified in verification/resource-workload.md. This draft is not final contract approval.

## Raw baseline compatibility

Keep accepted raw hold/integration arithmetic, including its existing possible overflow on extreme finite payloads; do not silently clip or add a new baseline-finiteness rejection that breaks W0 bit compatibility. Typed NumericalFailure refers to the unchanged core filter prediction/update failure. Resource/qualification profiles use their prescribed ordinary finite generated values. Error fixtures must distinguish filter failure from raw baseline behavior and cannot claim stronger legacy invariants.

## Diagnostic units

Each history measurement entry counts as one logical history observation whether it stores an index/reference or a copied payload; count original and candidate entries separately. Full input Trace payloads and global transport schedule are not these history entries and must remain included in total process memory. Peak reserved checkpoint/history-entry capacities count actual vector/array slots across original and candidate throughout the tick; they are not inferred from peak live lengths. Current Kalman/baseline scalar scratch is separately inventoried constant state. Tests require actual peakreserved capacities to cover peaklive entries; VER-002 inspects construction and bounds beyond selected inputs.
