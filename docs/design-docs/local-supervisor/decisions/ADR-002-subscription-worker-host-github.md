# ADR-002: subscription worker with explicit host GitHub bridge
Status: accepted (technical design choice; package approval pending).
Context: existing ChatGPT sign-in works, account/quota reads and a read-only CLI
smoke succeeded, but the standalone worker reported no GitHub connector tools.
Alternatives were API credentials/PAT provisioning, pretending connector tools
are inherited, and explicit file intents to the already-authorized host chat.
Decision: use owned stdio app-server for managed account/quota reads and
subscription-only bounded CLI workers; route GitHub actions as durable typed
intents to the host. Host performs read-only reconciliation before uncertain
writes and returns exact-head evidence. Never extract secrets or silently fall
back to paid APIs. Preserve owner quota reset cadence and weekly latch.
Consequences: no new login is required, but the host can be unavailable and
native unattended GitHub progression is not claimed. Final acceptance requires
the integrated host demonstration, including lost replies and restart.
Revisit: an independently authorized native GitHub adapter or app-server
compatibility change. Links: REQ-001–REQ-036, CON-002, CON-005, CON-006,
MOD-001–MOD-006, TST-CONTRACT-005, TST-NEG-008, TST-INV-002, TST-ACC-007,
TST-ACC-008.
