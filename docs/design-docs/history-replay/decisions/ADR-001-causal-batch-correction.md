# ADR-001: causal batch rollback

Status: accepted by main design approval issue22comment6637007196; product implementation acceptance remains separate.

Context: late observations are currently explicit stale drops. Alternatives: silently update at arrival time (changes measurement meaning); unbounded noncausal full replay (cost/knowledge violation); or bounded sample-time rollback of currently known observations.

Decision: choose a pure bounded candidate transaction, one canonical batch per tick, unchanged online output prefix, shared metadata-only transport. Main API selection is issue22comment6636494428; final executable contract review remains pending. Positive consequence: causal interpretation and constructive per-tick limits. Negative consequence: correction can cost501predictions/127updates and retain original+candidate scratch. Revisit if window>500, generalized input schedules, true streaming adapters or retrospective output is requested.

Links: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012, REQ-013, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-020, REQ-021, REQ-022, REQ-023, REQ-024, REQ-025, REQ-026, CON-001, CON-002, CON-003, CON-004, CON-005, CON-006, MOD-001, TST-ACC-001, TST-ACC-002, TST-ACC-003, TST-BEH-001, TST-BOUND-001, TST-NEG-001, TST-NEG-002, TST-BOUND-002, TST-BOUND-003, TST-BOUND-004, TST-BOUND-005, TST-BOUND-006, TST-BEH-002, TST-PROP-001, TST-PERF-001, TST-PERF-002, TST-BEH-003, TST-BEH-004, TST-PROP-002, TST-PROP-003, TST-CONTRACT-001, TST-CONTRACT-002, TST-CONTRACT-003, TST-INV-001, TST-BEH-005, TST-STATE-001, TST-STATE-002, TST-NEG-003, TST-PROP-004, TST-BEH-006, TST-STRUCT-001, TST-ANTI-001.
