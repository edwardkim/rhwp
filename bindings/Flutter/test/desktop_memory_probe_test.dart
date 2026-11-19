// Probe routing/typing controls with fake native receipts. These do not execute
// a GPU or replace the opt-in target-process memory/desktop qualification.
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'native_gpu_cache_trim_test.dart'
    show TrimPlugin, TrimTilePlugin, pressureView;
import 'native_gpu_lifecycle_test.dart' show opened, until, cleanup;
import 'support/desktop_consumer_probe.dart';
import 'support/fake_gpu_presentation.dart';

void main() {
  test(
    'optional memory cycles preserve captures and leave a later publication',
    () {
      expect(desktopMemoryPressureSchedule(3, 0), isEmpty);
      expect(desktopMemoryPressureSchedule(32, 2), [3, 17]);
      expect(desktopLinuxTileMemoryPressureSchedule(0), isEmpty);
      expect(desktopLinuxTileMemoryPressureSchedule(1), [1]);
      expect(desktopLinuxTileMemoryPressureSchedule(2), [1, 2]);
      expect(() => desktopLinuxTileMemoryPressureSchedule(3), throwsStateError);
      for (final frames in [32, 64]) {
        for (var cycles = 1; cycles <= 4; cycles++) {
          final schedule = desktopMemoryPressureSchedule(frames, cycles);
          expect(schedule.length, cycles);
          expect(schedule.toSet().length, cycles);
          expect(schedule, everyElement(inInclusiveRange(3, frames - 1)));
        }
      }
      for (final (frames, cycles) in [(31, 1), (65, 1), (32, -1), (32, 5)]) {
        expect(
          () => desktopMemoryPressureSchedule(frames, cycles),
          throwsStateError,
        );
      }
    },
  );

  for (final busy in [false, true]) {
    testWidgets(
      'real binding route observes ${busy ? 'busy/noop' : 'reclaimed'} typed controller receipt',
      (tester) async {
        final plugin = TrimPlugin()..busy = busy;
        plugin.install();
        final document = await opened(tester, GpuTransport());
        final controller = DesktopMemoryTextureController(
          document: document,
          presentation: FakePresentation(),
          platform: NativeTexturePlatform.linux,
        );
        try {
          await tester.pumpWidget(pressureView(controller));
          await until(tester, () => controller.canDisplay);
          // ChannelBuffers responds through the engine outside the test clock.
          final result = await tester.runAsync(() async {
            await dispatchDesktopMemoryPressure();
            final trim = controller.lastTrim;
            if (trim == null) {
              throw StateError('Lifecycle did not request trim.');
            }
            return trim.timeout(const Duration(seconds: 3));
          });
          expect(controller.trimCalls, 1);
          expect(plugin.trims, hasLength(1));
          final nativeArgs = plugin.trims.single.arguments! as Map;
          final receipt = desktopCacheTrimReceipt(result!);
          expect(receipt['textureId'], nativeArgs['textureId']);
          expect(receipt['deviceGeneration'], nativeArgs['deviceGeneration']);
          expect(receipt['requestId'], nativeArgs['requestId']);
          expect(receipt['scope'], 'managedGpuTargets');
          expect(receipt['supported'], isTrue);
          expect(receipt['pending'], isFalse);
          expect(receipt['busy'], busy);
          expect(receipt['before'], 8192);
          expect(receipt['after'], busy ? 8192 : 0);
          expect(receipt['trimmed'], busy ? 0 : 8192);
        } finally {
          await cleanup(tester, controller, document, plugin);
        }
      },
    );
  }

  testWidgets(
    'Linux group binding route retains both trim and idle member receipts',
    (tester) async {
      final plugin = TrimTilePlugin()
        ..secondPublished = true
        ..secondPending = false;
      plugin.install();
      final document = await opened(tester, GpuTransport());
      final controller = DesktopMemoryTextureController(
        document: document,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.linux,
        useTileGroups: true,
        trace: NativeGpuTrace(capacity: 64),
      );
      try {
        controller.configure(
          NativePageViewport.fit(
            const NativePageSize(600, 800),
            const Size(4098, 240),
          ),
          deviceScale: 1,
        );
        await tester.pumpWidget(
          MaterialApp(
            home: NativeGpuLifecycle(
              controller: controller,
              child: const SizedBox.shrink(),
            ),
          ),
        );
        await until(tester, () => controller.canDisplay);
        final stamp = NativePresentationStamp.fromJson(
          Map<String, Object?>.from(plugin.acceptedStamp!),
        );
        final observer = DesktopMemoryAcceptanceObserver(
          controller: controller,
          status: () async => plugin.group(),
          rssStartBytes: 1,
        );
        final cycle = await tester.runAsync(
          () => observer.pressureAfter(1, stamp),
        );
        expect(controller.trimCalls, 1);
        expect(
          plugin.trims.single.arguments,
          containsPair('groupId', controller.textureId),
        );
        final trim = cycle!['trim']! as Map;
        expect(trim['members'], hasLength(2));
        expect(trim['before'], 16384);
        expect(trim['after'], 0);
        expect(trim['trimmed'], 16384);
        final idle = cycle['idle']! as Map;
        for (final raw in idle['samples']! as List) {
          final sample = raw as Map;
          final members = (sample['native']! as Map)['members']! as List;
          expect(
            members.map((item) => (item as Map)['textureId']),
            controller.tileTextureIds,
          );
          expect(
            members.every(
              (item) => stamp.matches((item as Map)['renderedStamp']),
            ),
            isTrue,
          );
        }
        plugin.secondStamp = {
          ...plugin.acceptedStamp!,
          'revision': stamp.revision + 1,
        };
        await expectLater(observer.takeSample(stamp), throwsStateError);
        plugin.secondStamp = null;
      } finally {
        await cleanup(tester, controller, document, plugin);
      }
    },
  );

  testWidgets(
    'binding message cannot invent a trim without the lifecycle observer',
    (tester) async {
      final plugin = TrimPlugin();
      plugin.install();
      final document = await opened(tester, GpuTransport());
      final controller = DesktopMemoryTextureController(
        document: document,
        presentation: FakePresentation(),
        platform: NativeTexturePlatform.linux,
      );
      try {
        controller.configure(
          NativePageViewport.fit(
            const NativePageSize(600, 800),
            const Size(300, 240),
          ),
          deviceScale: 1,
        );
        await tester.pumpWidget(const MaterialApp(home: SizedBox.shrink()));
        await until(tester, () => controller.canDisplay);
        await tester.runAsync(dispatchDesktopMemoryPressure);
        expect(controller.trimCalls, 0);
        expect(controller.lastTrim, isNull);
        expect(plugin.trims, isEmpty);
      } finally {
        await cleanup(tester, controller, document, plugin);
      }
    },
  );
}
