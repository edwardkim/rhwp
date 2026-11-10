// 합성 계약: accepted batch의 producer 준비 확인, polling 간격과 ownership 관계 검사.
// 주석 갱신: 2026-11-10 KST. 신설 파일이므로 직전 파일 commit 없음.
// FakeMethodChannel과 가상 시계만 사용한다. 실물 GPU·Flutter display 측정이 아니다.

import 'dart:async';
import 'dart:convert';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'support/fake_gpu_presentation.dart';

class MeasuredPlugin extends FakeTexturePlugin {
  MeasuredPlugin() : super(NativeTexturePlatform.linux);

  @override
  Map<String, Object?> status() => {
    ...super.status(),
    'wholeFrameReadbacks': 0,
    'libraryPath': '/private/native/module.so',
    'limits': {
      ...super.status()['limits']! as Map,
      'producerOwnedBytes': 4096,
      'importPendingBytes': 0,
      'reservedTargetBytes': 8192,
      'descriptorTickets': -1,
      'unexpectedPrivateDetail': '/private/document.hwpx',
    },
  };

  int get statusCalls => calls.where((method) => method == 'status').length;
}

Future<void> until(WidgetTester tester, bool Function() ready) async {
  for (var index = 0; index < 40 && !ready(); ++index) {
    await tester.pump(const Duration(milliseconds: 5));
  }
  expect(ready(), isTrue);
}

Future<NativeDocument> open(WidgetTester tester, GpuTransport transport) async {
  final document = await NativeDocument.connect(transport);
  final opening = document.newDocument();
  await until(tester, () => document.info != null);
  await opening;
  return document;
}

void configure(NativeTextureController controller, {double width = 300}) =>
    controller.configure(
      NativePageViewport.fit(const NativePageSize(600, 800), Size(width, 240)),
      deviceScale: 1.5,
    );

NativeTextureController controllerFor(
  NativeDocument document,
  FakePresentation metadata, {
  NativeGpuTrace? trace,
  Duration timeout = const Duration(seconds: 2),
}) => NativeTextureController(
  document: document,
  presentation: metadata,
  platform: NativeTexturePlatform.linux,
  trace: trace,
  operationTimeout: timeout,
);

Future<void> cleanUp(
  WidgetTester tester,
  NativeTextureController controller,
  NativeDocument document,
  FakeTexturePlugin plugin,
) async {
  final closing = controller.close();
  await tester.pump();
  await closing;
  controller.dispose();
  await document.dispose();
  plugin.uninstall();
}

void main() {
  testWidgets(
    'accepted ready batch is checked before the next default 100 ms tick',
    (tester) async {
      final plugin = MeasuredPlugin()
        ..submitGate = Completer<void>()
        ..acceptedBeforeReply = true
        ..install();
      final document = await open(tester, GpuTransport());
      final metadata = FakePresentation();
      final trace = NativeGpuTrace(
        clockMicros: () => tester.binding.clock.now().microsecondsSinceEpoch,
      );
      final controller = controllerFor(document, metadata, trace: trace);
      final started = tester.binding.clock.now();
      try {
        configure(controller);
        await until(tester, () => plugin.submissions.isNotEmpty);
        expect(plugin.statusCalls, 1, reason: 'Initial device readiness only.');
        expect(controller.canDisplay, isFalse);
        final acceptedAt = tester.binding.clock.now();
        plugin.submitGate!.complete();
        await tester.pump();
        expect(controller.canDisplay, isTrue);
        expect(plugin.statusCalls, 2, reason: 'Exactly one publication probe.');
        expect(
          tester.binding.clock.now().difference(acceptedAt),
          Duration.zero,
        );
        expect(
          tester.binding.clock.now().difference(started),
          lessThan(const Duration(milliseconds: 100)),
        );
        expect(
          trace.events.map((event) => event.stage),
          containsAllInOrder([
            NativeGpuTraceStage.configured,
            NativeGpuTraceStage.invalidated,
            NativeGpuTraceStage.captureStarted,
            NativeGpuTraceStage.captureCompleted,
            NativeGpuTraceStage.submitStarted,
            NativeGpuTraceStage.accepted,
            NativeGpuTraceStage.statusPolled,
            NativeGpuTraceStage.producerReady,
          ]),
        );
        final ready = trace.events.last;
        expect(ready.counters[NativeGpuCounter.pluginProducerOwnedBytes], 4096);
        expect(ready.counters[NativeGpuCounter.pluginImportPendingBytes], 0);
        expect(
          ready.counters[NativeGpuCounter.pluginReservedTargetBytes],
          8192,
        );
        expect(ready.counters[NativeGpuCounter.pluginOutstandingFrames], 1);
        expect(ready.counters[NativeGpuCounter.pluginWholeFrameReadbacks], 0);
        expect(
          ready.counters,
          isNot(contains(NativeGpuCounter.pluginDescriptorTickets)),
        );
        expect(ready.counters[NativeGpuCounter.controllerRpc], 0);
        expect(ready.counters[NativeGpuCounter.isolateRpc], 0);
        expect(ready.counters[NativeGpuCounter.dartOwnedSnapshots], 0);
        expect(ready.counters[NativeGpuCounter.visiblePages], 1);
        expect(
          trace.events.any(
            (event) => event.counters[NativeGpuCounter.controllerRpc] == 1,
          ),
          isTrue,
        );
        expect(
          trace.events.every(
            (event) =>
                event.counters[NativeGpuCounter.controllerRpc]! <= 3 &&
                event.counters[NativeGpuCounter.isolateRpc]! <= 8,
          ),
          isTrue,
        );
        expect(
          metadata.checks,
          hasLength(2),
          reason: 'Capture and publication full stamps.',
        );
        final json = trace.toJson();
        final samples = json['requestToProducerReadyMicros']! as Map;
        expect(samples['lifetimeCompletedSamples'], 1);
        expect(samples['p95'], lessThan(100000));
        expect(json['compositorDisplayVerified'], isFalse);
        expect(json['consumerGpuCompletionVerified'], isFalse);
        expect(jsonEncode(json), isNot(contains('/private')));
      } finally {
        await cleanUp(tester, controller, document, plugin);
      }
    },
  );

  testWidgets(
    'slow producer gets one early probe then retains the 100 ms cadence',
    (tester) async {
      final plugin = MeasuredPlugin()
        ..publish = false
        ..install();
      final document = await open(tester, GpuTransport());
      final controller = controllerFor(document, FakePresentation());
      final started = tester.binding.clock.now();
      try {
        configure(controller);
        await until(tester, () => plugin.statusCalls == 2);
        expect(
          controller.trace,
          isNull,
          reason: 'Tracing is disabled by default.',
        );
        expect(controller.canDisplay, isFalse);
        await tester.pump(const Duration(milliseconds: 25));
        expect(
          plugin.statusCalls,
          2,
          reason: 'No hot retry after the early probe.',
        );
        plugin.publish = true;
        final elapsed = tester.binding.clock.now().difference(started);
        await tester.pump(const Duration(milliseconds: 99) - elapsed);
        expect(plugin.statusCalls, 2);
        expect(controller.canDisplay, isFalse);
        await tester.pump(const Duration(milliseconds: 1));
        expect(plugin.statusCalls, 3);
        expect(controller.canDisplay, isTrue);
      } finally {
        await cleanUp(tester, controller, document, plugin);
      }
    },
  );

  for (final stop in ['pause', 'close']) {
    testWidgets('$stop during submit blocks the early publication probe', (
      tester,
    ) async {
      final plugin = MeasuredPlugin()
        ..submitGate = Completer<void>()
        ..acceptedBeforeReply = true
        ..install();
      final document = await open(tester, GpuTransport());
      final metadata = FakePresentation();
      final trace = NativeGpuTrace();
      final controller = controllerFor(document, metadata, trace: trace);
      try {
        configure(controller);
        await until(tester, () => plugin.submissions.isNotEmpty);
        final stopping = stop == 'pause'
            ? controller.pause()
            : controller.close();
        await tester.pump();
        await stopping;
        plugin.submitGate!.complete();
        await tester.pump();
        expect(plugin.statusCalls, 1);
        expect(controller.canDisplay, isFalse);
        expect(trace.counts[NativeGpuTraceStage.producerReady] ?? 0, 0);
        expect(trace.counts[NativeGpuTraceStage.superseded], 1);
        expect(
          metadata.released,
          isEmpty,
          reason: 'Accepted handles remain plugin owned.',
        );
        if (stop == 'pause') {
          controller.resume();
          await until(tester, () => controller.canDisplay);
          expect(trace.counts[NativeGpuTraceStage.producerReady], 1);
        }
      } finally {
        await cleanUp(tester, controller, document, plugin);
      }
    });
  }

  testWidgets(
    'superseded accepted batch cannot publish or sample the old generation',
    (tester) async {
      final plugin = MeasuredPlugin()
        ..submitGate = Completer<void>()
        ..acceptedBeforeReply = true
        ..install();
      final document = await open(tester, GpuTransport());
      final trace = NativeGpuTrace();
      final controller = controllerFor(
        document,
        FakePresentation(),
        trace: trace,
      );
      try {
        configure(controller);
        await until(tester, () => plugin.submissions.isNotEmpty);
        configure(controller, width: 350);
        plugin.submitGate!.complete();
        await until(tester, () => controller.canDisplay);
        expect(plugin.submissions, hasLength(2));
        expect(
          plugin.statusCalls,
          3,
          reason: 'Two device checks and only the latest publication probe.',
        );
        expect(trace.counts[NativeGpuTraceStage.accepted], 2);
        expect(trace.counts[NativeGpuTraceStage.superseded], 1);
        final ready = trace.events.where(
          (event) => event.stage == NativeGpuTraceStage.producerReady,
        );
        expect(ready.single.generation, 2);
        final samples = trace.toJson()['requestToProducerReadyMicros']! as Map;
        expect(samples['lifetimeCompletedSamples'], 1);
        expect(plugin.releasedByPlugin, {101});
      } finally {
        await cleanUp(tester, controller, document, plugin);
      }
    },
  );

  testWidgets('early probe validates full source stamps before producerReady', (
    tester,
  ) async {
    final plugin = MeasuredPlugin()
      ..submitGate = Completer<void>()
      ..acceptedBeforeReply = true
      ..install();
    final document = await open(tester, GpuTransport());
    final metadata = FakePresentation();
    final trace = NativeGpuTrace();
    final controller = controllerFor(document, metadata, trace: trace);
    try {
      configure(controller);
      await until(tester, () => plugin.submissions.isNotEmpty);
      metadata.current = false;
      plugin.submitGate!.complete();
      await tester.pump();
      expect(controller.needsCpuFallback, isTrue);
      expect(trace.counts[NativeGpuTraceStage.producerReady] ?? 0, 0);
      expect(trace.counts[NativeGpuTraceStage.stale], 1);
      expect(trace.counts[NativeGpuTraceStage.failed], 1);
      expect(
        trace.events
            .where((event) => event.stage == NativeGpuTraceStage.fallback)
            .single
            .failure,
        NativeGpuTraceFailure.stale,
      );
      expect(metadata.released, isEmpty);
      expect(plugin.releasedByPlugin, {101});
    } finally {
      await cleanUp(tester, controller, document, plugin);
    }
  });

  testWidgets(
    'observer exceptions cannot skip caller snapshot cleanup on failure',
    (tester) async {
      final plugin = MeasuredPlugin()..install();
      final document = await open(tester, GpuTransport());
      final metadata = FakePresentation()..current = false;
      final trace = NativeGpuTrace(
        onEvent: (_) => throw StateError('/private/observer'),
      );
      final controller = controllerFor(document, metadata, trace: trace);
      final before = NativeTextureController.retainedOwnedSnapshots;
      try {
        configure(controller);
        await until(tester, () => controller.needsCpuFallback);
        expect(plugin.submissions, isEmpty);
        expect(metadata.released, [101]);
        expect(NativeTextureController.retainedOwnedSnapshots, before);
        expect(trace.toJson()['observerFailures'], greaterThan(0));
        expect(jsonEncode(trace.toJson()), isNot(contains('/private')));
      } finally {
        await cleanUp(tester, controller, document, plugin);
      }
    },
  );

  testWidgets(
    'slow publication retains its deadline and classified timeout trace',
    (tester) async {
      final plugin = MeasuredPlugin()
        ..publish = false
        ..install();
      final document = await open(tester, GpuTransport());
      final trace = NativeGpuTrace();
      final controller = controllerFor(
        document,
        FakePresentation(),
        trace: trace,
        timeout: const Duration(milliseconds: 120),
      );
      try {
        configure(controller);
        await until(tester, () => plugin.statusCalls == 2);
        await tester.pump(const Duration(milliseconds: 120));
        expect(controller.needsCpuFallback, isTrue);
        expect(trace.counts[NativeGpuTraceStage.timeout], 1);
        expect(trace.counts[NativeGpuTraceStage.producerReady] ?? 0, 0);
        expect(
          trace.events
              .where((event) => event.stage == NativeGpuTraceStage.fallback)
              .single
              .failure,
          NativeGpuTraceFailure.timeout,
        );
        final stoppedCalls = plugin.calls.length;
        await tester.pump(const Duration(seconds: 1));
        expect(
          plugin.calls,
          hasLength(stoppedCalls),
          reason: 'No retry loop after the deadline.',
        );
      } finally {
        await cleanUp(tester, controller, document, plugin);
      }
    },
  );
}
