"""Validate sealed issue #8 records and report paired seed-level intervals."""
import csv
import json
import math
import random
import statistics
import sys
from pathlib import Path

ARMS = {"H": (752, 0, 0, 0), "D": (502, 250, 0, 2100),
        "L": (637, 0, 115, 0), "DL": (426, 211, 115, 2080)}
METRICS = ("rmse", "post_rmse", "coverage", "post_coverage")
WINDOWS = (0, 500)
FAMILIES = ("straight", "maneuver")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def load_records(path, mode):
    starts, count = {"fixture": ((30000, 40000), 4), "heldout": ((31000, 41000), 50)}[mode]
    ranges = {f: range(s, s + count) for f, s in zip(FAMILIES, starts)}
    records = {}
    with path.open(newline="", encoding="utf-8") as stream:
        for row in csv.DictReader(stream, delimiter="\t"):
            family, seed, arm, window = row["family"], int(row["seed"]), row["arm"], int(row["window"])
            key = family, seed, arm, window
            require(family in ranges and seed in ranges[family] and arm in ARMS
                    and window in WINDOWS and key not in records, f"invalid or duplicate row {key}")
            counts = tuple(int(row[k]) for k in ("ontime", "late", "lost", "last_late"))
            require(counts == ARMS[arm], f"unexpected transport counts {key}")
            metrics = {k: float(row[k]) for k in METRICS}
            require(all(math.isfinite(v) and v >= 0 for v in metrics.values()), f"invalid metric {key}")
            require(all(metrics[k] <= 1 for k in ("coverage", "post_coverage")), f"invalid coverage {key}")
            if arm == "D":
                event, elapsed = int(row["event"]), int(row["elapsed"])
                require(event in (0, 1) and 99 <= elapsed <= 880 and (event or elapsed == 880),
                        f"invalid recovery {key}")
                metrics.update(event=event, elapsed=elapsed)
            else:
                require(not row["event"] and not row["elapsed"], f"unexpected recovery {key}")
            records[key] = metrics
    require(len(records) == count * 2 * 4 * 2, "incomplete required rows")
    for family, seeds in ranges.items():
        for seed in seeds:
            for arm in ("H", "L"):
                require(records[family, seed, arm, 0] == records[family, seed, arm, 500],
                        f"policies differ without delay: {family}/{seed}/{arm}")
    return ranges, records


def percentile(sorted_values, p):
    rank = p * (len(sorted_values) - 1)
    low = math.floor(rank)
    high = math.ceil(rank)
    return sorted_values[low] + (rank - low) * (sorted_values[high] - sorted_values[low])


def analyze(ranges, records):
    rng = random.Random(80122)
    count = len(ranges["straight"])
    draws = [tuple(tuple(rng.randrange(count) for _ in range(count)) for _ in FAMILIES)
             for _ in range(10_000)]

    def series(left, right, metric):
        values = [[records[f, s, *left][metric] -
                   (records[f, s, *right][metric] if right else 0)
                   for s in ranges[f]] for f in FAMILIES]
        means = [statistics.fmean(v) for v in values]
        boot = [[], [], []]
        for draw in draws:
            a, b = [statistics.fmean(v[i] for i in d) for v, d in zip(values, draw)]
            for bucket, estimate in zip(boot, (a, b, (a + b) / 2)):
                bucket.append(estimate)
        return {label: {"mean": estimate,
                        "interval95": [percentile(sorted(bucket), p) for p in (0.025, 0.975)]}
                for label, estimate, bucket in zip((*FAMILIES, "equal_weight"),
                                                   (*means, sum(means) / 2), boot)}

    result = {"primary_DL_W500_minus_W0_rmse": series(("DL", 500), ("DL", 0), "rmse")}
    result["arm_summaries"] = {
        f"{arm}_W{w}_{metric}": series((arm, w), None, metric)
        for arm in ARMS for w in WINDOWS for metric in METRICS}
    result["fault_minus_healthy"] = {
        f"{arm}_W{w}_{metric}": series((arm, w), ("H", w), metric)
        for arm in ("D", "L", "DL") for w in WINDOWS for metric in METRICS}
    result["recovery_delay_only"] = {
        metric: series(("D", 500), ("D", 0), metric) for metric in ("event", "elapsed")}
    result["recovery_by_policy"] = {
        f"D_W{w}_{metric}": series(("D", w), None, metric)
        for w in WINDOWS for metric in ("event", "elapsed")}
    result["censored"] = {
        f"{family}_W{w}": sum(not records[family, seed, "D", w]["event"] for seed in ranges[family])
        for family in FAMILIES for w in WINDOWS}
    return result


def main():
    mode, input_path, output_path = sys.argv[1:]
    require(sys.version_info[:2] == (3, 12), "protocol requires Python 3.12")
    ranges, records = load_records(Path(input_path), mode)
    result = {"mode": mode, "rows": len(records), "analysis": analyze(ranges, records)}
    with Path(output_path).open("x", encoding="utf-8", newline="\n") as out:
        json.dump(result, out, indent=2, allow_nan=False)
        out.write("\n")


if __name__ == "__main__":
    main()
