// Disposable actual desktop runner entry point. The external driver captures
// visible compositor pixels; no widget/mock screenshot is accepted as sampling.
// CPU replay is an explicit test oracle over the same immutable snapshot.
import 'dart:async';
import 'dart:convert';
import 'dart:ffi' hide Size;
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

typedef _RequestNative = Pointer<Utf8> Function(Pointer<Uint8>, UintPtr);
typedef _RequestDart = Pointer<Utf8> Function(Pointer<Uint8>, int);
typedef _FreeNative = Void Function(Pointer<Utf8>);
typedef _FreeDart = void Function(Pointer<Utf8>);
typedef _DataNative = Pointer<Uint8> Function(Uint64, Pointer<UintPtr>);
typedef _DataDart = Pointer<Uint8> Function(int, Pointer<UintPtr>);

void _require(bool value, String message) {
  if (!value) throw StateError(message);
}

class DesktopConsumerOracle {
  DesktopConsumerOracle(String path) {
    final library = DynamicLibrary.open(path);
    _request = library.lookupFunction<_RequestNative, _RequestDart>(
      'rhwp_presentation_request',
    );
    _free = library.lookupFunction<_FreeNative, _FreeDart>('rhwp_string_free');
    _data = library.lookupFunction<_DataNative, _DataDart>(
      'rhwp_presentation_lease_data',
    );
  }
  late final _RequestDart _request;
  late final _FreeDart _free;
  late final _DataDart _data;
  Map<String, Object?>? lastGeometry;

  Map<String, Object?> call(Map<String, Object?> request) {
    final bytes = utf8.encode(jsonEncode(request));
    _require(
      bytes.isNotEmpty && bytes.length <= 64 * 1024,
      'oracle request bound',
    );
    final input = calloc<Uint8>(bytes.length);
    Pointer<Utf8> output = nullptr;
    try {
      input.asTypedList(bytes.length).setAll(0, bytes);
      output = _request(input, bytes.length);
      _require(output != nullptr, 'missing oracle reply');
      var length = 0;
      while (length <= 1024 * 1024 && output.cast<Uint8>()[length] != 0) {
        length++;
      }
      _require(length <= 1024 * 1024, 'oracle reply bound');
      final result = Map<String, Object?>.from(
        jsonDecode(output.toDartString(length: length)) as Map,
      );
      _require(result['ok'] == true, 'oracle failed: $result');
      return result;
    } finally {
      if (output != nullptr) _free(output);
      calloc.free(input);
    }
  }

  Uint8List pixels({
    required int snapshot,
    required NativePresentationStamp stamp,
    required NativePageViewport viewport,
    required double dpr,
    required int width,
    required int height,
  }) {
    lastGeometry = null;
    _require(
      width > 0 && height > 0 && width * height <= 4 * 1024 * 1024,
      'oracle pixel bound',
    );
    final frame = call({
      'op': 'renderCpuBatch',
      'stamp': stamp.toJson(),
      'viewport': {'width': width, 'height': height},
      'placements': [
        {
          'snapshot': snapshot,
          'scale': viewport.zoom * dpr,
          'originX': viewport.origin.dx * dpr,
          'originY': viewport.origin.dy * dpr,
          'backgroundColor': 0xffffffff,
        },
      ],
      'clearColor': 0xff55575a,
    });
    final id = frame['frame']! as int;
    int? lease;
    final length = calloc<UintPtr>();
    try {
      final info = frame['info']! as Map;
      final descriptor = info['descriptor']! as Map;
      _require(
        info['coordinateSpace'] == 'viewportPixels' &&
            descriptor['pixelFormat'] == 'rgba8Premultiplied' &&
            descriptor['colorSpace'] == 'srgb' &&
            descriptor['origin'] == 'topLeft' &&
            descriptor['width'] == width &&
            descriptor['height'] == height &&
            descriptor['stride'] == width * 4,
        'CPU batch descriptor does not match the displayed target',
      );
      lease =
          call({
                'op': 'frameAcquire',
                'frame': id,
                'stamp': stamp.toJson(),
              })['lease']!
              as int;
      final pointer = _data(lease, length);
      _require(
        pointer != nullptr && length.value == width * height * 4,
        'invalid CPU oracle lease span',
      );
      final copied = Uint8List.fromList(pointer.asTypedList(length.value));
      lastGeometry = {
        'snapshot': snapshot,
        'stamp': stamp.toJson(),
        'viewport': {'width': width, 'height': height},
        'placement': {
          'scale': viewport.zoom * dpr,
          'originX': viewport.origin.dx * dpr,
          'originY': viewport.origin.dy * dpr,
        },
        'coordinateSpace': info['coordinateSpace'],
        'descriptor': Map<String, Object?>.from(descriptor),
      };
      return copied;
    } finally {
      calloc.free(length);
      try {
        if (lease != null) call({'op': 'leaseRelease', 'lease': lease});
      } finally {
        call({'op': 'frameRelease', 'frame': id});
      }
    }
  }
}

class _ObservedPresentation implements NativePresentation {
  _ObservedPresentation(this.inner, {this.maxChecks = 1024});
  final NativePresentation inner;
  final int maxChecks;
  int? snapshot;
  NativePresentationStamp? stamp;
  int checks = 0;
  @override
  bool get available => inner.available;
  @override
  String get libraryPath => inner.libraryPath;
  @override
  Future<Map<String, Object?>> snapshotInfo(int snapshot) =>
      inner.snapshotInfo(snapshot);
  @override
  Future<void> releaseSnapshot(int snapshot) => inner.releaseSnapshot(snapshot);
  @override
  Future<bool> snapshotCheck(
    int snapshot,
    NativePresentationStamp stamp,
  ) async {
    _require(++checks <= maxChecks, 'probe metadata call bound');
    final current = await inner.snapshotCheck(snapshot, stamp);
    if (current) {
      this.snapshot = snapshot;
      this.stamp = stamp;
    }
    return current;
  }
}

void main() => runApp(const MaterialApp(home: _Probe()));

class _Probe extends StatefulWidget {
  const _Probe();
  @override
  State<_Probe> createState() => _ProbeState();
}

class _ProbeState extends State<_Probe> {
  final _textureKey = GlobalKey();
  NativeDocument? _document;
  NativeTextureController? _controller;
  NativePageViewport? _viewport;
  _ObservedPresentation? _metadata;
  DesktopConsumerOracle? _oracle;
  late Directory _output;
  late String _nonce;
  late MethodChannel _channel;
  late Timer _watchdog;
  var _physicalSize = const Size(320, 240);
  var _stage = 'setup';
  var _frameCount = 3;
  var _sourceReferenceEnabled = false;
  var _continuousBackend = false;
  var _nativeCoverProbe = false;
  @override
  void initState() {
    super.initState();
    _watchdog = Timer(Duration(seconds: Platform.isLinux ? 290 : 110), () {
      stderr.writeln('RHWP_DESKTOP_CONSUMER_PROBE watchdog at $_stage');
      exit(124);
    });
    WidgetsBinding.instance.addPostFrameCallback((_) => unawaited(_run()));
  }

  Future<void> _write(String name, Map<String, Object?> value) async {
    final temporary = File('${_output.path}/$name.tmp');
    await temporary.writeAsString(jsonEncode(value), flush: true);
    await temporary.rename('${_output.path}/$name');
  }

  Future<Map<String, Object?>> _status() async => Map<String, Object?>.from(
    await _channel
            .invokeMethod<Map<Object?, Object?>>('status', {
              'textureId': _controller!.textureId,
              if (_metadata?.stamp != null &&
                  (Platform.isWindows ||
                      _controller!.platform == NativeTexturePlatform.macos))
                'deviceGeneration': _metadata!.stamp!.deviceGeneration,
            })
            .timeout(const Duration(seconds: 5)) ??
        {},
  );

  Future<void> _until(FutureOr<bool> Function() check, String message) async {
    final deadline = Stopwatch()..start();
    while (deadline.elapsed < const Duration(seconds: 25)) {
      if (await check()) return;
      await Future<void>.delayed(const Duration(milliseconds: 20));
    }
    throw TimeoutException(message);
  }

  Future<void> _publication(int index, String stage) async {
    _stage = stage;
    final controller = _controller!,
        document = _document!,
        metadata = _metadata!;
    WidgetsBinding.instance.scheduleFrame();
    await WidgetsBinding.instance.endOfFrame;
    await _until(() {
      _require(
        controller.phase != NativeTexturePhase.failed &&
            !controller.needsCpuFallback,
        'bounded GPU publication failed: ${controller.diagnostic}',
      );
      final stamp = metadata.stamp;
      return controller.canDisplay &&
          stamp != null &&
          stamp.revision == document.info!.revision &&
          stamp.viewGeneration == document.presentationGeneration;
    }, 'current full-stamp publication did not become visible');
    // Keep the real Texture mounted and give the engine actual paint frames.
    for (var frame = 0; frame < 3; frame++) {
      WidgetsBinding.instance.scheduleFrame();
      await WidgetsBinding.instance.endOfFrame;
    }
    final status = await _status(), stamp = metadata.stamp!;
    _require(
      stamp.matches(status['renderedStamp'] ?? status['acceptedStamp']) &&
          (Platform.isLinux
              ? status['registered'] == true &&
                    status['framesSubmitted'] is int &&
                    (status['framesSubmitted']! as int) >= index
              : status['hasFrame'] == true) &&
          await metadata.inner.snapshotCheck(metadata.snapshot!, stamp),
      'native publication/source changed before compositor capture',
    );
    final dpr = MediaQuery.devicePixelRatioOf(context);
    final pixels = _oracle!.pixels(
      snapshot: metadata.snapshot!,
      stamp: stamp,
      viewport: _viewport!,
      dpr: dpr,
      width: _physicalSize.width.toInt(),
      height: _physicalSize.height.toInt(),
    );
    final cpuGeometry = _oracle!.lastGeometry!;
    await File('${_output.path}/cpu-$index.rgba')
        .writeAsBytes(pixels, flush: true);
    var referenceScale = 1;
    Map<String, Object?>? referenceGeometry;
    if (_sourceReferenceEnabled) {
      // The same immutable snapshot and complete stamp are replayed directly.
      // Only target/placement geometry is scaled; configure/capture is not run.
      final view = _viewport!;
      for (final factor in [4, 3, 2]) {
        if (_physicalSize.width * factor <= 4096 &&
            _physicalSize.height * factor <= 4096 &&
            _physicalSize.width * _physicalSize.height * factor * factor <=
                4 * 1024 * 1024 &&
            view.zoom * dpr * factor <= 64 &&
            (view.origin.dx * dpr * factor).abs() <= 16777216 &&
            (view.origin.dy * dpr * factor).abs() <= 16777216) {
          referenceScale = factor;
          break;
        }
      }
      _require(
        referenceScale > 1,
        'referenceUnavailable: CPU geometry/16 MiB cap',
      );
      final factor = referenceScale;
      final reference = _oracle!.pixels(
        snapshot: metadata.snapshot!,
        stamp: stamp,
        viewport: NativePageViewport.placed(
          view.page,
          view.viewport * factor.toDouble(),
          zoom: view.zoom * factor,
          origin: view.origin * factor.toDouble(),
        ),
        dpr: dpr,
        width: _physicalSize.width.toInt() * factor,
        height: _physicalSize.height.toInt() * factor,
      );
      _require(
        await metadata.inner.snapshotCheck(metadata.snapshot!, stamp),
        'source changed during independent CPU reference replay',
      );
      referenceGeometry = _oracle!.lastGeometry!;
      await File('${_output.path}/cpu-reference-$index.rgba')
          .writeAsBytes(reference, flush: true);
    }
    final box = _textureKey.currentContext!.findRenderObject()! as RenderBox;
    final origin = box.localToGlobal(Offset.zero) * dpr;
    _require(
      (origin.dx - origin.dx.round()).abs() < .0001 &&
          (origin.dy - origin.dy.round()).abs() < .0001,
      'probe texture must begin on a physical pixel boundary',
    );
    await _write('stage-$index.json', {
      'protocol': 1,
      'nonce': _nonce,
      'index': index,
      'stage': stage,
      'pid': pid,
      'platform': Platform.operatingSystem,
      'width': _physicalSize.width.toInt(),
      'height': _physicalSize.height.toInt(),
      'physicalOffset': [origin.dx.round(), origin.dy.round()],
      'deviceScale': dpr,
      'stamp': stamp.toJson(),
      'limits': status['limits'],
      if (_sourceReferenceEnabled) ...{
        'cpuReferenceScale': referenceScale,
        'geometryProof': {'cpu': cpuGeometry, 'reference': referenceGeometry},
      },
      'nativeStatus': status,
    });
    final ack = File('${_output.path}/ack-$index.json');
    await _until(
      () => ack.exists(),
      'external compositor driver did not acknowledge $stage',
    );
    final reply = jsonDecode(await ack.readAsString()) as Map;
    _require(
      reply['protocol'] == 1 &&
          reply['nonce'] == _nonce &&
          reply['index'] == index,
      'foreign compositor acknowledgement',
    );
  }

  Future<void> _heldNativeCover() async {
    _stage = 'held-native-cover';
    final controller = _controller!;
    await controller.pause();
    for (var frame = 0; frame < 2; frame++) {
      WidgetsBinding.instance.scheduleFrame();
      await WidgetsBinding.instance.endOfFrame;
    }
    final status = await _status();
    _require(
      controller.isPaused &&
          !controller.canDisplay &&
          status['covered'] == true &&
          status['presentationKind'] == 'nativeView' &&
          status['viewBound'] == true &&
          status['layerReady'] == true,
      'mounted native view did not acknowledge its held cover',
    );
    var nativeViewMounted = false;
    var elements = 0;
    void visit(Element element) {
      _require(++elements <= 256, 'native view subtree bound');
      if (element.widget is AppKitView) nativeViewMounted = true;
      element.visitChildElements(visit);
    }

    (_textureKey.currentContext! as Element).visitChildElements(visit);
    _require(nativeViewMounted, 'held cover unmounted the native view');
    final dpr = MediaQuery.devicePixelRatioOf(context);
    final box = _textureKey.currentContext!.findRenderObject()! as RenderBox;
    final origin = box.localToGlobal(Offset.zero) * dpr;
    _require(
      (origin.dx - origin.dx.round()).abs() < .0001 &&
          (origin.dy - origin.dy.round()).abs() < .0001,
      'held cover lost its physical pixel alignment',
    );
    await _write('cover-1.json', {
      'protocol': 1,
      'nonce': _nonce,
      'pid': pid,
      'platform': Platform.operatingSystem,
      'width': _physicalSize.width.toInt(),
      'height': _physicalSize.height.toInt(),
      'physicalOffset': [origin.dx.round(), origin.dy.round()],
      'deviceScale': dpr,
      'clearColor': 0xff55575a,
      'nativeCovered': status['covered'],
      'presentationKind': status['presentationKind'],
      'nativeViewMounted': nativeViewMounted,
      'viewBound': status['viewBound'],
      'layerReady': status['layerReady'],
      'visibilitySerial': status['visibilitySerial'],
    });
    final ack = File('${_output.path}/cover-ack-1.json');
    await _until(() => ack.exists(), 'held cover was not externally sampled');
    final reply = jsonDecode(await ack.readAsString()) as Map;
    _require(
      reply['protocol'] == 1 && reply['nonce'] == _nonce && reply['index'] == 1,
      'foreign held-cover acknowledgement',
    );
    controller.resume();
  }

  Future<void> _run() async {
    var succeeded = false;
    try {
      _require(
        Platform.isWindows || Platform.isMacOS || Platform.isLinux,
        'actual desktop target is required',
      );
      final config = jsonDecode(
        await File(Platform.environment['RHWP_CONSUMER_PROBE_CONFIG']!)
            .readAsString(),
      ) as Map;
      _require(
        config['protocol'] == 1 && config['nonce'] is String,
        'invalid driver config',
      );
      _nonce = config['nonce']! as String;
      final backend = config['backend'];
      final platform = switch (backend) {
        null => NativeTexturePlatform.current()!,
        'windowsImmutableTransfer' when Platform.isWindows =>
          NativeTexturePlatform.windows,
        'macosMetalView' when Platform.isMacOS =>
          NativeTexturePlatform.macosMetalView,
        _ => throw StateError('probe backend does not match the actual OS'),
      };
      _continuousBackend = Platform.isLinux || backend != null;
      _nativeCoverProbe = (config['nativeCoverProbe'] as bool?) ?? false;
      _require(
        !_nativeCoverProbe || platform == NativeTexturePlatform.macosMetalView,
        'native cover sampling requires the macOS native view backend',
      );
      final presentationKind = platform == NativeTexturePlatform.macosMetalView
          ? 'nativeView'
          : 'externalTexture';
      _require(
        config['presentationKind'] == null ||
            config['presentationKind'] == presentationKind,
        'probe presentation kind does not match the selected backend',
      );
      _sourceReferenceEnabled =
          (config['cpuReference'] as bool?) ?? _continuousBackend;
      _frameCount =
          (config['frameCount'] as int?) ?? (config['frames'] as int?) ?? 3;
      _require(
        config['frameCount'] == null ||
            config['frames'] == null ||
            config['frameCount'] == config['frames'],
        'ambiguous probe frame count',
      );
      _require(
        _continuousBackend
            ? _frameCount >= 3 && _frameCount <= 64
            : _frameCount == 3,
        'probe frame count exceeds its platform contract',
      );
      if (_continuousBackend) {
        _watchdog.cancel();
        _watchdog = Timer(const Duration(seconds: 590), () {
          stderr.writeln('RHWP_DESKTOP_CONSUMER_PROBE watchdog at $_stage');
          exit(124);
        });
      }
      _output = Directory(config['output']! as String);
      final native = await File(config['native']! as String)
          .resolveSymbolicLinks();
      final owner = await NativeOwnerTransport.connect(
        dispatcherLibrary: config['dispatcher']! as String,
        nativeLibrary: native,
      );
      final document = _document = await NativeDocument.connect(owner);
      final fixture = File(config['fixture']! as String);
      _require(
        await fixture.length() <= maxReferenceBytes,
        'probe fixture byte limit',
      );
      final source = await fixture.open();
      try {
        final bytes = await source.read(maxReferenceBytes + 1);
        await document.openBytes(bytes);
      } finally {
        await source.close();
      }
      const point = {
        'section': 0,
        'paragraph': 0,
        'scalarOffset': 0,
        'cellPath': <Object>[],
      };
      await document.request('beginEdit', {
        'selection': {'anchor': point, 'focus': point},
      });
      final metadata = _metadata = _ObservedPresentation(
        await NativePresentation.connect(nativeLibrary: native),
        maxChecks: _frameCount > 4 ? 8192 : 1024,
      );
      _require(
        metadata.available,
        'full-stamp presentation capability is required',
      );
      _oracle = DesktopConsumerOracle(native);
      _channel = MethodChannel(platform.channelName);
      _controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: platform,
        allowImmutableOwnershipTransfer: backend == 'windowsImmutableTransfer',
        operationTimeout: const Duration(seconds: 8),
      );
      _viewport = NativePageViewport.fit(
        await document.pageSize(0),
        _physicalSize / MediaQuery.devicePixelRatioOf(context),
      );
      setState(() {});
      await _publication(1, 'initial');
      if (_nativeCoverProbe) await _heldNativeCover();
      await document.request('pasteText', {
        // The fixture begins with a plain empty paragraph before the picture.
        // Six blank paragraphs move its visible image without depending on a
        // system font or losing the inline table/picture source control.
        'text': '\n\n\n\n\n\n',
      });
      await _publication(2, 'revision');
      _physicalSize = const Size(352, 256);
      _viewport = NativePageViewport.fit(
        await document.pageSize(0),
        _physicalSize / MediaQuery.devicePixelRatioOf(context),
      );
      setState(() {});
      await _publication(3, 'resize');
      final baseline = _viewport!;
      for (var index = 4; index <= _frameCount; index++) {
        final dpr = MediaQuery.devicePixelRatioOf(context);
        _viewport = NativePageViewport.placed(
          baseline.page,
          baseline.viewport,
          zoom: baseline.zoom,
          origin: baseline.origin + Offset(index.isEven ? 4 / dpr : 0, 2 / dpr),
        );
        setState(() {});
        await _publication(index, 'continuous');
      }
      _stage = 'close';
      await _controller!.close();
      _require(
        _controller!.phase == NativeTexturePhase.closed,
        'texture retirement was not confirmed',
      );
      var closed = await _status();
      if (backend == 'windowsImmutableTransfer' ||
          platform == NativeTexturePlatform.macosMetalView) {
        // The registrar may signal removal immediately before its last owned
        // descriptor/creator pins are drained. Observe the completed status,
        // without interpreting it as a consumer GPU completion fence.
        await _until(() async {
          closed = await _status();
          if (platform == NativeTexturePlatform.macosMetalView) {
            return closed['ownerFinished'] == true &&
                closed['registered'] == false &&
                closed['nativeViewLayerDetached'] == true;
          }
          final limits = closed['limits'] as Map?;
          return closed['ownerFinished'] == true &&
              (closed['unregistered'] == true ||
                  closed['registered'] == false) &&
              limits?['producerOwnedFrames'] == 0 &&
              limits?['importPendingFrames'] == 0 &&
              limits?['descriptorTickets'] == 0;
        }, 'native registration/owned resources did not finish retiring');
      }
      _require(
        Platform.isLinux
            ? closed['cleanupComplete'] == true &&
                  closed['liveRasterSlots'] == 0 &&
                  closed['pendingRetireSlots'] == 0 &&
                  closed['orphanedRasterSlots'] == 0 &&
                  closed['abandonedTargets'] == 0 &&
                  closed['snapshotReleaseFailures'] == 0
            : closed['ownerFinished'] == true,
        'producer owner did not finish',
      );
      _require(
        closed['unregistered'] == true ||
            closed['textureUnregisteredNotification'] == true ||
            closed['registered'] == false,
        'actual texture unregistration was not observed',
      );
      await document.dispose();
      final stats = _oracle!.call({'op': 'stats'})['stats']! as Map;
      for (final name in [
        'sources',
        'snapshotHandles',
        'retainedSnapshots',
        'snapshotEstimatedBytes',
        'frameHandles',
        'retainedFrames',
        'frameBytes',
        'leases',
        'inFlightReplays',
      ]) {
        _require(stats[name] == 0, 'native teardown $name is not zero: $stats');
      }
      await _write('result.json', {
        'protocol': 1,
        'nonce': _nonce,
        'ok': true,
        'closed': closed,
        'stats': stats,
        'consumerGpuCompletionVerified': false,
        'continuousPresentationSupported':
            _continuousBackend &&
            _frameCount > 4 &&
            (Platform.isLinux ||
                (closed['limits']
                        as Map?)?['continuousPresentationSupported'] ==
                    true),
        'presentationKind': presentationKind,
        'publications': _frameCount,
      });
      succeeded = true;
    } catch (error, stack) {
      stderr.writeln('RHWP_DESKTOP_CONSUMER_PROBE $_stage: $error\n$stack');
    } finally {
      _watchdog.cancel();
      if (!succeeded) {
        try {
          await _controller?.close();
        } catch (_) {
          /* Disposable bounded quarantine remains. */
        }
        try {
          await _document?.dispose();
        } catch (_) {
          /* Preserve the original failure. */
        }
      }
      exit(succeeded ? 0 : 1);
    }
  }

  @override
  Widget build(BuildContext context) {
    final dpr = MediaQuery.devicePixelRatioOf(context);
    return Scaffold(
      backgroundColor: const Color(0xff55575a),
      body: Padding(
        padding: EdgeInsets.all(16 / dpr),
        child: _viewport == null
            ? const SizedBox.shrink()
            : SizedBox(
                key: _textureKey,
                width: _physicalSize.width / dpr,
                height: _physicalSize.height / dpr,
                child: NativeTextureView(
                  controller: _controller!,
                  viewport: _viewport!,
                  cpuFallback: (_) =>
                      const ColoredBox(color: Color(0xff55575a)),
                ),
              ),
      ),
    );
  }
}
