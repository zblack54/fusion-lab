# Terms
Run: one immutable validated configuration and one owned worker process.
Running: reservation held while the child is spawning, computing or finalizing.
Terminal: completed, cancelled or failed. No terminal-to-running transition exists.
Artifact: service-owned samples.csv or report.txt; not an arbitrary filesystem path.
Reservation: accounting against a configured operating budget, not an OS-enforced memory limit.
Event: run state metadata, not numerical observations or truth used by the estimator.
