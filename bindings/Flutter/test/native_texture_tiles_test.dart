// 합성 계약: 두 출력 target과 한 문서 owner의 자원·표시 관계를 검사한다.
// 2026-11-10: mock channel/widget checks; no OS GPU or Hancom pixel claim.
import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'native_texture_multipage_test.dart' show MultiTransport, opened, until;
import 'support/fake_gpu_presentation.dart';

class WideTransport extends MultiTransport {
  WideTransport() {
    pages = 2;
  }
  static const page = NativePageSize(5000, 600);
  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) async {
    if ((request['command'] as Map?)?['op'] == 'pageSize') {
      commands.add('pageSize');
      final id = ++serial;
      completions.add({
        'ok': true,
        'requestId': id,
        'kind': 'pageSize',
        'size': {'width': page.width, 'height': page.height},
      });
      return {'ok': true, 'requestId': id};
    }
    return super.hostRequest(request, data: data);
  }
}

class TilePlugin extends FakeTexturePlugin {
  TilePlugin(super.platform);
  final groups = <Map<String, Object?>>[];
  bool secondPublished = false, holdSecondClose = false, supported = true;
  List<int>? submitIds;
  Map<String, Object?>? secondStamp;
  Map<String, Object?> member(int id, {bool creating = false}) => {
    ...status(),
    'textureId': id,
    'hasFrame':
        publish &&
        (id == 1 || secondPublished) &&
        acceptedStamp != null &&
        !closed,
    'renderedStamp': publish && (id == 1 || secondPublished)
        ? (id == 2 ? secondStamp ?? acceptedStamp : acceptedStamp)
        : null,
    if (platform == NativeTexturePlatform.windows) ...{
      'closeAcknowledgementRequired': true,
      'limits': {
        ...status()['limits']! as Map,
        'closeAcknowledgementRequired': true,
      },
    },
    if (platform == NativeTexturePlatform.macosMetalView && creating) ...{
      'ready': false,
      'presentationKind': 'nativeView',
      'viewType': 'rhwp_native/metal_view',
    },
    if (closed && id == 2 && holdSecondClose) ...{
      'ownerFinished': false,
      'cleanupComplete': false,
      'registered': true,
      'unregistered': false,
      'nativeViewLayerDetached': false,
    },
  };
  Map<String, Object?> group({
    bool creating = false,
    bool acknowledged = false,
  }) => {
    'groupId': 1,
    'deviceGeneration': 1,
    'acceptedSnapshots': acceptedIds,
    'members': [
      for (final id in [1, 2])
        {
          ...member(id, creating: creating),
          if (acknowledged) 'acknowledged': !holdSecondClose,
        },
    ],
  };
  @override
  Future<Object?> handle(MethodCall call) async {
    final args = Map<String, Object?>.from(call.arguments as Map? ?? const {});
    calls.add(call.method);
    switch (call.method) {
      case 'capabilities':
        return {
          'tileGroups': supported,
          'maxTileGroupTiles': 2,
          'maxTileGroupSnapshots': 4,
          'sharedSnapshotLeases': true,
        };
      case 'createGroup':
        await createGate?.future;
        return group(creating: true);
      case 'statusGroup':
        await statusGate?.future;
        return group();
      case 'submitGroup':
        groups.add(args);
        if (submitError != null) throw PlatformException(code: submitError!);
        if (acceptedBeforeReply) {
          accept({
            ...args,
            'placements': (args['tiles']! as List).first['placements'],
          });
        }
        await submitGate?.future;
        if (!acceptedBeforeReply) {
          accept({
            ...args,
            'placements': (args['tiles']! as List).first['placements'],
          });
        }
        return {
          'accepted': true,
          'acceptedStamp': acceptedStamp,
          'acceptedSnapshots': submitIds ?? acceptedIds,
        };
      case 'closeGroup':
        await super.handle(const MethodCall('close', {}));
        return group();
      case 'retireGroupAck':
        await super.handle(const MethodCall('retireAck', {'textureId': 1}));
        return group(acknowledged: true);
      case 'setCoveredGroup':
        final value = Map<String, Object?>.from(
          await super.handle(
            MethodCall('setCovered', {...args, 'textureId': 1}),
          ) as Map,
        );
        return {
          'groupId': 1,
          'deviceGeneration': 1,
          'members': [
            for (final id in [1, 2]) {...value, 'textureId': id},
          ],
        };
    }
    throw MissingPluginException();
  }
}

Future<void> finish(
  WidgetTester tester,
  NativeTextureController controller,
  NativeDocument document,
  TilePlugin plugin,
) async {
  for (final gate in [
    plugin.createGate,
    plugin.submitGate,
    plugin.statusGate,
  ]) {
    if (gate != null && !gate.isCompleted) gate.complete();
  }
  plugin.holdSecondClose = false;
  final close = controller.close();
  await tester.pump(const Duration(milliseconds: 60));
  await close;
  await tester.pumpWidget(const SizedBox.shrink());
  controller.dispose();
  await document.dispose();
  plugin.uninstall();
  TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
      .setMockMethodCallHandler(SystemChannels.platform_views, null);
}

NativeTextureController controller(
  NativeDocument document,
  FakePresentation metadata,
  TilePlugin plugin,
) => NativeTextureController(
  document: document,
  presentation: metadata,
  platform: plugin.platform,
  useTileGroups: true,
  pollInterval: const Duration(milliseconds: 10),
  operationTimeout: const Duration(milliseconds: 500),
);

NativeDocumentViewport largeLayout({
  Size size = const Size(4500, 1000),
  double dpr = 1.25,
}) => NativeDocumentViewport(
  pages: List.filled(2, WideTransport.page),
  viewport: size,
  deviceScale: dpr,
  rasterPolicy: NativeViewportRasterPolicy.gpuTiled,
);

Widget tiledView(NativeTextureController gpu, NativeDocumentViewport layout) =>
    MaterialApp(
      home: MediaQuery(
        data: MediaQueryData(devicePixelRatio: layout.deviceScale),
        child: NativeTextureView.document(
          controller: gpu,
          documentViewport: layout,
          cpuFallback: (_) => const Text('bounded CPU owner'),
        ),
      ),
    );

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test('GPU geometry policy is explicit; full CPU fallback preserves every placement at bounded DPR', () {
    expect(
      () => NativeDocumentViewport(
        pages: List.filled(2, WideTransport.page),
        viewport: const Size(4500, 1000),
        deviceScale: 1.25,
      ),
      throwsA(isA<NativeCallException>()),
    );
    final layout = largeLayout();
    expect(layout.rasterBytes, greaterThan(maxReferenceBytes));
    final fallback = layout.boundedCpuFallback();
    expect(fallback.deviceScale, lessThan(layout.deviceScale));
    expect(fallback.rasterBytes, lessThanOrEqualTo(maxReferenceBytes));
    expect(fallback.visiblePages.keys, layout.visiblePages.keys);
    for (final page in layout.visiblePages.keys) {
      expect(
        fallback.geometryFor(page).pageBounds,
        layout.geometryFor(page).pageBounds,
      );
      expect(fallback.regionFor(page), layout.regionFor(page));
    }
  });

  for (final platform in [
    NativeTexturePlatform.linux,
    NativeTexturePlatform.windows,
    NativeTexturePlatform.macosMetalView,
  ]) {
    testWidgets(
      'two physical outputs share one full-viewport capture batch on ${platform.name}',
      (tester) async {
        final transport = WideTransport();
        final document = await opened(tester, transport);
        final metadata = FakePresentation();
        final plugin = TilePlugin(platform)..install();
        final gpu = controller(document, metadata, plugin);
        final layout = platform == NativeTexturePlatform.windows
            ? largeLayout(size: const Size(4100, 800), dpr: 1)
            : largeLayout();
        final viewCreates = <int>[];
        if (platform == NativeTexturePlatform.macosMetalView) {
          plugin.ready = false;
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
              .setMockMethodCallHandler(SystemChannels.platform_views, (
                call,
              ) async {
                if (call.method == 'create') {
                  final encoded =
                      (call.arguments as Map)['params']! as Uint8List;
                  final params = const StandardMessageCodec().decodeMessage(
                    ByteData.sublistView(encoded),
                  ) as Map;
                  viewCreates.add(params['handle']! as int);
                  if (viewCreates.length == 2) plugin.ready = true;
                }
                return null;
              });
        }
        try {
          await tester.binding.setSurfaceSize(layout.viewport);
          await tester.pumpWidget(tiledView(gpu, layout));
          await until(tester, () => plugin.groups.isNotEmpty);
          expect(transport.capturedPages, [0, 1]);
          expect(plugin.groups, hasLength(1));
          expect(gpu.canDisplay, isFalse);
          expect(
            find.byKey(const ValueKey('rhwp-native-texture-cover')),
            findsOneWidget,
          );
          expect(
            find.byType(
              platform == NativeTexturePlatform.macosMetalView
                  ? AppKitView
                  : Texture,
            ),
            findsNWidgets(2),
          );
          final tiles = plugin.groups.single['tiles']! as List;
          final ids = <int>{};
          for (var index = 0; index < 2; ++index) {
            final rect = gpu.physicalTiles[index];
            final position = tester.widget<Positioned>(
              find.byKey(ValueKey('rhwp-native-tile-${index + 1}')),
            );
            expect(position.left, rect.left / layout.deviceScale);
            expect(position.top, rect.top / layout.deviceScale);
            expect(position.width, rect.width / layout.deviceScale);
            expect(position.height, rect.height / layout.deviceScale);
            final placements = tiles[index]['placements'] as List;
            for (var page = 0; page < placements.length; ++page) {
              final item = placements[page] as Map,
                  geometry = layout.visiblePages[page]!;
              ids.add(item['snapshot']! as int);
              expect(
                item['originX'],
                geometry.origin.dx * layout.deviceScale - rect.left,
              );
              expect(
                item['originY'],
                geometry.origin.dy * layout.deviceScale - rect.top,
              );
              expect(item['scale'], layout.scale * layout.deviceScale);
            }
          }
          expect(ids, {101, 102});
          expect(metadata.released, isEmpty);
          expect(NativeTextureController.retainedOwnedSnapshots, 0);
          plugin.secondPublished = true;
          await until(tester, () => gpu.canDisplay);
          expect(
            find.byKey(const ValueKey('rhwp-native-texture-cover')),
            findsNothing,
          );
          expect(metadata.checks.length, greaterThanOrEqualTo(4));
          if (platform == NativeTexturePlatform.macosMetalView) {
            expect(viewCreates, [1, 2]);
          }
          final oldStamp = Map<String, Object?>.from(plugin.acceptedStamp!);
          gpu.configureDocument(
            largeLayout(
              size: layout.viewport,
              dpr: layout.deviceScale,
            ).boundedCpuFallback(),
          );
          expect(gpu.canDisplay, isFalse);
          await until(tester, () => plugin.groups.length == 2);
          plugin.secondStamp = oldStamp;
          await tester.pump(const Duration(milliseconds: 30));
          expect(
            gpu.canDisplay,
            isFalse,
            reason: 'A late old tile cannot uncover the new group.',
          );
          plugin.secondStamp = null;
          await until(tester, () => gpu.canDisplay);
        } finally {
          await finish(tester, gpu, document, plugin);
          await tester.binding.setSurfaceSize(null);
        }
        expect(plugin.releasedByPlugin, {101, 102, 103, 104});
      },
    );
  }

  testWidgets(
    'large paged viewport falls back to complete bounded lower-resolution CPU pages',
    (tester) async {
      final transport = WideTransport();
      final document = await opened(tester, transport);
      final plugin = TilePlugin(NativeTexturePlatform.linux)
        ..secondPublished = true
        ..install();
      final gpu = controller(document, FakePresentation(), plugin);
      NativeDocumentViewport? rendered;
      try {
        await tester.binding.setSurfaceSize(const Size(4500, 1000));
        await tester.pumpWidget(
          MaterialApp(
            home: MediaQuery(
              data: const MediaQueryData(devicePixelRatio: 1.25),
              child: NativePagedDocumentView(
                document: document,
                gpuController: gpu,
                editable: false,
                onViewportReady: (value) => rendered = value,
              ),
            ),
          ),
        );
        await until(tester, () => gpu.canDisplay);
        expect(transport.regions, isEmpty);
        expect(rendered!.rasterBytes, greaterThan(maxReferenceBytes));
        plugin.statusError = 'deviceLost';
        await until(
          tester,
          () => find.text('큰 화면을 낮은 해상도로 표시합니다.').evaluate().isNotEmpty,
        );
        expect(gpu.needsCpuFallback, isTrue);
        expect(transport.regions.map((r) => r['page']), [0, 1]);
        expect(
          transport.regions.map((r) => r['viewGeneration']).toSet(),
          hasLength(1),
        );
        final expected = rendered!.boundedCpuFallback();
        int bytes = 0;
        for (final request in transport.regions) {
          final region = request['region'] as Map;
          final scale = request['scale']! as double;
          expect(scale, expected.scale * expected.deviceScale);
          final width =
              (((region['x'] as double) + (region['width'] as double)) * scale)
                  .ceil() -
              ((region['x'] as double) * scale).floor();
          final height =
              (((region['y'] as double) + (region['height'] as double)) * scale)
                  .ceil() -
              ((region['y'] as double) * scale).floor();
          expect(width, lessThanOrEqualTo(4096));
          expect(height, lessThanOrEqualTo(4096));
          bytes += width * height * 4;
        }
        expect(bytes, lessThanOrEqualTo(maxReferenceBytes));
        expect(transport.buffers, isEmpty);
        expect(transport.released, hasLength(2));
        expect(find.textContaining('문서 보기:'), findsNothing);
      } finally {
        await finish(tester, gpu, document, plugin);
        await tester.binding.setSurfaceSize(null);
      }
    },
  );

  testWidgets(
    'a rejected second target releases only caller-owned captures once',
    (tester) async {
      final transport = WideTransport();
      final document = await opened(tester, transport);
      final metadata = FakePresentation();
      final plugin = TilePlugin(NativeTexturePlatform.linux)
        ..submitError = 'invalidArgument'
        ..install();
      final gpu = controller(document, metadata, plugin);
      try {
        gpu.configureDocument(largeLayout());
        await until(tester, () => gpu.phase == NativeTexturePhase.failed);
        await until(tester, () => metadata.released.length == 2);
        expect(metadata.released.toSet(), {101, 102});
        expect(plugin.owned, isEmpty);
        expect(plugin.releasedByPlugin, isEmpty);
        expect(gpu.canDisplay, isFalse);
      } finally {
        await finish(tester, gpu, document, plugin);
      }
    },
  );

  for (final malformed in [
    [101],
    [101, 101, 102],
  ]) {
    testWidgets(
      'malformed acceptance $malformed reconciles group ownership without caller releases',
      (tester) async {
        final document = await opened(tester, WideTransport());
        final metadata = FakePresentation();
        final plugin = TilePlugin(NativeTexturePlatform.linux)
          ..submitIds = malformed
          ..install();
        final gpu = controller(document, metadata, plugin);
        try {
          gpu.configureDocument(largeLayout());
          await until(tester, () => gpu.phase == NativeTexturePhase.failed);
          expect(metadata.released, isEmpty);
          expect(gpu.canDisplay, isFalse);
          expect(NativeTextureController.retainedOwnedSnapshots, 0);
        } finally {
          await finish(tester, gpu, document, plugin);
        }
        expect(plugin.releasedByPlugin, {101, 102});
      },
    );
  }

  testWidgets(
    'both textures stay mounted under cover until the second close completes',
    (tester) async {
      final document = await opened(tester, WideTransport());
      final plugin = TilePlugin(NativeTexturePlatform.linux)
        ..secondPublished = true
        ..holdSecondClose = true
        ..install();
      final gpu = controller(document, FakePresentation(), plugin);
      try {
        await tester.pumpWidget(tiledView(gpu, largeLayout()));
        await until(tester, () => gpu.canDisplay);
        final closing = gpu.close();
        await tester.pump(const Duration(milliseconds: 40));
        expect(gpu.phase, NativeTexturePhase.closing);
        expect(find.byType(Texture), findsNWidgets(2));
        expect(
          find.byKey(const ValueKey('rhwp-native-texture-cover')),
          findsOneWidget,
        );
        plugin.holdSecondClose = false;
        await until(tester, () => gpu.phase == NativeTexturePhase.closed);
        await closing;
        expect(find.byType(Texture), findsNothing);
        expect(gpu.retirementPending, isFalse);
      } finally {
        await finish(tester, gpu, document, plugin);
      }
    },
  );

  testWidgets('late group creation after unmount closes both members once', (
    tester,
  ) async {
    final document = await opened(tester, WideTransport());
    final plugin = TilePlugin(NativeTexturePlatform.linux)
      ..createGate = Completer<void>()
      ..install();
    final gpu = controller(document, FakePresentation(), plugin);
    try {
      await tester.pumpWidget(tiledView(gpu, largeLayout()));
      await until(tester, () => plugin.calls.contains('createGroup'));
      await tester.pumpWidget(const SizedBox.shrink());
      plugin.createGate!.complete();
      await until(tester, () => gpu.phase == NativeTexturePhase.closed);
      expect(plugin.calls.where((c) => c == 'closeGroup'), hasLength(1));
      expect(plugin.groups, isEmpty);
      expect(plugin.owned, isEmpty);
      expect(gpu.retirementPending, isFalse);
    } finally {
      await finish(tester, gpu, document, plugin);
    }
  });

  testWidgets(
    'unsupported CV/plugin capabilities fall back before capture or group allocation',
    (tester) async {
      final transport = WideTransport();
      final document = await opened(tester, transport);
      final plugin = TilePlugin(NativeTexturePlatform.macos)
        ..supported = false
        ..install();
      final gpu = controller(document, FakePresentation(), plugin);
      try {
        gpu.configureDocument(largeLayout());
        await until(tester, () => gpu.needsCpuFallback);
        expect(transport.capturedPages, isEmpty);
        expect(plugin.calls, ['capabilities']);
        await gpu.close();
        expect(gpu.retirementPending, isFalse);
      } finally {
        await finish(tester, gpu, document, plugin);
      }
    },
  );
}
