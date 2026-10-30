# Timestamped observation delivery: design preflight

Status: DRAFT for [issue6](https://github.com/zblack54/fusion-lab/issues/6). Design discussion only: no production behavior, diagnostic seam, executable harness, package lock or implementation approval. Main must select the first boundary and numerical/resource contracts before a formal contract is generated. This is a proposed partial prerequisite for [policy comparison22](https://github.com/zblack54/fusion-lab/issues/22) and the delay/loss portion of [research8](https://github.com/zblack54/fusion-lab/issues/8), not completion of those issues.

## Existing boundaries and reason for a narrow first slice

The current root run loop owns integer10ms truth advancement, synchronous filter prediction/update, sensor sampling, both baselines and evaluation. Position samples occur every20ticks, velocity every5; position precedes velocity when simultaneous. Sensors use independent ChaCha8 streams and draw scheduled position noise even during an outage. The private KalmanFilter receives only observations, never truth. Configured bias/drift change sensor observations, not that separation.

The accepted experiment service runs a child CLI and owns HTTP, artifact retention, cancellation/reaping and its Tokio runtime. It does not currently stream raw observations into a filter. The dashboard replays recorded numerical states; its presentation clock is not a delivery clock. Adding a Tokio channel directly inside physics would mix these boundaries and make scheduling an accidental experiment parameter.

Recommend first making observation time and deterministic delivery decisions explicit in a pure synchronous replay path. This gives a small inspectable stale-drop baseline before real channel lifetimes and before historical correction. The default run/CLI35column output and accepted service/dashboard wire contracts remain unchanged in this slice. A separate recorded-delivery command/example and artifact schema would be selected explicitly, rather than silently widening Config or the service request schema.

## Proposed first-slice input and output boundary

Prefer a private delivery module with a deliberately small validated public wrapper, not a new runtime-bearing crate or a public mutable KalmanFilter. Conceptually separate three values:

1. Generation provenance and optional evaluation truth, owned by the experiment/evaluation side.
2. Validated replay input: duration, estimator assumptions and timestamped observation payloads only. No truth states, trajectory, sensor seed or Experiment reference is passed to the estimator/baseline owner.
3. Replay result: post-arrival-tick estimates/covariances, policy-aligned baseline states, disposition ledger and counts. Evaluation receives result plus a separate truth series and checks tick alignment.

Names such as DeliveryTrace::from_experiment and replay_delivery illustrate the call boundary; main must choose the actual API before tests. For the smallest generator, extract only present position/velocity observations from an existing Experiment using a borrowed conversion. This reuses the established sensor realization exactly; it does not use the old filter estimates as new estimator inputs. The borrowed source Experiment may contain truth, but its observation projection must copy only sensor payload/identity/time, then end that borrow before replay. A later internal shared observation generator could remove the cost of the already-computed source estimate, but is not needed to prove the first contract and must not introduce a second inconsistent sensor path.

Avoid embedding Experiment or the source Config in replay inputs merely for convenience: Config includes seed/scenario and is more authority than the estimator requires. The replay owner needs only resolved assumed measurement variances and process density (existing bounds/floor), duration and validated observations. Generation provenance remains a companion artifact outside that owner.

For simple first-slice fault generation, propose fixed per-channel delay and an explicit drop-every-N schedule, with an optional separately validated explicit trace for hand-crafted reordering cases. Loss is selected from sensor identity/sequence, not by consuming either sensor RNG. Whether arbitrary trace import is exposed publicly or confined to a validated library constructor is a main decision; no broad plugin/transport abstraction is needed yet.

## Time, identity and canonical order

Use checked integer ticks everywhere. Tick0 and the configured final tick remain included; no wall clock or accumulated floating time advances physics. Distinct fields:

- sample_tick: when the sensor value was generated.
- arrival_tick: when it becomes available to the consumer in the recorded virtual delivery model. Require arrival_tick>=sample_tick.
- sensor: closed Position/Velocity identity.
- sequence: per-sensor scheduled sample index, starting at0, fixed by sample_tick/20 or sample_tick/5. Suppression/loss leaves gaps rather than renumbering later observations. Identity is (sensor,sequence), never arrival order.

For one timestamped observation, retain its exact finite two-axis payload unchanged. A source ledger records explicit scripted loss or queue loss separately from an available observation; an outage suppresses position delivery and is generation provenance, not a fabricated zero observation. Missing and zero are distinct.

Canonical consumption order is (arrival_tick,sample_tick,sensor_priority,sequence), with Position priority0 and Velocity1. Thus older sampled data is considered first among a shared arrival tick; simultaneous on-time position still precedes velocity. Timestamps never change in sorting. Record this order explicitly and reject duplicate observation identities, malformed sequence/time/sensor combinations and unsupported schema/clock before mutation. Do not silently accept a duplicate as new information or replace an earlier payload. A later adapter that permits duplicate delivery attempts needs an explicit attempt identity and deduplication contract; it is outside this first slice.

At each replay tick, predict exactly one10ms step (except0), advance integration using the previously accepted velocity, process arrivals in canonical order, then record the snapshot. Source truth is advanced independently by its existing synchronous generator/evaluator, never by awaiting delivery. The default zero-delay/no-loss trace must reproduce the old estimate/covariance/hold/integration path on the same supported environment, using identical operation/update ordering.

## Deliberately strict stale-drop baseline

Recommend the initial policy name OnTimeOnly: accept an observation only when sample_tick==current arrival tick. If sample_tick<arrival_tick, record StaleDropped and perform no filter or baseline update. If sample time is in the future, reject malformed input rather than predicting opportunistically. Do not treat an old position as though it measured current position, and do not rewind only a timestamp while leaving covariance at the arrival time.

This means every positive-delay observation is dropped by this baseline. That is intentional and should be displayed plainly. It is a conservative reference, not a claim that useful delayed measurements cannot be incorporated. A policy accepting observations merely because their sample index exceeds the last accepted index would still apply an old-time measurement to current state and is not an acceptable implicit alternative.

Hold and integration receive the same accepted observation subset, in the same order, as the filter. Hold changes only on an accepted position. Integration uses the last accepted velocity from the preceding tick and resets only on an accepted position. They cannot peek at source positions during outages or consume stale arrivals that the filter rejected while being labeled same-information baselines. Record offered, accepted and discarded counts so starvation is visible.

For later policy comparison22, all strategies must receive the same available arrival trace before their different policies. A bounded-history policy may use an observation the stale-drop policy discards; that is the declared treatment difference, not permission to give it a different sensor realization. Report policy-aligned baseline variants or one explicitly fixed common baseline separately, rather than quietly changing baseline information between comparisons.

## Proposed finite limits and deterministic overload

Main should approve exact limits before a harness. A small candidate domain is existing duration1–3600s, maximum delivery delay500ticks(5s), and at most90002scheduled observations (18001position+72001velocity at the maximum duration). Arrival arithmetic is checked; maximum planned arrival is360500ticks, safely below u32 but never relying on unchecked wrapping. Payloads must be finite; no artificial finite Gaussian amplitude bound is introduced. Existing numerical failures remain typed failures, not converted to dropped samples.

Model a virtual waiting buffer with configurable capacity1–128observations. It contains only events whose arrival is later than the current tick. Drain due arrivals before admitting new delayed samples at that tick. Zero-delay observations do not need a waiting slot. Generate/admit simultaneous source samples position then velocity. When a future observation would exceed the waiting capacity, deterministically drop that new observation, preserve older buffered entries, and record QueueFullDropped with identity, source/arrival times and occupancy. No blocking, hidden unbounded queue, evict-oldest ambiguity or change to source physics/RNG is permitted.

The waiting bound is independent of the bounded full immutable trace retained for replay. At most127distinct scheduled observations can be due at one tick under a500tick delay bound; exact maximum/burst reasoning must be verified against the chosen generator/identity constraints, not assumed for arbitrary imported traces. A full trace still has an explicit global event cap even if the waiting buffer is small. Recommend a bounded artifact schema with checked row count and byte cap (candidate32MiB), no unbounded strings, and a documented in-memory allowance measured before acceptance. Do not claim a file cap is an OS-enforced RSS limit.

Snapshots stop at the configured final simulation tick. Planned arrivals beyond it are accounted as UndeliveredAtHorizon, not applied, not described as already received and not used to extend truth/evaluation silently. End-of-run ledger accounting must partition generated deliverable observations into scripted loss, queue-full loss, stale drop, accepted and undelivered-at-horizon, with no unexplained pending item. If main instead wants tail draining beyond the evaluated horizon, that is a different contract requiring explicit snapshot/evaluation boundaries.

No real tasks, channel or wall-time consumer exists in the first slice. Virtual queue saturation is useful deterministic evidence but is not proof of actual slow-consumer behavior or asynchronous shutdown. The synchronous replay returns a complete bounded result or an error; numerical failure does not produce a successful partial replay. The caller owns the operation and outputs. Existing service cancellation still owns a child process; it must not be presented as newly validated observation-channel cancellation.

## Replay artifacts and human review evidence

Use a new versioned trace/ledger artifact rather than changing the current35column recording. Header/provenance identifies10ms clock, duration, schema/order/policy, estimator q/resolvedR/floor, fault plan/capacity, source observation inventory/hash and source/config/toolchain/target provenance. Estimator input receives only its observational projection. Round-trip f64 payloads are necessary for replay proof; the old12decimal plot CSV must not be described as an exact unrounded source trace. Store original payload/time/identity and dispositions without lossy rewriting.

A replay command validates header, counts, all input rows, uniqueness, bounds and full ordering before estimator state is committed. Same trace and declared environment must reproduce recorded numerical outputs and dispositions within an explicitly selected equality contract. Prefer exact same-target f64/output identity for this pure path, with cross-platform tolerance specified separately; never claim universal bitwise reproducibility. Check hashes and regeneration commands with at least one independently inspected small trace. Record failures rather than adjusting event order or tolerances after seeing them.

## Independent acceptance examples to specify after main selection

- Zero-delay/no-loss source trace: exact original samples/estimates/covariances/baselines at every tick on the reference environment; sensor RNG/source observations unchanged.
- A position at sample0/arrival0 is accepted; a velocity at sample0/arrival1 is StaleDropped. Replay state/covariance/baselines at tick1 match a trace omitting that delayed velocity, not a current-time velocity update. Ledger order and reason are explicit.
- Two different sensors with the same sample/arrival tick process position first. Two arrivals sharing arrival time but different sample times retain the stated older-sample-first ordering. Reordering source enumeration before canonical validation cannot silently change replay semantics.
- Capacity1: position sample0/arrival10 occupies the waiting slot; velocity sample0/arrival5 is QueueFullDropped. An on-time velocity sampled/arriving at5 bypasses the future waiting buffer and can be accepted. Position0 arriving10 is stale and removed; no hidden old observation updates the estimator.
- Arrival after the final tick is accounted UndeliveredAtHorizon; no extra snapshot or infinite drain. Exact boundary arrival at the final tick is considered normally.
- Reject sample>arrival, invalid sensor sequence, duplicate identity, nonfinite payload, out-of-delay/global/byte cap, malformed/truncated/unsupported schema and ordering before mutation/output publication. An actual numerical failure remains visible and preserves the filter's last valid state internally, with no completed replay claim.

These are proposed observable examples, not generated tests or evidence that the feature exists. A formal contract should select exact types, equality/error semantics, import rules, resource obligations and non-test verification before implementation. Avoid generating a large package that merely validates its own fixtures.

## Ordered later slices and issue completion boundaries

1. Main selects/approves this pure trace + OnTimeOnly + bounded virtual waiting contract and its narrow API/artifact/reproduction surface. Then create/review the proportionate formal executable contract, preserving existing physics/locked service/dashboard obligations. Implement and independently review a focused partial6PR.
2. Under a separately reviewed cross-component contract, add a bounded actual async delivery adapter outside physics. One session owner holds all producer/consumer handles, bounded channel/capacity, cancellation source and error state. Admission stops on cancellation; no new tasks after stop. Choose deterministic recorded reject/drop semantics versus producer waiting explicitly. Avoid letting a full channel delay source simulation time and resample different noise. Consumer never mutates truth. Record actual observed availability/arrival ordering before replay; replay consumes that trace rather than rerunning scheduler races.
3. Test actual queue saturation, controlled slow consumer, producer/consumer error, closure, cancellation at blocked-send/receive boundaries and shutdown joining every handle. Count/deduplicate each delivered/dropped identity, bound active tasks and bytes, and observe task errors separately from inner operation errors. No detached task, unbounded spawn_blocking backlog or assumption that timeout/abort reaped a child. Extend the existing service only through its controlled interface/contract revision; local application lifecycle remains owned.
4. For22, design bounded-history correction: explicit history span/event/state capacity, late admission cutoff, stable sample-order reapplication, covariance restoration and repropagation, CPU work bound, and whether old accepted observations/baselines are recomputed. Test a hand-derived correction case and exact history-bound edges before comparative work. Never bolt timestamp rewind onto the present current-state filter as if it were historical replay.
5. For8/22research, freeze hypotheses, fixed seed/scenario/delay/loss plans, same available trace inventory, metrics/recovery definitions, resource/latency measurement, stopping rule and negative-outcome reporting before execution. Numerical virtual latency and actual adapter wall latency are separate metrics. Existing outage evidence is partial8; it does not establish delay/loss behavior.

Issue6 remains open after the pure first slice because real bounded channels, slow consumers and owned shutdown are still absent. Issue22 remains future until a reviewed history policy exists. Issue8's completed outage/robustness evidence cannot be relabeled as delivery research. None of these designs requires a paid model API, deployment, a new estimator, EKF or a runtime in the numerical physics loop.

## First-slice API and artifact candidate for issue69

Main selected the pure first slice as [issue69](https://github.com/zblack54/fusion-lab/issues/69), with parent6remaining open. Selected boundaries: synchronous observation-only replay, OnTimeOnly, canonical arrival/sample/sensor/sequence ordering,90002observation cap, duration1–3600s, delay0–500ticks, waiting capacity1–128, due-arrival drain before future admission and zero-delay slot bypass. The exact API/format below remains a candidate for main review before any formal harness or diagnostic seam. No implementation authorization is implied.

### Minimal public surface

All data-bearing types have private fields and read-only accessors. Constructors validate before returning. Sensor is a closed Position/Velocity enum; ObservationId contains sensor and scheduled sequence. Tick/sequence fields use u32 with checked operations. No public mutable filter or runtime type is exposed. Illustrative Rust signatures are the proposed complete behavioral boundary, not source code:

~~~rust
Observation::new(sensor: Sensor, sequence: u32, sample_tick: u32,
                 planned_arrival_tick: u32, value: [f64; 2])
    -> Result<Observation, DeliveryError>;
ReplayModel::new(assumed_position_sigma: f64, assumed_velocity_sigma: f64,
                 acceleration_density: f64) -> Result<ReplayModel, DeliveryError>;
DeliveryPlan::new(position_delay_ticks: u32, velocity_delay_ticks: u32,
                  pending_capacity: usize,
                  position_drop_every: Option<NonZeroU32>,
                  velocity_drop_every: Option<NonZeroU32>)
    -> Result<DeliveryPlan, DeliveryError>;
Trace::new(duration_seconds: u32, pending_capacity: usize,
           drop_every: [Option<NonZeroU32>; 2], observations: Vec<Observation>)
    -> Result<Trace, DeliveryError>;
Trace::from_samples(duration_seconds: u32, source: &[Sample], plan: DeliveryPlan)
    -> Result<Trace, DeliveryError>;
Trace::decode_tsv(bytes: &[u8]) -> Result<(ReplayModel, Trace), DeliveryError>;
Trace::encode_tsv(&self, model: &ReplayModel) -> Result<Vec<u8>, DeliveryError>;
replay_delivery(model: &ReplayModel, trace: &Trace)
    -> Result<ReplayResult, DeliveryError>;
ReplayResult::encode_ledger_tsv(&self) -> Result<Vec<u8>, DeliveryError>;
~~~

Read-only accessors are part of this candidate: Observation exposes id(),sample_tick(),planned_arrival_tick(),value(); ObservationId exposes sensor(),sequence(). ReplayModel exposes assumed_position_sigma(),assumed_velocity_sigma(),acceleration_density(). Trace exposes duration_seconds(),pending_capacity(),drop_every(sensor),observations()->&[Observation]. ReplayResult exposes snapshots()->&[ReplaySnapshot],ledger()->&[LedgerEntry],arrival_order()->&[ObservationId]. ReplaySnapshot exposes tick(),estimate()->State,covariance()->&[[f64;4];4],held_position()->[f64;2],integrated_position()->[f64;2]. LedgerEntry exposes id(),sample_tick(),planned_arrival_tick(),decision_tick(),outcome(). ObservationId/Sensor/outcomes are Copy equality-capable values; no public constructor permits an unchecked identity or mutable state alias. Other scalar/accessor values return copies. No public snapshot/ledger/result constructor is proposed.

ReplayModel contains only requested assumed sigmas and q, each finite in0..=100; requested sigma is squared at the same numerical boundary and the existing1e-12variance floor applies. It contains no Config, true-noise sigma, truth, seed, trajectory or source Experiment. Duration belongs to Trace. DeliveryPlan adds no RNG and is used only for source projection; its fixed delays become explicit per-observation planned arrival times. Trace::new may therefore represent hand-crafted reordering without pretending every trace used a constant-delay generator.

from_samples checks dense tick ordering/count against the supplied duration, allowed observation sample ticks and finite payloads, then copies only tick/measurement fields. It never reads truth, source estimates/covariances/baselines or evaluator fields. A scheduled absent position remains absent; no zero or hidden outage value is invented. It assigns position sequence=sample_tick/20 and velocity sequence=sample_tick/5, leaving gaps. The source slice borrow ends when the owned Trace returns; replay has no reference back to it. Source Config/provenance remains caller-owned. A structural inspection and a case changing only non-observation Sample fields must establish that authority boundary.

Observation::new validates finite payload, sensor schedule/sequence consistency, causality and delay<=500, with checked arithmetic. Trace validates sample<=duration*100, uniqueness/global count/capacity and all remaining context bounds before allocating replay state. Constructor-provided arbitrary observation enumeration is normalized into canonical (planned_arrival_tick,sample_tick,Position-before-Velocity,sequence) order. Artifact import instead requires already canonical order and rejects any inversion/duplicate, rather than repairing an untrusted recorded order silently. The input may be sparse; its manifest count defines the exact offered observation inventory, not a fabricated complete source realization.

Drop-every-N uses scheduled sequence: drop when (sequence+1) mod N==0, with checked addition. N=1drops all; None drops none. For position N=3, missing/outage sequence1does not renumber sequence2: sequence2is still the scripted loss. This decision is made at sample time before zero-delay delivery or future-buffer admission. Canonical source processing for admission is (sample_tick,sensor_priority,sequence), independent of arrival enumeration.

### Result and decision ledger

ReplayResult owns exactly last_tick+1read-only ReplaySnapshot values and one ledger entry per offered observation, plus read-only arrival identity order. A snapshot exposes tick, estimate State, full covariance, held position and integrated position; it contains no truth, source Config or pre-arrival observation payload. LedgerEntry exposes identity, sample_tick, planned_arrival_tick, decision_tick and one closed outcome:

| Outcome | decision_tick | Reached consumer as an arrival? |
|---|---|---|
| ScriptedLoss | sample_tick | no |
| QueueFullDrop | sample_tick | no |
| Accepted | planned_arrival_tick | yes, on-time |
| StaleDrop | planned_arrival_tick | yes, late |
| UndeliveredAtHorizon | final tick | no |

The source ledger is serialized in (sample_tick,sensor_priority,sequence) order, even when its decision times are nonmonotonic. A distinct arrival-order identity list includes only Accepted/StaleDrop entries in canonical arrival order. Thus a source decision does not falsely assert that a lost or queue-rejected observation arrived. Planned arrival remains explicitly labeled as planned in every row. Due arrivals drain before new future admissions; on-time observations bypass waiting slots. Baseline/filter update occurs only for Accepted. Queue-full/scripted losses never enter the arrival list. Ledger outcome counts partition exactly the offered inventory, and no pending state remains after horizon finalization.

### Strict version1 TSV

Use ASCII, LF endings, a final newline, no BOM/comments/blank lines, exact header/column names and no extra fields. Canonical unsigned decimals have no sign/leading zeros except token0. Finite f64 tokens must round-trip through Rust parse and to_string unchanged, including the distinction between0and-0; this is a strict Rust-produced format, not a permissive generic spreadsheet importer. No JSON/serialization dependency is added.

Trace header order is fixed:

~~~text
fusion-lab-delivery-trace<TAB>1
clock_ms<TAB>10
clock_kind<TAB>virtual_planned
policy<TAB>OnTimeOnly
duration_seconds<TAB>D
model<TAB>position_assumed_sigma<TAB>P<TAB>velocity_assumed_sigma<TAB>V<TAB>acceleration_density<TAB>Q
buffer<TAB>capacity<TAB>C<TAB>position_drop_every<TAB>NP<TAB>velocity_drop_every<TAB>NV
observations<TAB>N
planned_arrival_tick<TAB>sample_tick<TAB>sensor<TAB>sequence<TAB>x<TAB>y
~~~

NP/NV token0means None; positive tokens fit NonZeroU32. Sensor tokens are position/velocity. Exactly Nobservation rows follow in canonical arrival order, including planned records later subject to scripted/queue loss. Delay/capacity/model values are parsed and revalidated; unknown version/policy/clock/fields are errors. The virtual_planned label is not a record of actual async scheduler timing. A future actual adapter requires its own reviewed arrival-recording contract.

Ledger header fixes version1, clock10, policyOnTimeOnly, duration, offered count and the five outcome counts. Its source section declares source_rows=N, then sensor/sequence/sample_tick/planned_arrival_tick/decision_tick/outcome columns in source order. Its arrivals section declares arrivals=M, then sensor/sequence rows in actual virtual-consumer order. Lowercase outcome tokens are scripted_loss,queue_full_drop,accepted,stale_drop,undelivered_at_horizon. No payload, truth or big snapshot file is required in the ledger. Ledger import is not a first-slice public API; replay computes it from validated input. Caller-owned companion provenance may hash the exact trace/ledger bytes with existing external tools; do not roll an ad-hoc cryptographic hash into the library.

The32MiB cap applies only to encoded trace input/output, not the existing360001numerical snapshots or source Experiment. The ledger has a separate16MiB encoded cap. Decoder checks input byte length before parsing/copying, row count before allocation and bounded token/line shapes; serializers return a complete capped Vec or error. An outer file-reading example must read at most cap+1bytes before calling decode, rather than allocating an unbounded file. Measured peakRSS candidate512MiB covers source Experiment + projection + replay + serialization in the declared maximum-duration workload; it is a non-OS-limit evidence obligation requiring actual environment/provenance and a defined sampler. A failed measurement triggers design/resource repair, not a false hard-cap claim.

### Errors, equality and commit semantics

Use a structured DeliveryError surface: InvalidDuration, InvalidCapacity, InvalidModel(field enum), TooManyObservations, InvalidSourceSamples(index,reason enum), SampleBeyondHorizon(id), InvalidSequence(id), ArrivalBeforeSample(id), DelayExceeded(id), NonFinitePayload(id,axis), DuplicateIdentity(id), NonCanonicalOrder(row), MalformedTsv(row,field,reason enum), UnsupportedSchema, ArtifactTooLarge(kind,limit), and NumericalFailure(tick,optional observation id,original Error). Callers match variants/fields, not display text. The exact variant spelling can be settled in the formal contract, but these distinct observable failure cases cannot be collapsed into successful drops. No I/O variant is needed for byte-slice/Vec APIs; the example owns filesystem errors/publication.

Validation errors return no Trace/Model/ReplayResult and do not mutate borrowed inputs or create files. replay_delivery owns its newly initialized filter/baselines internally and returns either the complete result or an error; no successful prefix is returned. Existing per-update numerical state/covariance commit semantics remain, but they do not turn a failed replay into partial success. Serialization likewise returns no partial successful buffer. Writing that buffer to disk can still fail partially; the caller/example must use a temporary artifact and publish only after successful complete write/flush, with failures disclosed. No claim of transactionality across arbitrary caller Write implementations or power loss is made.

Same-target reproducibility compares every snapshot f64 by to_bits and all ledger/arrival integer/outcome values exactly; canonical trace/ledger bytes must also match on regeneration. Ordinary f64 PartialEq alone does not establish signed-zero identity, and no Eq law is invented for NaN-capable public f64 values. Zero-delay/no-loss equivalence uses the old run's exact operation ordering and resolved model inputs. Cross-target equality is a separately documented tolerance/evidence question, not a blanket bitwise promise.

These selected interfaces still need main review and the later executable/non-test contract. No seam, tests or lock are authorized by this refinement itself. First-slice acceptance must state that it is pure virtual delivery evidence, not actual channel saturation/slow-consumer/cancellation proof and not completion of parent6.
