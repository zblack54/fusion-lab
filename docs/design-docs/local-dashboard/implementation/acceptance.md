# Final Conformance Review

Status: NOT_ACCEPTED

Timing review correction: original browser-maximum-loads.json preserves a4.9ms maximum of measured snippets, not continuous whole-worker work. It cannot establish REQ019's8ms work obligation because notification/bookkeeping gaps were unmeasured. RSS and control timings remain independently corroborated. New worker b26f8554 uses continuous active segments excluding only actual suspended awaits; a genuineWorker regression injects12ms into final notification and detects it. Its full native workload and exact-source gates must be refreshed. No threshold was relaxed; previous timing is retained as incomplete evidence.

Approved digest:d33ff5c8c1828d503d0cecfb144df72ab96621fa0f2e8dd8096c7f98ba0ddd95. Exact local source hashes:source-checkpoint.json; published parent PR44f6eb4a83ac468584921cc6646847ae5f602c223b, local issue57 source still requires publication/CI. Native Windows34locked executions and8supplementary regressions pass after57. No lock/test/dependency/numerical-core changes.

| Obligations | Actual evidence | Remaining acceptance work |
|---|---|---|
| REQ001 | Real embedded route/MIME/CSP HTTP acceptance passes | Final independent static-route/security audit |
| REQ002,014 | Real create/list/status/terminal SSE observed;47/52/53 repaired | Final-source cancellation, busy/overload/eviction/reconnect/uncertainPOST browser scenarios |
| REQ003–006,011 | Immutable parser/wire tests; realHTTPfetch test; all35fields compared at0/1500/2100/7777 in browser-fidelity.json | Final source audit; no outstanding numerical test failure |
| REQ007 | Independent5/10/13errors, known ellipse;49–51supplementary regressions | Reviewer final conformance; covariance remains descriptive |
| REQ008–009 | Locked replay/bounds tests,30real exact seeks,7777nonplotted browser sample | Final broader keyboard/playback walkthrough |
| REQ010,013,019,CON003,007 | Warmup+3maximumimports,50resets/49whileloading, boundednumeric100800280, plot655360; ownedrenderer lifetimeRSS253980672; worker4.9ms/691rows; supportedLongTasks0 | Independent resource-method review; render cadence source audit. Input→nextRAF p95proxy69.7ms is explicitly not physical display proof |
| REQ012,018,CON004 | Actual healthy/outage/recovery screenshots; exact sample comparison and explicit precision/delivery/downsample labels | Final-source visual/accessibility audit |
| REQ015 | Labeled native controls, keyboardArrowUp and exact seeks observed | Narrowviewport, complete keyboard/focus walkthrough |
| REQ016 | Real local170624208byte CSV imported without any running experiment; no upload path | Independent source audit |
| REQ017,CON002,006 | Service locked digest preserved; root gates and14service regressions pass; prior21contracts pass, final rerun pending atcheckpoint | Final21rerun completion and exact-head four-platform CI |
| REQ020,CON008 | Reproduction commands in native-evidence.md; rawJSON and known-tick screenshots | Final launch/learning handoff and native four-way CI |
| CON001,005 | Same-origin embedded assets, no runtime npm/cloud/API; replay/import never submit jobs | Independent network/mutation/error-path audit incl uncertainPOST |

VER001 final independent code audit pending. VER002 partial actual demonstrations as above. VER003 actual reference workload complete, independent review pending. VER004 final exact-head four-native matrix pending. VER005 final launch/interaction handoff pending. VER006 approved independent arithmetic unchanged. Acceptance authority remains the independently reviewing main orchestrator.

Native RSS method: browser_process_sample.py --monitor94864 --seconds600 --outputbrowser-final-memory.json (normal argv separate tokens). Held process handle and creationFILETIME preventPIDreuse;72947samples,418.094seconds,requested5ms/actualmax16ms interval, native lifetime peak fills sampling gaps. OS thread descriptions independently locate CrRendererMain and oneDedicatedWorker92088 in thatPID; full before/open/load/close originals preserved privately with SHA provenance, scoped public inventories explicitly derived/filter-only. The PID exited on tab close. Shared browser/GPU processes are excluded and never described as zero. Early spare-renderer attribution was rejected before measurement.
