// Actual GTK/Flutter compositor probe. Each native snapshot is captured once;
// independent CPU replays use bounded strips of that same immutable snapshot.
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'desktop_consumer_probe.dart' show DesktopConsumerOracle;

void require(bool value, String message) {
  if (!value) throw StateError(message);
}

class ObservedPresentation implements NativePresentation {
  ObservedPresentation(this.inner);
  final NativePresentation inner;
  final snapshots = <int, NativePresentationStamp>{};
  int checks = 0, releases = 0;
  @override
  bool get available => inner.available;
  @override
  String get libraryPath => inner.libraryPath;
  @override
  Future<Map<String, Object?>> snapshotInfo(int id) => inner.snapshotInfo(id);
  @override
  Future<void> releaseSnapshot(int id) async {
    ++releases;
    await inner.releaseSnapshot(id);
  }

  @override
  Future<bool> snapshotCheck(int id, NativePresentationStamp stamp) async {
    require(++checks <= 2048, 'bounded metadata polling');
    final current = await inner.snapshotCheck(id, stamp);
    if (current) snapshots[id] = stamp;
    require(snapshots.length <= 8, 'bounded captured snapshot observation');
    return current;
  }
}

class RecordedGroupChannel extends MethodChannel {
  RecordedGroupChannel() : super('rhwp_native/texture_linux');
  Map<String, Object?>? submitted;
  int calls = 0, submits = 0;
  @override
  Future<T?> invokeMethod<T>(String method, [dynamic arguments]) {
    require(++calls <= 1024, 'bounded platform polling');
    if (method == 'submitGroup') {
      ++submits;
      submitted = Map<String, Object?>.from(arguments as Map);
    }
    return super.invokeMethod<T>(method, arguments);
  }
}

void main() => runApp(const MaterialApp(home: TileProbe()));

class TileProbe extends StatefulWidget {
  const TileProbe({super.key});
  @override
  State<TileProbe> createState() => _TileProbeState();
}

class _TileProbeState extends State<TileProbe> {
  final textureKey = GlobalKey();
  final channel = RecordedGroupChannel();
  NativeDocument? document;
  NativeTextureController? controller;
  NativePageViewport? viewport;
  ObservedPresentation? metadata;
  DesktopConsumerOracle? oracle;
  late Directory output;
  late String nonce;
  late Timer watchdog;
  Size physical = const Size(4098, 480);
  String stage = 'connect';
  @override
  void initState() {
    super.initState();
    watchdog = Timer(const Duration(seconds: 280), () {
      stderr.writeln('RHWP_TILE_GROUP_PROBE watchdog at $stage');
      exit(124);
    });
    WidgetsBinding.instance.addPostFrameCallback((_) => unawaited(run()));
  }

  Future<void> write(String name, Object value) async {
    final temporary = File('${output.path}/$name.tmp');
    await temporary.writeAsString(jsonEncode(value), flush: true);
    await temporary.rename('${output.path}/$name');
  }

  Future<void> until(
    FutureOr<bool> Function() check,
    String description,
  ) async {
    final time = Stopwatch()..start();
    while (time.elapsed < const Duration(seconds: 25)) {
      if (await check()) return;
      await Future<void>.delayed(const Duration(milliseconds: 20));
    }
    throw TimeoutException(description);
  }

  Future<Map<String, Object?>> status() async => Map<String, Object?>.from(
    await channel.invokeMapMethod<String, Object?>('statusGroup', {
          'groupId': controller!.textureId,
          'deviceGeneration':
              (channel.submitted!['stamp']! as Map)['deviceGeneration'],
        }) ??
        const {},
  );

  NativePageViewport placed(NativePageSize page) {
    final dpr = MediaQuery.devicePixelRatioOf(context);
    final size = physical / dpr;
    return NativePageViewport.placed(
      page,
      size,
      zoom: .55,
      origin: Offset((size.width - page.width * .55) / 2 + .25 / dpr, .5 / dpr),
    );
  }

  Future<void> publication(int index) async {
    stage = 'publication-$index';
    await until(() {
      require(
        controller!.phase != NativeTexturePhase.failed &&
            !controller!.needsCpuFallback,
        'group did not publish: ${controller!.diagnostic}',
      );
      return controller!.canDisplay && channel.submits == index;
    }, 'both tile publications');
    for (var frame = 0; frame < 3; ++frame) {
      WidgetsBinding.instance.scheduleFrame();
      await WidgetsBinding.instance.endOfFrame;
    }
    final raw = await status();
    final members = raw['members']! as List;
    final submission = channel.submitted!;
    final stamp = NativePresentationStamp.fromJson(
      Map<String, Object?>.from(submission['stamp']! as Map),
    );
    final tiles = submission['tiles']! as List;
    final ids = <int>{};
    for (var tile = 0; tile < 2; ++tile) {
      require(
        (members[tile] as Map)['textureId'] ==
                controller!.tileTextureIds[tile] &&
            stamp.matches((members[tile] as Map)['renderedStamp']),
        'each physical tile has the current full stamp',
      );
      for (final placement in (tiles[tile] as Map)['placements'] as List) {
        ids.add((placement as Map)['snapshot']! as int);
      }
    }
    require(
      ids.length == 1 && metadata!.snapshots.length == index,
      'capture each page once, not once per tile',
    );
    final snapshot = ids.single;
    require(
      await metadata!.inner.snapshotCheck(snapshot, stamp),
      'source changed before independent CPU replay',
    );
    final strips = <Map<String, Object?>>[];
    final dpr = MediaQuery.devicePixelRatioOf(context);
    // Keep the complete document and native seam inside one independently
    // observable strip. Its 2x 4096-by-at-most-992 source stays below 16 MiB.
    for (final bounds in [
      (0, 1024),
      (1024, 2048),
      (3072, physical.width.toInt() - 3072),
    ]) {
      final (left, width) = bounds;
      final view = NativePageViewport.placed(
        viewport!.page,
        viewport!.viewport,
        zoom: viewport!.zoom,
        origin: viewport!.origin - Offset(left / dpr, 0),
      );
      final pixels = oracle!.pixels(
        snapshot: snapshot,
        stamp: stamp,
        viewport: view,
        dpr: dpr,
        width: width,
        height: physical.height.toInt(),
      );
      final cpu = oracle!.lastGeometry!;
      await File('${output.path}/cpu-$index-$left.rgba')
          .writeAsBytes(pixels, flush: true);
      final reference = oracle!.pixels(
        snapshot: snapshot,
        stamp: stamp,
        viewport: NativePageViewport.placed(
          view.page,
          view.viewport * 2,
          zoom: view.zoom * 2,
          origin: view.origin * 2,
        ),
        dpr: dpr,
        width: width * 2,
        height: physical.height.toInt() * 2,
      );
      await File('${output.path}/reference-$index-$left.rgba')
          .writeAsBytes(reference, flush: true);
      strips.add({
        'left': left,
        'width': width,
        'cpu': cpu,
        'reference': oracle!.lastGeometry!,
      });
    }
    require(
      await metadata!.inner.snapshotCheck(snapshot, stamp) &&
          controller!.canDisplay,
      'full source stamp changed during independent CPU replay',
    );
    final box = textureKey.currentContext!.findRenderObject()! as RenderBox;
    final origin = box.localToGlobal(Offset.zero) * dpr;
    require(
      (origin.dx - origin.dx.round()).abs() < .0001 &&
          (origin.dy - origin.dy.round()).abs() < .0001,
      'integer compositor capture origin',
    );
    await write('stage-$index.json', {
      'protocol': 1,
      'nonce': nonce,
      'pid': pid,
      'index': index,
      'width': physical.width.toInt(),
      'height': physical.height.toInt(),
      'physicalOffset': [origin.dx.round(), origin.dy.round()],
      'deviceScale': dpr,
      'stamp': stamp.toJson(),
      'snapshotIds': ids.toList(),
      'group': raw,
      'submission': submission,
      'tileRects': [
        for (final rect in controller!.physicalTiles)
          {
            'left': rect.left.toInt(),
            'top': rect.top.toInt(),
            'width': rect.width.toInt(),
            'height': rect.height.toInt(),
          },
      ],
      'strips': strips,
    });
    final ack = File('${output.path}/ack-$index.json');
    await until(
      () => ack.exists(),
      'external compositor capture acknowledgment',
    );
    final value = jsonDecode(await ack.readAsString()) as Map;
    require(
      value['nonce'] == nonce &&
          value['index'] == index &&
          value['protocol'] == 1,
      'foreign capture acknowledgment',
    );
  }

  Future<void> run() async {
    var succeeded = false;
    try {
      final configFile = File(
        Platform.environment['RHWP_CONSUMER_PROBE_CONFIG']!,
      );
      require(
        await configFile.length() <= 64 * 1024,
        'bounded probe configuration',
      );
      final config = jsonDecode(await configFile.readAsString()) as Map;
      require(config['protocol'] == 1, 'probe protocol');
      nonce = config['nonce']! as String;
      output = Directory(config['output']! as String);
      final native = await File(config['native']! as String)
          .resolveSymbolicLinks();
      final owner = await NativeOwnerTransport.connect(
        dispatcherLibrary: config['dispatcher']! as String,
        nativeLibrary: native,
      );
      document = await NativeDocument.connect(owner);
      final fixture = File(config['fixture']! as String);
      require(
        await fixture.length() <= maxReferenceBytes,
        'bounded input fixture',
      );
      await document!.openBytes(await fixture.readAsBytes());
      const point = {
        'section': 0,
        'paragraph': 0,
        'scalarOffset': 0,
        'cellPath': <Object>[],
      };
      await document!.request('beginEdit', {
        'selection': {'anchor': point, 'focus': point},
      });
      metadata = ObservedPresentation(
        await NativePresentation.connect(nativeLibrary: native),
      );
      require(metadata!.available, 'full-stamp metadata capability');
      oracle = DesktopConsumerOracle(native);
      controller = NativeTextureController(
        document: document!,
        presentation: metadata!,
        platform: NativeTexturePlatform.linux,
        channel: channel,
        useTileGroups: true,
        operationTimeout: const Duration(seconds: 8),
      );
      viewport = placed(await document!.pageSize(0));
      setState(() {});
      await publication(1);
      await document!.request('pasteText', {'text': '\n\n\n\n\n\n'});
      await publication(2);
      physical = const Size(4097, 496);
      viewport = placed(await document!.pageSize(0));
      setState(() {});
      await publication(3);
      stage = 'retire';
      await controller!.close();
      require(
        controller!.phase == NativeTexturePhase.closed &&
            !controller!.retirementPending,
        'both native owners retired',
      );
      final closed = await status();
      for (final member in closed['members']! as List) {
        final state = member as Map;
        require(
          state['registered'] == false && state['cleanupComplete'] == true,
          'each native registration retired',
        );
        for (final key in [
          'liveRasterSlots',
          'pendingRetireSlots',
          'orphanedRasterSlots',
          'abandonedTargets',
          'orphanedOwners',
          'snapshotReleaseFailures',
          'wholeFrameReadbacks',
        ]) {
          require(state[key] == 0, 'native tile teardown $key');
        }
      }
      await document!.dispose();
      final stats = oracle!.call({'op': 'stats'})['stats']! as Map;
      for (final key in [
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
        require(stats[key] == 0, 'Native owner teardown $key');
      }
      require(
        NativeTextureController.retainedOwnedSnapshots == 0 &&
            metadata!.releases == 0,
        'no leaked or twice-released caller snapshot',
      );
      await write('result.json', {
        'protocol': 1,
        'nonce': nonce,
        'ok': true,
        'publications': 3,
        'closed': closed,
        'stats': stats,
        'checks': metadata!.checks,
        'captures': metadata!.snapshots.length,
        'callerReleases': metadata!.releases,
        'consumerGpuCompletionVerified': false,
        'hardwareAccelerationVerified': false,
      });
      succeeded = true;
    } catch (error, stack) {
      stderr.writeln('RHWP_TILE_GROUP_PROBE $stage: $error\n$stack');
    } finally {
      watchdog.cancel();
      if (!succeeded) {
        try {
          await controller?.close();
        } catch (_) {
          /* Preserve original diagnostic. */
        }
        try {
          await document?.dispose();
        } catch (_) {
          /* Preserve original diagnostic. */
        }
      }
      exit(succeeded ? 0 : 1);
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    body: controller == null || viewport == null
        ? const SizedBox()
        : Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: viewport!.viewport.width,
              height: viewport!.viewport.height,
              child: NativeTextureView(
                key: textureKey,
                controller: controller!,
                viewport: viewport!,
                cpuFallback: (_) => const ColoredBox(color: Color(0xff334455)),
              ),
            ),
          ),
  );
}
