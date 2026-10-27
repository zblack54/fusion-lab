# Terms and units
Task: immutable positive u64 task ID, positive GitHub issue number, approved scope
digest and already-known dependency IDs. Duplicate ID is rejected.
Attempt: one claimed task execution, random owner token plus strictly increasing
generation, child PID/creation identity, session/checkpoint, deadline and evidence.
Ready means dependencies may still block selection; Done means merged evidence
is accepted, not merely that a worker emitted text.
Verified means checks passed at exactly the recorded head.
Intent: stable ASCII ID (1..128 characters from letters/digits/colon/underscore/
hyphen), kind, task and expected head. It is persisted before file export.
ACK: host's typed observation for an existing intent; applied, not_applied or
uncertain. Applied RequestReview attests approved independent review and passed
checks at the exact head, not merely posting a review request.
Quota uses percent consumed in [0,100], minutes per window, Unix UTC seconds
for reset. Used 100 is exhausted. Unknown is unavailable, never zero.
Scheduler deadlines use monotonic elapsed milliseconds within a process, with
persisted UTC due times reconciled conservatively after restart.
An eligible primary reset uses the server timestamp. WeeklyLatched persists
until explicit owner change; after reaching its due time, next weekly probe is
the previous due plus the least positive multiple of 604800 seconds exceeding
now, unless a later server weekly reset supplies the next due. Thus due=1000,
now=1000, allowance restored gives next_wake=605800, not a stale past wake.
A primary window is identified by duration 300 minutes and weekly by 10080;
missing/ambiguous relevant bucket or windows pauses. Backend compatibility
changes require explicit design revision rather than positional guessing.
