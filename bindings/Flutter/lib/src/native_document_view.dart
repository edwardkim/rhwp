import 'dart:async';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';

import 'native_document.dart';
import 'native_viewport.dart';

Future<ui.Image> decodeNativeRasterFrame(NativeRasterFrame frame) async {
  ui.ImmutableBuffer? buffer;
  ui.ImageDescriptor? descriptor;
  ui.Codec? codec;
  try {
    buffer = await ui.ImmutableBuffer.fromUint8List(frame.pixels);
    descriptor = ui.ImageDescriptor.raw(
      buffer,
      width: frame.width,
      height: frame.height,
      rowBytes: frame.stride,
      pixelFormat: ui.PixelFormat.rgba8888,
    );
    codec = await descriptor.instantiateCodec();
    return (await codec.getNextFrame()).image;
  } finally {
    codec?.dispose();
    descriptor?.dispose();
    buffer?.dispose();
  }
}

class _DecodedImage {
  const _DecodedImage(
    this.image,
    this.logicalSize,
    this.origin,
    this.pageWidth,
  );
  final ui.Image image;
  final Size logicalSize;
  final Offset origin;
  final double pageWidth;
}

/// A bounded, read-only CPU reference viewport. This is not a desktop editor
/// or an external GPU texture integration. One viewport owns a document's view
/// generation; use separate document workers for independent windows.
class NativeDocumentView extends StatefulWidget {
  const NativeDocumentView({super.key, required this.document, this.page = 0});
  final NativeDocument document;
  final int page;
  @override
  State<NativeDocumentView> createState() => _NativeDocumentViewState();
}

class _NativeDocumentViewState extends State<NativeDocumentView> {
  late CpuFramePresenter<_DecodedImage> _presenter;
  StreamSubscription<NativeDocumentInfo>? _changes;
  final _published = <ui.Image>{};
  Object? _viewportKey;
  Object? _error;
  _DecodedImage? _image;
  double _zoom = 1;
  double _pageWidth = 0;

  @override
  void initState() {
    super.initState();
    _bind();
  }

  void _bind() {
    _presenter = CpuFramePresenter(
      decode: (frame) async {
        final zoom = _zoom;
        final image = await decodeNativeRasterFrame(frame);
        return _DecodedImage(
          image,
          Size(
            frame.width / frame.scale * zoom,
            frame.height / frame.scale * zoom,
          ),
          Offset(
            frame.originX / frame.scale * zoom,
            frame.originY / frame.scale * zoom,
          ),
          _pageWidth,
        );
      },
      disposeImage: (decoded) => _disposeImage(decoded.image),
      isCurrent: (stamp) => mounted && widget.document.isCurrent(stamp),
      onImage: (decoded) {
        if (!mounted) return;
        if (decoded != null) _published.add(decoded.image);
        setState(() {
          _image = decoded;
        });
      },
      onError: (error) {
        if (mounted) {
          setState(() {
            _error = error;
          });
        }
      },
    );
    Object? identity = widget.document.info == null
        ? null
        : (
            widget.document.info!.documentId,
            widget.document.info!.revision,
            widget.document.info!.fontGeneration,
            widget.document.info!.pageCount,
          );
    _changes = widget.document.changes.listen((info) {
      final next = (
        info.documentId,
        info.revision,
        info.fontGeneration,
        info.pageCount,
      );
      if (identity == next) return;
      identity = next;
      if (!mounted) return;
      _presenter.clear();
      setState(() {
        _viewportKey = null;
        _error = null;
      });
    });
  }

  void _disposeImage(ui.Image image) {
    if (!_published.remove(image)) {
      image.dispose(); // A stale decode was never visible to a render object.
      return;
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      image.dispose();
    });
    // Published replacements live through the frame which removes RawImage.
    if (SchedulerBinding.instance.schedulerPhase == SchedulerPhase.idle) {
      SchedulerBinding.instance.scheduleFrame();
    }
  }

  @override
  void didUpdateWidget(NativeDocumentView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!identical(oldWidget.document, widget.document)) {
      unawaited(_changes?.cancel());
      if (!oldWidget.document.isDisposed) {
        oldWidget.document.beginView();
      }
      _presenter.dispose();
      _image = null;
      _viewportKey = null;
      _error = null;
      _bind();
    }
  }

  void _configure(Size viewport, double dpr) {
    final info = widget.document.info;
    if (info == null || widget.document.isDisposed) return;
    final key = (
      widget.document,
      info.documentId,
      info.revision,
      info.fontGeneration,
      widget.page,
      viewport,
      dpr,
    );
    if (key == _viewportKey) return;
    _viewportKey = key;
    final view = widget.document.beginView();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || _viewportKey != key) return;
      unawaited(_render(viewport, dpr, view));
    });
  }

  Future<void> _render(Size viewport, double dpr, int generation) async {
    final document = widget.document;
    final presenter = _presenter;
    presenter.clear();
    setState(() {
      _error = null;
    });
    try {
      final size = await document.pageSize(widget.page);
      if (!mounted ||
          !identical(document, widget.document) ||
          document.isDisposed ||
          generation != document.viewGeneration) {
        return;
      }
      final geometry = NativePageViewport.fit(size, viewport);
      _zoom = geometry.zoom;
      _pageWidth = size.width * _zoom;
      final frame = await document.renderRegion(
        page: widget.page,
        x: 0,
        y: 0,
        width: math.min(size.width, viewport.width / _zoom),
        height: math.min(size.height, viewport.height / _zoom),
        scale: _zoom * dpr,
        viewGeneration: generation,
      );
      if (!mounted || !identical(document, widget.document) || frame == null) {
        return;
      }
      presenter.offer(frame);
    } catch (error) {
      if (mounted &&
          identical(document, widget.document) &&
          generation == document.viewGeneration) {
        setState(() {
          _error = error;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      if (!constraints.hasBoundedWidth ||
          !constraints.hasBoundedHeight ||
          constraints.maxWidth <= 0 ||
          constraints.maxHeight <= 0) {
        return const Text('문서 화면에는 유한한 너비와 높이가 필요합니다.');
      }
      _configure(
        Size(constraints.maxWidth, constraints.maxHeight),
        MediaQuery.devicePixelRatioOf(context),
      );
      if (_error != null) return Center(child: Text('문서 표시 실패: $_error'));
      final image = _image;
      return ColoredBox(
        color: const Color(0xff55575a),
        child: ClipRect(
          child: Stack(
            fit: StackFit.expand,
            children: [
              if (image != null)
                Positioned(
                  left:
                      (constraints.maxWidth - image.pageWidth) / 2 +
                      image.origin.dx,
                  top: image.origin.dy,
                  width: image.logicalSize.width,
                  height: image.logicalSize.height,
                  child: RawImage(image: image.image, fit: BoxFit.fill),
                ),
            ],
          ),
        ),
      );
    },
  );

  @override
  void dispose() {
    unawaited(_changes?.cancel());
    if (!widget.document.isDisposed) widget.document.beginView();
    _presenter.dispose();
    super.dispose();
  }
}
