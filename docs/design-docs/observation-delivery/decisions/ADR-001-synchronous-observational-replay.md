# ADR-001 synchronous-observational-replay

Status: selected for draft design under main preparation authority; final contract approval pending.

Options were apply old observations at arrival time, correct bounded history now, or first establish strictOnTimeOnly. The draft selects OnTimeOnly because it never silently uses an old measurement as current and isolates a small deterministic reference. Positive delays are intentionally uninformative to this baseline; useful history correction remains22. Revisit only with a reviewed late-data policy. AffectedREQ001–023,029–032;CON001–003,005–006;MOD001/002; acceptance/state/numerical tests.
