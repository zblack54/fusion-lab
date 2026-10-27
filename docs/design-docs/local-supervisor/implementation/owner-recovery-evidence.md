# Controlled owner-recovery demonstration

NOT_ACCEPTED. This demonstration uses the existing test-owned fake-codex external protocol peer, not actual subscription inference or native sandbox write isolation. Raw commands/status/audit retained outside repo at outputs/supervisor-scope-smoke/controlled-recovery.json. Production source and locked contracts are unchanged.

Fresh disposable initial HEAD a56fa20ef17f3f6fc44f8fe547b908d91c706811. Approved read_only scope41f0d36cf33803469f07900aa1c04a2094894fc68fc2a8afc226ef13120eeafa deliberately ran the controlled readonly_mutates scenario. Peer actually committed fec5396c26880f8512a65a403cc724294dadbbce and returned its true head. Driver returned EvidenceMismatch, durable evidence_mismatch pause, active null, one launch, changed head visible. A separate CLI process enable and another step both rejected, launch count stayed1 and actual changed HEAD remained intact.

Explicit owner inspection observed the preserved Git mutation and row head. CLI cancel task1 retained cancelled task/head and cleared pause without reverting Git. Owner materialized a distinct scope d724f38bd94bdc8d78970a353f0cd0d3f4e678c403695a8a1e2cc45891c7d7aa for new task2, then controlled normal dispatch succeeded with launch count2. Old task remained cancelled, new task implemented_unverified, only synthetic local PR intent pending. Durable read_only_mutation_rejected audit retains original/changed heads and session; marker absent after cleanup. No GitHub intent was acted upon.

This proves the tested durable owner cancellation/new-scope recovery path while preserving evidence. It does not resolve the separate real worker Git index.lock denial, file-symlink privilege gap, or remaining production acceptance obligations.
