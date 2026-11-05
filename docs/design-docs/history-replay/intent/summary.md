# Bounded causal correction

Status: draft design only. Issue22 compares explicit stale rejection with finite sample-time correction on the same virtual admission stream. Accepted69 provides timestamps and an immutable OnTimeOnly comparator. Actual async transport under parent6 is outside scope. No new codec, accuracy series or estimator equations. The caller supplies validated Trace/ReplayModel and a0..500tick history request; success returns immutable arrival-time outputs and explicit decisions, errors expose no successful prefix.
