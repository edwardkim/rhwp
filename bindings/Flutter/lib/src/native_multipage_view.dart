import 'dart:async';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';

import 'native_document.dart';
import 'native_document_view.dart';
import 'native_editor_controller.dart';
import 'native_editor_view.dart';
import 'native_ffi.dart';
import 'native_viewport.dart';

/// Navigation state for one bounded Native document viewport.
class NativePagedViewportController extends ChangeNotifier {
  double _zoom = 1, _scroll = 0, _horizontal = 0;
  NativeDocumentViewport? _layout;
  NativeDocument? _documentOwner;
  int? _requestedPage;
  double get zoom => _zoom;
  double get scrollOffset => _scroll;
  double get horizontalOffset => _horizontal;
  int get pageCount => _layout?.pages.length ?? 0;
  int get currentPage => _layout?.visiblePages.keys.firstOrNull ?? 0;
  double get maxScrollExtent => _layout?.maxScrollExtent ?? 0;
  void setZoom(double value) {
    if (!value.isFinite || value < .25 || value > 4) {
      throw ArgumentError('Reference zoom must be between .25 and 4.');
    }
    if (_zoom == value) return;
    _zoom = value;
    notifyListeners();
  }

  void scrollTo(double offset, {double? horizontal}) {
    if (!offset.isFinite || (horizontal != null && !horizontal.isFinite)) {
      throw ArgumentError('Scroll offsets must be finite.');
    }
    final next = math.max(
      0.0,
      math.min(offset, _layout?.maxScrollExtent ?? offset),
    );
    final nextX = math.max(
      0.0,
      math.min(
        horizontal ?? _horizontal,
        _layout?.maxHorizontalExtent ?? horizontal ?? _horizontal,
      ),
    );
    if (next == _scroll && nextX == _horizontal) return;
    _scroll = next;
    _horizontal = nextX;
    _requestedPage = null;
    notifyListeners();
  }

  void jumpToPage(int page) {
    if (page < 0 || (_layout != null && page >= pageCount)) {
      throw RangeError('Page is outside the document.');
    }
    _requestedPage = page;
    if (_layout != null) {
      _scroll = math.min(_layout!.pageTop(page), maxScrollExtent);
    }
    notifyListeners();
  }

  void _accept(NativeDocumentViewport layout) {
    _layout = layout;
    _scroll = math.min(layout.scrollOffset, layout.maxScrollExtent);
    _horizontal = math.min(layout.horizontalOffset, layout.maxHorizontalExtent);
  }

  void _follow(Map<String, Object?>? caret) {
    final layout = _layout;
    if (caret == null || layout == null) return;
    final page = caret['page'] as int;
    if (page < 0 || page >= layout.pages.length) return;
    final rect = layout.geometryFor(page).caret(caret);
    double y = _scroll, x = _horizontal;
    if (rect.top < 0) y += rect.top;
    if (rect.bottom > layout.viewport.height) {
      y += rect.bottom - layout.viewport.height;
    }
    if (rect.left < 0) x += rect.left;
    if (rect.right > layout.viewport.width) {
      x += rect.right - layout.viewport.width;
    }
    scrollTo(y, horizontal: x);
  }
}

class _ViewportRequest {
  const _ViewportRequest(this.size, this.dpr, this.generation, this.serial);
  final Size size;
  final double dpr;
  final int generation, serial;
}

class _PageImage {
  const _PageImage(this.image, this.page, this.origin, this.size);
  final ui.Image image;
  final int page;
  final Offset origin;
  final Size size;
}

/// CPU multi-page reference viewport. One owner coordinates every visible page;
/// it never creates competing per-page [NativeDocumentView] generations.
/// Existing single-page external-texture views remain a separate opt-in path.
class NativePagedDocumentView extends StatefulWidget {
  const NativePagedDocumentView({
    super.key,
    required this.document,
    this.controller,
    this.editable = true,
    this.maxPages = 256,
    this.onViewportReady,
    this.onControllerChanged,
  });
  final NativeDocument document;
  final NativePagedViewportController? controller;
  final bool editable;
  final int maxPages;
  final ValueChanged<NativeEditorController?>? onControllerChanged;

  /// A decoded batch has been published for the next Flutter paint. This does
  /// not claim consumer GPU completion or hardware presentation.
  final ValueChanged<NativeDocumentViewport>? onViewportReady;
  @override
  State<NativePagedDocumentView> createState() =>
      _NativePagedDocumentViewState();
}

class _NativePagedDocumentViewState extends State<NativePagedDocumentView> {
  late NativePagedViewportController _controller;
  StreamSubscription<NativeDocumentInfo>? _changes;
  StreamSubscription<int>? _invalidations;
  List<NativePageSize>? _pages;
  List<_PageImage> _images = [];
  NativeDocumentViewport? _layout;
  Object? _identity, _key, _error;
  int _serial = 0;
  bool _running = false;
  _ViewportRequest? _latest;
  Future<void> _retiring = Future.value();

  Object? _sourceIdentity() {
    final info = widget.document.info;
    return info == null
        ? null
        : (info.documentId, info.revision, info.fontGeneration, info.pageCount);
  }

  @override
  void initState() {
    super.initState();
    _bindController();
    _bindDocument();
  }

  void _bindController() {
    _controller = widget.controller ?? NativePagedViewportController();
    _attachController();
    _controller.addListener(_navigate);
  }

  void _resetNavigation() {
    _controller._layout = null;
    _controller._requestedPage = null;
    _controller._scroll = 0;
    _controller._horizontal = 0;
  }

  void _attachController() {
    final previous = _controller._documentOwner;
    if (previous != null && !identical(previous, widget.document)) {
      _resetNavigation();
    }
    _controller._documentOwner = widget.document;
  }

  void _bindDocument() {
    _attachController();
    _identity = _sourceIdentity();
    _changes = widget.document.changes.listen((_) {
      final identity = _sourceIdentity();
      if (_identity != identity) {
        if ((_identity as (int, int, int, int)?)?.$1 !=
            (identity as (int, int, int, int)?)?.$1) {
          _resetNavigation();
        }
        _identity = identity;
        _invalidate(metadata: true);
      }
    });
    _invalidations = widget.document.rasterInvalidations.listen(
      (_) => _invalidate(metadata: true),
    );
  }

  void _navigate() => _invalidate();
  void _clearImages() {
    if (_images.isEmpty) return;
    final previous = _images;
    _images = [];
    final done = Completer<void>();
    final prior = _retiring;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      for (final item in previous) {
        item.image.dispose();
      }
      done.complete();
    });
    WidgetsBinding.instance.scheduleFrame();
    _retiring = Future.wait([prior, done.future]).then((_) {});
  }

  void _invalidate({bool metadata = false}) {
    if (!mounted) return;
    ++_serial;
    _latest = null;
    _key = null;
    if (!widget.document.isDisposed) widget.document.beginView();
    _clearImages();
    _layout = null;
    if (metadata) _pages = null;
    setState(() {
      _error = null;
    });
  }

  bool _current(_ViewportRequest request) =>
      mounted &&
      request.serial == _serial &&
      !widget.document.isDisposed &&
      request.generation == widget.document.viewGeneration;

  void _configure(Size size, double dpr) {
    if (widget.document.info == null ||
        widget.document.isDisposed ||
        size.isEmpty ||
        !size.isFinite) {
      return;
    }
    final key = (
      _sourceIdentity(),
      size,
      dpr,
      _controller.zoom,
      _controller.scrollOffset,
      _controller.horizontalOffset,
      _controller._requestedPage,
    );
    if (_key == key) return;
    _key = key;
    // This build paints the cover immediately on resize/DPR changes. Waiting
    // until the post-frame render would expose the previous placement once.
    _clearImages();
    _layout = null;
    final request = _ViewportRequest(
      size,
      dpr,
      widget.document.beginView(),
      ++_serial,
    );
    _latest = request;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) unawaited(_drain());
    });
  }

  Future<void> _drain() async {
    if (_running) return;
    _running = true;
    try {
      while (mounted && _latest != null) {
        final request = _latest!;
        _latest = null;
        await _render(request);
      }
    } finally {
      _running = false;
    }
  }

  Future<void> _render(_ViewportRequest request) async {
    final images = <_PageImage>[];
    try {
      _clearImages();
      if (mounted) setState(() {});
      await _retiring;
      if (!_current(request)) return;
      if (_pages == null) {
        final info = await widget.document.refreshInfo();
        if (!_current(request)) return;
        if (widget.maxPages < 1 ||
            widget.maxPages > 256 ||
            info.pageCount < 1 ||
            info.pageCount > widget.maxPages) {
          throw const NativeCallException(
            'pageLimit',
            'Reference document viewport supports at most 256 pages.',
          );
        }
        final pages = <NativePageSize>[];
        for (var page = 0; page < info.pageCount; ++page) {
          pages.add(await widget.document.pageSize(page));
          if (!_current(request)) return;
        }
        _pages = pages;
      }
      var layout = NativeDocumentViewport(
        pages: _pages!,
        viewport: request.size,
        scrollOffset: _controller.scrollOffset,
        horizontalOffset: _controller.horizontalOffset,
        zoom: _controller.zoom,
        deviceScale: request.dpr,
      );
      final targetPage = _controller._requestedPage;
      if (targetPage != null) {
        // A valid queued jump can outlive repagination. Resolve it against
        // this fresh authoritative page list rather than latching an error.
        final page = math.max(0, math.min(targetPage, _pages!.length - 1));
        layout = NativeDocumentViewport(
          pages: _pages!,
          viewport: request.size,
          scrollOffset: layout.pageTop(page),
          horizontalOffset: _controller.horizontalOffset,
          zoom: _controller.zoom,
          deviceScale: request.dpr,
        );
        _controller._requestedPage = null;
      }
      _controller._accept(layout);
      int decodedBytes = 0;
      for (final entry in layout.visiblePages.entries) {
        final region = layout.regionFor(entry.key);
        final frame = await widget.document.renderRegion(
          page: entry.key,
          x: region.left,
          y: region.top,
          width: region.width,
          height: region.height,
          scale: layout.scale * request.dpr,
          viewGeneration: request.generation,
        );
        if (!_current(request) || frame == null) return;
        decodedBytes += frame.stride * frame.height;
        if (decodedBytes > maxReferenceBytes) {
          throw const NativeCallException(
            'viewportLimit',
            'Decoded page regions exceed 16 MiB.',
          );
        }
        final image = await decodeNativeRasterFrame(frame);
        if (!_current(request) || !widget.document.isCurrent(frame.stamp)) {
          image.dispose();
          return;
        }
        images.add(
          _PageImage(
            image,
            entry.key,
            entry.value.origin +
                Offset(
                  frame.originX / request.dpr,
                  frame.originY / request.dpr,
                ),
            Size(frame.width / request.dpr, frame.height / request.dpr),
          ),
        );
      }
      if (!_current(request)) return;
      _key = (
        _sourceIdentity(),
        request.size,
        request.dpr,
        _controller.zoom,
        _controller.scrollOffset,
        _controller.horizontalOffset,
        _controller._requestedPage,
      );
      setState(() {
        _layout = layout;
        _images = List.of(images);
        _error = null;
      });
      images.clear();
      widget.onViewportReady?.call(layout);
    } catch (error) {
      if (_current(request)) {
        setState(() {
          _error = error;
          _layout = null;
        });
      }
    } finally {
      for (final item in images) {
        item.image.dispose();
      }
    }
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      final size = Size(constraints.maxWidth, constraints.maxHeight);
      _configure(size, MediaQuery.devicePixelRatioOf(context));
      Widget background(BuildContext context) => ColoredBox(
        color: const Color(0xffdddddd),
        child: CustomPaint(painter: _PagesPainter(_images)),
      );
      final layout = _layout;
      return Listener(
        onPointerSignal: (event) {
          if (event is PointerScrollEvent) {
            if (HardwareKeyboard.instance.isControlPressed ||
                HardwareKeyboard.instance.isMetaPressed) {
              _controller.setZoom(
                (_controller.zoom * (event.scrollDelta.dy < 0 ? 1.1 : 1 / 1.1))
                    .clamp(.25, 4),
              );
            } else {
              _controller.scrollTo(
                _controller.scrollOffset + event.scrollDelta.dy,
                horizontal: _controller.horizontalOffset + event.scrollDelta.dx,
              );
            }
          }
        },
        child: Stack(
          fit: StackFit.expand,
          children: [
            if (widget.editable)
              NativeEditorView(
                document: widget.document,
                pageViewports: layout?.visiblePages ?? const {},
                backgroundBuilder: background,
                onCaretChanged: _controller._follow,
                onControllerChanged: widget.onControllerChanged,
              )
            else
              background(context),
            if (_error != null)
              Align(
                alignment: Alignment.bottomCenter,
                child: ColoredBox(
                  color: const Color(0xeeffffff),
                  child: Text('문서 보기: $_error'),
                ),
              ),
          ],
        ),
      );
    },
  );
  @override
  void didUpdateWidget(NativePagedDocumentView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.controller != oldWidget.controller) {
      _controller.removeListener(_navigate);
      if (oldWidget.controller == null) _controller.dispose();
      _bindController();
      _invalidate();
    }
    if (widget.document != oldWidget.document) {
      unawaited(_changes?.cancel());
      unawaited(_invalidations?.cancel());
      if (!oldWidget.document.isDisposed) oldWidget.document.beginView();
      _pages = null;
      _layout = null;
      _bindDocument();
      _invalidate(metadata: true);
    } else if (widget.maxPages != oldWidget.maxPages) {
      _invalidate(metadata: true);
    }
  }

  @override
  void dispose() {
    ++_serial;
    _latest = null;
    unawaited(_changes?.cancel());
    unawaited(_invalidations?.cancel());
    if (!widget.document.isDisposed) widget.document.beginView();
    _clearImages();
    _controller.removeListener(_navigate);
    if (widget.controller == null) _controller.dispose();
    super.dispose();
  }
}

class _PagesPainter extends CustomPainter {
  const _PagesPainter(this.images);
  final List<_PageImage> images;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.clipRect(Offset.zero & size);
    for (final item in images) {
      canvas.drawImageRect(
        item.image,
        Rect.fromLTWH(
          0,
          0,
          item.image.width.toDouble(),
          item.image.height.toDouble(),
        ),
        item.origin & item.size,
        Paint()..filterQuality = FilterQuality.low,
      );
    }
  }

  @override
  bool shouldRepaint(_PagesPainter oldDelegate) => images != oldDelegate.images;
}
