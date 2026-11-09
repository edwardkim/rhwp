import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native.dart';

void main() {
  late Directory temporary, root;
  late String architecture, nativeName, ownerName;
  late Map<String, Object?> manifest;

  Uint8List header(bool shared, {String? arch}) {
    final result = Uint8List(128), data = ByteData(128);
    final selected = arch ?? architecture;
    if (Platform.isLinux) {
      result.setAll(0, [127, 69, 76, 70, 2, 1, 1]);
      data.setUint16(16, shared ? 3 : 2, Endian.little);
      data.setUint16(18, selected == 'x86_64' ? 62 : 183, Endian.little);
    } else if (Platform.isWindows) {
      result.setAll(0, [77, 90]);
      data.setUint32(60, 64, Endian.little);
      result.setAll(64, [80, 69, 0, 0]);
      data.setUint16(68, selected == 'x86_64' ? 0x8664 : 0xaa64, Endian.little);
      data.setUint16(86, shared ? 0x2000 : 0, Endian.little);
      data.setUint16(88, 0x20b, Endian.little);
    } else {
      result.setAll(0, [207, 250, 237, 254]);
      data.setUint32(
        4,
        selected == 'x86_64' ? 0x1000007 : 0x100000c,
        Endian.little,
      );
      data.setUint32(12, shared ? 6 : 2, Endian.little);
    }
    for (var i = 0; i < 128; i++) {
      result[i] |= data.getUint8(i);
    }
    return result;
  }

  File file(String path) => File('${root.path}/$path');
  Future<void> save() =>
      file(NativeRuntimeBundle.manifestName)
          .writeAsString(jsonEncode(manifest));
  Future<void> member(String path, List<int> data) async {
    final target = file(path);
    await target.parent.create(recursive: true);
    await target.writeAsBytes(data);
    final files = manifest['files']! as Map<String, Object?>;
    files[path] = {
      'kind': 'file',
      'bytes': data.length,
      'sha256': sha256.convert(data).toString(),
      'executable': false,
    };
  }

  setUp(() async {
    architecture = switch (Abi.current()) {
      Abi.linuxX64 || Abi.macosX64 || Abi.windowsX64 => 'x86_64',
      _ => 'arm64',
    };
    nativeName = Platform.isWindows
        ? 'rhwp_native_ffi.dll'
        : Platform.isMacOS
        ? 'librhwp_native_ffi.dylib'
        : 'librhwp_native_ffi.so';
    ownerName = Platform.isWindows
        ? 'rhwp_flutter_owner.dll'
        : Platform.isMacOS
        ? 'librhwp_flutter_owner.dylib'
        : 'librhwp_flutter_owner.so';
    temporary = await Directory.systemTemp.createTemp('rhwp-runtime-dart-');
    root = await Directory('${temporary.path}/bundle').create();
    manifest = {
      'kind': 'rhwp-flutter-desktop',
      'schemaVersion': 1,
      'platform': Platform.operatingSystem,
      'architecture': architecture,
      'sources': {
        'runtime': '1' * 40,
        'native': '2' * 40,
        'dispatcher': '3' * 40,
      },
      'abi': {'native': 1, 'dispatcher': 2},
      'runtime': 'app/runner',
      'native': 'native/$nativeName',
      'dispatcher': 'native/$ownerName',
      'files': <String, Object?>{
        'app': {'kind': 'directory'},
        'native': {'kind': 'directory'},
        'notices': {'kind': 'directory'},
      },
    };
    await member('app/runner', header(false));
    await member('native/$nativeName', header(true));
    await member('native/$ownerName', header(true));
    for (final name in [
      'LICENSE',
      'THIRD_PARTY.md',
      'THIRD_PARTY_LICENSES.md',
    ]) {
      await member('notices/$name', utf8.encode('supplied notice $name'));
    }
    await save();
  });
  tearDown(() async {
    await temporary.delete(recursive: true);
  });

  test('resolves all supplied libraries after bundle moves with no source tree', () async {
    root = await root.rename('${temporary.path}/relocated with spaces');
    final bundle = await NativeRuntimeBundle.load(root);
    expect(bundle.platform, Platform.operatingSystem);
    expect(bundle.architecture, architecture);
    expect(bundle.nativeLibrary.path, file('native/$nativeName').path);
    expect(bundle.dispatcherLibrary.path, file('native/$ownerName').path);
    expect(bundle.sourceRevisions['native'], '2' * 40);
    expect(
      () => bundle.sourceRevisions['native'] = 'x',
      throwsUnsupportedError,
    );
    // The header fixtures are not real binaries: load() must not execute them.
    expect(bundle.executable.path, file('app/runner').path);
  });

  test('rejects same-length corruption and unknown injected library', () async {
    final native = file('native/$nativeName');
    final data = await native.readAsBytes();
    data[100] ^= 1;
    await native.writeAsBytes(data);
    await expectLater(
      NativeRuntimeBundle.load(root),
      throwsA(isA<NativeCallException>()),
    );
    await member('native/$nativeName', header(true));
    await save();
    await file('app/injected.dll').writeAsString('extra');
    await expectLater(
      NativeRuntimeBundle.load(root),
      throwsA(isA<NativeCallException>()),
    );
  });

  test('requires current machine and real executable machine header', () async {
    final wrong = architecture == 'arm64' ? 'x86_64' : 'arm64';
    manifest['architecture'] = wrong;
    await save();
    await expectLater(
      NativeRuntimeBundle.load(root),
      throwsA(isA<NativeCallException>()),
    );
    manifest['architecture'] = architecture;
    await member('native/$nativeName', header(true, arch: wrong));
    await save();
    await expectLater(
      NativeRuntimeBundle.load(root),
      throwsA(isA<NativeCallException>()),
    );
  });

  test(
    'rejects executable role masquerading as library despite matching hash',
    () async {
      await member('native/$nativeName', header(false));
      await save();
      await expectLater(
        NativeRuntimeBundle.load(root),
        throwsA(isA<NativeCallException>()),
      );
    },
  );

  test(
    'rejects unsupported ABI, non-Git provenance and escaping manifest paths',
    () async {
      for (final change in <String, Object?>{
        'abi': {'native': 1, 'dispatcher': 1},
        'schemaVersion': true,
        'native': '../outside.so',
        'runtime': '/outside/app',
        'sources': {'native': '2' * 40},
      }.entries) {
        final original = manifest[change.key];
        manifest[change.key] = change.value;
        await save();
        await expectLater(
          NativeRuntimeBundle.load(root),
          throwsA(isA<NativeCallException>()),
        );
        manifest[change.key] = original;
      }
    },
  );

  test(
    'verifies notices and empty assets, without assuming optional source tools',
    () async {
      await member('app/empty', []);
      await save();
      expect((await NativeRuntimeBundle.load(root)).root.path, root.path);
      (manifest['files']! as Map).remove('notices/LICENSE');
      await file('notices/LICENSE').delete();
      await save();
      await expectLater(
        NativeRuntimeBundle.load(root),
        throwsA(isA<NativeCallException>()),
      );
    },
  );

  test(
    'revalidates before connect and never loads a changed fixture',
    () async {
      final bundle = await NativeRuntimeBundle.load(root);
      await file('native/$ownerName').writeAsString('corrupted before connect');
      await expectLater(bundle.connect(), throwsA(isA<NativeCallException>()));
    },
  );

  test('rejects an oversized manifest before decoding', () async {
    await file(NativeRuntimeBundle.manifestName)
        .writeAsBytes(Uint8List(1024 * 1024 + 1));
    await expectLater(
      NativeRuntimeBundle.load(root),
      throwsA(isA<NativeCallException>()),
    );
  });

  if (!Platform.isWindows) {
    test(
      'retains internal relative framework links but rejects escape/root links',
      () async {
        await Link('${root.path}/app/link').create('runner');
        (manifest['files']! as Map)['app/link'] = {
          'kind': 'link',
          'target': 'runner',
        };
        await save();
        expect(
          (await NativeRuntimeBundle.load(root)).architecture,
          architecture,
        );
        await Link('${root.path}/app/link').delete();
        await File('${temporary.path}/outside').writeAsString('outside');
        await Link('${root.path}/app/link').create('../../outside');
        await expectLater(
          NativeRuntimeBundle.load(root),
          throwsA(isA<NativeCallException>()),
        );
        final linked = Link('${temporary.path}/bundle-link');
        await linked.create(root.path);
        await expectLater(
          NativeRuntimeBundle.load(Directory(linked.path)),
          throwsA(isA<NativeCallException>()),
        );
      },
    );
  }
}
