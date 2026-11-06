import 'dart:async';
import 'dart:collection';
import 'dart:math' as math;
import 'dart:typed_data';

import 'native_ffi.dart';

class NativeDocumentInfo {
  const NativeDocumentInfo(
    this.documentId,
    this.revision,
    this.fontGeneration,
    this.pageCount,
  );
  factory NativeDocumentInfo.fromJson(Map<String, Object?> json) =>
      NativeDocumentInfo(
        json['documentId']! as int,
        json['revision']! as int,
        json['fontGeneration']! as int,
        json['pageCount']! as int,
      );
  final int documentId;
  final int revision;
  final int fontGeneration;
  final int pageCount;
  Map<String, Object?> get target => {
    'documentId': documentId,
    'revision': revision,
  };
}

class NativePageSize {
  const NativePageSize(this.width, this.height);
  final double width;
  final double height;
}

class NativeFrameStamp {
  const NativeFrameStamp(
    this.documentEpoch,
    this.documentId,
    this.revision,
    this.fontGeneration,
    this.viewGeneration,
  );
  final int documentEpoch;
  final int documentId;
  final int revision;
  final int fontGeneration;
  final int viewGeneration;
}

class NativeRasterFrame {
  const NativeRasterFrame({
    required this.pixels,
    required this.width,
    required this.height,
    required this.stride,
    required this.originX,
    required this.originY,
    required this.scale,
    required this.stamp,
  });

  /// Owned Dart bytes. The legacy native buffer has already been released.
  final Uint8List pixels;
  final int width;
  final int height;
  final int stride;
  final int originX;
  final int originY;
  final double scale;
  final NativeFrameStamp stamp;
}

class _Operation {
  _Operation(
    this.build, {
    this.openBytes,
    this.render = false,
    this.viewGeneration = 0,
  });
  final Map<String, Object?>? Function() build;
  final Uint8List? openBytes;
  final bool render;
  final int viewGeneration;
  final done = Completer<Object?>();
  int? requestId;
  bool cancelRequested = false;
  bool creditReleased = false;
  NativeFrameStamp? stamp;
  double scale = 1;
}

/// One document worker and one current viewport. Native Core is constructed and
/// run by Rust NativeRuntime; neither UI isolate nor dispatcher parses HWP.
class NativeDocument {
  NativeDocument._(this.transport, this.worker) {
    _timer = Timer.periodic(const Duration(milliseconds: 4), (_) => _poll());
  }
  static Future<NativeDocument> connect(NativeWorkerTransport transport) async {
    try {
      if (transport.capabilities['worker'] != true) {
        throw const NativeCallException(
          'unsupportedWorker',
          'Native library has no worker ABI.',
        );
      }
      return NativeDocument._(transport, await transport.createWorker());
    } catch (_) {
      await transport.close();
      rethrow;
    }
  }

  final NativeWorkerTransport transport;
  final int worker;
  final _queue = Queue<_Operation>();
  final _changes = StreamController<NativeDocumentInfo>.broadcast();
  late final Timer _timer;
  _Operation? _pending;
  bool _pumping = false;
  bool _polling = false;
  bool _disposed = false;
  int _queuedBytes = 0;
  int _documentEpoch = 0;
  int _viewGeneration = 0;
  NativeDocumentInfo? _info;
  Future<void>? _closeFuture;

  NativeDocumentInfo? get info => _info;
  bool get isDisposed => _disposed;
  int get viewGeneration => _viewGeneration;
  Stream<NativeDocumentInfo> get changes => _changes.stream;

  int beginView() {
    if (_disposed) throw const NativeDisposedException();
    ++_viewGeneration;
    _cancelRender();
    return _viewGeneration;
  }

  bool isCurrent(NativeFrameStamp stamp) {
    final info = _info;
    return !_disposed &&
        info != null &&
        stamp.documentEpoch == _documentEpoch &&
        stamp.documentId == info.documentId &&
        stamp.revision == info.revision &&
        stamp.fontGeneration == info.fontGeneration &&
        stamp.viewGeneration == _viewGeneration;
  }

  Future<Object?> _enqueue(_Operation operation) {
    final bytes = operation.openBytes?.length ?? 0;
    _checkAdmission(bytes);
    _queuedBytes += bytes;
    _queue.add(operation);
    unawaited(_pump());
    return operation.done.future;
  }

  void _checkAdmission(int bytes) {
    if (_disposed) throw const NativeDisposedException();
    if (_queue.length + (_pending == null ? 0 : 1) >= 64 ||
        _queuedBytes + bytes > 32 * 1024 * 1024) {
      throw const NativeCallException('queueLimit', 'Reference FIFO is full.');
    }
  }

  Future<NativeDocumentInfo> newDocument() {
    _checkAdmission(0);
    beginView();
    return _enqueue(_Operation(() => {'op': 'new', 'replace': _info?.target}))
        .then((value) => value! as NativeDocumentInfo);
  }

  Future<NativeDocumentInfo> openBytes(Uint8List bytes) {
    if (bytes.length > maxReferenceBytes) {
      throw const NativeCallException(
        'documentLimit',
        'Reference documents are limited to 16 MiB.',
      );
    }
    _checkAdmission(bytes.length);
    beginView();
    final owned = Uint8List.fromList(bytes);
    return _enqueue(_Operation(() => const {'op': 'open'}, openBytes: owned))
        .then((value) => value! as NativeDocumentInfo);
  }

  Future<NativeDocumentInfo> refreshInfo() =>
      _enqueue(_Operation(() => const {'op': 'info'}))
          .then((value) => value! as NativeDocumentInfo);

  Future<NativePageSize> pageSize(int page) => _enqueue(
    _Operation(
      () => {'op': 'pageSize', 'target': _requireInfo().target, 'page': page},
    ),
  ).then((value) => value! as NativePageSize);

  Future<NativeRasterFrame?> renderRegion({
    required int page,
    required double x,
    required double y,
    required double width,
    required double height,
    required double scale,
    int? viewGeneration,
  }) {
    final view = viewGeneration ?? _viewGeneration;
    final values = [x, y, width, height, scale, x + width, y + height];
    if (values.any((value) => !value.isFinite) ||
        width <= 0 ||
        height <= 0 ||
        scale <= 0 ||
        scale > 64) {
      throw ArgumentError('Invalid native viewport geometry.');
    }
    final endpoints = [
      x * scale,
      y * scale,
      (x + width) * scale,
      (y + height) * scale,
    ];
    if (endpoints.any((value) => !value.isFinite || value.abs() > 16777216)) {
      throw ArgumentError('Viewport exceeds exact native pixel coordinates.');
    }
    final pixels =
        (endpoints[2].ceil() - endpoints[0].floor()) *
        (endpoints[3].ceil() - endpoints[1].floor());
    if (pixels <= 0 || pixels * 4 > maxReferenceBytes) {
      throw const NativeCallException(
        'frameLimit',
        'Reference frames are limited to 16 MiB.',
      );
    }
    final operation = _Operation(
      () {
        if (view != _viewGeneration) return null;
        final info = _requireInfo();
        return {
          'op': transport.capabilities['renderTiledRegion'] == true
              ? 'renderTiledRegion'
              : 'renderRegion',
          'target': info.target,
          'fontGeneration': info.fontGeneration,
          'page': page,
          'region': {'x': x, 'y': y, 'width': width, 'height': height},
          'scale': scale,
        };
      },
      render: true,
      viewGeneration: view,
    )..scale = scale;
    return _enqueue(operation).then((value) => value as NativeRasterFrame?);
  }

  NativeDocumentInfo _requireInfo() =>
      _info ?? (throw StateError('Open a native document first.'));

  Future<void> _pump() async {
    if (_pumping || _disposed || _pending != null) return;
    _pumping = true;
    try {
      while (!_disposed && _pending == null && _queue.isNotEmpty) {
        final operation = _queue.removeFirst();
        _pending = operation;
        try {
          final command = operation.build();
          if (command == null) {
            _finish(operation, null);
            continue;
          }
          if (operation.render) {
            final info = _requireInfo();
            operation.stamp = NativeFrameStamp(
              _documentEpoch,
              info.documentId,
              info.revision,
              info.fontGeneration,
              operation.viewGeneration,
            );
          }
          final accepted = operation.openBytes != null
              ? await transport.hostRequest({
                  'op': 'workerOpen',
                  'worker': worker,
                  'replace': _info?.target,
                }, data: operation.openBytes)
              : await transport.hostRequest({
                  'op': 'workerSubmit',
                  'worker': worker,
                  'command': command,
                });
          operation.requestId = accepted['requestId']! as int;
          if (operation.render && operation.viewGeneration != _viewGeneration) {
            _cancelRender();
          }
        } catch (error, stack) {
          _finish(operation, null, error, stack);
        }
      }
    } finally {
      _pumping = false;
    }
  }

  void _finish(
    _Operation operation,
    Object? value, [
    Object? error,
    StackTrace? stack,
  ]) {
    if (identical(_pending, operation)) _pending = null;
    if (!operation.creditReleased) {
      _queuedBytes -= operation.openBytes?.length ?? 0;
      operation.creditReleased = true;
    }
    if (!operation.done.isCompleted) {
      if (error == null) {
        operation.done.complete(value);
      } else {
        operation.done.completeError(error, stack);
      }
    }
  }

  void _cancelRender() {
    final pending = _pending;
    if (pending == null ||
        !pending.render ||
        pending.cancelRequested ||
        pending.requestId == null ||
        _disposed) {
      return;
    }
    pending.cancelRequested = true;
    // Cancellation does not free the native FIFO slot. Its real completion
    // still has to arrive before the next accepted document operation starts.
    unawaited(
      transport
          .hostRequest({
            'op': 'workerCancel',
            'worker': worker,
            'requestId': pending.requestId,
          })
          .then<void>((_) {}, onError: (Object _, StackTrace _) {}),
    );
  }

  Future<void> _poll() async {
    if (_polling || _disposed || _pending?.requestId == null) return;
    _polling = true;
    try {
      final completion = await transport.pollWorker(worker);
      if (completion != null) await _complete(completion);
    } catch (error, stack) {
      if (!_disposed) {
        final pending = _pending;
        if (pending != null) _finish(pending, null, error, stack);
        unawaited(
          dispose().then<void>((_) {}, onError: (Object _, StackTrace _) {}),
        );
      }
    } finally {
      _polling = false;
      unawaited(_pump());
    }
  }

  Future<void> _complete(Map<String, Object?> result) async {
    final operation = _pending;
    final buffer = result['buffer'];
    if (_disposed ||
        operation == null ||
        operation.requestId != result['requestId']) {
      if (buffer is int) await transport.releaseBuffer(buffer);
      if (!_disposed) {
        throw const NativeCallException(
          'unexpectedCompletion',
          'Worker completion ID mismatch.',
        );
      }
      return;
    }
    if (result['ok'] != true) {
      if (buffer is int) await transport.releaseBuffer(buffer);
      _finish(
        operation,
        null,
        NativeCallException('${result['code']}', '${result['error']}'),
      );
      return;
    }
    if (result['kind'] == 'cancelled' || result['kind'] == 'skipped') {
      if (buffer is int) await transport.releaseBuffer(buffer);
      _finish(
        operation,
        null,
        operation.render
            ? null
            : const NativeCallException(
                'cancelled',
                'Document request was cancelled.',
              ),
      );
      return;
    }
    if (operation.render) {
      final stamp = operation.stamp!;
      if (!isCurrent(stamp) ||
          result['revision'] != stamp.revision ||
          result['fontGeneration'] != stamp.fontGeneration) {
        if (buffer is int) await transport.releaseBuffer(buffer);
        _finish(operation, null);
        return;
      }
      var copying = false;
      try {
        final width = result['width']! as int,
            height = result['height']! as int;
        final stride = result['stride']! as int,
            length = result['byteLength']! as int;
        if (buffer is! int ||
            width <= 0 ||
            height <= 0 ||
            stride < width * 4 ||
            length != stride * height ||
            length > maxReferenceBytes) {
          throw const NativeCallException(
            'invalidFrame',
            'Invalid native RGBA frame.',
          );
        }
        copying = true;
        final bytes = await transport.copyAndReleaseBuffer(buffer, length);
        if (bytes.length != length) {
          throw const NativeCallException(
            'invalidBufferLength',
            'Native pixel copy is incomplete.',
          );
        }
        if (!isCurrent(stamp)) {
          _finish(operation, null);
          return;
        }
        _finish(
          operation,
          NativeRasterFrame(
            pixels: bytes.asUnmodifiableView(),
            width: width,
            height: height,
            stride: stride,
            originX: result['originX']! as int,
            originY: result['originY']! as int,
            scale: operation.scale,
            stamp: stamp,
          ),
        );
      } catch (error, stack) {
        if (!copying && buffer is int) await transport.releaseBuffer(buffer);
        _finish(operation, null, error, stack);
      }
    } else if (result['kind'] == 'info') {
      final info = NativeDocumentInfo.fromJson(
        Map<String, Object?>.from(result['info']! as Map),
      );
      if (_info?.documentId != info.documentId) ++_documentEpoch;
      _info = info;
      _changes.add(info);
      _finish(operation, info);
    } else if (result['kind'] == 'pageSize') {
      final size = result['size']! as Map;
      final width = (size['width']! as num).toDouble(),
          height = (size['height']! as num).toDouble();
      if (!width.isFinite || !height.isFinite || math.min(width, height) <= 0) {
        _finish(
          operation,
          null,
          const NativeCallException(
            'invalidPageSize',
            'Invalid native page extent.',
          ),
        );
      } else {
        _finish(operation, NativePageSize(width, height));
      }
    } else {
      _finish(operation, result);
    }
  }

  Future<void> dispose() {
    if (_closeFuture != null) return _closeFuture!;
    _disposed = true;
    ++_viewGeneration;
    _timer.cancel();
    final operations = [..._queue, ?_pending];
    _queue.clear();
    _pending = null;
    _queuedBytes = 0;
    for (final operation in operations) {
      operation.creditReleased = true;
      if (!operation.done.isCompleted) {
        operation.done.completeError(const NativeDisposedException());
      }
    }
    _closeFuture = transport.close().whenComplete(() {
      unawaited(_changes.close());
    });
    return _closeFuture!;
  }
}

/// Serial image decoding with at most one decoding and one newest queued frame.
/// A decoded stale/disposed image is released, never published to a widget.
class CpuFramePresenter<T> {
  CpuFramePresenter({
    required this.decode,
    required this.disposeImage,
    required this.isCurrent,
    required this.onImage,
    required this.onError,
  });
  final Future<T> Function(NativeRasterFrame) decode;
  final void Function(T) disposeImage;
  final bool Function(NativeFrameStamp) isCurrent;
  final void Function(T?) onImage;
  final void Function(Object) onError;
  NativeRasterFrame? _queued;
  NativeRasterFrame? _decodingFrame;
  bool _decoding = false;
  bool _disposed = false;
  int _generation = 0;
  T? image;

  void offer(NativeRasterFrame frame) {
    if (_disposed || !isCurrent(frame.stamp)) return;
    _queued = frame;
    if (!_decoding) unawaited(_drain());
  }

  Future<void> _drain() async {
    _decoding = true;
    try {
      while (!_disposed && _queued != null) {
        final frame = _queued!;
        final generation = _generation;
        _queued = null;
        _decodingFrame = frame; // Keep Dart-owned pixels alive across decode.
        T? decoded;
        try {
          decoded = await decode(frame);
        } catch (error) {
          if (!_disposed && isCurrent(frame.stamp)) onError(error);
        } finally {
          _decodingFrame = null;
        }
        if (decoded == null) continue;
        if (_disposed || generation != _generation || !isCurrent(frame.stamp)) {
          disposeImage(decoded);
        } else {
          final previous = image;
          image = decoded;
          onImage(decoded);
          if (previous != null) disposeImage(previous);
        }
      }
    } finally {
      _decoding = false;
    }
  }

  void clear() {
    ++_generation;
    _queued = null;
    final previous = image;
    image = null;
    if (!_disposed) onImage(null);
    if (previous != null) disposeImage(previous);
  }

  void dispose() {
    if (_disposed) return;
    _disposed = true;
    clear();
  }

  /// Exposed for ownership tests; never more than two pixel frames are held.
  int get retainedFrameCount =>
      (_queued == null ? 0 : 1) + (_decodingFrame == null ? 0 : 1);
}
