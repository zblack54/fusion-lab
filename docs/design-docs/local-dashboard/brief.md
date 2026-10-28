# Local recorded experiment dashboard: review brief

Status: APPROVED_DESIGN; implementation and browser acceptance remain pending.

Source: actual GitHub issue #7, read on2026-10-26. The requested result is local inspection of the simulated trajectory, sensor measurements, filter/baseline estimates, uncertainty and errors through an outage and recovery. It includes run selection, configuration, status/cancellation and replay controls; display fidelity, bounded resources, screenshots and an interaction walkthrough are acceptance obligations.

## Proposed smallest architecture

Serve static HTML/CSS/native ES modules from the existing loopback Rust service on the same origin as `/v1`. Avoid a second proxy, cross-origin exceptions, CDN, npm frontend framework, cloud hosting or changes to the synchronous numerical core. Adding known UI routes is a feature #7 integration; existing service API/test lock stays intact.

Run lifecycle is live through the existing bounded status SSE. Numerical curves load from completed CSV artifacts, not a fictitious live observation stream. Issue #6 may later add an independent observation transport. Leaving/changing a view aborts its download/subscription; it does not cancel the underlying experiment. Cancellation requires its own explicit button.

One persistent Web Worker incrementally parses UTF-8 CSV and owns chunked Float64 storage. It retains at most360001 recorded rows ×35numeric columns ×8bytes =100800280numeric bytes, plus bounded chunk/row carry and metadata. No whole170MB CSV string, response.text(), unbounded message history or array of full per-row objects. Loading another run first aborts/cancels the old reader, releases prior storage references and acknowledges reset; pending view changes collapse to the latest one.

Charts draw no more than4096 selected **actual recorded rows** per trace. The slider/value panel can inspect every recorded row. UI always labels displayed plot points versus total recorded samples and explains that downsampling can hide short features. It never silently interpolates sensor measurements or calls rounded CSV a lossless numerical-state replay.

## Interaction and numerical display

* Compact configuration form for every existing field, including full unsigned64bit seed, scenario, duration/noise/process density and optional outage. Client validation improves feedback; service remains authoritative. One create request at a time; lost replies cause an explicit uncertain-status notice and run-list refresh, never an automatic duplicate POST.
* Run list and lifecycle panel; one selected SSE stream. Close on terminal events, ignore stale sequences/wrong ids, bounded reconnect/status refresh. Busy/overload/eviction/pending-cancellation states are visible.
* XY canvas: truth, KF, hold and integration baseline paths; position/velocity observation markers distinguished from unscheduled/outage samples. A recorded95% position covariance ellipse follows the current row; it is descriptive and does not claim calibration.
* Error/covariance timeline and outage band; exact current row shows tick/time, measured values or absence, state/covariance, Euclidean errors and sensor/outage/recovery status. Delivery is explicitly recorded immediate-mode; it does not invent delayed-message telemetry absent from the file.
* Play/pause, seek, speed and reset-replay operate on recorded ticks. Pause does not pause artifact consumption or stall the numerical worker. Reset returns the cursor to row0; it does not create another experiment. Accessibility includes labeled controls, keyboard operation, textual values and curve styles beyond color.

## Important boundary decisions

CSV uses the existing strict35column numeric/boolean dialect. Accept LF/CRLF, optional UTF-8 BOM and a final line without newline. Reject invalid UTF-8, reordered/missing/extra headers, partial observation pairs, non-finite required values, invalid tick/time order, oversized rows, extra rows/bytes and malformed booleans; publish no partial recording as complete. Empty scheduled measurements stay absent, never zero.

The browser must preserve service seed/sequence integers beyond2^53. Keep internal unsigned integer values as validated canonical decimal strings/BigInt, serialize a validated seed as an exact JSON number token, and parse the relevant raw service integer tokens without JSON.parse rounding. Do not change the service wire contract or narrow the backend seed range to work around JavaScript Number limitations.

Recorded covariance is twelve-decimal CSV output. Roundoff can produce degenerate zero axes; display permits nonnegative eigenvalues within documented rounding tolerance, while clearly flagging invalid covariance. Position ellipse uses the existing5.991464547107979 joint95% threshold. Error/ellipse expectations in tests come from analytically chosen synthetic rows, not the implementation's own computed outputs.

## Verification shape

Rust contract harness uses real service HTTP for static asset/routes, request/status/cancel and completed artifact integration. Node24 built-in tests exercise streamed parser chunks, exact u64 transport, replay, independently known covariance/error values, malformed data, row/byte limits and maximum-row resource behavior. No npm dependencies. Every JS case is invoked through a cataloged libtest wrapper that verifies actual selected test execution and exit outcome; syntax checks are required argv commands.

The harness must classify the **concrete** existing Rust service dependency closure and retain a generated Cargo.lock before review; the former zero-dependency-seam lock mistake must not recur. Static routes currently404, which provides a diagnostic real-HTTP red baseline. A separate minimal `data.mjs` export seam that throws a named not-implemented error is requested solely so Node behavioral tests can execute red rather than fail missing-module setup; no functional UI is implemented during design.

Real browser verification is a separate required acceptance plan: launch locally, exercise healthy/outage/recovery and every control, capture screenshots at known ticks, inspect exact values against fixtures, measure memory while loading the maximum artifact and repeatedly switching/aborting views, verify keyboard/contrast/resizing, and confirm slow display cannot stall the numerical core. Headless parser tests cannot substitute for those observations.

## Remaining design work

Complete schema3 catalogs, fixture/API definitions, executable Rust/Node contract tests, declared verification commands, compatibility inventory and initial-red evidence. Minimum production JS seam needs the main orchestrator's separately scoped authorization; service approved sources/locks/tests are otherwise untouched. No product-policy question requires the user's interruption at this stage. Independent review, approval and lock remain with the main orchestrator.
