// Executable integration entry point for a disposable Linux Flutter runner.
// Requires the actual P99 plugin, owner ABI 2, and P93 metadata. Copy this file
// to runner/lib/main.dart; keep ffi and the reference package as dependencies.
// RHWP_PROBE_NATIVE_LIBRARY/RHWP_PROBE_OWNER_LIBRARY name canonical libraries.
// RHWP_CONTROLLER_PROBE_EXPECT_UNSUPPORTED=1 requires an unavailable full-stamp
// capability to fail closed before any platform create/capture. There is no
// raw snapshotInfo substitute for the production full-stamp contract here.
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

// Only the explicitly cross-thread stats operation is added to the production
// metadata client's narrow surface. No legacy HOST, Core, replay, or GPU pointer.
class _StatsReader {
  _StatsReader(String path) : library = DynamicLibrary.open(path) {
    request = library.lookupFunction<_RequestNative, _RequestDart>(
      'rhwp_presentation_request',
    );
    free = library.lookupFunction<_FreeNative, _FreeDart>('rhwp_string_free');
  }
  final DynamicLibrary library;
  late final _RequestDart request;
  late final _FreeDart free;
  Map<String, Object?> read() {
    final bytes = utf8.encode('{"op":"stats"}');
    final input = calloc<Uint8>(bytes.length);
    Pointer<Utf8> output = nullptr;
    try {
      input.asTypedList(bytes.length).setAll(0, bytes);
      output = request(input, bytes.length);
      if (output == nullptr) throw StateError('no native stats');
      var length = 0;
      while (length <= 64 * 1024 && output.cast<Uint8>()[length] != 0) {
        length++;
      }
      if (length > 64 * 1024) throw StateError('stats reply exceeds bound');
      final result = Map<String, Object?>.from(
        jsonDecode(output.toDartString(length: length)) as Map,
      );
      if (result['ok'] != true || result['stats'] is! Map) {
        throw StateError('stats failed: $result');
      }
      return Map<String, Object?>.from(result['stats']! as Map);
    } finally {
      if (output != nullptr) free(output);
      calloc.free(input);
    }
  }
}

class _RecordedMetadata implements NativePresentation {
  _RecordedMetadata(this.inner);
  final NativePresentation inner;
  final verified = <int, NativePresentationStamp>{};
  final observed = <int>{};
  int checks = 0, releases = 0;
  Completer<void>? _pause;
  int _minimumPausedRevision = 0;
  bool pauseReached = false;
  void pauseNextRevision(int revision) {
    _minimumPausedRevision = revision;
    _pause = Completer<void>();
  }

  void resume() {
    final pause = _pause;
    _pause = null;
    if (pause != null && !pause.isCompleted) pause.complete();
  }

  @override
  String get libraryPath => inner.libraryPath;
  @override
  bool get available => inner.available;
  @override
  Future<Map<String, Object?>> snapshotInfo(int snapshot) =>
      inner.snapshotInfo(snapshot);
  @override
  Future<void> releaseSnapshot(int snapshot) async {
    releases++;
    await inner.releaseSnapshot(snapshot);
  }

  @override
  Future<bool> snapshotCheck(
    int snapshot,
    NativePresentationStamp stamp,
  ) async {
    checks++;
    observed.add(snapshot);
    if (observed.length > 8) {
      throw StateError('probe snapshot observation bound exceeded');
    }
    final current = await inner.snapshotCheck(snapshot, stamp);
    if (current) {
      verified[snapshot] = stamp;
      final pause = _pause;
      if (pause != null && stamp.revision >= _minimumPausedRevision) {
        pauseReached = true;
        // Delay only delivery of an already performed real full check, so the
        // actual cover frame is deterministic even on a very fast worker/GPU.
        await pause.future.timeout(const Duration(seconds: 2));
      }
    }
    return current;
  }
}

class _RecordedChannel extends MethodChannel {
  _RecordedChannel() : super('rhwp_native/texture_linux');
  final counts = <String, int>{};
  int total = 0;
  @override
  Future<T?> invokeMethod<T>(String method, [dynamic arguments]) {
    if (++total > 512) {
      return Future<T?>.error(StateError('probe method-call bound exceeded'));
    }
    counts[method] = (counts[method] ?? 0) + 1;
    // Delegate to the real plugin. There is no mock binary messenger.
    return super
        .invokeMethod<T>(method, arguments)
        .timeout(const Duration(seconds: 3));
  }

  Future<Map<String, Object?>> status(int texture) async =>
      Map<String, Object?>.from(
        await invokeMethod<Map<Object?, Object?>>('status', {
              'textureId': texture,
            }) ??
            {},
      );
}

void _require(bool condition, String message) {
  if (!condition) throw StateError(message);
}

void _zeroStats(Map<String, Object?> stats) {
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
}

Future<void> _until(
  FutureOr<bool> Function() condition,
  String description,
) async {
  final elapsed = Stopwatch()..start();
  while (elapsed.elapsed < const Duration(seconds: 8)) {
    if (await condition()) return;
    await Future<void>.delayed(const Duration(milliseconds: 20));
  }
  throw TimeoutException(description);
}

void main() => runApp(const MaterialApp(home: _Probe()));

class _Probe extends StatefulWidget {
  const _Probe();
  @override
  State<_Probe> createState() => _ProbeState();
}

class _ProbeState extends State<_Probe> {
  final _viewKey = GlobalKey();
  final _channel = _RecordedChannel();
  NativeDocument? _document;
  NativeTextureController? _controller;
  NativePageViewport? _viewport;
  _RecordedMetadata? _metadata;
  _StatsReader? _stats;
  final _phases = <String>[];
  String _stage = 'connect';
  int _fallbackBuilds = 0;
  late final Timer _watchdog;
  @override
  void initState() {
    super.initState();
    _watchdog = Timer(const Duration(seconds: 30), () {
      stderr.writeln(
        'RHWP_CONTROLLER_VIEW_PROBE ${jsonEncode({'ok': false, 'stage': _stage, 'error': 'watchdog deadline'})}',
      );
      exit(124);
    });
    WidgetsBinding.instance.addPostFrameCallback((_) => unawaited(probe()));
  }

  void _changed() {
    final phase = _controller!.phase.name;
    if (_phases.isEmpty || _phases.last != phase) {
      if (_phases.length >= 32) throw StateError('probe phase bound exceeded');
      _phases.add(phase);
    }
  }

  bool _inView(bool Function(Widget widget) match) {
    final context = _viewKey.currentContext;
    if (context is! Element) return false;
    var found = false, visited = 0;
    void visit(Element element) {
      if (++visited > 128) {
        throw StateError('probe element traversal bound exceeded');
      }
      if (match(element.widget)) found = true;
      element.visitChildElements(visit);
    }

    visit(context);
    return found;
  }

  bool get _covered =>
      _inView(
        (widget) => widget.key == const ValueKey('rhwp-native-texture-cover'),
      ) &&
      _inView(
        (widget) =>
            widget is ColoredBox && (widget.color.toARGB32() >> 24) == 255,
      );

  Future<Map<String, Object?>> _published(int minimumRevision) async {
    final controller = _controller!,
        document = _document!,
        metadata = _metadata!;
    Map<String, Object?>? result;
    await _until(() async {
      if (controller.phase == NativeTexturePhase.failed ||
          controller.needsCpuFallback) {
        throw StateError(
          'controller stopped before publication: ${controller.diagnostic}',
        );
      }
      if (!controller.canDisplay) return false;
      final status = await _channel.status(controller.textureId!);
      final revision = document.info!.revision;
      if (revision < minimumRevision) return false;
      for (final entry in metadata.verified.entries.toList().reversed) {
        final stamp = entry.value;
        if (stamp.documentId == document.info!.documentId &&
            stamp.revision == revision &&
            stamp.viewGeneration == document.presentationGeneration &&
            stamp.matches(status['renderedStamp'])) {
          _require(
            await metadata.snapshotCheck(entry.key, stamp),
            'independent full snapshot check failed',
          );
          if (!controller.canDisplay || _covered) {
            return false; // Wait for the actual Flutter rebuild.
          }
          result = {
            'snapshot': entry.key,
            'stamp': stamp.toJson(),
            'status': status,
          };
          return true;
        }
      }
      return false;
    }, 'current rendered stamp/full check did not become visible');
    return result!;
  }

  Future<void> probe() async {
    try {
      final path = await File(
        Platform.environment['RHWP_PROBE_NATIVE_LIBRARY']!,
      ).resolveSymbolicLinks();
      final owner = await NativeOwnerTransport.connect(
        dispatcherLibrary: Platform.environment['RHWP_PROBE_OWNER_LIBRARY']!,
        nativeLibrary: path,
      );
      _document = await NativeDocument.connect(owner);
      final document = _document!;
      await document.newDocument();
      const position = {
        'section': 0,
        'paragraph': 0,
        'scalarOffset': 0,
        'cellPath': <Object>[],
      };
      await document.request('beginEdit', {
        'selection': {'anchor': position, 'focus': position},
      });
      await document.request('pasteText', {
        'text': 'Controller / view full metadata',
      });
      _stats = _StatsReader(path);
      _zeroStats(_stats!.read());
      final metadata = _metadata = _RecordedMetadata(
        await NativePresentation.connect(nativeLibrary: path),
      );
      final expectUnsupported =
          Platform.environment['RHWP_CONTROLLER_PROBE_EXPECT_UNSUPPORTED'] ==
          '1';
      _require(
        expectUnsupported ? !metadata.available : metadata.available,
        'unexpected fullStampChecks capability for requested probe mode',
      );
      final controller = _controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        channel: _channel,
        pollInterval: const Duration(milliseconds: 20),
        operationTimeout: const Duration(seconds: 3),
      );
      controller.addListener(_changed);
      _viewport = NativePageViewport.fit(
        await document.pageSize(0),
        const Size(320, 240),
      );
      setState(() {});
      _stage = 'initialView';
      await _until(() => _viewKey.currentContext != null, 'view did not mount');
      if (expectUnsupported) {
        await _until(
          () => controller.needsCpuFallback,
          'missing full checks did not select fallback',
        );
        await WidgetsBinding.instance.endOfFrame;
        _require(
          _channel.total == 0 &&
              controller.textureId == null &&
              metadata.checks == 0 &&
              !controller.canDisplay &&
              _covered &&
              _fallbackBuilds > 0,
          'unavailable capability sent GPU work or exposed a Texture',
        );
        final beforeDispose = _stats!.read();
        _zeroStats(beforeDispose);
        await _teardown();
        _finish({
          'ok': true,
          'expectedUnsupported': true,
          'fullStampChecks': false,
          'platformCalls': _channel.counts,
          'fallbackBuilds': _fallbackBuilds,
          'beforeDisposeStats': beforeDispose,
          'stats': _stats!.read(),
          'phases': _phases,
        });
      }
      final first = await _published(document.info!.revision);
      _stage = 'mutate';
      final previousRevision = document.info!.revision;
      metadata.pauseNextRevision(previousRevision + 1);
      final edited = document.request('pasteText', {
        'text': ' / fresh revision',
      });
      _require(
        !controller.canDisplay &&
            controller.phase == NativeTexturePhase.pending,
        'accepted mutation did not invalidate synchronously',
      );
      await edited;
      await _until(
        () =>
            metadata.pauseReached &&
            _covered &&
            _inView((widget) => widget is Texture),
        'new revision did not paint an opaque cover over the mounted Texture',
      );
      metadata.resume();
      _stage = 'freshPublication';
      final second = await _published(previousRevision + 1);
      _require(
        (second['stamp']! as Map)['viewGeneration'] >
            (first['stamp']! as Map)['viewGeneration'],
        'fresh publication did not advance presentation generation',
      );
      _stage = 'close';
      final texture = controller.textureId!;
      await controller.close().timeout(const Duration(seconds: 6));
      Map<String, Object?>? closed;
      await _until(() async {
        closed = await _channel.status(texture);
        return closed!['cleanupComplete'] == true &&
            closed!['registered'] == false;
      }, 'controller close did not finish its own raster retire/unregister');
      _require(
        !controller.canDisplay &&
            closed!['snapshotReleaseFailures'] == 0 &&
            closed!['liveRasterSlots'] == 0 &&
            closed!['pendingRetireSlots'] == 0 &&
            closed!['orphanedRasterSlots'] == 0,
        'normal texture cleanup left resources: $closed',
      );
      await _teardown();
      final stats = _stats!.read();
      _zeroStats(stats);
      _require(
        NativeTextureController.retainedOwnedSnapshots == 0,
        'caller snapshot credit remained',
      );
      _finish({
        'ok': true,
        'expectedUnsupported': false,
        'fullStampChecks': true,
        'first': first,
        'second': second,
        'closed': closed,
        'stats': stats,
        'checks': metadata.checks,
        'callerReleases': metadata.releases,
        'platformCalls': _channel.counts,
        'phases': _phases,
        'synchronousInvalidation': true,
        'opaqueCoverObserved': true,
        'hardwareAccelerationQualified': false,
        'pixelComparisonQualified': false,
      });
    } catch (error, stack) {
      try {
        await _teardown();
      } catch (_) {
        /* Preserve the primary failure. */
      }
      _finish({
        'ok': false,
        'stage': _stage,
        'error': '$error',
        'stack': '$stack',
        'platformCalls': _channel.counts,
        'phases': _phases,
      });
    }
  }

  Future<void> _teardown() async {
    _metadata?.resume();
    final controller = _controller;
    if (controller != null) {
      await controller.close().timeout(const Duration(seconds: 6));
      controller.removeListener(_changed);
      controller.dispose();
      _controller = null;
    }
    await _document?.dispose().timeout(const Duration(seconds: 6));
    _document = null;
    final stats = _stats;
    if (stats != null) _zeroStats(stats.read());
    _require(
      NativeTextureController.retainedOwnedSnapshots == 0,
      'caller snapshot credit remained',
    );
  }

  Never _finish(Map<String, Object?> result) {
    _watchdog.cancel();
    stdout.writeln('RHWP_CONTROLLER_VIEW_PROBE ${jsonEncode(result)}');
    exit(result['ok'] == true ? 0 : 1);
  }

  @override
  Widget build(BuildContext context) {
    final controller = _controller, viewport = _viewport;
    return Scaffold(
      body: controller == null || viewport == null
          ? const SizedBox()
          : Center(
              child: SizedBox(
                width: 320,
                height: 240,
                child: NativeTextureView(
                  key: _viewKey,
                  controller: controller,
                  viewport: viewport,
                  cpuFallback: (_) {
                    _fallbackBuilds++;
                    return const ColoredBox(color: Color(0xff334455));
                  },
                ),
              ),
            ),
    );
  }
}
