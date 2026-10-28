"""Independent golden arithmetic and synthetic boundary fixtures; no study runs."""
import json
import math
from pathlib import Path
import tempfile
import unittest
from unittest import mock
from types import SimpleNamespace
import study

class GoldenArithmetic(unittest.TestCase):
    def test_correlated_covariance_and_invalid_covariance(self):
        row=dict(estimated_x=2.,estimated_y=1.,true_x=0.,true_y=0.,p00=4.,p01=1.,p10=1.,p11=2.)
        covered,within,distance,error=study.classified(row)
        self.assertEqual(distance,8/7)
        self.assertTrue(covered);self.assertFalse(within)
        self.assertEqual(error,math.sqrt(5))
        for changes in ({'p10':1.01},{'p00':0.},{'p11':.25}):
            with self.assertRaises(ValueError):study.classified(dict(row,**changes))
    def test_recovery_window_and_censoring(self):
        values=[False]*3001;values[2000:2100]=[True]*100
        self.assertIsNone(study.recovery(values))
        values[2100]=True;self.assertEqual(study.recovery(values),2000)
        values=[False]*3001;values[2900:3001]=[True]*101
        self.assertEqual(study.recovery(values),2900)
        values[2900]=False;self.assertIsNone(study.recovery(values))
    def test_seed_unit_statistics_and_mcse(self):
        result=study.statistics_for(list(range(1,33)))
        self.assertEqual(result['mean'],16.5)
        self.assertAlmostEqual(result['sample_sd']**2,88.)
        self.assertAlmostEqual(result['sample_sd']/math.sqrt(32),math.sqrt(2.75))
        records={}
        for key in study.KEYS:
            m={'samples':1000,'filter_rmse':float(key[0]-999)+key[2], 'hold_rmse':2.,'integration_rmse':3.,'position_coverage':.5}
            records[study.key_text(key)]={'key':list(key),'summary':{'outage':m,'overall':dict(m,samples=3001),'recovery_tick':None if key[0]%2 else 2000}}
        groups=study.aggregate(records)
        self.assertEqual(len(groups),18)
        self.assertEqual(groups[0]['paired_outage_rmse_vs_scale1']['mean'],-1.)
        self.assertEqual(groups[0]['paired_outage_rmse_vs_scale1']['sample_sd'],0.)
        self.assertEqual(groups[0]['paired_outage_rmse_vs_scale1']['monte_carlo_standard_error'],0.)
        self.assertEqual(groups[0]['recovery']['censored'],16)
        self.assertEqual(groups[0]['recovery']['recovered'],16)
        with self.assertRaises(ValueError):study.aggregate({})
    def test_inventory_and_duplicate_json_rejection(self):
        self.assertEqual(len(study.KEYS),576)
        self.assertEqual(len(set(study.KEYS)),576)
        with self.assertRaises(ValueError):study.check_inventory({'unexpected':{'key':[1]} })
        with self.assertRaises(ValueError):study.check_inventory({},True)
        with self.assertRaises(ValueError):study.check_inventory({'1000-straight-0-0':{'key':[1001,'straight',0,0]}})
        with self.assertRaises(ValueError):json.loads('{"key":1,"key":2}',object_pairs_hook=study.unique_object)
    def test_exact_thresholds_and_rounding_boundaries(self):
        row=dict(estimated_x=1.,estimated_y=0.,true_x=0.,true_y=0.,p00=1.,p01=0.,p10=0.,p11=1.)
        self.assertTrue(study.classified(row)[1])
        self.assertFalse(study.classified(dict(row,estimated_x=math.nextafter(1.,math.inf)))[1])
        # Correlated quadratic is required; diagonal-only gives1.5 instead of8/7.
        self.assertTrue(study.close(1.,1.+1e-8))
        self.assertFalse(study.close(1.,1.001))
    def test_synthetic_csv_roundtrip_and_config_validation(self):
        key=(1000,'straight',0,0)
        with tempfile.TemporaryDirectory() as temporary:
            stem=Path(temporary)/'synthetic'
            for exact in (False,True):
                with Path(str(stem)+('.roundtrip.csv' if exact else '.csv')).open('w',newline='') as stream:
                    writer=study.csv.DictWriter(stream,fieldnames=study.COLUMNS+(study.EXTRA if exact else []))
                    writer.writeheader()
                    for tick in range(3001):
                        row={field:0 for field in study.COLUMNS};row.update(tick=tick,time_s=tick/100,position_outage=str(1000<=tick<2000).lower())
                        for r in range(4):row['p%d%d'%(r,r)]=1
                        for fields,present in ((('position_x','position_y'),tick%20==0 and not 1000<=tick<2000),(('velocity_x','velocity_y'),tick%5==0)):
                            for field in fields:row[field]=0 if present else ''
                        if exact:row.update(source_covered='true',source_within_recovery='true')
                        writer.writerow(row)
            summary=Path(str(stem)+'.summary.tsv')
            summary.write_text('key\t1000\tstraight\t0\t0\nconfig\t30\t2\t0.3\t0.1\t10\t20\t1\t0.15\t0\t0\noverall\t3001\t0\t0\t0\t1\noutage\t1000\t0\t0\t0\t1\nrecovery\t2000\n')
            verified,rows=study.verify(stem,key)
            self.assertEqual(verified['summary']['recovery_tick'],2000)
            self.assertEqual(verified['rounding_boundary_rows'],[])
            self.assertEqual(len(rows),3001)
            original=summary.read_text()
            summary.write_text(original.replace('overall\t3001\t0\t0\t0\t1','overall\t3001\t0\t0\t0\t0.999999999'))
            with self.assertRaisesRegex(ValueError,'coverage'):study.verify(stem,key)
            summary.write_text(original)
            with self.assertRaises(ValueError):study.read_summary(summary,(1001,'straight',0,0))
            reference=[];study.invariants(rows,reference,key)
            study.invariants(rows,reference,key)
            rows[0]['true_x']=1
            with self.assertRaises(ValueError):study.invariants(rows,reference,key)
            # A rounded CSV crosses the operational1m boundary, but exact source
            # samples certify recovery2001 rather than rounded2000.
            epsilon=1e-13
            for suffix,exact in (('.csv',False),('.roundtrip.csv',True)):
                path=Path(str(stem)+suffix)
                with path.open(newline='') as stream:
                    reader=study.csv.DictReader(stream);fields=reader.fieldnames;records=list(reader)
                records[2000]['estimated_x']=repr(1.+epsilon) if exact else '1.000000000000'
                if exact:records[2000]['source_within_recovery']='false'
                with path.open('w',newline='') as stream:
                    writer=study.csv.DictWriter(stream,fieldnames=fields);writer.writeheader();writer.writerows(records)
            rmse=math.sqrt((1.+epsilon)**2/3001)
            summary.write_text(original.replace('overall\t3001\t0\t0\t0\t1',f'overall\t3001\t{rmse}\t0\t0\t1').replace('recovery\t2000','recovery\t2001'))
            resolved,_=study.verify(stem,key)
            self.assertEqual(resolved['rounded_recovery_tick'],2000)
            self.assertEqual(resolved['summary']['recovery_tick'],2001)
            self.assertEqual([r['tick'] for r in resolved['rounding_boundary_rows']],[2000])


class ExecutionGuards(unittest.TestCase):
    def test_freeze_mismatch_rejects_without_invocation(self):
        with tempfile.TemporaryDirectory() as temporary:
            args=SimpleNamespace(output=Path(temporary),binary=Path(temporary)/'never-run',freeze=True,execute=False,retry_key=None)
            with mock.patch.object(study,'fingerprints',return_value={'source':'original'}):study.execute(args)
            args.freeze=False;args.execute=True
            with mock.patch.object(study,'fingerprints',return_value={'source':'changed'}),mock.patch.object(study,'bounded_run') as run:
                with self.assertRaisesRegex(ValueError,'freeze mismatch'):study.execute(args)
                run.assert_not_called()
    def test_checkpoint_requires_ordered_prefix_and_stops_after_failure(self):
        first=study.KEYS[0];second=study.KEYS[1]
        failed={'key':list(first),'status':'failed'}
        study.check_inventory({study.key_text(first):failed})
        with self.assertRaises(ValueError):study.check_inventory({study.key_text(second):dict(failed,key=list(second))})
        with self.assertRaises(ValueError):study.check_inventory({study.key_text(first):failed,study.key_text(second):dict(failed,key=list(second))})

    def test_bounded_child_timeout_and_artifact_cap(self):
        with tempfile.TemporaryDirectory() as temporary:
            output=Path(temporary)
            with self.assertRaises(study.subprocess.TimeoutExpired):
                study.bounded_run([study.sys.executable,'-c','import time;time.sleep(5)'],output,.05)
            with self.assertRaisesRegex(ValueError,'artifact cap'):
                study.bounded_run([study.sys.executable,'-c',"from pathlib import Path;import time;Path(r'%s').write_bytes(b'x'*2048);time.sleep(5)"%str(output/'current.test')],output,2,cap_bytes=1024)
            self.assertTrue((output/'current.test').exists())

class CheckpointSharing(unittest.TestCase):
    def test_transient_replace_retries_only_promotion_and_preserves_errors(self):
        with tempfile.TemporaryDirectory() as temporary:
            checkpoint=Path(temporary)/'checkpoint.json';checkpoint.write_text('{"old":true}')
            original=Path.replace;calls=[]
            def sharing_then_release(source,target):
                calls.append(source.read_bytes())
                if len(calls)<3:raise PermissionError(13,'synthetic sharing violation')
                return original(source,target)
            with mock.patch.object(Path,'replace',sharing_then_release),mock.patch.object(study.time,'sleep') as sleep,mock.patch.object(study,'bounded_run') as run:
                study.commit_checkpoint(checkpoint,{'new':'verified'})
            self.assertEqual(json.loads(checkpoint.read_text()),{'new':'verified'})
            self.assertEqual(len(calls),3);self.assertEqual(len(set(calls)),1)
            self.assertEqual(sleep.call_count,2);run.assert_not_called()
            self.assertFalse(checkpoint.with_suffix('.tmp').exists())
            events=[json.loads(line) for line in checkpoint.with_name('checkpoint-replace-events.jsonl').read_text().splitlines()]
            self.assertEqual([event.get('attempt') for event in events],[1,2,3])
            self.assertEqual(events[-1]['status'],'promoted')
            self.assertEqual(events[0]['message'],'[Errno 13] synthetic sharing violation')
    def test_exhaustion_preserves_old_and_exact_pending_journal(self):
        with tempfile.TemporaryDirectory() as temporary:
            checkpoint=Path(temporary)/'checkpoint.json';original=b'{"old":true}';checkpoint.write_bytes(original)
            with mock.patch.object(Path,'replace',side_effect=PermissionError(13,'held reader')) as replace,mock.patch.object(study.time,'sleep') as sleep,mock.patch.object(study,'bounded_run') as run:
                with self.assertRaises(study.CheckpointReplaceError) as raised:study.commit_checkpoint(checkpoint,{'new':'verified'})
            self.assertEqual(replace.call_count,5);self.assertEqual(sleep.call_count,4);run.assert_not_called()
            self.assertEqual(checkpoint.read_bytes(),original)
            self.assertEqual(json.loads(checkpoint.with_suffix('.tmp').read_text()),{'new':'verified'})
            self.assertEqual(len(raised.exception.errors),5)
            events=checkpoint.with_name('checkpoint-replace-events.jsonl').read_text().splitlines()
            self.assertEqual(len(events),5)
            # Resume cannot silently rerun the already computed pending key.
            args=SimpleNamespace(output=checkpoint.parent,binary=checkpoint.parent/'never-run',freeze=False,execute=True,retry_key=None)
            (checkpoint.parent/'frozen.json').write_text('{"source":"same"}')
            with mock.patch.object(study,'fingerprints',return_value={'source':'same'}),mock.patch.object(study,'bounded_run') as run:
                with self.assertRaisesRegex(ValueError,'pending checkpoint'):study.execute(args)
                run.assert_not_called()
    def test_non_permission_failure_is_not_retried(self):
        with tempfile.TemporaryDirectory() as temporary:
            checkpoint=Path(temporary)/'checkpoint.json';checkpoint.write_text('old')
            with mock.patch.object(Path,'replace',side_effect=OSError(28,'no space')) as replace,mock.patch.object(study.time,'sleep') as sleep:
                with self.assertRaises(OSError):study.commit_checkpoint(checkpoint,{'new':True})
            self.assertEqual(replace.call_count,1);sleep.assert_not_called()
            self.assertEqual(checkpoint.read_text(),'old')
            self.assertTrue(checkpoint.with_suffix('.tmp').exists())
    def test_freeze_rejects_nonempty_directory_before_provenance_or_writes(self):
        with tempfile.TemporaryDirectory() as temporary:
            output=Path(temporary);sentinel=output/'current.owner-file';sentinel.write_text('preserve')
            args=SimpleNamespace(output=output,binary=output/'never-run',freeze=True,execute=False,retry_key=None)
            with mock.patch.object(study,'fingerprints') as fingerprints,mock.patch.object(study,'bounded_run') as run:
                with self.assertRaisesRegex(ValueError,'fresh dedicated empty'):study.execute(args)
                fingerprints.assert_not_called();run.assert_not_called()
            self.assertEqual(sentinel.read_text(),'preserve')
            self.assertEqual(list(output.iterdir()),[sentinel])
    @unittest.skipUnless(study.sys.platform=='win32','native Windows sharing semantics')
    def test_actual_windows_reader_without_delete_share_is_released(self):
        import ctypes
        from ctypes import wintypes
        kernel=ctypes.WinDLL('kernel32',use_last_error=True)
        kernel.CreateFileW.argtypes=[wintypes.LPCWSTR,wintypes.DWORD,wintypes.DWORD,ctypes.c_void_p,wintypes.DWORD,wintypes.DWORD,wintypes.HANDLE]
        kernel.CreateFileW.restype=wintypes.HANDLE
        kernel.CloseHandle.argtypes=[wintypes.HANDLE];kernel.CloseHandle.restype=wintypes.BOOL
        with tempfile.TemporaryDirectory() as temporary:
            checkpoint=Path(temporary)/'checkpoint.json';checkpoint.write_text('{"old":true}')
            handle=kernel.CreateFileW(str(checkpoint),0x80000000,3,None,3,0x80,None)
            if handle==ctypes.c_void_p(-1).value:raise ctypes.WinError(ctypes.get_last_error())
            closed=False
            def release_reader(_seconds):
                nonlocal closed
                if not closed:
                    if not kernel.CloseHandle(handle):raise ctypes.WinError(ctypes.get_last_error())
                    closed=True
            try:
                with mock.patch.object(study.time,'sleep',side_effect=release_reader):study.commit_checkpoint(checkpoint,{'new':'verified'})
            finally:
                if not closed:kernel.CloseHandle(handle)
            self.assertTrue(closed)
            self.assertEqual(json.loads(checkpoint.read_text()),{'new':'verified'})
            events=[json.loads(line) for line in checkpoint.with_name('checkpoint-replace-events.jsonl').read_text().splitlines()]
            self.assertEqual(events[0]['error_type'],'PermissionError')
            self.assertEqual(events[0]['winerror'],5)
            self.assertEqual(events[-1]['status'],'promoted')

if __name__=='__main__':unittest.main()
