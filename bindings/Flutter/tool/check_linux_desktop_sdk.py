#!/usr/bin/env python3
"""Compile the real Linux SDK bridge, then optionally build and sample GL probes.

No SDK/dependency installation or feature-default change is performed. Native
capability loading is isolated in a child and does not establish GPU sampling.
"""
import argparse
import ctypes
import json
import math
import os
from pathlib import Path
import platform
import shlex
import shutil
import signal
import subprocess
import sys
import time

from build_desktop_probe import artifact, checked_sha
from build_linux_probe import SDK_SUPERVISION_ENV
from check_macos_metal_sdk import execute_step, source_hashes
from native_artifact_provenance import load_native_artifact
from qualify_desktop_texture import MAX_BUILD_RECEIPT_BYTES, digest, read_json, write_json
from runtime_bundle import binary_identity
import gpu_memory_acceptance as memory_acceptance

TARGET = 'x86_64-unknown-linux-gnu'
FEATURE = 'native-skia-gl'
MAX_CAPABILITIES = 64 * 1024


def require_host(system=None, machine=None):
    if (system or sys.platform) != 'linux' or (machine or platform.machine()).lower() not in ('x86_64', 'amd64'):
        raise RuntimeError('actual Linux x86_64 GTK/GL SDK execution is required')


def validate_capabilities(value):
    if not isinstance(value, dict) or value.get('ok') is not True or type(value.get('abiVersion')) is not int or value['abiVersion'] != 1:
        raise ValueError('Native ABI 1 capability reply is required')
    if any(value.get(key) is not True for key in ('nativeSkia', 'worker', 'hostRequests')):
        raise ValueError('Native worker/Skia support is required')
    presentation, gl = value.get('presentation'), value.get('gpuGl')
    if not isinstance(presentation, dict) or any(presentation.get(key) is not True for key in ('workerRenderSnapshots', 'crossThreadHandles', 'fullStampChecks', 'cpuBatchFrames')):
        raise ValueError('full-stamp presentation and CPU oracle support are required')
    if (not isinstance(gl, dict) or type(gl.get('abiVersion')) is not int or gl['abiVersion'] != 1 or
            any(gl.get(key) is not True for key in ('available', 'hostFboRendering'))):
        raise ValueError('enabled native-skia-gl host-FBO capabilities are required; CPU Native is not GPU evidence')
    return {'ok': True, 'abiVersion': 1, 'nativeSkia': True, 'worker': True, 'hostRequests': True,
            'presentation': {key: presentation[key] for key in ('workerRenderSnapshots', 'crossThreadHandles', 'fullStampChecks', 'cpuBatchFrames')},
            'gpuGl': {key: gl[key] for key in ('abiVersion', 'available', 'hostFboRendering')}}


def probe_native(path, output):
    require_host()
    path = path.resolve(strict=True)
    before = digest(path)
    library = ctypes.CDLL(str(path))
    capability = library.rhwp_session_capabilities
    capability.argtypes, capability.restype = [], ctypes.c_void_p
    free = library.rhwp_string_free
    free.argtypes, free.restype = [ctypes.c_void_p], None
    pointer = capability()
    if not pointer:
        raise ValueError('Native capability call returned null')
    try:
        span = ctypes.cast(pointer, ctypes.POINTER(ctypes.c_ubyte))
        data = bytearray()
        for index in range(MAX_CAPABILITIES + 1):
            byte = span[index]
            if not byte:
                break
            data.append(byte)
        else:
            raise ValueError('Native capability reply exceeds 64 KiB')
        capabilities = validate_capabilities(json.loads(data))
    finally:
        free(pointer)
    if digest(path) != before:
        raise ValueError('Native library changed during its feature probe')
    if output.exists():
        raise ValueError('capability output must be new')
    write_json(output, {'scope': 'nativeFeatureProbeOnly', 'nativeSha256': before,
                        'capabilities': capabilities, 'consumerSamplingVerified': False,
                        'consumerGpuCompletionVerified': False, 'hardwareAccelerationVerified': False})


def qualify(args, *, repository=None, executor=execute_step):
    require_host()
    cycles = getattr(args, 'memory_pressure_cycles', 0)
    if type(cycles) is not int or not 0 <= cycles <= 2:
        raise ValueError('combined Linux memory-pressure cycles must be 0..2')
    if cycles and (args.compile_only or args.build_only):
        raise ValueError('memory-pressure cycles require actual consumer sampling, not compile-only/build-only')
    if not math.isfinite(args.step_timeout) or not 30 <= args.step_timeout <= 7200:
        raise ValueError('step timeout must be 30..7200 seconds')
    if bool(args.native_artifact_root) != bool(args.native_source_head):
        raise ValueError('Native reuse requires both artifact root and exact source head')
    if args.compile_only and args.native_artifact_root:
        raise ValueError('compile-only does not load or reuse Native')
    repo = Path(repository or Path(__file__).resolve().parents[3]).resolve(strict=True)
    before = source_hashes(repo)
    head = checked_sha(subprocess.check_output(['git', '-c', 'gc.auto=0', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip(), 'source head')
    sdk = args.flutter_sdk.resolve(strict=True)
    engine = sdk / 'bin/cache/artifacts/engine/linux-x64'
    dart = sdk / 'bin/cache/dart-sdk/bin/dart'
    snapshot = sdk / 'bin/cache/flutter_tools.snapshot'
    if not all(path.is_file() for path in (dart, snapshot, engine / 'flutter_linux/flutter_linux.h', engine / 'libflutter_linux_gtk.so')):
        raise ValueError('prepared real Linux Flutter SDK headers, engine and tools are required')
    compiler = shutil.which(args.cxx)
    if not compiler:
        raise ValueError('a real Linux C++ compiler is required')
    output = args.output.resolve()
    if output == repo or repo in output.parents:
        raise ValueError('SDK outputs must be outside the source checkout')
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    (output / 'tmp').mkdir(mode=0o700)
    receipt = output / 'linux-sdk-receipt.json'
    state = {'platform': 'linux', 'source_head': head, 'sourceFiles': before,
             'sourceHashKind': 'git-index-mode-blob-oid-and-path', 'rustTarget': TARGET,
             'requestedNativeFeatures': [FEATURE], 'steps': [], 'started': time.time(), 'build': 'pending',
             'targetSdkCompileLinkVerified': False, 'nativeBuildVerified': False,
             'nativeGlCapabilitiesVerified': False, 'probeBuildsVerified': False,
             'singleConsumerSamplingVerified': False, 'tileConsumerSamplingVerified': False,
             'consumerGpuCompletionVerified': False, 'hardwareAccelerationVerified': False}
    if cycles:
        state.update(requestedMemoryPressureCycles=cycles,
                     singleMemoryPressureVerified=False, tileMemoryPressureVerified=False)
    env = dict(os.environ, CI='true', FLUTTER_ROOT=str(sdk), FLUTTER_SUPPRESS_ANALYTICS='true',
               DART_SUPPRESS_ANALYTICS='true', PYTHONDONTWRITEBYTECODE='1', TMPDIR=str(output / 'tmp'),
               CARGO_TARGET_DIR=str(output / 'native-target'), CARGO_BUILD_BUILD_DIR=str(output / 'native-build'),
               CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0')
    env.pop(SDK_SUPERVISION_ENV, None)
    record = lambda: write_json(receipt, state)
    def unchanged():
        return source_hashes(repo) == before and subprocess.check_output(
            ['git', '-c', 'gc.auto=0', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip() == head
    def run(command, *, cwd=repo, timeout=None, supervise_children=False):
        if not unchanged():
            raise ValueError('source checkout changed before SDK step')
        step = {'command': command, 'cwd': str(cwd), 'started': time.time(), 'exitCode': None,
                'log': str(output / f'step-{len(state["steps"])}.log')}
        state['steps'].append(step)
        record()
        child_env = {**env, SDK_SUPERVISION_ENV: '1'} if supervise_children else env
        code = executor(command, repository=cwd, env=child_env, log_path=step['log'], step=step,
                        record=record, timeout=timeout or args.step_timeout,
                        stop_failed_group=True)
        if not unchanged():
            raise ValueError('source checkout changed during SDK step')
        if code:
            raise RuntimeError('SDK step failed; inspect completed log: ' + step['log'])
        return Path(step['log'])
    def text(path, maximum=1024 * 1024):
        if path.stat().st_size > maximum:
            raise ValueError('metadata output exceeds its read bound')
        return path.read_text(encoding='utf-8')
    try:
        record()
        flutter = [str(dart), '--disable-dart-dev', str(snapshot)]
        version = json.loads(text(run([*flutter, '--version', '--machine'])))
        state.update(flutterRevision=checked_sha(version.get('frameworkRevision'), 'Flutter revision'),
                     engineRevision=checked_sha(version.get('engineRevision'), 'engine revision'))
        state['compiler'] = text(run([compiler, '--version'])).splitlines()[0]
        flags = shlex.split(text(run(['pkg-config', '--cflags', '--libs', 'gtk+-3.0', 'epoxy']), 65536))
        state['sdkPackages'] = text(run(['pkg-config', '--modversion', 'gtk+-3.0', 'epoxy']), 65536).splitlines()
        state['flutterHeaders'] = {path.name: digest(path) for path in sorted((engine / 'flutter_linux').glob('*.h'))}
        state['flutterEngineRuntime'] = artifact(engine / 'libflutter_linux_gtk.so')
        plugin = output / 'librhwp_native_reference_plugin.so'
        run([compiler, '-std=c++17', '-shared', '-fPIC', '-Wall', '-Wextra', '-Werror', '-Wl,--no-undefined',
             '-I' + str(engine), '-I' + str(repo / 'bindings/Flutter/linux/include'),
             '-I' + str(repo / 'bindings/swift/Sources/CRhwpNative'),
             *[str(repo / 'bindings/Flutter/linux' / name) for name in ('rhwp_texture_plugin.cc', 'texture_state.cc', 'gl_state_guard.cc')],
             '-L' + str(engine), '-lflutter_linux_gtk', '-ldl', *flags, '-o', str(plugin)])
        if binary_identity(plugin, shared=True) != ('linux', {'x86_64'}):
            raise ValueError('SDK plugin does not match the Linux x86_64 host')
        state.update(targetSdkCompileLinkVerified=True, sdkPlugin=artifact(plugin))
        if not args.compile_only:
            native_receipt = output / 'native-build-receipt.json'
            native_head = args.native_source_head or head
            if args.native_artifact_root:
                reused = load_native_artifact(args.native_artifact_root, repo, 'linux', native_head)
                (output / 'native-artifact').mkdir(mode=0o700)
                native = output / 'native-artifact/librhwp_native_ffi.so'
                shutil.copyfile(reused['native'], native)
                shutil.copyfile(reused['nativeReceipt'], native_receipt)
                if digest(native) != reused['proof']['nativeSha256']:
                    raise ValueError('Native artifact changed while copying')
                state.update(nativeRebuilt=False, nativeReuse=reused['proof'])
            else:
                state['rustcVersion'] = text(run(['rustup', 'run', '1.93.1', 'rustc', '--version', '--verbose']), 16384)
                hosts = [line.removeprefix('host: ') for line in state['rustcVersion'].splitlines() if line.startswith('host: ')]
                if hosts != [TARGET]:
                    raise ValueError('the pinned Rust compiler host does not match Linux x86_64')
                state['rustcHost'] = TARGET
                run(['rustup', 'run', '1.93.1', 'cargo', 'rustc', '--locked',
                     '--manifest-path', str(repo / 'bindings/Native/Cargo.toml'), '--features', FEATURE,
                     '--lib', '--crate-type', 'cdylib', '--target', TARGET])
                native = output / f'native-target/{TARGET}/debug/librhwp_native_ffi.so'
                write_json(native_receipt, {'platform': 'linux', 'source_head': head, 'build': 'passed',
                                           'target': TARGET, 'features': [FEATURE],
                                           'artifacts': {native.name: artifact(native)}})
                state['nativeRebuilt'] = True
            state.update(nativeBuildVerified=True, nativeSourceHead=native_head, native=artifact(native))
            proof_path = output / 'native-gl-capabilities.json'
            run([sys.executable, '-B', str(repo / 'bindings/Flutter/tool/check_linux_desktop_sdk.py'),
                 '--probe-native', str(native), '--probe-output', str(proof_path)], timeout=60)
            proof = read_json(proof_path, limit=MAX_CAPABILITIES)
            validate_capabilities(proof.get('capabilities'))
            if (proof.get('scope') != 'nativeFeatureProbeOnly' or proof.get('nativeSha256') != state['native']['sha256'] or digest(native) != state['native']['sha256'] or
                    any(proof.get(key) is not False for key in ('consumerSamplingVerified', 'consumerGpuCompletionVerified', 'hardwareAccelerationVerified'))):
                raise ValueError('feature probe did not use the pinned Native artifact')
            state.update(nativeGlCapabilitiesVerified=True, nativeCapabilityProbe=proof)
            for tiled in (False, True):
                label = 'tile' if tiled else 'single'
                build = output / (label + '-build')
                run([sys.executable, '-B', str(repo / 'bindings/Flutter/tool/build_linux_probe.py'),
                     '--flutter-sdk', str(sdk), '--native', str(native), '--native-source-head', native_head,
                     '--output', str(build), *(['--tile-group'] if tiled else [])], supervise_children=True)
                built = read_json(build / 'build-receipt.json', limit=MAX_BUILD_RECEIPT_BYTES)
                if (type(built.get('buildExitCode')) is not int or built['buildExitCode'] != 0 or built.get('sourceHead') != head or
                        built.get('nativeSourceHead') != native_head or built.get('probeKind') != ('twoTileGroup' if tiled else 'singleTexture') or
                        built.get('flutterRevision') != state['flutterRevision'] or built.get('engineRevision') != state['engineRevision'] or
                        built.get('artifacts', {}).get('native', {}).get('sha256') != state['native']['sha256'] or digest(native) != state['native']['sha256']):
                    raise ValueError('probe build provenance does not match the pinned inputs')
                state[label + 'ProbeBuild'] = artifact(build / 'build-receipt.json')
                if not args.build_only:
                    command = [sys.executable, '-B', str(repo / ('bindings/Flutter/tool/qualify_linux_tile_group.py' if tiled else 'bindings/Flutter/tool/qualify_linux_texture.py'))]
                    for key in ('runner', 'native', 'dispatcher', 'fixture'):
                        command += ['--' + key, built['artifacts'][key]['path']]
                    capture = output / (label + '-capture')
                    command += ['--build-receipt', str(build / 'build-receipt.json'), '--output', str(capture)]
                    if not tiled:
                        command += ['--frames', '32']
                    if cycles:
                        command += ['--memory-pressure-cycles', str(cycles)]
                    run(command, timeout=330, supervise_children=True)
                    captured = read_json(capture / 'receipt.json', limit=32 * 1024 * 1024)
                    if (captured.get('qualification') != ('linux-two-tile-consumer-passed' if tiled else 'linux-consumer-passed') or
                            type(captured.get('exitCode')) is not int or captured['exitCode'] != 0 or captured.get('consumerGpuCompletionVerified') is not False or
                            captured.get('hardwareAccelerationVerified') is not False):
                        raise ValueError('actual consumer capture did not pass its bounded contract')
                    if cycles:
                        stages, result = captured.get('stages'), captured.get('result')
                        if (type(captured.get('memoryPressureCycles')) is not int or captured['memoryPressureCycles'] != cycles or
                                not isinstance(stages, list) or len(stages) != (3 if tiled else 32) or
                                any(not isinstance(entry, dict) or not isinstance(entry.get('stage'), dict) for entry in stages) or
                                not isinstance(result, dict) or type(captured.get('pid')) is not int or captured['pid'] <= 0):
                            raise ValueError('actual memory-pressure capture is missing its requested cycles/stages/owner')
                        summary = memory_acceptance.validate_memory_acceptance(result, cycles=cycles,
                            stages=[entry['stage'] for entry in stages], pid=captured['pid'],
                            publication_profile='linuxThreeStageTileGroup' if tiled else 'continuous32To64')
                        if captured.get('memoryAcceptance') != summary:
                            raise ValueError('actual memory-pressure summary differs from its native receipts')
                        state[label + 'MemoryPressureVerified'] = True
                        state[label + 'MemoryAcceptance'] = summary
                    state[label + 'ConsumerSamplingVerified'] = True
                    state[label + 'ConsumerReceipt'] = artifact(capture / 'receipt.json')
            state['probeBuildsVerified'] = True
        if not unchanged():
            raise ValueError('source checkout changed before final qualification')
        if not args.compile_only and digest(native) != state['native']['sha256']:
            raise ValueError('Native library changed before final qualification')
        if digest(engine / 'libflutter_linux_gtk.so') != state['flutterEngineRuntime']['sha256'] or any(
                digest(engine / 'flutter_linux' / name) != value for name, value in state['flutterHeaders'].items()):
            raise ValueError('Flutter SDK engine or public headers changed during qualification')
        state.update(build='passed', qualification=('linux-sdk-compiled' if args.compile_only else
                     'linux-desktop-built' if args.build_only else 'linux-desktop-consumer-passed'))
    except BaseException as error:
        state.update(build='failed', error=f'{type(error).__name__}: {error}')
        raise
    finally:
        state['finished'] = time.time()
        record()
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--flutter-sdk', type=Path)
    parser.add_argument('--output', type=Path)
    stage = parser.add_mutually_exclusive_group()
    stage.add_argument('--compile-only', action='store_true', help='SDK plugin compile/link only; no Native build or sampling')
    stage.add_argument('--build-only', action='store_true', help='also verify GL Native and build probes; no consumer sampling')
    parser.add_argument('--native-artifact-root', type=Path)
    parser.add_argument('--native-source-head')
    parser.add_argument('--cxx', default=os.environ.get('CXX', 'clang++'))
    parser.add_argument('--step-timeout', type=float, default=3600)
    parser.add_argument('--memory-pressure-cycles', type=int, default=0,
                        help='0 (default) or 1..2 actual pressure/trim/idle cycles in both consumer probes')
    parser.add_argument('--probe-native', type=Path, help=argparse.SUPPRESS)
    parser.add_argument('--probe-output', type=Path, help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.probe_native or args.probe_output:
        if not args.probe_native or not args.probe_output or args.flutter_sdk or args.output or args.memory_pressure_cycles:
            parser.error('internal capability probe requires only Native and a new output path')
        probe_native(args.probe_native, args.probe_output)
    else:
        if not args.flutter_sdk or not args.output:
            parser.error('--flutter-sdk and --output are required')
        def interrupt(signum, _frame):
            raise KeyboardInterrupt(f'Linux SDK qualification interrupted by signal {signum}')
        signal.signal(signal.SIGTERM, interrupt)
        print(qualify(args))


if __name__ == '__main__':
    main()
