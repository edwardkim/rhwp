#!/usr/bin/env python3
"""Portable orchestration fixtures, never Linux SDK/Native/GPU execution proof."""
import copy
import json
import os
from pathlib import Path
import signal
import struct
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tool'))
import check_linux_desktop_sdk as runner


def capabilities():
    return {'ok': True, 'abiVersion': 1, 'nativeSkia': True, 'worker': True, 'hostRequests': True,
            'presentation': dict.fromkeys(('workerRenderSnapshots', 'crossThreadHandles', 'fullStampChecks', 'cpuBatchFrames'), True),
            'gpuGl': {'abiVersion': 1, 'available': True, 'hostFboRendering': True}}


class LinuxSdkRunner(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='rhwp-linux-sdk-contract-')
        self.root = Path(self.temporary.name)
        self.repo = self.root / 'source'; self.repo.mkdir()
        self.sdk = self.root / 'sdk'
        for path in ('bin/cache/dart-sdk/bin/dart', 'bin/cache/flutter_tools.snapshot',
                     'bin/cache/artifacts/engine/linux-x64/flutter_linux/flutter_linux.h',
                     'bin/cache/artifacts/engine/linux-x64/libflutter_linux_gtk.so'):
            file = self.sdk / path; file.parent.mkdir(parents=True, exist_ok=True)
            file.write_bytes(b'portable orchestration fixture, never executable')
        self.args = SimpleNamespace(flutter_sdk=self.sdk, output=self.root / 'output', cxx=sys.executable,
                                    step_timeout=60, compile_only=False, build_only=False,
                                    native_artifact_root=None, native_source_head=None)
        self.commands, self.environments = [], []
        self.sources = {'fixture': '100644:blob'}
        self.capabilities = capabilities()
        self.fail_capture = False
        self.change_source = False
        self.wrong_probe_hash = False
        self.wrong_build_hash = False
        self.wrong_plugin_machine = False
        self.fail_command_keyword = None

    def tearDown(self):
        self.temporary.cleanup()

    def execute(self, command, *, env, log_path, step, record, stop_failed_group=False, **unused):
        # The fake executor exercises receipt ordering/rejection only. It does
        # not invoke a compiler, Rust, Flutter, Native code or a compositor.
        self.commands.append(command); self.environments.append(env)
        nested = Path(command[2]).name in {
            'build_linux_probe.py', 'qualify_linux_texture.py', 'qualify_linux_tile_group.py',
        } if len(command) > 2 and command[0] == sys.executable else False
        self.assertEqual(env.get(runner.SDK_SUPERVISION_ENV), '1' if nested else None)
        self.assertTrue(stop_failed_group)
        if self.fail_command_keyword in command:
            Path(log_path).write_text('controlled child failure')
            step.update(exitCode=7, finished=time.time(), portableFakeExecutor=True)
            record()
            return 7
        output = self.args.output
        log = ''
        if '--machine' in command:
            log = json.dumps({'frameworkRevision': 'b' * 40, 'engineRevision': 'c' * 40})
        elif 'rustc' in command and '--verbose' in command:
            log = 'rustc 1.93.1 (fixture)\nhost: x86_64-unknown-linux-gnu\n'
        elif '--version' in command:
            log = 'portable fake compiler\n'
        elif command[0] == 'pkg-config':
            log = '3.24.38\n1.5.10\n' if '--modversion' in command else '-I/fixture -lgtk-3 -lepoxy\n'
        elif '-shared' in command:
            header = bytearray(64); header[:7] = b'\x7fELF\x02\x01\x01'
            struct.pack_into('<HH', header, 16, 3, 183 if self.wrong_plugin_machine else 62)
            Path(command[-1]).write_bytes(header)  # Metadata fixture, not executable.
        elif 'cargo' in command:
            native = output / f'native-target/{runner.TARGET}/debug/librhwp_native_ffi.so'
            native.parent.mkdir(parents=True); native.write_bytes(b'not an executable Native library')
        elif '--probe-native' in command:
            native = Path(command[command.index('--probe-native') + 1])
            proof = Path(command[command.index('--probe-output') + 1])
            proof.write_text(json.dumps({'scope': 'nativeFeatureProbeOnly',
                'nativeSha256': '0' * 64 if self.wrong_probe_hash else runner.digest(native),
                'capabilities': self.capabilities, 'consumerSamplingVerified': False,
                'consumerGpuCompletionVerified': False, 'hardwareAccelerationVerified': False}))
        elif any(part.endswith('/build_linux_probe.py') for part in command):
            build = Path(command[command.index('--output') + 1]); build.mkdir()
            native = Path(command[command.index('--native') + 1])
            artifacts = {name: {'path': str(build / name)} for name in ('runner', 'dispatcher', 'fixture')}
            artifacts['native'] = {'path': str(native), 'sha256': '0' * 64 if self.wrong_build_hash else runner.digest(native)}
            (build / 'build-receipt.json').write_text(json.dumps({'buildExitCode': 0,
                'sourceHead': 'a' * 40, 'nativeSourceHead': 'a' * 40,
                'flutterRevision': 'b' * 40, 'engineRevision': 'c' * 40,
                'probeKind': 'twoTileGroup' if '--tile-group' in command else 'singleTexture', 'artifacts': artifacts}))
        elif any('/qualify_linux_' in part for part in command):
            capture = Path(command[command.index('--output') + 1]); capture.mkdir()
            tiled = any(part.endswith('/qualify_linux_tile_group.py') for part in command)
            (capture / 'receipt.json').write_text(json.dumps({'qualification': 'failed' if self.fail_capture else
                'linux-two-tile-consumer-passed' if tiled else 'linux-consumer-passed', 'exitCode': 0,
                'consumerGpuCompletionVerified': False, 'hardwareAccelerationVerified': False}))
        if self.change_source:
            self.sources['changed'] = 'different blob'
        Path(log_path).write_text(log)
        step.update(exitCode=0, finished=time.time(), portableFakeExecutor=True)
        record()
        return 0

    def run_fixture(self):
        with mock.patch.object(runner, 'source_hashes', side_effect=lambda _: dict(self.sources)), \
             mock.patch.object(runner.subprocess, 'check_output', return_value='a' * 40 + '\n'):
            return runner.qualify(self.args, repository=self.repo, executor=self.execute)

    def receipt(self):
        return json.loads((self.args.output / 'linux-sdk-receipt.json').read_text())

    def test_compile_only_never_claims_native_or_consumer_execution(self):
        self.args.compile_only = True
        self.run_fixture()
        receipt = self.receipt()
        self.assertEqual(receipt['qualification'], 'linux-sdk-compiled')
        self.assertTrue(receipt['targetSdkCompileLinkVerified'])
        for key in ('nativeBuildVerified', 'nativeGlCapabilitiesVerified', 'probeBuildsVerified',
                    'singleConsumerSamplingVerified', 'tileConsumerSamplingVerified', 'hardwareAccelerationVerified'):
            self.assertFalse(receipt[key])
        self.assertFalse(any('cargo' in command or '--probe-native' in command for command in self.commands))

    def test_gl_build_uses_one_feature_and_isolated_outputs_before_probes(self):
        self.args.build_only = True
        self.run_fixture()
        commands = [command for command in self.commands if 'cargo' in command]
        self.assertEqual(len(commands), 1)
        self.assertEqual(commands[0][commands[0].index('--features') + 1], 'native-skia-gl')
        self.assertEqual(commands[0][commands[0].index('--target') + 1], runner.TARGET)
        self.assertEqual(self.environments[0]['CARGO_TARGET_DIR'], str(self.args.output / 'native-target'))
        receipt = self.receipt()
        self.assertTrue(receipt['nativeGlCapabilitiesVerified'])
        self.assertTrue(receipt['probeBuildsVerified'])
        self.assertFalse(receipt['singleConsumerSamplingVerified'])
        self.assertFalse(receipt['tileConsumerSamplingVerified'])

    def test_full_orchestration_requires_both_distinct_consumer_receipts(self):
        self.run_fixture()
        receipt = self.receipt()
        self.assertEqual(receipt['qualification'], 'linux-desktop-consumer-passed')
        self.assertTrue(receipt['singleConsumerSamplingVerified'])
        self.assertTrue(receipt['tileConsumerSamplingVerified'])
        self.assertFalse(receipt['consumerGpuCompletionVerified'])
        self.assertFalse(receipt['hardwareAccelerationVerified'])

    def test_cpu_library_cannot_turn_a_feature_build_receipt_into_gpu_pass(self):
        self.capabilities['gpuGl']['available'] = False
        with self.assertRaisesRegex(ValueError, 'enabled native-skia-gl'):
            self.run_fixture()
        receipt = self.receipt()
        self.assertEqual(receipt['build'], 'failed')
        self.assertTrue(receipt['targetSdkCompileLinkVerified'])
        self.assertFalse(receipt['nativeGlCapabilitiesVerified'])
        self.assertFalse(receipt['probeBuildsVerified'])

    def test_probe_and_build_must_bind_the_same_native_bytes(self):
        for field in ('wrong_probe_hash', 'wrong_build_hash'):
            with self.subTest(field=field):
                self.args.output = self.root / field
                setattr(self, field, True)
                with self.assertRaises(ValueError): self.run_fixture()
                self.assertEqual(self.receipt()['build'], 'failed')
                self.assertFalse(self.receipt()['singleConsumerSamplingVerified'])
                setattr(self, field, False)

    def test_capture_failure_does_not_erase_compile_or_claim_sampling(self):
        self.fail_capture = True
        with self.assertRaisesRegex(ValueError, 'actual consumer capture'):
            self.run_fixture()
        receipt = self.receipt()
        self.assertEqual(receipt['build'], 'failed')
        self.assertTrue(receipt['targetSdkCompileLinkVerified'])
        self.assertTrue(receipt['nativeGlCapabilitiesVerified'])
        self.assertFalse(receipt['singleConsumerSamplingVerified'])
        self.assertFalse(receipt['tileConsumerSamplingVerified'])

    def test_source_changes_abort_before_a_success_claim(self):
        self.change_source = True
        with self.assertRaisesRegex(ValueError, 'source checkout changed'):
            self.run_fixture()
        self.assertEqual(len(self.commands), 1)
        self.assertEqual(self.receipt()['build'], 'failed')

    def test_wrong_sdk_plugin_machine_never_claims_compile_success(self):
        self.args.compile_only = True
        self.wrong_plugin_machine = True
        with self.assertRaisesRegex(ValueError, 'SDK plugin does not match'):
            self.run_fixture()
        self.assertFalse(self.receipt()['targetSdkCompileLinkVerified'])

    def test_failed_compile_records_exit_before_any_success_claim(self):
        self.args.compile_only = True
        self.fail_command_keyword = '-shared'
        with self.assertRaisesRegex(RuntimeError, 'SDK step failed'):
            self.run_fixture()
        value = self.receipt()
        self.assertEqual(value['steps'][-1]['exitCode'], 7)
        self.assertFalse(value['targetSdkCompileLinkVerified'])
        self.assertEqual(value['build'], 'failed')

    def test_other_os_arch_and_incomplete_reuse_are_rejected(self):
        for system, machine in [('darwin', 'x86_64'), ('linux', 'aarch64')]:
            with self.assertRaises(RuntimeError): runner.require_host(system, machine)
        self.args.native_source_head = 'a' * 40
        with self.assertRaisesRegex(ValueError, 'both artifact root'):
            self.run_fixture()
        self.assertFalse(self.args.output.exists())

    def test_capability_types_and_full_stamp_contract_are_required(self):
        projected = runner.validate_capabilities(capabilities())
        self.assertEqual(runner.validate_capabilities(projected), projected)
        for path in [('abiVersion',), ('nativeSkia',), ('presentation', 'fullStampChecks'),
                     ('presentation', 'cpuBatchFrames'), ('gpuGl', 'available'), ('gpuGl', 'hostFboRendering')]:
            for replacement in (None, 1, 'true'):
                value = copy.deepcopy(capabilities())
                target = value
                for key in path[:-1]: target = target[key]
                target[path[-1]] = replacement
                if path == ('abiVersion',) and replacement == 1: continue
                with self.assertRaises(ValueError): runner.validate_capabilities(value)


@unittest.skipUnless(sys.platform == 'linux', 'Linux SDK process supervision uses Linux process groups')
class LinuxSdkProcessSupervision(unittest.TestCase):
    def test_nested_worker_stops_on_outer_and_inner_deadlines(self):
        wrapper = '''
import os, sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
from build_linux_probe import sdk_supervises_children
from check_macos_metal_sdk import execute_step
output = Path(sys.argv[2])
worker = """
import os, signal, sys, time
from pathlib import Path
signal.signal(signal.SIGTERM, signal.SIG_IGN)
Path(sys.argv[1]).write_text(f'{os.getpid()} {os.getpgrp()}')
time.sleep(20)
"""
execute_step([sys.executable, '-B', '-c', worker, str(output / 'worker.pid')],
             repository=output, env=dict(os.environ), log_path=output / 'worker.log',
             step={}, record=lambda: None, timeout=float(sys.argv[3]),
             inherit_process_group=sdk_supervises_children())
'''
        for inner, outer in ((10, 2), (2, 10)):
            with self.subTest(inner=inner, outer=outer), tempfile.TemporaryDirectory() as temp:
                output = Path(temp)
                step, marker = {}, output / 'worker.pid'
                command = [sys.executable, '-B', '-c', wrapper,
                           str(Path(runner.__file__).parent), temp, str(inner)]
                sent = []
                killpg = os.killpg

                def signal_owned_group(pid, signum):
                    self.assertEqual(pid, step['pid'])
                    # Check before forwarding anything. Failed waitid means
                    # the test must not send a signal to this numeric PGID.
                    status = os.waitid(os.P_PID, pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
                    if inner < outer:
                        self.assertIsNotNone(status, 'the failed wrapper should have exited')
                    sent.append(signum)
                    killpg(pid, signum)

                def run():
                    return runner.execute_step(command, repository=output,
                        env={**os.environ, runner.SDK_SUPERVISION_ENV: '1'},
                        log_path=output / 'wrapper.log', step=step, record=lambda: None,
                        timeout=outer, stop_grace=.1, stop_failed_group=True)

                def worker_alive(pid):
                    try:
                        state = Path(f'/proc/{pid}/stat').read_text().split(') ', 1)[1][0]
                    except FileNotFoundError:
                        return False
                    return state not in ('Z', 'X')

                try:
                    with mock.patch.object(os, 'killpg', side_effect=signal_owned_group):
                        if outer < inner:
                            with self.assertRaises(TimeoutError):
                                run()
                            self.assertIs(step['timedOut'], True)
                        else:
                            self.assertNotEqual(run(), 0)
                    self.assertEqual(sent, [signal.SIGTERM, signal.SIGKILL])
                    with self.assertRaises(ChildProcessError):
                        os.waitid(os.P_PID, step['pid'], os.WEXITED | os.WNOHANG | os.WNOWAIT)
                    worker_pid, group = map(int, marker.read_text().split())
                    self.assertEqual(group, step['pid'])
                    until = time.monotonic() + .5
                    while worker_alive(worker_pid) and time.monotonic() < until:
                        time.sleep(.02)
                    self.assertFalse(worker_alive(worker_pid), 'nested SDK worker survived cleanup')
                finally:
                    if marker.exists():
                        worker_pid, group = map(int, marker.read_text().split())
                        if worker_alive(worker_pid) and os.getpgid(worker_pid) == group:
                            os.kill(worker_pid, signal.SIGKILL)

    def test_never_signals_a_group_after_losing_its_leader(self):
        with tempfile.TemporaryDirectory() as temp:
            output, step = Path(temp), {}
            command = [sys.executable, '-B', '-c', 'raise SystemExit(9)']
            leader = subprocess.Popen(command, start_new_session=True)
            _, status = os.waitpid(leader.pid, 0)
            try:
                with mock.patch.object(subprocess, 'Popen', return_value=leader), \
                     mock.patch.object(os, 'killpg') as killpg:
                    with self.assertRaises(ChildProcessError):
                        runner.execute_step(command, repository=output, env=dict(os.environ),
                            log_path=output / 'failed.log', step=step, record=lambda: None,
                            timeout=5, stop_failed_group=True)
                    killpg.assert_not_called()
            finally:
                leader.returncode = os.waitstatus_to_exitcode(status)
            self.assertIsNone(step['exitCode'])


if __name__ == '__main__': unittest.main()
