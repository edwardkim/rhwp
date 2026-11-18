import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';
import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:ffi/ffi.dart';

import 'native_document.dart';
import 'native_ffi.dart';

/// A stamp observed with the exact bytes read, rather than a later stat.
class NativeFileStamp {
  const NativeFileStamp(
    this.path,
    this.digest,
    this.size,
    this.modified,
    this.changed,
    this.mode,
  );
  final String path;
  final String digest;
  final int size;
  final DateTime modified;
  final DateTime changed;
  final int mode;
  bool matches(NativeFileStamp other) =>
      digest == other.digest &&
      size == other.size &&
      modified == other.modified &&
      changed == other.changed &&
      mode == other.mode;
  Map<String, Object?> toJson() => {
    'digest': digest,
    'size': size,
    'modifiedMicros': modified.microsecondsSinceEpoch,
    'changedMicros': changed.microsecondsSinceEpoch,
    'mode': mode,
  };
}

class NativeFileChanged implements Exception {
  const NativeFileChanged(this.path);
  final String path;
  @override
  String toString() => 'Destination changed since it was observed: $path';
}

/// Complete published bytes survive finalization or Native acknowledgement
/// failure. Inspect this path before retrying; this is not a failed write rollback.
class NativePublicationError implements Exception {
  const NativePublicationError(this.path, this.cause);
  final String path;
  final Object cause;
  bool get published => true;
  @override
  String toString() =>
      'Complete file published at $path; finalization failed: $cause';
}

class NativeSaveResult {
  const NativeSaveResult({
    required this.info,
    required this.cancelled,
    this.path,
    this.savedRevision,
  });
  final NativeDocumentInfo info;
  final bool cancelled;
  final String? path;
  final int? savedRevision;
}

typedef NativeLossApproval = Future<bool> Function(Map<String, Object?> report);
typedef NativeOverwriteApproval = Future<bool> Function(NativeFileStamp stamp);

/// Bounded file persistence for the existing worker protocol. File operations
/// use asynchronous dart:io; neither filesystem I/O nor dialogs hold the Native
/// document FIFO. The lifecycle host awaits [whenIdle] before replacing/closing.
class NativeFileStore {
  NativeFileStore(this.document);
  final NativeDocument document;
  String? _sourcePath;
  NativeFileStamp? _sourceStamp;
  int? _sourceDocument;
  Completer<void>? _active;
  (int, int)? _lastRecovery;
  String? get sourcePath =>
      document.info?.documentId == _sourceDocument ? _sourcePath : null;
  bool get isBusy => _active != null;
  Future<void> get whenIdle => _active?.future ?? Future<void>.value();

  Future<T> _serial<T>(Future<T> Function() action) async {
    if (isBusy) throw StateError('Finish the pending file operation first.');
    final done = _active = Completer<void>();
    try {
      return await action();
    } finally {
      _active = null;
      done.complete();
    }
  }

  Future<NativeDocumentInfo> openPath(
    String path, {
    NativeDocumentInfo? replacing,
  }) => _serial(() async {
    final expected = replacing ?? document.info;
    final read = await _read(path);
    final info = await document.openBytes(read.bytes, replacing: expected);
    _sourcePath = read.stamp.path;
    _sourceStamp = read.stamp;
    _sourceDocument = info.documentId;
    return info;
  });

  Future<NativeDocumentInfo> openRecovery(
    String path, {
    NativeDocumentInfo? replacing,
  }) => _serial(() async {
    final expected = replacing ?? document.info;
    final read = await _read(path, limit: maxReferenceBytes + _maxHeader + 12);
    final data = _decodeRecovery(read.bytes);
    final info = await document.openRecoveryBytes(data, replacing: expected);
    // Header sourcePath is provenance only. Never adopt it for overwrite.
    _sourcePath = null;
    _sourceStamp = null;
    _sourceDocument = info.documentId;
    return info;
  });

  Future<NativeSaveResult> save({
    String? destination,
    NativeLossApproval? approveLoss,
    NativeOverwriteApproval? approveOverwrite,
  }) => _serial(() async {
    final acceptedDocument = _acceptedDocument();
    final target = destination ?? sourcePath;
    if (target == null) throw StateError('A new document needs a destination.');
    final path = File(target).absolute.path;
    final extension = path.toLowerCase();
    final format = extension.endsWith('.hwpx')
        ? 'hwpx'
        : extension.endsWith('.hwp')
        ? 'hwp'
        : null;
    if (format == null) throw ArgumentError('Save as .hwp or .hwpx.');
    await document.refreshInfo();
    _sameDocument(acceptedDocument);
    NativeFileStamp? expected;
    if (path == sourcePath) {
      expected = _sourceStamp;
      await _checkExpected(path, expected);
    } else {
      expected = await _observe(path);
      if (expected != null &&
          (approveOverwrite == null || !await approveOverwrite(expected))) {
        return NativeSaveResult(info: document.info!, cancelled: true);
      }
    }
    _sameDocument(acceptedDocument);
    final prepared = await document.prepareSave(format);
    var published = false;
    Object? failure;
    try {
      if (prepared.requiresLossAcceptance &&
          (approveLoss == null || !await approveLoss(prepared.report))) {
        return NativeSaveResult(info: document.info!, cancelled: true);
      }
      _sameDocument(prepared.documentId);
      final observed = await _publish(path, prepared.bytes, expected);
      published = true;
      final current = await document.acknowledgeSave(prepared);
      _sourcePath = path;
      _sourceStamp = observed;
      _sourceDocument = prepared.documentId;
      return NativeSaveResult(
        info: current,
        cancelled: false,
        path: path,
        savedRevision: prepared.revision,
      );
    } catch (error) {
      failure = error;
      if (published && error is! NativePublicationError) {
        throw NativePublicationError(path, error);
      }
      rethrow;
    } finally {
      try {
        await document.cancelSave(prepared);
      } catch (_) {
        if (failure == null) rethrow;
      }
    }
  });

  /// One recovery artifact per dirty revision. The caller owns its timer and
  /// chooses a dedicated directory; no automatic deletion or source overwrite.
  Future<String?> writeRecovery(String directory) => _serial(() async {
    final acceptedDocument = _acceptedDocument();
    final info = await document.refreshInfo();
    _sameDocument(acceptedDocument);
    final key = (info.documentId, info.revision);
    if (!info.isDirty || _lastRecovery == key) return null;
    final prepared = await document.prepareSave('hwpx');
    String? publishedPath;
    Object? failure;
    try {
      final header = <String, Object?>{
        'version': 1,
        'format': 'hwpx',
        'revision': prepared.revision,
        'sourcePath': sourcePath,
        'sourceStamp': sourcePath == null ? null : _sourceStamp?.toJson(),
        'digest': sha256.convert(prepared.bytes).toString(),
        'byteLength': prepared.bytes.length,
        'report': prepared.report,
      };
      final encoded = utf8.encode(jsonEncode(header));
      if (encoded.length > _maxHeader) {
        throw StateError('Recovery header exceeds 64 KiB.');
      }
      final bytes = Uint8List(
        _magic.length + 4 + encoded.length + prepared.bytes.length,
      );
      bytes.setAll(0, _magic);
      ByteData.sublistView(bytes)
          .setUint32(_magic.length, encoded.length, Endian.little);
      bytes.setAll(_magic.length + 4, encoded);
      bytes.setAll(_magic.length + 4 + encoded.length, prepared.bytes);
      final folder = Directory(directory).absolute;
      if (await FileSystemEntity.type(folder.path, followLinks: false) ==
          FileSystemEntityType.notFound) {
        await folder.create(recursive: true);
      }
      if (await FileSystemEntity.type(folder.path, followLinks: false) !=
          FileSystemEntityType.directory) {
        throw ArgumentError('Recovery location must be a directory.');
      }
      var entries = 0, count = 0, total = 0;
      await for (final entry in folder.list(followLinks: false)) {
        if (++entries > 256) {
          throw StateError('Use a dedicated recovery directory.');
        }
        if (!entry.path.endsWith('.rhwp-recovery')) continue;
        if (await FileSystemEntity.type(entry.path, followLinks: false) !=
            FileSystemEntityType.file) {
          throw ArgumentError('Recovery entries must be regular files.');
        }
        ++count;
        total += (await entry.stat()).size;
      }
      if (count >= 32 || total + bytes.length > 256 * 1024 * 1024) {
        throw StateError(
          'Recovery directory budget exhausted; existing files are preserved.',
        );
      }
      _sameDocument(prepared.documentId);
      final nonce = List.generate(
        16,
        (_) => Random.secure().nextInt(256),
      ).map((v) => v.toRadixString(16).padLeft(2, '0')).join();
      final path =
          '${folder.path}${Platform.pathSeparator}$nonce.rhwp-recovery';
      await _publish(
        path,
        bytes,
        null,
        limit: maxReferenceBytes + _maxHeader + 12,
      );
      publishedPath = path;
      _lastRecovery = (prepared.documentId, prepared.revision);
      return path;
    } catch (error) {
      failure = error;
      rethrow;
    } finally {
      // A recovery artifact is never an acknowledged user save checkpoint.
      try {
        await document.cancelSave(prepared);
      } catch (error) {
        if (publishedPath != null) {
          throw NativePublicationError(publishedPath, error);
        }
        if (failure == null) rethrow;
      }
    }
  });

  int _acceptedDocument() {
    final info = document.info;
    if (document.isDisposed || info == null) {
      throw const NativeCallException(
        'staleDocument',
        'Create or open a document before file persistence.',
      );
    }
    return info.documentId;
  }

  void _sameDocument(int documentId) {
    if (document.isDisposed || document.info?.documentId != documentId) {
      throw const NativeCallException(
        'staleDocument',
        'File operation source was replaced or closed.',
      );
    }
  }
}

const _maxHeader = 64 * 1024;
final _magic = utf8.encode('RHWP-R1\n');

class _Read {
  const _Read(this.bytes, this.stamp);
  final Uint8List bytes;
  final NativeFileStamp stamp;
}

bool _sameStat(FileStat a, FileStat b) =>
    a.type == b.type &&
    a.size == b.size &&
    a.modified == b.modified &&
    a.changed == b.changed &&
    a.mode == b.mode;

Future<_Read> _read(String path, {int limit = maxReferenceBytes}) async {
  final file = File(path).absolute;
  if (await FileSystemEntity.type(file.path, followLinks: false) !=
      FileSystemEntityType.file) {
    throw ArgumentError('Document must be a regular file, not a symlink.');
  }
  final before = await file.stat();
  if (before.size > limit) throw StateError('Document exceeds the byte limit.');
  final opened = await file.open();
  late Uint8List bytes;
  try {
    bytes = await opened.read(limit + 1);
  } finally {
    await opened.close();
  }
  final after = await file.stat();
  if (await FileSystemEntity.type(file.path, followLinks: false) !=
          FileSystemEntityType.file ||
      !_sameStat(before, after) ||
      bytes.length != after.size) {
    throw NativeFileChanged(file.path);
  }
  if (bytes.length > limit) {
    throw StateError('Document exceeds the byte limit.');
  }
  return _Read(
    bytes,
    NativeFileStamp(
      file.path,
      sha256.convert(bytes).toString(),
      bytes.length,
      after.modified,
      after.changed,
      after.mode,
    ),
  );
}

Future<NativeFileStamp?> _observe(
  String path, {
  int limit = maxReferenceBytes,
}) async {
  if (await FileSystemEntity.type(path, followLinks: false) ==
      FileSystemEntityType.notFound) {
    return null;
  }
  return (await _read(path, limit: limit)).stamp;
}

Future<void> _checkExpected(
  String path,
  NativeFileStamp? expected, {
  int limit = maxReferenceBytes,
}) async {
  final actual = await _observe(path, limit: limit);
  if (expected == null
      ? actual != null
      : actual == null || !expected.matches(actual)) {
    throw NativeFileChanged(path);
  }
}

// Hard-link publication makes a new complete file visible atomically and fails
// if another writer won the destination name. No empty placeholder is exposed.
typedef _LinkNative = Int32 Function(Pointer<Utf8>, Pointer<Utf8>);
typedef _LinkDart = int Function(Pointer<Utf8>, Pointer<Utf8>);
typedef _WinLinkNative = Int32 Function(
  Pointer<Utf16>,
  Pointer<Utf16>,
  Pointer<Void>,
);
typedef _WinLinkDart = int Function(
  Pointer<Utf16>,
  Pointer<Utf16>,
  Pointer<Void>,
);
void _linkExclusive(String staged, String destination) {
  if (Platform.isWindows) {
    final create = DynamicLibrary.open('kernel32.dll')
        .lookupFunction<_WinLinkNative, _WinLinkDart>('CreateHardLinkW');
    final source = staged.toNativeUtf16(), target = destination.toNativeUtf16();
    try {
      if (create(target, source, nullptr) == 0) {
        throw FileSystemException(
          'Exclusive hard-link publication failed.',
          destination,
        );
      }
    } finally {
      calloc.free(source);
      calloc.free(target);
    }
  } else {
    final link = DynamicLibrary.process()
        .lookupFunction<_LinkNative, _LinkDart>('link');
    final source = staged.toNativeUtf8(), target = destination.toNativeUtf8();
    try {
      if (link(source, target) != 0) {
        throw FileSystemException(
          'Exclusive hard-link publication failed.',
          destination,
        );
      }
    } finally {
      calloc.free(source);
      calloc.free(target);
    }
  }
}

typedef _OpenNative = Int32 Function(Pointer<Utf8>, Int32);
typedef _OpenDart = int Function(Pointer<Utf8>, int);
typedef _FdNative = Int32 Function(Int32);
typedef _FdDart = int Function(int);
void _privateMode(String path) {
  final mode = DynamicLibrary.process().lookupFunction<_OpenNative, _OpenDart>(
    'chmod',
  );
  final name = path.toNativeUtf8();
  try {
    if (mode(name, 384) != 0) {
      throw FileSystemException(
        'Could not restrict staged file permissions.',
        path,
      );
    }
  } finally {
    calloc.free(name);
  }
}

void _syncDirectory(String directory) {
  // Windows has no portable directory fsync, matching the Native reference host.
  if (Platform.isWindows) return;
  final library = DynamicLibrary.process();
  final open = library.lookupFunction<_OpenNative, _OpenDart>('open');
  final sync = library.lookupFunction<_FdNative, _FdDart>('fsync');
  final close = library.lookupFunction<_FdNative, _FdDart>('close');
  final name = directory.toNativeUtf8();
  int descriptor;
  try {
    descriptor = open(name, 0);
  } finally {
    calloc.free(name);
  }
  if (descriptor < 0) {
    throw FileSystemException(
      'Could not open publication directory for flush.',
      directory,
    );
  }
  try {
    if (sync(descriptor) != 0) {
      throw FileSystemException(
        'Could not flush publication directory.',
        directory,
      );
    }
  } finally {
    close(descriptor);
  }
}

Future<NativeFileStamp> _publish(
  String path,
  Uint8List bytes,
  NativeFileStamp? expected, {
  int limit = maxReferenceBytes,
}) async {
  if (bytes.length > limit) {
    throw StateError('Serialized document exceeds the byte limit.');
  }
  await _checkExpected(path, expected, limit: limit);
  final parent = File(path).parent;
  // createTemp uses an exclusive random name in the destination filesystem.
  final stagingDirectory = await parent.createTemp('.rhwp-save-');
  final staged = File(
    '${stagingDirectory.path}${Platform.pathSeparator}document',
  );
  var published = false;
  try {
    final output = await staged.open(mode: FileMode.writeOnly);
    try {
      if (!Platform.isWindows) _privateMode(staged.path);
      await output.writeFrom(bytes);
      await output.flush();
    } finally {
      await output.close();
    }
    await _checkExpected(path, expected, limit: limit);
    if (expected == null) {
      _linkExclusive(staged.path, path);
      published = true;
      // Removing the second hard link changes ctime; observe the destination
      // only after this, or an immediate second save would see a false conflict.
      await staged.delete();
    } else {
      await staged.rename(path);
      published = true;
    }
    await Isolate.run(() => _syncDirectory(parent.path));
    final observed = (await _read(path, limit: limit)).stamp;
    if (observed.digest != sha256.convert(bytes).toString()) {
      throw NativeFileChanged(path);
    }
    return observed;
  } catch (error) {
    if (published) throw NativePublicationError(path, error);
    rethrow;
  } finally {
    try {
      await stagingDirectory.delete(recursive: true);
    } catch (error) {
      if (published) throw NativePublicationError(path, error);
      rethrow;
    }
  }
}

Uint8List _decodeRecovery(Uint8List bytes) {
  if (bytes.length < _magic.length + 4 ||
      !_magic.asMap().entries.every((e) => bytes[e.key] == e.value)) {
    throw const FormatException('Invalid recovery signature.');
  }
  final length = ByteData.sublistView(bytes)
      .getUint32(_magic.length, Endian.little);
  if (length > _maxHeader || bytes.length < _magic.length + 4 + length) {
    throw const FormatException('Invalid recovery header length.');
  }
  final end = _magic.length + 4 + length;
  final header = jsonDecode(utf8.decode(bytes.sublist(_magic.length + 4, end)));
  if (header is! Map ||
      header['version'] != 1 ||
      header['format'] != 'hwpx' ||
      header['revision'] is! int ||
      (header['revision']! as int) < 0 ||
      header['report'] is! Map ||
      (header['report'] as Map)['count'] is! int ||
      ((header['report'] as Map)['count']! as int) < 0 ||
      (header['sourcePath'] == null) != (header['sourceStamp'] == null) ||
      (header['sourcePath'] != null &&
          (header['sourcePath'] is! String || header['sourceStamp'] is! Map))) {
    throw const FormatException('Invalid recovery metadata.');
  }
  final payload = Uint8List.sublistView(bytes, end);
  if (payload.length > maxReferenceBytes ||
      header['byteLength'] != payload.length ||
      header['digest'] != sha256.convert(payload).toString()) {
    throw const FormatException(
      'Recovery payload length or checksum mismatch.',
    );
  }
  return payload;
}
