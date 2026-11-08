import 'dart:async';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_document.dart';
import 'package:rhwp_native_reference/src/native_ffi.dart';
import 'package:rhwp_native_reference/src/native_presentation.dart';
import 'package:rhwp_native_reference/src/native_texture_view.dart';
import 'package:rhwp_native_reference/src/native_viewport.dart';

import 'support/fake_gpu_presentation.dart';
import 'support/fake_transport.dart';

NativePageViewport viewport([double width = 300]) =>
    NativePageViewport.fit(const NativePageSize(600, 800), Size(width, 240));

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test('complete stamp comparison includes font/view/device/scale', () {
    final stamp = NativePresentationStamp.fromJson({
      'documentId': 1,
      'revision': 2,
      'fontGeneration': 3,
      'viewGeneration': 4,
      'deviceGeneration': 5,
      'deviceScale': 1.5,
    });
    expect(stamp.matches(stamp.toJson()), isTrue);
    for (final field in [
      'documentId',
      'revision',
      'fontGeneration',
      'viewGeneration',
      'deviceGeneration',
      'deviceScale',
    ]) {
      expect(stamp.matches({...stamp.toJson(), field: 99}), isFalse);
    }
    expect(
      () => NativePresentationStamp.fromJson({
        ...stamp.toJson(),
        'deviceScale': double.nan,
      }),
      throwsA(isA<Exception>()),
    );
  });

  for (final platform in NativeTexturePlatform.values) {
    test(
      '${platform.name} submits metadata and waits for producer publication and full check',
      () async {
        final transport = GpuTransport(),
            metadata = FakePresentation(),
            plugin = FakeTexturePlugin(platform)..publish = false;
        plugin.install();
        final document = await gpuDocument(transport);
        final controller = NativeTextureController(
          document: document,
          presentation: metadata,
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
        );
        addTearDown(() async {
          await controller.close();
          controller.dispose();
          await document.dispose();
          plugin.uninstall();
        });
        controller.configure(viewport(), deviceScale: 1.5);
        await waitFor(() => plugin.submissions.isNotEmpty);
        expect(controller.canDisplay, isFalse);
        expect(metadata.released, isEmpty);
        expect(transport.presentationCalls, ['configure', 'capture']);
        final command = plugin.submissions.single;
        expect(command.keys.toSet(), {
          'textureId',
          'stamp',
          'width',
          'height',
          'clearColor',
          'placements',
        });
        expect(command['width'], 450);
        expect((command['placements']! as List).single, {
          'snapshot': 101,
          'scale': .75,
          'originX': 0.0,
          'originY': 0.0,
          'backgroundColor': 0xffffffff,
        });
        plugin.publish = true;
        await waitFor(() => controller.canDisplay);
        expect(metadata.checks.length, greaterThanOrEqualTo(2));
        await controller.close();
        expect(metadata.released, isEmpty);
        expect(plugin.releasedByPlugin, {101});
      },
    );
  }
  test(
    'no full-stamp capability fails closed before platform create/capture',
    () async {
      final transport = GpuTransport(),
          metadata = FakePresentation()..available = false;
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final document = await gpuDocument(transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
      );
      controller.configure(viewport(), deviceScale: 1);
      await waitFor(() => controller.needsCpuFallback);
      expect(plugin.calls, isEmpty);
      expect(transport.snapshots, isEmpty);
      await controller.close();
      controller.dispose();
      await document.dispose();
      plugin.uninstall();
    },
  );
  test(
    'accepted mutation hides immediately and queues new capture behind it',
    () async {
      final transport = GpuTransport(), metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final document = await gpuDocument(transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      addTearDown(() async {
        await controller.close();
        controller.dispose();
        await document.dispose();
        plugin.uninstall();
      });
      controller.configure(viewport(), deviceScale: 1);
      await waitFor(() => controller.canDisplay);
      final old = document.info!.documentId;
      final opened = document.newDocument();
      expect(controller.canDisplay, isFalse);
      await opened;
      await waitFor(() => controller.canDisplay);
      expect(document.info!.documentId, isNot(old));
      expect(
        (plugin.submissions.last['stamp']! as Map)['documentId'],
        document.info!.documentId,
      );
    },
  );
  test(
    'resize while capture is pending releases the old capture exactly once',
    () async {
      final transport = GpuTransport()..autoSnapshots = false,
          metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final document = await gpuDocument(transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      addTearDown(() async {
        await controller.close();
        controller.dispose();
        await document.dispose();
        plugin.uninstall();
      });
      controller.configure(viewport(), deviceScale: 1);
      await waitFor(() => transport.waitingSnapshots.isNotEmpty);
      controller.configure(viewport(320), deviceScale: 1);
      transport.completeSnapshot();
      await waitFor(() => metadata.released.contains(101));
      await waitFor(() => transport.waitingSnapshots.isNotEmpty);
      transport.completeSnapshot();
      await waitFor(() => controller.canDisplay);
      expect(metadata.released, [101]);
      expect(plugin.acceptedIds, [102]);
    },
  );
  test(
    'dispose before a capture arrives preserves its cleanup callback',
    () async {
      final transport = GpuTransport()..autoSnapshots = false,
          metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final document = await gpuDocument(transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
      );
      controller.configure(viewport(), deviceScale: 1);
      await waitFor(() => transport.waitingSnapshots.isNotEmpty);
      await controller.close();
      controller.dispose();
      transport.completeSnapshot();
      await waitFor(() => metadata.released.contains(101));
      expect(plugin.submissions, isEmpty);
      expect(NativeTextureController.retainedOwnedSnapshots, 0);
      await document.dispose();
      plugin.uninstall();
    },
  );
  test(
    'definite backpressure releases caller ownership and selects CPU',
    () async {
      final transport = GpuTransport(), metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.macos)
        ..submitError = 'backpressure';
      plugin.install();
      final document = await gpuDocument(transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.macos,
        pollInterval: const Duration(milliseconds: 10),
      );
      controller.configure(viewport(), deviceScale: 1);
      await waitFor(
        () => controller.needsCpuFallback && metadata.released.isNotEmpty,
      );
      expect(metadata.released, [101]);
      expect(plugin.owned, isEmpty);
      await controller.close();
      controller.dispose();
      await document.dispose();
      plugin.uninstall();
    },
  );
  test(
    'unknown reply after acceptance reconciles receipt without double release',
    () async {
      final transport = GpuTransport(), metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.windows)
        ..acceptedBeforeReply = true
        ..submitError = 'nativeError';
      plugin.install();
      final document = await gpuDocument(transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.windows,
        pollInterval: const Duration(milliseconds: 10),
      );
      controller.configure(viewport(), deviceScale: 1);
      await waitFor(() => controller.phase == NativeTexturePhase.failed);
      expect(metadata.released, isEmpty);
      expect(NativeTextureController.retainedOwnedSnapshots, 0);
      await controller.close();
      controller.dispose();
      await document.dispose();
      plugin.uninstall();
    },
  );
  test(
    'timed-out ambiguous submit stays bounded until late closing rejection',
    () async {
      final transport = GpuTransport(), metadata = FakePresentation();
      final gate = Completer<void>();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)
        ..submitGate = gate;
      plugin.install();
      final document = await gpuDocument(transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
        operationTimeout: const Duration(milliseconds: 100),
      );
      controller.configure(viewport(), deviceScale: 1);
      await waitFor(() => controller.needsCpuFallback);
      expect(metadata.released, isEmpty);
      expect(NativeTextureController.retainedOwnedSnapshots, 1);
      gate.complete();
      await waitFor(() => metadata.released.contains(101));
      expect(NativeTextureController.retainedOwnedSnapshots, 0);
      await controller.close();
      controller.dispose();
      await document.dispose();
      plugin.uninstall();
    },
  );
  test('strict document/resource error remains a diagnostic', () async {
    final transport = GpuTransport(), metadata = FakePresentation();
    final plugin = FakeTexturePlugin(NativeTexturePlatform.windows)
      ..statusError = 'documentError';
    plugin.install();
    final document = await gpuDocument(transport);
    final controller = NativeTextureController(
      document: document,
      presentation: metadata,
      platform: NativeTexturePlatform.windows,
      pollInterval: const Duration(milliseconds: 10),
    );
    controller.configure(viewport(), deviceScale: 1);
    await waitFor(() => controller.phase == NativeTexturePhase.failed);
    expect(controller.needsCpuFallback, isFalse);
    expect(controller.canDisplay, isFalse);
    expect(controller.diagnostic, isA<NativeCallException>());
    expect(
      (controller.diagnostic! as NativeCallException).code,
      'documentError',
    );
    await controller.close();
    controller.dispose();
    await document.dispose();
    plugin.uninstall();
  });
  test('a viewport takeover never leaves Texture visible', () async {
    final transport = GpuTransport(), metadata = FakePresentation();
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final document = await gpuDocument(transport);
    final controller = NativeTextureController(
      document: document,
      presentation: metadata,
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
    );
    controller.configure(viewport(), deviceScale: 1);
    await waitFor(() => controller.canDisplay);
    document.beginPresentation();
    expect(controller.canDisplay, isFalse);
    await waitFor(() => controller.needsCpuFallback);
    await controller.close();
    controller.dispose();
    await document.dispose();
    plugin.uninstall();
  });
  test(
    'malicious viewport is rejected before capture or platform allocation',
    () async {
      final transport = GpuTransport(), metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final document = await gpuDocument(transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
      );
      controller.configure(viewport(1e30), deviceScale: 16);
      expect(controller.needsCpuFallback, isTrue);
      expect(plugin.calls, isEmpty);
      expect(transport.snapshots, isEmpty);
      await controller.close();
      controller.dispose();
      await document.dispose();
      plugin.uninstall();
    },
  );
  test(
    'canonical owner module identity is required before allocation',
    () async {
      final metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      for (final transport in <NativeWorkerTransport>[
        GpuTransport()..ownerLibraryPath = '/different/native.so',
        FakeTransport()..capabilities['ownerPresentationCleanup'] = true,
      ]) {
        final document = await NativeDocument.connect(transport);
        await document.newDocument();
        final controller = NativeTextureController(
          document: document,
          presentation: metadata,
          platform: NativeTexturePlatform.linux,
        );
        controller.configure(viewport(), deviceScale: 1);
        await waitFor(() => controller.phase == NativeTexturePhase.failed);
        expect(
          (controller.diagnostic! as NativeCallException).code,
          'nativeModuleMismatch',
        );
        expect(plugin.calls, isEmpty);
        await controller.close();
        controller.dispose();
        await document.dispose();
      }
      plugin.uninstall();
    },
  );
  test('false full stamp before submit releases the caller snapshot', () async {
    final transport = GpuTransport(),
        metadata = FakePresentation()..current = false;
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final document = await gpuDocument(transport);
    final controller = NativeTextureController(
      document: document,
      presentation: metadata,
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
    );
    controller.configure(viewport(), deviceScale: 1);
    await waitFor(
      () => controller.needsCpuFallback && metadata.released.isNotEmpty,
    );
    expect(metadata.released, [101]);
    expect(plugin.submissions, isEmpty);
    expect(controller.canDisplay, isFalse);
    await controller.close();
    controller.dispose();
    await document.dispose();
    plugin.uninstall();
  });
  test('false full stamp after publication covers without releasing plugin ownership', () async {
    final transport = GpuTransport(), metadata = FakePresentation();
    final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
    final document = await gpuDocument(transport);
    final controller = NativeTextureController(
      document: document,
      presentation: metadata,
      platform: NativeTexturePlatform.linux,
      pollInterval: const Duration(milliseconds: 10),
    );
    controller.configure(viewport(), deviceScale: 1);
    await waitFor(() => controller.canDisplay);
    document
        .beginView(); // The host may dispose its covered CPU reference view.
    await Future<void>.delayed(const Duration(milliseconds: 30));
    expect(controller.canDisplay, isTrue);
    metadata.current = false;
    await waitFor(() => controller.needsCpuFallback);
    expect(controller.canDisplay, isFalse);
    expect(metadata.released, isEmpty);
    expect(plugin.releasedByPlugin, {101});
    await controller.close();
    controller.dispose();
    await document.dispose();
    plugin.uninstall();
  });
  test(
    'capture timeout retains credit until a late snapshot can be released',
    () async {
      final transport = GpuTransport()..autoSnapshots = false,
          metadata = FakePresentation();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)..install();
      final document = await gpuDocument(transport);
      final controller = NativeTextureController(
        document: document,
        presentation: metadata,
        platform: NativeTexturePlatform.linux,
        pollInterval: const Duration(milliseconds: 10),
        operationTimeout: const Duration(milliseconds: 100),
      );
      controller.configure(viewport(), deviceScale: 1);
      await waitFor(() => controller.needsCpuFallback);
      expect(NativeTextureController.retainedOwnedSnapshots, 1);
      expect(metadata.released, isEmpty);
      transport.completeSnapshot();
      await waitFor(() => metadata.released.isNotEmpty);
      expect(metadata.released, [101]);
      expect(NativeTextureController.retainedOwnedSnapshots, 0);
      expect(plugin.submissions, isEmpty);
      await controller.close();
      controller.dispose();
      await document.dispose();
      plugin.uninstall();
    },
  );
  test(
    'timed out channel futures continue to consume the eight RPC credits',
    () async {
      final gate = Completer<void>();
      final plugin = FakeTexturePlugin(NativeTexturePlatform.linux)
        ..createGate = gate;
      plugin.install();
      final documents = <NativeDocument>[];
      final controllers = <NativeTextureController>[];
      addTearDown(() async {
        if (!gate.isCompleted) gate.complete();
        for (final controller in controllers) {
          await controller.close();
          controller.dispose();
        }
        for (final document in documents) {
          await document.dispose();
        }
        plugin.uninstall();
      });
      for (var index = 0; index < 9; ++index) {
        if (index == 8) {
          await waitFor(
            () => controllers.every((value) => value.needsCpuFallback),
          );
          expect(plugin.calls.where((value) => value == 'create').length, 8);
        }
        final document = await gpuDocument(GpuTransport());
        documents.add(document);
        final controller = NativeTextureController(
          document: document,
          presentation: FakePresentation(),
          platform: NativeTexturePlatform.linux,
          pollInterval: const Duration(milliseconds: 10),
          operationTimeout: const Duration(milliseconds: 100),
        );
        controllers.add(controller);
        controller.configure(viewport(), deviceScale: 1);
      }
      await waitFor(() => controllers.last.needsCpuFallback);
      expect(plugin.calls.where((value) => value == 'create').length, 8);
      expect(
        (controllers.last.diagnostic! as NativeCallException).code,
        'textureRpcLimit',
      );
      gate.complete();
      await waitFor(
        () => plugin.calls.where((value) => value == 'close').length == 8,
      );
      expect(NativeTextureController.retainedOwnedSnapshots, 0);
    },
  );
}
