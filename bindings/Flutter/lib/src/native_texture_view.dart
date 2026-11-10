import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart' show PlatformViewHitTestBehavior;
import 'package:flutter/services.dart';

import 'native_document.dart';
import 'native_ffi.dart';
import 'native_gpu_tiles.dart';
import 'native_gpu_trace.dart';
import 'native_presentation.dart';
import 'native_viewport.dart';

enum NativeTexturePlatform {
  linux('rhwp_native/texture_linux'),
  macos('rhwp_native/texture_macos'),
  macosMetalView('rhwp_native/metal_view_macos'),
  windows('rhwp_native/texture_windows');

  const NativeTexturePlatform(this.channelName);
  final String channelName;
  static NativeTexturePlatform? current({bool preferContinuous = false}) =>
      switch (defaultTargetPlatform) {
        TargetPlatform.linux => linux,
        TargetPlatform.macOS => preferContinuous ? macosMetalView : macos,
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
  _Plan(NativePageViewport viewport, this.dpr, int page, this.clear)
    : viewport = viewport.viewport,
      pages = [(page: page, viewport: viewport)];
  _Plan.document(NativeDocumentViewport layout, this.clear)
    : viewport = layout.viewport,
      dpr = layout.deviceScale,
      pages = [
        for (final entry in layout.visiblePages.entries)
          (page: entry.key, viewport: entry.value),
      ];
  final Size viewport;
  final double dpr;
  final int clear;
  final List<({int page, NativePageViewport viewport})> pages;
  Object? _pageKey(int index) {
    if (index >= pages.length) return null;
    final entry = pages[index];
    return (
      entry.page,
      entry.viewport.page.width,
      entry.viewport.page.height,
      entry.viewport.zoom,
      entry.viewport.origin,
    );
  }

  Object get key => (
    viewport,
    dpr,
    clear,
    _pageKey(0),
    _pageKey(1),
    _pageKey(2),
    _pageKey(3),
  );
  ({int width, int height}) geometry({bool tiled = false}) {
    final physicalWidth = viewport.width * dpr;
    final physicalHeight = viewport.height * dpr;
    if (!dpr.isFinite ||
        dpr <= 0 ||
        dpr > 16 ||
        !physicalWidth.isFinite ||
        !physicalHeight.isFinite ||
        physicalWidth <= 0 ||
        physicalHeight <= 0 ||
        physicalWidth > (tiled ? 8192 : 4096) ||
        physicalHeight > (tiled ? 8192 : 4096)) {
      throw const NativeCallException(
        'viewportLimit',
        'GPU viewport exceeds its dimension bound.',
      );
    }
    final width = physicalWidth.ceil(), height = physicalHeight.ceil();
    if (width * height > (tiled ? 8 : 4) * 1024 * 1024 ||
        pages.isEmpty ||
        pages.length > 4 ||
        clear < 0 ||
        clear > 0xffffffff) {
      throw const NativeCallException(
        'viewportLimit',
        'GPU viewport exceeds its pixel/page bound.',
      );
    }
    for (final entry in pages) {
      final scale = entry.viewport.zoom * dpr;
      final origin = entry.viewport.origin * dpr;
      if (entry.page < 0 ||
          !scale.isFinite ||
          scale <= 0 ||
          scale > 16 ||
          !origin.dx.isFinite ||
          !origin.dy.isFinite ||
          origin.dx.abs() > 16777216 ||
          origin.dy.abs() > 16777216) {
        throw const NativeCallException(
          'viewportLimit',
          'GPU placement exceeds its coordinate bound.',
        );
      }
    }
    return (width: width, height: height);
  }

  Map<String, Object?> submission(
    int texture,
    List<_OwnedSnapshot> snapshots, {
    bool tiled = false,
  }) {
    final size = geometry(tiled: tiled);
    return {
      'textureId': texture,
      'stamp': snapshots.first.stamp!.toJson(),
      'width': size.width,
      'height': size.height,
      'clearColor': clear,
      'placements': [
        for (var index = 0; index < pages.length; ++index)
          {
            'snapshot': snapshots[index].id,
            'scale': pages[index].viewport.zoom * dpr,
            'originX': pages[index].viewport.origin.dx * dpr,
            'originY': pages[index].viewport.origin.dy * dpr,
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
    this.allowImmutableOwnershipTransfer = false,
    this.useTileGroups = false,
    this.trace,
  }) : channel = useTileGroups
           ? NativeTileGroupChannel(
               channel ?? MethodChannel(platform.channelName),
               windows: platform == NativeTexturePlatform.windows,
             )
           : channel ?? MethodChannel(platform.channelName) {
    if (allowImmutableOwnershipTransfer &&
        platform != NativeTexturePlatform.windows) {
      throw ArgumentError(
        'Immutable ownership transfer is supported only by the Windows texture backend.',
      );
    }
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
  final bool allowImmutableOwnershipTransfer;

  /// Uses two existing native producer slots with one presentation owner.
  /// Requires an explicit tile-group capability from the selected plugin.
  final bool useTileGroups;
  NativeTileGroupChannel? get _tileChannel => channel is NativeTileGroupChannel
      ? channel as NativeTileGroupChannel
      : null;
  List<int> get tileTextureIds => _tileChannel?.textures ?? const [];
  List<Rect> get physicalTiles => _tileChannel?.physicalTiles ?? const [];
  double _outputDeviceScale = 1;
  double get outputDeviceScale => _outputDeviceScale;

  /// Optional producer diagnostics; no compositor display or GPU fence claim.
  final NativeGpuTrace? trace;
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
      _paused = false,
      _wanted = false,
      _stopped = false,
      _closing = false,
      _notifierDisposed = false;
  _Plan? _plan;
  List<_OwnedSnapshot> _frames = const [];
  _OwnedSnapshot? get _frame => _frames.firstOrNull;
  Object? _configurationKey;
  Future<void>? _closeFuture;
  Future<void>? _platformCloseFuture;
  Future<void>? _nativeRetireAckFuture;
  bool _platformCloseActive = false,
      _platformCloseSent = false,
      _createAttempted = false,
      _platformUnregistered = false,
      _nativeCloseComplete = false,
      _negotiatedRetireAck = false,
      _nativeRetireReady = false,
      _retirementPending = false;
  int _attachedViews = 0;
  Object? _retirementDiagnostic;
  Timer? _frameTimeout, _admissionTimeout, _retryTimer;
  int _visibilitySerial = 0, _nativeVisibleGeneration = -1;
  bool _coverPending = false,
      _coverWorking = false,
      _nativeCoverConfirmed = false;
  Completer<void>? _coverWaiter;
  Future<void>? _pauseFuture;
  Timer? _coverWaitDeadline;
  final _releases = <Future<void>>{};
  static int get retainedOwnedSnapshots => _OwnedSnapshot.retained.length;

  void _trace(
    NativeGpuTraceStage stage, {
    int? generation,
    NativeGpuTraceFailure? failure,
    Map<String, Object?>? status,
  }) {
    final recorder = trace;
    if (recorder == null) return;
    final counters = <NativeGpuCounter, int>{
      NativeGpuCounter.dartOwnedSnapshots: retainedOwnedSnapshots,
      NativeGpuCounter.controllerRpc: _rpc,
      NativeGpuCounter.isolateRpc: _globalRpc,
      NativeGpuCounter.visiblePages: _plan?.pages.length ?? 0,
    };
    final limits = status?['limits'];
    if (limits is Map) {
      for (final entry in const {
        'producerOwnedBytes': NativeGpuCounter.pluginProducerOwnedBytes,
        'importPendingBytes': NativeGpuCounter.pluginImportPendingBytes,
        'reservedTargetBytes': NativeGpuCounter.pluginReservedTargetBytes,
        'outstandingFrames': NativeGpuCounter.pluginOutstandingFrames,
        'descriptorTickets': NativeGpuCounter.pluginDescriptorTickets,
        'wholeFrameReadbacks': NativeGpuCounter.pluginWholeFrameReadbacks,
      }.entries) {
        final value = limits[entry.key];
        if (value is int) counters[entry.value] = value;
      }
    }
    final readbacks = status?['wholeFrameReadbacks'];
    if (readbacks is int) {
      counters[NativeGpuCounter.pluginWholeFrameReadbacks] = readbacks;
    }
    recorder.record(
      stage,
      generation: generation ?? _generation,
      failure: failure,
      counters: counters,
    );
  }

  void _traceSuperseded() {
    if (_generation > 0 && (_working || _phase == NativeTexturePhase.pending)) {
      _trace(NativeGpuTraceStage.superseded);
    }
  }

  bool get canDisplay =>
      _phase == NativeTexturePhase.visible &&
      _frames.isNotEmpty &&
      _frames.every(_localCurrent);
  bool get needsCpuFallback => _phase == NativeTexturePhase.fallback;
  bool get isPaused => _paused;
  bool get _nativeView => platform == NativeTexturePlatform.macosMetalView;
  bool get _requiresRetireAck => _nativeView || _negotiatedRetireAck;

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
        !_paused &&
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
    _configurePlan(plan, invalidationGeneration);
  }

  /// Publishes the visible pages of one Native document layout in one target.
  /// Every capture uses the same full stamp and retains the isolate-wide cap.
  void configureDocument(
    NativeDocumentViewport viewport, {
    int clearColor = 0xffdddddd,
    int invalidationGeneration = 0,
  }) {
    _configurePlan(
      _Plan.document(viewport, clearColor),
      invalidationGeneration,
    );
  }

  /// Covers the mounted registration while a replacement layout is pending.
  /// No old geometry may be captured or exposed until a new plan is supplied.
  void suspendViewport() {
    if (_closing || _stopped || _plan == null) return;
    _traceSuperseded();
    _plan = null;
    _configurationKey = null;
    ++_generation;
    _wanted = false;
    _phase = NativeTexturePhase.pending;
    _frameTimeout?.cancel();
    _admissionTimeout?.cancel();
    _retryTimer?.cancel();
    _retryTimer = null;
    _requestNativeCover();
    _signal();
  }

  void _configurePlan(_Plan plan, int invalidationGeneration) {
    final key = (plan.key, invalidationGeneration);
    if (key == _configurationKey) return;
    _configurationKey = key;
    _plan = plan;
    // Geometry admission runs before a native snapshot is allocated.
    try {
      final size = plan.geometry(tiled: useTileGroups);
      _tileChannel?.plan(size.width, size.height);
      _outputDeviceScale = plan.dpr;
    } catch (error) {
      _fail(error, fallback: true);
      return;
    }
    _invalidate(configurationChanged: true);
  }

  void invalidate() => _invalidate();

  void _invalidate({bool configurationChanged = false}) {
    if (_closing || _stopped || _plan == null || document.isDisposed) return;
    _traceSuperseded();
    ++_generation;
    _view = document.beginPresentation();
    _raster = document.rasterInvalidationGeneration;
    _wanted = !_paused;
    _phase = NativeTexturePhase.pending;
    if (configurationChanged) _trace(NativeGpuTraceStage.configured);
    _trace(NativeGpuTraceStage.invalidated);
    _frameTimeout?.cancel();
    _retryTimer?.cancel();
    _retryTimer = null;
    _admissionTimeout?.cancel();
    if (allowImmutableOwnershipTransfer && !_paused) {
      final generation = _generation;
      _admissionTimeout = Timer(operationTimeout, () {
        if (_current(generation)) {
          _fail(
            TimeoutException('GPU frame admission timed out.'),
            fallback: true,
          );
        }
      });
    }
    _requestNativeCover();
    _signal(); // Cover cached platform pixels in this same UI turn.
    if (!_paused) unawaited(_pump());
  }

  /// Stops presentation admission and waits for the native view to be hidden.
  /// Native registration, current frame leases and late RPC observers remain.
  Future<void> pause() {
    if (_closing || _stopped) {
      return Future.error(
        const NativeCallException('closed', 'Presentation is closing.'),
      );
    }
    if (_paused) return _pauseFuture ?? Future.value();
    _paused = true;
    if (_plan != null) {
      invalidate();
    } else {
      ++_generation;
      _phase = NativeTexturePhase.pending;
      _requestNativeCover();
      _signal();
    }
    return _pauseFuture = _waitForNativeCover();
  }

  /// Republishes the newest source and geometry after a presentation pause.
  void resume() {
    if (!_paused || _closing || _stopped) return;
    _paused = false;
    _pauseFuture = null;
    invalidate();
  }

  void _requestNativeCover() {
    if (!_nativeView || _platformUnregistered) return;
    ++_visibilitySerial;
    _nativeVisibleGeneration = -1;
    _nativeCoverConfirmed = false;
    _coverPending = true;
    unawaited(_flushNativeCover());
  }

  bool _visibilityReceipt(
    Map<String, Object?> reply,
    int id,
    int serial,
    bool covered,
  ) =>
      reply['textureId'] == id &&
      reply['visibilitySerial'] == serial &&
      reply['accepted'] == true &&
      reply['covered'] == covered;

  void _finishCoverWaiter([Object? error]) {
    final waiter = _coverWaiter;
    if (waiter == null) return;
    if (error == null &&
        (!_nativeCoverConfirmed || _coverPending || _coverWorking)) {
      return;
    }
    _coverWaiter = null;
    _coverWaitDeadline?.cancel();
    _coverWaitDeadline = null;
    if (error == null) {
      waiter.complete();
    } else {
      waiter.completeError(error);
    }
  }

  Future<void> _waitForNativeCover() {
    if (!_nativeView ||
        (_nativeCoverConfirmed && !_coverPending && !_coverWorking) ||
        (textureId == null && !_createAttempted)) {
      return Future.value();
    }
    final existing = _coverWaiter;
    if (existing != null) return existing.future;
    final waiter = Completer<void>();
    _coverWaiter = waiter;
    _coverWaitDeadline = Timer(operationTimeout, () {
      final error = TimeoutException('Native view cover was not confirmed.');
      _finishCoverWaiter(error);
      if (!_closing && !_stopped) _fail(error, fallback: true);
    });
    unawaited(_flushNativeCover());
    return waiter.future;
  }

  Future<void> _flushNativeCover() async {
    if (_coverWorking ||
        !_coverPending ||
        textureId == null ||
        _platformUnregistered ||
        _rpc >= 3 ||
        _globalRpc >= 8) {
      return;
    }
    _coverWorking = true;
    try {
      while (_coverPending && !_platformUnregistered && textureId != null) {
        if (_rpc >= 3 || _globalRpc >= 8) break;
        final id = textureId!, serial = _visibilitySerial;
        _coverPending = false;
        final reply = await _invoke('setCovered', {
          'textureId': id,
          'covered': true,
          'visibilitySerial': serial,
        });
        if (serial != _visibilitySerial) continue;
        if (!_visibilityReceipt(reply, id, serial, true)) {
          throw const NativeCallException(
            'invalidVisibilityReply',
            'Native view cover was not acknowledged.',
          );
        }
        _nativeCoverConfirmed = true;
      }
    } catch (error) {
      _finishCoverWaiter(error);
      if (!_closing && !_stopped) _fail(error, fallback: _canFallback(error));
    } finally {
      _coverWorking = false;
      _finishCoverWaiter();
    }
  }

  Future<bool> _uncoverNative(_OwnedSnapshot frame, int generation) async {
    if (!_current(generation) || !_localCurrent(frame)) return false;
    if (_nativeVisibleGeneration == generation) return true;
    if (!_nativeCoverConfirmed || _coverPending || _coverWorking) return false;
    final id = textureId!, serial = ++_visibilitySerial;
    final reply = await _invoke('setCovered', {
      'textureId': id,
      'covered': false,
      'visibilitySerial': serial,
      'stamp': frame.stamp!.toJson(),
    });
    if (!_current(generation) ||
        !_localCurrent(frame) ||
        serial != _visibilitySerial) {
      _requestNativeCover();
      return false;
    }
    if (!_visibilityReceipt(reply, id, serial, false) ||
        !frame.stamp!.matches(reply['renderedStamp'])) {
      throw const NativeCallException(
        'stalePresentation',
        'Native view did not confirm the current full stamp.',
      );
    }
    _nativeCoverConfirmed = false;
    _nativeVisibleGeneration = generation;
    return true;
  }

  void _signal() {
    if (!_notifierDisposed) notifyListeners();
  }

  bool _current(int generation) =>
      !_closing &&
      !_stopped &&
      !_paused &&
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
    final future = channel
        .invokeMethod<Object?>(method, {
          ...arguments,
          if (_negotiatedRetireAck && arguments.containsKey('textureId'))
            'deviceGeneration': _device,
        })
        .then(_map);
    final observed = future.then(
      (value) {
        --_rpc;
        --_globalRpc;
        _trace(NativeGpuTraceStage.resourceSampled);
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
        _trace(NativeGpuTraceStage.resourceSampled);
        failed?.call(error);
        if ((_closing || _stopped) &&
            method != 'close' &&
            !_platformCloseActive) {
          unawaited(_closePlatform());
        }
        Error.throwWithStackTrace(error, stack);
      },
    );
    _trace(NativeGpuTraceStage.resourceSampled);
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

  bool _receipt(
    _OwnedSnapshot snapshot,
    Map<String, Object?> result, {
    bool requireIds = false,
  }) {
    final ids = result['acceptedSnapshots'];
    return snapshot.stamp!.matches(result['acceptedStamp']) &&
        ((!requireIds && ids == null) ||
            (ids is List && ids.contains(snapshot.id)));
  }

  bool _definiteRejection(Object error) =>
      error is MissingPluginException ||
      (error is NativeCallException &&
          const {
            'textureRpcLimit',
            'unsupportedPresentation',
            'viewportLimit',
          }.contains(error.code)) ||
      (error is PlatformException &&
          const {
            'backpressure',
            'staleFrame',
            'contextChanged',
            'notReady',
            'closing',
            'invalidArgument',
            'arguments',
            'limitExceeded',
            'targetTooLarge',
            'retireLimit',
            'featureDisabled',
            'unsupported',
            'closed',
          }.contains(error.code));
  void _submitReply(
    _OwnedSnapshot snapshot,
    Map<String, Object?> result, {
    bool requireIds = false,
  }) {
    snapshot.sending = false;
    if (result['accepted'] == true &&
        _receipt(snapshot, result, requireIds: requireIds)) {
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

  bool _retryableBackpressure(Object error) =>
      allowImmutableOwnershipTransfer &&
      error is PlatformException &&
      error.code == 'backpressure' &&
      error.details is Map &&
      (error.details as Map)['backpressureRetryable'] == true;

  void _retryAdmission(int generation) {
    if (!_current(generation)) return;
    _retryTimer?.cancel();
    _retryTimer = Timer(pollInterval, () {
      _retryTimer = null;
      if (!_current(generation)) return;
      _view = document.beginPresentation();
      _raster = document.rasterInvalidationGeneration;
      _wanted = true;
      unawaited(_pump());
    });
  }

  Future<void> _pump() async {
    if (_working ||
        !_wanted ||
        _closing ||
        _stopped ||
        _paused ||
        _plan == null) {
      return;
    }
    _working = true;
    final generation = _generation, plan = _plan!, view = _view;
    var pollAccepted = false;
    _wanted = false;
    final snapshots = <_OwnedSnapshot>[];
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
        final created = await _invoke(
          'create',
          {
            'libraryPath': presentation.libraryPath,
            if (platform == NativeTexturePlatform.windows ||
                platform == NativeTexturePlatform.macos)
              'closeAcknowledgementRequired': true,
            if (allowImmutableOwnershipTransfer)
              'publicationMode': 'immutableOwnershipTransfer',
          },
          received: (reply) {
            final id = reply['textureId'], device = reply['deviceGeneration'];
            if (id is int && id > 0 && device is int && device > 0) {
              _negotiatedRetireAck =
                  (platform == NativeTexturePlatform.windows ||
                      platform == NativeTexturePlatform.macos) &&
                  reply['closeAcknowledgementRequired'] == true &&
                  reply['limits'] is Map &&
                  (reply['limits'] as Map)['closeAcknowledgementRequired'] ==
                      true;
              textureId = id;
              _device = device;
              if (_nativeView) unawaited(_flushNativeCover());
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
        if (allowImmutableOwnershipTransfer &&
            (created['publicationMode'] != 'immutableOwnershipTransfer' ||
                created['limits'] is! Map ||
                (created['limits'] as Map)['continuousPresentationSupported'] !=
                    true)) {
          throw const NativeCallException(
            'unsupportedPresentation',
            'The Windows plugin did not negotiate immutable ownership transfer.',
          );
        }
        if (_nativeView &&
            (created['presentationKind'] != 'nativeView' ||
                created['viewType'] != 'rhwp_native/metal_view' ||
                created['ready'] != false)) {
          throw const NativeCallException(
            'unsupportedPresentation',
            'The macOS plugin did not negotiate a native Metal view.',
          );
        }
      }
      await _waitReady(generation).timeout(operationTimeout);
      if (!_current(generation)) return;
      // Reserve the entire batch before the first owner call. Partial admission
      // cannot exceed the four-credit cap or publish only part of a viewport.
      for (var index = 0; index < plan.pages.length; ++index) {
        final owned = _OwnedSnapshot.reserve(presentation);
        if (owned == null) {
          throw const NativeCallException(
            'snapshotCreditLimit',
            'Owned or ambiguous snapshot credit is exhausted.',
          );
        }
        snapshots.add(owned);
      }
      _trace(NativeGpuTraceStage.captureStarted, generation: generation);
      for (var index = 0; index < snapshots.length; ++index) {
        if (!_current(generation)) return;
        final owned = snapshots[index];
        final rawCapture = document.capturePresentation(
          page: plan.pages[index].page,
          viewGeneration: view,
          deviceGeneration: _device,
          deviceScale: plan.dpr,
          releaseSnapshot: presentation.releaseSnapshot,
        );
        owned.capturePending = true;
        final capture = rawCapture.then(
          (result) {
            owned.capturePending = false;
            try {
              owned.bind(result);
              if (!_current(generation)) _release(owned);
              return result;
            } catch (_) {
              _release(owned);
              rethrow;
            }
          },
          onError: (Object error, StackTrace stack) {
            owned.capturePending = false;
            _release(owned);
            Error.throwWithStackTrace(error, stack);
          },
        );
        await capture.timeout(operationTimeout);
        if (!_current(generation)) return;
        if (!_localCurrent(owned) ||
            !snapshots.first.stamp!.matches(owned.stamp!.toJson()) ||
            !await presentation
                .snapshotCheck(owned.id!, owned.stamp!)
                .timeout(operationTimeout)) {
          throw const NativeCallException(
            'stalePresentation',
            'The viewport snapshots do not share one current full stamp.',
          );
        }
      }
      if (!_current(generation)) return;
      _trace(NativeGpuTraceStage.captureCompleted, generation: generation);
      for (final owned in snapshots) {
        owned.sending = true;
      }
      final requireIds = useTileGroups || snapshots.length > 1;
      Map<String, Object?> reply;
      try {
        _trace(NativeGpuTraceStage.submitStarted, generation: generation);
        reply = await _invoke(
          'submit',
          plan.submission(textureId!, snapshots, tiled: useTileGroups),
          received: (reply) {
            for (final owned in snapshots) {
              _submitReply(owned, reply, requireIds: requireIds);
            }
          },
          failed: (error) {
            for (final owned in snapshots) {
              _submitError(owned, error);
            }
          },
        );
      } catch (error) {
        // Only an explicit pre-accept rejection can release/retry a whole batch.
        if (_retryableBackpressure(error) &&
            snapshots.every((s) => !s.sending && !s.ambiguous) &&
            _current(generation)) {
          _trace(
            NativeGpuTraceStage.backpressure,
            generation: generation,
            failure: NativeGpuTraceFailure.backpressure,
          );
          await Future.wait(snapshots.map((s) => s.releaseOwned()))
              .timeout(operationTimeout);
          if (_current(generation)) _retryAdmission(generation);
          return;
        }
        for (final owned in snapshots) {
          if (!owned.settled && (owned.sending || owned.ambiguous)) {
            owned.ambiguous = true;
            await _reconcile(owned, requireIds: requireIds);
          }
        }
        rethrow;
      }
      if (reply['accepted'] != true || snapshots.any((s) => !s.settled)) {
        for (final owned in snapshots) {
          if (owned.ambiguous) await _reconcile(owned, requireIds: requireIds);
        }
        throw NativeCallException(
          snapshots.any((s) => s.ambiguous)
              ? 'ambiguousOwnership'
              : 'backpressure',
          'The viewport batch was not conclusively accepted.',
        );
      }
      _trace(
        NativeGpuTraceStage.accepted,
        generation: generation,
        status: reply,
      );
      if (!_current(generation)) return;
      _admissionTimeout?.cancel();
      _admissionTimeout = null;
      _frames = List.unmodifiable(snapshots);
      _phase = NativeTexturePhase.pending;
      _publicationDeadline(generation);
      _signal();
      pollAccepted = true;
    } catch (error) {
      if (_current(generation)) _fail(error, fallback: _canFallback(error));
    } finally {
      final callerReleases = <Future<void>>[];
      for (final owned in snapshots) {
        if (!owned.settled && !owned.sending && !owned.ambiguous) {
          _release(owned);
          final releasing = owned.releasing;
          if (releasing != null) callerReleases.add(releasing);
        }
      }
      // A replacement four-page plan needs all four credits. Do not race the
      // confirmed caller releases of an obsolete batch; ambiguous/pending IDs
      // are never refunded by this wait or by a new viewport generation.
      if (_wanted && !_closing && !_stopped && callerReleases.isNotEmpty) {
        try {
          await Future.wait(callerReleases).timeout(operationTimeout);
        } catch (error) {
          if (!_closing && !_stopped) {
            _fail(error, fallback: _canFallback(error));
          }
        }
      }
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
      if (_wanted && !_closing && !_stopped && !_paused) {
        unawaited(_pump());
      } else if (pollAccepted &&
          _current(generation) &&
          identical(_frame, snapshots.first)) {
        // Probe exactly once after acceptance, once submission is serialized.
        // A slow producer returns to the existing periodic polling cadence.
        unawaited(_poll());
      }
    }
  }

  Future<void> _waitReady(int generation) async {
    while (_current(generation)) {
      final status = await _invoke('status', {'textureId': textureId});
      _trace(
        NativeGpuTraceStage.statusPolled,
        generation: generation,
        status: status,
      );
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

  Future<void> _reconcile(
    _OwnedSnapshot snapshot, {
    bool requireIds = false,
  }) async {
    if (textureId == null || _rpc >= 3) return;
    try {
      final result = await _invoke('status', {'textureId': textureId});
      if (_receipt(snapshot, result, requireIds: requireIds)) {
        snapshot.transferred();
      }
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
            'targetTooLarge',
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
            'targetTooLarge',
            'retireLimit',
            'notReady',
            'closing',
            'closed',
            'contextChanged',
            'staleFrame',
          }.contains(error.code));

  void _fail(Object error, {required bool fallback}) {
    if (_closing || _stopped) return;
    final code = switch (error) {
      NativeCallException() => error.code,
      PlatformException() => error.code,
      _ => '',
    };
    final failure = switch (error) {
      TimeoutException() => NativeGpuTraceFailure.timeout,
      MissingPluginException() => NativeGpuTraceFailure.unsupported,
      _ => switch (code) {
        'stalePresentation' || 'staleFrame' => NativeGpuTraceFailure.stale,
        'backpressure' ||
        'snapshotCreditLimit' ||
        'textureRpcLimit' => NativeGpuTraceFailure.backpressure,
        'unsupportedPresentation' ||
        'unsupported' ||
        'featureDisabled' => NativeGpuTraceFailure.unsupported,
        'deviceLost' || 'contextChanged' => NativeGpuTraceFailure.device,
        _ => NativeGpuTraceFailure.other,
      },
    };
    diagnostic = error;
    _stopped = true;
    _wanted = false;
    _phase = fallback ? NativeTexturePhase.fallback : NativeTexturePhase.failed;
    _timer.cancel();
    _frameTimeout?.cancel();
    _admissionTimeout?.cancel();
    _retryTimer?.cancel();
    _retryTimer = null;
    _finishCoverWaiter(error);
    _requestNativeCover();
    if (failure == NativeGpuTraceFailure.timeout) {
      _trace(NativeGpuTraceStage.timeout, failure: failure);
    } else if (failure == NativeGpuTraceFailure.stale) {
      _trace(NativeGpuTraceStage.stale, failure: failure);
    }
    _trace(NativeGpuTraceStage.failed, failure: failure);
    if (fallback) _trace(NativeGpuTraceStage.fallback, failure: failure);
    _signal();
    unawaited(_closePlatform());
  }

  void _tick() {
    if (_nativeView) unawaited(_flushNativeCover());
    if (_working ||
        _polling ||
        _closing ||
        _stopped ||
        _paused ||
        _plan == null ||
        _retryTimer != null) {
      return;
    }
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
        _paused ||
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
      _trace(
        NativeGpuTraceStage.statusPolled,
        generation: generation,
        status: status,
      );
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
          _requestNativeCover();
          _publicationDeadline(generation);
          _signal();
        }
        return;
      }
      // A matching stamp alone cannot prove that every page was published.
      if ((useTileGroups || _frames.length > 1) &&
          !_frames.every((s) => _receipt(s, status, requireIds: true))) {
        throw const NativeCallException(
          'stalePresentation',
          'The published viewport is missing a page receipt.',
        );
      }
      for (final owned in _frames) {
        if (!await presentation
            .snapshotCheck(owned.id!, owned.stamp!)
            .timeout(operationTimeout)) {
          throw const NativeCallException(
            'stalePresentation',
            'The full source stamp is no longer current.',
          );
        }
      }
      if (!_current(generation) || !_localCurrent(frame)) return;
      if (_nativeView && (!await _uncoverNative(frame, generation))) return;
      if (_current(generation) && _localCurrent(frame)) {
        final changed = _phase != NativeTexturePhase.visible;
        _phase = NativeTexturePhase.visible;
        _frameTimeout?.cancel();
        if (changed) {
          _trace(
            NativeGpuTraceStage.producerReady,
            generation: generation,
            status: status,
          );
          _signal();
        }
      }
    } catch (error) {
      if (!_closing && !_stopped && generation == _generation) {
        _fail(error, fallback: _canFallback(error));
      }
    } finally {
      _polling = false;
    }
  }

  bool _observeCloseStatus(
    Map<String, Object?> status, {
    bool fromRetireAck = false,
  }) {
    final unregistered =
        status['registered'] == false ||
        status['unregistered'] == true ||
        status['textureUnregisteredNotification'] == true;
    final registrationChanged = unregistered && !_platformUnregistered;
    if (unregistered) _platformUnregistered = true;
    if (_requiresRetireAck &&
        unregistered &&
        status['closed'] == true &&
        status['ownerFinished'] == true &&
        (!_nativeView || status['nativeViewLayerDetached'] == true)) {
      _nativeRetireReady = true;
    }
    final complete = switch (platform) {
      NativeTexturePlatform.linux =>
        status['cleanupComplete'] == true && status['registered'] == false,
      NativeTexturePlatform.macosMetalView ||
      NativeTexturePlatform.macos ||
      NativeTexturePlatform.windows =>
        _requiresRetireAck
            ? fromRetireAck &&
                  _nativeRetireReady &&
                  status['textureId'] == textureId &&
                  status['acknowledged'] == true &&
                  status['closed'] == true &&
                  status['ownerFinished'] == true &&
                  status['registered'] == false &&
                  (_nativeView
                      ? status['nativeViewLayerDetached'] == true
                      : status['deviceGeneration'] == _device)
            : status['ownerFinished'] == true && unregistered,
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
    if (_requiresRetireAck &&
        _nativeRetireReady &&
        !_platformCloseActive &&
        !_nativeCloseComplete &&
        _nativeRetireAckFuture == null) {
      _ackRetirementWhenAvailable();
    }
    return complete;
  }

  void _ackRetirementWhenAvailable() {
    if (_rpc < 3 && _globalRpc < 8 && _nativeRetireAckFuture == null) {
      unawaited(_ackNativeRetirement().catchError((Object _) {}));
    }
  }

  Future<void> _ackNativeRetirement() {
    final existing = _nativeRetireAckFuture;
    if (existing != null) return existing;
    final done = Completer<void>();
    _nativeRetireAckFuture =
        done.future; // One acknowledgement, including late replies.
    unawaited(() async {
      try {
        await _invoke(
          'retireAck',
          {'textureId': textureId},
          received: (reply) {
            if (!_observeCloseStatus(reply, fromRetireAck: true)) {
              throw const NativeCallException(
                'retirePending',
                'Native close acknowledgement was not confirmed.',
              );
            }
          },
        );
        done.complete();
      } catch (error, stack) {
        _retirementUnconfirmed(error);
        done.completeError(error, stack);
      }
    }());
    return done.future;
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
      // A bounded close wait may expire just after observing owner completion.
      // Send its one explicit ack; its original late observer remains live.
      if (_requiresRetireAck && _nativeRetireReady && !_nativeCloseComplete) {
        _ackRetirementWhenAvailable();
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
        if (_requiresRetireAck && _nativeRetireReady) {
          await _ackNativeRetirement();
          if (_nativeCloseComplete) finish();
          return;
        }
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
    final generation = _generation;
    final superseded =
        _generation > 0 && (_working || _phase == NativeTexturePhase.pending);
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
    _admissionTimeout?.cancel();
    _retryTimer?.cancel();
    _retryTimer = null;
    _finishCoverWaiter(
      const NativeCallException('closed', 'Presentation is closing.'),
    );
    _requestNativeCover();
    if (superseded) {
      _trace(NativeGpuTraceStage.superseded, generation: generation);
    }
    _trace(NativeGpuTraceStage.closeRequested, generation: generation);
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
  }) : documentViewport = null;

  const NativeTextureView.document({
    super.key,
    required this.controller,
    required this.documentViewport,
    required this.cpuFallback,
    this.invalidationGeneration = 0,
    this.coverColor = const Color(0xffdddddd),
  }) : viewport = null,
       page = 0;

  final NativeTextureController controller;
  final NativePageViewport? viewport;
  final NativeDocumentViewport? documentViewport;
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
    final layout = widget.documentViewport;
    if (layout != null) {
      widget.controller.configureDocument(
        layout,
        clearColor: widget.coverColor.toARGB32(),
        invalidationGeneration: widget.invalidationGeneration,
      );
    } else if (widget.viewport != null) {
      widget.controller.configure(
        widget.viewport!,
        deviceScale: MediaQuery.devicePixelRatioOf(context),
        page: widget.page,
        clearColor: widget.coverColor.toARGB32(),
        invalidationGeneration: widget.invalidationGeneration,
      );
    } else {
      widget.controller.suspendViewport();
    }
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
    Widget output(int id) => IgnorePointer(
      child: controller.platform == NativeTexturePlatform.macosMetalView
          ? AppKitView(
              key: ValueKey(id),
              viewType: 'rhwp_native/metal_view',
              creationParams: {'handle': id},
              creationParamsCodec: const StandardMessageCodec(),
              hitTestBehavior: PlatformViewHitTestBehavior.transparent,
            )
          : Texture(textureId: id, filterQuality: FilterQuality.low),
    );
    return ClipRect(
      child: Stack(
        fit: StackFit.expand,
        children: [
          if (controller.shouldPaintTexture && controller.useTileGroups)
            for (
              var index = 0;
              index < controller.tileTextureIds.length;
              ++index
            )
              if (index < controller.physicalTiles.length)
                Positioned(
                  key: ValueKey(
                    'rhwp-native-tile-${controller.tileTextureIds[index]}',
                  ),
                  left:
                      controller.physicalTiles[index].left /
                      controller.outputDeviceScale,
                  top:
                      controller.physicalTiles[index].top /
                      controller.outputDeviceScale,
                  width:
                      controller.physicalTiles[index].width /
                      controller.outputDeviceScale,
                  height:
                      controller.physicalTiles[index].height /
                      controller.outputDeviceScale,
                  child: output(controller.tileTextureIds[index]),
                ),
          if (controller.shouldPaintTexture && !controller.useTileGroups)
            output(controller.textureId!),
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
      ),
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
