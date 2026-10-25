# Iteration note and knowledge handoff

Use this short record for a reviewable change. Put an iteration's explanation
next to its feature or experiment documentation, and link it from the issue/PR.
Replace placeholders with actual evidence; leave unrun checks explicit. A
completed code slice needs an explanation a reader can reproduce, not a dump of
internal logs. Scale detail to the change: a one-statement fix can use a paragraph
and one meaningful example; a model or ownership change needs more.

## Copyable outline

~~~markdown
# Change title

Status: proposed / implemented-unverified / verified / accepted
Tracking: issue, milestone, PR, exact implementation revision
Design authority: brief or formal package, decision status and approved digest if applicable

## What changed
Trigger, previous observable behavior, new observable behavior; one concrete example.

## Why it matters
Research/product question, alternatives, and reason for this choice.
Link the relevant ADR; distinguish proposed decisions from accepted ones.

## Math and assumptions
State, units, model, equation or invariant that changes (or say unchanged).
An independently derived small example; what would falsify the expectation?

## Code tour
Entry point, ownership/data flow, relevant source functions, tests and contract IDs.
Explain where truth is available and where it must remain inaccessible.

## Reproduce
Prerequisites and exact commands, fixed configuration/seeds, expected artifacts.
State overwritten outputs and any external evidence location.

## Evidence
Actual commands/results at the exact revision, independent oracle and review.
Before/after example or table; negative outcomes; failures and unrun checks.
Distinguish code existence, test passage, acceptance and deployed availability.

## Limits and next question
What was not measured or implemented, compatibility, risks, remaining issue links.
No universal performance, calibration or superiority claim from narrow evidence.

## Exercise
A prediction or small derivation the reader can try; provide a checkable answer
or explain how to independently verify it. Suggest one bounded follow-up.
~~~

## Completion check

A reviewer should be able to answer: what happened, why it was chosen, how the
math maps to code, how to rerun it, what evidence could be wrong, and what to try
next. Keep raw execution evidence available separately when needed. Update the
learning path and roadmap status when the change makes their descriptions stale.
Do not copy private local skill instructions into the public documentation.
