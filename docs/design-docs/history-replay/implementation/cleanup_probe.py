"""Actual small-child ownership fault proof, never a numerical resource run."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import time

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('history_sampler', Path(__file__).with_name('resource_probe.py'))
SAMPLER = importlib.util.module_from_spec(spec)
spec.loader.exec_module(SAMPLER)


class FaultChild:
    def __init__(self, child, persistent):
        self.child = child
        self.pid = child.pid
        self.stdin, self.stdout, self.stderr = child.stdin, child.stdout, child.stderr
        self.persistent = persistent
        self.recovered = False
        self.counts = dict(poll=0, kill=0, wait=0)

    def fault(self, name):
        self.counts[name] += 1
        if not self.recovered and (self.persistent or self.counts[name] == 1):
            raise OSError('injected '+name+' failure')

    def poll(self):
        self.fault('poll')
        return self.child.poll()

    def kill(self):
        self.fault('kill')
        return self.child.kill()

    def wait(self, timeout=None):
        self.fault('wait')
        return self.child.wait(timeout=timeout)


def proof(output, persistent):
    holder = {}
    errors = []
    observed = {}
    def spawn(*args, **kwargs):
        proxy = FaultChild(subprocess.Popen(*args, **kwargs), persistent)
        holder['proxy'] = proxy
        return proxy
    def reader(pid):
        read, close = SAMPLER.OWNER.memory_reader(pid)
        def close_after_reap():
            try:
                assert holder['proxy'].child.poll() is not None, 'memory handle closed before actual reap'
                observed['memory_handle_closed_after_reap'] = True
            finally:
                close()
        return read, close_after_reap
    def observer():
        try:
            deadline = time.monotonic()+15
            marker = output.with_name(output.stem+'.pending.json')
            while time.monotonic() < deadline:
                try:
                    guard = json.loads(SAMPLER.OWNER.OWNER_GUARD.read_text())
                    pending = json.loads(marker.read_text()) if guard.get('status') == 'owned_pending' else {}
                except (FileNotFoundError, json.JSONDecodeError):
                    pending = {}
                if pending.get('cleanup_status') == 'owned_pending':
                    proxy = holder['proxy']
                    assert proxy.child.poll() is None
                    assert pending['cleanup_owned_pid'] == proxy.pid == guard['child_pid']
                    assert pending['original_error_preserved'] == "RuntimeError('original injected workload error')"
                    assert not observed.get('memory_handle_closed_after_reap')
                    try:
                        with SAMPLER.OWNER.OWNER_GUARD.open('x'):
                            raise AssertionError('exclusive fence did not prevent next job')
                    except FileExistsError:
                        observed['next_job_blocked_by_exclusive_fence'] = True
                    observed['durable_pending_same_child'] = pending
                    proxy.recovered = True
                    return
                time.sleep(.02)
            raise RuntimeError('persistent pending marker not observed')
        except BaseException as error:
            errors.append(repr(error))
            if 'proxy' in holder:
                # Restore real operations so a failed proof still reaps its child.
                holder['proxy'].recovered = True
    controller = threading.Thread(target=observer) if persistent else None
    if controller is not None: controller.start()
    def fail(child, record):
        raise RuntimeError('original injected workload error')
    record = SAMPLER.measure_owned([sys.executable,'-c','import time; time.sleep(60)'],
        dict(profile='NONNUMERICAL',duration_seconds=1,numerical=False),output,
        popen_factory=spawn,reader_factory=reader,after_spawn=fail)
    if controller is not None: controller.join(timeout=20)
    assert not errors, errors
    assert controller is None or not controller.is_alive()
    proxy = holder['proxy']
    assert proxy.child.poll() is not None and record['reap_confirmed']
    assert record['error'] == "RuntimeError('original injected workload error')"
    assert record['original_error_preserved'] == record['error']
    assert record['cleanup_owned_pid'] == proxy.pid
    assert record['same_child_and_memory_handle_retained']
    assert all(proxy.counts[n] >= 2 for n in ['poll','kill','wait'])
    assert observed['memory_handle_closed_after_reap']
    return dict(passed=True, classification='NONNUMERICAL_CLEANUP_PROOF_NOT_RESOURCE_MEASUREMENT',
                persistent=persistent,actual_child_pid=proxy.pid, injected_operation_counts=proxy.counts,
                observations=observed, original_error=record['error'],reap_confirmed=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if output == SAMPLER.ROOT or SAMPLER.ROOT in output.parents or output.exists():
        raise RuntimeError('fresh external output required')
    SAMPLER.acquire(SAMPLER.ROOT,output)
    output.mkdir(parents=True)
    receipt = dict(success=False,classification='NONNUMERICAL_CLEANUP_PROOF',
                   sampler_sha256=SAMPLER.sha(Path(SAMPLER.__file__).read_bytes()),
                   cleanup_probe_sha256=SAMPLER.sha(Path(__file__).read_bytes()),owner_helper_sha256=SAMPLER.HELPER_SHA)
    try:
        receipt['transient'] = proof(output/'transient.json',False)
        receipt['persistent'] = proof(output/'persistent.json',True)
        # The command wrapper must preserve its original failure and retain the
        # compiler-descendant reconciliation fence even after direct-child reap.
        try:
            SAMPLER.owned_command([sys.executable,'-c','raise SystemExit(7)'],SAMPLER.ROOT,output/'command-failure.log')
            raise AssertionError('expected command failure')
        except RuntimeError as error:
            receipt['command_original_error'] = repr(error)
        guard = json.loads(SAMPLER.OWNER.OWNER_GUARD.read_text())
        assert guard['compiler_descendants_unverified']
        assert SAMPLER.release() is False and SAMPLER.OWNER.OWNER_GUARD.exists()
        receipt['command_failure_retained_fence'] = guard
        command = json.loads((output/'command-failure.json').read_text())
        assert command['reap_confirmed'] and command['exit'] == 7
        # This explicitly identified proof child executes only SystemExit(7),
        # creates no descendants, and is already actually reaped. Reconcile
        # only this known fixture, never a compiler tree or production failure.
        SAMPLER.OWNER.owner_update(status='proof_fixture_reconciled_known_no_descendants',compiler_descendants_unverified=False,child_pid=None)
        receipt['explicit_fixture_reconciliation'] = 'known SystemExit-only child; confirmed exit7/reap; no compiler executed'
        receipt['success'] = True
    except BaseException as error:
        receipt['error'] = repr(error)
    finally:
        receipt['fence_released'] = SAMPLER.release()
        SAMPLER.write_json(output/'receipt.json',receipt)
    print(json.dumps(dict(success=receipt['success'],receipt=str(output/'receipt.json'),fence_released=receipt['fence_released'])))
    return 0 if receipt['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
