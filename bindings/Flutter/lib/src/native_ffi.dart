import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

const maxReferenceBytes = 16 * 1024 * 1024;

class NativeCallException implements Exception {
  const NativeCallException(this.code, this.message);
  final String code;
  final String message;
  @override
  String toString() => 'NativeCallException($code): $message';
}

class NativeDisposedException implements Exception {
  const NativeDisposedException();
  @override
  String toString() => 'The native document owner is closing.';
}

/// A worker API, not a promise that a Dart isolate owns an OS thread.
abstract interface class NativeWorkerTransport {
  Map<String, Object?> get capabilities;
  Future<int> createWorker();
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  });
  Future<Map<String, Object?>?> pollWorker(int worker);
  Future<Uint8List> copyAndReleaseBuffer(int buffer, int byteLength);
  Future<void> releaseBuffer(int buffer);
  Future<void> close();
}

final class _Reply extends Struct {
  @Uint64()
  external int requestId;
  @Int32()
  external int operation;
  @Int32()
  external int error;
  external Pointer<Utf8> json;
  external Pointer<Utf8> releaseJson;
  external Pointer<Uint8> bytes;
  @UintPtr()
  external int byteLength;
  @UintPtr()
  external int chargedBytes;
  external Pointer<_Reply> next;
}

typedef _CreateNative = Pointer<Void> Function(Pointer<Utf8>);
typedef _CreateDart = Pointer<Void> Function(Pointer<Utf8>);
typedef _SubmitNative = Int32 Function(
  Pointer<Void>,
  Uint64,
  Int32,
  Pointer<Uint8>,
  UintPtr,
  Pointer<Uint8>,
  UintPtr,
  Uint64,
  UintPtr,
);
typedef _SubmitDart = int Function(
  Pointer<Void>,
  int,
  int,
  Pointer<Uint8>,
  int,
  Pointer<Uint8>,
  int,
  int,
  int,
);
typedef _PollNative = Pointer<_Reply> Function(Pointer<Void>);
typedef _PollDart = Pointer<_Reply> Function(Pointer<Void>);
typedef _FreeNative = Void Function(Pointer<Void>, Pointer<_Reply>);
typedef _FreeDart = void Function(Pointer<Void>, Pointer<_Reply>);
typedef _ClaimNative = Uint64 Function(Pointer<Void>, Pointer<_Reply>);
typedef _ClaimDart = int Function(Pointer<Void>, Pointer<_Reply>);
typedef _CloseNative = Void Function(Pointer<Void>);
typedef _CloseDart = void Function(Pointer<Void>);
typedef _StatusNative = Int32 Function(Pointer<Void>);
typedef _StatusDart = int Function(Pointer<Void>);

class _OwnedReply {
  const _OwnedReply(this.json, this.bytes);
  final Map<String, Object?> json;
  final Uint8List? bytes;
}

/// All legacy rhwp calls, strings, buffers and Rust TLS stay on the shim's
/// native owner thread. Dart FFI only submits copies and polls completed data.
final class NativeOwnerTransport implements NativeWorkerTransport {
  NativeOwnerTransport._(this._library, this._owner, this.nativeLibraryPath) {
    _timer = Timer.periodic(const Duration(milliseconds: 4), (_) => _poll());
  }

  static Future<NativeOwnerTransport> connect({
    required String dispatcherLibrary,
    required String nativeLibrary,
  }) async {
    final dispatcherPath = await File(dispatcherLibrary).resolveSymbolicLinks();
    final nativePath = await File(nativeLibrary).resolveSymbolicLinks();
    final library = DynamicLibrary.open(dispatcherPath);
    final abi = library.lookupFunction<Uint32 Function(), int Function()>(
      'rhwp_flutter_owner_abi_version',
    );
    final ownerAbi = abi();
    if (ownerAbi != 1 && ownerAbi != 2) {
      throw const NativeCallException(
        'ownerAbi',
        'Unsupported owner dispatcher ABI.',
      );
    }
    for (final name in ['submit', 'poll', 'close', 'finished', 'destroy']) {
      if (!library.providesSymbol('rhwp_flutter_owner_$name')) {
        throw NativeCallException(
          'ownerAbi',
          'Missing owner dispatcher function: $name',
        );
      }
    }
    if (!library.providesSymbol('rhwp_flutter_reply_free')) {
      throw const NativeCallException(
        'ownerAbi',
        'Missing dispatcher reply deallocator.',
      );
    }
    final create = library.lookupFunction<_CreateNative, _CreateDart>(
      'rhwp_flutter_owner_create',
    );
    final path = nativePath.toNativeUtf8();
    Pointer<Void> owner;
    try {
      owner = create(path);
    } finally {
      calloc.free(path);
    }
    if (owner == nullptr) {
      throw const NativeCallException(
        'ownerCreate',
        'Could not create owner thread.',
      );
    }
    final transport = NativeOwnerTransport._(library, owner, nativePath);
    try {
      transport._capabilities = {
        ...(await transport._startup.future).json,
        'ownerPresentationCleanup':
            ownerAbi >= 2 &&
            library.providesSymbol('rhwp_flutter_reply_claim_snapshot'),
      };
      if (transport._capabilities['abiVersion'] != 1) {
        throw const NativeCallException(
          'nativeAbi',
          'Unsupported legacy native ABI.',
        );
      }
      return transport;
    } catch (_) {
      await transport.close();
      rethrow;
    }
  }

  final String nativeLibraryPath;
  final DynamicLibrary _library;
  final Pointer<Void> _owner;
  late final _SubmitDart _submit = _library
      .lookupFunction<_SubmitNative, _SubmitDart>('rhwp_flutter_owner_submit');
  late final _PollDart _nativePoll = _library
      .lookupFunction<_PollNative, _PollDart>('rhwp_flutter_owner_poll');
  late final _FreeDart _free = _library.lookupFunction<_FreeNative, _FreeDart>(
    'rhwp_flutter_reply_free',
  );
  late final _ClaimDart? _claimSnapshot =
      _library.providesSymbol('rhwp_flutter_reply_claim_snapshot')
      ? _library.lookupFunction<_ClaimNative, _ClaimDart>(
          'rhwp_flutter_reply_claim_snapshot',
        )
      : null;
  late final _CloseDart _nativeClose = _library
      .lookupFunction<_CloseNative, _CloseDart>('rhwp_flutter_owner_close');
  late final _StatusDart _finished = _library
      .lookupFunction<_StatusNative, _StatusDart>(
        'rhwp_flutter_owner_finished',
      );
  late final _StatusDart _destroy = _library
      .lookupFunction<_StatusNative, _StatusDart>('rhwp_flutter_owner_destroy');
  final _startup = Completer<_OwnedReply>();
  final _pending = <int, Completer<_OwnedReply>>{};
  late final Timer _timer;
  final _closed = Completer<void>();
  Map<String, Object?> _capabilities = const {};
  bool _closing = false;
  bool _destroyed = false;
  int _serial = 0;

  @override
  Map<String, Object?> get capabilities => Map.unmodifiable(_capabilities);

  Future<_OwnedReply> _request(
    int operation, {
    Map<String, Object?>? request,
    Uint8List? data,
    int handle = 0,
    int bufferLength = 0,
  }) {
    if (_closing || _destroyed) throw const NativeDisposedException();
    final encoded = request == null
        ? Uint8List(0)
        : utf8.encode(jsonEncode(request));
    if (encoded.length > 1024 * 1024 ||
        (data?.length ?? 0) > maxReferenceBytes ||
        bufferLength < 0 ||
        bufferLength > maxReferenceBytes) {
      throw const NativeCallException(
        'byteLimit',
        'Reference request exceeds its byte limit.',
      );
    }
    final jsonPointer = encoded.isEmpty
        ? nullptr.cast<Uint8>()
        : calloc<Uint8>(encoded.length);
    final dataPointer = data == null || data.isEmpty
        ? nullptr.cast<Uint8>()
        : calloc<Uint8>(data.length);
    final id = ++_serial;
    final completer = Completer<_OwnedReply>();
    try {
      if (encoded.isNotEmpty) {
        jsonPointer.asTypedList(encoded.length).setAll(0, encoded);
      }
      if (data != null && data.isNotEmpty) {
        dataPointer.asTypedList(data.length).setAll(0, data);
      }
      final status = _submit(
        _owner,
        id,
        operation,
        jsonPointer,
        encoded.length,
        dataPointer,
        data?.length ?? 0,
        handle,
        bufferLength,
      );
      if (status != 0) {
        throw NativeCallException(
          'ownerAdmission$status',
          'Native owner rejected RPC admission.',
        );
      }
      _pending[id] = completer;
      return completer.future;
    } finally {
      calloc.free(jsonPointer);
      calloc.free(dataPointer);
    }
  }

  Map<String, Object?> _decode(Pointer<Utf8> pointer) {
    if (pointer == nullptr) {
      throw const NativeCallException('nullReply', 'Missing owner reply.');
    }
    final value = jsonDecode(pointer.toDartString());
    if (value is! Map<String, dynamic>) {
      throw const NativeCallException(
        'invalidReply',
        'Native reply is not an object.',
      );
    }
    final json = Map<String, Object?>.from(value);
    if (json['ok'] != true) {
      throw NativeCallException(
        '${json['code'] ?? 'nativeError'}',
        '${json['error'] ?? 'Native operation failed.'}',
      );
    }
    return json;
  }

  void _poll() {
    if (_destroyed) return;
    for (var count = 0; count < 16; count++) {
      final pointer = _nativePoll(_owner);
      if (pointer == nullptr) break;
      final reply = pointer.ref;
      final id = reply.requestId;
      final completer = id == 0 ? _startup : _pending.remove(id);
      _OwnedReply? owned;
      Object? error;
      StackTrace? trace;
      try {
        if (_closing) throw const NativeDisposedException();
        if (reply.error != 0) {
          throw NativeCallException(
            'ownerReply${reply.error}',
            'Native owner could not complete RPC.',
          );
        }
        final json = _decode(reply.json);
        if (reply.releaseJson != nullptr) _decode(reply.releaseJson);
        if (reply.byteLength > maxReferenceBytes ||
            (reply.byteLength > 0 && reply.bytes == nullptr)) {
          throw const NativeCallException(
            'invalidBuffer',
            'Owner returned invalid pixel storage.',
          );
        }
        // This allocation is Dart-owned. No asTypedList view escapes reply_free.
        final bytes = reply.bytes == nullptr
            ? null
            : Uint8List.fromList(reply.bytes.asTypedList(reply.byteLength));
        owned = _OwnedReply(json, bytes);
        if (completer != null && !completer.isCompleted && !_closing) {
          _claimSnapshot?.call(_owner, pointer);
        }
      } catch (caught, stack) {
        error = caught;
        trace = stack;
      } finally {
        _free(_owner, pointer);
      }
      if (completer != null && !completer.isCompleted) {
        if (error != null) {
          completer.completeError(error, trace);
        } else {
          completer.complete(owned!);
        }
      }
    }
    if (!_startup.isCompleted && _finished(_owner) != 0) {
      _startup.completeError(
        const NativeCallException(
          'ownerStartup',
          'Native owner stopped before startup completed.',
        ),
      );
      unawaited(close());
    }
    if (_closing && _finished(_owner) != 0 && _pending.isEmpty) {
      final cleanup = _destroy(_owner);
      if (cleanup >= 0) {
        _destroyed = true;
        _timer.cancel();
        if (!_startup.isCompleted) {
          _startup.completeError(const NativeDisposedException());
        }
        if (cleanup == 0) {
          _closed.complete();
        } else {
          _closed.completeError(
            const NativeCallException(
              'cleanupFailed',
              'Native owner cleanup reported an error.',
            ),
          );
        }
      }
    }
  }

  @override
  Future<int> createWorker() async {
    final serializationReservation = capabilities['maxDocumentBytes'];
    if (serializationReservation is! int ||
        serializationReservation < maxReferenceBytes ||
        serializationReservation > 128 * 1024 * 1024) {
      throw const NativeCallException(
        'nativeLimit',
        'Invalid Native serialization reservation limit.',
      );
    }
    final result = await _request(
      2,
      request: {
        'op': 'workerCreate',
        // PrepareSave reserves the Native serialization ceiling before execution.
        // This does not allocate that amount or raise the 16 MiB transfer limit.
        'config': {
          'maxOutstanding': 4,
          'maxPendingBytes': serializationReservation,
        },
      },
    );
    return result.json['worker']! as int;
  }

  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) async {
    const allowed = {
      'workerSubmit',
      'workerOpen',
      'workerOpenRecovery',
      'workerCancel',
      'workerClose',
      'workerStatus',
      'presentationConfigure',
      'view',
    };
    if (!allowed.contains(request['op'])) {
      throw ArgumentError(
        'Use typed worker lifecycle methods for ${request['op']}.',
      );
    }
    return (await _request(1, request: request, data: data)).json;
  }

  @override
  Future<Map<String, Object?>?> pollWorker(int worker) async {
    final result = await _request(3, handle: worker);
    final completion = result.json['completion'];
    return completion == null
        ? null
        : Map<String, Object?>.from(completion as Map);
  }

  @override
  Future<Uint8List> copyAndReleaseBuffer(int buffer, int byteLength) async {
    var accepted = false;
    try {
      final future = _request(5, handle: buffer, bufferLength: byteLength);
      accepted = true;
      final result = await future;
      return result.bytes ??
          (throw const NativeCallException(
            'missingPixels',
            'Buffer copy returned no bytes.',
          ));
    } finally {
      if (!accepted && !_closing) await releaseBuffer(buffer);
      // Once accepted, the shim releases even when native copy or allocation
      // fails. During close, its tracked-buffer sweep owns the remaining handle.
    }
  }

  @override
  Future<void> releaseBuffer(int buffer) async {
    if (_closing) return; // Owner cleanup owns every published native handle.
    await _request(6, handle: buffer);
  }

  @override
  Future<void> close() {
    if (!_closing) {
      _closing = true;
      _nativeClose(_owner);
    }
    return _closed.future;
  }
}
