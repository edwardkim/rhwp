// Actual controller, binding and MethodChannel contracts with fake-clock/native
// receipts. These tests do not execute a platform GPU or measure process memory.
import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'native_gpu_lifecycle_test.dart' show opened, until, cleanup;
import 'native_texture_tiles_test.dart' show TilePlugin;
import 'support/fake_gpu_presentation.dart';

Map<String, Object?> receipt(
  Map<String, Object?> args, {
  bool pending = false,
  bool busy = false,
}) => {
  'textureId': args['textureId'],
  'deviceGeneration': args['deviceGeneration'],
  'requestId': args['requestId'],
  'targetBytes': args['targetBytes'],
  'scope': 'managedGpuTargets',
  'before': pending ? 0 : 8192,
  'after': pending
      ? 0
      : busy
      ? 8192
      : 0,
  'trimmed': pending || busy ? 0 : 8192,
  'pending': pending,
  'busy': pending || busy,
  'reason': pending
      ? 'queued'
      : busy
      ? 'pinned'
      : 'trimmed',
};

class TrimPlugin extends FakeTexturePlugin {
  TrimPlugin([super.platform = NativeTexturePlatform.linux]);
  final trims = <MethodCall>[];
  bool pending = false, busy = false, unsupported = false;
  Object? trimError;
  Completer<void>? gate;
  Map<String, Object?> overrides = {};

  @override
  Future<Object?> handle(MethodCall call) async {
    if (call.method != 'trimCache' && call.method != 'trimCacheStatus') {
      return super.handle(call);
    }
    calls.add(call.method);
    trims.add(call);
    if (unsupported) throw MissingPluginException();
    final args = Map<String, Object?>.from(call.arguments! as Map);
    final result = {
      ...receipt(args, pending: pending, busy: busy),
      ...overrides,
    };
    await gate?.future;
    final error = trimError;
    if (error != null) throw error;
    return result;
  }
}

class TrimTilePlugin extends TilePlugin {
  TrimTilePlugin() : super(NativeTexturePlatform.linux);
  final trims = <MethodCall>[];
  bool secondPending = true;
  @override
  Future<Object?> handle(MethodCall call) async {
    if (call.method != 'trimCache' && call.method != 'trimCacheStatus') {
      return super.handle(call);
    }
    final args = Map<String, Object?>.from(call.arguments! as Map);
    trims.add(call);
    if (args['groupId'] != 1) {
      throw PlatformException(
        code: 'invalidArgument',
        message: 'group ID required',
      );
    }
    return {
      ...args,
      'textureId': 1,
      'scope': 'managedGpuTargets',
      'before': secondPending ? 8192 : 16384,
      'after': 0,
      'trimmed': secondPending ? 8192 : 16384,
      'pending': secondPending,
      'busy': secondPending,
      'reason': secondPending ? 'pending' : 'complete',
      'members': [
        receipt({...args, 'textureId': 1}),
        receipt({...args, 'textureId': 2}, pending: secondPending),
      ],
    };
  }
}

Future<(NativeDocument, NativeTextureController)> ready(
  WidgetTester tester,
  TrimPlugin plugin,
) async {
  plugin.install();
  final document = await opened(tester, GpuTransport());
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
    deviceScale: 1,
  );
  await until(tester, () => controller.canDisplay);
  return (document, controller);
}

Widget pressureView(
  NativeTextureController controller, {
  ValueChanged<Object>? onTrimError,
}) => MaterialApp(
  home: NativeGpuLifecycle(
    controller: controller,
    onTrimError: onTrimError,
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

const identity = <String, Object?>{
  'textureId': 5,
  'deviceGeneration': 7,
  'requestId': 11,
  'targetBytes': 0,
};
NativeGpuCacheTrimResult parse(Map<String, Object?> raw) =>
    NativeGpuCacheTrimResult.fromJson(
      raw,
      textureId: 5,
      deviceGeneration: 7,
      requestId: 11,
      targetBytes: 0,
    );

void main() {
  test(
    'trim receipt binds owner/request and consistent managed-byte reduction',
    () {
      final result = parse(receipt(identity));
      expect(result.supported, isTrue);
      expect(result.scope, 'managedGpuTargets');
      expect(result.before, 8192);
      expect(result.after, 0);
      expect(result.trimmed, 8192);
      expect(result.pending, isFalse);
      expect(result.busy, isFalse);
      expect(result.members, isEmpty);
    },
  );

  test('pending and pinned receipts remain distinct final outcomes', () {
    final queued = parse(receipt(identity, pending: true));
    expect(queued.pending, isTrue);
    expect(queued.before, 0);
    final pinned = parse(receipt(identity, busy: true));
    expect(pinned.pending, isFalse);
    expect(pinned.busy, isTrue);
    expect(pinned.after, 8192);
    expect(pinned.trimmed, 0);
  });

  for (final field in [
    'textureId',
    'deviceGeneration',
    'requestId',
    'targetBytes',
  ]) {
    test('a receipt with another $field is rejected', () {
      expect(
        () =>
            parse({...receipt(identity), field: (identity[field]! as int) + 1}),
        throwsA(isA<NativeCallException>()),
      );
    });
  }
  for (final mutation in <Map<String, Object?>>[
    {'before': -1},
    {'after': 8193},
    {'trimmed': 8191},
    {'trimmed': 8192.0},
    {'pending': 0},
    {'busy': 'false'},
    {'reason': false},
    {'scope': 'processVram'},
    {'members': []},
    {'pending': true, 'busy': false},
    {'after': 1, 'trimmed': 8191},
  ]) {
    test('malformed trim metadata $mutation is rejected', () {
      expect(
        () => parse({...receipt(identity), ...mutation}),
        throwsA(isA<NativeCallException>()),
      );
    });
  }

  Map<String, Object?> groupReceipt() => {
    ...identity,
    'groupId': 5,
    'targetBytes': 4096,
    'scope': 'managedGpuTargets',
    'before': 16384,
    'after': 8192,
    'trimmed': 8192,
    'pending': false,
    'busy': false,
    'reason': 'complete',
    'members': [
      for (final id in [101, 102])
        {
          ...identity,
          'textureId': id,
          'targetBytes': 4096,
          'scope': 'managedGpuTargets',
          'before': 8192,
          'after': 4096,
          'trimmed': 4096,
          'pending': false,
          'busy': false,
          'reason': 'trimmed',
        },
    ],
  };
  NativeGpuCacheTrimResult parseGroup(Map<String, Object?> raw) =>
      NativeGpuCacheTrimResult.fromJson(
        raw,
        textureId: 5,
        deviceGeneration: 7,
        requestId: 11,
        targetBytes: 4096,
        memberTextureIds: const [101, 102],
      );

  test('group target is per owner while managed-byte counters are summed', () {
    final result = parseGroup(groupReceipt());
    expect(result.after, greaterThan(result.targetBytes));
    expect(result.busy, isFalse);
    expect(result.members.map((member) => member.textureId), [101, 102]);
    expect(result.members.every((member) => member.after == 4096), isTrue);
    expect(() => result.members.clear(), throwsUnsupportedError);
  });

  test('group receipts reject incomplete/mixed identities, sums and flags', () {
    for (final change in [
      'member identity',
      'member request',
      'total',
      'pending',
    ]) {
      final raw = groupReceipt();
      final members = raw['members']! as List;
      switch (change) {
        case 'member identity':
          (members[1] as Map)['textureId'] = 101;
        case 'member request':
          (members[1] as Map)['requestId'] = 12;
        case 'total':
          raw['before'] = 16385;
          raw['trimmed'] = 8193;
        case 'pending':
          raw['pending'] = true;
      }
      expect(() => parseGroup(raw), throwsA(isA<NativeCallException>()));
    }
  });

  testWidgets('tile-group trims route and finish every member of one request', (
    tester,
  ) async {
    final plugin = TrimTilePlugin()
      ..secondPublished = true
      ..install();
    final document = await opened(tester, GpuTransport());
    final controller = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: plugin.platform,
      useTileGroups: true,
    );
    try {
      controller.configure(
        NativePageViewport.fit(
          const NativePageSize(600, 800),
          const Size(300, 240),
        ),
        deviceScale: 1,
      );
      await until(tester, () => controller.canDisplay);
      final trimming = controller.trimCaches(targetBytes: 1024);
      await until(tester, () => plugin.trims.isNotEmpty);
      expect((plugin.trims.first.arguments as Map)['groupId'], 1);
      plugin.secondPending = false;
      await tester.pump(const Duration(milliseconds: 100));
      final result = await trimming;
      expect(result.members.map((member) => member.textureId), [1, 2]);
      expect(result.trimmed, 16384);
      expect(result.pending, isFalse);
      expect(plugin.trims.map((call) => call.method), [
        'trimCache',
        'trimCacheStatus',
      ]);
      expect(controller.canDisplay, isTrue);
    } finally {
      await cleanup(tester, controller, document, plugin);
    }
  });

  testWidgets('pressure before registration never creates a GPU owner', (
    tester,
  ) async {
    final plugin = TrimPlugin()..install();
    final document = await opened(tester, GpuTransport());
    final controller = NativeTextureController(
      document: document,
      presentation: FakePresentation(),
      platform: plugin.platform,
    );
    try {
      await tester.pumpWidget(
        MaterialApp(
          home: NativeGpuLifecycle(
            controller: controller,
            child: const SizedBox.shrink(),
          ),
        ),
      );
      tester.binding.handleMemoryPressure();
      await tester.pump();
      final result = await controller.trimCaches();
      expect(result.supported, isFalse);
      expect(result.reason, 'notReady');
      expect(plugin.calls, isEmpty);
      expect(plugin.trims, isEmpty);
    } finally {
      await cleanup(tester, controller, document, plugin);
    }
  });

  testWidgets(
    'concurrent trim calls share one owner request and first target',
    (tester) async {
      final gate = Completer<void>();
      final plugin = TrimPlugin()..gate = gate;
      final (document, controller) = await ready(tester, plugin);
      try {
        final owned = Set<int>.of(plugin.owned);
        final first = controller.trimCaches();
        final coalesced = controller.trimCaches(targetBytes: 4096);
        expect(coalesced, same(first));
        await until(tester, () => plugin.trims.isNotEmpty);
        expect(plugin.trims, hasLength(1));
        expect((plugin.trims.single.arguments as Map)['targetBytes'], 0);
        plugin.gate!.complete();
        await tester.pump();
        final result = await first;
        expect(result.trimmed, 8192);
        expect(result.requestId, 1);
        expect(controller.canDisplay, isTrue);
        expect(plugin.owned, owned);
        plugin.gate = null;
        final second = controller.trimCaches();
        await tester.pump();
        expect((await second).requestId, 2);
        expect(plugin.trims, hasLength(2));
      } finally {
        if (!gate.isCompleted) gate.complete();
        await cleanup(tester, controller, document, plugin);
      }
    },
  );

  testWidgets(
    'only pending receipts are polled and a pinned final result stops',
    (tester) async {
      final plugin = TrimPlugin()..pending = true;
      final (document, controller) = await ready(tester, plugin);
      try {
        final trimming = controller.trimCaches();
        await until(tester, () => plugin.trims.length == 1);
        await tester.pump(const Duration(milliseconds: 100));
        expect(plugin.trims.length, greaterThan(1));
        expect(plugin.trims.last.method, 'trimCacheStatus');
        plugin.pending = false;
        plugin.busy = true;
        await tester.pump(const Duration(milliseconds: 100));
        final result = await trimming;
        expect(result.pending, isFalse);
        expect(result.busy, isTrue);
        expect(result.trimmed, 0);
        final calls = plugin.trims.length;
        await tester.pump(const Duration(seconds: 1));
        expect(plugin.trims, hasLength(calls));
        expect(controller.canDisplay, isTrue);
      } finally {
        await cleanup(tester, controller, document, plugin);
      }
    },
  );

  testWidgets(
    'trim polling has one overall deadline without failing rendering',
    (tester) async {
      final plugin = TrimPlugin()..pending = true;
      final (document, controller) = await ready(tester, plugin);
      try {
        final expected = expectLater(
          controller.trimCaches(),
          throwsA(isA<TimeoutException>()),
        );
        for (var tick = 0; tick < 21; ++tick) {
          await tester.pump(const Duration(milliseconds: 100));
        }
        await expected;
        expect(plugin.trims.length, greaterThan(1));
        expect(plugin.trims.length, lessThanOrEqualTo(22));
        final calls = plugin.trims.length;
        await tester.pump(const Duration(seconds: 3));
        expect(plugin.trims, hasLength(calls));
        expect(controller.canDisplay, isTrue);
        expect(controller.diagnostic, isNull);
      } finally {
        await cleanup(tester, controller, document, plugin);
      }
    },
  );

  testWidgets(
    'old plugin fallback is explicit and does not alter publication',
    (tester) async {
      final plugin = TrimPlugin()..unsupported = true;
      final (document, controller) = await ready(tester, plugin);
      try {
        final trimming = controller.trimCaches();
        await tester.pump();
        final result = await trimming;
        expect(result.supported, isFalse);
        expect(result.reason, 'unsupported');
        expect(result.trimmed, 0);
        expect(controller.canDisplay, isTrue);
        expect(controller.diagnostic, isNull);
      } finally {
        await cleanup(tester, controller, document, plugin);
      }
    },
  );

  testWidgets(
    'wrong receipt identity rejects trim without failing the viewport',
    (tester) async {
      final plugin = TrimPlugin()..overrides['requestId'] = 99;
      final (document, controller) = await ready(tester, plugin);
      try {
        final expected = expectLater(
          controller.trimCaches(),
          throwsA(
            isA<NativeCallException>().having(
              (error) => error.code,
              'code',
              'invalidCacheTrimReply',
            ),
          ),
        );
        await tester.pump();
        await expected;
        expect(controller.canDisplay, isTrue);
        expect(controller.diagnostic, isNull);
      } finally {
        await cleanup(tester, controller, document, plugin);
      }
    },
  );

  testWidgets('closing cancels local observation before a late trim error', (
    tester,
  ) async {
    final gate = Completer<void>();
    final plugin = TrimPlugin()..gate = gate;
    final (document, controller) = await ready(tester, plugin);
    try {
      NativeGpuCacheTrimResult? cancelled;
      unawaited(
        controller.trimCaches().then((result) {
          cancelled = result;
        }),
      );
      await until(tester, () => plugin.trims.isNotEmpty);
      final closing = controller.close();
      await tester.pump();
      expect(
        cancelled,
        isNotNull,
        reason: 'Close completes local observation immediately.',
      );
      expect(cancelled!.reason, 'closed');
      await closing;
      expect(controller.phase, NativeTexturePhase.closed);
      gate.completeError(PlatformException(code: 'lateNativeError'));
      await tester.pump();
      expect(plugin.trims, hasLength(1));
      expect(controller.phase, NativeTexturePhase.closed);
    } finally {
      if (!gate.isCompleted) gate.complete();
      await cleanup(tester, controller, document, plugin);
    }
  });

  testWidgets('a new DPR frame remains valid after an older trim completes', (
    tester,
  ) async {
    final gate = Completer<void>();
    final plugin = TrimPlugin()..gate = gate;
    final (document, controller) = await ready(tester, plugin);
    try {
      final trimming = controller.trimCaches();
      await until(tester, () => plugin.trims.isNotEmpty);
      controller.configure(
        NativePageViewport.fit(
          const NativePageSize(600, 800),
          const Size(300, 240),
        ),
        deviceScale: 2,
      );
      await until(tester, () => controller.canDisplay);
      gate.complete();
      await tester.pump();
      expect((await trimming).supported, isTrue);
      expect(controller.outputDeviceScale, 2);
      expect(controller.canDisplay, isTrue);
    } finally {
      if (!gate.isCompleted) gate.complete();
      await cleanup(tester, controller, document, plugin);
    }
  });

  testWidgets('invalid target is rejected before any trim RPC', (tester) async {
    final plugin = TrimPlugin();
    final (document, controller) = await ready(tester, plugin);
    try {
      for (final invalid in [-1, NativeGpuCacheTrimResult.maxTargetBytes + 1]) {
        expect(
          () => controller.trimCaches(targetBytes: invalid),
          throwsArgumentError,
        );
      }
      expect(plugin.trims, isEmpty);
    } finally {
      await cleanup(tester, controller, document, plugin);
    }
  });

  testWidgets(
    'memory pressure coalesces while hidden and resume still publishes',
    (tester) async {
      final gate = Completer<void>();
      final plugin = TrimPlugin()..gate = gate;
      final (document, controller) = await ready(tester, plugin);
      try {
        await tester.pumpWidget(pressureView(controller));
        await until(tester, () => controller.canDisplay);
        tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
        final captures = plugin.submissions.length;
        tester.binding.handleMemoryPressure();
        tester.binding.handleMemoryPressure();
        tester.binding.handleMemoryPressure();
        await until(tester, () => plugin.trims.isNotEmpty);
        expect(plugin.trims, hasLength(1));
        expect((plugin.trims.single.arguments as Map)['targetBytes'], 0);
        expect(controller.isPaused, isTrue);
        expect(plugin.submissions, hasLength(captures));
        tester.binding.handleAppLifecycleStateChanged(
          AppLifecycleState.resumed,
        );
        await until(tester, () => controller.canDisplay);
        gate.complete();
        await tester.pump();
        expect(controller.canDisplay, isTrue);
      } finally {
        if (!gate.isCompleted) gate.complete();
        await cleanup(tester, controller, document, plugin);
      }
    },
  );

  testWidgets(
    'only the current lifecycle owner receives optional trim errors',
    (tester) async {
      final gate = Completer<void>();
      final plugin = TrimPlugin()..gate = gate;
      final (document, controller) = await ready(tester, plugin);
      final errors = <Object>[];
      try {
        await tester.pumpWidget(
          pressureView(controller, onTrimError: errors.add),
        );
        await until(tester, () => controller.canDisplay);
        tester.binding.handleMemoryPressure();
        tester.binding.handleMemoryPressure();
        await until(tester, () => plugin.trims.isNotEmpty);
        gate.completeError(PlatformException(code: 'trimNativeError'));
        await until(tester, () => errors.isNotEmpty);
        expect(errors, hasLength(1));
        expect(controller.canDisplay, isTrue);
        expect(controller.diagnostic, isNull);
      } finally {
        if (!gate.isCompleted) gate.complete();
        await cleanup(tester, controller, document, plugin);
      }
    },
  );

  for (final replacement in ['unmount', 'new controller']) {
    testWidgets('a late trim error is ignored after $replacement', (
      tester,
    ) async {
      final gate = Completer<void>();
      final plugin = TrimPlugin()..gate = gate;
      final (document, controller) = await ready(tester, plugin);
      final errors = <Object>[];
      Widget lifecycle(NativeTextureController owner) => MaterialApp(
        home: NativeGpuLifecycle(
          controller: owner,
          onTrimError: errors.add,
          child: const SizedBox(width: 300, height: 240),
        ),
      );
      final nextPlugin = TrimPlugin(NativeTexturePlatform.windows);
      NativeDocument? nextDocument;
      NativeTextureController? next;
      try {
        await tester.pumpWidget(lifecycle(controller));
        await until(tester, () => controller.canDisplay);
        final observer = tester.state(
          find.byType(NativeGpuLifecycle),
        ) as WidgetsBindingObserver;
        tester.binding.handleMemoryPressure();
        await until(tester, () => plugin.trims.isNotEmpty);
        if (replacement == 'unmount') {
          await tester.pumpWidget(const SizedBox.shrink());
        } else {
          final pair = await ready(tester, nextPlugin);
          nextDocument = pair.$1;
          final replacementController = pair.$2;
          next = replacementController;
          await tester.pumpWidget(lifecycle(replacementController));
          await until(tester, () => replacementController.canDisplay);
        }
        expect(controller.phase, NativeTexturePhase.visible);
        gate.completeError(PlatformException(code: 'oldTrimFailure'));
        await tester.pump(const Duration(milliseconds: 30));
        expect(errors, isEmpty);
        if (replacement == 'unmount') {
          final calls = plugin.trims.length;
          observer.didHaveMemoryPressure(); // A previously captured observer.
          await tester.pump();
          expect(plugin.trims, hasLength(calls));
        }
        if (next != null) expect(next.canDisplay, isTrue);
      } finally {
        if (!gate.isCompleted) gate.complete();
        if (next != null) {
          await cleanup(tester, next, nextDocument!, nextPlugin);
        }
        await cleanup(tester, controller, document, plugin);
      }
    });
  }
}
