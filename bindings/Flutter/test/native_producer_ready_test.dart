// Synthetic messenger/timing/ownership contracts, never actual GPU evidence.
import 'dart:async';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';
import 'package:rhwp_native_reference/src/native_producer_ready.dart';

import 'support/fake_gpu_presentation.dart';
import 'native_texture_tiles_test.dart' show TilePlugin;

class ReadyPlugin extends FakeTexturePlugin {
  ReadyPlugin(super.platform);
  bool available = true;
  bool rejectListen = false;
  int listens = 0, cancels = 0;
  Completer<void>? frozenGate, cancelGate;
  int get statusCalls => calls.where((call) => call == 'status').length;
  String get eventName => '${platform.channelName}/events';

  @override
  void install() {
    super.install();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(MethodChannel(eventName), (call) async {
          if (call.method == 'listen') ++listens;
          if (call.method == 'listen' && rejectListen) {
            throw PlatformException(code: 'unavailable');
          }
          if (call.method == 'cancel') {
            ++cancels;
            await cancelGate?.future;
          }
          return null;
        });
  }

  @override
  void uninstall() {
    super.uninstall();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(MethodChannel(eventName), null);
  }

  @override
  Future<Object?> handle(MethodCall call) async {
    if (call.method == 'status' && frozenGate != null) {
      calls.add(call.method);
      final captured = status();
      final gate = frozenGate!;
      frozenGate = null;
      await gate.future;
      return captured;
    }
    final result = await super.handle(call);
    if (call.method == 'create') {
      return {...result! as Map, 'producerReadyEvents': available};
    }
    return result;
  }

  Map<String, Object?> event() => {
    'event': 'producerReady',
    'textureId': nextTexture - 1,
    'deviceGeneration': 1,
    'serial': serial,
    'stamp': {...acceptedStamp!},
  };

  Future<void> emit(Object? value) => TestDefaultBinaryMessengerBinding
      .instance
      .defaultBinaryMessenger
      .handlePlatformMessage(
        eventName,
        const StandardMethodCodec().encodeSuccessEnvelope(value),
        (_) {},
      );
}

class ReadyTilePlugin extends TilePlugin {
  ReadyTilePlugin() : super(NativeTexturePlatform.linux);
  @override
  Map<String, Object?> member(int id, {bool creating = false}) => {
    ...super.member(id, creating: creating),
    'producerReadyEvents': true,
  };
}

Future<void> until(WidgetTester tester, bool Function() ready) async {
  for (var attempt = 0; attempt < 30 && !ready(); ++attempt) {
    await tester.pump(const Duration(milliseconds: 1));
  }
  expect(ready(), isTrue);
}

Future<(NativeDocument, NativeTextureController)> start(
  WidgetTester tester,
  ReadyPlugin plugin,
) async {
  plugin.publish = false;
  plugin.install();
  final document = await NativeDocument.connect(GpuTransport());
  final opening = document.newDocument();
  await until(tester, () => document.info != null);
  await opening;
  final controller = NativeTextureController(
    document: document,
    presentation: FakePresentation(),
    platform: plugin.platform,
  );
  controller.configure(
    NativePageViewport.fit(
      const NativePageSize(600, 800),
      const Size(300, 240),
    ),
    deviceScale: 1.25,
  );
  await until(tester, () => plugin.statusCalls == 2);
  return (document, controller);
}

Future<void> finish(
  WidgetTester tester,
  NativeDocument document,
  NativeTextureController controller,
  ReadyPlugin plugin,
) async {
  var completed = false;
  final closed = controller.close().whenComplete(() => completed = true);
  for (var attempt = 0; attempt < 50 && !completed; ++attempt) {
    await tester.pump(const Duration(milliseconds: 20));
  }
  expect(
    completed,
    isTrue,
    reason: 'The fake platform must confirm retirement.',
  );
  await closed;
  controller.dispose();
  await document.dispose();
  await tester.pump();
  plugin.uninstall();
}

void main() {
  for (final platform in NativeTexturePlatform.values) {
    testWidgets(
      'producer event wakes ${platform.name} before the 100 ms tick',
      (tester) async {
        final plugin = ReadyPlugin(platform);
        final (document, controller) = await start(tester, plugin);
        try {
          expect(plugin.listens, 1);
          expect(controller.canDisplay, isFalse);
          plugin.publish = true;
          await plugin.emit(plugin.event());
          await tester.pump();
          expect(controller.canDisplay, isTrue);
          expect(plugin.statusCalls, 3);
          await plugin.emit(plugin.event());
          await plugin.emit({...plugin.event(), 'serial': 0});
          await tester.pump();
          expect(
            plugin.statusCalls,
            3,
            reason: 'Duplicate/older hints do not poll.',
          );
        } finally {
          await finish(tester, document, controller, plugin);
        }
        expect(plugin.cancels, 1);
      },
    );
  }

  testWidgets('unsupported streams keep the existing polling path', (
    tester,
  ) async {
    final plugin = ReadyPlugin(NativeTexturePlatform.linux)..available = false;
    final (document, controller) = await start(tester, plugin);
    try {
      expect(plugin.listens, 0);
      plugin.publish = true;
      await tester.pump(const Duration(milliseconds: 80));
      expect(controller.canDisplay, isFalse);
      await tester.pump(const Duration(milliseconds: 20));
      expect(controller.canDisplay, isTrue);
    } finally {
      await finish(tester, document, controller, plugin);
    }
  });

  testWidgets(
    'hints must match every source/view/device field and native owner',
    (tester) async {
      final plugin = ReadyPlugin(NativeTexturePlatform.linux);
      final (document, controller) = await start(tester, plugin);
      try {
        final original = plugin.event();
        for (final field in [
          'documentId',
          'revision',
          'fontGeneration',
          'viewGeneration',
          'deviceGeneration',
          'deviceScale',
        ]) {
          final stamp = Map<String, Object?>.from(original['stamp']! as Map);
          stamp[field] = (stamp[field]! as num) + 1;
          await plugin.emit({...original, 'stamp': stamp});
        }
        for (final invalid in [
          null,
          {'event': 'producerReady'},
          {...original, 'textureId': 99},
          {...original, 'deviceGeneration': 2},
          {...original, 'serial': 1.0},
          {...original, 'serial': true},
          {...original, 'event': 'consumerGpuComplete'},
        ]) {
          await plugin.emit(invalid);
        }
        await tester.pump();
        expect(plugin.statusCalls, 2);
        await plugin.emit(original);
        await tester.pump();
        expect(plugin.statusCalls, 3);
        expect(
          controller.canDisplay,
          isFalse,
          reason: 'An event alone is not publication proof.',
        );
        plugin.publish = true;
        await tester.pump(const Duration(milliseconds: 100));
        expect(
          controller.canDisplay,
          isTrue,
          reason: 'Missing hints retain the polling fallback.',
        );
      } finally {
        await finish(tester, document, controller, plugin);
      }
    },
  );

  testWidgets('a hint during a pending status triggers one follow-up check', (
    tester,
  ) async {
    final plugin = ReadyPlugin(NativeTexturePlatform.linux);
    final (document, controller) = await start(tester, plugin);
    try {
      final gate = Completer<void>();
      plugin.frozenGate = gate;
      await plugin.emit(plugin.event());
      await tester.pump();
      expect(plugin.statusCalls, 3);
      plugin.publish = true;
      await plugin.emit({...plugin.event(), 'serial': plugin.serial + 1});
      await plugin.emit({...plugin.event(), 'serial': plugin.serial + 1});
      gate.complete();
      await tester.pump();
      expect(plugin.statusCalls, 4);
      expect(controller.canDisplay, isTrue);
    } finally {
      await finish(tester, document, controller, plugin);
    }
  });

  testWidgets('close detaches events before a late publication', (
    tester,
  ) async {
    final plugin = ReadyPlugin(NativeTexturePlatform.linux);
    final (document, controller) = await start(tester, plugin);
    final late = plugin.event();
    final closing = controller.close();
    await tester.pump();
    await closing;
    final calls = plugin.statusCalls;
    await plugin.emit(late);
    await tester.pump();
    expect(plugin.statusCalls, calls);
    expect(controller.canDisplay, isFalse);
    expect(plugin.cancels, 1);
    await finish(tester, document, controller, plugin);
  });

  testWidgets('activation errors do not replace renderer state or polling', (
    tester,
  ) async {
    final plugin = ReadyPlugin(NativeTexturePlatform.linux)
      ..rejectListen = true;
    final (document, controller) = await start(tester, plugin);
    try {
      expect(plugin.listens, 1);
      expect(controller.diagnostic, isNull);
      plugin.publish = true;
      await tester.pump(const Duration(milliseconds: 100));
      expect(controller.canDisplay, isTrue);
    } finally {
      await finish(tester, document, controller, plugin);
    }
  });

  testWidgets(
    'both tile publications must pass status after their individual hints',
    (tester) async {
      final events = ReadyPlugin(NativeTexturePlatform.linux)..install();
      final plugin = ReadyTilePlugin()
        ..publish = false
        ..install();
      final document = await NativeDocument.connect(GpuTransport());
      final opening = document.newDocument();
      await until(tester, () => document.info != null);
      await opening;
      final controller = NativeTextureController(
        document: document,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.linux,
        useTileGroups: true,
      );
      try {
        controller.configure(
          NativePageViewport.fit(
            const NativePageSize(600, 800),
            const Size(300, 240),
          ),
          deviceScale: 1.25,
        );
        await until(
          tester,
          () => plugin.calls.where((call) => call == 'statusGroup').length == 2,
        );
        expect(events.listens, 1);
        plugin.publish = true;
        final hint = {
          'event': 'producerReady',
          'deviceGeneration': 1,
          'serial': plugin.serial,
          'stamp': plugin.acceptedStamp,
        };
        await events.emit({...hint, 'textureId': 1});
        await tester.pump();
        expect(controller.canDisplay, isFalse);
        plugin.secondPublished = true;
        await events.emit({...hint, 'textureId': 2});
        await tester.pump();
        expect(controller.canDisplay, isTrue);
      } finally {
        await finish(tester, document, controller, events);
      }
    },
  );

  testWidgets(
    'two owners share one listener and one owner cannot cancel another',
    (tester) async {
      final plugin = ReadyPlugin(NativeTexturePlatform.linux)..install();
      final first = <Object?>[], second = <Object?>[];
      final channel = MethodChannel(plugin.platform.channelName);
      final a = NativeProducerReadyEvents.attach(channel, first.add);
      final b = NativeProducerReadyEvents.attach(channel, second.add);
      await tester.pump();
      expect(plugin.listens, 1);
      await plugin.emit({'value': 1});
      expect(first, hasLength(1));
      expect(second, hasLength(1));
      a();
      a();
      await tester.pump();
      expect(plugin.cancels, 0);
      await plugin.emit({'value': 2});
      expect(first, hasLength(1));
      expect(second, hasLength(2));
      plugin.cancelGate = Completer<void>();
      b();
      final c = NativeProducerReadyEvents.attach(channel, first.add);
      await tester.pump();
      expect(
        plugin.listens,
        1,
        reason: 'Wait for old native cancellation before a new listen.',
      );
      plugin.cancelGate!.complete();
      await tester.pump();
      await plugin.emit({'value': 3});
      expect(first, hasLength(2));
      expect(second, hasLength(2));
      c();
      await tester.pump();
      expect(plugin.listens, plugin.cancels);
      plugin.uninstall();
    },
  );
}
