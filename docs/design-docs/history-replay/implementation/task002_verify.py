"""Record TASK-002 local evidence without changing approved test inputs."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent / 'task002-evidence'
OUT.mkdir(exist_ok=True)
TRUSTED = Path('C:/Users/Admin/Documents/Codex/2026-10-14/i-x20/work/fusion-lab/.agents/skills/rust/spec-design/scripts')
DIGEST = '8c57cf49bd99b927012ac7215ea44688bc8cde928ea6b233a2f37abc41f13095'
env = os.environ.copy()
env.update(RUSTDOCFLAGS='-D warnings', CARGO_TERM_COLOR='never')

def hashes():
    paths = sorted((ROOT / 'src').glob('*.rs')) + [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock']
    return {p.relative_to(ROOT).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}

def run(argv, name):
    path = OUT / (name + '.log')
    if path.exists():
        raise RuntimeError('refusing to overwrite evidence: '+str(path))
    started = time.monotonic()
    with path.open('wb') as log:
        result = subprocess.run(argv, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
    record = dict(argv=argv, cwd=str(ROOT), exit=result.returncode, seconds=time.monotonic()-started,
                  rustdocflags=env['RUSTDOCFLAGS'], log=path.name, sha256=hashlib.sha256(path.read_bytes()).hexdigest())
    (OUT / (name + '.json')).write_text(json.dumps(record, indent=2))
    print(json.dumps(record), flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)

before = hashes()
(OUT / 'source-before-verification.json').write_text(json.dumps(before, indent=2))
validator = [sys.executable, str(TRUSTED / 'validate_design_package.py'), 'docs/design-docs/history-replay', '--require-lock', '--baseline-digest', DIGEST]
run(validator, 'contract-before-verification')
for selector in ['tst_acc_001_late_velocity_rational_oracle', 'tst_perf_001_exact_maximum_correction_batch',
                 'tst_neg_004_rewind_overflow_preserves_arrival_and_replay_tick', 'tst_beh_006_zero_window_signed_zero_legacy_bits']:
    run(['cargo', 'test', '--manifest-path', 'tests/history-replay/Cargo.toml', '--locked', '--test', 'contracts', selector, '--', '--exact', '--nocapture', '--test-threads=1'], selector)
run([sys.executable, str(TRUSTED / 'verify_test_suite.py'), 'docs/design-docs/history-replay', '--mode', 'implementation', '--baseline-digest', DIGEST, '--offline'], 'full-implementation-verifier')
run(validator, 'contract-after-verification')
after = hashes()
(OUT / 'source-after-verification.json').write_text(json.dumps(after, indent=2))
assert before == after, 'source changed during verification'
baseline = json.loads((OUT.parent / 'task001-final-evidence/source-after.json').read_text())
baseline = {p.replace('\\', '/'): h for p, h in baseline.items()}
changed = [p for p in baseline if baseline[p] != after[p]]
assert changed == ['src/history_replay.rs'], changed
execution = json.loads((OUT.parent / 'execution.json').read_text())
assert execution['success'] and len(execution['tests']) == 76
assert all(t['status'] == 'pass' for t in execution['tests'])
assert len(execution['checks']) == 7 and all(c['status'] == 'pass' for c in execution['checks'])
(OUT / 'summary.json').write_text(json.dumps(dict(changed_production=changed, source_sha256=after['src/history_replay.rs'],
    immutable_baseline_digest=DIGEST, tests=len(execution['tests']), checks=execution['checks'],
    protected_core_sha256={p: h for p, h in after.items() if p not in changed},
    verification_source_unchanged=True, success=True,
    limitation='Local Windows TASK-002 only; native matrix/resource/example acceptance remains outstanding.'), indent=2))
print('TASK-002 local verification passed', flush=True)
