// Probe routing/typing controls with fake native receipts. These do not execute
// a GPU, measure live memory, or replace the opt-in real desktop qualification.
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'native_gpu_cache_trim_test.dart' show TrimPlugin, pressureView;
import 'native_gpu_lifecycle_test.dart' show opened, until, cleanup;
import 'support/desktop_consumer_probe.dart';
import 'support/fake_gpu_presentation.dart';

void main() {
  test(
    'optional memory cycles preserve captures and leave a later publication',
    () {
      expect(desktopMemoryPressureSchedule(3, 0), isEmpty);
      expect(desktopMemoryPressureSchedule(32, 2), [3, 17]);
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
