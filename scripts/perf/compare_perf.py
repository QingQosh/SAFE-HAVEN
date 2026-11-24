#!/usr/bin/env python3
"""Compare current performance metrics against a committed baseline.

Fails (non-zero exit) when response time regresses more than 10% or memory
usage regresses more than 15% versus the baseline. Also appends a history
record so performance trends can be tracked over time.

Usage:
    python scripts/perf/compare_perf.py \
        --baseline scripts/perf/baseline.json \
        --current perf-current.json \
        --history scripts/perf/history.jsonl
"""

import argparse
import json
import os
import sys
from datetime import datetime, timezone

RESPONSE_TIME_THRESHOLD = 0.10  # fail if response time increases > 10%
MEMORY_THRESHOLD = 0.15  # fail if memory usage increases > 15%


def load_json(path):
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def pct_increase(baseline, current):
    if baseline == 0:
        return 0.0 if current == 0 else float("inf")
    return (current - baseline) / baseline


def compare(baseline, current):
    failures = []
    results = []

    for metric, threshold in (
        ("response_time_ms", RESPONSE_TIME_THRESHOLD),
        ("memory_usage_mb", MEMORY_THRESHOLD),
    ):
        base_value = baseline.get(metric)
        cur_value = current.get(metric)
        if base_value is None or cur_value is None:
            failures.append("missing metric '%s' in baseline or current data" % metric)
            continue

        increase = pct_increase(base_value, cur_value)
        regressed = increase > threshold
        results.append(
            {
                "metric": metric,
                "baseline": base_value,
                "current": cur_value,
                "increase_pct": round(increase * 100, 2),
                "threshold_pct": round(threshold * 100, 2),
                "regressed": regressed,
            }
        )
        if regressed:
            failures.append(
                "%s regressed by %.2f%% (baseline=%s, current=%s, allowed=%.2f%%)"
                % (metric, increase * 100, base_value, cur_value, threshold * 100)
            )

    return results, failures


def append_history(path, current, results):
    record = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "metrics": current,
        "comparison": results,
    }
    directory = os.path.dirname(path)
    if directory:
        os.makedirs(directory, exist_ok=True)
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(json.dumps(record) + "\n")


def main(argv=None):
    parser = argparse.ArgumentParser(description="Performance regression check")
    parser.add_argument("--baseline", required=True, help="Baseline metrics JSON")
    parser.add_argument("--current", required=True, help="Current metrics JSON")
    parser.add_argument("--history", help="Optional JSONL file for trend tracking")
    args = parser.parse_args(argv)

    baseline = load_json(args.baseline)
    current = load_json(args.current)

    results, failures = compare(baseline, current)

    for result in results:
        status = "REGRESSED" if result["regressed"] else "ok"
        print(
            "%s: baseline=%s current=%s change=%+.2f%% (limit %.2f%%) [%s]"
            % (
                result["metric"],
                result["baseline"],
                result["current"],
                result["increase_pct"],
                result["threshold_pct"],
                status,
            )
        )

    if args.history:
        append_history(args.history, current, results)

    if failures:
        print("\nPerformance regression detected:", file=sys.stderr)
        for failure in failures:
            print("  - " + failure, file=sys.stderr)
        return 1

    print("\nNo performance regressions detected.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
