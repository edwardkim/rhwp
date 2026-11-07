import 'dart:math' as math;
import 'dart:ui';

import 'native_document.dart';

/// The reference host shows the top of one page. The same transform is used by
/// CPU images, native hit testing, selection overlays and IME candidate bounds.
class NativePageViewport {
  NativePageViewport.fit(this.page, this.viewport) {
    if (![
      page.width,
      page.height,
      viewport.width,
      viewport.height,
    ].every((value) => value.isFinite && value > 0)) {
      throw ArgumentError(
        'Page and viewport require finite positive dimensions.',
      );
    }
    zoom = math.min(1, viewport.width / page.width);
    origin = Offset((viewport.width - page.width * zoom) / 2, 0);
  }
  final NativePageSize page;
  final Size viewport;
  late final double zoom;
  late final Offset origin;
  Offset toNative(Offset local) => (local - origin) / zoom;
  Rect rectangle(Map bounds) => Rect.fromLTWH(
    origin.dx + (bounds['x']! as num) * zoom,
    origin.dy + (bounds['y']! as num) * zoom,
    (bounds['width']! as num) * zoom,
    (bounds['height']! as num) * zoom,
  );
  Rect caret(Map caret) => Rect.fromLTWH(
    origin.dx + (caret['x']! as num) * zoom,
    origin.dy + (caret['y']! as num) * zoom,
    1,
    (caret['height']! as num) * zoom,
  );
}
