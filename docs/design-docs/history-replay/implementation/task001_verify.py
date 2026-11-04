"""Mutable TASK-001 evidence runner; does not redefine the approved contract."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent / ('task001-final-evidence' if sys.argv[1] == 'final' else 'task001-evidence')
OUT.mkdir(exist_ok=True)
VALIDATOR = 'C:/Users/Admin/Documents/Codex/2026-10-14/i-x20/work/fusion-lab/.agents/skills/rust/spec-design/scripts/validate_design_package.py'
DIGEST = '8c57cf49bd99b927012ac7215ea44688bc8cde928ea6b233a2f37abc41f13095'

def hashes():
    paths = sorted((ROOT / 'src').glob('*.rs')) + [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock']
    return {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}

def run(argv, name, env=None):
    started = time.monotonic()
    path = OUT / (name + '.log')
    with path.open('wb') as log:
        result = subprocess.run(argv, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
    record = dict(argv=argv, cwd=str(ROOT), exit=result.returncode, seconds=time.monotonic()-started,
                  log=path.name, sha256=hashlib.sha256(path.read_bytes()).hexdigest())
    (OUT / (name + '.json')).write_text(json.dumps(record, indent=2))
    print(json.dumps(record), flush=True)
    return result.returncode

def validate(name):
    return run([sys.executable, VALIDATOR, 'docs/design-docs/history-replay', '--require-lock', '--baseline-digest', DIGEST], name)

if sys.argv[1] == 'before':
    (OUT / 'source-before.json').write_text(json.dumps(hashes(), indent=2))
    raise SystemExit(validate('contract-before'))

if sys.argv[1] == 'inventory':
    OUT = Path(__file__).resolve().parent / 'task001-final-evidence'
    before = json.loads((OUT.parent / 'task001-evidence/source-before.json').read_text())
    after = hashes()
    changed = [p for p in before if before[p] != after[p]]
    records = [json.loads(p.read_text()) for p in sorted(OUT.glob('*.json')) if p.name != 'source-after.json']
    assert changed == ['src\\history_replay.rs'], changed
    assert all(r['exit'] == 0 for r in records)
    for alias in ['1.99.0', 'stable']:
        run(['rustc', '+'+alias, '-Vv'], alias+'-rustc')
    (OUT / 'summary.json').write_text(json.dumps(dict(changed_source=changed, unchanged_protected_sources={p: h for p, h in after.items() if p not in changed}, final_source_sha256=after['src\\history_replay.rs'], command_records=len(records), all_exit_zero=True, rustdocflags='-D warnings', scope='TASK-001 only; replay diagnostic retained; all38 not yet green'), indent=2))
    raise SystemExit(0)

env = os.environ.copy()
env.update(RUSTDOCFLAGS='-D warnings', CARGO_TERM_COLOR='never')
selectors = ['tst_bound_001_window_zero_and_maximum_valid', 'tst_neg_001_window_501_rejected',
             'tst_neg_002_window_integer_max_rejected', 'tst_struct_001_public_types_send_sync_signature',
             'tst_anti_001_private_plan_cannot_be_forged']
codes = []
for alias in ['1.99.0', 'stable']:
    for selector in selectors:
        codes.append(run(['cargo', '+'+alias, 'test', '--manifest-path', 'tests/history-replay/Cargo.toml', '--locked', '--test', 'contracts', selector, '--', '--exact', '--nocapture', '--test-threads=1'], alias+'-'+selector, env))
gates = [['fmt', '--all', '--', '--check'], ['check', '--locked', '--all-targets'],
         ['clippy', '--locked', '--all-targets', '--', '-D', 'warnings'], ['test', '--locked'],
         ['test', '--locked', '--doc'], ['doc', '--locked', '--no-deps'],
         ['clippy', '--manifest-path', 'tests/history-replay/Cargo.toml', '--locked', '--all-targets', '--', '-D', 'warnings']]
for index, gate in enumerate(gates, 1):
    codes.append(run(['cargo', *gate], 'gate-'+str(index), env))
codes.append(validate('contract-after'))
(OUT / 'source-after.json').write_text(json.dumps(hashes(), indent=2))
print(json.dumps(dict(exit_codes=codes, all_passed=not any(codes))))
raise SystemExit(bool(any(codes)))
