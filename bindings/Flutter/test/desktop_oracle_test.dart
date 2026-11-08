import 'dart:io';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'support/desktop_consumer_probe.dart';

void main() {
  const native = String.fromEnvironment('RHWP_NATIVE_LIBRARY');
  const owner = String.fromEnvironment('RHWP_OWNER_LIBRARY');
  test(
    'actual CPU oracle copies a bounded lease and rejects a stale source',
    () async {
      final transport = await NativeOwnerTransport.connect(
        dispatcherLibrary: owner,
        nativeLibrary: native,
      );
      final document = await NativeDocument.connect(transport);
      final metadata = await NativePresentation.connect(nativeLibrary: native);
      final oracle = DesktopConsumerOracle(native);
      int? snapshot;
      int? changedSnapshot;
      try {
        await document.openBytes(
          await File('test/fixtures/desktop-consumer.hwpx').readAsBytes(),
        );
        final capture = await document.capturePresentation(
          page: 0,
          viewGeneration: document.beginPresentation(),
          deviceGeneration: 1,
          deviceScale: 1,
          releaseSnapshot: metadata.releaseSnapshot,
        );
        snapshot = capture['snapshot']! as int;
        final stamp = NativePresentationStamp.fromJson(
          Map<String, Object?>.from(capture['stamp']! as Map),
        );
        final viewport = NativePageViewport.fit(
          await document.pageSize(0),
          const Size(320, 240),
        );
        final pixels = oracle.pixels(
          snapshot: snapshot,
          stamp: stamp,
          viewport: viewport,
          dpr: 1,
          width: 320,
          height: 240,
        );
        expect(pixels.length, 320 * 240 * 4);
        expect(
          List.generate(320 * 240, (index) => pixels[index * 4 + 3]),
          everyElement(255),
        );
        var darkPixels = 0;
        final colors = <int>{};
        for (var offset = 0; offset < pixels.length; offset += 4) {
          if (pixels[offset] < 220 &&
              pixels[offset + 1] < 220 &&
              pixels[offset + 2] < 220) {
            darkPixels++;
          }
          colors.add(
            pixels[offset] << 16 | pixels[offset + 1] << 8 | pixels[offset + 2],
          );
        }
        expect(darkPixels, greaterThanOrEqualTo(32));
        expect(colors.length, greaterThanOrEqualTo(3));
        final stats = oracle.call({'op': 'stats'})['stats']! as Map;
        expect(stats['frameHandles'], 0);
        expect(stats['leases'], 0);
        expect(stats['snapshotHandles'], 1);
        const point = {
          'section': 0,
          'paragraph': 0,
          'scalarOffset': 0,
          'cellPath': <Object>[],
        };
        await document.request('beginEdit', {
          'selection': {'anchor': point, 'focus': point},
        });
        await document.request('pasteText', {'text': '\n\n\n\n\n\n'});
        expect(
          () => oracle.pixels(
            snapshot: snapshot!,
            stamp: stamp,
            viewport: viewport,
            dpr: 1,
            width: 320,
            height: 240,
          ),
          throwsStateError,
        );
        final next = await document.capturePresentation(
          page: 0,
          viewGeneration: document.beginPresentation(),
          deviceGeneration: 1,
          deviceScale: 1,
          releaseSnapshot: metadata.releaseSnapshot,
        );
        changedSnapshot = next['snapshot']! as int;
        final nextStamp = NativePresentationStamp.fromJson(
          Map<String, Object?>.from(next['stamp']! as Map),
        );
        final changed = oracle.pixels(
          snapshot: changedSnapshot,
          stamp: nextStamp,
          viewport: viewport,
          dpr: 1,
          width: 320,
          height: 240,
        );
        expect(changed, isNot(orderedEquals(pixels)));
        final after = oracle.call({'op': 'stats'})['stats']! as Map;
        expect(after['frameHandles'], 0);
        expect(after['leases'], 0);
      } finally {
        if (changedSnapshot != null) {
          await metadata.releaseSnapshot(changedSnapshot);
        }
        if (snapshot != null) await metadata.releaseSnapshot(snapshot);
        await document.dispose();
      }
    },
    skip: native.isEmpty || owner.isEmpty
        ? 'requires actual Native and owner dispatcher libraries'
        : false,
  );
}
