"""Record pinned dependency metadata without credentials or host paths."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser=argparse.ArgumentParser()
parser.add_argument("--output",type=Path,required=True)
args=parser.parse_args()
root=Path(__file__).resolve().parents[4]
manifest=root/"tests/local-supervisor/Cargo.toml"
data=json.loads(subprocess.check_output(["cargo","metadata","--format-version","1","--manifest-path",str(manifest),"--locked","--offline"],cwd=root,text=True))
packages=[]
for package in sorted(data["packages"],key=lambda p:(p["name"],p["version"])):
    if package["source"] is None:
        continue
    rust=package.get("rust_version")
    if rust and tuple(map(int,rust.split(".")[:2]))>(1,99):
        raise SystemExit("Dependency MSRV exceeds promised 1.99")
    if not package.get("license"):
        raise SystemExit("Dependency license missing: "+package["name"])
    packages.append({"name":package["name"],"version":package["version"],
                     "license":package["license"],"declared_rust_version":rust})
report={"harness_lock_sha256":hashlib.sha256((manifest.parent/"Cargo.lock").read_bytes()).hexdigest(),
        "production_lock_sha256":hashlib.sha256((root/"crates/local-supervisor/Cargo.lock").read_bytes()).hexdigest(),
        "registry_packages":packages}
args.output.write_text(json.dumps(report,indent=2)+"\n")
print(json.dumps({"registry_package_count":len(packages),"licenses":sorted({p["license"] for p in packages}),"output":str(args.output)}))
