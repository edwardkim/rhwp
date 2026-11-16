// Fake-clock widget/channel regressions for idle RPC admission. No actual GPU,
// power, CPU scheduling, compositor display or consumer-fence evidence.
import 'dart:async';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';
import 'package:rhwp_native_reference/src/native_producer_ready.dart';

import 'native_producer_ready_test.dart'
    show ReadyPlugin, ReadyTilePlugin, finish, until;
import 'support/fake_gpu_presentation.dart';

class RetiringReadyPlugin extends ReadyPlugin {
  RetiringReadyPlugin() : super(NativeTexturePlatform.linux);
  bool holdRetirement = true;
  @override
  Map<String, Object?> status() => {
    ...super.status(),
    if (closed && holdRetirement) ...{
      'registered': true,
      'ownerFinished': false,
      'cleanupComplete': false,
    },
  };
}

Future<
  (GpuTransport, NativeDocument, NativeTextureController, FakePresentation)
>
startIdle(WidgetTester tester, ReadyPlugin plugin) async {
  plugin.publish = false;
  plugin.install();
  final transport = GpuTransport();
  final document = await NativeDocument.connect(transport);
  final opening = document.newDocument();
  await until(tester, () => document.info != null);
  await opening;
  final metadata = FakePresentation();
  final controller = NativeTextureController(
    document: document,
    presentation: metadata,
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
  return (transport, document, controller, metadata);
}

Future<void> publishIdle(
  WidgetTester tester,
  ReadyPlugin plugin,
  NativeTextureController controller,
) async {
  plugin.publish = true;
  await plugin.emit(plugin.event());
  await tester.pump();
  expect(controller.canDisplay, isTrue);
}

Future<void> eventBytes(ReadyPlugin plugin, ByteData? data) =>
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .handlePlatformMessage(plugin.eventName, data, (_) {});

Future<void> legacyTicks(WidgetTester tester, int count) async {
  for (var tick = 0; tick < count; ++tick) {
    await tester.pump(const Duration(milliseconds: 100));
  }
}

void main() {
  for (final platform in NativeTexturePlatform.values) {
    testWidgets('${platform.name} current publication has no idle GPU RPC', (
      tester,
    ) async {
      final plugin = ReadyPlugin(platform);
      final (_, document, controller, metadata) = await startIdle(
        tester,
        plugin,
      );
      try {
        await publishIdle(tester, plugin, controller);
        final status = plugin.statusCalls, snapshots = metadata.checks.length;
        await tester.pump(const Duration(seconds: 10));
        expect(controller.canDisplay, isTrue);
        expect(plugin.statusCalls, status);
        expect(metadata.checks, hasLength(snapshots));
      } finally {
        await finish(tester, document, controller, plugin);
      }
    });
  }

  testWidgets('event capability does not suppress polls before listen ack', (
    tester,
  ) async {
    final plugin = ReadyPlugin(NativeTexturePlatform.linux)
      ..listenGate = Completer<void>();
    final (_, document, controller, _) = await startIdle(tester, plugin);
    try {
      await publishIdle(tester, plugin, controller);
      final before = plugin.statusCalls;
      await legacyTicks(tester, 3);
      expect(plugin.statusCalls, before + 3);
      plugin.listenGate!.complete();
      await tester.pump();
      final acknowledged = plugin.statusCalls;
      await tester.pump(const Duration(seconds: 5));
      expect(plugin.statusCalls, acknowledged);
    } finally {
      if (!plugin.listenGate!.isCompleted) plugin.listenGate!.complete();
      await finish(tester, document, controller, plugin);
    }
  });

  testWidgets('late listen completion cannot disable polling after timeout', (
    tester,
  ) async {
    final plugin = ReadyPlugin(NativeTexturePlatform.linux)
      ..listenGate = Completer<void>();
    final (_, document, controller, _) = await startIdle(tester, plugin);
    try {
      await publishIdle(tester, plugin, controller);
      await legacyTicks(tester, 21);
      expect(controller.canDisplay, isTrue);
      expect(controller.diagnostic, isNull);
      expect(plugin.cancels, 1);
      plugin.listenGate!.complete();
      await tester.pump();
      final timedOut = plugin.statusCalls;
      await legacyTicks(tester, 3);
      expect(plugin.statusCalls, timedOut + 3);
    } finally {
      if (!plugin.listenGate!.isCompleted) plugin.listenGate!.complete();
      await finish(tester, document, controller, plugin);
    }
  });

  for (final unavailable in ['unsupported', 'listen error']) {
    testWidgets('$unavailable retains legacy polls after publication', (
      tester,
    ) async {
      final plugin = ReadyPlugin(NativeTexturePlatform.linux)
        ..available = unavailable != 'unsupported'
        ..rejectListen = unavailable == 'listen error';
      final (_, document, controller, _) = await startIdle(tester, plugin);
      try {
        plugin.publish = true;
        await legacyTicks(tester, 1);
        expect(controller.canDisplay, isTrue);
        final published = plugin.statusCalls;
        await legacyTicks(tester, 3);
        expect(plugin.statusCalls, published + 3);
      } finally {
        await finish(tester, document, controller, plugin);
      }
    });
  }

  for (final failure in [
    'end of stream',
    'error envelope',
    'malformed envelope',
  ]) {
    testWidgets('$failure restores idle fallback polling', (tester) async {
      final plugin = ReadyPlugin(NativeTexturePlatform.linux);
      final (_, document, controller, _) = await startIdle(tester, plugin);
      try {
        await publishIdle(tester, plugin, controller);
        final published = plugin.statusCalls;
        await tester.pump(const Duration(seconds: 1));
        expect(plugin.statusCalls, published);
        final ByteData? envelope = switch (failure) {
          'end of stream' => null,
          'error envelope' => const StandardMethodCodec().encodeErrorEnvelope(
            code: 'eventTransportUnavailable',
          ),
          _ => ByteData.sublistView(Uint8List.fromList([255])),
        };
        await eventBytes(plugin, envelope);
        await tester.pump();
        await legacyTicks(tester, 3);
        expect(controller.canDisplay, isTrue);
        expect(controller.diagnostic, isNull);
        expect(plugin.statusCalls, published + 3);
      } finally {
        await finish(tester, document, controller, plugin);
      }
    });
  }

  testWidgets('shared event availability reaches every current owner', (
    tester,
  ) async {
    final plugin = ReadyPlugin(NativeTexturePlatform.linux)
      ..listenGate = Completer<void>()
      ..install();
    final channel = MethodChannel(plugin.platform.channelName);
    final first = <bool>[], second = <bool>[], replacement = <bool>[];
    void firstEvent(Object? _) {}
    void secondEvent(Object? _) {}
    void replacementEvent(Object? _) {}
    final a = NativeProducerReadyEvents.attach(
      channel,
      firstEvent,
      onAvailabilityChanged: first.add,
    );
    final b = NativeProducerReadyEvents.attach(
      channel,
      secondEvent,
      onAvailabilityChanged: second.add,
    );
    expect(first, [false]);
    expect(second, [false]);
    plugin.listenGate!.complete();
    await tester.pump();
    expect(first, [false, true]);
    expect(second, [false, true]);
    a();
    await eventBytes(plugin, null);
    expect(first, [false, true], reason: 'The detached owner is not notified.');
    expect(second, [false, true, false]);
    b();
    await tester.pump();
    final c = NativeProducerReadyEvents.attach(
      channel,
      replacementEvent,
      onAvailabilityChanged: replacement.add,
    );
    await tester.pump();
    expect(replacement, [false, true]);
    c();
    await tester.pump();
    plugin.uninstall();
  });

  testWidgets(
    'a missing ready hint keeps pending publication polling bounded',
    (tester) async {
      final plugin = ReadyPlugin(NativeTexturePlatform.linux);
      final (_, document, controller, _) = await startIdle(tester, plugin);
      try {
        final accepted = plugin.statusCalls;
        await legacyTicks(tester, 3);
        expect(controller.canDisplay, isFalse);
        expect(plugin.statusCalls, accepted + 3);
        plugin.publish = true; // Deliberately send no event.
        await legacyTicks(tester, 1);
        expect(controller.canDisplay, isTrue);
        final published = plugin.statusCalls;
        await tester.pump(const Duration(seconds: 5));
        expect(plugin.statusCalls, published);
      } finally {
        await finish(tester, document, controller, plugin);
      }
    },
  );

  testWidgets('an event-capable stalled producer still reaches its deadline', (
    tester,
  ) async {
    final plugin = ReadyPlugin(NativeTexturePlatform.linux);
    final (_, document, controller, _) = await startIdle(tester, plugin);
    try {
      await legacyTicks(tester, 21);
      expect(controller.canDisplay, isFalse);
      expect(controller.needsCpuFallback, isTrue);
      expect(controller.diagnostic, isA<TimeoutException>());
      expect(plugin.statusCalls, greaterThan(2));
      expect(plugin.statusCalls, lessThanOrEqualTo(23));
      final timedOut = plugin.statusCalls;
      await tester.pump(const Duration(seconds: 5));
      expect(plugin.statusCalls, timedOut);
    } finally {
      await finish(tester, document, controller, plugin);
    }
  });

  testWidgets(
    'document revision wakes an idle owner and rechecks publication',
    (tester) async {
      final plugin = ReadyPlugin(NativeTexturePlatform.linux);
      final (transport, document, controller, _) = await startIdle(
        tester,
        plugin,
      );
      try {
        await publishIdle(tester, plugin, controller);
        final submissions = plugin.submissions.length;
        plugin.publish = false;
        ++transport.revision;
        final refresh = document.refreshInfo();
        await until(
          tester,
          () => document.info?.revision == transport.revision,
        );
        await refresh;
        await until(tester, () => plugin.submissions.length > submissions);
        expect(controller.canDisplay, isFalse);
        expect(plugin.acceptedStamp!['revision'], transport.revision);
        await publishIdle(tester, plugin, controller);
        final published = plugin.statusCalls;
        await tester.pump(const Duration(seconds: 3));
        expect(plugin.statusCalls, published);
      } finally {
        await finish(tester, document, controller, plugin);
      }
    },
  );

  testWidgets('DPR change rejects the old hint and wakes an idle owner', (
    tester,
  ) async {
    final plugin = ReadyPlugin(NativeTexturePlatform.linux);
    final (_, document, controller, _) = await startIdle(tester, plugin);
    try {
      await publishIdle(tester, plugin, controller);
      final old = plugin.event(), submissions = plugin.submissions.length;
      plugin.publish = false;
      controller.configure(
        NativePageViewport.fit(
          const NativePageSize(600, 800),
          const Size(300, 240),
        ),
        deviceScale: 2,
      );
      expect(controller.canDisplay, isFalse);
      await until(tester, () => plugin.submissions.length > submissions);
      await plugin.emit(old);
      await tester.pump();
      expect(controller.canDisplay, isFalse);
      expect(plugin.acceptedStamp!['deviceScale'], 2);
      await publishIdle(tester, plugin, controller);
    } finally {
      await finish(tester, document, controller, plugin);
    }
  });

  testWidgets('local presentation ownership is checked during idle ticks', (
    tester,
  ) async {
    final plugin = ReadyPlugin(NativeTexturePlatform.linux);
    final (_, document, controller, _) = await startIdle(tester, plugin);
    try {
      await publishIdle(tester, plugin, controller);
      document.beginPresentation(); // A second viewport claimed the owner.
      expect(controller.canDisplay, isFalse);
      await legacyTicks(tester, 1);
      expect(controller.needsCpuFallback, isTrue);
      expect(
        controller.diagnostic,
        isA<NativeCallException>().having(
          (error) => error.code,
          'code',
          'stalePresentation',
        ),
      );
    } finally {
      await finish(tester, document, controller, plugin);
    }
  });

  testWidgets(
    'native view pause cover and resume work after idle polling stops',
    (tester) async {
      final plugin = ReadyPlugin(NativeTexturePlatform.macosMetalView);
      final (_, document, controller, _) = await startIdle(tester, plugin);
      try {
        await publishIdle(tester, plugin, controller);
        final paused = controller.pause();
        await tester.pump();
        await paused;
        expect(controller.isPaused, isTrue);
        expect(plugin.covered, isTrue);
        final status = plugin.statusCalls;
        await tester.pump(const Duration(seconds: 3));
        expect(plugin.statusCalls, status);
        plugin.publish = false;
        controller.resume();
        expect(controller.canDisplay, isFalse);
        await until(tester, () => plugin.submissions.length == 2);
        await publishIdle(tester, plugin, controller);
        expect(plugin.covered, isFalse);
      } finally {
        await finish(tester, document, controller, plugin);
      }
    },
  );

  testWidgets(
    'idle suspension never suppresses registration retirement polls',
    (tester) async {
      final plugin = RetiringReadyPlugin();
      final (_, document, controller, _) = await startIdle(tester, plugin);
      try {
        await publishIdle(tester, plugin, controller);
        final idle = plugin.statusCalls;
        await tester.pump(const Duration(seconds: 1));
        expect(plugin.statusCalls, idle);
        var closed = false;
        final closing = controller.close().then((_) => closed = true);
        await tester.pump();
        expect(plugin.cancels, 1);
        await legacyTicks(tester, 3);
        expect(closed, isFalse);
        expect(plugin.statusCalls, greaterThan(idle));
        plugin.holdRetirement = false;
        await legacyTicks(tester, 1);
        await closing;
        expect(closed, isTrue);
        expect(controller.retirementPending, isFalse);
      } finally {
        plugin.holdRetirement = false;
        await finish(tester, document, controller, plugin);
      }
    },
  );

  testWidgets('tile group polls until both members publish then becomes idle', (
    tester,
  ) async {
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
    int statusCalls() =>
        plugin.calls.where((call) => call == 'statusGroup').length;
    try {
      controller.configure(
        NativePageViewport.fit(
          const NativePageSize(600, 800),
          const Size(300, 240),
        ),
        deviceScale: 1.25,
      );
      await until(tester, () => statusCalls() == 2);
      plugin.publish = true;
      await legacyTicks(tester, 2);
      expect(controller.canDisplay, isFalse);
      final pending = statusCalls();
      plugin.secondPublished = true;
      await legacyTicks(tester, 1);
      expect(controller.canDisplay, isTrue);
      expect(statusCalls(), pending + 1);
      final visible = statusCalls();
      await tester.pump(const Duration(seconds: 5));
      expect(statusCalls(), visible);
    } finally {
      await finish(tester, document, controller, events);
    }
  });
}
