# Terms

A tick is an integer10ms numerical instant. Sample time is generation time; planned arrival is virtual availability, not a claim of actual receipt. An offered observation has a sensor plus its original scheduled sequence; outages/loss leave sequence gaps. A source decision occurs at sampling (scripted/queue loss), arrival (accepted/stale), or final horizon (undelivered).

OnTimeOnly accepts only arrived sample_tick==current_tick observations. Positive delay never authorizes a current-state update. Waiting capacity counts only future observations. Zero delay bypasses waiting slots. Source ledger order is sample/sensor/sequence; arrival order includes only actually considered Accepted/StaleDrop observations in planned-arrival/sample/sensor/sequence order. Position precedes velocity on a tie.

A complete result contains snapshots0through100*duration inclusive, every source disposition and actual-consumer identity order. It contains no truth. RSS/CPU limits are measured quality obligations, not OS-enforced quotas. Expected red proves a diagnostic suite executes; it does not prove delivery behavior exists.
