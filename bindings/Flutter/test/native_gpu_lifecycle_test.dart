// Flutter lifecycle/stamp contracts, without OS window or document pixel claims.
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_document.dart';
import 'package:rhwp_native_reference/src/native_gpu_lifecycle.dart';
import 'package:rhwp_native_reference/src/native_texture_view.dart';
import 'package:rhwp_native_reference/src/native_viewport.dart';

import 'support/fake_gpu_presentation.dart';

Future<void> until(WidgetTester tester, bool Function() ready) async {
  for (var i = 0; i < 100 && !ready(); i++) {
    await tester.pump(const Duration(milliseconds: 10));
  }
  expect(ready(), isTrue);
}

Future<NativeDocument> opened(
  WidgetTester tester,
  GpuTransport transport,
) async {
  final owner = await NativeDocument.connect(transport);
  final opening = owner.newDocument();
  await until(tester, () => owner.info != null);
  await opening;
  return owner;
}

Widget view(NativeTextureController controller) => MaterialApp(
  home: NativeGpuLifecycle(
    controller: controller,
    child: Center(
      child: SizedBox(
        width: 300,
        height: 240,
        child: NativeTextureView(
          controller: controller,
          viewport: NativePageViewport.fit(
            const NativePageSize(600, 800),
            const Size(300, 240),
          ),
          cpuFallback: (_) => const Text('CPU reference'),
        ),
      ),
    ),
  ),
);

Future<void> cleanup(
  WidgetTester tester,
  NativeTextureController controller,
  NativeDocument document,
  FakeTexturePlugin plugin,
) async {
  tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
  final closing = controller.close();
  await tester.pump(const Duration(milliseconds: 40));
  await closing;
  await tester.pumpWidget(const SizedBox.shrink());
  controller.dispose();
  await document.dispose();
  plugin.uninstall();
}

void main() {
  for (final hidden in [
    AppLifecycleState.hidden,
    AppLifecycleState.paused,
    AppLifecycleState.detached,
  ]) {
    testWidgets('$hidden holds admission until a current source is resumed', (
      tester,
    ) async {
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetDevicePixelRatio);
      final transport = GpuTransport();
      final document = await opened(tester, transport);
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final controller = NativeTextureController(
        document: document,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      await tester.pumpWidget(view(controller));
      await until(tester, () => controller.canDisplay);
      tester.binding.handleAppLifecycleStateChanged(hidden);
      expect(controller.canDisplay, isFalse);
      final captures = transport.snapshots.length;
      final calls = plugin.calls.length;
      transport.revision++;
      final refresh = document.refreshInfo();
      await tester.pump(const Duration(milliseconds: 30));
      await refresh;
      await tester.pump(const Duration(seconds: 1));
      expect(transport.snapshots, hasLength(captures));
      expect(plugin.calls, hasLength(calls));
      expect(document.isDisposed, isFalse);
      expect(find.byType(Texture), findsOneWidget);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await until(tester, () => controller.canDisplay);
      expect(
        (plugin.submissions.last['stamp'] as Map)['revision'],
        transport.revision,
      );
      expect(transport.closes, 0);
      await cleanup(tester, controller, document, plugin);
    });
  }

  testWidgets('inactive but visible desktop does not suspend presentation', (
    tester,
  ) async {
    final document = await opened(tester, GpuTransport());
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final controller = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
    );
    await tester.pumpWidget(view(controller));
    await until(tester, () => controller.canDisplay);
    final generation = document.presentationGeneration;
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.inactive);
    expect(controller.canDisplay, isTrue);
    await tester.pump(const Duration(milliseconds: 30));
    expect(document.presentationGeneration, generation);
    await cleanup(tester, controller, document, plugin);
  });

  testWidgets('DPR change covers synchronously and submits new physical size', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetDevicePixelRatio);
    final document = await opened(tester, GpuTransport());
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final controller = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
    );
    await tester.pumpWidget(view(controller));
    await until(tester, () => controller.canDisplay);
    final oldGeneration = document.presentationGeneration;
    final submissions = plugin.submissions.length;
    tester.view.devicePixelRatio = 2;
    expect(controller.canDisplay, isFalse);
    expect(plugin.submissions, hasLength(submissions));
    await until(tester, () => controller.canDisplay);
    final last = plugin.submissions.last;
    expect((last['stamp'] as Map)['deviceScale'], 2);
    expect(
      (last['stamp'] as Map)['viewGeneration'],
      greaterThan(oldGeneration),
    );
    expect(last['width'], 600);
    expect(last['height'], 480);
    await cleanup(tester, controller, document, plugin);
  });

  testWidgets('unrelated global metric notifications do not recapture', (
    tester,
  ) async {
    final transport = GpuTransport();
    final document = await opened(tester, transport);
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final controller = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
    );
    await tester.pumpWidget(view(controller));
    await until(tester, () => controller.canDisplay);
    final generation = document.presentationGeneration;
    final captures = transport.snapshots.length;
    tester.binding.handleMetricsChanged();
    expect(controller.canDisplay, isTrue);
    await tester.pump(const Duration(milliseconds: 30));
    expect(document.presentationGeneration, generation);
    expect(transport.snapshots, hasLength(captures));
    await cleanup(tester, controller, document, plugin);
  });

  testWidgets('several metric changes resume only the last geometry', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetDevicePixelRatio);
    final document = await opened(tester, GpuTransport());
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final controller = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
    );
    await tester.pumpWidget(view(controller));
    await until(tester, () => controller.canDisplay);
    final count = plugin.submissions.length;
    tester.view.devicePixelRatio = 1.25;
    tester.view.devicePixelRatio = 1.5;
    tester.view.devicePixelRatio = 2;
    expect(controller.canDisplay, isFalse);
    await until(tester, () => controller.canDisplay);
    expect(plugin.submissions.skip(count), hasLength(1));
    expect((plugin.submissions.last['stamp'] as Map)['deviceScale'], 2);
    await cleanup(tester, controller, document, plugin);
  });
}
