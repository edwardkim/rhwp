import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

const stamp = NativePresentationStamp(
  documentId: 1,
  revision: 2,
  fontGeneration: 3,
  viewGeneration: 4,
  deviceGeneration: 5,
  deviceScale: 1.25,
);
Map<String, Object?> sample({
  Map<String, Object?>? counters,
  String backend = 'openGlGanesh',
}) => {
  'schemaVersion': 1,
  'backend': backend,
  'scope': NativeGpuCacheSample.scope,
  'stamp': stamp.toJson(),
  'counters': counters ?? {'cacheHits': 0, 'managedBytes': 1024},
};
Map<String, Object?> status(Map<String, Object?> value) => {
  'producerDiagnostics': value,
};

void main() {
  test('known zero is retained while missing counters stay unknown', () {
    final value = NativeGpuCacheSample.fromStatus(status(sample()), stamp)!;
    expect(value.counters, {'cacheHits': 0, 'managedBytes': 1024});
    expect(value.counters.containsKey('cacheMisses'), isFalse);
    expect(value.toJson()['scope'], 'publishedFramePostRenderSnapshot');
  });
  test('full stamp rejects every stale identity component', () {
    for (final key in stamp.toJson().keys) {
      final value = sample();
      value['stamp'] = {
        ...stamp.toJson(),
        key: key == 'deviceScale' ? 2.0 : 99,
      };
      expect(
        NativeGpuCacheSample.fromStatus(status(value), stamp),
        isNull,
        reason: key,
      );
    }
  });
  test('malformed, unsupported and closed statuses are omitted', () {
    for (final change in [
      {'schemaVersion': 2},
      {'scope': 'liveDriverVram'},
      {'backend': 'cpu'},
      {'stamp': {}},
      {'counters': <String, Object?>{}},
    ]) {
      expect(
        NativeGpuCacheSample.fromStatus(
          status({...sample(), ...change}),
          stamp,
        ),
        isNull,
      );
    }
    expect(NativeGpuCacheSample.fromStatus({}, stamp), isNull);
    expect(
      NativeGpuCacheSample.fromStatus({
        ...status(sample()),
        'closed': true,
      }, stamp),
      isNull,
    );
  });
  test('negative floating boolean and oversized counters are unknown', () {
    final value = NativeGpuCacheSample.fromStatus(
      status(
        sample(
          counters: {
            'cacheHits': -1,
            'cacheMisses': 1.0,
            'cacheEvictions': true,
            'managedBytes': 9007199254740992,
            'cachedTiles': 0,
          },
        ),
      ),
      stamp,
    )!;
    expect(value.counters, {'cachedTiles': 0});
  });
  test('group totals require a measured field from both members', () {
    final value = NativeGpuCacheSample.fromStatus({
      'producerDiagnosticsSamples': [
        sample(counters: {'cacheHits': 2, 'managedBytes': 100}),
        sample(counters: {'cacheHits': 3}),
      ],
    }, stamp)!;
    expect(value.members, 2);
    expect(value.counters, {'cacheHits': 5});
  });
  test(
    'missing member mixed backend and overflow never become partial totals',
    () {
      for (final second in [null, sample(backend: 'metalGanesh')]) {
        expect(
          NativeGpuCacheSample.fromStatus({
            'producerDiagnosticsSamples': [sample(), second],
          }, stamp),
          isNull,
        );
      }
      final value = NativeGpuCacheSample.fromStatus({
        'producerDiagnosticsSamples': [
          sample(counters: {'cacheHits': 9007199254740991}),
          sample(counters: {'cacheHits': 1}),
        ],
      }, stamp);
      expect(value, isNull);
    },
  );
  test(
    'trace serializes measured counters with the validated stamp and scope',
    () {
      final trace = NativeGpuTrace(clockMicros: () => 10);
      final value = NativeGpuCacheSample.fromStatus(status(sample()), stamp)!;
      trace.record(
        NativeGpuTraceStage.producerReady,
        generation: 4,
        producerDiagnostics: value,
      );
      final event = (trace.toJson()['events'] as List).single as Map;
      expect(event['producerDiagnostics'], value.toJson());
      expect((event['counters'] as Map)['nativeGpuCacheHits'], 0);
      expect(
        (event['counters'] as Map).containsKey('nativeGpuCacheMisses'),
        isFalse,
      );
      expect(trace.toJson()['compositorDisplayVerified'], isFalse);
    },
  );
}
