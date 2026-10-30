# Draft public API and version-one artifact contract

### Minimal public surface

All data-bearing types have private fields and read-only accessors. Constructors validate before returning. Sensor is a closed Position/Velocity enum; ObservationId contains sensor and scheduled sequence. Tick/sequence fields use u32 with checked operations. No public mutable filter or runtime type is exposed. The following signatures define this normative draft boundary:

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

Read-only accessors are part of this candidate: Observation exposes id(),sample_tick(),planned_arrival_tick(),value(); ObservationId exposes sensor(),sequence(). ReplayModel exposes assumed_position_sigma(),assumed_velocity_sigma(),acceleration_density(). Trace exposes duration_seconds(),pending_capacity(),drop_every(sensor),observations()->&[Observation]. ReplayResult exposes snapshots()->&[ReplaySnapshot],ledger()->&[LedgerEntry],arrival_order()->&[ObservationId]. ReplaySnapshot exposes tick(),estimate()->State,covariance()->&[[f64;4];4],held_position()->[f64;2],integrated_position()->[f64;2]. LedgerEntry exposes id(),sample_tick(),planned_arrival_tick(),decision_tick(),outcome(),queue_occupancy()->Option<u16>. ObservationId/Sensor/outcomes are Copy equality-capable values; no public constructor permits an unchecked identity or mutable state alias. Other scalar/accessor values return copies. No public snapshot/ledger/result constructor is proposed.

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
position_assumed_sigma<TAB>P
velocity_assumed_sigma<TAB>V
acceleration_density<TAB>Q
buffer<TAB>capacity<TAB>C<TAB>position_drop_every<TAB>NP<TAB>velocity_drop_every<TAB>NV
observations<TAB>N
planned_arrival_tick<TAB>sample_tick<TAB>sensor<TAB>sequence<TAB>x<TAB>y
~~~

NP/NV token0means None; positive tokens fit NonZeroU32. Sensor tokens are position/velocity. Exactly Nobservation rows follow in canonical arrival order, including planned records later subject to scripted/queue loss. Delay/capacity/model values are parsed and revalidated; unknown version/policy/clock/fields are errors. The virtual_planned label is not a record of actual async scheduler timing. A future actual adapter requires its own reviewed arrival-recording contract.

Ledger header fixes version1, clock10, policyOnTimeOnly, duration, offered count and the five outcome counts. Its source section declares source_rows=N, then sensor/sequence/sample_tick/planned_arrival_tick/decision_tick/outcome/queue_occupancy columns in source order. Its arrivals section declares arrivals=M, then sensor/sequence rows in actual virtual-consumer order. Lowercase outcome tokens are scripted_loss,queue_full_drop,accepted,stale_drop,undelivered_at_horizon. queue_occupancy is canonical0 for not applicable and1..128 only for QueueFullDrop, equal to that validated capacity; its getter is Some only for QueueFullDrop. No payload, truth or big snapshot file is required in the ledger. Ledger import is not a first-slice public API; replay computes it from validated input. Caller-owned companion provenance may hash the exact trace/ledger bytes with existing external tools; do not roll an ad-hoc cryptographic hash into the library.

The32MiB cap applies only to encoded trace input/output, not the existing360001numerical snapshots or source Experiment. The ledger has a separate16MiB encoded cap. Decoder checks input byte length before parsing/copying, row count before allocation and bounded token/line shapes; serializers return a complete capped Vec or error. An outer file-reading example must read at most cap+1bytes before calling decode, rather than allocating an unbounded file. Measured peakRSS candidate512MiB covers source Experiment + projection + replay + serialization in the declared maximum-duration workload; it is a non-OS-limit evidence obligation requiring actual environment/provenance and a defined sampler. A failed measurement triggers design/resource repair, not a false hard-cap claim.

### Errors, equality and commit semantics

Use a structured DeliveryError surface, with InvalidPlanDelay(sensor,delay_ticks) for an invalid plan before any observation exists: InvalidDuration, InvalidCapacity, InvalidModel(field enum), TooManyObservations, InvalidSourceSamples(index,reason enum), SampleBeyondHorizon(id), InvalidSequence(id), ArrivalBeforeSample(id), DelayExceeded(id), NonFinitePayload(id,axis), DuplicateIdentity(id), NonCanonicalOrder(row), MalformedTsv(row,field,reason enum), UnsupportedSchema, ArtifactTooLarge(kind,limit), and NumericalFailure(tick,optional observation id,original Error). Callers match variants/fields, not display text. Variant names and fields match the diagnostic API surface; these cases cannot be collapsed into successful drops. MalformedTsv uses one-based row/field locations, with row0/field0 reserved for whole-artifact encoding/newline errors. InvalidSourceSamples uses source.len() for WrongCount, otherwise the offending zero-based source index. NonCanonicalOrder reports the second out-of-order row (first observation is row12). Duplicate checking covers the full identity inventory before any import-order repair. Duration/capacity/count are rejected before contextual allocation; ArtifactTooLarge on input precedes syntax inspection.

No I/O variant is needed for byte-slice/Vec APIs; the example owns filesystem errors/publication.

Validation errors return no Trace/Model/ReplayResult and do not mutate borrowed inputs or create files. replay_delivery owns its newly initialized filter/baselines internally and returns either the complete result or an error; no successful prefix is returned. Existing per-update numerical state/covariance commit semantics remain, but they do not turn a failed replay into partial success. Serialization likewise returns no partial successful buffer. Writing that buffer to disk can still fail partially; the caller/example must use a temporary artifact and publish only after successful complete write/flush, with failures disclosed. No claim of transactionality across arbitrary caller Write implementations or power loss is made.

Same-target reproducibility compares every snapshot f64 by to_bits and all ledger/arrival integer/outcome values exactly; canonical trace/ledger bytes must also match on regeneration. Ordinary f64 PartialEq alone does not establish signed-zero identity, and no Eq law is invented for NaN-capable public f64 values. Zero-delay/no-loss equivalence uses the old run's exact operation ordering and resolved model inputs. Cross-target equality is a separately documented tolerance/evidence question, not a blanket bitwise promise.

This formal draft still requires main review and external approval/locking before implementation. First-slice acceptance must state that it is pure virtual delivery evidence, not actual channel saturation/slow-consumer/cancellation proof and not completion of parent6.

## Additional exact constants and diagnostics

Version1 starts at state[0,0,0,0] with covariance diagonal[25,25,4,4]. Continuous white acceleration q uses the unchanged F/Q equations and Joseph update in the root numerical contract, with requested variance floor1e-12. No prior/floor override field exists in the model or header.

Tokens are at most327ASCIIbytes; line bodies excluding LF at most1024bytes. Three separate model lines are mandatory: the earlier combined-line proposal was1053bytes for valid smallest-positive values and is not this schema. Canonical finite float text is exactly parse-to-f64 then default RustDisplay-to-string, including -0; no permissive alternate exponent or1.0 spelling when Display is1. Integer grammar uses decimal digits only, with no leading zeros except0. Domain nonfinite payload errors preserve id/axis even for records that would later be stale or lost.

The exact error enum and read-only getter signatures are declared in the diagnostic src/delivery.rs surface. It has no behavior beyond one named panic. InvalidPlanDelay identifies a sensor stream rather than inventing an observation id. Observations at u32-maximum compatible sensor schedules can be constructed but a duration-bounded Trace rejects them as beyond horizon. All eight validated structs have private fields; no public constructor exposes an unchecked result, snapshot or ledger.

Ledger exact header order is magic(version1),clock_ms(10),policy(OnTimeOnly),duration_seconds,offered,counts,source_rows,column header,source rows,arrivals,column header,arrival ids. Counts order is accepted,stale_drop,scripted_loss,queue_full_drop,undelivered_at_horizon. The test-owned support golden artifacts give an independent complete three-observation byte example. The library does not import ledgers.

## Decoder failure locations

Input byte cap precedes syntax checks. Validate UTF8 first and return whole-artifact InvalidEncoding on failure. After successful UTF8 validation, reject a leading BOM or any CR as NonLfOrBom BEFORE rejecting other non-ASCII text as InvalidEncoding. Then require final LF and apply line/token checks. Missing final LF reports MissingFinalNewline. These whole-artifact failures use row0/field0. Line-body1024 limits are checked before header/field parsing; a line over the limit reports its one-based row and field0. A token over327 reports its one-based row/field before numeric parsing. The three split model lines make the first observation row12. Canonical integer parsing distinguishes numeric-but-noncanonical leading/sign tokens from invalid integer spelling; parsed nonfinite observation payloads return NonFinitePayload with sensor/id/axis even if they would later be stale. Full-inventory duplicate identity rejection also applies when its rows are nonadjacent in canonical arrival order.
