#!/usr/bin/env python3
"""Build the existing reference app, or explicitly launch a trusted runtime bundle."""

import argparse
import json
import math
import os
from pathlib import Path
import platform as host_platform
import plistlib
import shutil
import subprocess
import sys
import time

from build_desktop_probe import artifact, checked_sha, sources, verify_copied_sources
from check_macos_metal_sdk import execute_step
from check_windows_desktop_sdk import run_step as windows_step
from qualify_desktop_texture import digest, write_json
import runtime_bundle


def host_target(system=None, machine=None):
    system = sys.platform if system is None else system
    machine = host_platform.machine() if machine is None else machine
    target = {'linux': 'linux', 'darwin': 'macos', 'win32': 'windows'}.get(system)
    architecture = {'x86_64': 'x86_64', 'amd64': 'x86_64',
                    'aarch64': 'arm64', 'arm64': 'arm64'}.get(machine.lower())
    if target is None or architecture is None:
        raise ValueError('requires a supported 64-bit desktop host')
    return target, architecture


def launch_plan(directory, *, system=None, machine=None, environment=None):
    """Validate everything before executing; verification itself never runs code."""
    target, architecture = host_target(system, machine)
    # Do not resolve first: verify must reject a symlink/reparse root itself.
    directory = Path(directory).absolute()
    manifest = runtime_bundle.verify(directory, expected_platform=target,
                                     expected_architecture=architecture)
    directory = directory.resolve(strict=True)
    executable = runtime_bundle.checked(directory, manifest['runtime'])
    if target != 'windows' and not manifest['files'][manifest['runtime']]['executable']:
        raise ValueError('runtime entry point is not executable')
    env = dict(os.environ if environment is None else environment)
    # The example checks the OS variable before any compile-time path. Neither
    # its launch cwd nor an old RHWP_NATIVE_LIBRARY can select another bundle.
    env['RHWP_RUNTIME_BUNDLE'] = str(directory)
    return [str(executable)], executable.parent, env


def launch(directory):
    command, cwd, env = launch_plan(directory)
    # No shell, downloader, SDK lookup, loader-path mutation or implicit build.
    # This is explicitly executing trusted code, not a sandbox or signature.
    return subprocess.call(command, cwd=cwd, env=env)


def built_runtime(runner, target, architecture, mode):
    cpu = {'x86_64': 'x64', 'arm64': 'arm64'}[architecture]
    if target == 'linux':
        directory = runner / f'build/linux/{cpu}/{mode}/bundle'
        executable = 'rhwp_reference_app'
    elif target == 'windows':
        directory = runner / f'build/windows/{cpu}/runner/{mode.title()}'
        executable = 'rhwp_reference_app.exe'
    else:
        directory = runner / f'build/macos/Build/Products/{mode.title()}'
        applications = list(directory.glob('*.app'))
        if len(applications) != 1:
            raise ValueError('expected exactly one built reference app')
        # Package only the app, not dSYM/link/intermediate siblings.
        app = applications[0]
        with (app / 'Contents/Info.plist').open('rb') as source:
            info = plistlib.load(source)
        name = info.get('CFBundleExecutable') if isinstance(info, dict) else None
        if not isinstance(name, str) or '/' in name:
            raise ValueError('expected a macOS CFBundleExecutable filename')
        runtime_bundle.relative(name)
        directory = runner.parent / 'macos-runtime'
        directory.mkdir()
        shutil.copytree(app, directory / app.name, symlinks=True)
        # Debug builds can include .debug.dylib and __preview.dylib beside the
        # main executable. Preserve them and use the bundle's declared entry.
        executable = (Path(app.name) / 'Contents/MacOS' / name).as_posix()
    if not (directory / executable).is_file():
        raise ValueError('Flutter did not produce the reference runtime')
    return directory, executable


def configure_reference_runner(runner, example, target):
    """Use the existing app with generated OS scaffolding, not a second UI."""
    spec = (example / 'pubspec.yaml').read_text(encoding='utf-8')
    if spec.count('    path: ../\n') != 1:
        raise ValueError('unexpected reference example dependency')
    spec = spec.replace('    path: ../\n', '    path: ../source/bindings/Flutter\n')
    (runner / 'pubspec.yaml').write_text(spec, encoding='utf-8')
    shutil.rmtree(runner / 'lib')
    shutil.copytree(example / 'lib', runner / 'lib')
    if target == 'macos':
        # This developer reference uses explicit file paths and sibling Native
        # libraries. It does not implement App Store sandbox/bookmark access.
        for name in ('DebugProfile.entitlements', 'Release.entitlements'):
            path = runner / 'macos/Runner' / name
            with path.open('rb') as source:
                entitlements = plistlib.load(source)
            if not isinstance(entitlements, dict) or entitlements.get('com.apple.security.app-sandbox') is not True:
                raise ValueError('unexpected generated macOS sandbox entitlements')
            entitlements['com.apple.security.app-sandbox'] = False
            with path.open('wb') as output:
                plistlib.dump(entitlements, output)


def build(args):
    target, architecture = host_target()
    if not math.isfinite(args.step_timeout) or not 30 <= args.step_timeout <= 7200:
        raise ValueError('step timeout must be between 30 and 7200 seconds')
    repository = Path(__file__).resolve().parents[3]
    source_head = checked_sha(subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=repository, text=True).strip(), 'source head')
    inputs = ['bindings/Flutter', 'bindings/swift/Sources/CRhwpNative',
              *runtime_bundle.NOTICES, 'licenses']
    if subprocess.check_output(['git', 'status', '--porcelain', '--', *inputs],
                               cwd=repository, text=True):
        raise ValueError('commit the reference app inputs before building')
    native_head = checked_sha(args.native_source, 'declared Native source head')
    native = args.native.absolute()
    native_metadata, _ = runtime_bundle._read(native)
    native_platform, native_architectures = runtime_bundle.binary_identity(native, shared=True)
    if native_platform != target or architecture not in native_architectures:
        raise ValueError('supplied Native library does not match the build host')
    sdk = args.flutter_sdk.resolve(strict=True)
    dart = sdk / ('bin/cache/dart-sdk/bin/dart.exe' if target == 'windows'
                  else 'bin/cache/dart-sdk/bin/dart')
    snapshot = sdk / 'bin/cache/flutter_tools.snapshot'
    if not dart.is_file() or not snapshot.is_file():
        raise ValueError('prepare the Flutter SDK and tool snapshot first')
    if target == 'windows' and not shutil.which('cl'):
        raise ValueError('run from the matching MSVC Native Tools environment (cl is required)')
    flutter = [str(dart), '--disable-dart-dev', str(snapshot)]
    output = args.output.resolve()
    if output == repository or repository in output.parents:
        raise ValueError('build output must be outside the source checkout')
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    original = sources(repository)
    # Freeze redistribution notices alongside every tracked Flutter/header input.
    notice_paths = subprocess.check_output(['git', 'ls-files', '-z', '--',
                                            *runtime_bundle.NOTICES, 'licenses'],
                                           cwd=repository).split(b'\0')
    for path in notice_paths:
        if path:
            relative = os.fsdecode(path)
            original[relative] = digest(repository / relative)
    copied = output / 'source'
    for relative in original:
        destination = copied / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(repository / relative, destination)
    native_hash = native_metadata['sha256']
    env = dict(os.environ, CI='true', FLUTTER_ROOT=str(sdk),
               FLUTTER_SUPPRESS_ANALYTICS='true', DART_SUPPRESS_ANALYTICS='true',
               PYTHONDONTWRITEBYTECODE='1')
    receipt = {'kind': 'rhwp-reference-app-build', 'schemaVersion': 1,
               'platform': target, 'architecture': architecture, 'sourceHead': source_head,
               'nativeSourceHead': native_head, 'nativeBuildVerified': False,
               'nativeProvenance': 'supplied library; hash and caller-declared source, no Native rebuild',
               'mode': args.mode, 'gpu': not args.cpu, 'continuous': args.continuous,
               'sourceFiles': original, 'steps': [], 'buildExitCode': None,
               'started': time.time(), 'pid': os.getpid()}
    receipt_path = output / 'build-receipt.json'

    def record():
        write_json(receipt_path, receipt)

    def run(command, cwd):
        index = len(receipt['steps'])
        if target == 'windows':
            windows_step(command, cwd=cwd, env=env, output=output, receipt=receipt,
                         record=record, timeout=args.step_timeout)
        else:
            step = {'command': command, 'cwd': str(cwd), 'started': time.time(),
                    'exitCode': None, 'log': str(output / f'step-{index}.log')}
            receipt['steps'].append(step)
            record()
            code = execute_step(command, repository=cwd, env=env, log_path=step['log'],
                                step=step, record=record, timeout=args.step_timeout)
            if code:
                raise RuntimeError('build failed; inspect completed log: ' + step['log'])
        log = Path(receipt['steps'][index]['log'])
        # Only --version is parsed; build/compiler output stays in its log.
        return log

    try:
        record()
        version_log = run([*flutter, '--version', '--machine'], output)
        if version_log.stat().st_size > 1024 * 1024:
            raise ValueError('Flutter version output exceeds its bound')
        version = json.loads(version_log.read_text(encoding='utf-8'))
        receipt.update(flutterRevision=checked_sha(version.get('frameworkRevision'), 'Flutter revision'),
                       engineRevision=checked_sha(version.get('engineRevision'), 'engine revision'),
                       flutterVersion=version.get('frameworkVersion'))
        run([*flutter, 'create', '--no-pub', '--platforms=' + target,
             '--project-name=rhwp_reference_app', 'runner'], output)
        runner = output / 'runner'
        example = copied / 'bindings/Flutter/example'
        configure_reference_runner(runner, example, target)
        receipt['generatedRunner'] = {
            path.relative_to(runner).as_posix(): digest(path)
            for path in sorted(runner.rglob('*')) if path.is_file()}
        receipt['macosSandbox'] = False if target == 'macos' else None
        run([*flutter, 'pub', 'get', '--offline'], runner)
        run([*flutter, 'build', target, '--' + args.mode,
             '--dart-define=RHWP_GPU=' + str(not args.cpu).lower(),
             '--dart-define=RHWP_GPU_CONTINUOUS=' + str(args.continuous).lower(),
             '--dart-define=RHWP_GPU_TRACE=' + str(args.trace).lower()], runner)
        runtime, executable = built_runtime(runner, target, architecture, args.mode)
        dispatcher = output / runtime_bundle.NAMES[target][1]
        run([sys.executable, '-B', str(copied / 'bindings/Flutter/tool/build_dispatcher.py'),
             '--output', str(dispatcher)], output)
        manifest = runtime_bundle.package(
            runtime, executable, native, dispatcher, copied, output / 'bundle', target, architecture,
            {'runtime': source_head, 'native': native_head, 'dispatcher': source_head})
        receipt['artifacts'] = {'runtime': artifact(runtime / executable),
                                'native': artifact(native), 'dispatcher': artifact(dispatcher)}
        receipt['bundleManifestSha256'] = digest(output / 'bundle/rhwp-runtime.json')
        receipt['bundleFiles'] = len(manifest['files'])
        receipt['copiedRuntimeSources'] = verify_copied_sources(copied, original)
        if any(digest(repository / path) != value for path, value in original.items()):
            raise ValueError('source changed during build')
        if digest(native) != native_hash:
            raise ValueError('supplied Native changed during build')
        if (subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repository,
                                    text=True).strip() != source_head or
                subprocess.check_output(['git', 'status', '--porcelain', '--', *inputs],
                                        cwd=repository, text=True)):
            raise ValueError('pinned checkout changed during build')
        receipt.update(buildExitCode=0, runtimeLaunched=False)
    except BaseException as error:
        receipt.update(buildExitCode=1, error=f'{type(error).__name__}: {error}')
        raise
    finally:
        receipt['finished'] = time.time()
        record()
    print(receipt_path)
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    commands = parser.add_subparsers(dest='command', required=True)
    create = commands.add_parser('build', allow_abbrev=False)
    create.add_argument('--flutter-sdk', type=Path, required=True)
    create.add_argument('--native', type=Path, required=True)
    create.add_argument('--native-source', required=True)
    create.add_argument('--output', type=Path, required=True)
    create.add_argument('--mode', choices=('debug', 'release'), default='release')
    create.add_argument('--cpu', action='store_true', help='build the CPU-only reference viewport')
    create.add_argument('--continuous', action='store_true', help='opt in to continuous Windows/Metal presentation')
    create.add_argument('--trace', action='store_true', help='include bounded producer diagnostics')
    create.add_argument('--step-timeout', type=float, default=900)
    start = commands.add_parser('run', allow_abbrev=False,
                                help='verify then execute a trusted bundle on this desktop')
    start.add_argument('--bundle', type=Path, required=True)
    args = parser.parse_args()
    try:
        return build(args) if args.command == 'build' else launch(args.bundle)
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print('rhwp reference app: ' + str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
