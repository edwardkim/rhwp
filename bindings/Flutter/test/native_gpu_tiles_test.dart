import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_gpu_tiles.dart';
import 'package:rhwp_native_reference/src/native_ffi.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test('two targets cover a viewport rejected by one target without seams', () {
    final plan = NativeGpuTilePlan(width: 8192, height: 1024);
    expect(plan.physicalTiles, [
      const Rect.fromLTWH(0, 0, 4096, 1024),
      const Rect.fromLTWH(4096, 0, 4096, 1024),
    ]);
    expect(
      plan.physicalTiles[0].intersect(plan.physicalTiles[1]).isEmpty,
      isTrue,
    );
    expect(
      plan.physicalTiles.fold<double>(0, (n, r) => n + r.width * r.height),
      8192 * 1024,
    );
    final odd = NativeGpuTilePlan(width: 3199, height: 1001);
    expect(odd.physicalTiles[0].right, odd.physicalTiles[1].left);
    expect(odd.physicalTiles[1].right, 3199);
  });
  test(
    'vertical and fractional-DPR placement retain integer physical boundaries',
    () {
      final plan = NativeGpuTilePlan(width: 1024, height: 8192);
      expect(plan.physicalTiles[0].bottom, plan.physicalTiles[1].top);
      const dpr = 1.25;
      for (final rect in plan.physicalTiles) {
        expect((rect.top / dpr) * dpr, rect.top);
        expect((rect.height / dpr) * dpr, rect.height);
      }
    },
  );
  test(
    'Windows admission charges both aligned surfaces instead of raw RGBA only',
    () {
      expect(
        NativeGpuTilePlan.windowsFrameCharge(1920, 1080),
        16 * 1024 * 1024,
      );
      expect(NativeGpuTilePlan.windowsFrameCharge(4096, 1536), 0);
      final plan = NativeGpuTilePlan(
        width: 4098,
        height: 512,
        windowsTwoSurfaceCharge: true,
      );
      expect(plan.physicalTiles.length, 2);
      for (final tile in plan.physicalTiles) {
        expect(
          NativeGpuTilePlan.windowsFrameCharge(
            tile.width.toInt(),
            tile.height.toInt(),
          ),
          lessThanOrEqualTo(16 * 1024 * 1024),
        );
      }
      expect(
        () => NativeGpuTilePlan(
          width: 4096,
          height: 1536,
          windowsTwoSurfaceCharge: true,
        ),
        throwsA(isA<NativeCallException>()),
      );
    },
  );
  test(
    'more than two targets and empty extents fail before allocating a snapshot',
    () {
      for (final size in [(0, 2), (1, 1), (8193, 1), (4096, 4096)]) {
        expect(
          () => NativeGpuTilePlan(width: size.$1, height: size.$2),
          throwsA(isA<NativeCallException>()),
        );
      }
    },
  );

  const delegate = MethodChannel('rhwp-native/tile-contract');
  final calls = <MethodCall>[];
  const stamp = {
    'documentId': 1,
    'revision': 2,
    'fontGeneration': 3,
    'viewGeneration': 4,
    'deviceGeneration': 5,
    'deviceScale': 1.25,
  };
  Map<String, Object?> member(int id, {bool current = true}) => {
    'textureId': id,
    'deviceGeneration': 5,
    'ready': true,
    'hasFrame': current,
    'renderedStamp': current ? stamp : <String, Object?>{},
    'acceptedStamp': stamp,
    'acceptedSnapshots': [101, 103],
    'registered': true,
    'closeAcknowledgementRequired': true,
  };
  Map<String, Object?> group(List<Object?> members) => {
    'groupId': 11,
    'deviceGeneration': 5,
    'members': members,
  };
  var statuses = <Object?>[];
  var supported = true;
  setUp(() {
    calls.clear();
    supported = true;
    statuses = [member(11), member(12)];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(delegate, (call) async {
          calls.add(call);
          if (call.method == 'capabilities') {
            return {
              'tileGroups': supported,
              'sharedSnapshotLeases': true,
              'maxTileGroupTiles': 2,
              'maxTileGroupSnapshots': 4,
            };
          }
          if (call.method == 'createGroup' ||
              call.method == 'statusGroup' ||
              call.method == 'closeGroup' ||
              call.method == 'retireGroupAck' ||
              call.method == 'setCoveredGroup') {
            return group(statuses);
          }
          if (call.method == 'submitGroup') {
            return {
              'accepted': true,
              'acceptedStamp': stamp,
              'acceptedSnapshots': [101, 103],
            };
          }
          throw MissingPluginException();
        });
  });
  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(delegate, null);
  });
  test(
    'one capture batch is submitted once with both translated tile placements',
    () async {
      final channel = NativeTileGroupChannel(delegate, windows: false)
        ..plan(8192, 1024);
      final created = await channel.invokeMapMethod<String, Object?>('create', {
        'libraryPath': '/native',
      });
      expect(created?['textureId'], 11);
      expect(channel.textures, [11, 12]);
      final result = await channel.invokeMapMethod<String, Object?>('submit', {
        'textureId': 11,
        'width': 8192,
        'height': 1024,
        'stamp': stamp,
        'clearColor': 0xff55575a,
        'placements': [
          {'snapshot': 101, 'scale': 1.25, 'originX': 12.5, 'originY': -7.5},
          {'snapshot': 103, 'scale': 1.25, 'originX': 12.5, 'originY': 900.0},
        ],
      });
      expect(result?['acceptedSnapshots'], [101, 103]);
      expect(calls.where((c) => c.method == 'submitGroup').length, 1);
      final submitted = calls.last.arguments as Map;
      final tiles = submitted['tiles'] as List;
      expect((tiles[0]['placements'] as List)[0]['originX'], 12.5);
      expect((tiles[1]['placements'] as List)[0]['originX'], 12.5 - 4096);
      expect(
        (tiles[0]['placements'] as List)[0]['snapshot'],
        (tiles[1]['placements'] as List)[0]['snapshot'],
      );
      expect((tiles[1]['placements'] as List)[0]['scale'], 1.25);
    },
  );
  test(
    'a first tile publication never uncovers a pending second tile',
    () async {
      final channel = NativeTileGroupChannel(delegate, windows: false);
      await channel.invokeMethod<Object?>('create', {});
      statuses = [member(11), member(12, current: false)];
      final waiting = await channel.invokeMapMethod<String, Object?>('status', {
        'textureId': 11,
      });
      expect(waiting?['hasFrame'], isFalse);
      expect(waiting?['renderedStamp'], isEmpty);
      statuses = [member(11), member(12)];
      final ready = await channel.invokeMapMethod<String, Object?>('status', {
        'textureId': 11,
      });
      expect(ready?['hasFrame'], isTrue);
      expect(ready?['renderedStamp'], stamp);
      expect(ready?['acceptedSnapshots'], [101, 103]);
    },
  );
  test('a changed device in either tile invalidates the group', () async {
    final channel = NativeTileGroupChannel(delegate, windows: false);
    await channel.invokeMethod<Object?>('create', {});
    statuses = [
      member(11),
      {...member(12), 'deviceGeneration': 6},
    ];
    await expectLater(
      channel.invokeMethod<Object?>('status', {}),
      throwsA(isA<NativeCallException>()),
    );
  });
  test('close and native cover wait for both child receipts', () async {
    final channel = NativeTileGroupChannel(delegate, windows: false);
    await channel.invokeMethod<Object?>('create', {});
    statuses = [
      {
        ...member(11),
        'closed': true,
        'ownerFinished': true,
        'covered': true,
        'visibilitySerial': 9,
      },
      {
        ...member(12),
        'closed': false,
        'ownerFinished': false,
        'covered': false,
        'visibilitySerial': 8,
      },
    ];
    final close = await channel.invokeMapMethod<String, Object?>('close', {});
    expect(close?['closed'], isFalse);
    expect(close?['ownerFinished'], isFalse);
    final cover = await channel.invokeMapMethod<String, Object?>('setCovered', {
      'covered': true,
      'visibilitySerial': 9,
    });
    expect(cover?['covered'], isFalse);
    expect(cover?['visibilitySerial'], 0);
  });
  test(
    'unsupported plugins fail explicitly before native group creation',
    () async {
      supported = false;
      final channel = NativeTileGroupChannel(delegate, windows: false);
      await expectLater(
        channel.invokeMethod<Object?>('create', {}),
        throwsA(isA<NativeCallException>()),
      );
      expect(calls.map((c) => c.method), ['capabilities']);
    },
  );
  test(
    'duplicate native member IDs cannot alias two tiles to one owner',
    () async {
      statuses = [member(11), member(11)];
      final channel = NativeTileGroupChannel(delegate, windows: false);
      await expectLater(
        channel.invokeMethod<Object?>('create', {}),
        throwsA(isA<NativeCallException>()),
      );
    },
  );
}
