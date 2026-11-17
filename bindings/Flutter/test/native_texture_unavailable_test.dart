import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_document.dart';
import 'package:rhwp_native_reference/src/native_ffi.dart';
import 'package:rhwp_native_reference/src/native_texture_view.dart';
import 'package:rhwp_native_reference/src/native_viewport.dart';

import 'support/fake_gpu_presentation.dart';

// Channel envelopes from the Windows, CV texture and native Metal view status
// producers. They intentionally have no fake-only `code` alias. No GPU/SDK run
// is claimed by this controller/overlay regression.
class UnavailableTexturePlugin extends FakeTexturePlugin {
  UnavailableTexturePlugin(super.platform, this.failureCode, this.message) {
    ready = false;
  }
  final String? failureCode;
  final String message;
  bool failedOwnerClosed = false;

  @override
  Map<String, Object?> status() {
    final result = super.status()
      ..remove('code')
      ..remove('errorCode')
      ..addAll({
        'textureId': 1,
        'ready': false,
        'ownerFinished': true,
        'closed': closed || failedOwnerClosed,
        'closing': closed || failedOwnerClosed,
        'error': message,
        if (failureCode != null) 'errorCode': failureCode,
        'hasFrame': false,
      });
    if (platform == NativeTexturePlatform.windows) {
      result.addAll({
        'deviceFailure': false,
        'unregistered': closed,
        'acceptedSerial': 0,
        'renderedSerial': 0,
        'descriptorOpenedSerial': 0,
      });
    } else if (platform == NativeTexturePlatform.macos) {
      result['textureUnregisteredNotification'] = closed;
    } else {
      result.addAll({
        'apiReady': false,
        'presentationKind': 'nativeView',
        'layerReady': false,
        'nativeViewLayerDetached': closed,
      });
    }
    return result;
  }
}

Future<void> until(WidgetTester tester, bool Function() ready) async {
  for (var attempt = 0; attempt < 100 && !ready(); ++attempt) {
    await tester.pump(const Duration(milliseconds: 10));
  }
  expect(ready(), isTrue);
}

Widget viewport(NativeTextureController controller) => MaterialApp(
  home: SizedBox(
    width: 300,
    height: 240,
    child: NativeTextureView(
      controller: controller,
      viewport: NativePageViewport.fit(
        const NativePageSize(600, 800),
        const Size(300, 240),
      ),
      cpuFallback: (_) => const Text('CPU reference'),
    ),
  ),
);

void main() {
  for (final platform in [
    NativeTexturePlatform.windows,
    NativeTexturePlatform.macos,
    NativeTexturePlatform.macosMetalView,
  ]) {
    for (final failure in [
      (code: 'backendUnavailable', fallback: true, closed: false),
      (code: 'featureDisabled', fallback: true, closed: false),
      (code: 'unsupported', fallback: true, closed: false),
      (code: 'resourceError', fallback: false, closed: true),
      (code: null, fallback: false, closed: false),
    ]) {
      testWidgets('${platform.name} preserves ${failure.code} startup status', (
        tester,
      ) async {
        final transport = GpuTransport();
        final document = await NativeDocument.connect(transport);
        final opening = document.newDocument();
        await until(tester, () => document.info != null);
        await opening;
        final metadata = FakePresentation();
        final plugin =
            UnavailableTexturePlugin(
                platform,
                failure.code,
                failure.fallback
                    ? 'GPU backend unavailable'
                    : 'Native replay failed',
              )
              ..failedOwnerClosed = failure.closed
              ..install();
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(
              SystemChannels.platform_views,
              (_) async => null,
            );
        final controller = NativeTextureController(
          document: document,
          presentation: metadata,
          platform: platform,
          pollInterval: const Duration(milliseconds: 10),
          operationTimeout: const Duration(milliseconds: 200),
        );
        try {
          await tester.pumpWidget(viewport(controller));
          await until(
            tester,
            () =>
                controller.phase == NativeTexturePhase.failed ||
                controller.needsCpuFallback,
          );
          await tester.pump();
          expect(controller.needsCpuFallback, failure.fallback);
          expect(
            controller.diagnostic,
            isA<NativeCallException>().having(
              (error) => error.code,
              'code',
              failure.code ?? 'nativeTextureError',
            ),
          );
          expect(
            find.text('CPU reference'),
            failure.fallback ? findsOneWidget : findsNothing,
          );
          expect(
            find.textContaining('문서 표시 실패:'),
            failure.fallback ? findsNothing : findsOneWidget,
          );
          expect(transport.snapshots, isEmpty);
          expect(plugin.submissions, isEmpty);
          expect(metadata.released, isEmpty);
          await tester.pump(const Duration(milliseconds: 100));
          final calls = plugin.calls.length;
          await tester.pump(const Duration(seconds: 1));
          expect(
            plugin.calls,
            hasLength(calls),
            reason: 'No unavailable retry loop',
          );
        } finally {
          var closed = false;
          final closing = controller.close().then((_) => closed = true);
          await until(tester, () => closed);
          await closing;
          await tester.pumpWidget(const SizedBox.shrink());
          controller.dispose();
          await document.dispose();
          plugin.uninstall();
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
              .setMockMethodCallHandler(SystemChannels.platform_views, null);
        }
      });
    }
  }
}
