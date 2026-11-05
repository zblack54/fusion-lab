"""History native resource instrumentation; build-only until exact-source review.

Reuses the immutable accepted helper for environment/memory/owned reap only.
No accepted helper workload or its older product thresholds are reused.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import tempfile
import threading
import time
import tomllib

BASELINE = '8c57cf49bd99b927012ac7215ea44688bc8cde928ea6b233a2f37abc41f13095'
HELPER_SHA = 'ab851ab12ec6f54ee52baec37b0b59b80f8db069b5a92179c3f12f23cc9b924c'
ROOT = Path(__file__).resolve().parents[4]
MIB = 1024 * 1024
WATCHDOG_SECONDS = 180
PRODUCT_SECONDS = 120
PRODUCT_BYTES = 768 * MIB
OWNER = None
DRIVER = r'''use fusion_lab::*;
use std::{
    error::Error,
    hint::black_box,
    io::{self, Write},
    mem::{size_of, size_of_val},
};

fn hbits(s: &HistorySnapshot) -> [u64; 24] {
    let mut out = [0; 24];
    for (slot, value) in out.iter_mut().zip(
        s.estimate()
            .position
            .into_iter()
            .chain(s.estimate().velocity)
            .chain(s.covariance().into_iter().flatten())
            .chain(s.held_position())
            .chain(s.integrated_position()),
    ) {
        *slot = value.to_bits();
    }
    out
}
fn dbits(s: &ReplaySnapshot) -> [u64; 24] {
    let mut out = [0; 24];
    for (slot, value) in out.iter_mut().zip(
        s.estimate()
            .position
            .into_iter()
            .chain(s.estimate().velocity)
            .chain(s.covariance().iter().flatten().copied())
            .chain(s.held_position())
            .chain(s.integrated_position()),
    ) {
        *slot = value.to_bits();
    }
    out
}
fn comparisons(a: [u64; 24], b: [u64; 24]) -> [bool; 24] {
    let result = std::array::from_fn(|i| a[i] == b[i]);
    assert!(result.into_iter().all(|equal| equal));
    result
}
fn mapped(outcome: DeliveryOutcome) -> HistoryOutcome {
    match outcome {
        DeliveryOutcome::ScriptedLoss => HistoryOutcome::ScriptedLoss,
        DeliveryOutcome::QueueFullDrop => HistoryOutcome::QueueFullDrop,
        DeliveryOutcome::Accepted => HistoryOutcome::AcceptedOnTime,
        DeliveryOutcome::StaleDrop => HistoryOutcome::TooOldDrop,
        DeliveryOutcome::UndeliveredAtHorizon => HistoryOutcome::UndeliveredAtHorizon,
    }
}
fn same_entry(a: &HistoryLedgerEntry, b: &HistoryLedgerEntry) -> bool {
    (
        a.id(),
        a.sample_tick(),
        a.planned_arrival_tick(),
        a.decision_tick(),
        a.queue_occupancy(),
        a.outcome(),
    ) == (
        b.id(),
        b.sample_tick(),
        b.planned_arrival_tick(),
        b.decision_tick(),
        b.queue_occupancy(),
        b.outcome(),
    )
}
fn work_values(w: &HistoryWork) -> (u32, Option<u32>, [usize; 8]) {
    (
        w.tick(),
        w.restored_sample_tick(),
        [
            w.prediction_calls(),
            w.measurement_calls(),
            w.committed_checkpoints(),
            w.committed_observations(),
            w.peak_live_checkpoints(),
            w.peak_live_history_observations(),
            w.reserved_checkpoint_slots(),
            w.reserved_observation_slots(),
        ],
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let profile = args.next().ok_or("expected A or C")?;
    let pdelay = match profile.as_str() {
        "A" => 499,
        "C" => 0,
        _ => return Err("expected A or C".into()),
    };
    let mode = args.next().ok_or("expected warmup or maximum")?;
    let duration = match mode.as_str() {
        "warmup" => 1,
        "maximum" => 3600,
        _ => return Err("expected warmup or maximum".into()),
    };
    let artifact_dir =
        std::path::PathBuf::from(args.next().ok_or("expected owned artifact directory")?);
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let config = Config {
        duration_seconds: duration,
        seed: 42,
        outage: None,
        ..Config::default()
    };
    assert_eq!(config.velocity_bias, [0.0; 2]);
    assert_eq!(config.velocity_drift_rate, [0.0; 2]);
    let source = run(&config)?;
    let model = ReplayModel::new(1.0, 0.15, 0.1)?;
    let trace = Trace::from_samples(
        duration,
        &source.samples,
        DeliveryPlan::new(pdelay, 500, 128, None, None)?,
    )?;
    let encoded = trace.encode_tsv(&model)?;
    let model_bytes = format!(
        "position_assumed_sigma\t{}\nvelocity_assumed_sigma\t{}\nacceleration_density\t{}\n",
        model.assumed_position_sigma(),
        model.assumed_velocity_sigma(),
        model.acceleration_density()
    )
    .into_bytes();
    let publish = |name: &str, bytes: &[u8]| -> io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(artifact_dir.join(name))?;
        file.write_all(bytes)?;
        file.flush()
    };
    publish("canonical-before.tsv", &encoded)?;
    publish("model-before.tsv", &model_bytes)?;
    let corrected = replay_history(&model, &trace, &HistoryPlan::new(500)?)?;
    let zero = replay_history(&model, &trace, &HistoryPlan::new(0)?)?;
    let legacy = replay_delivery(&model, &trace)?;
    let repeated = replay_history(&model, &trace, &HistoryPlan::new(500)?)?;
    let regenerated = trace.encode_tsv(&model)?;
    assert_eq!(encoded, regenerated);
    let regenerated_model_bytes = format!(
        "position_assumed_sigma\t{}\nvelocity_assumed_sigma\t{}\nacceleration_density\t{}\n",
        model.assumed_position_sigma(),
        model.assumed_velocity_sigma(),
        model.acceleration_density()
    )
    .into_bytes();
    assert_eq!(model_bytes, regenerated_model_bytes);
    publish("canonical-after.tsv", &regenerated)?;
    publish("model-after.tsv", &regenerated_model_bytes)?;
    let count = duration as usize * 100 + 1;
    let offered = (duration as usize * 100 / 20 + 1) + (duration as usize * 100 / 5 + 1);
    assert_eq!(source.samples.len(), count);
    assert_eq!(trace.observations().len(), offered);
    for result in [&corrected, &zero, &repeated] {
        assert_eq!(result.snapshots().len(), count);
        assert_eq!(result.work().len(), count);
        assert_eq!(result.ledger().len(), offered);
    }
    assert_eq!(legacy.snapshots().len(), count);
    assert_eq!(legacy.ledger().len(), offered);
    assert_eq!(zero.arrival_order(), legacy.arrival_order());
    assert_eq!(corrected.arrival_order(), legacy.arrival_order());
    assert_eq!(corrected.arrival_order(), repeated.arrival_order());
    let mut zero_equal = Vec::with_capacity(count);
    let mut repeat_equal = Vec::with_capacity(count);
    let mut ledger_equal = Vec::with_capacity(offered);
    let mut work_equal = Vec::with_capacity(count);
    let mut prediction_total = 0usize;
    let mut measurement_total = 0usize;
    let mut maxima = [0usize; 8];
    for index in 0..count {
        let (z, l, h, r) = (
            &zero.snapshots()[index],
            &legacy.snapshots()[index],
            &corrected.snapshots()[index],
            &repeated.snapshots()[index],
        );
        assert_eq!(
            (z.tick(), l.tick(), h.tick(), r.tick()),
            (index as u32, index as u32, index as u32, index as u32)
        );
        zero_equal.push(comparisons(hbits(z), dbits(l)));
        repeat_equal.push(comparisons(hbits(h), hbits(r)));
        let equal = work_values(&corrected.work()[index]) == work_values(&repeated.work()[index]);
        assert!(equal);
        work_equal.push(equal);
        for (result, window) in [
            (&corrected, 500usize),
            (&zero, 0usize),
            (&repeated, 500usize),
        ] {
            let w = &result.work()[index];
            assert_eq!(w.tick(), index as u32);
            assert!(w.prediction_calls() <= window + 1 && w.measurement_calls() <= 127);
            assert!(w.committed_checkpoints() <= window + 2 && w.committed_observations() <= 127);
            assert!(
                w.peak_live_checkpoints() <= 2 * (window + 2)
                    && w.peak_live_history_observations() <= 254
            );
            assert!(w.reserved_checkpoint_slots() >= w.peak_live_checkpoints());
            assert!(w.reserved_observation_slots() >= w.peak_live_history_observations());
        }
        let w = &corrected.work()[index];
        prediction_total += w.prediction_calls();
        measurement_total += w.measurement_calls();
        for (max, value) in maxima.iter_mut().zip(work_values(w).2) {
            *max = (*max).max(value);
        }
    }
    let mut outcomes = [0usize; 6];
    for index in 0..offered {
        let (h, z, l, r) = (
            &corrected.ledger()[index],
            &zero.ledger()[index],
            &legacy.ledger()[index],
            &repeated.ledger()[index],
        );
        let zero_equal = (
            z.id(),
            z.sample_tick(),
            z.planned_arrival_tick(),
            z.decision_tick(),
            z.queue_occupancy(),
            z.outcome(),
        ) == (
            l.id(),
            l.sample_tick(),
            l.planned_arrival_tick(),
            l.decision_tick(),
            l.queue_occupancy(),
            mapped(l.outcome()),
        );
        let identity_equal = (
            h.id(),
            h.sample_tick(),
            h.planned_arrival_tick(),
            h.decision_tick(),
            h.queue_occupancy(),
        ) == (
            l.id(),
            l.sample_tick(),
            l.planned_arrival_tick(),
            l.decision_tick(),
            l.queue_occupancy(),
        );
        let policy_equal = match l.outcome() {
            DeliveryOutcome::StaleDrop => h.outcome() == HistoryOutcome::CorrectedLate,
            other => h.outcome() == mapped(other),
        };
        let repeated_equal = same_entry(h, r);
        assert!(zero_equal && identity_equal && policy_equal && repeated_equal);
        ledger_equal.push([zero_equal, identity_equal, policy_equal, repeated_equal]);
        let slot = match h.outcome() {
            HistoryOutcome::ScriptedLoss => 0,
            HistoryOutcome::QueueFullDrop => 1,
            HistoryOutcome::AcceptedOnTime => 2,
            HistoryOutcome::CorrectedLate => 3,
            HistoryOutcome::TooOldDrop => 4,
            HistoryOutcome::UndeliveredAtHorizon => 5,
        };
        outcomes[slot] += 1;
    }
    assert_eq!(outcomes.iter().sum::<usize>(), offered);
    assert_eq!(outcomes[0] + outcomes[1] + outcomes[4], 0);
    // Slice byte figures are retained logical payload MINIMA, never allocator
    // totals or inaccessible private Vec capacities. Process HWM covers all.
    let logical_bytes = size_of_val(source.samples.as_slice())
        + size_of_val(trace.observations())
        + [&corrected, &zero, &repeated]
            .iter()
            .map(|r| {
                size_of_val(r.snapshots())
                    + size_of_val(r.work())
                    + size_of_val(r.ledger())
                    + size_of_val(r.arrival_order())
            })
            .sum::<usize>()
        + size_of_val(legacy.snapshots())
        + size_of_val(legacy.ledger())
        + size_of_val(legacy.arrival_order());
    let owned_capacity_bytes = (encoded.capacity()
        + regenerated.capacity()
        + model_bytes.capacity()
        + regenerated_model_bytes.capacity())
        * size_of::<u8>()
        + (zero_equal.capacity() + repeat_equal.capacity()) * size_of::<[bool; 24]>()
        + ledger_equal.capacity() * size_of::<[bool; 4]>()
        + work_equal.capacity() * size_of::<bool>()
        + source.samples.capacity() * size_of::<Sample>();
    println!(
        "COMPLETE\t{{\"schema\":1,\"profile\":\"{}\",\"duration_seconds\":{},\"source_samples\":{},\"observations\":{},\"snapshots_per_result\":{},\"canonical_bytes\":{},\"canonical_capacity\":{},\"regenerated_capacity\":{},\"zero_component_rows\":{},\"repeat_component_rows\":{},\"ledger_rows\":{},\"work_rows\":{},\"logical_payload_minimum_bytes\":{},\"owned_public_capacity_bytes\":{},\"source_sample_capacity\":{},\"zero_equality_capacity\":{},\"repeat_equality_capacity\":{},\"ledger_equality_capacity\":{},\"work_equality_capacity\":{},\"model_capacity\":{},\"regenerated_model_capacity\":{},\"prediction_total\":{},\"measurement_total\":{},\"work_maxima\":{:?},\"outcomes\":{:?},\"all_equalities_passed\":true}}",
        profile,
        duration,
        source.samples.len(),
        offered,
        count,
        encoded.len(),
        encoded.capacity(),
        regenerated.capacity(),
        zero_equal.len(),
        repeat_equal.len(),
        ledger_equal.len(),
        work_equal.len(),
        logical_bytes,
        owned_capacity_bytes,
        source.samples.capacity(),
        zero_equal.capacity(),
        repeat_equal.capacity(),
        ledger_equal.capacity(),
        work_equal.capacity(),
        model_bytes.capacity(),
        regenerated_model_bytes.capacity(),
        prediction_total,
        measurement_total,
        maxima,
        outcomes
    );
    io::stdout().flush()?;
    black_box((
        &config,
        &model,
        &source,
        &trace,
        &corrected,
        &zero,
        &legacy,
        &repeated,
        &encoded,
        &regenerated,
        &zero_equal,
        &repeat_equal,
        &ledger_equal,
        &work_equal,
        &model_bytes,
        &regenerated_model_bytes,
    ));
    let mut ack = String::new();
    io::stdin().read_line(&mut ack)?;
    if ack != "release\n" {
        return Err("missing owned release acknowledgement".into());
    }
    black_box((
        &config,
        &model,
        &source,
        &trace,
        &corrected,
        &zero,
        &legacy,
        &repeated,
        &encoded,
        &regenerated,
        &zero_equal,
        &repeat_equal,
        &ledger_equal,
        &work_equal,
        &model_bytes,
        &regenerated_model_bytes,
    ));
    Ok(())
}
'''


def sha(data):
    return hashlib.sha256(data).hexdigest()


def remaining_watchdog(deadline, phase):
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise TimeoutError(f'owned watchdog elapsed during {phase}')
    return remaining


def file_sha(path, deadline=None, expected_bytes=None):
    digest = hashlib.sha256()
    size = 0
    if deadline is not None:
        remaining_watchdog(deadline, 'held artifact hash')
    with path.open('rb') as file:
        for block in iter(lambda: file.read(65536), b''):
            if deadline is not None:
                remaining_watchdog(deadline, 'held artifact hash')
            size += len(block)
            if expected_bytes is not None and size > expected_bytes:
                raise RuntimeError(f'artifact exceeded expected bytes: {path.name}')
            digest.update(block)
    if deadline is not None:
        remaining_watchdog(deadline, 'held artifact hash')
    if expected_bytes is not None and size != expected_bytes:
        raise RuntimeError(f'artifact size mismatch: {path.name}')
    return digest.hexdigest()


def trace_artifacts(directory, complete, deadline=None):
    expected = b'position_assumed_sigma\t1\nvelocity_assumed_sigma\t0.15\nacceleration_density\t0.1\n'
    canonical_bytes = complete['canonical_bytes']
    if not 0 < canonical_bytes <= 32*MIB:
        raise RuntimeError('canonical artifact size exceeds encoder bound')
    sizes = {'canonical-before.tsv':canonical_bytes,'canonical-after.tsv':canonical_bytes,
             'model-before.tsv':len(expected),'model-after.tsv':len(expected)}
    result = {name: dict(sha256=file_sha(directory/name, deadline, size),bytes=size)
              for name,size in sizes.items()}
    if result['canonical-before.tsv'] != result['canonical-after.tsv']:
        raise RuntimeError('actual canonical before/after artifact SHA or byte count mismatch')
    if result['model-before.tsv'] != result['model-after.tsv']:
        raise RuntimeError('actual model before/after artifact SHA mismatch')
    if result['model-before.tsv']['sha256'] != sha(expected):
        raise RuntimeError('wrong actual model assumptions')
    if deadline is not None:
        remaining_watchdog(deadline, 'held artifact hash')
    return result


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2)+'\n', encoding='utf-8', newline='\n')


def load_owner(repo):
    global OWNER
    path = repo / 'docs/design-docs/observation-delivery/implementation/resource_probe.py'
    if sha(path.read_bytes()) != HELPER_SHA:
        raise RuntimeError('accepted owner helper SHA mismatch')
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location('accepted_history_owner', path)
    OWNER = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(OWNER)
    return OWNER


def acquire(repo, output):
    owner = load_owner(repo)
    owner.OWNER_GUARD = repo / 'docs/design-docs/history-replay/implementation/.resource-probe-owner.json'
    owner.OWNER_TOKEN = os.urandom(16).hex()
    with owner.OWNER_GUARD.open('x', encoding='utf-8') as file:
        json.dump(dict(token=owner.OWNER_TOKEN, owner_pid=os.getpid(), status='owned_preparing', output=str(output), baseline_digest=BASELINE), file)
    return owner


def release():
    state = json.loads(OWNER.OWNER_GUARD.read_text())
    if state.get('compiler_descendants_unverified') or state.get('child_pid') is not None:
        OWNER.owner_update(status='manual_reconciliation_required')
        return False
    OWNER.owner_update(status='finished_direct_children_reaped', process_tree_cleanup_guaranteed=False)
    OWNER.OWNER_GUARD.unlink()
    return True


def inventory(repo):
    paths = [repo/'Cargo.toml', repo/'Cargo.lock', repo/'rust-toolchain.toml']
    paths += sorted((repo/'src').glob('*.rs')) + [repo/'examples/history_replay.rs']
    paths += sorted(p for p in (repo/'tests/history-replay').rglob('*') if p.is_file() and not any(x in p.parts for x in ('target','__pycache__','.git')))
    package = repo/'docs/design-docs/history-replay'
    paths += sorted(p for p in package.rglob('*') if p.is_file() and p.relative_to(package).parts[0] not in ('implementation','validation'))
    paths += [package/'validation/test-lock.json', package/'validation/design-execution.json', repo/'docs/design-docs/observation-delivery/implementation/resource_probe.py']
    return {p.relative_to(repo).as_posix(): sha(p.read_bytes()) for p in paths}


def closure(data):
    return [p for p in tomllib.loads(data.decode('utf-8'))['package'] if 'source' in p]


def owned_command(argv, repo, output, timeout=240):
    """Compiler errors retain the fence even if the direct child was reaped.

    No process-tree containment is claimed. An exceptional compiler may have
    descendants, so no later job is allowed until main reconciles the fence.
    """
    record = dict(argv=argv, cwd=str(repo), timeout_seconds=timeout)
    started = time.monotonic()
    child = None
    with tempfile.TemporaryFile() as raw:
        try:
            child = subprocess.Popen(argv, cwd=repo, stdout=raw, stderr=subprocess.STDOUT)
            record['pid'] = child.pid
            OWNER.owner_update(status='command_running', child_pid=child.pid, child_argv=argv)
            while child.poll() is None:
                if time.monotonic()-started > timeout:
                    raise TimeoutError('owned command watchdog')
                if os.fstat(raw.fileno()).st_size > 8*MIB:
                    raise RuntimeError('command output cap')
                time.sleep(.02)
            record['exit'] = child.wait(timeout=1)
            record['reap_confirmed'] = True
            if record['exit']:
                raise RuntimeError('owned command failed')
            OWNER.owner_update(status='command_reaped', child_pid=None)
        except BaseException as error:
            record['error'] = repr(error)
            if child is not None:
                OWNER.reap_owned(child, record, output.with_suffix('.failure.json'))
                OWNER.owner_update(status='manual_reconciliation_required', compiler_descendants_unverified=True, child_pid=child.pid)
                record['compiler_descendants_unverified'] = True
            raise
        finally:
            raw.seek(0)
            data = raw.read(8*MIB+1)
            output.write_bytes(data)
            record.update(seconds=time.monotonic()-started, log_sha256=sha(data), log_bytes=len(data))
            write_json(output.with_suffix('.json'), record)
    return record


def validate_complete(line, expected):
    if len(line) > 16384 or not line.startswith(b'COMPLETE\t') or not line.endswith(b'\n'):
        raise RuntimeError('missing complete-work protocol')
    value = json.loads(line[len(b'COMPLETE\t'):])
    if value.get('schema') != 1 or value.get('profile') != expected['profile']:
        raise RuntimeError('wrong completion schema/profile')
    duration = expected['duration_seconds']
    count = duration*100+1
    offered = duration*100//20+1+duration*100//5+1
    for name, actual in [('duration_seconds',duration),('source_samples',count),('observations',offered),('snapshots_per_result',count),('zero_component_rows',count),('repeat_component_rows',count),('ledger_rows',offered),('work_rows',count)]:
        if value.get(name) != actual:
            raise RuntimeError('wrong complete-work inventory: '+name)
    if value.get('all_equalities_passed') is not True:
        raise RuntimeError('equality proof absent')
    horizon = duration*100
    pdelay = 499 if expected['profile'] == 'A' else 0
    position_arrivals = (horizon-pdelay)//20+1 if horizon >= pdelay else 0
    velocity_arrivals = (horizon-500)//5+1 if horizon >= 500 else 0
    on_time = position_arrivals if pdelay == 0 else 0
    corrected = position_arrivals+velocity_arrivals-on_time
    if value.get('outcomes') != [0,0,on_time,corrected,0,offered-on_time-corrected]:
        raise RuntimeError('wrong analytical admission/outcome counts')
    maxima = value.get('work_maxima')
    if not isinstance(maxima,list) or len(maxima) != 8 or any(type(x) is not int or x < 0 for x in maxima):
        raise RuntimeError('missing actual work/storage maxima')
    if any(actual > bound for actual,bound in zip(maxima[:6],[501,127,502,127,1004,254])) or maxima[6] < maxima[4] or maxima[7] < maxima[5]:
        raise RuntimeError('work/storage maxima violate declared bounds')
    for name in ['canonical_bytes','canonical_capacity','regenerated_capacity','logical_payload_minimum_bytes','owned_public_capacity_bytes','prediction_total','measurement_total']:
        if type(value.get(name)) is not int or value[name] < 0:
            raise RuntimeError('missing actual allocation/call inventory: '+name)
    for name,minimum in [('source_sample_capacity',count),('zero_equality_capacity',count),('repeat_equality_capacity',count),('ledger_equality_capacity',offered),('work_equality_capacity',count)]:
        if type(value.get(name)) is not int or value[name] < minimum:
            raise RuntimeError('wrong actual owned capacity: '+name)
    return value


def measure_owned(argv, expected, output, popen_factory=subprocess.Popen, reader_factory=None, after_spawn=None):
    """Same child/handle is held until actual reap, including injected faults."""
    artifacts = None
    if expected.get('numerical',True):
        artifacts = output.with_suffix('.artifacts')
        artifacts.mkdir()
        argv = [*argv,str(artifacts)]
    record = dict(argv=argv, workload=expected, watchdog_seconds=WATCHDOG_SECONDS,
                  product_seconds=PRODUCT_SECONDS, product_bytes=PRODUCT_BYTES,
                  classification='NUMERICAL_RESOURCE_CANDIDATE' if expected.get('numerical',True) else 'NONNUMERICAL_CLEANUP_PROOF')
    started = time.monotonic()
    deadline = started + WATCHDOG_SECONDS
    child = None
    close = lambda: None
    stdout_lines = queue.Queue()
    stderr = bytearray()
    stderr_thread = None
    stdout_thread = None
    try:
        child = popen_factory(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        record['pid'] = child.pid
        OWNER.owner_update(status='child_running', child_pid=child.pid, child_argv=argv)
        read, close = (reader_factory or OWNER.memory_reader)(child.pid)
        if after_spawn is not None:
            after_spawn(child, record)
        def read_stdout():
            stdout_lines.put(child.stdout.readline(16385))
        def read_stderr():
            while True:
                block = child.stderr.read(4096)
                if not block:
                    return
                if len(stderr)+len(block) > 65536:
                    stderr.extend(b'[stderr capped]')
                    return
                stderr.extend(block)
        stdout_thread = threading.Thread(target=read_stdout, daemon=True)
        stderr_thread = threading.Thread(target=read_stderr, daemon=True)
        stdout_thread.start(); stderr_thread.start()
        phase = 'waiting for COMPLETE'
        line = stdout_lines.get(timeout=remaining_watchdog(deadline, phase))
        remaining_watchdog(deadline, phase)
        record['seconds_spawn_to_complete_work'] = time.monotonic()-started
        record['complete_line'] = line.decode('utf-8', errors='replace')
        record['complete'] = validate_complete(line, expected)
        if child.poll() is not None:
            raise RuntimeError('child exited before held acknowledgement')
        phase = 'held memory read'
        record['held_complete_work_memory'] = read()
        remaining_watchdog(deadline, phase)
        if artifacts is not None:
            phase = 'held artifact hash'
            hash_started = time.monotonic()
            record['held_hash_deadline_method'] = 'cooperative checks around synchronous reads; an OS call can return after the deadline'
            try:
                record['artifacts_at_held_complete_work'] = trace_artifacts(artifacts,record['complete'],deadline)
            finally:
                hash_finished = time.monotonic()
                record['seconds_held_artifact_hash'] = hash_finished-hash_started
                record['held_hash_watchdog_overshoot_seconds'] = max(0.0,hash_finished-deadline)
        record['retained_phase'] = 'all source/trace/results/diagnostics/canonical/equality buffers live awaiting owner ACK'
        phase = 'releasing held child'
        remaining_watchdog(deadline, phase)
        child.stdin.write(b'release\n'); child.stdin.flush(); child.stdin.close()
        phase = 'waiting for released child'
        record['exit'] = child.wait(timeout=remaining_watchdog(deadline, phase))
        record['reap_confirmed'] = True
        OWNER.owner_update(status='child_reaped', child_pid=None)
        remaining_watchdog(deadline, phase)
        if os.name == 'nt':
            record['post_exit_retained_handle_memory'] = read()
        else:
            record['teardown_limitation'] = 'Linux VmHWM read at held complete work; teardown after ACK not observable in /proc after reap'
        record['lifetime_peak_bytes'] = max(record['held_complete_work_memory']['lifetime_peak_bytes'],record.get('post_exit_retained_handle_memory',{}).get('lifetime_peak_bytes',0))
        record['seconds_total_including_hold_and_teardown'] = time.monotonic()-started
        if artifacts is not None:
            record['artifacts_after_reap'] = trace_artifacts(artifacts,record['complete'])
            if record['artifacts_after_reap'] != record['artifacts_at_held_complete_work']:
                raise RuntimeError('canonical/model artifacts changed across held acknowledgement')
        if record['exit'] != 0:
            raise RuntimeError('driver failed after owned release')
        record['within_120_seconds'] = record['seconds_spawn_to_complete_work'] <= PRODUCT_SECONDS
        record['within_768_MiB'] = record['lifetime_peak_bytes'] <= PRODUCT_BYTES
        record['success'] = record['within_120_seconds'] and record['within_768_MiB']
    except BaseException as error:
        record.update(success=False, error=repr(error))
        if child is not None:
            OWNER.reap_owned(child, record, output)
            OWNER.owner_update(status='child_reaped_after_error', child_pid=None)
    finally:
        # reap_owned cannot return while the same child remains uncertain.
        close()
        if stdout_thread is not None: stdout_thread.join(timeout=1)
        if stderr_thread is not None: stderr_thread.join(timeout=1)
        if child is not None:
            for pipe in (child.stdin, child.stdout, child.stderr):
                if pipe is not None and not pipe.closed:
                    pipe.close()
        record.update(stderr=bytes(stderr).decode('utf-8',errors='replace'), seconds_owner_total=time.monotonic()-started)
        write_json(output,record)
    return record


def generated_inventory(project):
    return {name: dict(sha256=sha((project/name).read_bytes()),bytes=(project/name).stat().st_size)
            for name in ['Cargo.toml','Cargo.lock','src/main.rs']}


def prepare(repo, output):
    project = output/'project'
    (project/'src').mkdir(parents=True)
    manifest = '[package]\nname="history-replay-resource-probe"\nversion="0.0.0"\nedition="2024"\nrust-version="1.99"\npublish=false\n[workspace]\nresolver="3"\n[dependencies]\nfusion-lab={path='+json.dumps(repo.as_posix())+'}\n[lints.rust]\nunsafe_code="forbid"\n'
    lock = (repo/'tests/history-replay/Cargo.lock').read_bytes().replace(b'history-replay-spec-tests',b'history-replay-resource-probe')
    root_closure = closure((repo/'Cargo.lock').read_bytes())
    if root_closure != closure(lock):
        raise RuntimeError('generated registry closure differs from exact root lock')
    for name,text in [('Cargo.toml',manifest),('src/main.rs',DRIVER)]:
        actual = text.encode('utf-8')
        if b'\r' in actual: raise RuntimeError('generated logical source contains CR')
        (project/name).write_bytes(actual)
        if (project/name).read_bytes() != actual: raise RuntimeError('generated UTF8/LF readback mismatch')
    (project/'Cargo.lock').write_bytes(lock)
    return project, dict(registry_entries=root_closure, registry_json_sha256=sha(json.dumps(root_closure,sort_keys=True).encode('utf-8')),
                         root_lock_sha256=sha((repo/'Cargo.lock').read_bytes()), generated_lock_sha256=sha(lock))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo',type=Path,default=ROOT)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--toolchain',choices=['1.99.0','stable'],required=True)
    parser.add_argument('--run-series',action='store_true',help='requires prior main review; one warmup then three maximum runs per profile')
    args = parser.parse_args()
    repo,output = args.repo.resolve(),args.output.resolve()
    if output == repo or repo in output.parents: raise RuntimeError('generated build/output must stay outside repository')
    if output.exists(): raise RuntimeError('output must be fresh; never overwrite or rerun failed evidence')
    acquire(repo,output)
    output.mkdir(parents=True)
    receipt = dict(schema=1,stage='RESOURCE_SERIES_CANDIDATE' if args.run_series else 'BUILD_ONLY_NOT_RESOURCE_ACCEPTANCE',
                   success=False, baseline_digest=BASELINE, sampler_sha256=sha(Path(__file__).read_bytes()),
                   owner_helper_sha256=HELPER_SHA,
                   product_seconds=PRODUCT_SECONDS,product_bytes=PRODUCT_BYTES,watchdog_seconds=WATCHDOG_SECONDS,
                   commands=[],runs=[],inventory_limit='slice lengths*sizeof are retained logical payload minima; explicit owned Vec capacities are exact; neither sum is allocator-total; whole-process HWM includes allocator overhead')
    try:
        receipt['environment'] = OWNER.environment()
        receipt['source_before'] = inventory(repo)
        project,receipt['closure_before'] = prepare(repo,output)
        receipt['generated_before'] = generated_inventory(project)
        receipt['driver_logical_utf8_lf_sha256'] = sha(DRIVER.encode('utf-8'))
        for name,argv in [('compiler',['rustc','+'+args.toolchain,'-Vv']),('format',['rustfmt','+'+args.toolchain,'--edition','2024','--check',str(project/'src/main.rs')]),('build',['cargo','+'+args.toolchain,'build','--release','--locked','--offline','--manifest-path',str(project/'Cargo.toml')])]:
            receipt['commands'].append(owned_command(argv,repo,output/(name+'.log')))
        receipt['generated_after_build'] = generated_inventory(project)
        if receipt['generated_after_build'] != receipt['generated_before']: raise RuntimeError('generated inputs changed during build')
        binary = project/'target/release'/('history-replay-resource-probe.exe' if os.name=='nt' else 'history-replay-resource-probe')
        receipt['binary'] = dict(path=str(binary),sha256=sha(binary.read_bytes()),bytes=binary.stat().st_size)
        if args.run_series:
            for profile in ['A','C']:
                for mode,number in [('warmup',0),('maximum',1),('maximum',2),('maximum',3)]:
                    binary_before = file_sha(binary)
                    result = measure_owned([str(binary),profile,mode],dict(profile=profile,duration_seconds=1 if mode=='warmup' else 3600),output/(profile+'-'+mode+'-'+str(number)+'.json'))
                    result['binary_sha256_before'] = binary_before
                    result['binary_sha256_after'] = file_sha(binary)
                    if binary_before != receipt['binary']['sha256'] or result['binary_sha256_after'] != binary_before:
                        result.update(success=False,error='binary changed across workload')
                    write_json(output/(profile+'-'+mode+'-'+str(number)+'.json'),result)
                    receipt['runs'].append(result)
                    if not result['success']: raise RuntimeError('resource process failed; no retry or next workload')
        receipt['source_after'] = inventory(repo)
        receipt['generated_after'] = generated_inventory(project)
        receipt['closure_after'] = dict(root=closure((repo/'Cargo.lock').read_bytes()),generated=closure((project/'Cargo.lock').read_bytes()))
        if receipt['source_before'] != receipt['source_after'] or receipt['generated_before'] != receipt['generated_after']: raise RuntimeError('source/generated fingerprint changed')
        if receipt['closure_after']['root'] != receipt['closure_before']['registry_entries'] or receipt['closure_after']['generated'] != receipt['closure_before']['registry_entries']: raise RuntimeError('registry closure changed')
        receipt['success'] = True
    except BaseException as error:
        receipt['error'] = repr(error)
    finally:
        receipt['fence_released'] = release()
        write_json(output/'receipt.json',receipt)
    print(json.dumps(dict(success=receipt['success'],stage=receipt['stage'],receipt=str(output/'receipt.json'),fence_released=receipt['fence_released'])))
    return 0 if receipt['success'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
