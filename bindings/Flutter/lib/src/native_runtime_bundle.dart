import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';

import 'native_ffi.dart';

/// A verified, relocatable set of supplied desktop runtime files.
///
/// Hashes detect accidental changes; a manifest is not a signature or permission
/// to execute untrusted code. [load] never opens a native library. [connect]
/// explicitly loads the supplied code through the existing owner-thread shim.
class NativeRuntimeBundle {
  NativeRuntimeBundle._(
    this.root,
    this.platform,
    this.architecture,
    this.sourceRevisions,
    this.nativeLibrary,
    this.dispatcherLibrary,
    this.executable,
  );

  final Directory root;
  final String platform, architecture;
  final Map<String, String> sourceRevisions;
  final File nativeLibrary, dispatcherLibrary, executable;

  static const manifestName = 'rhwp-runtime.json';
  static const maxEntries = 8192;
  static const maxFileBytes = 512 * 1024 * 1024;
  static const maxBundleBytes = 2 * 1024 * 1024 * 1024;
  static const _names = {
    'linux': ['librhwp_native_ffi.so', 'librhwp_flutter_owner.so'],
    'windows': ['rhwp_native_ffi.dll', 'rhwp_flutter_owner.dll'],
    'macos': ['librhwp_native_ffi.dylib', 'librhwp_flutter_owner.dylib'],
  };

  /// Verify every file, internal relative link, target header and source label.
  /// Only the current desktop OS/ABI is admitted; this is not a cross-OS probe.
  static Future<NativeRuntimeBundle> load(Directory supplied) async {
    final before = await supplied.stat();
    if (await FileSystemEntity.type(supplied.path, followLinks: false) !=
            FileSystemEntityType.directory ||
        before.type != FileSystemEntityType.directory) {
      _invalid('Runtime root must be a directory, not a link.');
    }
    final root = Directory(await supplied.resolveSymbolicLinks());
    final manifest = File(_join(root, manifestName));
    final raw = await _boundedBytes(manifest, 1024 * 1024);
    final decoded = jsonDecode(utf8.decode(raw));
    if (decoded is! Map ||
        decoded['schemaVersion'] is! int ||
        decoded['schemaVersion'] != 1 ||
        decoded['kind'] != 'rhwp-flutter-desktop') {
      _invalid('Unsupported runtime manifest.');
    }
    final platform = decoded['platform'],
        architecture = decoded['architecture'];
    final actualArchitecture = switch (Abi.current()) {
      Abi.linuxX64 || Abi.macosX64 || Abi.windowsX64 => 'x86_64',
      Abi.linuxArm64 || Abi.macosArm64 || Abi.windowsArm64 => 'arm64',
      _ => '',
    };
    if (platform != Platform.operatingSystem ||
        !_names.containsKey(platform) ||
        architecture != actualArchitecture ||
        actualArchitecture.isEmpty) {
      _invalid('Runtime platform/architecture differs from this host.');
    }
    final sources = decoded['sources'];
    if (sources is! Map ||
        sources.length != 3 ||
        !['runtime', 'native', 'dispatcher'].every(
          (key) =>
              sources[key] is String &&
              RegExp(r'^[0-9a-f]{40}$').hasMatch(sources[key] as String),
        )) {
      _invalid('Missing declared source identities.');
    }
    final files = decoded['files'];
    if (files is! Map || files.isEmpty || files.length > maxEntries) {
      _invalid('Invalid bounded runtime file manifest.');
    }
    final actual = await _scan(root);
    final names = files.keys.map((key) {
      if (key is! String) _invalid('Invalid manifest member.');
      return _relative(key);
    }).toSet();
    if (names.contains(manifestName) ||
        actual.length != names.length + 1 ||
        !actual.keys.every(
          (name) => name == manifestName || names.contains(name),
        )) {
      _invalid('Runtime members differ from the manifest.');
    }
    var total = 0;
    for (final name in names) {
      final expected = files[name];
      if (expected is! Map) _invalid('Invalid runtime member description.');
      final type = actual[name];
      if (expected['kind'] == 'directory') {
        if (type != FileSystemEntityType.directory || expected.length != 1) {
          _invalid('Runtime directory differs from the manifest.');
        }
      } else if (expected['kind'] == 'link') {
        if (type != FileSystemEntityType.link || expected.length != 2) {
          _invalid('Runtime link differs from the manifest.');
        }
        final target = await Link(_join(root, name)).target();
        if (_relative(target) != expected['target']) {
          _invalid('Runtime link target differs from the manifest.');
        }
      } else if (expected['kind'] == 'file') {
        if (type != FileSystemEntityType.file || expected.length != 4) {
          _invalid('Runtime member must be a regular file.');
        }
        final size = expected['bytes'], digest = expected['sha256'];
        if (size is! int ||
            size < 0 ||
            size > maxFileBytes ||
            digest is! String ||
            !RegExp(r'^[0-9a-f]{64}$').hasMatch(digest) ||
            expected['executable'] is! bool) {
          _invalid('Invalid runtime file size/digest/mode.');
        }
        total += size;
        if (total > maxBundleBytes) _invalid('Runtime exceeds its byte bound.');
        final file = File(_join(root, name));
        final stamp = await file.stat();
        if (stamp.size != size ||
            (Platform.isWindows ? false : stamp.mode & 0x49 != 0) !=
                (Platform.isWindows ? false : expected['executable'])) {
          _invalid('Runtime file size/mode mismatch.');
        }
        final sum = await sha256.bind(file.openRead(0, size + 1)).first;
        if (sum.toString() != digest || !_same(stamp, await file.stat())) {
          _invalid('Runtime file changed or digest mismatched.');
        }
      } else {
        _invalid('Unknown runtime member kind.');
      }
    }
    final runtime = decoded['runtime'];
    final native = 'native/${_names[platform]![0]}';
    final owner = 'native/${_names[platform]![1]}';
    final abi = decoded['abi'];
    if (runtime is! String ||
        !runtime.startsWith('app/') ||
        decoded['native'] != native ||
        decoded['dispatcher'] != owner ||
        abi is! Map ||
        abi.length != 2 ||
        abi['native'] is! int ||
        abi['native'] != 1 ||
        abi['dispatcher'] is! int ||
        abi['dispatcher'] != 2) {
      _invalid('Invalid runtime executable/ABI roles.');
    }
    for (final role in [runtime, native, owner]) {
      if (files[role] is! Map || (files[role] as Map)['kind'] != 'file') {
        _invalid('Missing regular executable role.');
      }
      await _binary(
        File(_join(root, role)),
        platform as String,
        architecture as String,
        role != runtime,
      );
    }
    for (final name in [
      'LICENSE',
      'THIRD_PARTY.md',
      'THIRD_PARTY_LICENSES.md',
    ]) {
      if (files['notices/$name'] is! Map ||
          (files['notices/$name'] as Map)['kind'] != 'file') {
        _invalid('Missing runtime redistribution notices.');
      }
    }
    if (!_equalScan(actual, await _scan(root))) {
      _invalid('Runtime changed during verification.');
    }
    return NativeRuntimeBundle._(
      root,
      platform as String,
      architecture as String,
      Map.unmodifiable(Map<String, String>.from(sources)),
      File(_join(root, native)),
      File(_join(root, owner)),
      File(_join(root, runtime)),
    );
  }

  /// Revalidate the movable bundle immediately before loading trusted code.
  /// Existing dispatcher and Native capability checks stay authoritative.
  Future<NativeOwnerTransport> connect() async {
    final verified = await load(root);
    final transport = await NativeOwnerTransport.connect(
      dispatcherLibrary: verified.dispatcherLibrary.path,
      nativeLibrary: verified.nativeLibrary.path,
    );
    if (transport.capabilities['ownerPresentationCleanup'] != true ||
        transport.capabilities['nativeSkia'] != true ||
        transport.capabilities['worker'] != true ||
        transport.capabilities['hostRequests'] != true) {
      await transport.close();
      throw const NativeCallException(
        'runtimeAbi',
        'Runtime lacks the worker/Skia/owner ABI 2 contract.',
      );
    }
    return transport;
  }

  static Never _invalid(String message) =>
      throw NativeCallException('runtimeBundle', message);

  static String _relative(String value) {
    if (value.isEmpty ||
        utf8.encode(value).length > 4096 ||
        value.contains('\\') ||
        value.contains(':') ||
        value.contains('\x00') ||
        value.startsWith('/') ||
        value
            .split('/')
            .any((part) => part.isEmpty || part == '.' || part == '..')) {
      _invalid('Invalid bundle-relative path.');
    }
    return value;
  }

  static String _join(Directory root, String relative) =>
      '${root.path}${Platform.pathSeparator}${_relative(relative).replaceAll('/', Platform.pathSeparator)}';
  static bool _same(FileStat a, FileStat b) =>
      a.type == b.type &&
      a.size == b.size &&
      a.mode == b.mode &&
      a.modified == b.modified &&
      a.changed == b.changed;
  static bool _equalScan(
    Map<String, FileSystemEntityType> a,
    Map<String, FileSystemEntityType> b,
  ) => a.length == b.length && a.keys.every((key) => a[key] == b[key]);

  static Future<Uint8List> _boundedBytes(File file, int maximum) async {
    if (await FileSystemEntity.type(file.path, followLinks: false) !=
        FileSystemEntityType.file) {
      _invalid('Manifest must be a regular file.');
    }
    final before = await file.stat();
    if (before.size > maximum) _invalid('Runtime manifest exceeds its bound.');
    final input = await file.open();
    try {
      final data = await input.read(maximum + 1);
      if (data.length > maximum ||
          data.length != before.size ||
          !_same(before, await file.stat())) {
        _invalid('Runtime manifest changed during read.');
      }
      return data;
    } finally {
      await input.close();
    }
  }

  static Future<Map<String, FileSystemEntityType>> _scan(Directory root) async {
    final found = <String, FileSystemEntityType>{};
    final folded = <String>{};
    final pending = <(Directory, int)>[(root, 0)];
    final prefix = '${root.path}${Platform.pathSeparator}';
    while (pending.isNotEmpty) {
      final (directory, depth) = pending.removeLast();
      if (depth > 32) _invalid('Runtime directory depth exceeds its bound.');
      await for (final item in directory.list(followLinks: false)) {
        final name = _relative(
          item.path
              .substring(prefix.length)
              .replaceAll(Platform.pathSeparator, '/'),
        );
        if (!folded.add(name.toLowerCase()) || found.length >= maxEntries) {
          _invalid('Duplicate or excessive runtime members.');
        }
        final canonical = await item.resolveSymbolicLinks();
        if (!canonical.startsWith(prefix)) {
          _invalid('Runtime member escapes root.');
        }
        final type = await FileSystemEntity.type(item.path, followLinks: false);
        if (type == FileSystemEntityType.directory) {
          pending.add((Directory(item.path), depth + 1));
        } else if (type == FileSystemEntityType.link) {
          _relative(await Link(item.path).target());
        } else if (type != FileSystemEntityType.file) {
          _invalid('Runtime member is not a regular file/directory/link.');
        }
        found[name] = type;
      }
    }
    return found;
  }

  static Future<void> _binary(
    File file,
    String platform,
    String architecture,
    bool shared,
  ) async {
    final input = await file.open();
    try {
      final length = await input.length();
      Future<ByteData> at(int offset, int size) async {
        if (offset < 0 || size < 0 || offset + size > length) {
          _invalid('Truncated executable header.');
        }
        await input.setPosition(offset);
        final bytes = await input.read(size);
        if (bytes.length != size) _invalid('Truncated executable header.');
        return ByteData.sublistView(bytes);
      }

      final h = await at(0, length < 64 ? length : 64);
      bool magic(ByteData data, List<int> values) =>
          data.lengthInBytes >= values.length &&
          List.generate(values.length, (i) => data.getUint8(i)).toString() ==
              values.toString();
      if (platform == 'linux') {
        if (h.lengthInBytes < 64 ||
            !magic(h, [127, 69, 76, 70, 2, 1, 1]) ||
            h.getUint16(18, Endian.little) !=
                (architecture == 'x86_64' ? 62 : 183) ||
            !(shared ? [3] : [2, 3]).contains(h.getUint16(16, Endian.little))) {
          _invalid('ELF target/library type mismatch.');
        }
      } else if (platform == 'windows') {
        if (h.lengthInBytes < 64 || !magic(h, [77, 90])) {
          _invalid('Invalid PE header.');
        }
        final offset = h.getUint32(60, Endian.little);
        if (offset > 1024 * 1024) {
          _invalid('PE header offset exceeds its bound.');
        }
        final pe = await at(offset, 26);
        if (!magic(pe, [80, 69, 0, 0]) ||
            pe.getUint16(24, Endian.little) != 0x20b ||
            pe.getUint16(4, Endian.little) !=
                (architecture == 'x86_64' ? 0x8664 : 0xaa64) ||
            (pe.getUint16(22, Endian.little) & 0x2000 != 0) != shared) {
          _invalid('PE target/library type mismatch.');
        }
      } else {
        int cpu(String arch) => arch == 'x86_64' ? 0x1000007 : 0x100000c;
        Future<int> macho(int offset, int size) async {
          final header = await at(offset, 32);
          if (size < 32 ||
              !magic(header, [207, 250, 237, 254]) ||
              !(shared ? [6, 8] : [2]).contains(
                header.getUint32(12, Endian.little),
              )) {
            _invalid('Mach-O library type mismatch.');
          }
          return header.getUint32(4, Endian.little);
        }

        if (magic(h, [207, 250, 237, 254])) {
          if (await macho(0, length) != cpu(architecture)) {
            _invalid('Mach-O target mismatch.');
          }
        } else {
          final fat64 = magic(h, [202, 254, 186, 191]);
          if (!fat64 && !magic(h, [202, 254, 186, 190])) {
            _invalid('Invalid Mach-O header.');
          }
          if (h.lengthInBytes < 8) _invalid('Truncated fat Mach-O header.');
          final count = h.getUint32(4, Endian.big), width = fat64 ? 32 : 20;
          if (count < 1 || count > 16) {
            _invalid('Invalid fat Mach-O slice count.');
          }
          var matched = false;
          final spans = <(int, int)>[];
          for (var i = 0; i < count; i++) {
            final entry = await at(8 + i * width, width);
            final offset = fat64
                ? entry.getUint64(8, Endian.big)
                : entry.getUint32(8, Endian.big);
            final size = fat64
                ? entry.getUint64(16, Endian.big)
                : entry.getUint32(12, Endian.big);
            if (offset < 8 + count * width ||
                size < 32 ||
                offset + size > length ||
                spans.any(
                  (span) => offset < span.$2 && span.$1 < offset + size,
                )) {
              _invalid('Invalid fat Mach-O slice range.');
            }
            final actual = await macho(offset, size);
            if (actual != entry.getUint32(0, Endian.big)) {
              _invalid('Fat Mach-O target differs from slice.');
            }
            matched |= actual == cpu(architecture);
            spans.add((offset, offset + size));
          }
          if (!matched) _invalid('Missing Mach-O target slice.');
        }
      }
    } finally {
      await input.close();
    }
  }
}
