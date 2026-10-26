"""Probe a real empty Windows junction root without symlink privilege elevation."""
import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import tempfile


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--output",type=pathlib.Path,required=True)
    args=parser.parse_args()
    if os.name!="nt":raise SystemExit("Windows-only reparse-point demonstration")
    repo=pathlib.Path(__file__).resolve().parents[3]
    host=repo/"crates/experiment-service/target/release/examples/probe_host.exe"
    worker=repo/"crates/experiment-service/target/release/examples/probe_worker.exe"
    with tempfile.TemporaryDirectory(prefix="fusion-reparse-probe-") as temporary:
        base=pathlib.Path(temporary);target=base/"empty-target";target.mkdir();junction=base/"root-junction"
        command=["cmd.exe","/d","/c","mklink","/J",str(junction),str(target)]
        created=subprocess.run(command,capture_output=True,text=True,timeout=10)
        result={"command":command,"create_returncode":created.returncode,"create_stdout":created.stdout,"create_stderr":created.stderr,"host_sha256":hashlib.sha256(host.read_bytes()).hexdigest(),"probe_sha256":hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()}
        assert created.returncode==0,result
        rejected=subprocess.run([str(host),str(junction),str(worker)],input="shutdown\n",capture_output=True,text=True,timeout=10)
        result.update({"startup_returncode":rejected.returncode,"startup_stderr":rejected.stderr,"target_still_empty":list(target.iterdir())==[],"root_is_junction":junction.is_junction()})
        assert rejected.returncode!=0 and "artifact root must be empty and not a link" in rejected.stderr and result["target_still_empty"] and result["root_is_junction"],result
        result["status"]="empty_junction_root_rejected"
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps({"status":result["status"],"output":str(args.output)}))


if __name__=="__main__":main()
