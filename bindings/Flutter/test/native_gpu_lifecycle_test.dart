// Flutter lifecycle/stamp contracts, without OS window or document pixel claims.
import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
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

Widget view(
  NativeTextureController controller, {
  bool fillWindow = false,
  ValueChanged<Object>? onPauseError,
}) => MaterialApp(
  home: NativeGpuLifecycle(
    controller: controller,
    onPauseError: onPauseError,
    child: Center(
      child: SizedBox(
        width: fillWindow ? null : 300,
        height: fillWindow ? null : 240,
        child: LayoutBuilder(
          builder: (context, constraints) => NativeTextureView(
            controller: controller,
            viewport: NativePageViewport.fit(
              const NativePageSize(600, 800),
              constraints.biggest,
            ),
            cpuFallback: (_) => const Text('CPU reference'),
          ),
        ),
      ),
    ),
  ),
);

List<MethodCall> mockAppKitViews() {
  final calls = <MethodCall>[];
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  messenger.setMockMethodCallHandler(SystemChannels.platform_views, (
    call,
  ) async {
    calls.add(call);
    return null;
  });
  addTearDown(() {
    messenger.setMockMethodCallHandler(SystemChannels.platform_views, null);
  });
  return calls;
}

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
    testWidgets('$hidden on initial mount does not create or capture', (
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
      try {
        tester.binding.handleAppLifecycleStateChanged(hidden);
        // Hidden bindings do not schedule frames; force this initial mount.
        tester.binding.scheduleForcedFrame();
        await tester.pumpWidget(view(controller));
        await tester.pump(const Duration(milliseconds: 100));
        expect(controller.isPaused, isTrue);
        expect(controller.canDisplay, isFalse);
        expect(transport.snapshots, isEmpty);
        expect(plugin.calls, isEmpty);
        expect(find.byType(NativeTextureView), findsOneWidget);
        tester.binding.handleAppLifecycleStateChanged(
          AppLifecycleState.resumed,
        );
        await until(tester, () => controller.canDisplay);
        expect(plugin.calls.where((call) => call == 'create'), hasLength(1));
        expect(transport.snapshots, hasLength(1));
        expect(document.isDisposed, isFalse);
      } finally {
        await cleanup(tester, controller, document, plugin);
      }
    });

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

  testWidgets(
    'a controller replaced while hidden waits for resume to capture',
    (tester) async {
      final transport = GpuTransport();
      final document = await opened(tester, transport);
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      NativeTextureController make() => NativeTextureController(
        document: document,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      final original = make();
      NativeTextureController? replacement;
      try {
        await tester.pumpWidget(view(original));
        await until(tester, () => original.canDisplay);
        final mountedView = tester.state(find.byType(NativeTextureView));
        tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
        final retiring = original.close();
        await tester.pump(const Duration(milliseconds: 40));
        await retiring;
        expect(original.phase, NativeTexturePhase.closed);
        final captures = transport.snapshots.length;
        final calls = plugin.calls.length;
        plugin.closed = false;
        final next = replacement = make();
        tester.binding.scheduleForcedFrame();
        await tester.pumpWidget(view(next));
        await tester.pump(const Duration(milliseconds: 100));
        expect(tester.state(find.byType(NativeTextureView)), same(mountedView));
        expect(next.isPaused, isTrue);
        expect(next.canDisplay, isFalse);
        expect(transport.snapshots, hasLength(captures));
        expect(plugin.calls, hasLength(calls));
        expect(transport.closes, 0);
        tester.binding.handleAppLifecycleStateChanged(
          AppLifecycleState.resumed,
        );
        await until(tester, () => next.canDisplay);
        expect(transport.snapshots, hasLength(captures + 1));
        expect(plugin.calls.where((call) => call == 'create'), hasLength(2));
        expect(next.document, same(document));
      } finally {
        await cleanup(tester, replacement ?? original, document, plugin);
        if (replacement != null) original.dispose();
      }
    },
  );

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

  testWidgets(
    'a physical size change covers before the new layout is captured',
    (tester) async {
      tester.view.devicePixelRatio = 1;
      tester.view.physicalSize = const Size(300, 240);
      addTearDown(tester.view.resetDevicePixelRatio);
      addTearDown(tester.view.resetPhysicalSize);
      final transport = GpuTransport();
      final document = await opened(tester, transport);
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final controller = NativeTextureController(
        document: document,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      try {
        await tester.pumpWidget(view(controller, fillWindow: true));
        await until(tester, () => controller.canDisplay);
        final oldGeneration = document.presentationGeneration;
        final captures = transport.snapshots.length;
        final submissions = plugin.submissions.length;
        tester.view.physicalSize = const Size(420, 320);
        expect(controller.canDisplay, isFalse);
        expect(controller.isPaused, isTrue);
        expect(transport.snapshots, hasLength(captures));
        expect(plugin.submissions, hasLength(submissions));
        await until(tester, () => controller.canDisplay);
        expect(plugin.submissions.skip(submissions), hasLength(1));
        final last = plugin.submissions.last;
        expect(last['width'], 420);
        expect(last['height'], 320);
        expect((last['stamp'] as Map)['deviceScale'], 1);
        expect(
          (last['stamp'] as Map)['viewGeneration'],
          greaterThan(oldGeneration),
        );
      } finally {
        await cleanup(tester, controller, document, plugin);
      }
    },
  );

  testWidgets('metrics finishing while hidden do not resume admission', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(300, 240);
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.view.resetPhysicalSize);
    final transport = GpuTransport();
    final document = await opened(tester, transport);
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final controller = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
    );
    try {
      await tester.pumpWidget(view(controller, fillWindow: true));
      await until(tester, () => controller.canDisplay);
      final mountedView = tester.state(find.byType(NativeTextureView));
      final captures = transport.snapshots.length;
      final submissions = plugin.submissions.length;
      tester.view.physicalSize = const Size(360, 280);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
      tester.view.physicalSize = const Size(420, 320);
      await tester
          .pump(); // Applies geometry and runs the last metrics callback.
      await tester.pump(const Duration(milliseconds: 100));
      expect(controller.isPaused, isTrue);
      expect(controller.canDisplay, isFalse);
      expect(transport.snapshots, hasLength(captures));
      expect(plugin.submissions, hasLength(submissions));
      expect(tester.state(find.byType(NativeTextureView)), same(mountedView));
      expect(find.byType(Texture), findsOneWidget);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await until(tester, () => controller.canDisplay);
      expect(plugin.submissions.skip(submissions), hasLength(1));
      expect(plugin.submissions.last['width'], 420);
      expect(plugin.submissions.last['height'], 320);
    } finally {
      await cleanup(tester, controller, document, plugin);
    }
  });

  testWidgets('AppKit resume cannot uncover before delayed native hide', (
    tester,
  ) async {
    final platformCalls = mockAppKitViews();
    final document = await opened(tester, GpuTransport());
    final plugin = FakeTexturePlugin(NativeTexturePlatform.macosMetalView)
      ..install();
    final controller = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.macosMetalView,
      pollInterval: const Duration(milliseconds: 10),
    );
    final errors = <Object>[];
    final gate = Completer<void>();
    try {
      await tester.pumpWidget(view(controller, onPauseError: errors.add));
      await until(tester, () => controller.canDisplay);
      final mountedView = tester.state(find.byType(NativeTextureView));
      final oldGeneration = document.presentationGeneration;
      final oldCovers = plugin.covers.length;
      final oldSubmissions = plugin.submissions.length;
      expect(plugin.covered, isFalse);
      plugin.coverGate = gate;
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
      expect(controller.canDisplay, isFalse);
      await until(tester, () => plugin.covers.length > oldCovers);
      expect(controller.isPaused, isTrue);
      // A Flutter cover does not claim that the native layer has hidden yet.
      expect(plugin.covered, isFalse);
      expect(find.byType(AppKitView), findsOneWidget);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await until(tester, () => plugin.submissions.length > oldSubmissions);
      await tester.pump(const Duration(milliseconds: 30));
      expect(controller.canDisplay, isFalse);
      expect(
        plugin.covers.skip(oldCovers).where((call) => call['covered'] == false),
        isEmpty,
      );
      expect(tester.state(find.byType(NativeTextureView)), same(mountedView));
      gate.complete();
      await until(tester, () => controller.canDisplay);
      final uncovered = plugin.covers.last;
      expect(uncovered['covered'], isFalse);
      expect(uncovered['stamp'], plugin.acceptedStamp);
      expect(
        (uncovered['stamp'] as Map)['viewGeneration'],
        greaterThan(oldGeneration),
      );
      expect(errors, isEmpty);
      expect(
        (platformCalls.firstWhere((call) => call.method == 'create').arguments
            as Map)['viewType'],
        'rhwp_native/metal_view',
      );
    } finally {
      if (!gate.isCompleted) gate.complete();
      await cleanup(tester, controller, document, plugin);
    }
  });

  testWidgets(
    'a current AppKit hide failure reaches the host and stays covered',
    (tester) async {
      mockAppKitViews();
      final document = await opened(tester, GpuTransport());
      final plugin = FakeTexturePlugin(NativeTexturePlatform.macosMetalView)
        ..install();
      final controller = NativeTextureController(
        document: document,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.macosMetalView,
        pollInterval: const Duration(milliseconds: 10),
      );
      final errors = <Object>[];
      final gate = Completer<void>();
      try {
        await tester.pumpWidget(view(controller, onPauseError: errors.add));
        await until(tester, () => controller.canDisplay);
        final covers = plugin.covers.length;
        plugin.coverGate = gate;
        tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
        await until(tester, () => plugin.covers.length > covers);
        gate.completeError(PlatformException(code: 'contextChanged'));
        await until(tester, () => errors.isNotEmpty);
        expect(errors, hasLength(1));
        expect(
          errors.single,
          isA<PlatformException>().having(
            (error) => error.code,
            'code',
            'contextChanged',
          ),
        );
        expect(controller.needsCpuFallback, isTrue);
        expect(controller.canDisplay, isFalse);
        expect(find.text('CPU reference'), findsOneWidget);
        final submissions = plugin.submissions.length;
        tester.binding.handleAppLifecycleStateChanged(
          AppLifecycleState.resumed,
        );
        await tester.pump(const Duration(milliseconds: 50));
        expect(plugin.submissions, hasLength(submissions));
        expect(controller.needsCpuFallback, isTrue);
      } finally {
        if (!gate.isCompleted) gate.complete();
        await cleanup(tester, controller, document, plugin);
      }
    },
  );

  testWidgets('an old AppKit pause error is not reported for its replacement', (
    tester,
  ) async {
    mockAppKitViews();
    final document = await opened(tester, GpuTransport());
    final oldPlugin = FakeTexturePlugin(NativeTexturePlatform.macosMetalView)
      ..install();
    final nextPlugin = FakeTexturePlugin(NativeTexturePlatform.linux)
      ..install();
    final original = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.macosMetalView,
      pollInterval: const Duration(milliseconds: 10),
    );
    NativeTextureController? replacement;
    final errors = <Object>[];
    final gate = Completer<void>();
    try {
      await tester.pumpWidget(view(original, onPauseError: errors.add));
      await until(tester, () => original.canDisplay);
      final mountedView = tester.state(find.byType(NativeTextureView));
      final covers = oldPlugin.covers.length;
      oldPlugin.coverGate = gate;
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
      await until(tester, () => oldPlugin.covers.length > covers);
      final next = replacement = NativeTextureController(
        document: document,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      tester.binding.scheduleForcedFrame();
      await tester.pumpWidget(view(next, onPauseError: errors.add));
      expect(tester.state(find.byType(NativeTextureView)), same(mountedView));
      expect(next.isPaused, isTrue);
      expect(nextPlugin.calls, isEmpty);
      gate.completeError(PlatformException(code: 'contextChanged'));
      await tester.pump(const Duration(milliseconds: 40));
      await original.close();
      expect(original.phase, NativeTexturePhase.closed);
      expect(errors, isEmpty);
      expect(nextPlugin.calls, isEmpty);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await until(tester, () => next.canDisplay);
      expect(errors, isEmpty);
      expect(document.isDisposed, isFalse);
    } finally {
      if (!gate.isCompleted) gate.complete();
      await cleanup(
        tester,
        replacement ?? original,
        document,
        replacement == null ? oldPlugin : nextPlugin,
      );
      if (replacement != null) {
        original.dispose();
        oldPlugin.uninstall();
      } else {
        nextPlugin.uninstall();
      }
    }
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
