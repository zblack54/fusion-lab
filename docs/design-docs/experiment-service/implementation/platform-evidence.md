# Native platform execution

Status: VER-003 initial implementation evidence; acceptance remains NOT_ACCEPTED.

Exact implementation head: `fbddc63cd21b0f3b4975d91cdc42929ada400d9c`.
Approved external contract digest: `3862f6d829e8a739580454f0edfbfb6aff696007286c51dbcfd61bd6f7529f1f`.

[Service workflow run 40241705025](https://github.com/zblack54/fusion-lab/actions/runs/40241705025) and [root workflow run 40241704965](https://github.com/zblack54/fusion-lab/actions/runs/40241704965) completed successfully. The orchestrator inspected every service job's decoded execution log, including real test counts and compiler versions.

| Native runner | Compiler | Locked protocol tests | Supplemental ownership tests |
|---|---|---|---|
| windows-latest | 1.99.0 (b940084d7) | 21 passed, 0 ignored, 36.99 s | 4 passed |
| windows-latest | stable 1.99.1 (58cf8f9c3) | 21 passed, 0 ignored, 38.25 s | 4 passed |
| ubuntu-latest | 1.99.0 (b940084d7) | 21 passed, 0 ignored, 15.41 s | 4 passed |
| ubuntu-latest | stable 1.99.1 (58cf8f9c3) | 21 passed, 0 ignored, 15.43 s | 4 passed |

All four variants also passed adapter formatting, check, Clippy with warnings denied, doctests, rustdoc with warnings denied and harness Clippy. Root numerical formatting/check/Clippy/tests/doctests/docs and CLI smoke passed on all four variants. The strict documentation build passed. Zero-test doctest targets are not counted as behavioral coverage.

These durations are CI execution observations, not performance benchmarks. New production changes require renewed exact-head checks; results do not establish VER-002 aggregate resource bounds or VER-004 injected-failure demonstrations. The subsequent documentation-only head must still pass its own workflow before merge.
