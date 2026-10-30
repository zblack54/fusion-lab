#!/usr/bin/env python3
"""Materialize two already-approved empty checkout directories; never repair bytes."""
import argparse,hashlib,json,pathlib,stat
APPROVED='e0d3a23f4335193f6442fa7d1b244621b93bbf3f08ee9ba9c4b860c2eb15911b'
DIRECTORIES=('tests/observation-delivery/benches','tests/observation-delivery/fixtures')
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def refuse_link(path):
    info=path.lstat()
    if stat.S_ISLNK(info.st_mode) or getattr(info,'st_file_attributes',0)&0x400:
        raise ValueError('symlink/reparse path refused: '+str(path))
    if not stat.S_ISDIR(info.st_mode): raise ValueError('non-directory component refused: '+str(path))
def check_path(repo,path):
    if not path.is_relative_to(repo): raise ValueError('path escapes repository')
    refuse_link(repo)
    cursor=repo
    for component in path.relative_to(repo).parts:
        cursor=cursor/component
        if cursor.exists() or cursor.is_symlink(): refuse_link(cursor)
    if not path.resolve().is_relative_to(repo): raise ValueError('resolved path escapes repository')
def fingerprints(repo):
    suite=repo/'tests/observation-delivery'
    result={}
    for path in suite.rglob('*'):
        if path.is_symlink() or getattr(path.lstat(),'st_file_attributes',0)&0x400: raise ValueError('reparse input refused: '+str(path))
        if path.is_file() and not any(x in path.relative_to(suite).parts for x in ('target','__pycache__','.git')):
            result[path.relative_to(repo).as_posix()]=sha(path)
    lock=repo/'docs/design-docs/observation-delivery/validation/test-lock.json'
    result[lock.relative_to(repo).as_posix()]=sha(lock)
    return result

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline-digest',required=True,help='actual externally approved digest; not read as expected value from candidate')
    parser.add_argument('--repo',type=pathlib.Path,default=pathlib.Path(__file__).resolve().parents[4])
    args=parser.parse_args();repo=args.repo.absolute()
    if args.baseline_digest!=APPROVED: raise ValueError('unsupported externally supplied approved baseline')
    if repo.resolve()!=repo: raise ValueError('repository root must not traverse a symlink/reparse alias')
    lock=repo/'docs/design-docs/observation-delivery/validation/test-lock.json'
    check_path(repo,lock.parent)
    if lock.is_symlink() or getattr(lock.lstat(),'st_file_attributes',0)&0x400: raise ValueError('lock link/reparse refused')
    candidate=json.loads(lock.read_text(encoding='utf-8'))
    if candidate.get('digest')!=args.baseline_digest: raise ValueError('candidate lock differs from externally supplied digest')
    targets=[repo/name for name in DIRECTORIES]
    # Validate both destinations before creating either one. No files are removed.
    for path in targets:
        check_path(repo,path)
        if path.exists() and any(path.iterdir()): raise ValueError('approved empty directory is nonempty: '+str(path))
    before=fingerprints(repo)
    created=[]
    for path in targets:
        check_path(repo,path)
        if not path.exists(): path.mkdir();created.append(path.relative_to(repo).as_posix())
        check_path(repo,path)
        if any(path.iterdir()): raise ValueError('directory changed during preparation: '+str(path))
    after=fingerprints(repo)
    if before!=after: raise RuntimeError('input file fingerprints changed; no byte repair attempted')
    print(json.dumps(dict(status='ENVIRONMENT_MATERIALIZED_NOT_VALIDATION_OR_ACCEPTANCE',baseline_digest=args.baseline_digest,created_directories=created,approved_directories=list(DIRECTORIES),file_fingerprints_before=before,file_fingerprints_after=after,all_input_bytes_unchanged=True,precondition='Exact Git blob bytes: core.autocrlf=false; no normalization performed',next_step='Run host-trusted require-lock/external-digest implementation verification')))
if __name__=='__main__': main()
