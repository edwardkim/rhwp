import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/scheduler.dart';

import 'native_document.dart';
import 'native_ffi.dart';
import 'native_texture_view.dart';

/// Owns renderer replacement for one document without replacing its editor,
/// worker, file store, or input queue. Recovery is always an explicit host action.
///
/// Keep the old viewport mounted while [recover] or [close] waits: Linux may
/// require a final painted texture callback to finish native retirement.
class NativeGpuSession extends ChangeNotifier {
  NativeGpuSession({
    required NativeTextureController initialController,
    required this.createController,
  }) : _controller = initialController,
       document = initialController.document {
    _controller.addListener(_changed);
  }

  final NativeDocument document;
  final NativeTextureController Function() createController;
  NativeTextureController _controller;
  NativeTextureController get controller => _controller;
  Future<bool>? _recovery;
  Future<bool>? _close;
  bool _closing = false, _disposed = false, _notificationScheduled = false;
  Object? recoveryDiagnostic;

  bool get recovering => _recovery != null;
  bool get canRecover =>
      !_closing &&
      !document.isDisposed &&
      const {
        NativeTexturePhase.fallback,
        NativeTexturePhase.retirePending,
        NativeTexturePhase.closed,
      }.contains(_controller.phase);

  void _changed() {
    if (_disposed) return;
    // The mounted child configures its renderer during layout/dependencies.
    // Forwarding that notification to an ancestor builder in the same build
    // would dirty the ancestor while Flutter is rebuilding its descendants.
    // The actual texture still observes its own controller cover immediately.
    if (SchedulerBinding.instance.schedulerPhase ==
        SchedulerPhase.persistentCallbacks) {
      if (_notificationScheduled) return;
      _notificationScheduled = true;
      SchedulerBinding.instance.addPostFrameCallback((_) {
        _notificationScheduled = false;
        if (!_disposed) notifyListeners();
      });
    } else {
      notifyListeners();
    }
  }

  /// Returns true only when a replacement controller was installed. Its first
  /// GPU frame still needs normal negotiation, capture and publication checks.
  /// A false result preserves the current controller and its CPU fallback.
  ///
  /// Ambiguous native retirement never authorizes another create. Creating a
  /// new Dart controller also never refunds native immutable/quarantine credits.
  /// The synchronous factory must return a fresh controller for [document].
  Future<bool> recover() {
    final pending = _recovery;
    if (pending != null) return pending;
    if (!canRecover) return Future.value(false);
    final done = Completer<bool>();
    _recovery = done.future;
    recoveryDiagnostic = null;
    _changed();
    unawaited(() async {
      try {
        final previous = _controller;
        await previous.close();
        if (_closing || document.isDisposed) {
          done.complete(false);
          return;
        }
        if (previous.phase != NativeTexturePhase.closed ||
            previous.retirementPending) {
          recoveryDiagnostic =
              previous.retirementDiagnostic ??
              const NativeCallException(
                'retirePending',
                'The previous GPU owner has not confirmed retirement.',
              );
          done.complete(false);
          return;
        }
        final next = createController();
        if (identical(next, previous) ||
            !identical(next.document, document) ||
            next.phase != NativeTexturePhase.initializing) {
          // A factory contract violation does not establish ownership of an
          // already used or foreign document's controller. Its caller retains
          // cleanup ownership; never close another live viewport here.
          throw const NativeCallException(
            'invalidRendererReplacement',
            'Recovery requires a fresh GPU controller for the same document.',
          );
        }
        if (_closing || document.isDisposed) {
          await next.close();
          next.dispose();
          done.complete(false);
          return;
        }
        previous.removeListener(_changed);
        _controller = next;
        next.addListener(_changed);
        // The old view may detach on the next Flutter frame. close is already
        // confirmed and its late ownership observers survive notifier disposal.
        previous.dispose();
        done.complete(true);
      } catch (error, stack) {
        recoveryDiagnostic = error;
        done.completeError(error, stack);
      } finally {
        _recovery = null;
        _changed();
      }
    }());
    return done.future;
  }

  /// Acknowledges renderer retirement, not consumer GPU completion. A false
  /// result keeps the owning document alive so the host can report pending work.
  Future<bool> close() {
    final pending = _close;
    if (pending != null) return pending;
    final done = Completer<bool>();
    _close = done.future;
    _closing = true;
    final recovery = _recovery;
    _changed();
    unawaited(() async {
      try {
        await _controller.close();
        if (recovery != null) {
          try {
            await recovery;
          } catch (_) {
            // Recovery diagnostics remain visible; close still owns cleanup.
          }
        }
        await _controller.close();
        final confirmed =
            _controller.phase == NativeTexturePhase.closed &&
            !_controller.retirementPending;
        // The original late reply may confirm retirement after this bounded
        // wait. Keep closing=true (no new create) but let a subsequent host
        // close recheck that confirmed state without repeating native cleanup.
        if (!confirmed) _close = null;
        done.complete(confirmed);
      } catch (error, stack) {
        recoveryDiagnostic = error;
        done.completeError(error, stack);
      }
    }());
    return done.future;
  }

  @override
  void dispose() {
    if (_disposed) return;
    _disposed = true;
    _controller.removeListener(_changed);
    unawaited(close().whenComplete(() => _controller.dispose()));
    super.dispose();
  }
}
