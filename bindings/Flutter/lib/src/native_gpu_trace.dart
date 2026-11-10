import 'dart:collection';
import 'dart:math' as math;

/// These are producer/controller milestones, never compositor display or fences.
enum NativeGpuTraceStage {
  configured,
  invalidated,
  captureStarted,
  captureCompleted,
  submitStarted,
  accepted,
  statusPolled,
  producerReady,
  resourceSampled,
  superseded,
  backpressure,
  stale,
  timeout,
  fallback,
  failed,
  closeRequested,
}

/// No native error text, document content or module/file paths enter the trace.
enum NativeGpuTraceFailure {
  timeout,
  stale,
  backpressure,
  unsupported,
  device,
  other,
}

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
}

final class NativeGpuTraceEvent {
  NativeGpuTraceEvent._(
    this.stage,
    this.generation,
    this.elapsedMicros,
    this.failure,
    Map<NativeGpuCounter, int> counters,
  ) : counters = Map.unmodifiable(counters);
  final NativeGpuTraceStage stage;
  final int generation, elapsedMicros;
  final NativeGpuTraceFailure? failure;
  final Map<NativeGpuCounter, int> counters;

  Map<String, Object?> toJson() => {
    'stage': stage.name,
    'generation': generation,
    'elapsedMicros': elapsedMicros,
    if (failure != null) 'failure': failure!.name,
    if (counters.isNotEmpty)
      'counters': {
        for (final entry in counters.entries) entry.key.name: entry.value,
      },
  };
}

/// Optional bounded diagnostics. No recorder means no clock/ring/observer work.
///
/// Lifetime event counts and completed sample counts are separate from the
/// retained event/sample windows. Percentiles use nearest-rank samples retained
/// in this recorder, not every frame and not hardware display latency.
final class NativeGpuTrace {
  NativeGpuTrace({
    this.capacity = 128,
    int Function()? clockMicros,
    this.onEvent,
  }) {
    if (capacity < 1 || capacity > 512) {
      throw ArgumentError.value(
        capacity,
        'capacity',
        'Must be between 1 and 512.',
      );
    }
    final watch = Stopwatch()..start();
    _clock = clockMicros ?? (() => watch.elapsedMicroseconds);
  }
  final int capacity;
  final void Function(NativeGpuTraceEvent event)? onEvent;
  late final int Function() _clock;
  final _events = Queue<NativeGpuTraceEvent>();
  final _samples = Queue<int>();
  final _counts = <NativeGpuTraceStage, int>{};
  int _lastMicros = 0, _droppedEvents = 0, _completed = 0;
  int _instrumentationFailures = 0, _observerFailures = 0;
  int? _origin, _requestGeneration, _requestedAt;
  bool _notifying = false;

  List<NativeGpuTraceEvent> get events => List.unmodifiable(_events);
  Map<NativeGpuTraceStage, int> get counts => Map.unmodifiable(_counts);

  /// Observers and optional clocks cannot throw into rendering/lease cleanup.
  void record(
    NativeGpuTraceStage stage, {
    required int generation,
    NativeGpuTraceFailure? failure,
    Map<NativeGpuCounter, int> counters = const {},
  }) {
    try {
      final now = _clock();
      if (generation < 0 || now < 0 || (_origin != null && now < _lastMicros)) {
        ++_instrumentationFailures;
        return;
      }
      _origin ??= now;
      _lastMicros = now;
      final filtered = <NativeGpuCounter, int>{
        for (final entry in counters.entries)
          if (entry.value >= 0 && entry.value <= 9007199254740991)
            entry.key: entry.value,
      };
      final event = NativeGpuTraceEvent._(
        stage,
        generation,
        now - _origin!,
        failure,
        filtered,
      );
      _counts.update(stage, (count) => count + 1, ifAbsent: () => 1);
      if (_events.length == capacity) {
        _events.removeFirst();
        ++_droppedEvents;
      }
      _events.add(event);
      if (stage == NativeGpuTraceStage.invalidated) {
        _requestGeneration = generation;
        _requestedAt = now;
      } else if (stage == NativeGpuTraceStage.producerReady &&
          generation == _requestGeneration &&
          _requestedAt != null) {
        if (_samples.length == capacity) _samples.removeFirst();
        _samples.add(now - _requestedAt!);
        ++_completed;
        _requestGeneration = null;
        _requestedAt = null;
      } else if (generation == _requestGeneration &&
          (stage == NativeGpuTraceStage.superseded ||
              stage == NativeGpuTraceStage.failed ||
              stage == NativeGpuTraceStage.closeRequested)) {
        _requestGeneration = null;
        _requestedAt = null;
      }
      if (!_notifying && onEvent != null) {
        _notifying = true;
        try {
          onEvent!(event);
        } catch (_) {
          ++_observerFailures;
        } finally {
          _notifying = false;
        }
      }
    } catch (_) {
      ++_instrumentationFailures;
    }
  }

  Map<String, Object?> toJson() {
    final samples = _samples.toList()..sort();
    int? percentile(int percent) => samples.isEmpty
        ? null
        : samples[math.max(0, (samples.length * percent / 100).ceil() - 1)];
    return {
      'schemaVersion': 1,
      'kind': 'rhwp-gpu-producer-trace',
      'clock': 'monotonicMicroseconds',
      'capacity': capacity,
      'retainedEvents': _events.length,
      'droppedEvents': _droppedEvents,
      'lifetimeCounts': {
        for (final entry in _counts.entries) entry.key.name: entry.value,
      },
      'requestToProducerReadyMicros': {
        'scope': 'retainedCompletedSamples',
        'method': 'nearestRank',
        'retainedSamples': samples.length,
        'lifetimeCompletedSamples': _completed,
        'droppedSamples': _completed - samples.length,
        'p50': percentile(50),
        'p95': percentile(95),
        'p99': percentile(99),
      },
      'instrumentationFailures': _instrumentationFailures,
      'observerFailures': _observerFailures,
      'compositorDisplayVerified': false,
      'consumerGpuCompletionVerified': false,
      'events': [for (final event in _events) event.toJson()],
    };
  }
}
