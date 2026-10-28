# Final conformance review

Status: READY_FOR_MAIN_ACCEPTANCE — local verification and independent bounded source/conformance review complete; exact published-head native CI and main acceptance remain.

Approved external dashboard digest: d33ff5c8c1828d503d0cecfb144df72ab96621fa0f2e8dd8096c7f98ba0ddd95. Accepted service digest3862f6d829e8a739580454f0edfbfb6aff696007286c51dbcfd61bd6f7529f1f is preserved. No locked tests, dependencies, numerical source or threshold changed. Source inventory: source-checkpoint.json.

## Per-obligation evidence

| Obligations | Completed local evidence | Release gate |
|---|---|---|
| REQ001 | Real HTTP root/assets/MIME/CSP locked acceptance; explicit embedded route whitelist, no filesystem route translation | Independent final source review and CI |
| REQ002,014 | Real config/create/status/SSE, busy409, Cancel→cancelled with controlled child listener closed, retention1 eviction, worker_failed, overload503. Reconnect fixture opens four SSE attempts with1.057/2.051/4.051s gaps then stops for manual recovery. | Review/CI |
| REQ003–006,011 | Unchanged17-test parser/wire suite executes34times across local1.99/stable; full35-field literal oracle and real HTTP artifact; browser rows0/1500/2100/7777 compare to actual CSV. Presence[0,0] vsabsence and fullu64 exact; numeric-token spoof regression passes. | Review/CI |
| REQ007 | Independent5/10/13 errors and ellipse oracle; anisotropy/MAX_VALUE/tiny examples retained; no calibration claim | Review/CI |
| REQ008–009 | Play/pause/seek/speed/reset contract tests; real keyboard and30exact seeks, nonplotted7777; distinct recorded indices≤4096; no interpolation | Review/CI |
| REQ010,013,019; CON003,007 | Continuous-worker warmup+3maximum170624208byte imports;100800280numericbytes, maxima6.8/5.3/4.7/4.7ms,≤651rows betweenyields; oneWorker/latestpendingselection, worker ownership resets. Fiftyselection/reset interactions,49 explicitly priorLoading;30additional seeks. Ownedrenderer+DedicatedWorker lifetimeRSS252489728,27351samples/158.563s,16ms maxgap; tabclose/PIDexit confirmed. Loading49 p95nextRAFproxy72.3ms/max74.8; supportedLongTasks0; source render≤30fps and two capped canvases. | Review/CI; reference-host/proxy limitations below |
| REQ012,018; CON004 | Actual healthy0/outage1500/recovery2100/max7777 screenshots and exact DOM/CSV records; total/plotted/precision/delivery/calibration labels present | Review/CI |
| REQ015 | Real390×844 override actualclient375/scroll375, no horizontaloverflow; ArrowUp1501, EnterPlay, SpacePause, EnterReset0, TabSpeed; curve styles and exact text | Review/CI |
| REQ016 | Actual localCSV import without experiment/upload, same bounded worker parser | Review/CI |
| REQ017; CON002,006 | Final root6gates and service6gates pass;14service production regressions; trusted service21pass; both external lock validators valid0errors0warnings. Numerical sources unchanged. | Four native OS/compiler variants at final published head |
| REQ020; CON008 | Root/serviceREADME launch path, helpGET/dashboard sentence, owner lesson, actual Chromium156/Win10 UA, referencehardware metadata, known-tick screenshots and walkthrough | Review/CI |
| CON001,005 | Sameorigin/self-only assets, no npm/cloud/API. Replay/view never submit/cancel. Actual uncertainPOST fixture beforePOST1/GET1 vsafterPOST1/GET2; accepted/lost reply stays uncertain, existinglocalCSVcursor1500 retained. FailedGET503 explicitmanualrecovery. New malformed200list regression records red→green and validates entire list before DOMmutation; selectedrun1/options/cursor90/workermessages preserved, noPOSTretry. | Review/CI |

## Actual final local checks

Trusted dashboard implementation runner:34/34pass, all5supplemental checks, errors=[] against externald33 baseline. Supplementary production Node tests:12pass/0fail, including new native-select malformed-list case and realWorker12ms notification fault. Root fmt/check/clippy/test/doctest/doc and service fmt/check/clippy/test/doctest/doc allpass; service14regressions pass; trusted unchanged service21pass. Raw final-local-gates.json and execution.json retain exact commands/output.

## Scope interpretation and measurement limits

Issue7comment6451759075 records main's v1 interpretation: XY crosses show position observations in metres; velocity observations are exact [vx,vy] values(m/s) and received/unscheduled status, not vectors or another plot. No locked file changed; no vector scale invented.

The independent reviewer corroborated continuous b26 worker/resource evidence. Earlier4.9ms snippet evidence is historical and cannot establish whole-worker timing. Current12th regression/validation-loop change affects only malformed-list rejection before DOMmutation; #60 error-path repair and help/docs do not change parser, worker, rendering or replay hot paths, as independently reviewed. Do not label a historical executable hash as the current executable: source hashes and measured method identify each session.

The p95 metric is input timestamp through a later animation-frame opportunity, a presentation proxy, not physical screen proof. Loading-only49 samples are independently retained before bounded64history replaces older events. Fiftyreset interactions are not claimed as fifty proven active-reader aborts. Native held process handles identify an existingblank renderer gaining aDedicatedWorker then exiting when its ownedtab closes; no absent-before/newPID claim. Sharedbrowser/GPU processes are excluded, not zero. Windows reference RSS is not an OS cap or Linux browser measurement. Raw process inventories remain private; published derived views retain SHA provenance and owned rows only.

VER001–003 and005 local evidence is complete and independently reviewed by /root/final_review; VER004 requires current-head four-native CI; VER006 independently approved arithmetic unchanged. Only main records final ACCEPTED after those gates. No selfacceptance is claimed.
