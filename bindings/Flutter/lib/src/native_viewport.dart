import 'dart:math' as math;
import 'dart:ui';

import 'native_document.dart';
import 'native_ffi.dart';

/// A page placement in viewport logical pixels. The same transform is used by
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
  NativePageViewport.placed(
    this.page,
    this.viewport, {
    required this.zoom,
    required this.origin,
  }) {
    if (![
          page.width,
          page.height,
          viewport.width,
          viewport.height,
          zoom,
        ].every((value) => value.isFinite && value > 0) ||
        !origin.dx.isFinite ||
        !origin.dy.isFinite) {
      throw ArgumentError(
        'Page placement requires finite positive dimensions and scale.',
      );
    }
  }
  Rect get pageBounds => origin & Size(page.width * zoom, page.height * zoom);
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

/// One bounded layout plan for all visible pages, in viewport logical pixels.
/// Page sizes come from Native; this does not repaginate or infer document layout.
class NativeDocumentViewport {
  NativeDocumentViewport({
    required List<NativePageSize> pages,
    required this.viewport,
    this.scrollOffset = 0,
    this.horizontalOffset = 0,
    this.zoom = 1,
    this.deviceScale = 1,
    this.pageGap = 16,
    this.maxVisiblePages = 4,
  }) : pages = List.unmodifiable(pages) {
    if (pages.isEmpty ||
        pages.length > 256 ||
        ![
          viewport.width,
          viewport.height,
          zoom,
          deviceScale,
        ].every((value) => value.isFinite && value > 0) ||
        ![
          scrollOffset,
          horizontalOffset,
          pageGap,
        ].every((value) => value.isFinite && value >= 0) ||
        zoom < .25 ||
        zoom > 4 ||
        deviceScale > 8 ||
        maxVisiblePages < 1 ||
        maxVisiblePages > 4) {
      throw ArgumentError('Invalid bounded document viewport.');
    }
    double widest = 0;
    for (final page in pages) {
      if (![
        page.width,
        page.height,
      ].every((value) => value.isFinite && value > 0 && value <= 16777216)) {
        throw ArgumentError('Invalid native page extent.');
      }
      widest = math.max(widest, page.width);
    }
    scale = math.min(1, viewport.width / widest) * zoom;
    contentWidth = math.max(viewport.width, widest * scale);
    double top = 0;
    for (final page in pages) {
      _tops.add(top);
      top += page.height * scale + pageGap;
      if (!top.isFinite || top > 16777216) {
        throw ArgumentError('Document viewport extent exceeds its limit.');
      }
    }
    contentHeight = top - pageGap;
    maxScrollExtent = math.max(0, contentHeight - viewport.height);
    maxHorizontalExtent = math.max(0, contentWidth - viewport.width);
    final scrollY = math.min(scrollOffset, maxScrollExtent);
    final scrollX = math.min(horizontalOffset, maxHorizontalExtent);
    final clip = Offset.zero & viewport;
    final placements = <int, NativePageViewport>{};
    int bytes = 0;
    for (var index = 0; index < pages.length; index++) {
      final geometry = NativePageViewport.placed(
        pages[index],
        viewport,
        zoom: scale,
        origin: Offset(
          (contentWidth - pages[index].width * scale) / 2 - scrollX,
          _tops[index] - scrollY,
        ),
      );
      final visible = geometry.pageBounds.intersect(clip);
      if (visible.isEmpty) continue;
      if (placements.length >= maxVisiblePages) {
        throw const NativeCallException(
          'visiblePageLimit',
          'At most four page regions can be visible; increase zoom.',
        );
      }
      final region = Rect.fromPoints(
        geometry.toNative(visible.topLeft),
        geometry.toNative(visible.bottomRight),
      );
      final rasterScale = scale * deviceScale;
      final width =
          (region.right * rasterScale).ceil() -
          (region.left * rasterScale).floor();
      final height =
          (region.bottom * rasterScale).ceil() -
          (region.top * rasterScale).floor();
      bytes += width * height * 4;
      if (width > 4096 || height > 4096 || bytes > maxReferenceBytes) {
        throw const NativeCallException(
          'viewportLimit',
          'Visible page regions exceed the 16 MiB reference raster budget.',
        );
      }
      placements[index] = geometry;
    }
    visiblePages = Map.unmodifiable(placements);
    rasterBytes = bytes;
  }
  final List<NativePageSize> pages;
  final Size viewport;
  final double scrollOffset, horizontalOffset, zoom, deviceScale, pageGap;
  final int maxVisiblePages;
  final List<double> _tops = [];
  late final double scale, contentWidth, contentHeight;
  late final double maxScrollExtent, maxHorizontalExtent;
  late final Map<int, NativePageViewport> visiblePages;
  late final int rasterBytes;
  double pageTop(int page) => _tops[page];
  NativePageViewport geometryFor(int page) => NativePageViewport.placed(
    pages[page],
    viewport,
    zoom: scale,
    origin: Offset(
      (contentWidth - pages[page].width * scale) / 2 -
          math.min(horizontalOffset, maxHorizontalExtent),
      _tops[page] - math.min(scrollOffset, maxScrollExtent),
    ),
  );
  Rect regionFor(int page) {
    final geometry = visiblePages[page];
    if (geometry == null) throw ArgumentError('Page is not visible.');
    final visible = geometry.pageBounds.intersect(Offset.zero & viewport);
    return Rect.fromPoints(
      geometry.toNative(visible.topLeft),
      geometry.toNative(visible.bottomRight),
    );
  }
}
