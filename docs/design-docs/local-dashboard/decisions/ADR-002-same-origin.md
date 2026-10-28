# ADR-002: Same-origin native adapter
Status: proposed

Context: local-only inspection, honest fidelity and bounded memory are required; the service already owns its origin and artifact lifecycle.
Options: Separate development proxy/CORS exceptions, framework/devserver, or static files embedded by the existing Rust service.
Decision: Choose existing-service static assets and native browser modules/worker, with no CDN/npm runtime and no numerical modification.
Positive: small runtime and explicit ownership. Negative: full recording still costs about96MiB and browser evidence is required; extending existing known routes needs integration review. Revisit if a separately approved live-observation protocol or measured resource requirement changes.
Affected: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012, REQ-013, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-020; CON-001, CON-002, CON-003, CON-004, CON-005, CON-006, CON-007, CON-008; MOD-001, MOD-002, MOD-003; TST-ACC-001, TST-BEH-001, TST-STATE-001, TST-CONTRACT-001, TST-NEG-001, TST-BOUND-001, TST-PROP-001, TST-INV-001, TST-STRUCT-001, TST-PERF-001, TST-ANTI-001, TST-NEG-002, TST-BOUND-002, TST-NEG-003, TST-CONTRACT-003, TST-ACC-002, TST-CONTRACT-002.
