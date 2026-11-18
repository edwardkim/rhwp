// 합성 계약: bounded event/sample ring, monotonic clock, typed privacy와 observer 격리.
// 주석 갱신: 2026-11-10 KST. 신설 파일이므로 직전 파일 commit 없음.
// 문서 시각 좌표·실물 GPU·Flutter display의 측정 기대값이 아니다.

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_gpu_trace.dart';

void main() {
  test(
    'bounded rings distinguish lifetime counts and retained sample percentiles',
    () {
      var clock = 0;
      final trace = NativeGpuTrace(capacity: 3, clockMicros: () => clock);
      for (var generation = 1; generation <= 5; generation++) {
        trace.record(NativeGpuTraceStage.invalidated, generation: generation);
        clock += generation * 10;
        trace.record(NativeGpuTraceStage.producerReady, generation: generation);
      }
      final json = trace.toJson();
      expect(trace.events, hasLength(3));
      expect(json['droppedEvents'], 7);
      expect(trace.counts[NativeGpuTraceStage.producerReady], 5);
      expect(json['requestToProducerReadyMicros'], {
        'scope': 'retainedCompletedSamples',
        'method': 'nearestRank',
        'retainedSamples': 3,
        'lifetimeCompletedSamples': 5,
        'droppedSamples': 2,
        'p50': 40,
        'p95': 50,
        'p99': 50,
      });
      expect(() => trace.events.clear(), throwsUnsupportedError);
      expect(() => trace.counts.clear(), throwsUnsupportedError);
    },
  );

  test(
    'only the matching latest generation contributes a completed sample',
    () {
      var clock = 10;
      final trace = NativeGpuTrace(clockMicros: () => clock);
      trace.record(NativeGpuTraceStage.invalidated, generation: 1);
      clock = 20;
      trace.record(NativeGpuTraceStage.invalidated, generation: 2);
      clock = 30;
      trace.record(NativeGpuTraceStage.producerReady, generation: 1);
      clock = 40;
      trace.record(NativeGpuTraceStage.producerReady, generation: 2);
      clock = 50;
      trace.record(NativeGpuTraceStage.producerReady, generation: 2);
      final samples = trace.toJson()['requestToProducerReadyMicros']! as Map;
      expect(samples['lifetimeCompletedSamples'], 1);
      expect(samples['p95'], 20);
    },
  );

  test(
    'counter schema is typed and numeric; output has no arbitrary detail text',
    () {
      final trace = NativeGpuTrace(clockMicros: () => 100);
      trace.record(
        NativeGpuTraceStage.statusPolled,
        generation: 1,
        counters: {
          NativeGpuCounter.dartOwnedSnapshots: 4,
          NativeGpuCounter.pluginReservedTargetBytes: -1,
          NativeGpuCounter.nativeFrameBytes: 9007199254740992,
        },
      );
      expect(trace.events.single.counters, {
        NativeGpuCounter.dartOwnedSnapshots: 4,
      });
      expect(
        () => trace.events.single.counters.clear(),
        throwsUnsupportedError,
      );
      final output = jsonEncode(trace.toJson());
      expect(output, isNot(contains('libraryPath')));
      expect(output, isNot(contains('documentId')));
      expect(output, isNot(contains('errorMessage')));
      expect(trace.toJson()['consumerGpuCompletionVerified'], isFalse);
      expect(trace.toJson()['compositorDisplayVerified'], isFalse);
    },
  );

  test(
    'throwing observers and clocks never escape to the rendering caller',
    () {
      var clock = 100;
      final trace = NativeGpuTrace(
        clockMicros: () => clock,
        onEvent: (_) => throw StateError('/private/library/path'),
      );
      expect(
        () => trace.record(NativeGpuTraceStage.invalidated, generation: 1),
        returnsNormally,
      );
      clock = 90;
      trace.record(NativeGpuTraceStage.accepted, generation: 1);
      clock = 110;
      trace.record(NativeGpuTraceStage.producerReady, generation: 1);
      expect(trace.toJson()['observerFailures'], 2);
      expect(trace.toJson()['instrumentationFailures'], 1);
      expect(jsonEncode(trace.toJson()), isNot(contains('private')));
      final broken = NativeGpuTrace(
        clockMicros: () => throw StateError('broken clock'),
      );
      expect(
        () => broken.record(NativeGpuTraceStage.invalidated, generation: 1),
        returnsNormally,
      );
      expect(broken.toJson()['instrumentationFailures'], 1);
    },
  );

  test(
    'reentrant observer is notified once and all output remains bounded',
    () {
      late NativeGpuTrace trace;
      trace = NativeGpuTrace(
        capacity: 2,
        clockMicros: () => 1,
        onEvent: (_) {
          trace.record(NativeGpuTraceStage.statusPolled, generation: 1);
        },
      );
      trace.record(NativeGpuTraceStage.invalidated, generation: 1);
      expect(trace.events, hasLength(2));
      expect(trace.counts[NativeGpuTraceStage.statusPolled], 1);
      expect(trace.toJson()['observerFailures'], 0);
    },
  );

  test('capacity limits are explicit', () {
    expect(() => NativeGpuTrace(capacity: 0), throwsArgumentError);
    expect(() => NativeGpuTrace(capacity: 513), throwsArgumentError);
    expect(NativeGpuTrace().toJson()['requestToProducerReadyMicros'], {
      'scope': 'retainedCompletedSamples',
      'method': 'nearestRank',
      'retainedSamples': 0,
      'lifetimeCompletedSamples': 0,
      'droppedSamples': 0,
      'p50': null,
      'p95': null,
      'p99': null,
    });
  });

  test('default recorder stays at 128 events after long input bursts', () {
    var clock = 0;
    final trace = NativeGpuTrace(clockMicros: () => clock++);
    for (var generation = 1; generation <= 1000; ++generation) {
      trace.record(NativeGpuTraceStage.invalidated, generation: generation);
      trace.record(NativeGpuTraceStage.producerReady, generation: generation);
    }
    expect(trace.events, hasLength(128));
    expect(trace.toJson()['droppedEvents'], 1872);
    final samples = trace.toJson()['requestToProducerReadyMicros']! as Map;
    expect(samples['retainedSamples'], 128);
    expect(samples['lifetimeCompletedSamples'], 1000);
    expect(samples['droppedSamples'], 872);
  });

  for (final terminal in [
    NativeGpuTraceStage.superseded,
    NativeGpuTraceStage.failed,
    NativeGpuTraceStage.closeRequested,
  ]) {
    test('$terminal cannot later complete a request latency sample', () {
      var clock = 0;
      final trace = NativeGpuTrace(clockMicros: () => clock++);
      trace.record(NativeGpuTraceStage.invalidated, generation: 1);
      trace.record(terminal, generation: 1);
      trace.record(NativeGpuTraceStage.producerReady, generation: 1);
      final samples = trace.toJson()['requestToProducerReadyMicros']! as Map;
      expect(samples['lifetimeCompletedSamples'], 0);
    });
  }
}
