import 'native_presentation.dart';

enum NativeGpuCounter {
  dartOwnedSnapshots,
  controllerRpc,
  isolateRpc,
  visiblePages,
  pluginProducerOwnedBytes,
  pluginImportPendingBytes,
  pluginReservedTargetBytes,
  pluginOutstandingFrames,
  pluginDescriptorTickets,
  pluginWholeFrameReadbacks,
  nativeFrameBytes,
  nativeSnapshotHandles,
  nativeLeases,
  nativeGpuCacheHits,
  nativeGpuCacheMisses,
  nativeGpuCacheEvictions,
  nativeGpuCachedTiles,
  nativeGpuCachedBytes,
  nativeGpuManagedTargets,
  nativeGpuManagedBytes,
  nativeGpuCanonicalReplays,
  nativeGpuViewportFallbacks,
  nativeGpuTargetAdmissionFallbacks,
}

/// A measurement belonging to the published frame, not a live post-trim query.
final class NativeGpuCacheSample {
  NativeGpuCacheSample._(
    this.backend,
    this.stamp,
    this.members,
    Map<String, int> counters,
  ) : counters = Map.unmodifiable(counters);
  static const scope = 'publishedFramePostRenderSnapshot';
  static const maximum = 9007199254740991;
  static const counterNames = {
    'cacheHits': NativeGpuCounter.nativeGpuCacheHits,
    'cacheMisses': NativeGpuCounter.nativeGpuCacheMisses,
    'cacheEvictions': NativeGpuCounter.nativeGpuCacheEvictions,
    'cachedTiles': NativeGpuCounter.nativeGpuCachedTiles,
    'cachedBytes': NativeGpuCounter.nativeGpuCachedBytes,
    'managedTargets': NativeGpuCounter.nativeGpuManagedTargets,
    'managedBytes': NativeGpuCounter.nativeGpuManagedBytes,
    'canonicalReplays': NativeGpuCounter.nativeGpuCanonicalReplays,
    'viewportFallbacks': NativeGpuCounter.nativeGpuViewportFallbacks,
    'targetAdmissionFallbacks':
        NativeGpuCounter.nativeGpuTargetAdmissionFallbacks,
  };
  final String backend;
  final NativePresentationStamp stamp;
  final int members;
  final Map<String, int> counters;

  /// Missing fields stay unknown. Group totals require the field in both tiles.
  static NativeGpuCacheSample? fromStatus(
    Map<String, Object?> status,
    NativePresentationStamp expected,
  ) {
    try {
      NativePresentationStamp.fromJson(expected.toJson());
      if (status['closed'] == true || status['closing'] == true) return null;
      final raw = status['producerDiagnosticsSamples'];
      final samples = raw ?? [status['producerDiagnostics']];
      if (samples is! List ||
          samples.isEmpty ||
          samples.length > 2 ||
          (raw != null && samples.length != 2)) {
        return null;
      }
      String? backend;
      final values = <Map>[];
      for (final sample in samples) {
        if (sample is! Map ||
            sample['schemaVersion'] != 1 ||
            sample['scope'] != scope ||
            !expected.matches(sample['stamp']) ||
            sample['counters'] is! Map) {
          return null;
        }
        final source = sample['backend'];
        if (!const {
              'openGlGanesh',
              'metalGanesh',
              'd3d12Ganesh',
            }.contains(source) ||
            (backend != null && source != backend)) {
          return null;
        }
        final checked = NativePresentationStamp.fromJson(
          Map<String, Object?>.from(sample['stamp'] as Map),
        );
        if (checked.toJson().values.any((v) => v is int && v > maximum)) {
          return null;
        }
        backend = source as String;
        values.add(sample['counters'] as Map);
      }
      final counters = <String, int>{};
      for (final name in counterNames.keys) {
        var sum = 0;
        var known = true;
        for (final value in values) {
          final counter = value[name];
          if (counter is! int ||
              counter < 0 ||
              counter > maximum ||
              sum > maximum - counter) {
            known = false;
            break;
          }
          sum += counter;
        }
        if (known) counters[name] = sum;
      }
      return counters.isEmpty
          ? null
          : NativeGpuCacheSample._(
              backend!,
              expected,
              samples.length,
              counters,
            );
    } catch (_) {
      return null;
    }
  }

  Map<NativeGpuCounter, int> get traceCounters => {
    for (final entry in counters.entries) counterNames[entry.key]!: entry.value,
  };
  Map<String, Object?> toJson() => {
    'schemaVersion': 1,
    'backend': backend,
    'scope': scope,
    'stamp': stamp.toJson(),
    'members': members,
    'counters': counters,
  };
}
