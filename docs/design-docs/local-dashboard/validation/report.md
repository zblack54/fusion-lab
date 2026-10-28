# Validation report
Status: AI_READY

## Scope and authority

Formal draft targets actual merged service baseline14880d627b5584a98f34fe374e3f79e74e64f80b. Main subsequently merged documentation-only changes; numerical/service production behavior at this baseline is unchanged. Actual issue #7 was read before drafting. Scope is the smallest same-origin local Rust/static HTML/CSS/native-JS dashboard, recorded replay and bounded streaming parser; no cloud/Sites/CDN/npm runtime, paid API or numerical implementation changes.

All P1–P7 technical design records are complete for independent review:2acceptance scenarios,20requirements,3planned ownership modules,8constraints,17cataloged tests,6non-test verification plans,4dependency-ordered slices,2proposed ADRs, zero unresolved product questions or granted waivers. All11test classes are represented: acceptance2,behavioral1,boundary2,negative3,property1,state-transition1,contract3,performance1,anti-pattern1,structural1,invariant1.

Main explicitly authorized only `crates/experiment-service/web/data.mjs` as a diagnostic design seam. It exports named boundaries throwing LOCAL_DASHBOARD_NOT_IMPLEMENTED and implements no UI/parser/replay behavior. Existing service source/API/lock/tests remain untouched. Main /root independently approves status=approved,ai_ready=true under owner delegation and AGENTS.md. Independent reviewer cleared semantic oracles and main corroborated arithmetic; final-byte execution and externally published lock remain required before implementation.

## Executable baseline

Native Windows x86_64-msvc, stable Rust1.99.0, Node24.22.1. `cargo generate-lockfile --manifest-path tests/local-dashboard/Cargo.toml` retained the concrete57package service dependency closure before execution. Root numerical CLI inputs and existing service are classified production dependencies; all Rust/Node test sources and expected values are inside the dedicated test harness. No external test-support crate, NPM dependency, generated input or machine-specific Cargo override is needed.

Commands actually run from repository root:

* `cargo fmt --manifest-path tests/local-dashboard/Cargo.toml`: passed.
* `cargo clippy --manifest-path tests/local-dashboard/Cargo.toml --locked --all-targets -- -D warnings`: passed.
* `node --check crates/experiment-service/web/data.mjs`: passed.
* `node --check tests/local-dashboard/node/data.test.mjs`: passed.
* Trusted `validate_design_package.py docs/design-docs/local-dashboard`: VALID,zero errors, intentional unlocked warning.
* Trusted `verify_test_suite.py docs/design-docs/local-dashboard --mode design`: success for CFG-001 stable and CFG-002 Rust1.99.0, each enumerating17tests,16expected-red plus1passing characterization (34executions total:32expected-red,2passes), zero unexpected failures. Supplemental Clippy and both Node syntax commands passed. Final exact-byte report is validation/design-execution.json.

Fifteen Node-backed tests execute one selected built-in TAP test each through a Rust libtest wrapper. The wrapper checks Node>=24, process exit, actual `# pass 1` and `# fail 0`; missing/ignored/zero-match execution cannot silently pass. Their red failures reach the named diagnostic module boundary and remain normal one-test libtest failures. Static-route acceptance uses the real service's actual404 and the diagnostic `dashboard route unavailable`; no fake server or static markup implementation was created. The green real-HTTP characterization builds the real numerical CLI, submits fullu64max seed, verifies101samples/35columns and independently known final truth.

An initial catalog classification incorrectly expected the JS diagnostic on the static-route Rust test. The real run failed for the intended404with a different message; its catalog was corrected to the genuine HTTP failure before final verification. No required behavior or assertion was weakened. No compile failure or missing module was misreported as expected red.

## Test efficacy and unmeasured obligations

Synthetic CSV values are authored independently: known3/4,6/8,5/12errors produce5/10/13; covariance[[5,4],[4,5]] has eigenvalues9/1and angleπ/4. Byte partitions include1byte UTF-8-BOM fragments and CRLF. Tests cover all row values, absence versus zero, partial pairs, malformed schema/numbers/booleans, invalid UTF-8, incomplete records, hard row/byte/line and alternate-construction caps, snapshot isolation and incremental consumption before finish. Replay tests use injected times and explicit0.25/1/4speed cases. Full unsigned64bit seed/sequence and valid-context duplicate JSON keys are tested without Number rounding.

The maximum-row test generates360001independent rows incrementally and checks actual Node RSS against a512MiB reference budget plus declared numeric allocation cap. It has not passed implementation behavior yet—the diagnostic seam is red. It does not pretend its shorter synthetic rows are the actual170MB artifact; VER-003 separately requires the real maximum artifact in a real browser, repeated abort/selection ownership, frame/yield and memory measurements. The actual HTTP->fetch stream->parser integration is separately cataloged.

Potential ceremonial tests were avoided: no source-string layout assertions, no fake numerical successful output, no self-generated expected ellipse, no nonexistent browser screenshot claim. Test review strengthened speed behavior, valid-context duplicate keys, capped alternate constructors and buffer reuse/progress observation before declaring this package ready. No production mutation score is claimed while implementation is absent.

VER-001 exact-code architectural review, VER-002 real browser interactions/fidelity, VER-003 real browser resource/responsiveness, VER-004 current native OS/toolchain/service compatibility and VER-005 actual launch/screenshots/walkthrough remain implementation acceptance plans. VER-006 is reviewed design arithmetic only; its storage argument is not a measured browser result. Initial page/worker launch uses the local service; local CSV import needs no running experiment and no upload, with no unpromised offline-cache feature.

## Handoff and checkpoint

Final independent review aligned spec.target.toolchains and validation configurations with CON-008/VER-004: both stable and1.99.0 now execute locally on Windows. The verified-empty optional harness benches/fixtures/compile directories were removed before the final refresh; Git preservation no longer relies on empty directories. Linux remains future implementation acceptance evidence, not a design execution claim.

Independent prelock review found two gaps, repaired while still draft. REQ-009 and architecture now require finite integer maxPoints>=2, reject0/1/NaN/fractional/infinite limits with RangeError, clamp valid larger values to4096, and define singleton/empty results. TST-STRUCT-001 preserves prior assertions and adds0/1/2, singleton and4097clamp cases. TST-ACC-001 now additionally deep-compares an independently authored literal35column row against the entire expected object, including distinct tick/time, truth, estimates, explicit presentzero observation pairs, both baselines, outage and all16distinct covariance values. TST-INV-001 directly contrasts both null measurements with present[0,0]pairs. Existing selectors/counts remain17tests:16expected-red and1green; final actual refresh is recorded in design-execution.json. No behavior was implemented and no approval or lock was issued.

No test lock, dashboard approval or AI_READY claim. The next action is main's independent review of exact contracts/tests/dependency inventory, particularly browser budgets, u64 token parsing and required real-browser evidence. After authorized approval only, update metadata/ADRs, refresh execution evidence, lock and publish the digest externally. Implementation then follows the four approved slices and final per-obligation conformance audit. All service approved artifacts remain intact.

Final main approval:39 locked entries; external digest `d33ff5c8c1828d503d0cecfb144df72ab96621fa0f2e8dd8096c7f98ba0ddd95`, issue7 comment6429218937 and host approved-contracts.json. Final require-lock plus external baseline:VALID,0errors,0warnings. Refreshed approved-byte runner34executions:16expected-red+1pass per configuration; three supplemental gates pass. Product acceptance remains pending.
