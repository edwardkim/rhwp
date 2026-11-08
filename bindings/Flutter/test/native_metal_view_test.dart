// Mocked channel/widget contracts; these are not actual Metal sampling evidence.
import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart' show PlatformViewHitTestBehavior;
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_document.dart';
import 'package:rhwp_native_reference/src/native_editor_controller.dart';
import 'package:rhwp_native_reference/src/native_texture_view.dart';
import 'package:rhwp_native_reference/src/native_viewport.dart';

import 'support/fake_gpu_presentation.dart';

class MetalPlugin extends FakeTexturePlugin {
  MetalPlugin() : super(NativeTexturePlatform.macosMetalView) {
    ready = false;
  }
  bool holdClose = false, ownerComplete = false, layerComplete = true;
  @override
  Map<String, Object?> status() => {
    ...super.status(),
    'covered': covered,
    'nativeViewLayerDetached': closed && layerComplete,
    if (closed && holdClose) ...{
      'ownerFinished': ownerComplete,
      'registered': !ownerComplete,
    },
  };
}

class PlatformViewsMock {
  PlatformViewsMock(this.plugin) {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(SystemChannels.platform_views, handle);
  }
  final MetalPlugin plugin;
  final creates = <Map<String, Object?>>[], disposed = <int>[];
  Future<Object?> handle(MethodCall call) async {
    if (call.method == 'create') {
      final args = Map<String, Object?>.from(call.arguments as Map);
      final encoded = args['params']! as Uint8List;
      args['decodedParams'] = const StandardMessageCodec().decodeMessage(
        ByteData.sublistView(encoded),
      );
      creates.add(args);
      plugin.ready = true; // Factory/window/geometry binding receipt.
    } else if (call.method == 'dispose') {
      disposed.add(call.arguments! as int);
    }
    return null;
  }

  void uninstall() => TestDefaultBinaryMessengerBinding
      .instance
      .defaultBinaryMessenger
      .setMockMethodCallHandler(SystemChannels.platform_views, null);
}

Future<void> until(WidgetTester tester, bool Function() ready) async {
  for (var index = 0; index < 100 && !ready(); ++index) {
    await tester.pump(const Duration(milliseconds: 10));
  }
  expect(ready(), isTrue);
}

NativePageViewport geometry([double width = 300]) =>
    NativePageViewport.fit(const NativePageSize(600, 800), Size(width, 240));

Widget viewport(NativeTextureController controller, {double width = 300}) =>
    MaterialApp(
      home: MediaQuery(
        data: const MediaQueryData(devicePixelRatio: 1),
        child: SizedBox(
          width: width,
          height: 240,
          child: NativeTextureView(
            controller: controller,
            viewport: geometry(width),
            cpuFallback: (_) => const Text('CPU input owner'),
          ),
        ),
      ),
    );

class Fixture {
  Fixture(
    this.owner,
    this.transport,
    this.metadata,
    this.plugin,
    this.views,
    this.controller,
  );
  final NativeDocument owner;
  final GpuTransport transport;
  final FakePresentation metadata;
  final MetalPlugin plugin;
  final PlatformViewsMock views;
  final NativeTextureController controller;
  Future<void> close(WidgetTester tester) async {
    for (final gate in [
      plugin.createGate,
      plugin.submitGate,
      plugin.statusGate,
      plugin.coverGate,
      plugin.uncoverGate,
      plugin.uncoverAckGate,
      plugin.retireAckGate,
      plugin.closeReplyGate,
    ]) {
      if (gate != null && !gate.isCompleted) gate.complete();
    }
    final closing = controller.close();
    await tester.pump(const Duration(milliseconds: 40));
    await closing;
    await tester.pumpWidget(const SizedBox.shrink());
    controller.dispose();
    await owner.dispose();
    plugin.uninstall();
    views.uninstall();
  }
}

Future<Fixture> fixture(
  WidgetTester tester, {
  MetalPlugin? plugin,
  FakePresentation? metadata,
  Duration timeout = const Duration(milliseconds: 500),
}) async {
  plugin ??= MetalPlugin();
  metadata ??= FakePresentation();
  plugin.install();
  final views = PlatformViewsMock(plugin), transport = GpuTransport();
  final owner = await NativeDocument.connect(transport);
  final opening = owner.newDocument();
  await until(tester, () => owner.info != null);
  await opening;
  final controller = NativeTextureController(
    document: owner,
    presentation: metadata,
    platform: NativeTexturePlatform.macosMetalView,
    pollInterval: const Duration(milliseconds: 10),
    operationTimeout: timeout,
  );
  return Fixture(owner, transport, metadata, plugin, views, controller);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test('native Metal view selection is explicit and macOS only', () {
    try {
      debugDefaultTargetPlatformOverride = TargetPlatform.macOS;
      expect(NativeTexturePlatform.current(), NativeTexturePlatform.macos);
      expect(
        NativeTexturePlatform.current(preferContinuous: true),
        NativeTexturePlatform.macosMetalView,
      );
      debugDefaultTargetPlatformOverride = TargetPlatform.windows;
      expect(
        NativeTexturePlatform.current(preferContinuous: true),
        NativeTexturePlatform.windows,
      );
      debugDefaultTargetPlatformOverride = TargetPlatform.linux;
      expect(
        NativeTexturePlatform.current(preferContinuous: true),
        NativeTexturePlatform.linux,
      );
    } finally {
      debugDefaultTargetPlatformOverride = null;
    }
  });

  testWidgets(
    'factory binds the native handle beneath an opaque cover without a Texture',
    (tester) async {
      final f = await fixture(tester, plugin: MetalPlugin()..publish = false);
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.plugin.submissions.isNotEmpty);
        expect(f.views.creates.single['viewType'], 'rhwp_native/metal_view');
        expect(f.views.creates.single['decodedParams'], {
          'handle': f.controller.textureId,
        });
        expect(find.byType(Texture), findsNothing);
        final view = tester.widget<AppKitView>(find.byType(AppKitView));
        expect(view.hitTestBehavior, PlatformViewHitTestBehavior.transparent);
        expect(view.creationParamsCodec, isA<StandardMessageCodec>());
        expect(
          find.ancestor(
            of: find.byType(AppKitView),
            matching: find.byWidgetPredicate(
              (widget) => widget is IgnorePointer && widget.ignoring,
            ),
          ),
          findsOneWidget,
        );
        expect(
          find.byKey(const ValueKey('rhwp-native-texture-cover')),
          findsOneWidget,
        );
        expect(f.plugin.covered, isTrue);
        f.plugin.publish = true;
        await until(tester, () => f.controller.canDisplay);
        final reveal = f.plugin.covers.last;
        expect(reveal['covered'], isFalse);
        expect(reveal['stamp'], f.plugin.acceptedStamp);
        expect(f.metadata.checks.length, greaterThanOrEqualTo(2));
        expect(f.metadata.released, isEmpty);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'visibility waits for the matching native uncover acknowledgment',
    (tester) async {
      final gate = Completer<void>();
      final f = await fixture(
        tester,
        plugin: MetalPlugin()..uncoverAckGate = gate,
      );
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(
          tester,
          () => f.plugin.covers.any((call) => call['covered'] == false),
        );
        await tester.pump(const Duration(milliseconds: 40));
        expect(f.controller.canDisplay, isFalse);
        expect(
          find.byKey(const ValueKey('rhwp-native-texture-cover')),
          findsOneWidget,
        );
        gate.complete();
        await until(tester, () => f.controller.canDisplay);
        expect(f.plugin.covered, isFalse);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'rejected native full-stamp uncover remains covered and never releases native ownership',
    (tester) async {
      final f = await fixture(
        tester,
        plugin: MetalPlugin()..rejectUncover = true,
      );
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.needsCpuFallback);
        expect(f.plugin.covered, isTrue);
        expect(f.metadata.released, isEmpty);
        expect(f.plugin.releasedByPlugin, {101});
        await until(tester, () => f.plugin.retireAcks.length == 1);
        await tester.pump(const Duration(milliseconds: 20));
        final closing = f.controller.close();
        await tester.pump(const Duration(milliseconds: 30));
        await closing;
        expect(f.controller.phase, NativeTexturePhase.closed);
        expect(f.controller.retirementPending, isFalse);
        expect(f.plugin.retireAcks, hasLength(1));
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'a late old uncover acknowledgment cannot expose a new generation',
    (tester) async {
      final gate = Completer<void>();
      final f = await fixture(
        tester,
        plugin: MetalPlugin()..uncoverAckGate = gate,
      );
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(
          tester,
          () => f.plugin.covers.any((call) => call['covered'] == false),
        );
        final oldSerial = f.plugin.visibilitySerial;
        f.plugin.publish = false;
        f.controller.invalidate();
        expect(f.controller.canDisplay, isFalse);
        await until(
          tester,
          () => f.plugin.covered && f.plugin.visibilitySerial > oldSerial,
        );
        gate.complete();
        await tester.pump(const Duration(milliseconds: 40));
        expect(f.controller.canDisplay, isFalse);
        expect(f.plugin.covered, isTrue);
        f.plugin.uncoverAckGate = null;
        f.plugin.publish = true;
        await until(tester, () => f.controller.canDisplay);
        expect(f.plugin.submissions.length, greaterThanOrEqualTo(2));
        expect(f.metadata.released, isEmpty);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'pause wins against a delayed old uncover before native application',
    (tester) async {
      final gate = Completer<void>();
      final f = await fixture(
        tester,
        plugin: MetalPlugin()..uncoverGate = gate,
      );
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(
          tester,
          () => f.plugin.covers.any((call) => call['covered'] == false),
        );
        final paused = f.controller.pause();
        await until(
          tester,
          () => f.plugin.covered && f.plugin.covers.last['covered'] == true,
        );
        await paused;
        gate.complete();
        await tester.pump(const Duration(milliseconds: 50));
        expect(f.controller.isPaused, isTrue);
        expect(f.controller.canDisplay, isFalse);
        expect(f.plugin.covered, isTrue);
        expect(f.plugin.submissions, hasLength(1));
        expect(f.plugin.closed, isFalse);
        f.plugin.uncoverGate = null;
        f.controller.resume();
        await until(tester, () => f.controller.canDisplay);
        expect(f.plugin.submissions, hasLength(2));
        expect(f.metadata.released, isEmpty);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'pause awaits hiding and retains the same view document input owner and native leases',
    (tester) async {
      final f = await fixture(tester);
      final editor = NativeEditorController(f.owner);
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.canDisplay);
        final editorGeneration = f.owner.editorGeneration,
            handle = f.controller.textureId;
        final gate = Completer<void>();
        f.plugin.coverGate = gate;
        var complete = false;
        final paused = f.controller.pause().then((_) {
          complete = true;
        });
        await tester.pump(const Duration(milliseconds: 50));
        expect(complete, isFalse);
        expect(f.controller.isPaused, isTrue);
        expect(f.controller.canDisplay, isFalse);
        expect(f.plugin.owned, {101});
        expect(f.plugin.releasedByPlugin, isEmpty);
        expect(find.byType(AppKitView), findsOneWidget);
        expect(find.text('CPU input owner'), findsOneWidget);
        expect(f.views.disposed, isEmpty);
        expect(f.views.creates, hasLength(1));
        expect(identical(editor.document, f.controller.document), isTrue);
        expect(f.owner.editorGeneration, editorGeneration);
        gate.complete();
        await tester.pump(const Duration(milliseconds: 20));
        await paused;
        expect(f.plugin.covered, isTrue);
        final calls = f.plugin.calls.length,
            captures = f.transport.snapshots.length;
        await tester.pump(const Duration(milliseconds: 100));
        expect(f.plugin.calls.length, calls);
        expect(f.transport.snapshots.length, captures);
        f.controller.resume();
        await until(tester, () => f.controller.canDisplay);
        expect(f.controller.textureId, handle);
        expect(f.owner.editorGeneration, editorGeneration);
        expect(f.views.creates, hasLength(1));
        expect(f.metadata.released, isEmpty);
      } finally {
        editor.dispose();
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'paused source replacement and resize publish only the latest plan after resume',
    (tester) async {
      final f = await fixture(tester);
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.canDisplay);
        final paused = f.controller.pause();
        await tester.pump(const Duration(milliseconds: 20));
        await paused;
        final oldDocument = f.owner.info!.documentId;
        final replacing = f.owner.newDocument();
        await until(tester, () => f.owner.info!.documentId != oldDocument);
        await replacing;
        await tester.pumpWidget(viewport(f.controller, width: 360));
        await tester.pump(const Duration(milliseconds: 50));
        expect(f.transport.snapshots, [101]);
        expect(f.plugin.submissions, hasLength(1));
        expect(f.plugin.covered, isTrue);
        f.controller.resume();
        await until(tester, () => f.controller.canDisplay);
        final latest = f.plugin.submissions.last;
        expect(
          (latest['stamp'] as Map)['documentId'],
          f.owner.info!.documentId,
        );
        expect(latest['width'], 360);
        expect(f.transport.snapshots, [101, 102]);
        expect(f.metadata.released, isEmpty);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets('pause during a full source check never sends an old uncover', (
    tester,
  ) async {
    final f = await fixture(tester, plugin: MetalPlugin()..publish = false);
    final gate = Completer<bool>();
    try {
      await tester.pumpWidget(viewport(f.controller));
      await until(tester, () => f.plugin.submissions.isNotEmpty);
      f.metadata.checkGate = gate;
      f.plugin.publish = true;
      await until(tester, () => f.metadata.checks.length >= 2);
      final paused = f.controller.pause();
      await tester.pump(const Duration(milliseconds: 20));
      await paused;
      gate.complete(true);
      await tester.pump(const Duration(milliseconds: 40));
      expect(
        f.plugin.covers.where((call) => call['covered'] == false),
        isEmpty,
      );
      expect(f.controller.canDisplay, isFalse);
      expect(f.plugin.covered, isTrue);
      expect(f.plugin.owned, {101});
      f.metadata.checkGate = null;
      f.controller.resume();
      await until(tester, () => f.controller.canDisplay);
      expect(f.plugin.submissions, hasLength(2));
      expect(f.metadata.released, isEmpty);
    } finally {
      if (!gate.isCompleted) gate.complete(true);
      await f.close(tester);
    }
  });

  testWidgets(
    'pause during an accepted submit retains its late ownership observer without replay',
    (tester) async {
      final gate = Completer<void>();
      final f = await fixture(
        tester,
        plugin: MetalPlugin()
          ..acceptedBeforeReply = true
          ..submitGate = gate,
      );
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.plugin.owned.isNotEmpty);
        final paused = f.controller.pause();
        await tester.pump(const Duration(milliseconds: 20));
        await paused;
        expect(NativeTextureController.retainedOwnedSnapshots, 1);
        gate.complete();
        await tester.pump(const Duration(milliseconds: 30));
        expect(NativeTextureController.retainedOwnedSnapshots, 0);
        expect(f.plugin.owned, {101});
        expect(f.metadata.released, isEmpty);
        expect(f.plugin.submissions, hasLength(1));
        f.plugin.submitGate = null;
        f.controller.resume();
        await until(tester, () => f.controller.canDisplay);
        expect(f.plugin.submissions, hasLength(2));
        expect(f.plugin.releasedByPlugin, {101});
        expect(f.metadata.released, isEmpty);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'pause during a late create awaits native cover and keeps one registration',
    (tester) async {
      final gate = Completer<void>();
      final f = await fixture(tester, plugin: MetalPlugin()..createGate = gate);
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.plugin.calls.contains('create'));
        var complete = false;
        final paused = f.controller.pause().then((_) {
          complete = true;
        });
        await tester.pump(const Duration(milliseconds: 20));
        expect(complete, isFalse);
        gate.complete();
        await until(tester, () => complete);
        await paused;
        expect(f.plugin.covered, isTrue);
        expect(f.plugin.submissions, isEmpty);
        expect(f.transport.snapshots, isEmpty);
        f.controller.resume();
        await until(tester, () => f.controller.canDisplay);
        expect(
          f.plugin.calls.where((method) => method == 'create'),
          hasLength(1),
        );
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'paused invalidation storms coalesce bounded native cover requests',
    (tester) async {
      final f = await fixture(tester);
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.canDisplay);
        final gate = Completer<void>();
        f.plugin.coverGate = gate;
        final covers = f.plugin.covers.length;
        final paused = f.controller.pause();
        await tester.pump(const Duration(milliseconds: 10));
        for (var index = 0; index < 100; ++index) {
          f.controller.invalidate();
        }
        await tester.pump(const Duration(milliseconds: 40));
        expect(f.plugin.covers.length, covers + 1);
        expect(f.plugin.submissions, hasLength(1));
        gate.complete();
        await tester.pump(const Duration(milliseconds: 20));
        await paused;
        expect(f.plugin.covers.length, covers + 2);
        expect(f.plugin.covered, isTrue);
        expect(f.metadata.released, isEmpty);
        expect(NativeTextureController.retainedOwnedSnapshots, 0);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'close cancels an in-progress pause and resume cannot recreate the view',
    (tester) async {
      final f = await fixture(tester);
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.canDisplay);
        final gate = Completer<void>();
        f.plugin.coverGate = gate;
        final failed = expectLater(
          f.controller.pause(),
          throwsA(isA<Exception>()),
        );
        await tester.pump(const Duration(milliseconds: 10));
        final closing = f.controller.close();
        await tester.pump(const Duration(milliseconds: 30));
        await closing;
        await failed;
        gate.complete();
        await tester.pump(const Duration(milliseconds: 30));
        f.controller.resume();
        await tester.pump(const Duration(milliseconds: 50));
        expect(f.controller.phase, NativeTexturePhase.closed);
        expect(f.plugin.covered, isTrue);
        expect(
          f.plugin.calls.where((method) => method == 'create'),
          hasLength(1),
        );
        expect(f.plugin.submissions, hasLength(1));
        expect(f.metadata.released, isEmpty);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'a native cover timeout is bounded and never refunds an accepted snapshot in Dart',
    (tester) async {
      final f = await fixture(
        tester,
        timeout: const Duration(milliseconds: 100),
      );
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.canDisplay);
        final gate = Completer<void>();
        f.plugin.coverGate = gate;
        final failed = expectLater(
          f.controller.pause(),
          throwsA(isA<TimeoutException>()),
        );
        await tester.pump(const Duration(milliseconds: 150));
        await failed;
        expect(f.controller.needsCpuFallback, isTrue);
        expect(f.plugin.covered, isTrue);
        expect(f.metadata.released, isEmpty);
        expect(f.plugin.releasedByPlugin, {101});
        gate.complete();
        await tester.pump(const Duration(milliseconds: 30));
        expect(f.controller.canDisplay, isFalse);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'native layer detach alone does not complete owner and registration retirement',
    (tester) async {
      final f = await fixture(tester, plugin: MetalPlugin()..holdClose = true);
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.canDisplay);
        var complete = false;
        final closing = f.controller.close().then((_) {
          complete = true;
        });
        await tester.pump(const Duration(milliseconds: 40));
        expect(f.plugin.status()['nativeViewLayerDetached'], isTrue);
        expect(complete, isFalse);
        expect(f.controller.phase, NativeTexturePhase.closing);
        expect(find.byType(AppKitView), findsOneWidget);
        f.plugin.ownerComplete = true;
        await until(tester, () => complete);
        await closing;
        expect(f.controller.phase, NativeTexturePhase.closed);
        expect(f.controller.retirementPending, isFalse);
        expect(f.metadata.released, isEmpty);
      } finally {
        f.plugin.ownerComplete = true;
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'Metal view retirement waits for its explicit matching acknowledgement',
    (tester) async {
      final f = await fixture(tester);
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.canDisplay);
        final gate = Completer<void>();
        f.plugin.retireAckGate = gate;
        var complete = false;
        final closing = f.controller.close().then((_) {
          complete = true;
        });
        await until(tester, () => f.plugin.retireAcks.isNotEmpty);
        expect(f.plugin.retireAcks.single, {
          'textureId': f.controller.textureId,
        });
        expect(complete, isFalse);
        expect(f.controller.retirementPending, isTrue);
        expect(f.controller.phase, NativeTexturePhase.closing);
        gate.complete();
        await until(tester, () => complete);
        await closing;
        expect(f.controller.retirementPending, isFalse);
        expect(f.controller.phase, NativeTexturePhase.closed);
        await f.controller.close();
        expect(f.plugin.retireAcks, hasLength(1));
        expect(f.metadata.released, isEmpty);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'owner completion alone does not acknowledge an attached native layer',
    (tester) async {
      final f = await fixture(
        tester,
        plugin: MetalPlugin()..layerComplete = false,
      );
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.canDisplay);
        var complete = false;
        final closing = f.controller.close().then((_) {
          complete = true;
        });
        await tester.pump(const Duration(milliseconds: 50));
        expect(f.plugin.status()['ownerFinished'], isTrue);
        expect(f.plugin.retireAcks, isEmpty);
        expect(complete, isFalse);
        f.plugin.layerComplete = true;
        await until(tester, () => complete);
        await closing;
        expect(f.plugin.retireAcks, hasLength(1));
        expect(f.controller.retirementPending, isFalse);
      } finally {
        f.plugin.layerComplete = true;
        await f.close(tester);
      }
    },
  );

  final invalidAcknowledgements = <Map<String, Object?>>[
    {'textureId': 999},
    {'acknowledged': false},
    {'closed': false},
    {'ownerFinished': false},
    {'registered': true},
    {'nativeViewLayerDetached': false},
  ];
  for (var index = 0; index < invalidAcknowledgements.length; ++index) {
    testWidgets(
      'invalid Metal retirement acknowledgement $index preserves pending state',
      (tester) async {
        final f = await fixture(
          tester,
          timeout: const Duration(milliseconds: 100),
        );
        try {
          await tester.pumpWidget(viewport(f.controller));
          await until(tester, () => f.controller.canDisplay);
          f.plugin.retireAckOverrides = invalidAcknowledgements[index];
          final closing = f.controller.close();
          await tester.pump(const Duration(milliseconds: 140));
          await closing;
          expect(f.controller.retirementPending, isTrue);
          expect(f.controller.phase, NativeTexturePhase.retirePending);
          expect(f.plugin.retireAcks, hasLength(1));
          await tester.pump(const Duration(milliseconds: 200));
          expect(f.plugin.retireAcks, hasLength(1));
          expect(f.metadata.released, isEmpty);
        } finally {
          await f.close(tester);
        }
      },
    );
  }

  for (final code in ['retirePending', 'closed']) {
    testWidgets(
      'Metal retireAck $code failure is bounded without an automatic retry',
      (tester) async {
        final f = await fixture(
          tester,
          timeout: const Duration(milliseconds: 100),
        );
        try {
          await tester.pumpWidget(viewport(f.controller));
          await until(tester, () => f.controller.canDisplay);
          f.plugin.retireAckError = code;
          final closing = f.controller.close();
          await tester.pump(const Duration(milliseconds: 140));
          await closing;
          expect(f.controller.retirementPending, isTrue);
          expect(f.controller.retirementDiagnostic, isA<PlatformException>());
          expect(f.plugin.retireAcks, hasLength(1));
          await tester.pump(const Duration(milliseconds: 200));
          expect(f.plugin.retireAcks, hasLength(1));
          expect(f.metadata.released, isEmpty);
        } finally {
          await f.close(tester);
        }
      },
    );
  }

  testWidgets(
    'a late original retireAck resolves bounded pending retirement without resending',
    (tester) async {
      final f = await fixture(
        tester,
        timeout: const Duration(milliseconds: 100),
      );
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.canDisplay);
        final gate = Completer<void>();
        f.plugin.retireAckGate = gate;
        final closing = f.controller.close();
        await until(tester, () => f.plugin.retireAcks.isNotEmpty);
        await tester.pump(const Duration(milliseconds: 150));
        await closing;
        expect(f.controller.retirementPending, isTrue);
        expect(f.plugin.retireAcks, hasLength(1));
        gate.complete();
        await until(
          tester,
          () => f.controller.phase == NativeTexturePhase.closed,
        );
        expect(f.controller.retirementPending, isFalse);
        expect(f.plugin.retireAcks, hasLength(1));
        expect(f.metadata.released, isEmpty);
      } finally {
        await f.close(tester);
      }
    },
  );

  testWidgets(
    'late owner completion sends one bounded retirement acknowledgement after close wait',
    (tester) async {
      final f = await fixture(
        tester,
        timeout: const Duration(milliseconds: 100),
      );
      try {
        await tester.pumpWidget(viewport(f.controller));
        await until(tester, () => f.controller.canDisplay);
        final gate = Completer<void>();
        f.plugin.closeReplyGate = gate;
        final closing = f.controller.close();
        await tester.pump(const Duration(milliseconds: 150));
        await closing;
        expect(f.controller.retirementPending, isTrue);
        expect(f.plugin.retireAcks, isEmpty);
        gate.complete();
        await until(
          tester,
          () => f.controller.phase == NativeTexturePhase.closed,
        );
        expect(f.controller.retirementPending, isFalse);
        expect(f.plugin.retireAcks, hasLength(1));
        expect(f.metadata.released, isEmpty);
      } finally {
        await f.close(tester);
      }
    },
  );
}
