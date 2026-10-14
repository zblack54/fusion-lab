# Development

Use Rust 1.99 or newer. Install a platform-native linker; on Windows use Visual Studio C++ Build Tools. The repository uses stable via `rust-toolchain.toml` and tests the declared floor in CI. No asynchronous runtime is needed for v0.1.

## Checks

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --doc
cargo doc --locked --no-deps
```

Optional task runner:

```sh
cargo install cargo-make
cargo make pr-check
```

`pr-check` checks formatting without changing files. Use `cargo fmt --all` to apply formatting. There are no optional feature combinations to exercise.

## Documentation

Python 3.10+ is required. The helper creates `.venv-docs` and installs `docs/requirements.txt` on first use, using the correct executable layout on Windows or POSIX systems:

```sh
python scripts/docs.py build
python scripts/docs.py serve
```

The build runs MkDocs in strict mode. Static pages go into ignored `site/`. The helper records the dependency file content in the virtualenv and reinstalls when it changes. Open Rust API documentation with `cargo doc --locked --no-deps --open`.

## Agent guidance provenance

The owner's local Rust skill suite was read from `zblack54/copilot-skills` at revision `8f461b9d6e19aa5fbb54ee7bc9d1dccdc55a2771`, including relevant supporting references. `AGENTS.md` routes Codex to `.agents/skills/rust/SKILL.md` when that optional local copy is present. The source repository is private, and its contents are not committed here. Project-specific instructions and all build requirements are available to every contributor.

The source setup agent informed cargo-make and MkDocs configuration. Its Bash virtualenv and `.claude` symlink setup were adapted to a portable Python helper and an optional local Codex skill copy. No submodules or administrator-required Windows symlinks are needed. Update local guidance explicitly from upstream; do not edit it as incidental application work.
