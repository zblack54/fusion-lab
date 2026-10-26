# Validation report: SPEC-SUPERVISOR controlled revision two

Status: AI_READY

Main /root approves revision2 under owner delegation and AGENTS.md after independent reviewer clearance and main review of exact-byte prompt/readonly mutation oracles, architecture and dependency closure. Final-byte execution and new externally recorded lock follow. This approval authorizes implementation, not native sandbox or product acceptance.

## Scope, baseline and ownership

Published native baseline: 9204bdbfeaa24c0923aee0ac3c33ebc293e1570d.
Main independently advanced service/docs work to da784bb6
without changing preserved supervisor source assumptions. Original revision-one
contract, harness and full native implementation remain in parent-managed
contract history. The current lib/main are minimal diagnostic seams with the
preserved consumer types plus the specified Pause::EvidenceMismatch variant; no scope-gate behavior, scheduler or actual inference was
implemented during this design revision. Existing native modules remain
preserved for main to restore only after approval.

Native Windows x86_64-pc-windows-msvc; Edition 2024, Rust floor 1.99, resolver 3.
CFG-001 is Rust 1.99.0; CFG-002 is stable. Numerical core APIs and its Windows/Linux
promise are unchanged. No unrelated package, core, workflow or root lock edit.

## Catalog and traceability

Eight acceptance scenarios, 44 active requirements, seven ownership modules,
ten constraints, 56 required executable tests, nine verification records and
five dependency-ordered slices. All 43 original selectors remain; none was
removed, ignored or conditionally exempted. Thirteen new tests add real scope
and control-isolation entrypoint contracts. All eleven test classes are covered:

| Class | Count |
|---|---:|
| negative | 18 |
| acceptance | 10 |
| behavioral | 3 |
| boundary | 6 |
| invariant | 2 |
| state_transition | 4 |
| anti_pattern | 5 |
| property | 1 |
| contract | 5 |
| performance | 1 |
| structural | 1 |

REQ-037–044 specify exact-byte SHA256 scope binding, strict typed identity,
actual approved instructions in the prompt, enforced sandbox mode, private
control isolation, raw/encoded caps, CLI checkout compatibility and durable read-only HEAD-mutation rejection. CON-008/009/010
cover extra/temp write allowances, bounded non-link scope reads and preserved mutation evidence with explicit owner disposition. Reciprocal
traceability links tests and concrete VER-009 to MOD-007 and proposed ADR-003.
TASK-005 depends on durable state and precedes native worker dispatch; it does
not create a dependency cycle. There are no waivers or unresolved product questions.

## Actual final execution

The trusted schema-three runner from the parent's pinned external tools was run:

~~~text
python <trusted-tools>/verify_test_suite.py docs/design-docs/local-supervisor --mode design --offline --timeout 60
python <trusted-tools>/validate_design_package.py docs/design-docs/local-supervisor
~~~

Final runner: success=true, errors=[], exactly 56 expected-red observations per
CFG-001 and CFG-002 (112 total), zero unexpected failures and zero unexpected
passes. Every selector was compiled, enumerated and executed individually;
source-name matches or zero-test exits do not establish coverage. Every red
result is the normal diagnostic seam marker SUPERVISOR_NOT_IMPLEMENTED, not a
build error, timeout, malformed fixture or missing test. All six CMD-001–006
supplemental formatting/Clippy/doctest/documentation gates passed. Structural
validation returned zero errors and only the intentional unlocked warning.
validation/design-execution.json records exact commands, compiler metadata,
inputs, outputs and fingerprints. Its candidate fingerprint is
fec3b49bc5a8ca3e47bd272e98bfa7a8c660457cbeddc48aa940c63b8310775e; this is NOT an approval baseline or lock digest.

A first structural check found a contract-kind selector incorrectly listed as an
acceptance-kind selector; that metadata link was corrected before successful
validation. A focused encoded-prompt test independently verified its overflow
oracle before reaching the named diagnostic seam. The first offline dependency
check reported missing cpufeatures; normal Cargo fetch filled the exact closure.
No gate, assertion, TLS validation or diagnostic expectation was weakened.

## Dependencies and immutable fixture boundary

Production and harness both pin sha2=0.10.9; direct harness serde_json=1.0.149 is
already the reviewed production version. Both lockfiles were refreshed and the
offline inventory regenerated. The closure is now 46 registry packages versus
38 in revision one: all 38 old versions are unchanged, eight SHA256-related
packages were added, all license expressions are recorded and no declared Rust
floor exceeds 1.99. Some releases declare no MSRV; actual native 1.99.0/stable
compilation establishes this project's support, not an invented upstream floor.
Read verification/dependency-review.md and dependency-inventory.json for exact
versions, features, license expressions and both lock fingerprints.

The test-owned SHA256 helper checks the hard-coded known abc vector, then hashes
exact fixture bytes independently of the supervisor. Fixtures live entirely in
the declared harness. Native tasks get actual materialization and a separate
checkout child; state, bridge and scope control remain siblings outside it.
Host ACK helpers use the actual approved digest. Pure domain tests retain toy
identifiers because they do not dispatch. The fake Codex remains an external
protocol peer, observes the real prompt/config and implements no supervisor
state or approval. No runtime fake-peer exception is proposed.

## Efficacy and non-test limits

Missing, altered, wrong-identity, malformed, oversized and overlapping scopes
require exact typed rejection, full committed snapshot preservation, zero worker
launches, no sentinel/native reservation and no PR intent. Positive dispatch
requires the received raw approved_scope UTF8 SHA256 to equal the independently materialized file digest, observed expected_head to equal initial Git HEAD, approved text/digest and all sandbox extra/temp controls. Separate SDK
and CLI read-only positives preserve legacy same-path behavior and independently assert unchanged initial/post Git HEAD; workspace-write
without isolation rejects. The final-prompt test uses valid raw bytes below
32 KiB whose escaping exceeds the required 64 KiB envelope, distinguishing byte
budgets from source length. Original auth/quota, ownership, lifecycle, ACK and
CLI assertions remain in force with otherwise valid native scopes.

The additional postlaunch TST-NEG-018 has a controlled peer actually commit despite read-only mode and return its true new head. It requires EvidenceMismatch, a durable evidence pause, one recorded launch, preserved changed HEAD/file, no accepted implementation/PR intent and no Enable(true), quota-reset Tick or restart bypass. Recovery requires explicit owner inspection, cancellation and newly approved task/scope; it never automatically reverts Git. VER-009 covers before/after audit and owner recovery demonstration.

Expected red proves executable contract reachability and compilation only.
No production scope mutation test, real inference success, actual Windows write
denial, cryptographic service boundary or product acceptance is claimed. The
configuration reference supplies setting names, not proof that the pinned native
sandbox enforces them. VER-009 explicitly requires native key acceptance,
control-file write-denial smoke, alias/reparse/growth checks and independent
approval linkage. Privilege or capability blockers remain acceptance gaps,
never test skips. Existing VER-003–008 remain mandatory live obligations.

## Handoff

Main independently reviews this exact draft, including proposed ADR-003, strict
wire types, helper migration, dependency closure, prompt budget and host/control
trust boundary. Only after approval should main refresh approved metadata and
execution, lock with a NEW externally recorded digest, restore preserved native
implementation and implement the universal gate. The restored old native lib must retain the newly specified Pause::EvidenceMismatch enum variant; reapply it before compilation rather than overwriting the new interface surface. Never use the candidate's own
hash as approval authority. No actual inference before that gate, no paid API,
no deployment, no extra OS scheduler and no private skill copying occurred.

Final main approval:42 locked entries; external digest `99944ec04e65655e9c5fcb1b70a59639aa9f9c85c36147a47c46c93581a92444`, issue4 comment6429242615 and host approved-contracts.json. Final require-lock baseline:0errors0warnings. Refreshed approved-byte runner56expected-red eachCFG1/2, all6gates pass. Implementation and non-test acceptance remain pending.
