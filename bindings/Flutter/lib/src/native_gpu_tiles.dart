import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'native_ffi.dart';

/// Physical output tiles, not Native page tiles or reusable consumer targets.
/// A pair consumes the existing two native registration/producer slots.
class NativeGpuTilePlan {
  NativeGpuTilePlan({
    required int width,
    required int height,
    this.windowsTwoSurfaceCharge = false,
  }) {
    if (width <= 0 ||
        height <= 0 ||
        width > 8192 ||
        height > 8192 ||
        width * height > 8 * 1024 * 1024) {
      throw const NativeCallException(
        'viewportLimit',
        'The viewport exceeds the bounded two-tile extent.',
      );
    }
    List<Rect>? selected;
    for (final horizontal in [width >= height, width < height]) {
      final extent = horizontal ? width : height;
      if (extent < 2) continue;
      final split = (extent / 2).ceil();
      final candidate = horizontal
          ? [
              Rect.fromLTWH(0, 0, split.toDouble(), height.toDouble()),
              Rect.fromLTWH(
                split.toDouble(),
                0,
                (width - split).toDouble(),
                height.toDouble(),
              ),
            ]
          : [
              Rect.fromLTWH(0, 0, width.toDouble(), split.toDouble()),
              Rect.fromLTWH(
                0,
                split.toDouble(),
                width.toDouble(),
                (height - split).toDouble(),
              ),
            ];
      if (candidate.every(_admitted)) {
        selected = candidate;
        break;
      }
    }
    if (selected == null) {
      throw const NativeCallException(
        'viewportLimit',
        'The viewport cannot fit two native target budgets.',
      );
    }
    physicalTiles = List.unmodifiable(selected);
  }
  final bool windowsTwoSurfaceCharge;
  late final List<Rect> physicalTiles;
  static int windowsFrameCharge(int width, int height) {
    if (width <= 0 || height <= 0 || width > 4096 || height > 4096) return 0;
    final required = ((width * 4 + 255) ~/ 256 * 256) * height;
    var surface = 64 * 1024;
    while (surface < required && surface < 8 * 1024 * 1024) {
      surface *= 2;
    }
    return surface >= required ? surface * 2 : 0;
  }

  bool _admitted(Rect tile) {
    final width = tile.width.toInt(), height = tile.height.toInt();
    if (width > 4096 || height > 4096 || width * height > 4 * 1024 * 1024) {
      return false;
    }
    // The common plan also covers Metal's 256-byte row alignment.
    final charge = windowsTwoSurfaceCharge
        ? windowsFrameCharge(width, height)
        : ((width * 4 + 255) ~/ 256 * 256) * height;
    return charge > 0 && charge <= 16 * 1024 * 1024;
  }
}

/// Adapts one P111 controller/owned snapshot batch to the native tile-pair API.
/// It never creates a second document/view owner or transfers an ID twice.
class NativeTileGroupChannel extends MethodChannel {
  NativeTileGroupChannel(this.delegate, {required this.windows})
    : super(delegate.name);
  final MethodChannel delegate;
  final bool windows;
  NativeGpuTilePlan? _plan;
  List<int> _textures = const [];
  int? _group, _device;
  List<int> get textures => _textures;
  List<Rect> get physicalTiles => _plan?.physicalTiles ?? const [];
  void plan(int width, int height) {
    _plan = NativeGpuTilePlan(
      width: width,
      height: height,
      windowsTwoSurfaceCharge: windows,
    );
  }

  @override
  Future<T?> invokeMethod<T>(String method, [dynamic arguments]) async {
    final args = Map<String, Object?>.from(arguments as Map? ?? const {});
    if (method == 'create') {
      final caps = await delegate.invokeMapMethod<String, Object?>(
        'capabilities',
        const {},
      );
      if (caps?['tileGroups'] != true ||
          caps?['sharedSnapshotLeases'] != true ||
          caps?['maxTileGroupTiles'] != 2 ||
          caps?['maxTileGroupSnapshots'] != 4) {
        throw const NativeCallException(
          'unsupportedPresentation',
          'This plugin does not support bounded tile groups.',
        );
      }
      final raw = await delegate.invokeMapMethod<String, Object?>(
        'createGroup',
        args,
      );
      final result = _members(raw, creating: true);
      return result as T?;
    }
    if (method == 'submit') {
      final width = args['width'], height = args['height'];
      if (width is! int || height is! int) {
        throw const NativeCallException(
          'viewportLimit',
          'Invalid physical viewport.',
        );
      }
      plan(width, height);
      final placements = args['placements'];
      if (placements is! List || _textures.length != 2) {
        throw const NativeCallException(
          'invalidTileReply',
          'Missing tile members or page placements.',
        );
      }
      final tiles = <Map<String, Object?>>[];
      for (var index = 0; index < 2; ++index) {
        final rect = physicalTiles[index];
        tiles.add({
          'textureId': _textures[index],
          'width': rect.width.toInt(),
          'height': rect.height.toInt(),
          'placements': [
            for (final item in placements)
              {
                ...Map<String, Object?>.from(item as Map),
                'originX': (item['originX'] as num).toDouble() - rect.left,
                'originY': (item['originY'] as num).toDouble() - rect.top,
              },
          ],
        });
      }
      final reply = await delegate.invokeMapMethod<String, Object?>(
        'submitGroup',
        {
          'groupId': _group,
          'deviceGeneration': _device,
          'stamp': args['stamp'],
          'clearColor': args['clearColor'],
          'tiles': tiles,
        },
      );
      if (reply == null) {
        throw const NativeCallException(
          'invalidTileReply',
          'Missing group acceptance receipt.',
        );
      }
      if (reply['accepted'] == true) {
        final ids = reply['acceptedSnapshots'];
        final expected = placements.map((v) => (v as Map)['snapshot']).toSet();
        if (ids is! List ||
            ids.length != expected.length ||
            ids.toSet().length != ids.length ||
            !setEquals(ids.toSet(), expected) ||
            reply['acceptedStamp'] is! Map ||
            args['stamp'] is! Map ||
            !mapEquals(reply['acceptedStamp'] as Map, args['stamp'] as Map)) {
          // The Native call may already own the entire batch. Leave ownership
          // ambiguous and let the controller reconcile the group status.
          throw const NativeCallException(
            'invalidTileReply',
            'The complete group acceptance receipt is required.',
          );
        }
      }
      return reply as T?;
    }
    final groupMethod = switch (method) {
      'status' => 'statusGroup',
      'close' => 'closeGroup',
      'retireAck' => 'retireGroupAck',
      'setCovered' => 'setCoveredGroup',
      _ => null,
    };
    if (groupMethod == null) return delegate.invokeMethod<T>(method, arguments);
    final raw = await delegate.invokeMapMethod<String, Object?>(groupMethod, {
      ...args,
      'groupId': _group,
      'deviceGeneration': _device,
    });
    return _members(raw) as T?;
  }

  Map<String, Object?> _members(
    Map<String, Object?>? raw, {
    bool creating = false,
  }) {
    final id = raw?['groupId'],
        device = raw?['deviceGeneration'],
        members = raw?['members'];
    if (id is! int ||
        id <= 0 ||
        device is! int ||
        device <= 0 ||
        members is! List ||
        members.length != 2) {
      throw const NativeCallException(
        'invalidTileReply',
        'Malformed two-tile group receipt.',
      );
    }
    if (members.any((v) => v is! Map)) {
      throw const NativeCallException(
        'invalidTileReply',
        'Malformed tile member status.',
      );
    }
    final values = members
        .map((v) => Map<String, Object?>.from(v as Map))
        .toList();
    final ids = values.map((v) => v['textureId']).toList();
    if (ids.any((v) => v is! int || v <= 0) ||
        ids[0] == ids[1] ||
        id != ids[0] ||
        (!creating &&
            (_group != id ||
                _device != device ||
                !listEquals(ids, _textures))) ||
        values.any(
          (v) =>
              v['deviceGeneration'] != null && v['deviceGeneration'] != device,
        )) {
      throw const NativeCallException(
        'contextChanged',
        'Tile member identity/device generation changed.',
      );
    }
    if (creating) {
      _group = id;
      _device = device;
      _textures = List<int>.unmodifiable(ids.cast<int>());
    }
    bool all(String key) => values.every((v) => v[key] == true);
    bool any(String key) => values.any((v) => v[key] == true);
    Map<String, Object?> stamp(String key) {
      final first = values.first[key];
      return first is Map &&
              values.every(
                (v) => v[key] is Map && mapEquals(first, v[key] as Map),
              )
          ? Map<String, Object?>.from(first)
          : const {};
    }

    final result = <String, Object?>{
      ...values.first,
      'textureId': id,
      'deviceGeneration': device,
      'acceptedStamp': stamp('acceptedStamp'),
      'renderedStamp': stamp('renderedStamp'),
      'registered': any('registered'),
      'unregistered': values.every(
        (v) => v['registered'] == false || v['unregistered'] == true,
      ),
    };
    for (final key in [
      'ready',
      'apiReady',
      'hasFrame',
      'closed',
      'ownerFinished',
      'cleanupComplete',
      'acknowledged',
      'viewBound',
      'layerReady',
      'nativeViewLayerDetached',
      'covered',
      'accepted',
      'closeAcknowledgementRequired',
    ]) {
      result[key] = all(key);
    }
    for (final key in [
      'pending',
      'inFlight',
      'closing',
      'quarantined',
      'deviceFailure',
    ]) {
      result[key] = any(key);
    }
    result['closing'] = any('closing') || any('closed');
    final snapshots = <int>{};
    for (final value in values) {
      final ids = value['acceptedSnapshots'];
      if (ids is List) {
        for (final id in ids) {
          if (id is int) snapshots.add(id);
        }
      }
    }
    final outerIds = raw?['acceptedSnapshots'];
    if (outerIds is List) {
      for (final id in outerIds) {
        if (id is int) snapshots.add(id);
      }
    }
    if (snapshots.isNotEmpty) {
      result['acceptedSnapshots'] = snapshots.toList()..sort();
    }
    for (final key in ['error', 'message', 'code', 'errorCode']) {
      final errors = values
          .map((v) => v[key])
          .whereType<String>()
          .where((v) => v.isNotEmpty);
      result[key] = errors.isEmpty ? '' : errors.first;
    }
    final serial = values.first['visibilitySerial'];
    result['visibilitySerial'] =
        values.every((v) => v['visibilitySerial'] == serial) ? serial : 0;
    for (final key in [
      'frames',
      'framesSubmitted',
      'acceptedSerial',
      'renderedSerial',
    ]) {
      final numbers = values.map((v) => v[key]).whereType<int>().toList();
      if (numbers.length == 2) result[key] = math.min(numbers[0], numbers[1]);
    }
    return result;
  }
}
