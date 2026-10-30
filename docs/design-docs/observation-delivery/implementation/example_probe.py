"""Owned example demonstrations; Linux faults use a one-page private tmpfs."""
from __future__ import annotations
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
import resource_probe as owner

ROOT = Path(__file__).resolve().parents[4]
CAP = 65536

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def command(argv, output, name):
    record = {'argv': [str(a) for a in argv], 'owned_pid': None}
    stdout, stderr = output / (name + '.stdout'), output / (name + '.stderr')
    started = time.monotonic()
    with stdout.open('xb') as out, stderr.open('xb') as err:
        child = subprocess.Popen(record['argv'], cwd=ROOT, stdout=out, stderr=err)
        record['owned_pid'] = child.pid
        try:
            record['returncode'] = child.wait(timeout=60)
            record['reap_confirmed'] = True
        except BaseException as error:
            record['error'] = repr(error)
            owner.reap_owned(child, record, output / (name + '.json'))
            raise
        finally:
            # A successful wait is the only normal return path.
            if record.get('reap_confirmed') is not True:
                owner.reap_owned(child, record, output / (name + '.json'))
            record['seconds'] = time.monotonic() - started
            (output / (name + '.json')).write_text(json.dumps(record, indent=2) + '\n')
    for key, path in [('stdout', stdout), ('stderr', stderr)]:
        if path.stat().st_size > CAP:
            raise RuntimeError('example output exceeded bounded diagnostic cap')
        record[key] = path.read_text(errors='replace')
    return record

def owned_mount(path):
    for line in Path('/proc/self/mountinfo').read_text().splitlines():
        fields = line.split()
        decoded = re.sub(r'\\([0-7]{3})', lambda m: chr(int(m[1], 8)), fields[4])
        if decoded == str(path):
            tail = fields[fields.index('-') + 1:]
            return {'device': fields[2], 'mount_point': decoded, 'mount_options': fields[5],
                    'filesystem': tail[0], 'super_options': tail[2]}
    return None

def space(path):
    stat = os.statvfs(path)
    return {'fragment_bytes': stat.f_frsize, 'blocks': stat.f_blocks,
            'free_blocks': stat.f_bfree, 'available_blocks': stat.f_bavail,
            'available_inodes': stat.f_favail}

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

def fault_case(binary, scratch, output, name, prefill, positive_trace):
    mount = scratch / name
    mount.mkdir()
    receipt = {'case': name, 'success': False}
    try:
        mounted = command(['sudo', '-n', 'mount', '-t', 'tmpfs', '-o',
                           'size=4096,nr_inodes=32,nodev,nosuid,noexec', 'tmpfs', mount],
                          output, name + '-mount')
        require(mounted['returncode'] == 0, 'owned tmpfs mount failed')
        info = owned_mount(mount)
        require(info is not None and info['filesystem'] == 'tmpfs', 'owned mount identity missing')
        receipt['mount'] = info
        receipt['empty_space'] = empty = space(mount)
        require(os.sysconf('SC_PAGE_SIZE') == 4096, 'fixture requires actual 4096-byte pages')
        require(empty['fragment_bytes'] == 4096 and empty['blocks'] == 1
                and empty['available_blocks'] == 1 and empty['available_inodes'] >= 8,
                'unexpected one-page tmpfs capacity/inodes')
        if prefill:
            with (mount / 'owned-prefill').open('xb') as handle:
                require(handle.write(bytes(4096)) == 4096, 'short fixture prefill')
                handle.flush()
                os.fsync(handle.fileno())
            require(space(mount)['available_blocks'] == 0, 'prefill did not occupy the page')
        target = mount / 'new-output'
        result = command([binary, target], output, name + '-example')
        receipt['command'] = result
        require(result['returncode'] != 0 and target.is_dir(), 'fault must occur after owned dir creation')
        require('os error 28' in result['stderr'], 'observed error is not ENOSPC')
        require('Published complete' not in result['stdout'], 'failure falsely printed publication success')
        files = list(target.iterdir())
        require(len(files) <= 4 and all(p.is_file() for p in files), 'unexpected owned fixture entries')
        receipt['files'] = {p.name: {'bytes': p.stat().st_size, 'sha256': sha(p)} for p in files}
        require(not (target / 'ledger.tsv').exists(), 'failed ledger must not be published')
        if prefill:
            require('trace publication failed' in result['stderr']
                    and 'no ledger was published' in result['stderr'], 'trace failure context missing')
            require(not (target / 'trace.tsv').exists(), 'failed trace must not be published')
        else:
            require('ledger publication failed' in result['stderr']
                    and 'complete trace.tsv' in result['stderr'], 'partial publication context missing')
            require((target / 'trace.tsv').read_bytes() == positive_trace,
                    'published prior trace differs from positive reference')
        receipt['remaining_space'] = space(mount)
        receipt['success'] = True
    except BaseException as error:
        receipt['error'] = repr(error)
        raise
    finally:
        # Never remove a mounted directory or hide an unmount failure.
        try:
            if owned_mount(mount) is not None:
                unmounted = command(['sudo', '-n', 'umount', mount], output, name + '-umount')
                receipt['unmount'] = unmounted
                require(unmounted['returncode'] == 0 and owned_mount(mount) is None,
                        'owned mount cleanup failed; fixture retained')
        except BaseException as cleanup_error:
            receipt['success'] = False
            receipt['cleanup_error'] = repr(cleanup_error)
            raise
        finally:
            (output / (name + '-receipt.json')).write_text(json.dumps(receipt, indent=2) + '\n')
    return receipt

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    binary, output = args.binary.resolve(), args.output.resolve()
    require(binary.is_file() and ROOT in binary.parents, 'binary must be in this owned checkout')
    require(ROOT in output.parents and output != ROOT, 'evidence directory must be below checkout')
    output.mkdir()
    target_root = (ROOT / 'target').resolve()
    target_root.mkdir(exist_ok=True)
    scratch = Path(tempfile.mkdtemp(prefix='delivery-example-', dir=target_root)).resolve()
    require(scratch.parent == target_root and scratch.name.startswith('delivery-example-'),
            'scratch ownership mismatch')
    paths = [ROOT / 'src/delivery.rs', ROOT / 'examples/observation_delivery.rs',
             ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', Path(__file__).resolve(), Path(owner.__file__).resolve()]
    before = {str(p.relative_to(ROOT)): sha(p) for p in paths}
    receipt = {'success': False, 'platform': platform.platform(), 'python': sys.version,
               'source_before': before, 'binary': str(binary.relative_to(ROOT)), 'binary_sha256': sha(binary)}
    try:
        positive_dir = scratch / 'positive'
        positive = command([binary, positive_dir], output, 'positive')
        require(positive['returncode'] == 0, 'positive example failed')
        require('101 snapshots, 27 offered observations' in positive['stdout'], 'unexpected example workload')
        require({p.name for p in positive_dir.iterdir()} == {'trace.tsv', 'ledger.tsv'},
                'positive publication left unexpected files')
        trace = (positive_dir / 'trace.tsv').read_bytes()
        rows = (positive_dir / 'ledger.tsv').read_text(encoding='ascii').splitlines()
        header = rows.index('sensor\tsequence\tsample_tick\tplanned_arrival_tick\tdecision_tick\toutcome\tqueue_occupancy')
        source_count = int(next(row for row in rows if row.startswith('source_rows\t')).split('\t')[1])
        require(source_count == 27, 'unexpected source ledger count')
        source_rows = rows[header + 1:header + 1 + source_count]
        require(len(source_rows) == source_count and all(len(row.split('\t')) == 7 for row in source_rows),
                'malformed source ledger section')
        counts = Counter(row.split('\t')[5] for row in source_rows)
        require(dict(counts) == {'accepted': 6, 'stale_drop': 20, 'undelivered_at_horizon': 1},
                'one-second ledger differs from independent schedule calculation')
        receipt['observed_ledger_counts'] = dict(counts)
        prior = {p.name: sha(p) for p in positive_dir.iterdir()}
        repeated = command([binary, positive_dir], output, 'existing-directory')
        require(repeated['returncode'] != 0 and 'AlreadyExists' in repeated['stderr'],
                'existing-directory negative did not report AlreadyExists')
        require(prior == {p.name: sha(p) for p in positive_dir.iterdir()}, 'negative changed existing artifacts')
        receipt['positive'], receipt['existing_directory'] = positive, repeated
        receipt['artifact_hashes'] = prior
        receipt['artifact_bytes'] = {p.name: p.stat().st_size for p in positive_dir.iterdir()}
        if sys.platform == 'linux':
            receipt['faults'] = [fault_case(binary, scratch, output, 'trace-enospc', True, trace),
                                 fault_case(binary, scratch, output, 'ledger-enospc', False, trace)]
        else:
            receipt['faults'] = 'not run: owned tmpfs cases require native Linux; no waiver claimed'
        receipt['source_after'] = {str(p.relative_to(ROOT)): sha(p) for p in paths}
        require(before == receipt['source_after'], 'source changed during demonstration')
        receipt['success'] = True
    except BaseException as error:
        receipt['error'] = repr(error)
        raise
    finally:
        receipt['scratch_cleanup'] = False
        try:
            if sys.platform == 'linux':
                require(all(owned_mount(p) is None for p in scratch.iterdir() if p.is_dir()),
                        'owned mount still active; no recursive cleanup')
            require(scratch.parent == target_root and scratch.resolve() == scratch,
                    'resolved cleanup path changed')
            shutil.rmtree(scratch)
            receipt['scratch_cleanup'] = True
        except BaseException as cleanup_error:
            receipt['success'] = False
            receipt['cleanup_error'] = repr(cleanup_error)
            raise
        finally:
            (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({'success': receipt['success'], 'platform': receipt['platform'],
                      'faults_run': isinstance(receipt.get('faults'), list)}))
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
