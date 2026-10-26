# ADR003: hash-bound approved scope and isolated host control

Status: proposed; independent main approval and revised external lock pending.

## Context
Task.scope_digest alone does not tell a native worker what to implement. If the
worker can write state/ACK/scope control, it can accidentally forge the authority
that is supposed to review it. Issue41 requires actual bounded approved content
and control separation before unattended inference.

## Options
1. Forward only the digest/issue and assume remote issue tools or prompt policy.
   Rejected: content can be unresolved, and writeable ACKs defeat host review.
2. Let worker fetch/approve its own mutable scope. Rejected: authority becomes
   circular and cannot be tied to an independently reviewed exact payload.
3. Materialize private versioned bytes, bind SHA256 to task/issue/checkout, and
   enforce mode/control isolation before any native effect. Proposed choice.

## Consequences
The host has explicit approval work; both lock closures gain pinned SHA2. Native
fixtures and CLI checkout configuration change while preserving old assertions.
Raw32KiB and encoded64KiB caps bound allocation/dispatch. Missing, changed or
wrong-identity scopes fail closed; no test-peer exemption is permitted. Temp and
extra write allowances must be excluded, with actual Windows denial evidence.
The trust boundary is a private authorized host, not a malicious OS owner.

## Revisit
Revisit on scope wire version, sandbox capability, checkout aliases, approval
store ownership, or hash/dependency change. Such changes require renewed design
review and lock; this ADR grants no implementation or deployment approval.

Covers REQ037–044, CON008–010, MOD007, TASK005, VER009 and tests
TST-NEG-013–017, TST-BOUND-005–006, TST-ANTI-003–005, TST-ACC-009–010,
plus strengthened TST-CONTRACT-005 and TST-ACC-008.
