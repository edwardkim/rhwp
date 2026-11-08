import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'native_document.dart';
import 'native_ffi.dart';
import 'native_presentation.dart';
import 'native_viewport.dart';

enum NativeTexturePlatform {
  linux('rhwp_native/texture_linux'),
  macos('rhwp_native/texture_macos'),
  windows('rhwp_native/texture_windows');

  const NativeTexturePlatform(this.channelName);
  final String channelName;
  static NativeTexturePlatform? current() => switch (defaultTargetPlatform) {
    TargetPlatform.linux => linux,
    TargetPlatform.macOS => macos,
    TargetPlatform.windows => windows,
    _ => null,
  };
}

enum NativeTexturePhase {
  initializing,
  pending,
  visible,
  fallback,
  failed,
  closing,
  retirePending,
  closed,
}

// One UI isolate retains at most four owned/ambiguous captures, including late
// channel futures. Ambiguity never authorizes release of a possibly transferred
// native handle. Process-wide native/plugin quotas remain independent limits.
final class _OwnedSnapshot {
  _OwnedSnapshot(this.metadata) {
    retained.add(this);
  }
  static final retained = <_OwnedSnapshot>{};
  static _OwnedSnapshot? reserve(NativePresentation metadata) =>
      retained.length >= 4 ? null : _OwnedSnapshot(metadata);
  final NativePresentation metadata;
  int? id;
  NativePresentationStamp? stamp;
  bool settled = false,
      capturePending = false,
      sending = false,
      ambiguous = false,
      releaseAttempted = false;
  Future<void>? releasing;

  void bind(Map<String, Object?> value) {
    final snapshot = value['snapshot'];
    if (snapshot is! int || snapshot <= 0) {
      throw const NativeCallException(
        'invalidSnapshot',
        'Missing snapshot handle.',
      );
    }
    id = snapshot;
    final rawStamp = value['stamp'];
    if (rawStamp is! Map) {
      throw const NativeCallException(
        'invalidStamp',
        'Missing captured source stamp.',
      );
    }
    stamp = NativePresentationStamp.fromJson(
      Map<String, Object?>.from(rawStamp),
    );
  }

  void transferred() {
    if (settled) return;
    settled = true;
    sending = false;
    ambiguous = false;
    retained.remove(this);
  }

  Future<void> releaseOwned() {
    if (settled || capturePending || sending || ambiguous) {
      return Future.value();
    }
    return releasing ??= _release();
  }

  Future<void> _release() async {
    if (releaseAttempted) return;
    releaseAttempted = true;
    try {
      if (id != null) await metadata.releaseSnapshot(id!);
      settled = true;
      retained.remove(this);
    } catch (_) {
      ambiguous = true; // A failed cleanup is neither success nor permission to retry a release.
      rethrow;
    }
  }
}

class _Plan {
  const _Plan(this.viewport, this.dpr, this.page, this.clear);
  final NativePageViewport viewport;
  final double dpr;
  final int page, clear;
  Object get key => (
    viewport.page.width,
    viewport.page.height,
    viewport.viewport,
    viewport.zoom,
    viewport.origin,
    dpr,
    page,
    clear,
  );
  ({int width, int height, double scale, double x, double y}) geometry() {
    final physicalWidth = viewport.viewport.width * dpr;
    final physicalHeight = viewport.viewport.height * dpr;
    if (!dpr.isFinite ||
        dpr <= 0 ||
        dpr > 16 ||
        !physicalWidth.isFinite ||
        !physicalHeight.isFinite ||
        physicalWidth <= 0 ||
        physicalHeight <= 0 ||
        physicalWidth > 4096 ||
        physicalHeight > 4096) {
      throw const NativeCallException(
        'viewportLimit',
        'GPU viewport exceeds its dimension bound.',
      );
    }
    final width = physicalWidth.ceil(), height = physicalHeight.ceil();
    final scale = viewport.zoom * dpr;
    final x = viewport.origin.dx * dpr, y = viewport.origin.dy * dpr;
    if (width * height > 4 * 1024 * 1024 ||
        !scale.isFinite ||
        scale <= 0 ||
        scale > 16 ||
        !x.isFinite ||
        !y.isFinite ||
        x.abs() > 16777216 ||
        y.abs() > 16777216 ||
        page < 0 ||
        clear < 0 ||
        clear > 0xffffffff) {
      throw const NativeCallException(
        'viewportLimit',
        'GPU viewport exceeds its pixel/coordinate bound.',
      );
    }
    return (width: width, height: height, scale: scale, x: x, y: y);
  }

  Map<String, Object?> submission(int texture, _OwnedSnapshot snapshot) {
    final size = geometry();
    return {
      'textureId': texture,
      'stamp': snapshot.stamp?.toJson(),
      'width': size.width,
      'height': size.height,
      'clearColor': clear,
      'placements': [
        {
          'snapshot': snapshot.id,
          'scale': size.scale,
          'originX': size.x,
          'originY': size.y,
          'backgroundColor': 0xffffffff,
        },
      ],
    };
  }
}

/// Bounded GPU control only. NativeDocument serializes configure/capture behind
/// edits on its legacy owner. This object calls cross-thread metadata and small
/// platform messages; it does not parse documents or replay pixels on Dart.
class NativeTextureController extends ChangeNotifier {
  NativeTextureController({
    required this.document,
    required this.presentation,
    required this.platform,
    MethodChannel? channel,
    this.pollInterval = const Duration(milliseconds: 100),
    this.operationTimeout = const Duration(seconds: 2),
  }) : channel = channel ?? MethodChannel(platform.channelName) {
    if (pollInterval < const Duration(milliseconds: 10) ||
        pollInterval > const Duration(seconds: 1) ||
        operationTimeout <= Duration.zero ||
        operationTimeout > const Duration(seconds: 10)) {
      throw ArgumentError(
        'GPU polling and timeout must have positive bounded cadence.',
      );
    }
    _info = document.info;
    _infoChanges = document.changes.listen((info) {
      final previous = _info;
      _info = info;
      if (previous?.documentId != info.documentId ||
          previous?.revision != info.revision ||
          previous?.fontGeneration != info.fontGeneration) {
        invalidate();
      }
    });
    _invalidations = document.rasterInvalidations.listen((_) {
      if (document.isDisposed) {
        unawaited(close());
      } else {
        invalidate();
      }
    });
    _timer = Timer.periodic(pollInterval, (_) => _tick());
  }
  final NativeDocument document;
  final NativePresentation presentation;
  final NativeTexturePlatform platform;
  final MethodChannel channel;
  final Duration pollInterval, operationTimeout;
  late final Timer _timer;
  late final StreamSubscription<NativeDocumentInfo> _infoChanges;
  late final StreamSubscription<int> _invalidations;
  NativeDocumentInfo? _info;
  NativeTexturePhase _phase = NativeTexturePhase.initializing;
  NativeTexturePhase get phase => _phase;
  Object? diagnostic;
  int? textureId;
  int _device = 0, _generation = 0, _view = 0, _raster = 0, _rpc = 0;
  static int _globalRpc = 0;
  bool _working = false,
      _polling = false,
      _wanted = false,
      _stopped = false,
      _closing = false,
      _notifierDisposed = false;
  _Plan? _plan;
  _OwnedSnapshot? _frame;
  Object? _configurationKey;
  Future<void>? _closeFuture;
  Future<void>? _platformCloseFuture;
  bool _platformCloseActive = false,
      _platformCloseSent = false,
      _createAttempted = false,
      _platformUnregistered = false,
      _nativeCloseComplete = false,
      _retirementPending = false;
  int _attachedViews = 0;
  Object? _retirementDiagnostic;
  Timer? _frameTimeout;
  final _releases = <Future<void>>{};
  static int get retainedOwnedSnapshots => _OwnedSnapshot.retained.length;

  bool get canDisplay =>
      _phase == NativeTexturePhase.visible && _localCurrent(_frame);
  bool get needsCpuFallback => _phase == NativeTexturePhase.fallback;

  /// Tracks owner/registration close only, not immutable consumer GPU credits.
  bool get retirementPending => _retirementPending;
  Object? get retirementDiagnostic => _retirementDiagnostic;
  bool get shouldPaintTexture =>
      textureId != null &&
      !_platformUnregistered &&
      _phase != NativeTexturePhase.closed;

  void _attachView() => ++_attachedViews;

  void _detachView() {
    if (_attachedViews > 0) --_attachedViews;
    if (_attachedViews == 0 && textureId != null && !_nativeCloseComplete) {
      _retirementPending = true;
      _retirementDiagnostic ??= const NativeCallException(
        'textureUnmounted',
        'Texture unmounted before native retirement; bounded native retire/quarantine may remain.',
      );
    }
  }

  bool _localCurrent(_OwnedSnapshot? snapshot) {
    final stamp = snapshot?.stamp, info = document.info;
    return !_closing &&
        !_stopped &&
        !document.isDisposed &&
        stamp != null &&
        info != null &&
        stamp.documentId == info.documentId &&
        stamp.revision == info.revision &&
        stamp.fontGeneration == info.fontGeneration &&
        stamp.viewGeneration == document.presentationGeneration &&
        stamp.deviceGeneration == _device &&
        stamp.deviceScale == _plan?.dpr &&
        _raster == document.rasterInvalidationGeneration;
  }

  void configure(
    NativePageViewport viewport, {
    required double deviceScale,
    int page = 0,
    int clearColor = 0xff55575a,
    int invalidationGeneration = 0,
  }) {
    final plan = _Plan(viewport, deviceScale, page, clearColor);
    final key = (plan.key, invalidationGeneration);
    if (key == _configurationKey) return;
    _configurationKey = key;
    _plan = plan;
    // Geometry admission runs before a native snapshot is allocated.
    try {
      plan.geometry();
    } catch (error) {
      _fail(error, fallback: true);
      return;
    }
    invalidate();
  }

  void invalidate() {
    if (_closing || _stopped || _plan == null || document.isDisposed) return;
    ++_generation;
    _view = document.beginPresentation();
    _raster = document.rasterInvalidationGeneration;
    _wanted = true;
    _phase = NativeTexturePhase.pending;
    _frameTimeout?.cancel();
    _signal(); // Cover cached platform pixels in this same UI turn.
    unawaited(_pump());
  }

  void _signal() {
    if (!_notifierDisposed) notifyListeners();
  }

  bool _current(int generation) =>
      !_closing &&
      !_stopped &&
      generation == _generation &&
      !document.isDisposed &&
      _view == document.presentationGeneration &&
      _raster == document.rasterInvalidationGeneration;

  Map<String, Object?> _map(Object? value) {
    if (value is! Map ||
        value.length > 64 ||
        value.keys.any((key) => key is! String)) {
      throw const NativeCallException(
        'invalidTextureReply',
        'Expected a bounded texture status map.',
      );
    }
    final result = Map<String, Object?>.from(value);
    final ids = result['acceptedSnapshots'];
    if (ids != null && (ids is! List || ids.length > 8)) {
      throw const NativeCallException(
        'invalidTextureReply',
        'Invalid accepted snapshot receipt.',
      );
    }
    return result;
  }

  Future<Map<String, Object?>> _invoke(
    String method,
    Map<String, Object?> arguments, {
    void Function(Map<String, Object?>)? received,
    void Function(Object)? failed,
  }) {
    if (_rpc >= 3 || _globalRpc >= 8) {
      const error = NativeCallException(
        'textureRpcLimit',
        'Texture RPC credit is exhausted.',
      );
      failed?.call(error);
      throw error;
    }
    ++_rpc;
    ++_globalRpc;
    final future = channel.invokeMethod<Object?>(method, arguments).then(_map);
    final observed = future.then(
      (value) {
        --_rpc;
        --_globalRpc;
        received?.call(value);
        if ((_closing || _stopped) &&
            method != 'close' &&
            !_platformCloseActive) {
          unawaited(_closePlatform());
        }
        return value;
      },
      onError: (Object error, StackTrace stack) {
        --_rpc;
        --_globalRpc;
        failed?.call(error);
        if ((_closing || _stopped) &&
            method != 'close' &&
            !_platformCloseActive) {
          unawaited(_closePlatform());
        }
        Error.throwWithStackTrace(error, stack);
      },
    );
    // timeout does not cancel or forget the original reply/ownership observer.
    return observed.timeout(operationTimeout);
  }

  void _release(_OwnedSnapshot snapshot) {
    final future = snapshot.releaseOwned();
    _releases.add(future);
    unawaited(
      future.then<void>(
        (_) {
          _releases.remove(future);
        },
        onError: (Object error) {
          _releases.remove(future);
          if (!_closing) _fail(error, fallback: false);
        },
      ),
    );
  }

  bool _receipt(_OwnedSnapshot snapshot, Map<String, Object?> result) {
    final ids = result['acceptedSnapshots'];
    return snapshot.stamp!.matches(result['acceptedStamp']) &&
        (ids == null || (ids is List && ids.contains(snapshot.id)));
  }

  bool _definiteRejection(Object error) =>
      error is MissingPluginException ||
      (error is NativeCallException && error.code == 'textureRpcLimit') ||
      (error is PlatformException &&
          const {
            'backpressure',
            'staleFrame',
            'contextChanged',
            'notReady',
            'closing',
            'invalidArgument',
            'limitExceeded',
            'retireLimit',
            'featureDisabled',
            'unsupported',
            'closed',
          }.contains(error.code));
  void _submitReply(_OwnedSnapshot snapshot, Map<String, Object?> result) {
    snapshot.sending = false;
    if (result['accepted'] == true && _receipt(snapshot, result)) {
      snapshot.transferred();
    } else if (result['accepted'] == false) {
      snapshot.ambiguous = false;
      _release(snapshot);
    } else {
      snapshot.ambiguous = true;
    }
  }

  void _submitError(_OwnedSnapshot snapshot, Object error) {
    snapshot.sending = false;
    if (_definiteRejection(error)) {
      snapshot.ambiguous = false;
      _release(snapshot);
    } else {
      snapshot.ambiguous = true;
    }
  }

  Future<void> _pump() async {
    if (_working || !_wanted || _closing || _stopped || _plan == null) return;
    _working = true;
    final generation = _generation, plan = _plan!, view = _view;
    _wanted = false;
    _OwnedSnapshot? snapshot;
    try {
      if (!presentation.available ||
          document.transport.capabilities['ownerPresentationCleanup'] != true) {
        throw const NativeCallException(
          'unsupportedPresentation',
          'Full source checks and owner snapshot cleanup are required.',
        );
      }
      final transport = document.transport;
      final ownerPath = switch (transport) {
        NativeOwnerTransport owner => owner.nativeLibraryPath,
        NativePresentationOwnerIdentity identity => identity.nativeLibraryPath,
        _ => null,
      };
      if (ownerPath == null || ownerPath != presentation.libraryPath) {
        throw const NativeCallException(
          'nativeModuleMismatch',
          'GPU and document owners must use the same canonical native module.',
        );
      }
      if (textureId == null) {
        _createAttempted = true;
        await _invoke(
          'create',
          {'libraryPath': presentation.libraryPath},
          received: (reply) {
            final id = reply['textureId'], device = reply['deviceGeneration'];
            if (id is int && id > 0 && device is int && device > 0) {
              textureId = id;
              _device = device;
              if (_closing || _stopped) {
                unawaited(_closePlatform());
              } else {
                _signal();
              }
            }
          },
          failed: (error) {
            if (_definiteRejection(error)) _createAttempted = false;
          },
        );
        if (textureId == null || _device == 0) {
          throw const NativeCallException(
            'invalidTextureReply',
            'Missing texture identity/device generation.',
          );
        }
      }
      await _waitReady(generation).timeout(operationTimeout);
      if (!_current(generation)) return;
      snapshot = _OwnedSnapshot.reserve(presentation);
      if (snapshot == null) {
        throw const NativeCallException(
          'snapshotCreditLimit',
          'Owned or ambiguous snapshot credit is exhausted.',
        );
      }
      final rawCapture = document.capturePresentation(
        page: plan.page,
        viewGeneration: view,
        deviceGeneration: _device,
        deviceScale: plan.dpr,
        releaseSnapshot: presentation.releaseSnapshot,
      );
      snapshot.capturePending = true;
      final capture = rawCapture.then(
        (result) {
          snapshot!.capturePending = false;
          try {
            snapshot.bind(result);
            if (!_current(generation)) _release(snapshot);
            return result;
          } catch (_) {
            _release(snapshot);
            rethrow;
          }
        },
        onError: (Object error, StackTrace stack) {
          snapshot!.capturePending = false;
          _release(snapshot);
          Error.throwWithStackTrace(error, stack);
        },
      );
      await capture.timeout(operationTimeout);
      if (!_current(generation)) return;
      if (!_localCurrent(snapshot) ||
          !await presentation
              .snapshotCheck(snapshot.id!, snapshot.stamp!)
              .timeout(operationTimeout)) {
        throw const NativeCallException(
          'stalePresentation',
          'Captured snapshot is no longer current.',
        );
      }
      if (!_current(generation)) {
        _release(snapshot);
        return;
      }
      snapshot.sending = true;
      final owned = snapshot;
      Map<String, Object?> reply;
      try {
        reply = await _invoke(
          'submit',
          plan.submission(textureId!, snapshot),
          received: (reply) => _submitReply(owned, reply),
          failed: (error) => _submitError(owned, error),
        );
      } catch (_) {
        if (!snapshot.settled && (snapshot.sending || snapshot.ambiguous)) {
          snapshot.ambiguous = true;
          await _reconcile(snapshot);
        }
        rethrow;
      }
      if (reply['accepted'] != true || !snapshot.settled) {
        if (snapshot.ambiguous) await _reconcile(snapshot);
        throw NativeCallException(
          snapshot.ambiguous ? 'ambiguousOwnership' : 'backpressure',
          'Texture submission was not conclusively accepted.',
        );
      }
      if (!_current(generation)) return;
      _frame = snapshot;
      _phase = NativeTexturePhase.pending;
      _publicationDeadline(generation);
      _signal();
    } catch (error) {
      if (snapshot != null &&
          !snapshot.settled &&
          !snapshot.sending &&
          !snapshot.ambiguous) {
        _release(snapshot);
      }
      if (_current(generation)) _fail(error, fallback: _canFallback(error));
    } finally {
      _working = false;
      if (!_closing &&
          !_stopped &&
          generation == _generation &&
          _view != document.presentationGeneration) {
        _fail(
          const NativeCallException(
            'stalePresentation',
            'Another viewport took presentation ownership.',
          ),
          fallback: true,
        );
      }
      if (_wanted && !_closing && !_stopped) unawaited(_pump());
    }
  }

  Future<void> _waitReady(int generation) async {
    while (_current(generation)) {
      final status = await _invoke('status', {'textureId': textureId});
      _checkStatus(status);
      if (platform == NativeTexturePlatform.linux || status['ready'] == true) {
        return;
      }
      await Future<void>.delayed(pollInterval);
    }
  }

  void _publicationDeadline(int generation) {
    _frameTimeout?.cancel();
    _frameTimeout = Timer(operationTimeout, () {
      if (_current(generation) && _phase != NativeTexturePhase.visible) {
        _fail(
          TimeoutException('GPU frame publication timed out.'),
          fallback: true,
        );
      }
    });
  }

  Future<void> _reconcile(_OwnedSnapshot snapshot) async {
    if (textureId == null || _rpc >= 3) return;
    try {
      final result = await _invoke('status', {'textureId': textureId});
      if (_receipt(snapshot, result)) snapshot.transferred();
      // An older status is not a rejection: a late submit may still execute.
    } catch (_) {
      /* Retain the bounded ambiguous handle; never guess ownership. */
    }
  }

  void _checkStatus(Map<String, Object?> status) {
    if (status['closed'] == true || status['closing'] == true) {
      throw const NativeCallException('closed', 'Texture owner is closing.');
    }
    final device = status['deviceGeneration'];
    if (device is! int || device <= 0 || device != _device) {
      throw const NativeCallException(
        'contextChanged',
        'Texture device generation changed.',
      );
    }
    final code = status['errorCode'] ?? status['code'],
        message = status['error'] ?? status['message'];
    if (status['deviceFailure'] == true) {
      throw NativeCallException(
        'deviceLost',
        message is String ? message : 'The GPU device was removed.',
      );
    }
    if ((code is String && code.isNotEmpty) ||
        (message is String && message.isNotEmpty)) {
      throw NativeCallException(
        code is String && code.isNotEmpty ? code : 'nativeTextureError',
        message is String
            ? message.substring(0, math.min(message.length, 2048))
            : 'Texture rendering failed.',
      );
    }
  }

  bool _canFallback(Object error) =>
      error is MissingPluginException ||
      error is TimeoutException ||
      (error is NativeCallException &&
          const {
            'unsupportedPresentation',
            'unsupported',
            'featureDisabled',
            'backpressure',
            'retireLimit',
            'snapshotCreditLimit',
            'contextChanged',
            'deviceLost',
            'closed',
            'stalePresentation',
            'staleFrame',
            'ambiguousOwnership',
            'textureRpcLimit',
          }.contains(error.code)) ||
      (error is PlatformException &&
          const {
            'unsupported',
            'featureDisabled',
            'backpressure',
            'retireLimit',
            'notReady',
            'closing',
            'closed',
            'contextChanged',
            'staleFrame',
          }.contains(error.code));

  void _fail(Object error, {required bool fallback}) {
    if (_closing || _stopped) return;
    diagnostic = error;
    _stopped = true;
    _wanted = false;
    _phase = fallback ? NativeTexturePhase.fallback : NativeTexturePhase.failed;
    _timer.cancel();
    _frameTimeout?.cancel();
    _signal();
    unawaited(_closePlatform());
  }

  void _tick() {
    if (_working || _polling || _closing || _stopped) return;
    if (_wanted) {
      unawaited(_pump());
      return;
    }
    if (_frame != null) unawaited(_poll());
  }

  Future<void> _poll() async {
    if (_polling ||
        _closing ||
        _stopped ||
        _frame == null ||
        textureId == null) {
      return;
    }
    _polling = true;
    final frame = _frame!, generation = _generation;
    try {
      if (!_localCurrent(frame)) {
        throw const NativeCallException(
          'stalePresentation',
          'Another source/view owns the viewport.',
        );
      }
      final status = await _invoke('status', {'textureId': textureId});
      _checkStatus(status);
      if (!_current(generation)) return;
      final rendered = platform == NativeTexturePlatform.macos
          ? status['hasFrame'] == true &&
                frame.stamp!.matches(status['acceptedStamp'])
          : frame.stamp!.matches(status['renderedStamp']) &&
                (platform == NativeTexturePlatform.linux ||
                    status['hasFrame'] == true);
      if (!rendered) {
        if (_phase == NativeTexturePhase.visible) {
          _phase = NativeTexturePhase.pending;
          _publicationDeadline(generation);
          _signal();
        }
        return;
      }
      if (!await presentation
          .snapshotCheck(frame.id!, frame.stamp!)
          .timeout(operationTimeout)) {
        throw const NativeCallException(
          'stalePresentation',
          'The full source stamp is no longer current.',
        );
      }
      if (_current(generation) && _localCurrent(frame)) {
        final changed = _phase != NativeTexturePhase.visible;
        _phase = NativeTexturePhase.visible;
        _frameTimeout?.cancel();
        if (changed) _signal();
      }
    } catch (error) {
      if (!_closing && !_stopped && generation == _generation) {
        _fail(error, fallback: _canFallback(error));
      }
    } finally {
      _polling = false;
    }
  }

  bool _observeCloseStatus(Map<String, Object?> status) {
    final unregistered =
        status['registered'] == false ||
        status['unregistered'] == true ||
        status['textureUnregisteredNotification'] == true;
    final registrationChanged = unregistered && !_platformUnregistered;
    if (unregistered) _platformUnregistered = true;
    final complete = switch (platform) {
      NativeTexturePlatform.linux =>
        status['cleanupComplete'] == true && status['registered'] == false,
      NativeTexturePlatform.macos || NativeTexturePlatform.windows =>
        status['ownerFinished'] == true && unregistered,
    };
    if (complete) {
      // Owner/registration completion is not consumer GPU completion or a
      // refund of immutable/quarantined frame credits on any platform.
      _nativeCloseComplete = true;
      _retirementPending = false;
      _retirementDiagnostic = null;
      if (_closing) _phase = NativeTexturePhase.closed;
      _signal();
    } else if (registrationChanged) {
      _signal();
    }
    return complete;
  }

  void _retirementUnconfirmed(Object error) {
    _retirementPending = true;
    _retirementDiagnostic ??= error;
    if (_closing) _phase = NativeTexturePhase.retirePending;
    _signal();
  }

  Future<void> _closePlatform() {
    final existing = _platformCloseFuture;
    if (existing != null) return existing;
    final done = Completer<void>();
    _platformCloseFuture = done.future;
    _platformCloseActive = true;
    Timer? poll, deadline;
    var invoking = false;
    void finish({Object? error}) {
      if (done.isCompleted) return;
      poll?.cancel();
      deadline?.cancel();
      _platformCloseActive = false;
      if (error != null) {
        _retirementUnconfirmed(error);
        // A create/credit can arrive after this bounded wait. Its existing
        // ownership observer may start the one native close once the ID is known.
        if (!_platformCloseSent) _platformCloseFuture = null;
      }
      done.complete();
    }

    void received(Map<String, Object?> status) {
      if (_observeCloseStatus(status)) finish();
    }

    Future<void> attempt() async {
      if (done.isCompleted || invoking) return;
      final id = textureId;
      if (id == null) {
        if (!_createAttempted && !_working) {
          _nativeCloseComplete =
              true; // No native texture handle was transferred.
          _retirementPending = false;
          _retirementDiagnostic = null;
          if (_closing) _phase = NativeTexturePhase.closed;
          _signal();
          finish();
        }
        return;
      }
      if (_rpc >= 3 || _globalRpc >= 8) return;
      invoking = true;
      try {
        final method = _platformCloseSent ? 'status' : 'close';
        _platformCloseSent = true;
        await _invoke(method, {'textureId': id}, received: received);
      } catch (error) {
        // Native close/status is never retried beyond this single deadline.
        // Late original replies still observe real cleanup and release RPC credit.
        finish(error: error);
      } finally {
        invoking = false;
      }
    }

    deadline = Timer(operationTimeout, () {
      finish(
        error: const NativeCallException(
          'retirePending',
          'Native texture retirement was not confirmed before the deadline; bounded native retire/quarantine may remain.',
        ),
      );
    });
    poll = Timer.periodic(pollInterval, (_) => unawaited(attempt()));
    unawaited(attempt());
    return done.future;
  }

  Future<void> close() {
    if (_closeFuture != null) return _closeFuture!;
    final done = Completer<void>();
    _closeFuture =
        done.future; // Publish idempotency before notifying listeners.
    _closing = true;
    _stopped = true;
    ++_generation;
    _wanted = false;
    _phase = NativeTexturePhase.closing;
    _retirementPending =
        !_nativeCloseComplete && (textureId != null || _createAttempted);
    _timer.cancel();
    _frameTimeout?.cancel();
    _signal();
    unawaited(_infoChanges.cancel());
    unawaited(_invalidations.cancel());
    unawaited(() async {
      await _closePlatform();
      if (_nativeCloseComplete) {
        _phase = NativeTexturePhase.closed;
        _signal();
      } else {
        _retirementUnconfirmed(
          _retirementDiagnostic ??
              const NativeCallException(
                'retirePending',
                'Native texture retirement remains unconfirmed.',
              ),
        );
      }
      if (_releases.isNotEmpty) {
        try {
          await Future.wait(_releases.toList()).timeout(operationTimeout);
        } catch (_) {}
      }
      done.complete();
    }());
    return _closeFuture!;
  }

  @override
  void dispose() {
    unawaited(close());
    _notifierDisposed = true;
    super.dispose();
  }
}

/// External texture stays painted underneath an opaque cover while a new frame
/// is pending. Offstage/Opacity(0) would stop Linux populate and its close path.
/// The supplied fallback belongs to the host's single CPU/input viewport owner.
class NativeTextureView extends StatefulWidget {
  const NativeTextureView({
    super.key,
    required this.controller,
    required this.viewport,
    required this.cpuFallback,
    this.page = 0,
    this.invalidationGeneration = 0,
    this.coverColor = const Color(0xff55575a),
  });
  final NativeTextureController controller;
  final NativePageViewport viewport;
  final WidgetBuilder cpuFallback;
  final int page, invalidationGeneration;
  final Color coverColor;
  @override
  State<NativeTextureView> createState() => _NativeTextureViewState();
}

class _NativeTextureViewState extends State<NativeTextureView> {
  @override
  void initState() {
    super.initState();
    widget.controller._attachView();
    widget.controller.addListener(_changed);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _configure();
  }

  void _configure() {
    widget.controller.configure(
      widget.viewport,
      deviceScale: MediaQuery.devicePixelRatioOf(context),
      page: widget.page,
      clearColor: widget.coverColor.toARGB32(),
      invalidationGeneration: widget.invalidationGeneration,
    );
  }

  void _changed() {
    if (mounted) setState(() {});
  }

  @override
  void didUpdateWidget(NativeTextureView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!identical(oldWidget.controller, widget.controller)) {
      oldWidget.controller.removeListener(_changed);
      oldWidget.controller._detachView();
      unawaited(oldWidget.controller.close());
      widget.controller._attachView();
      widget.controller.addListener(_changed);
    }
    _configure();
  }

  @override
  Widget build(BuildContext context) {
    final controller = widget.controller;
    final visible = controller.canDisplay;
    return Stack(
      fit: StackFit.expand,
      children: [
        if (controller.shouldPaintTexture)
          IgnorePointer(
            child: Texture(
              textureId: controller.textureId!,
              filterQuality: FilterQuality.low,
            ),
          ),
        if (!visible)
          Positioned.fill(
            key: const ValueKey('rhwp-native-texture-cover'),
            child: ColoredBox(
              color: widget.coverColor.withAlpha(255),
              child: controller.phase == NativeTexturePhase.failed
                  ? Center(child: Text('문서 표시 실패: ${controller.diagnostic}'))
                  : widget.cpuFallback(context),
            ),
          ),
      ],
    );
  }

  @override
  void dispose() {
    widget.controller.removeListener(_changed);
    widget.controller._detachView();
    unawaited(widget.controller.close());
    super.dispose();
  }
}
