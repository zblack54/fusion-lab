# Terms

Arrival tick is virtual availability, not wall-clock receipt. Sample tick is observation time. Prior anchor precedes sample0; end-tick checkpoint includes all known canonical updates through that sample tick. Online snapshot is the state emitted at its original arrival tick, never rewritten retrospectively. Correction consumes only payloads available by the current arrival tick. Live object count differs from reserved slot capacity and whole-process lifetime RSS.
