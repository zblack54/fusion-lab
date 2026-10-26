# Preapproval dependency review (current revision2 closure:46 registry packages)

Status: reviewed by design author; independent package rereview pending.
The chronological sections below preserve original28-package and revision1
38-package reviews; the final revision2 section/inventory is authoritative for
the current46-package candidate.
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


## Revision1 (issue #37), main + independent reviewer source audit

Current closure has38 registry packages. Adds Windows-only direct subc-jobobject=0.2.2 (MIT; no declared MSRV), plus its unconditional Tokio-process and Windows FFI closure; all28 prior registry versions are unchanged. Tokio1.54.1 declares1.85, windows-sys0.61.2 declares1.71; full inventory preserves every license/floor and both lock hashes. Native1.99 compilation and final expected-red runner establish supported configuration.

Reviewed exact registry source lib.rs/sys.rs/cleanup.rs. Root job is static, noninheritable, kill-on-close and installed only at standalone driver entry before any children. Per-worker safe primitives retain Child+Job across suspended spawn, assignment and resume, then use kernel membership count for proof. Manual retained ownership avoids convenience setup error paths losing handles or using unbounded wait. PID liveness helpers are excluded as proof. Unknown post-crash ownership stays fenced; unnamed job reopen is unavailable. Native setup/crash/grandchild evidence remains VER004.

Rejected alternative process-wrap10.0.1 after source inspection: its std Job Object uses kill_on_drop=false, and completion-port wait helper does not establish ACTIVE_PROCESS_ZERO; direct-root try_wait is insufficient. No dependency on that crate is added. This is a focused source/closure review, not a vulnerability audit or unexecuted native-proof claim. Own unsafe remains forbidden.

## Controlled revision2 hash closure (issue41)

Both production and independent harness fixtures pin sha2=0.10.9 with default
std features. Native cargo info inspected the exact downloaded release: license
MIT OR Apache-2.0; no rust-version is declared, so no upstream MSRV is invented.
The exact normalized release manifest and source in the registry cache were
reviewed. Primary release source: [sha2 0.10.9 manifest](https://docs.rs/crate/sha2/0.10.9/source/Cargo.toml).
This is the RustCrypto SHA256 implementation, not handwritten application crypto.
Optional asm/sha2-asm/force-soft/oid features are not enabled. CPU feature
detection is supplied by the reviewed transitive crate, not own unsafe code.

The eight added registry packages are sha2 0.10.9, digest 0.10.7, block-buffer
0.10.4, crypto-common 0.1.7, generic-array 0.14.7, cpufeatures 0.2.17, typenum
1.20.1 and version_check 0.9.5. All38 prior registry names/versions remain
unchanged; the full reviewed closure is now46. Complete license expressions and
declared floors are preserved in dependency-inventory.json. New licenses are
MIT and MIT/Apache alternatives; typenum declares1.41, other newly added releases
do not all declare a floor. Actual native1.99.0/stable builds establish project
compatibility. No declared floor exceeds1.99 and no license metadata is absent.

Harness direct serde_json=1.0.149 is the already reviewed production version; it
allows explicit independent fixture JSON rather than asking the production gate
to author its own expected input. SHA256 fixture helper checks the fixed known
abc vector, then hashes exact scope file bytes independently of the supervisor.
Offline resolution initially reported missing cpufeatures index input; the
normal verified Cargo fetch filled the exact closure, with no TLS bypass or
API/provider secret use. Both refreshed locks and their SHA256 fingerprints
are inventoried before preapproval verification. No root manifest/core changed.
