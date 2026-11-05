"""Build and run one fresh deterministic one-second example with raw evidence."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import sys

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('history_sampler',Path(__file__).with_name('resource_probe.py'))
SAMPLER = importlib.util.module_from_spec(spec)
spec.loader.exec_module(SAMPLER)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--toolchain',choices=['1.99.0','stable'],required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    if output == SAMPLER.ROOT or SAMPLER.ROOT in output.parents or output.exists():
        raise RuntimeError('fresh external output required')
    SAMPLER.acquire(SAMPLER.ROOT,output)
    output.mkdir(parents=True)
    receipt = dict(success=False,stage='ONE_SECOND_EXAMPLE_NOT_MAXIMUM_RESOURCE_EVIDENCE',
                   commands=[],
                   example_probe_sha256=SAMPLER.sha(Path(__file__).read_bytes()),
                   sampler_sha256=SAMPLER.sha(Path(SAMPLER.__file__).read_bytes()),
                   owner_helper_sha256=SAMPLER.HELPER_SHA)
    try:
        receipt['source_before'] = SAMPLER.inventory(SAMPLER.ROOT)
        for name,argv in [('compiler',['rustc','+'+args.toolchain,'-Vv']),
            ('format',['rustfmt','+'+args.toolchain,'--edition','2024','--check',str(SAMPLER.ROOT/'examples/history_replay.rs')]),
            ('build',['cargo','+'+args.toolchain,'build','--locked','--offline','--example','history_replay'])]:
            receipt['commands'].append(SAMPLER.owned_command(argv,SAMPLER.ROOT,output/(name+'.log')))
        binary = SAMPLER.ROOT/'target/debug/examples'/('history_replay.exe' if os.name=='nt' else 'history_replay')
        receipt['binary'] = dict(path=str(binary),sha256=SAMPLER.sha(binary.read_bytes()),bytes=binary.stat().st_size)
        receipt['commands'].append(SAMPLER.owned_command([str(binary)],SAMPLER.ROOT,output/'example.log',timeout=30))
        text = (output/'example.log').read_text()
        for expected in ['101 snapshots, 3 offered, 2 on time, 1 corrected late','immutable tick 0','W1 correction','W0 stale drop','no accuracy or calibration claim']:
            assert expected in text, expected
        receipt['source_after'] = SAMPLER.inventory(SAMPLER.ROOT)
        assert receipt['source_before'] == receipt['source_after']
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
