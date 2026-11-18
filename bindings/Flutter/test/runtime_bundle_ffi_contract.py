#!/usr/bin/env python3
"""Linux C-ABI fixture relocation, not a real rhwp renderer or Flutter runner."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tool'))
import runtime_bundle

# The fixture deliberately implements only startup capability/owner shutdown.
# Successful execution must not be reported as document or Skia rendering.
NATIVE_FIXTURE = r'''
#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#ifndef SKIA
#define SKIA 1
#endif
static char *reply(const char *v) { size_t n=strlen(v)+1; char *p=malloc(n); if(p)memcpy(p,v,n); return p; }
char *rhwp_session_capabilities(void) {
  return reply(SKIA ? "{\"ok\":true,\"abiVersion\":1,\"nativeSkia\":true,\"hostRequests\":true,\"worker\":true}"
                    : "{\"ok\":true,\"abiVersion\":1,\"nativeSkia\":false,\"hostRequests\":true,\"worker\":true}");
}
void rhwp_string_free(char *v) { free(v); }
char *rhwp_native_host_request(const uint8_t *p,size_t n,const uint8_t *b,size_t z) {
  (void)p;(void)n;(void)b;(void)z;return reply("{\"ok\":false,\"code\":\"fixtureOnly\"}");
}
char *rhwp_session_buffer_copy(uint64_t h,uint8_t *p,size_t n) {
  (void)h;(void)p;(void)n;return reply("{\"ok\":false,\"code\":\"fixtureOnly\"}");
}
char *rhwp_session_buffer_release(uint64_t h) { (void)h;return reply("{\"ok\":true}"); }
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--dart', type=Path, required=True)
    args = parser.parse_args()
    if sys.platform != 'linux':
        parser.error('this explicit C-ABI fixture command requires Linux')
    package = Path(__file__).resolve().parents[1]
    tool = package / 'tool/runtime_bundle_smoke.dart'
    with tempfile.TemporaryDirectory(prefix='rhwp-runtime-ffi-') as directory:
        root = Path(directory)
        source = root / 'native.c'; source.write_text(NATIVE_FIXTURE)
        owner = root / 'owner.so'
        subprocess.run(['cc', '-std=c11', '-O2', '-fPIC', '-Wall', '-Wextra', '-Werror',
                        '-pthread', '-shared', str(package / 'src/owner_dispatcher.c'),
                        '-ldl', '-o', str(owner)], check=True, capture_output=True, text=True)
        runtime = root / 'runtime'; runtime.mkdir()
        # Real executable header and relocatable regular assets, not a Flutter build.
        shutil.copyfile('/bin/true', runtime / 'runner'); (runtime / 'runner').chmod(0o755)
        (runtime / 'asset.txt').write_text('fixture asset')
        notices = root / 'notices'; notices.mkdir()
        for name in runtime_bundle.NOTICES: (notices / name).write_text('C ABI fixture notice\n')
        architecture = runtime_bundle.binary_identity(owner, True)[1].pop()
        checks = []
        for label, skia in [('supported', 1), ('unsupported', 0)]:
            native = root / (label + '.so')
            subprocess.run(['cc', '-std=c11', '-O2', '-fPIC', '-Wall', '-Wextra', '-Werror',
                            '-shared', '-DSKIA=' + str(skia), str(source), '-o', str(native)],
                           check=True, capture_output=True, text=True)
            target = root / (label + '-package')
            runtime_bundle.package(runtime, 'runner', native, owner, notices, target,
                                   'linux', architecture, {name: '1' * 40 for name in ('runtime', 'native', 'dispatcher')})
            relocated = root / 'with spaces' / label
            relocated.parent.mkdir(exist_ok=True); target.rename(relocated)
            native.unlink()
            process = subprocess.run([str(args.dart), 'run', str(tool), str(relocated)],
                                     cwd=package, capture_output=True, text=True, timeout=30)
            if skia:
                assert process.returncode == 0, process.stderr[-2000:]
                value = json.loads(process.stdout)
                assert value['ok'] and value['ownerAbi2'] and value['nativeAbi'] == 1
                assert value['worker'] and value['nativeSkia'] and not value['presentationVerified']
                checks.append({'fixture': label, 'exitCode': 0, 'relocatedOwnerConnectAndClose': True})
            else:
                assert process.returncode != 0 and 'runtimeAbi' in process.stderr, process.stderr[-2000:]
                checks.append({'fixture': label, 'rejectedByActualCapabilities': True})
        bad_owner = root / 'bad-owner.so'; bad_source = root / 'bad-owner.c'
        bad_source.write_text('#include <stdint.h>\nuint32_t rhwp_flutter_owner_abi_version(void) {return 3;}\n')
        subprocess.run(['cc', '-shared', '-fPIC', str(bad_source), '-o', str(bad_owner)], check=True, capture_output=True)
        original = root / 'with spaces/supported'; native = original / 'native/librhwp_native_ffi.so'
        target = root / 'wrong-owner'
        runtime_bundle.package(runtime, 'runner', native, bad_owner, notices, target, 'linux', architecture,
                               {name: '1' * 40 for name in ('runtime', 'native', 'dispatcher')})
        process = subprocess.run([str(args.dart), 'run', str(tool), str(target)], cwd=package,
                                 capture_output=True, text=True, timeout=30)
        assert process.returncode != 0 and 'ownerAbi' in process.stderr, process.stderr[-2000:]
        checks.append({'fixture': 'owner-abi3', 'rejectedByActualOwnerAbi': True})
        print(json.dumps({'ok': True, 'actualDartFfi': True, 'actualDispatcher': True,
                          'nativeIsCAbiFixture': True, 'realRhwpRendererVerified': False,
                          'flutterRunnerVerified': False, 'checks': checks}))


if __name__ == '__main__':
    main()
