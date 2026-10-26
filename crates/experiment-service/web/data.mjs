// Replaceable diagnostic design seam, separately authorized by the orchestrator.
// No parsing, replay, browser, HTTP-route or UI behavior is implemented here.
function missing() { throw new Error("LOCAL_DASHBOARD_NOT_IMPLEMENTED"); }
export class CsvRecordingParser { constructor() { missing(); } }
export class Replay { constructor() { missing(); } }
export function parseSnapshot() { missing(); }
export function serializeConfig() { missing(); }
export function positionEllipse() { missing(); }
export function rowErrors() { missing(); }
export function selectPlotRows() { missing(); }
