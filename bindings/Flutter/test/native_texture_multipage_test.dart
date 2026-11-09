// 합성 계약: 같은 stamp의 페이지 batch·소유권·공유 viewport·CPU fallback 관계 검사.
// 주석 갱신: 2026-11-10T02:03:57.269385+09:00. 신설 파일이므로 직전 파일 commit 없음.
// 문서 조판 좌표·한컴 시각 일치의 고정 기대값이 아니다. 기존 P105 geometry 계약을 사용한다.

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'native_multipage_view_test.dart' show PagedTransport;
import 'native_editor_view_test.dart' show ViewTransport;
import 'support/fake_gpu_presentation.dart';
import 'support/fake_transport.dart';

mixin Captures on FakeTransport implements NativePresentationOwnerIdentity {
  @override
  String get nativeLibraryPath => '/canonical/librhwp_native_ffi.so';
  int nextSnapshot = 100, faultAt = 0;
  bool autoSnapshots = true;
  final capturedPages = <int>[];
  final waiting = <Map<String, Object?>>[];
  void enableGpu() => capabilities.addAll({
    'ownerPresentationCleanup': true,
    'presentation': {
      'workerRenderSnapshots': true,
      'crossThreadHandles': true,
      'fullStampChecks': true,
    },
  });
  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) async {
    if (request['op'] == 'presentationConfigure') return {'ok': true};
    final command = request['command'];
    if (command is Map && command['op'] == 'exportRenderSnapshot') {
      capturedPages.add(command['page']! as int);
      if (capturedPages.length == faultAt) {
        throw const NativeCallException('resourceError', 'Page replay failed.');
      }
      final id = ++serial;
      final result = <String, Object?>{
        'ok': true,
        'requestId': id,
        'kind': 'renderSnapshot',
        'snapshot': ++nextSnapshot,
        'info': {...info, 'current': true, 'profile': 'screen'},
      };
      if (autoSnapshots) {
        completions.add(result);
      } else {
        waiting.add(result);
      }
      return {'ok': true, 'requestId': id};
    }
    return super.hostRequest(request, data: data);
  }

  void completeCapture() => completions.add(waiting.removeAt(0));
}

class MultiTransport extends PagedTransport with Captures {
  MultiTransport() {
    enableGpu();
  }
}

class EditingTransport extends ViewTransport with Captures {
  EditingTransport() {
    enableGpu();
  }
}

class PartialReplyPlugin extends FakeTexturePlugin {
  PartialReplyPlugin(super.platform);
  @override
  Future<Object?> handle(MethodCall call) async {
    final reply = await super.handle(call);
    if (call.method == 'submit') {
      return {
        ...reply! as Map,
        'acceptedSnapshots': [acceptedIds.first],
      };
    }
    return reply;
  }
}

class HeldReleasePresentation extends FakePresentation {
  final releaseGate = Completer<void>();
  final releaseAttempts = <int>[];
  @override
  Future<void> releaseSnapshot(int snapshot) async {
    releaseAttempts.add(snapshot);
    await releaseGate.future;
    await super.releaseSnapshot(snapshot);
  }
}

Future<void> until(WidgetTester tester, bool Function() ready) async {
  for (var index = 0; index < 180 && !ready(); ++index) {
    await tester.pump(const Duration(milliseconds: 10));
    await tester.runAsync(() => Future<void>.delayed(Duration.zero));
  }
  expect(ready(), isTrue);
}

Future<NativeDocument> opened(
  WidgetTester tester,
  FakeTransport transport,
) async {
  final owner = await NativeDocument.connect(transport);
  final opening = owner.newDocument();
  await until(tester, () => owner.info != null);
  await opening;
  return owner;
}

NativeDocumentViewport layout({
  double scroll = 0,
  int pages = 3,
  double dpr = 1.5,
}) => NativeDocumentViewport(
  pages: List.filled(pages, const NativePageSize(300, 200)),
  viewport: const Size(300, 260),
  scrollOffset: scroll,
  deviceScale: dpr,
);
NativeTextureController gpu(
  NativeDocument owner,
  FakePresentation metadata,
  NativeTexturePlatform platform,
) => NativeTextureController(
  document: owner,
  presentation: metadata,
  platform: platform,
  pollInterval: const Duration(milliseconds: 10),
  operationTimeout: const Duration(milliseconds: 250),
);
Future<void> cleanup(
  WidgetTester tester,
  NativeTextureController controller,
  NativeDocument owner,
  FakeTexturePlugin plugin,
) async {
  final closing = controller.close();
  await tester.pump(const Duration(milliseconds: 30));
  await closing;
  await tester.pumpWidget(const SizedBox.shrink());
  controller.dispose();
  await owner.dispose();
  plugin.uninstall();
}

void main() {
  for (final platform in NativeTexturePlatform.values) {
    testWidgets('one same-stamp two-page batch on ${platform.name}', (
      tester,
    ) async {
      final transport = MultiTransport();
      final owner = await opened(tester, transport);
      final metadata = FakePresentation();
      final plugin = FakeTexturePlugin(platform)..install();
      final controller = gpu(owner, metadata, platform);
      final before = NativeTextureController.retainedOwnedSnapshots;
      try {
        final viewport = layout();
        controller.configureDocument(viewport);
        await until(tester, () => controller.canDisplay);
        expect(transport.capturedPages, [0, 1]);
        expect(plugin.submissions, hasLength(1));
        final submission = plugin.submissions.single;
        expect(
          submission['width'],
          (viewport.viewport.width * viewport.deviceScale).ceil(),
        );
        expect(
          submission['height'],
          (viewport.viewport.height * viewport.deviceScale).ceil(),
        );
        final placements = submission['placements']! as List;
        expect(placements, hasLength(viewport.visiblePages.length));
        for (var index = 0; index < placements.length; ++index) {
          final page = viewport.visiblePages.entries.elementAt(index);
          final item = placements[index] as Map;
          expect(item['snapshot'], 101 + index);
          expect(item['originX'], page.value.origin.dx * viewport.deviceScale);
          expect(item['originY'], page.value.origin.dy * viewport.deviceScale);
          expect(item['scale'], page.value.zoom * viewport.deviceScale);
        }
        expect(metadata.checks.length, greaterThanOrEqualTo(4));
        for (final stamp in metadata.checks) {
          expect(stamp.matches(submission['stamp']), isTrue);
        }
        expect(metadata.released, isEmpty);
        expect(NativeTextureController.retainedOwnedSnapshots, before);
      } finally {
        await cleanup(tester, controller, owner, plugin);
      }
      expect(plugin.releasedByPlugin, {101, 102});
    });
  }
  testWidgets('second page capture failure releases the whole caller batch', (
    tester,
  ) async {
    final transport = MultiTransport()..faultAt = 2;
    final owner = await opened(tester, transport);
    final metadata = FakePresentation();
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final controller = gpu(owner, metadata, plugin.platform);
    final before = NativeTextureController.retainedOwnedSnapshots;
    try {
      controller.configureDocument(layout());
      await until(tester, () => controller.phase == NativeTexturePhase.failed);
      expect(
        controller.needsCpuFallback,
        isFalse,
        reason: 'A document resource error stays visible.',
      );
      expect(plugin.submissions, isEmpty);
      await until(
        tester,
        () => NativeTextureController.retainedOwnedSnapshots == before,
      );
      expect(metadata.released, [101]);
    } finally {
      await cleanup(tester, controller, owner, plugin);
    }
  });
  testWidgets(
    'navigation during capture discards old pages before newest batch',
    (tester) async {
      final transport = MultiTransport()..autoSnapshots = false;
      final owner = await opened(tester, transport);
      final metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final controller = gpu(owner, metadata, plugin.platform);
      try {
        controller.configureDocument(layout());
        await until(tester, () => transport.waiting.isNotEmpty);
        transport.completeCapture();
        await until(tester, () => transport.waiting.isNotEmpty);
        controller.configureDocument(layout(scroll: 100));
        transport.completeCapture();
        await until(tester, () => transport.waiting.isNotEmpty);
        transport.completeCapture();
        await until(tester, () => transport.waiting.isNotEmpty);
        transport.completeCapture();
        await until(tester, () => controller.canDisplay);
        expect(plugin.submissions, hasLength(1));
        expect(
          (plugin.submissions.single['placements'] as List).map(
            (p) => (p as Map)['snapshot'],
          ),
          [103, 104],
        );
        expect(metadata.released.toSet(), {101, 102});
        expect(metadata.released, hasLength(2));
      } finally {
        await cleanup(tester, controller, owner, plugin);
      }
    },
  );
  testWidgets('suspended layout stays covered without polling or new capture', (
    tester,
  ) async {
    final transport = MultiTransport();
    final owner = await opened(tester, transport);
    final metadata = FakePresentation();
    final plugin = FakeTexturePlugin(NativeTexturePlatform.macosMetalView)
      ..install();
    final controller = gpu(owner, metadata, plugin.platform);
    try {
      controller.configureDocument(layout());
      await until(tester, () => controller.canDisplay);
      controller.suspendViewport();
      expect(controller.canDisplay, isFalse);
      await tester.pump(const Duration(milliseconds: 40));
      final calls = plugin.calls.length;
      await tester.pump(const Duration(milliseconds: 300));
      expect(plugin.calls.length, calls);
      expect(transport.capturedPages, [0, 1]);
      expect(controller.phase, NativeTexturePhase.pending);
      expect(plugin.covered, isTrue);
      controller.configureDocument(layout(scroll: 100));
      await until(tester, () => controller.canDisplay);
      expect(plugin.submissions, hasLength(2));
    } finally {
      await cleanup(tester, controller, owner, plugin);
    }
  });
  testWidgets(
    'four pending pages share the isolate cap with another controller',
    (tester) async {
      final held = MultiTransport()..autoSnapshots = false;
      final first = await opened(tester, held);
      final otherTransport = MultiTransport();
      final second = await opened(tester, otherTransport);
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final otherPlugin = FakeTexturePlugin(NativeTexturePlatform.windows)
        ..install();
      final firstMetadata = FakePresentation(),
          secondMetadata = FakePresentation();
      final one = gpu(first, firstMetadata, plugin.platform);
      final two = gpu(second, secondMetadata, otherPlugin.platform);
      final before = NativeTextureController.retainedOwnedSnapshots;
      try {
        one.configureDocument(
          NativeDocumentViewport(
            pages: List.filled(4, const NativePageSize(300, 200)),
            viewport: const Size(300, 848),
          ),
        );
        await until(tester, () => held.waiting.isNotEmpty);
        expect(NativeTextureController.retainedOwnedSnapshots, before + 4);
        two.configureDocument(layout());
        await until(tester, () => two.needsCpuFallback);
        expect(otherTransport.capturedPages, isEmpty);
        expect(secondMetadata.released, isEmpty);
        for (var page = 0; page < 4; ++page) {
          await until(tester, () => held.waiting.isNotEmpty);
          held.completeCapture();
        }
        await until(tester, () => one.canDisplay);
        expect(held.capturedPages, [0, 1, 2, 3]);
        expect(plugin.submissions, hasLength(1));
        expect(plugin.acceptedIds, [101, 102, 103, 104]);
        expect(NativeTextureController.retainedOwnedSnapshots, before);
      } finally {
        await cleanup(tester, one, first, plugin);
        await cleanup(tester, two, second, otherPlugin);
      }
    },
  );
  testWidgets(
    'superseded four-page credit waits for confirmed caller release',
    (tester) async {
      final transport = MultiTransport();
      final owner = await opened(tester, transport);
      final metadata = HeldReleasePresentation()..checkGate = Completer<bool>();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final controller = gpu(owner, metadata, plugin.platform);
      NativeDocumentViewport four(double scroll) => NativeDocumentViewport(
        pages: List.filled(4, const NativePageSize(300, 200)),
        viewport: const Size(300, 800),
        scrollOffset: scroll,
      );
      try {
        controller.configureDocument(four(0));
        await until(tester, () => metadata.checks.isNotEmpty);
        controller.configureDocument(four(48));
        metadata.checkGate!.complete(true);
        await until(tester, () => metadata.releaseAttempts.isNotEmpty);
        await tester.pump(const Duration(milliseconds: 50));
        expect(controller.needsCpuFallback, isFalse);
        expect(transport.capturedPages, [
          0,
        ], reason: 'Do not race old caller-owned credits.');
        metadata.releaseGate.complete();
        await until(tester, () => controller.canDisplay);
        expect(transport.capturedPages, [0, 0, 1, 2, 3]);
        expect(metadata.released, [101]);
        expect(plugin.acceptedIds, [102, 103, 104, 105]);
      } finally {
        if (!metadata.checkGate!.isCompleted) {
          metadata.checkGate!.complete(true);
        }
        if (!metadata.releaseGate.isCompleted) metadata.releaseGate.complete();
        await cleanup(tester, controller, owner, plugin);
      }
    },
  );
  testWidgets(
    'a late second capture is released after the bounded batch timeout',
    (tester) async {
      final transport = MultiTransport()..autoSnapshots = false;
      final owner = await opened(tester, transport);
      final metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final controller = gpu(owner, metadata, plugin.platform);
      final before = NativeTextureController.retainedOwnedSnapshots;
      try {
        controller.configureDocument(layout());
        await until(tester, () => transport.waiting.isNotEmpty);
        transport.completeCapture();
        await until(tester, () => transport.waiting.isNotEmpty);
        await until(tester, () => controller.needsCpuFallback);
        expect(plugin.submissions, isEmpty);
        expect(metadata.released, [101]);
        expect(NativeTextureController.retainedOwnedSnapshots, before + 1);
        transport.completeCapture();
        await until(
          tester,
          () => NativeTextureController.retainedOwnedSnapshots == before,
        );
        expect(metadata.released.toSet(), {101, 102});
        expect(metadata.released, hasLength(2));
      } finally {
        await cleanup(tester, controller, owner, plugin);
      }
    },
  );
  testWidgets(
    'partial accepted receipt never uncovers or caller-releases a page',
    (tester) async {
      final owner = await opened(tester, MultiTransport());
      final metadata = FakePresentation();
      final plugin = PartialReplyPlugin(NativeTexturePlatform.linux)..install();
      final controller = gpu(owner, metadata, plugin.platform);
      final before = NativeTextureController.retainedOwnedSnapshots;
      try {
        controller.configureDocument(layout());
        await until(tester, () => controller.needsCpuFallback);
        expect(controller.canDisplay, isFalse);
        expect(metadata.released, isEmpty);
        expect(
          NativeTextureController.retainedOwnedSnapshots,
          before,
          reason: 'The complete subsequent status reconciles the second page.',
        );
      } finally {
        await cleanup(tester, controller, owner, plugin);
      }
      expect(plugin.releasedByPlugin, {101, 102});
    },
  );
  testWidgets('a published receipt losing a page hides the complete viewport', (
    tester,
  ) async {
    final owner = await opened(tester, MultiTransport());
    final plugin = FakeTexturePlugin(NativeTexturePlatform.windows)..install();
    final controller = gpu(owner, FakePresentation(), plugin.platform);
    try {
      controller.configureDocument(layout());
      await until(tester, () => controller.canDisplay);
      plugin.acceptedIds = [101];
      await until(tester, () => controller.needsCpuFallback);
      expect(controller.canDisplay, isFalse);
    } finally {
      await cleanup(tester, controller, owner, plugin);
    }
  });
  testWidgets(
    'paged GPU view reuses one layout and invokes CPU only on fallback',
    (tester) async {
      final transport = MultiTransport();
      final owner = await opened(tester, transport);
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final controller = gpu(owner, FakePresentation(), plugin.platform);
      final pages = NativePagedViewportController();
      try {
        await tester.pumpWidget(
          MaterialApp(
            home: Center(
              child: SizedBox(
                width: 300,
                height: 260,
                child: NativePagedDocumentView(
                  document: owner,
                  controller: pages,
                  gpuController: controller,
                  editable: false,
                ),
              ),
            ),
          ),
        );
        await until(tester, () => controller.canDisplay);
        expect(pages.pageCount, 3);
        expect(transport.regions, isEmpty);
        expect(find.byType(NativeDocumentView), findsNothing);
        pages.scrollTo(100);
        expect(controller.canDisplay, isFalse);
        await until(
          tester,
          () => plugin.submissions.length == 2 && controller.canDisplay,
        );
        expect(transport.regions, isEmpty);
        plugin.statusError = 'deviceLost';
        await until(
          tester,
          () => controller.needsCpuFallback && transport.regions.length == 2,
        );
        expect(transport.regions.map((r) => r['page']), [0, 1]);
        expect(
          transport.regions.map((r) => r['viewGeneration']).toSet(),
          hasLength(1),
        );
      } finally {
        await cleanup(tester, controller, owner, plugin);
        pages.dispose();
      }
    },
  );
  testWidgets('paged GPU path keeps the existing editable input owner', (
    tester,
  ) async {
    final transport = EditingTransport();
    final owner = await opened(tester, transport);
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final controller = gpu(owner, FakePresentation(), plugin.platform);
    NativeEditorController? editor;
    try {
      await tester.pumpWidget(
        MaterialApp(
          home: Center(
            child: SizedBox(
              width: 24,
              height: 24,
              child: NativePagedDocumentView(
                document: owner,
                gpuController: controller,
                onControllerChanged: (value) => editor = value,
              ),
            ),
          ),
        ),
      );
      await until(tester, () => controller.canDisplay && editor?.ready == true);
      expect(find.byType(NativeEditorView), findsOneWidget);
      expect(find.byType(NativeDocumentView), findsNothing);
      expect(
        transport.commands.where(
          (c) => c == 'renderRegion' || c == 'renderTiledRegion',
        ),
        isEmpty,
      );
    } finally {
      await cleanup(tester, controller, owner, plugin);
    }
    expect(editor, isNull);
  });
}
