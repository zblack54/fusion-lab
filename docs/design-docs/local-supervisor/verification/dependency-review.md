# Preapproval dependency review

Status: reviewed by design author; independent package rereview pending.
This closes the dependency gap before locking; it implements no supervisor behavior.

| Direct dependency | Exact version/features | Purpose | Declared Rust floor | License |
|---|---|---|---|---|
| rusqlite | =0.40.1, default-features=false, bundled | Safe Rust SQLite transactions; build included SQLite rather than use ambient system SQLite | Not declared in this release's published metadata; actual 1.99.0 compilation is required | MIT |
| serde | =1.0.228, default std plus derive | Typed bounded persisted/wire records | 1.56 (derive 1.61) | MIT OR Apache-2.0 |
| serde_json | =1.0.149, default std | JSONL and host bridge protocol | 1.68 | MIT OR Apache-2.0 |
| getrandom | =0.3.4, std | OS entropy for random owner tokens, without custom unsafe code | 1.63 | MIT OR Apache-2.0 |

Primary release manifests were inspected, not inferred from latest branches:
[rusqlite v0.40.1](https://github.com/rusqlite/rusqlite/blob/v0.40.1/Cargo.toml),
[serde v1.0.228](https://github.com/serde-rs/serde/blob/v1.0.228/serde/Cargo.toml),
[serde_json v1.0.149](https://github.com/serde-rs/json/blob/v1.0.149/Cargo.toml),
[getrandom v0.3.4](https://github.com/rust-random/getrandom/blob/v0.3.4/Cargo.toml).
cargo info for each exact release agreed with the license/declared-floor values.
No latest-version claim is needed for these deliberately pinned versions.

Production Cargo.lock and test-owned harness Cargo.lock were regenerated together.
The generated dependency-inventory.json records all 28 registry packages, versions,
license expressions, declared floors and both lockfile SHA-256 values. All declared
floors are <=1.99; some packages (including rusqlite/libsqlite3-sys) do not declare
one, so successful native MSRV compilation establishes this project's supported
configuration rather than an invented upstream MSRV. Platform-conditional WASI/EFI
packages appear in the full lock closure even though Windows builds do not use them.

Licenses include MIT, Apache-2.0 alternatives, Unlicense alternatives, and
unicode-ident's additional Unicode-3.0 condition. The complete expressions are
preserved in the inventory; the EFI LGPL term is one of multiple license options,
not a required choice for the Windows application. Bundled libsqlite3-sys 0.38.2
has MIT metadata; its SQLite source header explicitly disclaims copyright.
Keep dependency/license notices when distributing. This is a factual inventory
review, not a claim of a complete security or legal audit.

The bundled feature adds a C build through cc/MSVC and fixes the selected SQLite
source via the registry checksum. Native cargo check --locked --all-targets compiled
the bundled source and both crates successfully. The final trusted design runner
also compiles with 1.99.0 and stable and must pass after these dependency changes.
No build-time bindgen, extension loading, SQLCipher, network runtime, external
provider SDK, async runtime or Job Object wrapper is enabled. Own source keeps
forbid(unsafe_code); dependencies may contain reviewed FFI/unsafe internals.

A safe process-tree wrapper is not silently deferred as a dependency addition:
the currently approved alternative is conservative descendant ambiguity pause.
Adding a wrapper later that changes the locked harness dependency closure needs
a controlled design revision before modifying that lock.

Reproduce inventory:
python docs/design-docs/local-supervisor/verification/review_dependencies.py
--output docs/design-docs/local-supervisor/verification/dependency-inventory.json
(run as one argv command from repo root). The script uses cargo metadata --locked
--offline, omits credential/host identity data, rejects declared floors >1.99 and
missing license metadata. It does not prove unsupported target compilation or
absence of vulnerabilities. Initial offline inventory failed because target-only
libc/WASI/EFI crates were uncached; cargo fetch --locked filled the exact closure,
then the offline inventory succeeded. No manifest or lock outside owned roots
was changed.

