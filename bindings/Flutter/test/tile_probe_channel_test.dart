// Portable platform-channel receipt controls, not native desktop execution.
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'support/linux_tile_group_probe.dart' show RecordedGroupChannel;

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  for (final platform in [
    NativeTexturePlatform.linux,
    NativeTexturePlatform.windows,
  ]) {
    test(
      'only accepted ${platform.name} submissions advance the probe',
      () async {
        final channel = RecordedGroupChannel(platform);
        final messenger =
            TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
        var attempt = 0;
        messenger.setMockMethodCallHandler(channel, (call) async {
          expect(call.method, 'submitGroup');
          if (++attempt == 1) {
            throw PlatformException(code: 'backpressure');
          }
          if (attempt == 2) return {'accepted': false};
          return {
            'accepted': true,
            'acceptedSnapshots': [19],
          };
        });
        addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
        await expectLater(
          channel.invokeMethod<Object?>('submitGroup', {'serial': 1}),
          throwsA(isA<PlatformException>()),
        );
        expect(channel.submits, 0);
        expect(channel.submitted, isNull);
        await channel.invokeMethod<Object?>('submitGroup', {'serial': 2});
        expect(channel.submits, 0);
        expect(channel.acceptance, isNull);
        await channel.invokeMethod<Object?>('submitGroup', {'serial': 3});
        expect(channel.submits, 1);
        expect(channel.calls, 3);
        expect(channel.submitted, {'serial': 3});
        expect(channel.acceptance, {
          'accepted': true,
          'acceptedSnapshots': [19],
        });
      },
    );
  }
}
