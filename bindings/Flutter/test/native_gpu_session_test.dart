// Controller/owner lifecycle contracts; no document raster golden is asserted.
import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_document.dart';
import 'package:rhwp_native_reference/src/native_ffi.dart';
import 'package:rhwp_native_reference/src/native_gpu_session.dart';
import 'package:rhwp_native_reference/src/native_texture_view.dart';
import 'package:rhwp_native_reference/src/native_viewport.dart';

import 'support/fake_gpu_presentation.dart';
import 'native_texture_multipage_test.dart'
    as multi
    show MultiTransport, layout;

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
  final document = await NativeDocument.connect(transport);
  final opening = document.newDocument();
  await until(tester, () => document.info != null);
  await opening;
  return document;
}

class _SlowRetirement extends FakeTexturePlugin {
  _SlowRetirement() : super(NativeTexturePlatform.linux);
  bool finished = false;
  @override
  Map<String, Object?> status() => {
    ...super.status(),
    if (closed) 'cleanupComplete': finished,
  };
}

Widget view(NativeGpuSession session) => MaterialApp(
  home: ListenableBuilder(
    listenable: session,
    builder: (context, _) => NativeTextureView(
      controller: session.controller,
      viewport: NativePageViewport.fit(
        const NativePageSize(600, 800),
        const Size(300, 240),
      ),
      cpuFallback: (_) => const Text('CPU fallback'),
    ),
  ),
);

void main() {
  testWidgets(
    'multi-page recovery keeps one document and retires all old IDs',
    (tester) async {
      final transport = multi.MultiTransport();
      final document = await NativeDocument.connect(transport);
      final opening = document.newDocument();
      await until(tester, () => document.info != null);
      await opening;
      final metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      NativeTextureController make() => NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      final session = NativeGpuSession(
        initialController: make(),
        createController: () {
          plugin.closed = false;
          plugin.statusError = null;
          return make();
        },
      );
      await tester.pumpWidget(
        MaterialApp(
          home: ListenableBuilder(
            listenable: session,
            builder: (context, _) => NativeTextureView.document(
              controller: session.controller,
              documentViewport: multi.layout(),
              cpuFallback: (_) => const Text('CPU fallback'),
            ),
          ),
        ),
      );
      await until(tester, () => session.controller.canDisplay);
      final firstIds = plugin.acceptedIds.toSet();
      expect(firstIds, hasLength(2));
      final firstStamp = Map.from(plugin.acceptedStamp!);
      plugin.statusError = 'deviceLost';
      await until(
        tester,
        () => session.controller.phase == NativeTexturePhase.fallback,
      );
      final replacing = session.recover();
      await tester.pump(const Duration(milliseconds: 40));
      expect(await replacing, isTrue);
      await until(tester, () => session.controller.canDisplay);
      expect(plugin.releasedByPlugin, containsAll(firstIds));
      expect(plugin.acceptedIds, hasLength(2));
      expect(plugin.acceptedIds.toSet().intersection(firstIds), isEmpty);
      expect(
        plugin.acceptedStamp!['viewGeneration'] as int,
        greaterThan(firstStamp['viewGeneration'] as int),
      );
      expect(transport.closes, 0);
      expect(metadata.released.toSet().intersection(firstIds), isEmpty);
      expect(NativeTextureController.retainedOwnedSnapshots, 0);
      final closing = session.close();
      await tester.pump(const Duration(milliseconds: 40));
      expect(await closing, isTrue);
      await tester.pumpWidget(const SizedBox.shrink());
      session.dispose();
      await document.dispose();
      plugin.uninstall();
    },
  );

  testWidgets('late close confirmation permits a later successful close', (
    tester,
  ) async {
    final document = await opened(tester, GpuTransport());
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final controller = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
      operationTimeout: const Duration(milliseconds: 80),
    );
    final session = NativeGpuSession(
      initialController: controller,
      createController: () => throw StateError('close must not recreate'),
    );
    await tester.pumpWidget(view(session));
    await until(tester, () => controller.canDisplay);
    plugin.closeReplyGate = Completer<void>();
    final first = session.close();
    await tester.pump(const Duration(milliseconds: 100));
    expect(await first, isFalse);
    expect(controller.retirementPending, isTrue);
    plugin.closeReplyGate!.complete();
    await until(tester, () => controller.phase == NativeTexturePhase.closed);
    expect(controller.retirementPending, isFalse);
    expect(await session.close(), isTrue);
    expect(plugin.calls.where((call) => call == 'close'), hasLength(1));
    await tester.pumpWidget(const SizedBox.shrink());
    session.dispose();
    await document.dispose();
    plugin.uninstall();
  });

  testWidgets('device loss retires before one explicit replacement', (
    tester,
  ) async {
    final transport = GpuTransport();
    final document = await opened(tester, transport);
    final metadata = FakePresentation();
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    NativeTextureController make() => NativeTextureController(
      document: document,
      presentation: metadata,
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
    );
    final original = make();
    var creations = 0;
    NativeTexturePhase? phaseAtCreation;
    bool? pendingAtCreation;
    final session = NativeGpuSession(
      initialController: original,
      createController: () {
        creations++;
        phaseAtCreation = original.phase;
        pendingAtCreation = original.retirementPending;
        plugin.closed = false;
        plugin.statusError = null;
        return make();
      },
    );
    await tester.pumpWidget(view(session));
    await until(tester, () => original.canDisplay);
    final documentId = document.info!.documentId;
    plugin.statusError = 'deviceLost';
    await until(tester, () => original.phase == NativeTexturePhase.fallback);
    // Recovery never starts merely because the plugin reported a failure.
    await tester.pump(const Duration(milliseconds: 50));
    expect(creations, 0);
    final first = session.recover();
    expect(identical(first, session.recover()), isTrue);
    await tester.pump(const Duration(milliseconds: 40));
    expect(await first, isTrue);
    expect(creations, 1);
    expect(phaseAtCreation, NativeTexturePhase.closed);
    expect(pendingAtCreation, isFalse);
    expect(session.controller, isNot(same(original)));
    expect(session.controller.document, same(document));
    await until(tester, () => session.controller.canDisplay);
    expect(document.info!.documentId, documentId);
    expect(transport.closes, 0);
    final closing = session.close();
    expect(identical(closing, session.close()), isTrue);
    await tester.pump(const Duration(milliseconds: 40));
    expect(await closing, isTrue);
    await tester.pumpWidget(const SizedBox.shrink());
    session.dispose();
    await document.dispose();
    plugin.uninstall();
  });

  testWidgets('pending retirement blocks factory and preserves mounted owner', (
    tester,
  ) async {
    final document = await opened(tester, GpuTransport());
    final plugin = _SlowRetirement()..install();
    final original = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
      operationTimeout: const Duration(milliseconds: 80),
    );
    var creations = 0;
    final session = NativeGpuSession(
      initialController: original,
      createController: () {
        creations++;
        throw StateError('must not be reached');
      },
    );
    await tester.pumpWidget(view(session));
    await until(tester, () => original.canDisplay);
    plugin.statusError = 'deviceLost';
    await until(tester, () => original.phase == NativeTexturePhase.fallback);
    final recovery = session.recover();
    await tester.pump(const Duration(milliseconds: 100));
    expect(await recovery, isFalse);
    expect(original.retirementPending, isTrue);
    expect(session.controller, same(original));
    expect(creations, 0);
    expect(document.isDisposed, isFalse);
    final closing = session.close();
    await tester.pump(const Duration(milliseconds: 100));
    expect(await closing, isFalse);
    await tester.pumpWidget(const SizedBox.shrink());
    session.dispose();
    await document.dispose();
    plugin.uninstall();
  });

  testWidgets('healthy and document replay failures are not device recovery', (
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
    var creations = 0;
    final session = NativeGpuSession(
      initialController: controller,
      createController: () {
        creations++;
        throw StateError('must not be reached');
      },
    );
    await tester.pumpWidget(view(session));
    await until(tester, () => controller.canDisplay);
    expect(await session.recover(), isFalse);
    plugin.statusError = 'resourceDecodeFailed';
    await until(tester, () => controller.phase == NativeTexturePhase.failed);
    await tester.pump();
    expect(await session.recover(), isFalse);
    expect(creations, 0);
    expect(find.textContaining('resourceDecodeFailed'), findsOneWidget);
    final closing = session.close();
    await tester.pump(const Duration(milliseconds: 40));
    await closing;
    await tester.pumpWidget(const SizedBox.shrink());
    session.dispose();
    await document.dispose();
    plugin.uninstall();
  });

  testWidgets(
    'factory errors preserve a retired source and allow explicit retry',
    (tester) async {
      final document = await opened(tester, GpuTransport());
      NativeTextureController make() => NativeTextureController(
        document: document,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.linux,
      );
      final original = make();
      await original.close();
      var fail = true;
      final session = NativeGpuSession(
        initialController: original,
        createController: () {
          if (fail) throw StateError('factory failure');
          return make();
        },
      );
      await expectLater(session.recover(), throwsStateError);
      expect(session.controller, same(original));
      expect(session.recoveryDiagnostic, isA<StateError>());
      fail = false;
      expect(await session.recover(), isTrue);
      expect(session.recoveryDiagnostic, isNull);
      expect(session.controller.phase, NativeTexturePhase.initializing);
      expect(await session.close(), isTrue);
      session.dispose();
      await document.dispose();
    },
  );

  testWidgets(
    'foreign factory result does not close another document viewport',
    (tester) async {
      final owner = await opened(tester, GpuTransport());
      final other = await opened(tester, GpuTransport());
      NativeTextureController make(NativeDocument d) => NativeTextureController(
        document: d,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.linux,
      );
      final original = make(owner), foreign = make(other);
      await original.close();
      final session = NativeGpuSession(
        initialController: original,
        createController: () => foreign,
      );
      await expectLater(session.recover(), throwsA(isA<NativeCallException>()));
      expect(foreign.phase, NativeTexturePhase.initializing);
      expect(other.isDisposed, isFalse);
      expect(session.controller, same(original));
      expect(await session.close(), isTrue);
      session.dispose();
      await foreign.close();
      foreign.dispose();
      await owner.dispose();
      await other.dispose();
    },
  );

  testWidgets('close during retirement prevents replacement creation', (
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
    var creations = 0;
    final session = NativeGpuSession(
      initialController: controller,
      createController: () {
        creations++;
        throw StateError('must not be reached');
      },
    );
    await tester.pumpWidget(view(session));
    await until(tester, () => controller.canDisplay);
    plugin.statusError = 'deviceLost';
    await until(tester, () => controller.phase == NativeTexturePhase.fallback);
    plugin.closeReplyGate = Completer<void>();
    final recovery = session.recover();
    final closing = session.close();
    plugin.closeReplyGate!.complete();
    await tester.pump(const Duration(milliseconds: 40));
    expect(await recovery, isFalse);
    expect(await closing, isTrue);
    expect(creations, 0);
    await tester.pumpWidget(const SizedBox.shrink());
    session.dispose();
    await document.dispose();
    plugin.uninstall();
  });
}
