import 'dart:async';
import 'dart:ui' show AppExitResponse;

import 'package:flutter/widgets.dart';

/// Keeps the native viewport mounted while a cancelable desktop exit waits for
/// the host's save decision, GPU retirement and document owner close.
/// Forced process termination cannot be intercepted by this widget.
class NativeWindowClose extends StatefulWidget {
  const NativeWindowClose({
    super.key,
    required this.onClose,
    required this.child,
    this.onError,
  });

  final Future<bool> Function() onClose;
  final ValueChanged<Object>? onError;
  final Widget child;

  @override
  State<NativeWindowClose> createState() => _NativeWindowCloseState();
}

class _NativeWindowCloseState extends State<NativeWindowClose>
    with WidgetsBindingObserver {
  Future<AppExitResponse>? _pending;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  Future<AppExitResponse> didRequestAppExit() {
    if (_pending != null) return _pending!;
    final close = widget.onClose;
    final completer = Completer<AppExitResponse>();
    _pending = completer.future;
    unawaited(() async {
      var response = AppExitResponse.cancel;
      try {
        final closed = await close();
        if (mounted && widget.onClose == close && closed) {
          response = AppExitResponse.exit;
        }
      } catch (error) {
        // Failure to clean up must not let the platform's error handler exit.
        if (mounted) {
          try {
            widget.onError?.call(error);
          } catch (_) {
            // A diagnostic observer cannot turn failed cleanup into permission.
          }
        }
      } finally {
        _pending = null;
        completer.complete(response);
      }
    }());
    return completer.future;
  }

  @override
  Widget build(BuildContext context) => widget.child;

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }
}
