import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_document.dart';
import 'package:rhwp_native_reference/src/native_texture_view.dart';
import 'package:rhwp_native_reference/src/native_viewport.dart';

import 'support/fake_gpu_presentation.dart';

class WindowsTransferPlugin extends FakeTexturePlugin {
  WindowsTransferPlugin() : super(NativeTexturePlatform.windows);
  final creates = <Map<String, Object?>>[];
  bool negotiate = true, rejectForever = false;
  int rejections = 0;
  String rejectionCode = 'backpressure';
  Object? rejectionDetails = const {'backpressureRetryable': true};
  String _mode = 'legacyImmutable';

  @override
  Map<String, Object?> status() => {
    ...super.status(),
    'publicationMode': _mode,
    'limits': {
      ...super.status()['limits']! as Map,
      'continuousPresentationSupported': _mode == 'immutableOwnershipTransfer',
    },
  };

  @override
  Future<Object?> handle(MethodCall call) async {
    final args = Map<String, Object?>.from(call.arguments as Map? ?? const {});
    if (call.method == 'create') {
      creates.add(args);
      _mode =
          negotiate && args['publicationMode'] == 'immutableOwnershipTransfer'
          ? 'immutableOwnershipTransfer'
          : 'legacyImmutable';
    }
    if (call.method == 'submit' && (rejectForever || rejections > 0)) {
      if (rejections > 0) --rejections;
      calls.add('submit');
      submissions.add(args);
      throw PlatformException(code: rejectionCode, details: rejectionDetails);
    }
    return super.handle(call);
  }
}

class ReleaseGatePresentation extends FakePresentation {
  final attempts = <int>[];
  final gate = Completer<void>();
  @override
  Future<void> releaseSnapshot(int snapshot) async {
    attempts.add(snapshot);
    await gate.future;
    await super.releaseSnapshot(snapshot);
  }
}

class NegotiatedClosePlugin extends FakeTexturePlugin {
  NegotiatedClosePlugin(super.platform);
  bool topEcho = true, limitsEcho = true, requested = false;
  final requests = <({String method, Map<String, Object?> arguments})>[];
  bool get negotiated => requested && topEcho && limitsEcho;
  @override
  Map<String, Object?> status() => {
    ...super.status(),
    if (topEcho) 'closeAcknowledgementRequired': requested,
    'limits': {
      ...super.status()['limits']! as Map,
      if (limitsEcho) 'closeAcknowledgementRequired': requested,
    },
  };
  @override
  Future<Object?> handle(MethodCall call) async {
    final args = Map<String, Object?>.from(call.arguments as Map? ?? const {});
    requests.add((method: call.method, arguments: args));
    if (call.method == 'create') {
      requested = args['closeAcknowledgementRequired'] == true;
    } else if (negotiated && args['deviceGeneration'] != 1) {
      throw PlatformException(code: 'contextChanged');
    }
    return super.handle(call);
  }
}

Future<void> until(WidgetTester tester, bool Function() ready) async {
  for (var index = 0; index < 100 && !ready(); ++index) {
    await tester.pump(const Duration(milliseconds: 10));
  }
  expect(ready(), isTrue);
}

Future<NativeDocument> document(
  WidgetTester tester,
  GpuTransport transport,
) async {
  final result = await NativeDocument.connect(transport);
  final opening = result.newDocument();
  await until(tester, () => result.info != null);
  await opening;
  return result;
}

Widget viewport(NativeTextureController controller) => MaterialApp(
  home: MediaQuery(
    data: const MediaQueryData(devicePixelRatio: 1),
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
  testWidgets('Windows transfer mode is explicit and must be negotiated', (
    tester,
  ) async {
    final plugin = WindowsTransferPlugin()..install();
    final owner = await document(tester, GpuTransport());
    final controller = NativeTextureController(
      document: owner,
      presentation: FakePresentation(),
      platform: NativeTexturePlatform.windows,
      allowImmutableOwnershipTransfer: true,
      pollInterval: const Duration(milliseconds: 10),
    );
    try {
      await tester.pumpWidget(viewport(controller));
      await until(tester, () => controller.canDisplay);
      expect(
        plugin.creates.single['publicationMode'],
        'immutableOwnershipTransfer',
      );
      expect(plugin.submissions, hasLength(1));
    } finally {
      await cleanup(tester, controller, owner, plugin);
    }
  });
  testWidgets(
    'transfer opt-in rejects other texture platforms before registration',
    (tester) async {
      final owner = await document(tester, GpuTransport());
      for (final platform in [
        NativeTexturePlatform.linux,
        NativeTexturePlatform.macos,
      ]) {
        expect(
          () => NativeTextureController(
            document: owner,
            presentation: FakePresentation(),
            platform: platform,
            allowImmutableOwnershipTransfer: true,
          ),
          throwsArgumentError,
        );
      }
      await owner.dispose();
    },
  );
  testWidgets('an older Windows plugin cannot silently select legacy mode', (
    tester,
  ) async {
    final plugin = WindowsTransferPlugin()..negotiate = false;
    plugin.install();
    final transport = GpuTransport(), metadata = FakePresentation();
    final owner = await document(tester, transport);
    final controller = NativeTextureController(
      document: owner,
      presentation: metadata,
      platform: NativeTexturePlatform.windows,
      allowImmutableOwnershipTransfer: true,
      pollInterval: const Duration(milliseconds: 10),
    );
    try {
      await tester.pumpWidget(viewport(controller));
      await until(tester, () => controller.needsCpuFallback);
      expect(transport.snapshots, isEmpty);
      expect(plugin.submissions, isEmpty);
      expect(plugin.calls.where((name) => name == 'close'), hasLength(1));
    } finally {
      await cleanup(tester, controller, owner, plugin);
    }
  });
  testWidgets(
    'preaccept backpressure releases once before a fresh bounded retry',
    (tester) async {
      final plugin = WindowsTransferPlugin()..rejections = 1;
      plugin.install();
      final transport = GpuTransport(), metadata = ReleaseGatePresentation();
      final owner = await document(tester, transport);
      final controller = NativeTextureController(
        document: owner,
        presentation: metadata,
        platform: NativeTexturePlatform.windows,
        allowImmutableOwnershipTransfer: true,
        pollInterval: const Duration(milliseconds: 10),
      );
      try {
        await tester.pumpWidget(viewport(controller));
        await until(tester, () => metadata.attempts.isNotEmpty);
        await tester.pump(const Duration(milliseconds: 60));
        expect(plugin.submissions, hasLength(1));
        expect(transport.snapshots, [101]);
        expect(metadata.attempts, [101]);
        expect(controller.canDisplay, isFalse);
        metadata.gate.complete();
        await until(tester, () => controller.canDisplay);
        expect(transport.snapshots, [101, 102]);
        expect(metadata.released, [101]);
        expect(plugin.submissions, hasLength(2));
        final first = plugin.submissions.first['stamp']! as Map;
        final second = plugin.submissions.last['stamp']! as Map;
        expect(
          second['viewGeneration'] as int,
          greaterThan(first['viewGeneration'] as int),
        );
        expect(plugin.owned, {102});
      } finally {
        if (!metadata.gate.isCompleted) metadata.gate.complete();
        await cleanup(tester, controller, owner, plugin);
      }
      expect(NativeTextureController.retainedOwnedSnapshots, 0);
    },
  );
  testWidgets('retry admission has one deadline across all rejected captures', (
    tester,
  ) async {
    final plugin = WindowsTransferPlugin()..rejectForever = true;
    plugin.install();
    final transport = GpuTransport(), metadata = FakePresentation();
    final owner = await document(tester, transport);
    final controller = NativeTextureController(
      document: owner,
      presentation: metadata,
      platform: NativeTexturePlatform.windows,
      allowImmutableOwnershipTransfer: true,
      pollInterval: const Duration(milliseconds: 10),
      operationTimeout: const Duration(milliseconds: 100),
    );
    try {
      await tester.pumpWidget(viewport(controller));
      await until(tester, () => controller.needsCpuFallback);
      expect(plugin.submissions.length, inInclusiveRange(1, 10));
      await tester.pump(const Duration(milliseconds: 30));
      final calls = plugin.calls.length;
      await tester.pump(const Duration(seconds: 1));
      expect(plugin.calls.length, calls);
      expect(metadata.released.toSet(), transport.snapshots.toSet());
      expect(metadata.released.length, metadata.released.toSet().length);
    } finally {
      await cleanup(tester, controller, owner, plugin);
    }
    expect(NativeTextureController.retainedOwnedSnapshots, 0);
  });
  final invalidRetryDetails = <Object?>[
    null,
    {'backpressureRetryable': false},
    {'backpressureRetryable': 'true'},
    'retry',
  ];
  for (var index = 0; index < invalidRetryDetails.length; ++index) {
    testWidgets(
      'backpressure without an explicit true retry receipt $index stays terminal',
      (tester) async {
        final plugin = WindowsTransferPlugin()
          ..rejections = 1
          ..rejectionDetails = invalidRetryDetails[index];
        plugin.install();
        final metadata = FakePresentation();
        final owner = await document(tester, GpuTransport());
        final controller = NativeTextureController(
          document: owner,
          presentation: metadata,
          platform: NativeTexturePlatform.windows,
          allowImmutableOwnershipTransfer: true,
          pollInterval: const Duration(milliseconds: 10),
        );
        try {
          await tester.pumpWidget(viewport(controller));
          await until(tester, () => controller.needsCpuFallback);
          await tester.pump(const Duration(milliseconds: 100));
          expect(plugin.submissions, hasLength(1));
          expect(metadata.released, [101]);
        } finally {
          await cleanup(tester, controller, owner, plugin);
        }
      },
    );
  }
  testWidgets(
    'default Windows mode neither opts in nor retries legacy backpressure',
    (tester) async {
      final plugin = WindowsTransferPlugin()..rejections = 1;
      plugin.install();
      final owner = await document(tester, GpuTransport());
      final controller = NativeTextureController(
        document: owner,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.windows,
        pollInterval: const Duration(milliseconds: 10),
      );
      try {
        await tester.pumpWidget(viewport(controller));
        await until(tester, () => controller.needsCpuFallback);
        expect(plugin.creates.single.containsKey('publicationMode'), isFalse);
        expect(plugin.submissions, hasLength(1));
      } finally {
        await cleanup(tester, controller, owner, plugin);
      }
    },
  );
  testWidgets(
    'oversized transfer target rejects definitively without retry or retained IDs',
    (tester) async {
      final plugin = WindowsTransferPlugin()
        ..rejections = 1
        ..rejectionCode = 'targetTooLarge';
      plugin.install();
      final metadata = FakePresentation();
      final owner = await document(tester, GpuTransport());
      final controller = NativeTextureController(
        document: owner,
        presentation: metadata,
        platform: NativeTexturePlatform.windows,
        allowImmutableOwnershipTransfer: true,
        pollInterval: const Duration(milliseconds: 10),
      );
      try {
        await tester.pumpWidget(viewport(controller));
        await until(tester, () => controller.needsCpuFallback);
        expect(plugin.submissions, hasLength(1));
        expect(metadata.released, [101]);
        expect(NativeTextureController.retainedOwnedSnapshots, 0);
      } finally {
        await cleanup(tester, controller, owner, plugin);
      }
    },
  );
  testWidgets(
    'an accepted producer status error never replays the accepted snapshot',
    (tester) async {
      final plugin = WindowsTransferPlugin()..publish = false;
      plugin.install();
      final metadata = FakePresentation();
      final owner = await document(tester, GpuTransport());
      final controller = NativeTextureController(
        document: owner,
        presentation: metadata,
        platform: NativeTexturePlatform.windows,
        allowImmutableOwnershipTransfer: true,
        pollInterval: const Duration(milliseconds: 10),
      );
      try {
        await tester.pumpWidget(viewport(controller));
        await until(tester, () => plugin.owned.isNotEmpty);
        plugin.statusError = 'backpressure';
        await until(tester, () => controller.needsCpuFallback);
        expect(plugin.submissions, hasLength(1));
        expect(metadata.released, isEmpty);
        expect(plugin.releasedByPlugin, {101});
      } finally {
        await cleanup(tester, controller, owner, plugin);
      }
    },
  );
  testWidgets(
    'accepted then ambiguous transport failure reconciles without retry',
    (tester) async {
      final plugin = WindowsTransferPlugin()
        ..acceptedBeforeReply = true
        ..submitError = 'transportLost';
      plugin.install();
      final metadata = FakePresentation();
      final owner = await document(tester, GpuTransport());
      final controller = NativeTextureController(
        document: owner,
        presentation: metadata,
        platform: NativeTexturePlatform.windows,
        allowImmutableOwnershipTransfer: true,
        pollInterval: const Duration(milliseconds: 10),
      );
      try {
        await tester.pumpWidget(viewport(controller));
        await until(
          tester,
          () => controller.phase == NativeTexturePhase.failed,
        );
        expect(plugin.submissions, hasLength(1));
        expect(metadata.released, isEmpty);
        expect(plugin.releasedByPlugin, {101});
        expect(NativeTextureController.retainedOwnedSnapshots, 0);
      } finally {
        await cleanup(tester, controller, owner, plugin);
      }
    },
  );
  testWidgets(
    'an admission timeout reconciles accepted ownership before late RPC completion',
    (tester) async {
      final gate = Completer<void>();
      final plugin = WindowsTransferPlugin()
        ..acceptedBeforeReply = true
        ..submitGate = gate;
      plugin.install();
      final metadata = FakePresentation();
      final owner = await document(tester, GpuTransport());
      final controller = NativeTextureController(
        document: owner,
        presentation: metadata,
        platform: NativeTexturePlatform.windows,
        allowImmutableOwnershipTransfer: true,
        pollInterval: const Duration(milliseconds: 10),
        operationTimeout: const Duration(milliseconds: 100),
      );
      try {
        await tester.pumpWidget(viewport(controller));
        await until(tester, () => plugin.submissions.isNotEmpty);
        await tester.pump(const Duration(milliseconds: 150));
        expect(controller.needsCpuFallback, isTrue);
        expect(metadata.released, isEmpty);
        // The accepted ledger resolves ownership while the original channel
        // future still retains its RPC credit until its real completion.
        expect(NativeTextureController.retainedOwnedSnapshots, 0);
        expect(plugin.releasedByPlugin, {101});
        gate.complete();
        await tester.pump(const Duration(milliseconds: 30));
        expect(plugin.submissions, hasLength(1));
        expect(metadata.released, isEmpty);
        expect(plugin.releasedByPlugin, {101});
        expect(NativeTextureController.retainedOwnedSnapshots, 0);
      } finally {
        if (!gate.isCompleted) gate.complete();
        await cleanup(tester, controller, owner, plugin);
      }
    },
  );

  for (final platform in [
    NativeTexturePlatform.windows,
    NativeTexturePlatform.macos,
  ]) {
    testWidgets(
      '${platform.name} explicit close preserves completed automatic fallback retirement',
      (tester) async {
        final plugin = NegotiatedClosePlugin(platform)
          ..submitError = 'contextChanged';
        plugin.install();
        final metadata = FakePresentation();
        final owner = await document(tester, GpuTransport());
        final controller = NativeTextureController(
          document: owner,
          presentation: metadata,
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
        );
        try {
          await tester.pumpWidget(viewport(controller));
          await until(tester, () => plugin.retireAcks.length == 1);
          await tester.pump(const Duration(milliseconds: 20));
          expect(controller.needsCpuFallback, isTrue);
          expect(controller.retirementPending, isFalse);
          final closing = controller.close();
          await tester.pump(const Duration(milliseconds: 30));
          await closing;
          expect(controller.phase, NativeTexturePhase.closed);
          expect(controller.retirementPending, isFalse);
          expect(
            plugin.calls.where((method) => method == 'close'),
            hasLength(1),
          );
          expect(plugin.retireAcks, hasLength(1));
          expect(metadata.released, [101]);
        } finally {
          await cleanup(tester, controller, owner, plugin);
        }
      },
    );

    testWidgets(
      '${platform.name} automatic fallback keeps an unconfirmed acknowledgement pending through explicit close',
      (tester) async {
        final gate = Completer<void>();
        final plugin = NegotiatedClosePlugin(platform)
          ..submitError = 'contextChanged'
          ..retireAckGate = gate;
        plugin.install();
        final owner = await document(tester, GpuTransport());
        final controller = NativeTextureController(
          document: owner,
          presentation: FakePresentation(),
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
          operationTimeout: const Duration(milliseconds: 100),
        );
        try {
          await tester.pumpWidget(viewport(controller));
          await until(tester, () => plugin.retireAcks.length == 1);
          final closing = controller.close();
          await tester.pump(const Duration(milliseconds: 150));
          await closing;
          expect(controller.retirementPending, isTrue);
          expect(controller.phase, NativeTexturePhase.retirePending);
          gate.complete();
          await until(
            tester,
            () => controller.phase == NativeTexturePhase.closed,
          );
          expect(controller.retirementPending, isFalse);
          expect(plugin.retireAcks, hasLength(1));
        } finally {
          if (!gate.isCompleted) gate.complete();
          await cleanup(tester, controller, owner, plugin);
        }
      },
    );

    testWidgets(
      '${platform.name} negotiates generation-checked retirement independently of publication mode',
      (tester) async {
        final plugin = NegotiatedClosePlugin(platform)..install();
        final metadata = FakePresentation();
        final owner = await document(tester, GpuTransport());
        final controller = NativeTextureController(
          document: owner,
          presentation: metadata,
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
        );
        try {
          await tester.pumpWidget(viewport(controller));
          await until(tester, () => controller.canDisplay);
          expect(
            plugin.requests.first.arguments['closeAcknowledgementRequired'],
            isTrue,
          );
          final gate = Completer<void>();
          plugin.retireAckGate = gate;
          plugin.retireAckOverrides = {'nativeViewLayerDetached': false};
          var complete = false;
          final closing = controller.close().then((_) {
            complete = true;
          });
          await until(tester, () => plugin.retireAcks.isNotEmpty);
          expect(plugin.retireAcks.single, {
            'textureId': controller.textureId,
            'deviceGeneration': 1,
          });
          expect(complete, isFalse);
          gate.complete();
          await until(tester, () => complete);
          await closing;
          expect(controller.retirementPending, isFalse);
          expect(
            plugin.requests
                .where((request) => request.method != 'create')
                .every((request) => request.arguments['deviceGeneration'] == 1),
            isTrue,
          );
          expect(plugin.retireAcks, hasLength(1));
          expect(metadata.released, isEmpty);
        } finally {
          if (plugin.retireAckGate case final gate? when !gate.isCompleted) {
            gate.complete();
          }
          await cleanup(tester, controller, owner, plugin);
        }
      },
    );

    testWidgets(
      '${platform.name} rejects an acknowledgement for another device generation',
      (tester) async {
        final plugin = NegotiatedClosePlugin(platform)..install();
        final owner = await document(tester, GpuTransport());
        final controller = NativeTextureController(
          document: owner,
          presentation: FakePresentation(),
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
          operationTimeout: const Duration(milliseconds: 100),
        );
        try {
          await tester.pumpWidget(viewport(controller));
          await until(tester, () => controller.canDisplay);
          plugin.retireAckOverrides = {'deviceGeneration': 2};
          final closing = controller.close();
          await tester.pump(const Duration(milliseconds: 150));
          await closing;
          expect(controller.retirementPending, isTrue);
          expect(plugin.retireAcks, hasLength(1));
          await tester.pump(const Duration(milliseconds: 200));
          expect(plugin.retireAcks, hasLength(1));
        } finally {
          await cleanup(tester, controller, owner, plugin);
        }
      },
    );

    for (final echoes in [(false, true), (true, false), (false, false)]) {
      testWidgets(
        '${platform.name} incomplete close negotiation $echoes preserves the old plugin path',
        (tester) async {
          final plugin = NegotiatedClosePlugin(platform)
            ..topEcho = echoes.$1
            ..limitsEcho = echoes.$2;
          plugin.install();
          final owner = await document(tester, GpuTransport());
          final controller = NativeTextureController(
            document: owner,
            presentation: FakePresentation(),
            platform: platform,
            pollInterval: const Duration(milliseconds: 10),
          );
          try {
            await tester.pumpWidget(viewport(controller));
            await until(tester, () => controller.canDisplay);
            final closing = controller.close();
            await tester.pump(const Duration(milliseconds: 30));
            await closing;
            expect(controller.retirementPending, isFalse);
            expect(plugin.retireAcks, isEmpty);
            expect(
              plugin.requests
                  .where((request) => request.method != 'create')
                  .every(
                    (request) =>
                        !request.arguments.containsKey('deviceGeneration'),
                  ),
              isTrue,
            );
          } finally {
            await cleanup(tester, controller, owner, plugin);
          }
        },
      );
    }

    testWidgets(
      '${platform.name} negotiated retireAck errors stay pending without retries',
      (tester) async {
        final plugin = NegotiatedClosePlugin(platform)..install();
        final owner = await document(tester, GpuTransport());
        final controller = NativeTextureController(
          document: owner,
          presentation: FakePresentation(),
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
          operationTimeout: const Duration(milliseconds: 100),
        );
        try {
          await tester.pumpWidget(viewport(controller));
          await until(tester, () => controller.canDisplay);
          plugin.retireAckError = 'retirePending';
          final closing = controller.close();
          await tester.pump(const Duration(milliseconds: 150));
          await closing;
          expect(controller.retirementPending, isTrue);
          expect(plugin.retireAcks, hasLength(1));
          await tester.pump(const Duration(milliseconds: 200));
          expect(plugin.retireAcks, hasLength(1));
        } finally {
          await cleanup(tester, controller, owner, plugin);
        }
      },
    );

    testWidgets(
      '${platform.name} original late acknowledgement settles retirement without replay',
      (tester) async {
        final plugin = NegotiatedClosePlugin(platform)..install();
        final owner = await document(tester, GpuTransport());
        final controller = NativeTextureController(
          document: owner,
          presentation: FakePresentation(),
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
          operationTimeout: const Duration(milliseconds: 100),
        );
        try {
          await tester.pumpWidget(viewport(controller));
          await until(tester, () => controller.canDisplay);
          final gate = Completer<void>();
          plugin.retireAckGate = gate;
          final closing = controller.close();
          await until(tester, () => plugin.retireAcks.isNotEmpty);
          await tester.pump(const Duration(milliseconds: 150));
          await closing;
          expect(controller.retirementPending, isTrue);
          gate.complete();
          await until(
            tester,
            () => controller.phase == NativeTexturePhase.closed,
          );
          expect(controller.retirementPending, isFalse);
          expect(plugin.retireAcks, hasLength(1));
        } finally {
          if (plugin.retireAckGate case final gate? when !gate.isCompleted) {
            gate.complete();
          }
          await cleanup(tester, controller, owner, plugin);
        }
      },
    );

    testWidgets(
      '${platform.name} late create negotiates before its closing observer sends identity',
      (tester) async {
        final gate = Completer<void>();
        final plugin = NegotiatedClosePlugin(platform)..createGate = gate;
        plugin.install();
        final transport = GpuTransport();
        final owner = await document(tester, transport);
        final controller = NativeTextureController(
          document: owner,
          presentation: FakePresentation(),
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
          operationTimeout: const Duration(milliseconds: 100),
        );
        try {
          await tester.pumpWidget(viewport(controller));
          await until(tester, () => plugin.calls.contains('create'));
          final closing = controller.close();
          await tester.pump(const Duration(milliseconds: 150));
          await closing;
          expect(controller.retirementPending, isTrue);
          gate.complete();
          await until(
            tester,
            () => controller.phase == NativeTexturePhase.closed,
          );
          expect(plugin.retireAcks.single['deviceGeneration'], 1);
          expect(
            plugin.requests
                .where((request) => request.method == 'close')
                .single
                .arguments['deviceGeneration'],
            1,
          );
          expect(transport.snapshots, isEmpty);
        } finally {
          if (!gate.isCompleted) gate.complete();
          await cleanup(tester, controller, owner, plugin);
        }
      },
    );
  }

  testWidgets(
    'old Windows plugin automatic fallback remains retired after explicit close',
    (tester) async {
      final plugin = WindowsTransferPlugin()..submitError = 'contextChanged';
      plugin.install();
      final owner = await document(tester, GpuTransport());
      final controller = NativeTextureController(
        document: owner,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.windows,
        pollInterval: const Duration(milliseconds: 10),
      );
      try {
        await tester.pumpWidget(viewport(controller));
        await until(tester, () => plugin.closed);
        await tester.pump(const Duration(milliseconds: 20));
        expect(controller.needsCpuFallback, isTrue);
        final closing = controller.close();
        await tester.pump(const Duration(milliseconds: 30));
        await closing;
        expect(controller.phase, NativeTexturePhase.closed);
        expect(controller.retirementPending, isFalse);
        expect(plugin.retireAcks, isEmpty);
        expect(plugin.calls.where((method) => method == 'close'), hasLength(1));
      } finally {
        await cleanup(tester, controller, owner, plugin);
      }
    },
  );
}
