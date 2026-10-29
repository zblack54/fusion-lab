# ADR-003 bounded-quality-and-staged-adapters

Status: selected for draft design under main preparation authority; final contract approval pending.

Options were introduce runtime/channels/history together or first prove a pure bounded replay. The draft selects the latter, with explicit512MiB/30s measured obligations and no OSquota claim. Actual channel/slowconsumer/cancellation/historycontracts remainlater6/22work. Negative consequence: no actual async capability ships in this slice. Revisit on measuredbudgetfailure or lateradapterdesign, preserving actualfailures. AffectedREQ032–037;CON006–008;MOD001–004;VER003–006.
