import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';

import 'native_texture_view.dart';

/// Pauses an existing renderer while its desktop view is hidden and covers old
/// pixels synchronously when the view's physical size or DPR changes.
///
/// This widget owns neither the document nor controller disposal. The child
/// remains mounted so native registration and late cleanup observers survive.
class NativeGpuLifecycle extends StatefulWidget {
  const NativeGpuLifecycle({
    super.key,
    required this.controller,
    required this.child,
    this.onPauseError,
  });

  final NativeTextureController controller;
  final Widget child;
  final ValueChanged<Object>? onPauseError;

  @override
  State<NativeGpuLifecycle> createState() => _NativeGpuLifecycleState();
}

class _NativeGpuLifecycleState extends State<NativeGpuLifecycle>
    with WidgetsBindingObserver {
  ui.FlutterView? _view;
  Size? _physicalSize;
  double? _deviceScale;
  AppLifecycleState? _lifecycle;
  int _metricsGeneration = 0;
  bool _metricsPending = false;

  bool get _hidden => const {
    AppLifecycleState.hidden,
    AppLifecycleState.paused,
    AppLifecycleState.detached,
  }.contains(_lifecycle);

  @override
  void initState() {
    super.initState();
    _lifecycle = WidgetsBinding.instance.lifecycleState;
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final current = View.of(context);
    final changed = _view != null && !identical(_view, current);
    _view = current;
    _physicalSize = current.physicalSize;
    _deviceScale = current.devicePixelRatio;
    if (changed) {
      _holdForGeometry();
    } else {
      _sync();
    }
  }

  @override
  void didUpdateWidget(NativeGpuLifecycle oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!identical(oldWidget.controller, widget.controller)) _sync();
  }

  void _pause() {
    final controller = widget.controller;
    unawaited(
      controller.pause().catchError((Object error) {
        // A renderer already stopped by fallback/close is still covered. A
        // current native hide failure remains reportable to its owning host.
        if (mounted && identical(controller, widget.controller)) {
          widget.onPauseError?.call(error);
        }
      }),
    );
  }

  void _sync() {
    if (_hidden || _metricsPending) {
      _pause();
    } else {
      widget.controller.resume();
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    _lifecycle = state;
    _sync();
  }

  @override
  void didChangeMetrics() {
    final view = _view;
    if (view == null ||
        (_physicalSize == view.physicalSize &&
            _deviceScale == view.devicePixelRatio)) {
      return; // Another Flutter view's global metrics notification.
    }
    _physicalSize = view.physicalSize;
    _deviceScale = view.devicePixelRatio;
    _holdForGeometry();
  }

  void _holdForGeometry() {
    _metricsPending = true;
    final generation = ++_metricsGeneration;
    _pause(); // Happens before the next layout can paint an old DPR frame.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || generation != _metricsGeneration) return;
      _metricsPending = false;
      _sync(); // The mounted child has now received its current layout/DPR.
    });
    WidgetsBinding.instance.scheduleFrame();
  }

  @override
  Widget build(BuildContext context) => widget.child;

  @override
  void dispose() {
    ++_metricsGeneration;
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }
}
