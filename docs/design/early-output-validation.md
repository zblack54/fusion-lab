# Early CLI output validation — issue #9

The CLI SHALL reject an output extension other than lowercase .csv before entering the numerical run. This avoids work for an unusable destination. When duration and extension are both invalid, extension validation takes precedence. Valid-output behavior, library APIs, dependencies, MSRV, and numerical contracts remain unchanged. Filesystem permissions and destination creation remain handled by existing I/O code; this change does not promise atomic output.

Verification: tests/cli.rs invokes the actual Cargo-built CLI with invalid duration and extension; the output-extension error must win over run's configuration validation. Review src/main.rs to confirm validation precedes run and filesystem mutation. Preserve existing tests and all AGENTS.md gates. No test lock is required for this bounded change.

Authority: owner authorized autonomous local development in this chat on 2026-10-25, with issue-first tracking. Design accepted within that delegation. Implementation may proceed; final acceptance requires recorded execution evidence.
