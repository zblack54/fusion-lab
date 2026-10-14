"""Build or preview documentation in a repository-local virtual environment."""
import argparse
import os
from pathlib import Path
import subprocess
import venv


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['build', 'serve'])
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    environment = root / '.venv-docs'
    python = environment / ('Scripts/python.exe' if os.name == 'nt' else 'bin/python')
    if not python.exists():
        venv.EnvBuilder(with_pip=True).create(environment)
    requirements = root / 'docs/requirements.txt'
    marker = environment / '.requirements-content'
    content = requirements.read_text(encoding='utf-8')
    if not marker.exists() or marker.read_text(encoding='utf-8') != content:
        subprocess.run([str(python), '-m', 'pip', 'install', '-r', str(requirements)], check=True)
        marker.write_text(content, encoding='utf-8')
    command = [str(python), '-m', 'mkdocs', args.action]
    if args.action == 'build':
        command.append('--strict')
    subprocess.run(command, cwd=root, check=True)


if __name__ == '__main__':
    main()
