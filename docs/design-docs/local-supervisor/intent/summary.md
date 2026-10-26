# Intent: local supervisor
Replace manual bounded-work handoffs with a Windows Rust application that durably
claims approved issue tasks, uses the owner's existing ChatGPT subscription,
runs bounded Codex workers, and resumes after crashes without duplicate writers
or remote effects. Issue #4 depends on policy #2 and the read-only probe #3.
The existing goal remains the active orchestrator until an explicit one-owner
handoff; this design starts no recurring process.

Actors are the owner, the durable supervisor, a bounded Codex child, and the
current host chat with authenticated GitHub connector tools. The worker smoke
reported no GitHub connector tools: remote writes are first-class host intents
and ACKs, not presumed native credentials. Unavailable host means durable pause.

Scope includes actual CLI and stdio processes, durable state, checkout locks,
quota pauses/recovery latch, dependencies, limited repair, cancellation,
probe triage, and issue/PR/review/merge evidence. It excludes paid fallback,
deployment, trading, direct credential extraction, parallel writers, and changes
to numerical behavior. Initial application support is native Windows only;
the existing numerical library keeps its Windows/Linux support.

Acceptance uses controlled external processes and files plus implementation-stage
native/GitHub demonstrations. A red executable suite is a design result, not
implementation. The issue remains open until the live integrated demonstration,
restart/cleanup obligations and exact-head host lifecycle are accepted.
