import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_document.dart';
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

void main() {
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
