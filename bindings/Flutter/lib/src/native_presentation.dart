import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

import 'native_ffi.dart';

/// Custom worker transports must explicitly attest the same process-resident
/// native module identity. The standard NativeOwnerTransport exposes this path
/// directly; a capability bit alone is insufficient to mix opaque IDs safely.
abstract interface class NativePresentationOwnerIdentity {
  String get nativeLibraryPath;
}

/// Complete P89 source identity, including the configured view and device.
final class NativePresentationStamp {
  const NativePresentationStamp({
    required this.documentId,
    required this.revision,
    required this.fontGeneration,
    required this.viewGeneration,
    required this.deviceGeneration,
    required this.deviceScale,
  });

  factory NativePresentationStamp.fromJson(Map<String, Object?> value) {
    int integer(String name, {bool positive = false}) {
      final number = value[name];
      if (number is! int || number < (positive ? 1 : 0)) {
        throw const NativeCallException(
          'invalidStamp',
          'Invalid presentation stamp.',
        );
      }
      return number;
    }

    final scale = value['deviceScale'];
    if (scale is! num || !scale.isFinite || scale <= 0 || scale > 16) {
      throw const NativeCallException('invalidStamp', 'Invalid device scale.');
    }
    return NativePresentationStamp(
      documentId: integer('documentId', positive: true),
      revision: integer('revision'),
      fontGeneration: integer('fontGeneration'),
      viewGeneration: integer('viewGeneration', positive: true),
      deviceGeneration: integer('deviceGeneration', positive: true),
      deviceScale: scale.toDouble(),
    );
  }

  final int documentId,
      revision,
      fontGeneration,
      viewGeneration,
      deviceGeneration;
  final double deviceScale;

  Map<String, Object?> toJson() => {
    'documentId': documentId,
    'revision': revision,
    'fontGeneration': fontGeneration,
    'viewGeneration': viewGeneration,
    'deviceGeneration': deviceGeneration,
    'deviceScale': deviceScale,
  };

  bool matches(Object? value) {
    if (value is! Map) return false;
    return toJson().entries.every((entry) => value[entry.key] == entry.value);
  }
}

/// Only small, explicitly cross-thread P89 metadata calls live here. No legacy
/// HOST/worker/buffer call, CPU replay, GPU pointer or pixel payload is exposed.
abstract interface class NativePresentation {
  static Future<NativePresentation> connect({
    required String nativeLibrary,
  }) async {
    final path = await File(nativeLibrary).resolveSymbolicLinks();
    return _FfiPresentation(path, DynamicLibrary.open(path));
  }

  String get libraryPath;
  bool get available;
  Future<Map<String, Object?>> snapshotInfo(int snapshot);
  Future<bool> snapshotCheck(int snapshot, NativePresentationStamp stamp);

  /// Remains valid after document/widget disposal. The native owner dispatcher
  /// pins its module and this object never unloads it or owns a worker handle.
  Future<void> releaseSnapshot(int snapshot);
}

typedef _CapabilitiesNative = Pointer<Utf8> Function();
typedef _CapabilitiesDart = Pointer<Utf8> Function();
typedef _MetadataNative = Pointer<Utf8> Function(Pointer<Uint8>, UintPtr);
typedef _MetadataDart = Pointer<Utf8> Function(Pointer<Uint8>, int);
typedef _FreeNative = Void Function(Pointer<Utf8>);
typedef _FreeDart = void Function(Pointer<Utf8>);

final class _FfiPresentation implements NativePresentation {
  _FfiPresentation(this.libraryPath, this._library) {
    if (!_library.providesSymbol('rhwp_session_capabilities') ||
        !_library.providesSymbol('rhwp_string_free')) {
      throw const NativeCallException(
        'unsupportedPresentation',
        'Native metadata ABI is unavailable.',
      );
    }
    _free = _library.lookupFunction<_FreeNative, _FreeDart>('rhwp_string_free');
    final capabilities = _library
        .lookupFunction<_CapabilitiesNative, _CapabilitiesDart>(
          'rhwp_session_capabilities',
        );
    final value = _read(capabilities());
    final presentation = value['presentation'];
    available =
        presentation is Map &&
        presentation['workerRenderSnapshots'] == true &&
        presentation['crossThreadHandles'] == true &&
        presentation['fullStampChecks'] == true &&
        _library.providesSymbol('rhwp_presentation_request');
    if (_library.providesSymbol('rhwp_presentation_request')) {
      _request = _library.lookupFunction<_MetadataNative, _MetadataDart>(
        'rhwp_presentation_request',
      );
    }
  }

  @override
  final String libraryPath;
  final DynamicLibrary _library;
  late final _FreeDart _free;
  _MetadataDart? _request;
  @override
  late final bool available;

  Map<String, Object?> _read(Pointer<Utf8> pointer) {
    if (pointer == nullptr) {
      throw const NativeCallException(
        'invalidPresentationReply',
        'Native metadata returned no result.',
      );
    }
    try {
      final bytes = pointer.cast<Uint8>();
      var length = 0;
      while (length <= 1024 * 1024 && bytes[length] != 0) {
        length++;
      }
      if (length > 1024 * 1024) {
        throw const NativeCallException(
          'presentationLimit',
          'Native metadata reply exceeds its bound.',
        );
      }
      final result = jsonDecode(pointer.toDartString(length: length));
      if (result is! Map<String, dynamic>) {
        throw const NativeCallException(
          'invalidPresentationReply',
          'Expected a native metadata object.',
        );
      }
      final value = Map<String, Object?>.from(result);
      if (value['ok'] != true) {
        throw NativeCallException(
          '${value['code'] ?? 'nativeError'}',
          '${value['error'] ?? 'Native metadata failed.'}',
        );
      }
      return value;
    } finally {
      _free(pointer);
    }
  }

  Map<String, Object?> _call(
    Map<String, Object?> value, {
    bool requireFullChecks = true,
  }) {
    final request = _request;
    if (request == null || (requireFullChecks && !available)) {
      throw const NativeCallException(
        'unsupportedPresentation',
        'Full source/view/device checks are required.',
      );
    }
    final bytes = utf8.encode(jsonEncode(value));
    if (bytes.length > 64 * 1024) {
      throw const NativeCallException(
        'presentationLimit',
        'Metadata request exceeds its bound.',
      );
    }
    final pointer = calloc<Uint8>(bytes.length);
    try {
      pointer.asTypedList(bytes.length).setAll(0, bytes);
      return _read(request(pointer, bytes.length));
    } finally {
      calloc.free(pointer);
    }
  }

  void _id(int snapshot) {
    if (snapshot <= 0) throw ArgumentError.value(snapshot, 'snapshot');
  }

  @override
  Future<Map<String, Object?>> snapshotInfo(int snapshot) async {
    _id(snapshot);
    final info = _call({'op': 'snapshotInfo', 'snapshot': snapshot})['info'];
    if (info is! Map) {
      throw const NativeCallException(
        'invalidPresentationReply',
        'Missing snapshot metadata.',
      );
    }
    return Map<String, Object?>.from(info);
  }

  @override
  Future<bool> snapshotCheck(
    int snapshot,
    NativePresentationStamp stamp,
  ) async {
    _id(snapshot);
    final result = _call({
      'op': 'snapshotCheck',
      'snapshot': snapshot,
      'stamp': stamp.toJson(),
    });
    if (result['current'] is! bool) {
      throw const NativeCallException(
        'invalidPresentationReply',
        'Missing full-stamp validation result.',
      );
    }
    return result['current']! as bool;
  }

  @override
  Future<void> releaseSnapshot(int snapshot) async {
    _id(snapshot);
    // An already owned handle still needs release when a feature probe fails;
    // this destructor-like operation does not require current source checks.
    _call({
      'op': 'snapshotRelease',
      'snapshot': snapshot,
    }, requireFullChecks: false);
  }
}
