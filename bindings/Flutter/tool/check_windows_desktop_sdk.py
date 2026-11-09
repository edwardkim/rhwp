#!/usr/bin/env python3
"""Build and execute the pinned Windows desktop qualification on genuine Windows.

This is a validation command, without publish, secret, settings or deployment work.
Each child writes only to its owned log until it exits; fixed inputs stay pinned.
"""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

from qualify_desktop_texture import digest, require_actual_windows, write_json


def run_step(command, *, cwd, env, output, receipt, record, timeout=3600):
    index = len(receipt['steps'])
    log_path = output / f'step-{index}.log'
    step = {'command': command, 'cwd': str(cwd), 'started': time.time(),
            'exitCode': None, 'log': str(log_path), 'timeoutSeconds': timeout}
    receipt['steps'].append(step)
    record()
    with log_path.open('wb') as log:
        log_path.chmod(0o600)
        flags = subprocess.CREATE_NEW_PROCESS_GROUP if sys.platform == 'win32' else 0
        process = subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                   stdout=log, stderr=subprocess.STDOUT, creationflags=flags)
        step['pid'] = process.pid
        try:
            record()
            code = process.wait(timeout=timeout)
        except BaseException:
            step['interrupted'] = True
            if sys.platform == 'win32':
                subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'],
                               stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL, timeout=30, check=False)
            else:
                process.terminate()
            try:
                process.wait(timeout=30)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            raise
        finally:
            step.update(exitCode=process.returncode, finished=time.time())
            record()
    if code:
        # Read an excerpt only after the child has fully exited.
        tail = log_path.read_text(encoding='utf-8', errors='replace').splitlines()[-50:]
        raise RuntimeError('step failed: ' + str(log_path) + '\n' + '\n'.join(tail))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--flutter-sdk', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--native-artifact-root', type=Path)
    parser.add_argument('--native-source-head')
    args = parser.parse_args()
    if bool(args.native_artifact_root) != bool(args.native_source_head):
        parser.error('Native artifact reuse requires both its directory and exact source head')
    require_actual_windows()
    repo = Path(__file__).resolve().parents[3]
    source_head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()
    if subprocess.check_output(['git', 'status', '--porcelain'], cwd=repo, text=True):
        raise ValueError('the complete qualification source must be committed and clean')
    sdk = args.flutter_sdk.resolve(strict=True)
    output = args.output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    receipt_path = output / 'windows-sdk-receipt.json'
    state = {'platform': 'windows', 'source_head': source_head, 'steps': [], 'build': 'pending',
             'started': time.time(), 'pid': os.getpid(), 'consumerGpuCompletionVerified': False,
             'continuousPresentationSupported': False, 'hardwareAccelerationVerified': False}
    record = lambda: write_json(receipt_path, state)
    env = dict(os.environ, CARGO_TARGET_DIR=str(output / 'native-target'),
               CARGO_BUILD_BUILD_DIR=str(output / 'native-build'), CARGO_BUILD_JOBS='2',
               CARGO_INCREMENTAL='0', PYTHONDONTWRITEBYTECODE='1', CI='true',
               FLUTTER_SUPPRESS_ANALYTICS='true', DART_SUPPRESS_ANALYTICS='true')
    try:
        record()
        # Compile every real SDK-facing plugin unit before the expensive Rust
        # build. This uses the pinned public/client headers, never a fake SDK.
        engine = sdk / 'bin/cache/artifacts/engine/windows-x64'
        client = engine / 'cpp_client_wrapper/include'
        if not (client / 'flutter/plugin_registrar_windows.h').is_file():
            raise ValueError('precache the pinned Windows C++ client wrapper before qualification')
        objects = output / 'plugin-sdk-objects'
        objects.mkdir()
        registration = objects / 'generated-registration-contract.cpp'
        registration.write_text(
            '#include <rhwp_native_reference/rhwp_d3d_plugin.h>\n'
            'void RegisterQualificationPlugin(FlutterDesktopPluginRegistrarRef registrar) {\n'
            '  RhwpD3dPluginRegisterWithRegistrar(registrar);\n}\n', encoding='utf-8')
        for source in [registration, *[repo / 'bindings/Flutter/windows' / filename
                                     for filename in ['rhwp_d3d_plugin.cpp', 'Producer.cpp', 'NativeApi.cpp']]]:
            run_step(['cl', '/nologo', '/c', '/std:c++17', '/EHsc', '/W4', '/WX',
                      '/DRHWP_NATIVE_REFERENCE_PLUGIN_IMPL', '/DNOMINMAX', '/DWIN32_LEAN_AND_MEAN',
                      '/I' + str(engine), '/I' + str(client),
                      '/I' + str(repo / 'bindings/Flutter/windows/include'),
                      '/I' + str(repo / 'bindings/swift/Sources/CRhwpNative'),
                      str(source), '/Fo' + str(objects / (source.name + '.obj'))],
                     cwd=repo, env=env, output=output, receipt=state, record=record, timeout=120)
        native_receipt = output / 'native-build-receipt.json'
        if args.native_artifact_root:
            from native_artifact_provenance import load_native_artifact
            reused = load_native_artifact(args.native_artifact_root, repo, 'windows', args.native_source_head)
            directory = output / 'native-artifact'
            directory.mkdir(mode=0o700)
            native = directory / 'rhwp_native_ffi.dll'
            shutil.copyfile(reused['native'], native)
            shutil.copyfile(reused['nativeReceipt'], native_receipt)
            if digest(native) != digest(reused['native']):
                raise ValueError('Native artifact changed while copying')
            state.update(nativeRebuilt=False, nativeSourceHead=args.native_source_head,
                         nativeReuse=reused['proof'])
        else:
            command = ['rustup', 'run', '1.93.1', 'cargo', 'build', '--locked', '--release',
                       '-p', 'rhwp-native-ffi', '--features', 'native-skia-d3d',
                       '--target', 'x86_64-pc-windows-msvc']
            run_step(command, cwd=repo, env=env, output=output, receipt=state, record=record,
                     timeout=5400)
            native = output / 'native-target/x86_64-pc-windows-msvc/release/rhwp_native_ffi.dll'
            write_json(native_receipt, {'platform': 'windows', 'source_head': source_head, 'build': 'passed',
                                       'target': 'x86_64-pc-windows-msvc', 'features': ['native-skia-d3d'],
                                       'artifacts': {'rhwp_native_ffi.dll': digest(native)}})
            state.update(nativeRebuilt=True, nativeSourceHead=source_head)
        record()
        build = output / 'desktop-build'
        run_step([sys.executable, '-B', str(repo / 'bindings/Flutter/tool/build_desktop_probe.py'),
                  '--flutter-sdk', str(sdk), '--native', str(native),
                  '--native-receipt', str(native_receipt), '--output', str(build)],
                 cwd=repo, env=env, output=output, receipt=state, record=record)
        proof = json.loads((build / 'build-receipt.json').read_text(encoding='utf-8'))
        artifacts = proof['artifacts']
        capture = output / 'desktop-capture'
        run_step([sys.executable, '-B', str(repo / 'bindings/Flutter/tool/qualify_desktop_texture.py'),
                  '--runner', artifacts['runner']['path'], '--native', artifacts['native']['path'],
                  '--dispatcher', artifacts['dispatcher']['path'],
                  '--fixture', artifacts['fixture']['path'],
                  '--build-receipt', str(build / 'build-receipt.json'), '--cpu-reference', '--output', str(capture)],
                 cwd=repo, env=env, output=output, receipt=state, record=record, timeout=300)
        # A separate process starts with an empty plugin-owned frame budget.
        # Legacy immutable credits intentionally cannot be refunded by close.
        continuous = output / 'continuous-capture'
        run_step([sys.executable, '-B', str(repo / 'bindings/Flutter/tool/qualify_windows_continuous.py'),
                  '--runner', artifacts['runner']['path'], '--native', artifacts['native']['path'],
                  '--dispatcher', artifacts['dispatcher']['path'],
                  '--fixture', artifacts['fixture']['path'],
                  '--build-receipt', str(build / 'build-receipt.json'), '--frames', '32',
                  '--timeout', '600', '--output', str(continuous)],
                 cwd=repo, env=env, output=output, receipt=state, record=record, timeout=630)
        if subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip() != source_head:
            raise ValueError('source head changed during qualification')
        if subprocess.check_output(['git', 'status', '--porcelain'], cwd=repo, text=True):
            raise ValueError('source checkout changed during qualification')
        state.update(build='passed', native=str(native), nativeSha256=digest(native),
                     nativeReceipt=str(native_receipt), desktopCapture=str(capture),
                     continuousCapture=str(continuous), boundedConsumerExecutionVerified=True,
                     continuousConsumerExecutionVerified=True, continuousPresentationSupported=True)
    except BaseException as error:
        state.update(build='failed', error=f'{type(error).__name__}: {error}')
        raise
    finally:
        state['finished'] = time.time()
        record()
    print(receipt_path)


if __name__ == '__main__':
    main()
