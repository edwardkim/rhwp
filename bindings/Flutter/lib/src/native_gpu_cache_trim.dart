import 'native_ffi.dart';

/// A cache-trim receipt for managed Native GPU targets. It does not include
/// Flutter's immutable public images, driver allocations or consumer completion.
final class NativeGpuCacheTrimResult {
  const NativeGpuCacheTrimResult._({
    required this.supported,
    required this.textureId,
    required this.deviceGeneration,
    required this.requestId,
    required this.targetBytes,
    required this.before,
    required this.after,
    required this.trimmed,
    required this.pending,
    required this.busy,
    required this.reason,
    this.members = const [],
  });

  const NativeGpuCacheTrimResult.unavailable({
    required this.reason,
    this.textureId,
    this.deviceGeneration,
    this.requestId,
    this.targetBytes = 0,
  }) : supported = false,
       before = 0,
       after = 0,
       trimmed = 0,
       pending = false,
       busy = false,
       members = const [];

  static const int maxTargetBytes = 192 * 1024 * 1024;
  final bool supported;
  final int? textureId, deviceGeneration, requestId;
  final int targetBytes, before, after, trimmed;
  final bool pending, busy;
  final String reason;
  final List<NativeGpuCacheTrimResult> members;
  String get scope => 'managedGpuTargets';

  /// Tile-group byte counters are sums; targetBytes is the goal of each owner,
  /// so a completed group's after count may exceed a single owner's target.
  factory NativeGpuCacheTrimResult.fromJson(
    Map<String, Object?> value, {
    required int textureId,
    required int deviceGeneration,
    required int requestId,
    required int targetBytes,
    List<int> memberTextureIds = const [],
  }) {
    NativeCallException invalid() => const NativeCallException(
      'invalidCacheTrimReply',
      'GPU cache trim receipt does not match its owner, request or byte totals.',
    );
    NativeGpuCacheTrimResult member(
      Map item,
      int expectedTexture, {
      bool aggregate = false,
    }) {
      int integer(String key) {
        final field = item[key];
        if (field is! int || field < 0 || field > 0x7fffffffffffffff) {
          throw invalid();
        }
        return field;
      }

      final before = integer('before'), after = integer('after');
      final trimmed = integer('trimmed');
      if (integer('textureId') != expectedTexture ||
          integer('deviceGeneration') != deviceGeneration ||
          integer('requestId') != requestId ||
          integer('targetBytes') != targetBytes ||
          item['scope'] != 'managedGpuTargets' ||
          item['pending'] is! bool ||
          item['busy'] is! bool ||
          item['reason'] is! String ||
          after > before ||
          trimmed != before - after) {
        throw invalid();
      }
      if ((item['pending'] == true && item['busy'] != true) ||
          (!aggregate &&
              item['pending'] == false &&
              item['busy'] == false &&
              after > targetBytes)) {
        throw invalid();
      }
      return NativeGpuCacheTrimResult._(
        supported: true,
        textureId: expectedTexture,
        deviceGeneration: deviceGeneration,
        requestId: requestId,
        targetBytes: targetBytes,
        before: before,
        after: after,
        trimmed: trimmed,
        pending: item['pending'] as bool,
        busy: item['busy'] as bool,
        reason: item['reason'] as String,
      );
    }

    if (textureId <= 0 ||
        deviceGeneration <= 0 ||
        requestId <= 0 ||
        targetBytes < 0 ||
        targetBytes > maxTargetBytes) {
      throw invalid();
    }
    final root = member(
      value,
      textureId,
      aggregate: memberTextureIds.isNotEmpty,
    );
    if (memberTextureIds.isEmpty) {
      if (value.containsKey('members') || value.containsKey('groupId')) {
        throw invalid();
      }
      return root;
    }
    final items = value['members'];
    if (memberTextureIds.length != 2 ||
        memberTextureIds.any((id) => id <= 0) ||
        memberTextureIds.toSet().length != 2 ||
        value['groupId'] != textureId ||
        items is! List ||
        items.length != 2 ||
        items.any((item) => item is! Map)) {
      throw invalid();
    }
    final members = List<NativeGpuCacheTrimResult>.generate(
      2,
      (index) => member(items[index] as Map, memberTextureIds[index]),
      growable: false,
    );
    if (root.before != members.fold<int>(0, (sum, item) => sum + item.before) ||
        root.after != members.fold<int>(0, (sum, item) => sum + item.after) ||
        root.trimmed !=
            members.fold<int>(0, (sum, item) => sum + item.trimmed) ||
        root.pending != members.any((item) => item.pending) ||
        root.busy != members.any((item) => item.busy)) {
      throw invalid();
    }
    return NativeGpuCacheTrimResult._(
      supported: true,
      textureId: textureId,
      deviceGeneration: deviceGeneration,
      requestId: requestId,
      targetBytes: targetBytes,
      before: root.before,
      after: root.after,
      trimmed: root.trimmed,
      pending: root.pending,
      busy: root.busy,
      reason: root.reason,
      members: List.unmodifiable(members),
    );
  }
}
