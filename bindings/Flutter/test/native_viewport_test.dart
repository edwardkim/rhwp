import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native.dart';
import 'package:rhwp_native_reference/src/native_viewport.dart';

void main() {
  test(
    'same transform maps DPR-independent pointer and native caret geometry',
    () {
      final viewport = NativePageViewport.fit(
        const NativePageSize(600, 900),
        const Size(900, 700),
      );
      expect(viewport.zoom, 1);
      expect(viewport.origin, const Offset(150, 0));
      expect(viewport.toNative(const Offset(160, 20)), const Offset(10, 20));
      expect(
        viewport.caret({'x': 10, 'y': 20, 'height': 12}),
        const Rect.fromLTWH(160, 20, 1, 12),
      );
      final narrow = NativePageViewport.fit(
        const NativePageSize(600, 900),
        const Size(300, 400),
      );
      expect(narrow.toNative(const Offset(20, 30)), const Offset(40, 60));
      expect(
        narrow.rectangle({'x': 40, 'y': 60, 'width': 20, 'height': 10}),
        const Rect.fromLTWH(20, 30, 10, 5),
      );
    },
  );
  test('invalid viewport is rejected before geometry or worker input', () {
    expect(
      () =>
          NativePageViewport.fit(const NativePageSize(0, 1), const Size(1, 1)),
      throwsArgumentError,
    );
    expect(
      () => NativePageViewport.fit(
        const NativePageSize(1, 1),
        const Size(double.infinity, 1),
      ),
      throwsArgumentError,
    );
  });
}
