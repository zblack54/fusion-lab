# ADR-001: durable fenced native process ownership
Status: accepted (technical design choice; package approval pending).
Context: REQ-001 through REQ-036 require resumable local work without duplicate
checkout mutation. Memory-only state loses attempts; timeout-only leases can
duplicate a live writer. Alternatives were memory/JSON snapshots, distributed
leases, and a local transactional database plus native file lock.
Decision: use durable SQLite state, stable attempt/effect IDs, safe native file
locking and generation fencing. Reconcile process identity and descendants;
ambiguity pauses. Any Job Object dependency must expose a reviewed safe API;
the design does not permit own unsafe code or assume tree cleanup.
Consequences: state migration/locking/crash-cut evidence is necessary, and
ambiguity can require owner intervention. Windows is the initial application
platform; the numerical core remains unchanged.
Revisit: multi-host execution or demonstrated inability to establish local
ownership. Links: REQ-001–REQ-036, CON-001, CON-003, CON-004, CON-007,
MOD-001–MOD-006, TST-INV-001, TST-NEG-003, TST-ACC-002, TST-STATE-004.
