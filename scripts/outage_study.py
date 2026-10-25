"""Prespecified paired outage baseline; standard library, no model tuning."""
import argparse
import csv
import datetime
import hashlib
import io
import json
import math
from pathlib import Path
import platform
import random
import statistics
import subprocess
import tempfile

SCENARIOS = {"straight": range(10000, 10050), "maneuver": range(20000, 20050)}
WINDOWS = {"overall": (0, 3001), "outage": (1000, 2000), "post": (2000, 3001)}
ESTIMATORS = {"filter": ("estimated_x", "estimated_y"), "hold": ("hold_x", "hold_y"),
              "integration": ("integrated_x", "integrated_y")}
THRESHOLD = 5.991464547107979
BOOTSTRAPS = 10000


def sha(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def ellipse(dx, dy, a, b, c, d):
    if not all(math.isfinite(v) for v in (dx, dy, a, b, c, d)):
        raise ValueError("nonfinite position/covariance")
    if abs(b-c) > 1e-10 or a <= 0:
        raise ValueError("invalid covariance symmetry/diagonal")
    l00 = math.sqrt(a)
    l10 = c/l00
    remainder = d-l10*l10
    if remainder <= 0:
        raise ValueError("non-positive covariance")
    # Mahalanobis square is ||L^-1 e||^2.
    z0 = dx/l00
    z1 = (dy-l10*z0)/math.sqrt(remainder)
    return z0*z0+z1*z1 <= THRESHOLD


def metrics(rows):
    squared = {name: [] for name in ESTIMATORS}
    covered, trace = [], []
    for row in rows:
        tx, ty = float(row["true_x"]), float(row["true_y"])
        for name, (x, y) in ESTIMATORS.items():
            squared[name].append((float(row[x])-tx)**2 + (float(row[y])-ty)**2)
        dx, dy = float(row["estimated_x"])-tx, float(row["estimated_y"])-ty
        a, b, c, d = (float(row[key]) for key in ("p00", "p01", "p10", "p11"))
        covered.append(ellipse(dx, dy, a, b, c, d))
        trace.append(a+d)
    result = {}
    for window, (start, stop) in WINDOWS.items():
        result[window] = {name+"_rmse": math.sqrt(statistics.fmean(values[start:stop]))
                          for name, values in squared.items()}
        result[window].update(coverage=statistics.fmean(covered[start:stop]),
                              covariance_trace=statistics.fmean(trace[start:stop]),
                              samples=stop-start)
    result["trace_ticks"] = {str(t): trace[t] for t in (999, 1999, 2000, 3000)}
    return result, squared["filter"]


def recovery(healthy, outage):
    # Prefix sums fix both membership and arithmetic independently of window slicing.
    hp, op = [0.0], [0.0]
    for h, o in zip(healthy, outage):
        hp.append(hp[-1]+h)
        op.append(op[-1]+o)
    for tick in range(2000, 2902):
        h = math.sqrt(max(0.0, (hp[tick+100]-hp[tick])/100))
        o = math.sqrt(max(0.0, (op[tick+100]-op[tick])/100))
        if o <= h+0.1:
            return (tick-2000)/100
    return None


def run(binary, scenario, seed, outage, path):
    command = [str(binary), "--duration", "30", "--seed", str(seed), "--scenario", scenario,
               "--position-sigma", "1.0", "--velocity-sigma", "0.15",
               "--acceleration-density", "0.1", "--output", str(path)]
    command += ["--outage-start", "10", "--outage-end", "20"] if outage else ["--no-outage"]
    process = subprocess.run(command, capture_output=True, text=True, timeout=10, check=False)
    if process.returncode:
        raise RuntimeError(f"child argv={command!r} returned {process.returncode}: {process.stderr}")
    with path.open(newline="") as stream:
        reader = csv.DictReader(stream)
        expected = ["tick", "time_s", "true_x", "true_y", "true_vx", "true_vy", "estimated_x",
                    "estimated_y", "estimated_vx", "estimated_vy", "position_x", "position_y",
                    "velocity_x", "velocity_y", "hold_x", "hold_y", "integrated_x", "integrated_y",
                    "position_outage"] + [f"p{r}{c}" for r in range(4) for c in range(4)]
        if reader.fieldnames != expected:
            raise ValueError("unexpected CSV schema")
        rows = list(reader)
    if len(rows) != 3001:
        raise ValueError("unexpected sample count")
    for tick, row in enumerate(rows):
        if None in row or any(value is None for value in row.values()):
            raise ValueError("malformed row")
        if int(row["tick"]) != tick or float(row["time_s"]) != tick/100:
            raise ValueError("tick/time mismatch")
        if row["position_outage"] != str(outage and 1000 <= tick < 2000).lower():
            raise ValueError("outage boundary mismatch")
        for keys, present in ((["position_x", "position_y"], tick % 20 == 0 and not(outage and 1000 <= tick < 2000)),
                              (["velocity_x", "velocity_y"], tick % 5 == 0)):
            if any(bool(row[key]) != present for key in keys):
                raise ValueError("measurement schedule mismatch")
        for key, value in row.items():
            if key != "position_outage" and value and not math.isfinite(float(value)):
                raise ValueError("nonfinite CSV")
    result, errors = metrics(rows)
    return rows, result, errors, {"argv": command, "csv_sha256": sha(path),
                                  "report_sha256": sha(path.with_suffix(".txt")),
                                  "csv_bytes": path.stat().st_size, "returncode": process.returncode}


def quantile(sorted_values, p):
    index = (len(sorted_values)-1)*p
    low = int(index)
    high = min(low+1, len(sorted_values)-1)
    return sorted_values[low] + (index-low)*(sorted_values[high]-sorted_values[low])


def mean_interval(values, resamples):
    replicates = sorted(statistics.fmean(values[i] for i in indices) for indices in resamples)
    return {"mean": statistics.fmean(values), "lower": quantile(replicates, .025),
            "upper": quantile(replicates, .975)}


def summarize(records):
    randomizer = random.Random(20261025)
    resamples = [[randomizer.randrange(50) for _ in range(50)] for _ in range(BOOTSTRAPS)]
    result = {"conditions": {}, "paired_differences": {}, "baseline_differences": {}}
    for condition in ("healthy", "outage"):
        result["conditions"][condition] = {}
        for window in WINDOWS:
            result["conditions"][condition][window] = {
                key: mean_interval([r[condition][window][key] for r in records], resamples)
                for key in ("filter_rmse", "hold_rmse", "integration_rmse", "coverage", "covariance_trace")}
    for window in WINDOWS:
        result["paired_differences"][window] = {
            key: mean_interval([r["outage"][window][key]-r["healthy"][window][key] for r in records], resamples)
            for key in ("filter_rmse", "coverage")}
        result["baseline_differences"][window] = {}
        for baseline in ("hold", "integration"):
            differences = [r["outage"][window]["filter_rmse"]-r["outage"][window][baseline+"_rmse"] for r in records]
            result["baseline_differences"][window][baseline] = dict(
                mean_interval(differences, resamples), wins=sum(v < 0 for v in differences),
                ties=sum(v == 0 for v in differences), losses=sum(v > 0 for v in differences))
    recovered = [r["recovery_seconds"] for r in records if r["recovery_seconds"] is not None]
    result["recovery"] = {"recovered": len(recovered), "not_recovered": len(records)-len(recovered),
                          "median_recovered_seconds": statistics.median(recovered) if recovered else None,
                          "min_recovered_seconds": min(recovered) if recovered else None,
                          "max_recovered_seconds": max(recovered) if recovered else None}
    result["trace_ratios"] = {str(t): {"min": min(r["trace_ratios"][str(t)] for r in records),
                                      "max": max(r["trace_ratios"][str(t)] for r in records)}
                              for t in (1999, 2000, 3000)}
    return result


def self_check():
    row = {"true_x": "0", "true_y": "0", "estimated_x": "3", "estimated_y": "4",
           "hold_x": "0", "hold_y": "2", "integrated_x": "1", "integrated_y": "0",
           "p00": "1", "p01": "0", "p10": "0", "p11": "1"}
    result, _ = metrics([row]*3001)
    assert result["outage"]["filter_rmse"] == 5
    assert result["outage"]["hold_rmse"] == 2
    assert result["outage"]["integration_rmse"] == 1
    assert result["outage"]["coverage"] == 0
    assert ellipse(0, 0, 1, 0, 0, 1)
    assert ellipse(1, 1, 2, 1, 1, 2)
    assert not ellipse(10, 10, 2, 1, 1, 2)
    assert recovery([0]*3001, [0]*3001) == 0
    assert recovery([0]*3001, [1]*3001) is None
    assert quantile([0, 10], .25) == 2.5
    oracle = mean_interval([2, 6], [[0, 0], [0, 1], [1, 1]])
    assert oracle == {"mean": 4.0, "lower": 2.1, "upper": 5.9}
    assert len(set(SCENARIOS["straight"]) | set(SCENARIOS["maneuver"])) == 100
    assert sum(len(seeds) for seeds in SCENARIOS.values())*2 == 200


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/release/fusion-lab.exe"))
    parser.add_argument("--output", type=Path)
    parser.add_argument("--self-check", action="store_true")
    args = parser.parse_args()
    self_check()
    if args.self_check:
        print("analytical metric/recovery/quantile checks passed")
        return
    if args.output is None:
        parser.error("--output is required")
    root = Path(__file__).resolve().parent.parent
    if args.output.resolve().is_relative_to(root):
        parser.error("evidence must be outside repository")
    binary = args.binary.resolve(strict=True)
    data = {"metadata": {"recorded_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                         "platform": platform.platform(), "python": platform.python_version(),
                         "binary": str(binary), "binary_sha256": sha(binary),
                         "cargo_lock_sha256": sha(root/"Cargo.lock"),
                         "protocol_sha256": sha(root/"docs/experiments/outage-study-protocol.md"),
                         "driver_sha256": sha(__file__)}, "records": [], "summary": {}, "failure": None}
    for command in (["git", "rev-parse", "HEAD"], ["git", "status", "--porcelain"], ["rustc", "-Vv"]):
        actual = [command[0], "-c", "safe.directory="+str(root.resolve()), *command[1:]] if command[0] == "git" else command
        data["metadata"][" ".join(command)] = subprocess.check_output(actual, cwd=root, text=True).strip()
    data["metadata"]["source_sha256"] = {str(path.relative_to(root)): sha(path)
        for path in sorted((root/"src").rglob("*.rs"))}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    scenario = seed = condition = None
    try:
        with tempfile.TemporaryDirectory(prefix="fusion-outage-study-") as temp:
            for scenario, seeds in SCENARIOS.items():
                for seed in seeds:
                    pair = {"scenario": scenario, "seed": seed}
                    rows = {}
                    errors = {}
                    for condition in ("healthy", "outage"):
                        path = Path(temp)/(condition+".csv")
                        rows[condition], pair[condition], errors[condition], pair[condition+"_artifact"] = run(
                            binary, scenario, seed, condition == "outage", path)
                    for tick, (h, o) in enumerate(zip(rows["healthy"], rows["outage"])):
                        keys = ["true_x", "true_y", "true_vx", "true_vy", "velocity_x", "velocity_y"]
                        if not 1000 <= tick < 2000:
                            keys += ["position_x", "position_y"]
                        if any(h[key] != o[key] for key in keys):
                            raise ValueError("paired truth/observation streams differ")
                    pair["recovery_seconds"] = recovery(errors["healthy"], errors["outage"])
                    pair["trace_ratios"] = {str(t): pair["outage"]["trace_ticks"][str(t)] /
                                            pair["healthy"]["trace_ticks"][str(t)] for t in (1999, 2000, 3000)}
                    data["records"].append(pair)
                    for name in ("healthy", "outage"):
                        (Path(temp)/(name+".csv")).unlink()
                        (Path(temp)/(name+".txt")).unlink()
                data["summary"][scenario] = summarize([r for r in data["records"] if r["scenario"] == scenario])
                print(scenario+": 50 paired seeds complete", flush=True)
    except Exception as error:
        data["failure"] = {"type": type(error).__name__, "message": str(error),
                           "scenario": scenario, "seed": seed, "condition": condition}
        raise
    finally:
        args.output.write_text(json.dumps(data, indent=2, allow_nan=False)+"\n", encoding="utf-8")
    print("completed 200 CLI runs; evidence: "+str(args.output), flush=True)


if __name__ == "__main__":
    main()
