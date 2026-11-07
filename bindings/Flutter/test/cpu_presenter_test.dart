import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native.dart';

import 'support/fake_transport.dart';

class ImageHandle {
  int disposals = 0;
}

NativeRasterFrame frame(int view) => NativeRasterFrame(
  pixels: Uint8List(4),
  width: 1,
  height: 1,
  stride: 4,
  originX: 0,
  originY: 0,
  scale: 1,
  stamp: NativeFrameStamp(1, 7, 0, 0, view),
);

void main() {
  test(
    'decode is serial, coalesces queued frames, and drops stale images',
    () async {
      final gates = <Completer<ImageHandle>>[];
      final published = <ImageHandle?>[];
      var view = 1;
      final presenter = CpuFramePresenter<ImageHandle>(
        decode: (_) {
          final gate = Completer<ImageHandle>();
          gates.add(gate);
          return gate.future;
        },
        disposeImage: (image) {
          image.disposals++;
        },
        isCurrent: (stamp) => stamp.viewGeneration == view,
        onImage: published.add,
        onError: (error) => fail('$error'),
      );
      presenter.offer(frame(1));
      view = 2;
      presenter.offer(frame(2));
      view = 3;
      presenter.offer(frame(3));
      expect(presenter.retainedFrameCount, 2);
      expect(gates.length, 1);
      final stale = ImageHandle();
      gates[0].complete(stale);
      await waitFor(() => gates.length == 2);
      expect(stale.disposals, 1);
      expect(published, isEmpty);
      final current = ImageHandle();
      gates[1].complete(current);
      await waitFor(() => presenter.image != null);
      expect(published, [current]);
      presenter.dispose();
      expect(current.disposals, 1);
      expect(presenter.retainedFrameCount, 0);
    },
  );

  test('late decode after dispose is freed without a callback', () async {
    final gate = Completer<ImageHandle>();
    final published = <ImageHandle?>[];
    final presenter = CpuFramePresenter<ImageHandle>(
      decode: (_) => gate.future,
      disposeImage: (image) {
        image.disposals++;
      },
      isCurrent: (_) => true,
      onImage: published.add,
      onError: (error) => fail('$error'),
    );
    presenter.offer(frame(1));
    presenter.dispose();
    expect(presenter.retainedFrameCount, 1);
    final decoded = ImageHandle();
    gate.complete(decoded);
    await waitFor(() => decoded.disposals == 1);
    expect(published, isEmpty);
    expect(presenter.retainedFrameCount, 0);
  });

  test(
    'clear invalidates an in-flight decode even without a view stamp change',
    () async {
      final gate = Completer<ImageHandle>();
      final published = <ImageHandle?>[];
      final presenter = CpuFramePresenter<ImageHandle>(
        decode: (_) => gate.future,
        disposeImage: (image) {
          image.disposals++;
        },
        isCurrent: (_) => true,
        onImage: published.add,
        onError: (error) => fail('$error'),
      );
      presenter.offer(frame(1));
      presenter.clear();
      final decoded = ImageHandle();
      gate.complete(decoded);
      await waitFor(() => decoded.disposals == 1);
      expect(published, [null]);
      presenter.dispose();
    },
  );
}
