#!/usr/bin/env python3
"""Build only the owner-thread FFI shim; rhwp itself is supplied by the host."""

import argparse
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    source = Path(__file__).resolve().parents[1] / 'src' / 'owner_dispatcher.c'
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    if os.name == 'nt':
        compiler = shlex.split(os.environ.get('CC', 'cl'))
        command = [*compiler, '/nologo', '/LD', '/std:c11', '/W4', '/WX', str(source), '/link', '/OUT:' + str(output)]
    else:
        compiler = shlex.split(os.environ.get('CC', 'cc'))
        command = [*compiler, '-std=c11', '-O2', '-fPIC', '-Wall', '-Wextra', '-Werror', '-pthread',
                   '-dynamiclib' if sys.platform == 'darwin' else '-shared', str(source), '-o', str(output)]
        if sys.platform != 'darwin':
            command.append('-ldl')
    with tempfile.TemporaryDirectory(prefix='rhwp-owner-build-') as directory:
        subprocess.run(command, cwd=directory, check=True)
    print(output)


if __name__ == '__main__':
    main()
