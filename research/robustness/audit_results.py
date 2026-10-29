"""Independently audit complete study records without importing the executor.

Only retained summaries and aggregates are required. Optional executed source
and binary checks must point to the original frozen artifacts, not a later
hardened executor revision. Deleted raw CSVs are not reverified by this script.
"""
import argparse
import hashlib
import gzip
import itertools
import json
import math
from pathlib import Path


KEYS = list(itertools.product(range(1000, 1032), ('straight', 'maneuver'), range(3), range(3)))


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key: ' + key)
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError('nonfinite JSON constant: ' + value)


def payload(path):
    raw = path.read_bytes()
    return gzip.decompress(raw) if path.suffix == '.gz' else raw


def read(path):
    return json.loads(payload(path).decode('utf-8'), object_pairs_hook=unique,
                      parse_constant=reject_constant)


def label(key):
    return '-'.join(map(str, key))


def stats(values):
    count = len(values)
    mean = math.fsum(values) / count
    return {'n': count, 'mean': mean,
            'sample_sd': math.sqrt(math.fsum((value - mean) ** 2 for value in values) / (count - 1)),
            'min': min(values), 'max': max(values)}


def compare(expected, actual, path=''):
    if isinstance(expected, dict):
        require(isinstance(actual, dict) and set(expected) == set(actual), path + ': object keys')
        return sum(compare(value, actual[key], path + '/' + key) for key, value in expected.items())
    if isinstance(expected, (list, tuple)):
        require(isinstance(actual, list) and len(expected) == len(actual), path + ': array length')
        return sum(compare(left, right, path + '/' + str(index))
                   for index, (left, right) in enumerate(zip(expected, actual)))
    if isinstance(expected, (int, float)):
        require(isinstance(actual, (int, float)) and math.isfinite(expected) and math.isfinite(actual),
                path + ': nonfinite/non-numeric value')
        require(abs(expected - actual) <= 1e-12 + 1e-12 * max(abs(expected), abs(actual)),
                path + ': aggregate mismatch')
        return 1
    require(expected == actual, path + ': value mismatch')
    return 0


def hash_shape(value):
    return isinstance(value, str) and len(value) == 64 and all(char in '0123456789abcdef' for char in value)


def audit(data_dir, source_root=None, binary=None):
    frozen = read(data_dir / 'frozen.json')
    record_paths = [path for path in [data_dir / 'records.json', data_dir / 'records.json.gz'] if path.exists()]
    require(len(record_paths) == 1, 'provide exactly one records.json or records.json.gz')
    record_path = record_paths[0]
    records = read(record_path)
    aggregates = read(data_dir / 'aggregates.json')
    require(frozen['inventory'] == [list(key) for key in KEYS], 'frozen inventory mismatch')
    require(len(records) == 576 and set(records) == {label(key) for key in KEYS}, 'record inventory mismatch')
    require(all(hash_shape(value) for value in frozen['files'].values()), 'invalid frozen source hash')
    require(hash_shape(frozen['binary_sha256']), 'invalid frozen binary hash')
    if source_root is not None:
        root = source_root.resolve()
        for name, digest in frozen['files'].items():
            path = (root / name).resolve()
            require(path.is_relative_to(root), 'source manifest path escapes root')
            require(sha(path) == digest, 'executed source mismatch: ' + name)
    if binary is not None:
        require(sha(binary) == frozen['binary_sha256'], 'executed binary mismatch')

    artifact_sizes = []
    boundary_keys = []
    failed_attempts = []
    for key in KEYS:
        item = records[label(key)]
        require(item['status'] == 'verified' and item['key'] == list(key), 'record status/key mismatch')
        require(item['command'][1:5] == list(map(str, key)), 'command key mismatch')
        for window, count in [('overall', 3001), ('outage', 1000)]:
            metrics = item['summary'][window]
            require(metrics['samples'] == count and all(math.isfinite(value) for value in metrics.values()),
                    'metric count/nonfinite value')
            require(all(metrics[field] >= 0 for field in ['filter_rmse', 'hold_rmse', 'integration_rmse']),
                    'negative RMSE')
            coverage = metrics['position_coverage']
            require(0 <= coverage <= 1 and abs(coverage * count - round(coverage * count)) < 1e-9,
                    'coverage is not a counted fraction')
        tick = item['summary']['recovery_tick']
        require(tick is None or type(tick) is int and 2000 <= tick <= 2900, 'invalid recovery tick')
        require(hash_shape(item['csv_sha256']) and hash_shape(item['roundtrip_sha256']), 'invalid artifact hash')
        require(0 < item['csv_bytes'] <= 32 * 1024 * 1024, 'invalid CSV size')
        artifact_sizes.append(item['csv_bytes'])
        attempts = item['attempts']
        require(1 <= len(attempts) <= 2 and attempts[-1]['status'] == 'verified', 'invalid attempt history')
        require(attempts[-1]['key'] == list(key) and attempts[-1]['summary'] == item['summary'],
                'final attempt differs from record')
        for attempt in attempts[:-1]:
            require(attempt['status'] == 'failed' and attempt['transient'], 'invalid retry history')
            failed_attempts.append({'key': list(key), 'error': attempt['error']})
        if item['rounding_boundary_rows']:
            boundary_keys.append(label(key))

    independently = []
    for trajectory, scale, bias in itertools.product(('straight', 'maneuver'), range(3), range(3)):
        selected = [records[label((seed, trajectory, scale, bias))]['summary'] for seed in range(1000, 1032)]
        group = {'trajectory': trajectory, 'scale': [.5, 1., 2.][scale],
                 'bias': [[0., 0.], [.3, -.15], [1., -.5]][bias]}
        for window, field in [('outage', 'filter_rmse'), ('outage', 'position_coverage'),
                              ('overall', 'filter_rmse'), ('overall', 'position_coverage'),
                              ('outage', 'hold_rmse'), ('outage', 'integration_rmse')]:
            group[window + '_' + field] = stats([item[window][field] for item in selected])
        differences = [records[label((seed, trajectory, scale, bias))]['summary']['outage']['filter_rmse']
                       - records[label((seed, trajectory, 1, bias))]['summary']['outage']['filter_rmse']
                       for seed in range(1000, 1032)]
        paired = stats(differences)
        paired['monte_carlo_standard_error'] = paired['sample_sd'] / math.sqrt(32)
        group['paired_outage_rmse_vs_scale1'] = paired
        latencies = [(item['recovery_tick'] - 2000) / 100 for item in selected if item['recovery_tick'] is not None]
        group['recovery'] = {'total': 32, 'recovered': len(latencies), 'censored': 32 - len(latencies),
                             'recovered_latency_seconds': stats(latencies) if len(latencies) > 1 else {'values': latencies}}
        independently.append(group)
    comparisons = compare(independently, aggregates)
    return {'status': 'independent full-record aggregate audit passed',
            'runs': 576, 'snapshots': 576 * 3001, 'groups': 18,
            'aggregate_numeric_comparisons': comparisons,
            'protocol_sha256': frozen['files']['docs/design/robustness-protocol.md'],
            'revision': frozen['revision'],
            'frozen_sha256': sha(data_dir / 'frozen.json'),
            'records_sha256': hashlib.sha256(payload(record_path)).hexdigest(),
            'records_archive_sha256': sha(record_path) if record_path.suffix == '.gz' else None,
            'aggregates_sha256': sha(data_dir / 'aggregates.json'),
            'auditor_sha256': sha(Path(__file__)),
            'executed_source_verification': 'passed' if source_root is not None else 'not requested',
            'executed_binary_verification': 'passed' if binary is not None else 'not requested',
            'csv_bytes_total': sum(artifact_sizes), 'rounding_boundary_keys': boundary_keys,
            'failed_numerical_attempts': failed_attempts,
            'limitations': ['Independent aggregate audit uses retained source summaries; deleted raw CSVs are not re-read.',
                            'Per-run numerical checks and invariants were performed by the frozen executor.',
                            'Checkpoint acknowledgement failures are documented separately; they are not numerical retries.',
                            'Boundary artifact contents, if any, require a separate raw-artifact audit.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--data-dir', type=Path, required=True)
    parser.add_argument('--executed-source-root', '--source-root', dest='source_root', type=Path)
    parser.add_argument('--executed-binary', type=Path)
    parser.add_argument('--output', type=Path, help='Optional JSON audit evidence destination')
    args = parser.parse_args()
    result = audit(args.data_dir, args.source_root, args.executed_binary)
    rendered = json.dumps(result, indent=2)
    if args.output is not None:
        args.output.write_text(rendered + '\n', encoding='utf-8')
    print(rendered)


if __name__ == '__main__':
    main()
