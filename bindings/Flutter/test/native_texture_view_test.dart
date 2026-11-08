import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_document.dart';
import 'package:rhwp_native_reference/src/native_ffi.dart';
import 'package:rhwp_native_reference/src/native_texture_view.dart';
import 'package:rhwp_native_reference/src/native_viewport.dart';

import 'support/fake_gpu_presentation.dart';

Future<void> pumpUntil(WidgetTester tester, bool Function() ready) async {
  for (var index = 0; index < 100 && !ready(); index++) {
    await tester.pump(const Duration(milliseconds: 10));
  }
  expect(ready(), isTrue);
}

Future<NativeDocument> opened(
  WidgetTester tester,
  GpuTransport transport,
) async {
  final document = await NativeDocument.connect(transport);
  final opening = document.newDocument();
  await pumpUntil(tester, () => document.info != null);
  await opening;
  return document;
}

Widget view(NativeTextureController controller, {double width = 300}) =>
    MaterialApp(
      home: MediaQuery(
        data: const MediaQueryData(devicePixelRatio: 1.5),
        child: SizedBox(
          width: width,
          height: 240,
          child: NativeTextureView(
            controller: controller,
            viewport: NativePageViewport.fit(
              const NativePageSize(600, 800),
              Size(width, 240),
            ),
            cpuFallback: (_) => const Text('CPU reference'),
          ),
        ),
      ),
    );

class RetiringTexturePlugin extends FakeTexturePlugin {
  RetiringTexturePlugin(super.platform);
  bool closeRequested = false, ownerFinished = false, registered = true;

  @override
  Map<String, Object?> status() => {
    ...super.status(),
    'closing': closeRequested,
    'closed': closeRequested,
    'cleanupComplete': ownerFinished,
    'ownerFinished': ownerFinished,
    'registered': registered,
    'unregistered': !registered,
    'textureUnregisteredNotification': !registered,
    // Registration/producer completion never returns these immutable credits.
    'limits': {'outstandingFrames': 4, 'consumerRetirement': false},
  };

  @override
  Future<Object?> handle(MethodCall call) async {
    if (call.method == 'close') {
      calls.add(call.method);
      closeRequested = true;
      return status();
    }
    return super.handle(call);
  }

  void finishOwner() {
    ownerFinished = true;
    releasedByPlugin.addAll(owned);
    owned.clear();
  }
}

Future<void> cleanUpTexture(
  WidgetTester tester,
  NativeTextureController controller,
  NativeDocument document,
  RetiringTexturePlugin plugin,
) async {
  plugin.finishOwner();
  plugin.registered = false;
  final closing = controller.close();
  await tester.pump(const Duration(milliseconds: 20));
  await closing;
  await tester.pumpWidget(const SizedBox.shrink());
  controller.dispose();
  await document.dispose();
  plugin.uninstall();
}

void main() {
  testWidgets(
    'close keeps Linux Texture painted until cleanup and unregister both finish',
    (tester) async {
      final metadata = FakePresentation();
      final plugin = RetiringTexturePlugin(NativeTexturePlatform.linux)
        ..install();
      final document = await opened(tester, GpuTransport());
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      try {
        await tester.pumpWidget(view(controller));
        await pumpUntil(tester, () => controller.canDisplay);
        var closed = false;
        final request = controller.close();
        expect(identical(controller.close(), request), isTrue);
        final closing = request.then((_) => closed = true);
        await tester.pump(const Duration(milliseconds: 20));
        expect(controller.phase.name, 'closing');
        expect(closed, isFalse);
        expect(find.byType(Texture), findsOneWidget);
        final cover = tester.widget<ColoredBox>(
          find
              .descendant(
                of: find.byKey(const ValueKey('rhwp-native-texture-cover')),
                matching: find.byType(ColoredBox),
              )
              .first,
        );
        expect(cover.color.a, 1.0);
        plugin.finishOwner();
        await tester.pump(const Duration(milliseconds: 20));
        expect(
          closed,
          isFalse,
          reason: 'Registered Texture still needs its callback.',
        );
        expect(find.byType(Texture), findsOneWidget);
        plugin.registered = false;
        await pumpUntil(tester, () => closed);
        await closing;
        await tester.pump();
        expect(controller.phase.name, 'closed');
        expect(controller.retirementPending, isFalse);
        expect(find.byType(Texture), findsNothing);
        expect(metadata.released, isEmpty);
        expect(plugin.releasedByPlugin, {101});
        expect(plugin.calls.where((call) => call == 'close').length, 1);
        expect(plugin.status()['limits'], {
          'outstandingFrames': 4,
          'consumerRetirement': false,
        });
      } finally {
        await cleanUpTexture(tester, controller, document, plugin);
      }
    },
  );
  testWidgets(
    'retirement timeout is bounded without refunding plugin ownership',
    (tester) async {
      final metadata = FakePresentation();
      final plugin = RetiringTexturePlugin(NativeTexturePlatform.linux)
        ..install();
      final document = await opened(tester, GpuTransport());
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
        operationTimeout: const Duration(milliseconds: 100),
      );
      try {
        await tester.pumpWidget(view(controller));
        await pumpUntil(tester, () => controller.canDisplay);
        var closed = false;
        final request = controller.close();
        final closing = request.then((_) => closed = true);
        await pumpUntil(tester, () => closed);
        await closing;
        expect(controller.phase.name, 'retirePending');
        expect(controller.retirementPending, isTrue);
        expect(
          controller.retirementDiagnostic,
          isA<NativeCallException>().having(
            (error) => error.code,
            'code',
            'retirePending',
          ),
        );
        expect(find.byType(Texture), findsOneWidget);
        expect(metadata.released, isEmpty);
        expect(plugin.owned, {101});
        final calls = plugin.calls.length;
        expect(identical(controller.close(), request), isTrue);
        await tester.pump(const Duration(seconds: 1));
        expect(
          plugin.calls.length,
          calls,
          reason: 'No unbounded status/retry loop.',
        );
        expect(plugin.calls.where((call) => call == 'close').length, 1);
      } finally {
        await cleanUpTexture(tester, controller, document, plugin);
      }
    },
  );
  testWidgets('unmount before retirement reports bounded pending cleanup', (
    tester,
  ) async {
    final plugin = RetiringTexturePlugin(NativeTexturePlatform.linux)
      ..install();
    final document = await opened(tester, GpuTransport());
    final controller = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
      operationTimeout: const Duration(milliseconds: 100),
    );
    try {
      await tester.pumpWidget(view(controller));
      await pumpUntil(tester, () => controller.canDisplay);
      await tester.pumpWidget(const SizedBox.shrink());
      var closed = false;
      final closing = controller.close().then((_) => closed = true);
      await pumpUntil(tester, () => closed);
      await closing;
      expect(controller.phase.name, 'retirePending');
      expect(controller.retirementPending, isTrue);
      expect(
        controller.retirementDiagnostic,
        isA<NativeCallException>().having(
          (error) => error.code,
          'code',
          'textureUnmounted',
        ),
      );
      expect(plugin.owned, {101});
    } finally {
      await cleanUpTexture(tester, controller, document, plugin);
    }
  });
  for (final platform in [
    NativeTexturePlatform.macos,
    NativeTexturePlatform.windows,
  ]) {
    testWidgets(
      '${platform.name} close needs owner completion and retains immutable frames',
      (tester) async {
        final plugin = RetiringTexturePlugin(platform)..install();
        final document = await opened(tester, GpuTransport());
        final controller = NativeTextureController(
          document: document,
          presentation: FakePresentation(),
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
        );
        try {
          await tester.pumpWidget(view(controller));
          await pumpUntil(tester, () => controller.canDisplay);
          var closed = false;
          final closing = controller.close().then((_) => closed = true);
          plugin.registered = false;
          await tester.pump(const Duration(milliseconds: 20));
          await tester.pump();
          expect(closed, isFalse);
          expect(controller.phase.name, 'closing');
          expect(find.byType(Texture), findsNothing);
          plugin.finishOwner();
          await pumpUntil(tester, () => closed);
          await closing;
          expect(controller.retirementPending, isFalse);
          expect(controller.phase.name, 'closed');
          expect(plugin.status()['limits'], {
            'outstandingFrames': 4,
            'consumerRetirement': false,
          });
        } finally {
          await cleanUpTexture(tester, controller, document, plugin);
        }
      },
    );
  }
  testWidgets(
    'pending Texture stays painted under an opaque CPU cover until publication',
    (tester) async {
      final transport = GpuTransport(), metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)
        ..publish = false;
      plugin.install();
      final document = await opened(tester, transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      await tester.pumpWidget(view(controller));
      await pumpUntil(tester, () => plugin.submissions.isNotEmpty);
      expect(find.byType(Texture), findsOneWidget);
      expect(find.text('CPU reference'), findsOneWidget);
      expect(
        find.byKey(const ValueKey('rhwp-native-texture-cover')),
        findsOneWidget,
      );
      plugin.publish = true;
      await pumpUntil(tester, () => controller.canDisplay);
      await tester.pump();
      expect(
        find.byKey(const ValueKey('rhwp-native-texture-cover')),
        findsNothing,
      );
      expect(find.byType(Texture), findsOneWidget);
      await tester.pumpWidget(const SizedBox.shrink());
      await controller.close();
      controller.dispose();
      await document.dispose();
      plugin.uninstall();
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets(
    'resize covers immediately and never exposes the previous frame as the new view',
    (tester) async {
      final transport = GpuTransport(), metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final document = await opened(tester, transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      await tester.pumpWidget(view(controller));
      await pumpUntil(tester, () => controller.canDisplay);
      plugin.publish = false;
      await tester.pumpWidget(view(controller, width: 320));
      expect(controller.canDisplay, isFalse);
      expect(
        find.byKey(const ValueKey('rhwp-native-texture-cover')),
        findsOneWidget,
      );
      await pumpUntil(tester, () => plugin.submissions.length == 2);
      expect(
        (plugin.submissions.last['stamp']! as Map)['viewGeneration'],
        document.presentationGeneration,
      );
      await tester.pumpWidget(const SizedBox.shrink());
      await controller.close();
      controller.dispose();
      await document.dispose();
      plugin.uninstall();
    },
  );
  testWidgets(
    'unmount during a submit preserves late accepted ownership and closes its texture',
    (tester) async {
      final transport = GpuTransport(), metadata = FakePresentation();
      final gate = Completer<void>();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.macos)
        ..acceptedBeforeReply = true
        ..submitGate = gate;
      plugin.install();
      final document = await opened(tester, transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.macos,
        pollInterval: const Duration(milliseconds: 10),
      );
      await tester.pumpWidget(view(controller));
      await pumpUntil(tester, () => plugin.submissions.isNotEmpty);
      await tester.pumpWidget(const SizedBox.shrink());
      gate.complete();
      await tester.pump(const Duration(milliseconds: 100));
      await controller.close();
      expect(metadata.released, isEmpty);
      expect(plugin.releasedByPlugin, {101});
      expect(NativeTextureController.retainedOwnedSnapshots, 0);
      controller.dispose();
      await document.dispose();
      plugin.uninstall();
    },
  );
  testWidgets(
    'missing producer publication uses a bounded deadline then CPU fallback',
    (tester) async {
      final transport = GpuTransport(), metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.windows)
        ..publish = false;
      plugin.install();
      final document = await opened(tester, transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.windows,
        pollInterval: const Duration(milliseconds: 10),
        operationTimeout: const Duration(milliseconds: 100),
      );
      await tester.pumpWidget(view(controller));
      await pumpUntil(tester, () => plugin.submissions.isNotEmpty);
      await tester.pump(const Duration(milliseconds: 150));
      expect(controller.needsCpuFallback, isTrue);
      expect(find.text('CPU reference'), findsOneWidget);
      final calls = plugin.calls.length;
      await tester.pump(const Duration(seconds: 1));
      expect(plugin.calls.length, calls);
      await tester.pumpWidget(const SizedBox.shrink());
      await controller.close();
      controller.dispose();
      await document.dispose();
      plugin.uninstall();
    },
  );
}
