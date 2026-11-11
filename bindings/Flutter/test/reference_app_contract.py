#!/usr/bin/env python3
"""Launcher contracts; header fixtures are not renderer or GPU evidence."""

import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tool'))
import reference_app
import runtime_bundle_contract as fixtures


class ReferenceAppContracts(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.RuntimeBundleContracts()
        self.fixture.setUp()
        self.addCleanup(self.fixture.tearDown)

    def test_current_desktop_mapping_and_unknown_hosts(self):
        for system, machine, expected in [('linux', 'x86_64', ('linux', 'x86_64')),
                                          ('linux', 'aarch64', ('linux', 'arm64')),
                                          ('win32', 'AMD64', ('windows', 'x86_64')),
                                          ('darwin', 'arm64', ('macos', 'arm64'))]:
            self.assertEqual(reference_app.host_target(system, machine), expected)
        for system, machine in [('freebsd', 'x86_64'), ('linux', 'i686'), ('win32', 'ARM')]:
            with self.assertRaises(ValueError):
                reference_app.host_target(system, machine)

    def test_relocated_plan_has_no_build_source_or_shell_dependency(self):
        f = self.fixture
        if os.name == 'nt':
            f.configure('windows')
        f.package()
        moved = f.root / "moved app ' ; $(echo untouched)"
        f.output.rename(moved)
        shutil.rmtree(f.runtime)
        f.native.unlink()
        f.dispatcher.unlink()
        command, cwd, env = reference_app.launch_plan(
            moved, system='win32' if os.name == 'nt' else 'linux', machine='x86_64',
            environment={'RHWP_RUNTIME_BUNDLE': '/old/path', 'DISPLAY': ':123'})
        self.assertEqual(command, [str(moved / 'app/app')])
        self.assertEqual(cwd, moved / 'app')
        self.assertEqual(env, {'RHWP_RUNTIME_BUNDLE': str(moved), 'DISPLAY': ':123'})

    def test_foreign_bundle_is_rejected_before_execution(self):
        f = self.fixture
        f.package()
        for system, machine in [('win32', 'AMD64'), ('linux', 'aarch64')]:
            with self.assertRaisesRegex(ValueError, 'mismatch'):
                reference_app.launch_plan(f.output, system=system, machine=machine)

    def test_corrupt_asset_or_extra_module_prevents_launch(self):
        f = self.fixture
        f.package()
        asset = f.output / 'app/data/asset.bin'
        old = asset.read_bytes()
        asset.write_bytes(b'x' * len(old))
        with patch.object(reference_app.subprocess, 'call') as execute, patch.object(
                reference_app, 'host_target', return_value=('linux', 'x86_64')):
            with self.assertRaisesRegex(ValueError, 'digest, size, or mode mismatch'):
                reference_app.launch(f.output)
            execute.assert_not_called()
        asset.write_bytes(old)
        (f.output / 'app/unlisted.so').write_bytes(b'not in build')
        with self.assertRaises(ValueError):
            reference_app.launch_plan(f.output, system='linux', machine='x86_64')

    @unittest.skipIf(os.name == 'nt', 'POSIX executable permission and symlink contract')
    def test_nonexecutable_entry_point_and_link_root_reject(self):
        f = self.fixture
        manifest = f.package()
        executable = f.output / 'app/app'
        executable.chmod(0o644)
        manifest['files']['app/app']['executable'] = False
        f.save_manifest(manifest)
        with self.assertRaisesRegex(ValueError, 'not executable'):
            reference_app.launch_plan(f.output, system='linux', machine='x86_64')
        if os.name != 'nt':
            linked = f.root / 'linked'
            linked.symlink_to(f.output, target_is_directory=True)
            with self.assertRaises(ValueError):
                reference_app.launch_plan(linked, system='linux', machine='x86_64')

    @unittest.skipUnless(sys.platform == 'linux', 'actual small ELF launcher contract')
    def test_real_child_receives_moved_root_and_exit_status(self):
        f = self.fixture
        source = f.root / 'entry.c'
        source.write_text('#include <stdio.h>\n#include <stdlib.h>\n#include <unistd.h>\n'
                          'int main(void) { char cwd[8192]; if (!getcwd(cwd, sizeof(cwd))) return 99; '
                          'puts(getenv("RHWP_RUNTIME_BUNDLE")); puts(cwd); return 23; }\n')
        subprocess.run(['cc', '-Wall', '-Wextra', '-Werror', str(source),
                        '-o', str(f.runtime / 'app')], check=True, capture_output=True, timeout=30)
        f.package()
        moved = f.root / 'relocated actual child'
        f.output.rename(moved)
        shutil.rmtree(f.runtime)
        f.native.unlink()
        f.dispatcher.unlink()
        result = subprocess.run([sys.executable, '-B', reference_app.__file__, 'run',
                                 '--bundle', str(moved)], cwd=f.root,
                                capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 23, result.stderr)
        self.assertEqual(result.stdout.splitlines(), [str(moved), str(moved / 'app')])

    def test_known_runner_locations_and_missing_build(self):
        f = self.fixture
        runner = f.root / 'runner'
        for target, architecture, mode, suffix in [
                ('linux', 'x86_64', 'release', 'build/linux/x64/release/bundle/rhwp_reference_app'),
                ('windows', 'arm64', 'debug', 'build/windows/arm64/runner/Debug/rhwp_reference_app.exe')]:
            with self.assertRaises(ValueError):
                reference_app.built_runtime(runner, target, architecture, mode)
            path = runner / suffix
            path.parent.mkdir(parents=True)
            path.write_bytes(b'location only; never executed')
            directory, executable = reference_app.built_runtime(runner, target, architecture, mode)
            self.assertEqual(directory / executable, path)

    def test_generated_macos_reference_can_access_sibling_bundle_and_document_paths(self):
        f = self.fixture
        runner, example = f.root / 'runner', f.root / 'example'
        (runner / 'lib').mkdir(parents=True)
        (runner / 'lib/main.dart').write_text('discarded template')
        (example / 'lib').mkdir(parents=True)
        (example / 'lib/main.dart').write_text('unchanged reference app')
        (example / 'pubspec.yaml').write_text('    path: ../\n')
        macos = runner / 'macos/Runner'
        macos.mkdir(parents=True)
        for name in ('DebugProfile.entitlements', 'Release.entitlements'):
            (macos / name).write_bytes(plistlib.dumps({
                'com.apple.security.app-sandbox': True,
                'com.apple.security.network.client': True}))
        reference_app.configure_reference_runner(runner, example, 'macos')
        self.assertEqual((runner / 'lib/main.dart').read_text(), 'unchanged reference app')
        for path in macos.glob('*.entitlements'):
            self.assertEqual(plistlib.loads(path.read_bytes()), {
                'com.apple.security.app-sandbox': False,
                'com.apple.security.network.client': True})


if __name__ == '__main__':
    unittest.main()
