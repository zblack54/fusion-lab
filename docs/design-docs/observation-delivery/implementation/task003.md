# TASK003: bounded canonical artifacts

Status: implemented; independent codec/VER002 review and trusted implementation receipt pending. Only src/delivery.rs changed production; external baselinee0d3a23f4335193f6442fa7d1b244621b93bbf3f08ee9ba9c4b860c2eb15911b is unchanged. Source SHA256 12d7fbdcee02d7ae986658bb2f43116f56283f1689adc2c74e4f360d45389120.

Trace and ledger encode one bounded record at a time, checking the next append before extending the output. Validated integer and finiteDisplay domains bound each temporary line below1024bytes; no oversized whole artifact is materialized before its32MiB/16MiB check. A validfinite huge trace may return ArtifactTooLarge; no partial buffer escapes.

Decoder borrows input, rejects bytecap before UTF8 and uses explicit BOM/CR/nonASCII/finalLF ordering. It scans bounded line/token shapes before contextual allocation, validates duration/model/capacity/declaredcount and exactremainingrowcount before reserving observation storage. Scalar parsers reject noncanonical decimals/Display spellings while retaining typed nonfinite observation errors. Every observation passes domain validation. Full-inventory duplicate validation precedes reporting a stored original-order inversion; successful import never silently repairs order.

Exact43selectors passed on each localalias (86/86), including all referencebits, signedzero/extremefloats, analyticalnumericalfailures, caps, codecs, queueproperties and fresh-current-root privacy proof. All6strictrootgates pass; require-lock0errors0warnings. Bothaliases currently1.99.0. Raw exactchecks are task003-tests.json and task003-root-gates.json. Official trusted implementation-mode verification uses the actual externalbaseline and records implementation/execution.json.

No locked test/catalog/fixture/API/requirement, old numerical/sensor/CLI/dependency or accepted service/dashboard contract changed. TASK004’s example, measured native resources, four-platform implementation proof and final conformance remain incomplete; no feature acceptance or parent6 completion is claimed.

Trusted implementation verification completed successfully: all86tests pass, all7supplemental gates pass, errors0, externally supplied approved digest verified. Raw executionSHA256 `11957fb08c504743a5939120742a3f259a987f6be7e1bc0545b9da3ca6d75eff` (989268bytes). This local result still does not fulfill TASK004 native/resource/example obligations.
