# Current native dashboard evidence

Status: READY_FOR_MAIN_ACCEPTANCE. Local checks and bounded independent source/conformance review are complete; final published-head Windows/Linux × Rust1.99/stable CI and main's acceptance remain required. This file does not self-accept the product.

## Source and commands

source-checkpoint.json hashes every service source/web file and manifests, labels the local tree dirty and records its actual main revision. final-local-summary.json records34passing locked dashboard executions,21passing locked service tests,12supplementary Node regressions and all12required root/service gates. final-local-gates.json retains command stdout/stderr. The final source changes validate a complete bounded run list before DOMmutation; issue60-malformed-list.json preserves the actual failing native-selection regression followed by the repair. No locked surface, dependency, numerical source or budget changed.

Launch from the repository root:

~~~text
cargo build --locked
cargo run --manifest-path crates/experiment-service/Cargo.toml --locked -- --bind 127.0.0.1:8080
~~~

Open http://127.0.0.1:8080/. The service requires an empty artifact root; use a new empty --artifact-root if earlier interrupted data remains. Never delete an unknown directory to force startup. Root/service READMEs and --help identify the local dashboard. Imports remain local browser files; replay controls do not mutate numerical work.

## Recorded values and interaction

browser-fidelity.json plus browser-sample0/1500/2100/7777.json and screenshots cover full35field comparison against actual recorded CSV. A real170624208byte maximum file has360001rows; its exact cursor can inspect row7777 although plots are bounded4096actual samples. Healthy/outage/recovery screenshots remain current for unchanged rendering/data paths. Exact position/velocity values and absences are preserved. XY markers show position observations; velocity is exact values/status only, under main's issue7comment6451759075 clarification. No invented velocity vector scale or calibration claim.

keyboard-narrow-demonstration.json records actual390×844override/client375scroll375, ArrowUp, Play/Pause/Reset and Tab focus. Screenshots show curve distinctions beyond color, textual values, fidelity and delivery/calibration labels.

## Resource proof and scope

browser-continuous-summary.json is the reviewed whole-segment b26 method. Warmup+3imports yield6.8/5.3/4.7/4.7ms maxima and≤651parsedrows betweenyields; numeric100800280bytes. Fiftyselection/reset interactions include49priorLoading;30furtherexactseeks. Loading-only49 p95nextRAFproxy72.3ms/max74.8; supportedmainLongTasks0. NextRAF is a presentation proxy, not physical display proof. Independent review recomputed the dedicated loading sample window, not the final clipped64history.

Owned renderer+DedicatedWorker lifetimeRSS252489728bytes was read via a held process handle,27351samples/158.563seconds, requested5ms spacing/max16msgap, errors[], process exit when ownedtab closes. Existingblank/spare PID50516 gained DedicatedWorker59728 after navigation; there is no absent-before/newPID claim. Shared browser/GPU processes are not charged as selected-tab memory. Windows reference evidence is not an OS hardcap or Linux browser measurement. BrowserUA Chrome156/Win10x64 is in reconnect-demonstration.json; reference-host.json records hardware after workload, not runtime memory pressure.

Raw full process inventories were preserved privately; public scoped views explicitly retain original SHA provenance and owned rows only. Earlier snippet-only4.9ms files and screenshot are historical, cannot prove whole-worker8ms and were superseded without relaxing a threshold. The realWorker12ms notification regression detects the original missed-work defect. The final malformed-list validation/error/help changes do not alter measured parser/replay/render hot paths; this is source-equivalence evidence, not a falsely labelled new binary benchmark.

## Failure demonstrations

Actual busy409, Cancel→cancelled with childlistenerclosed, retention1eviction/worker_failed and32partial-requestoverload→503 are recorded in eviction/overload JSON and screenshots. Controlled worker cases demonstrate UIlifecycle/errors, not numerical accuracy. reconnect-demonstration.json records4SSEopens spaced1.057/2.051/4.051seconds, followed by manual recovery and no fifthattempt.

Bounded local fault proxy accepted one POST then dropped its reply. Before repair GETlistcount1(initialonly); after repair GETlistcount2(initial+recovery) with POST1, persistent uncertainty and retained localCSVcursor1500. Failed recoveryGET503 remains manual and explicit. Fixtures translate Origin only for finite local tests, never production/security behavior. New malformed200list adapter regression uses native replaceChildren reset semantics and proves complete validation before mutation. No automaticPOSTretry or inferred matching run.

The previous native-evidence-before-final.md and raw JSON retain intermediate failures/results. acceptance.md is the current per-obligation table; final acceptance belongs to main after exact-head CI.
