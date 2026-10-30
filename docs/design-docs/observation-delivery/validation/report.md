# Validation report

Status: NOT_READY

## Controlled revision2 draft — issue72

Main authorized draft preparation only in issue72comment6527480873 after author and independent unchanged-core probes reproduced two exact-rational versus floating roundoff conflicts. TST-BEH-004 position[5,10] and TST-STATE-001 velocity[2,1] retain their independent expected values but now use existing componentwise close with1e-12 tolerance. All reference-target bitwise equality, signedzero/copy/prior assertions, other tests, requirements, API and numerical/resource bounds remain unchanged.

The complete original567approved package/lock and partialTASK002sourcee262afab are preserved in protected host history; original failure logs remain unchanged. Diagnostic source is restored byte-for-byte from the approved snapshot. Registry remains567; P5/P7 are pending refreshed diagnostic verification and independent review. No revision2 approval, lock or implementation permission exists.

## Historical revision1 report (superseded readiness only)

# Validation report

Historical status: AI_READY

All seven phase exit gates were completed and Status: READY_FOR_REVIEW was recorded while draft/ai_ready=false before applying main’s established approval. Main approved technical design revision1 in [issue69comment6526992382](https://github.com/zblack54/fusion-lab/issues/69#issuecomment-6526992382), after independent review and exact-head42085624 normal CI and all four native artifact reviews. Metadata now records approved/ai_ready=true. Main alone performs the immediate lock and external digest publication; implementation remains prohibited until that verification. AI_READY describes approved design readiness, not implemented product acceptance.

## Final design evidence

Catalog counts remain6acceptance scenarios,37requirements,8constraints,4modules,43selectors,6verification plans,7supplemental gates and4ordered tasks; no waivers or class dispositions. VER001/002 design arguments and the43-selector oracle/privacy quality review are independently confirmed. Their implementation-stage inspections and VER003–005 measurements/demonstrations remain mandatory and unperformed. All four hosted native combinations have41expected-red+2structural-green with source/rawlog hashes independently verified; see verification/native-matrix.md. Strict Windows trusted P7 is refreshed against final metadata below. Original failures remain preserved. Parent6/22/8 completion is not claimed.

## Preserved failures and repairs before approval

The initial draft compile found one extra closing delimiter; it was repaired before execution. A first privacy checker accepted type-name mentions too loosely; tightening it exposed qualifiedRusttype names, then the checker was corrected to require per-typeE0451messages, supporting qualifiednames. The first officialP7run had all86intended test outcomes and all6legacygates correct but failed harnessClippy on option_env-unwrapping and a longfunction-pointer annotation. Those test-only lints were fixed with env! provenance and a typealias, not suppressed or weakened. The first failed P7report is preserved outside thecandidate. The fresh report succeeds for every local executable obligation.

Actual RustDisplay probing found negative-smallest-subnormal327bytes and the earlier combined3-model-token header1053bytes. Main approved three fixed model lines before formalization, preserving1024linecap. The unchanged-estimator finiteMAXprobe confirmed successful finite tick0updates followed by typed tick1prediction failure with state preserved. Independently derived scalar/cross-covariance, queue/drain/bypass, scheduled-loss-gap, horizon,127arrivalburst and source-poisoning oracles are reflected in the harness; this expected-red stage does not prove their future implementation.

## Publication transport

The public draft retains the complete raw validation/design-execution.json at its canonical path. A lossless gzip may be supplementary transport only and never replaces or omits that raw normative evidence. If a gzip is used for transport, decompress byte-for-byte and compare its uncompressed SHA256 before trusted checker reuse, final approval or locking. This is not a waiver and does not make approved design evidence mutable: its hash belongs to the eventual approved lock. Only implementation/execution.json is mutable implementation evidence.

## Preapproval suite quality repairs

Independent review identified and repaired three efficacy gaps before approval: the per-sensor loss grid now uses asymmetric None/N1/N3/N4pairs with exact expected identity sets; the privacy proof now compiles a fresh locked/offline consumer of the CURRENT root source (positive then exact eightE0451negative), never selecting cachedrlibs; and early task exit selectors are stage-local with all deferred integration required in TASK003. The fresh-consumer first attempt exposed Windowsverbatim canonicalpath syntax in Cargo TOML; normalizing only that prefix preserves current source identity. Structural compiler proof receives240sexternalmetadata budget, distinct from the30sproduction workload budget. These changes leave all43final selectors required.

The fresh-consumer proof also exposed a real Windows fixture-copy mtime issue: after positive compilation, fs::copy preserved an older negative-fixture timestamp, so Cargo returned an incorrect stale positive success. The strengthened test failed closed; writing the negative fixture bytes with a current timestamp forces the same fresh consumer to be rechecked. This is a preapproval test-infrastructure repair, not a cached-rlib fallback or weakened negative assertion. Both exact commands and positive/negative diagnostics are preserved in the new raw structural proof.

## Strict rustdoc CI repair before approval

Draft PR71head27d17f73e4f10c04778315500bd967c7a4903536 passed all four native design baselines (run40955812598), but ordinary rootCI40955812478 failed Linuxstablejob123735503068: the unquoted covariance bracket text at src/delivery.rs:211 was treated as a broken intra-doc link under RUSTDOCFLAGS=-Dwarnings. Other fail-fast rootjobs were cancelled, not accepted or bypassed. Earlier local CMDdoc checks did not set this strict profile and therefore did not prove CI parity.

The repair changes only rustdoc markup to inline-code brackets, with no suppression or behavior change. CMD004/005/006now carry portable Cargo build.rustdocflags=[-D,warnings]arguments; the actual trusted verifier and public native driver also force RUSTDOCFLAGS=-Dwarnings. Direct strict Cargo flags reproduced the original broken-link failure before markup repair. Cargo env-table injection alone was observed not to enforce Cargo rustdoc flags and was not adopted. No shell, new helper or dependency is required. CON006now explicitly refers to production runtime/crate/dependencies, allowing the deliberate test-owned consumer crates. Decoder wording now orders UTF8validation, leadingBOM/anyCR rejection, genericnonASCIIrejection and finalLF/line/token validation explicitly. All43selectors and numerical/resource contracts remain unchanged.

Status remainsNOT_READY until the repaired exact head has green native and ordinary CI plus main final review; no phaseapproval/lock/implementation follows from these documentation/profile repairs.

## Approval checkpoint supersedes earlier draft readiness statements

Earlier NOT_READY statements above describe preserved preapproval checkpoints. Final independent review and exact-head CI resolved those design-stage blockers. No numerical/resource requirement, test selector, API behavior or production implementation changed during approval metadata application. Main alone adds the verified lock digest and final locked validator result here before handoff.

## Final approved-byte strict P7 refresh

The final metadata refresh succeeded: each local configuration41expected-red+2structural-green, CMD001–007allpass, errors0, with strict rustdoc. Canonical raw evidence SHA256 `d64054e53bfcd275f2ebc355ed2beee262c92267b6b498019a2c29d74ae1fc2e` (1026501bytes); contract digest `567eff28d5ec5b6af564284cb0f53292fbfe30b1b7292b9703be4d7b1838c3b9`. Strict documentation build passed. Main must now create/verify the lock and publish its external digest before implementation.

## Main trusted lock and external digest

Main locked55entries with approved digest `567eff28d5ec5b6af564284cb0f53292fbfe30b1b7292b9703be4d7b1838c3b9`. Final trusted --require-lock validation:0errors/0warnings. External authority: issue69comment6527067252. Finalrawdesignexecution1026501bytes/SHA256d64054e53bfcd275f2ebc355ed2beee262c92267b6b498019a2c29d74ae1fc2e. This report is excluded from the lock. Scoped implementation may now begin; product remainsNOT_ACCEPTED until all final obligations pass.

## Revision2 draft diagnostic refresh

Strict trusted diagnostic P7 passed: each local1.99.0/stable alias41named expected-red+2structural-green, all7supplemental gates pass, errors0. Strict docs build passed. Raw design-executionSHA256 `991075207264b0419ba7c58efcccb901c61ca735bf4b1f26014fa39d29730a2f` (1027321bytes). These are diagnostic baseline results, not successful replay implementation or revision2 approval. Independent/main review and any required hosted requalification remain pending; registry still authorizes only the prior567contract.
