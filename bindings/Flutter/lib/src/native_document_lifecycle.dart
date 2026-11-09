import 'dart:async';

import 'package:flutter/foundation.dart';

import 'native_document.dart';
import 'native_ffi.dart';
import 'native_file_store.dart';

enum NativeUnsavedDecision { save, discard, cancel }

/// Reference-host decisions around an existing document and its file store.
///
/// [settleInput] must drain the attached editor's accepted input before returning
/// (for NativeEditorController, call focusLost). Keep the viewport mounted while
/// [retireViewport] waits for the platform's normal texture retirement path.
/// This controller does not create a parser, an editor or a platform window.
class NativeDocumentLifecycle extends ChangeNotifier {
  NativeDocumentLifecycle({
    required this.document,
    required this.files,
    required this.settleInput,
    required this.confirmUnsaved,
    required this.chooseSavePath,
    this.approveLoss,
    this.approveOverwrite,
    this.retireViewport,
  }) {
    if (!identical(files.document, document)) {
      throw ArgumentError('File store and lifecycle must share one document.');
    }
    _changes = document.changes.listen((_) => _notify());
  }

  final NativeDocument document;
  final NativeFileStore files;
  final Future<void> Function() settleInput;
  final Future<NativeUnsavedDecision> Function(NativeDocumentInfo)
  confirmUnsaved;
  final Future<String?> Function() chooseSavePath;
  final Future<bool> Function(Map<String, Object?>)? approveLoss;
  final Future<bool> Function(NativeFileStamp)? approveOverwrite;

  /// True confirms platform producer retirement and texture unregistration.
  /// A bounded close Future alone, including retirePending, is not confirmation.
  final Future<bool> Function()? retireViewport;
  late final StreamSubscription<NativeDocumentInfo> _changes;

  bool _busy = false;
  bool _closed = false;
  bool _disposed = false;
  Object? _error;
  Future<bool>? _closeFuture;

  bool get isBusy => _busy;
  bool get isClosed => _closed;
  Object? get error => _error;
  String? get sourcePath => files.sourcePath;

  void _notify() {
    if (!_disposed) notifyListeners();
  }

  Future<bool> _run(Future<bool> Function() action) {
    if (_disposed || _closed || document.isDisposed) {
      throw const NativeDisposedException();
    }
    if (_busy) {
      throw const NativeCallException(
        'lifecycleBusy',
        'Finish the pending document decision first.',
      );
    }
    _busy = true;
    _error = null;
    _notify();
    return Future<bool>.sync(action)
        .catchError((Object error, StackTrace stack) {
          _error = error;
          Error.throwWithStackTrace(error, stack);
        })
        .whenComplete(() {
          _busy = false;
          _notify();
        });
  }

  Future<NativeDocumentInfo> _settledInfo() async {
    await settleInput();
    await files.whenIdle;
    return document.refreshInfo();
  }

  Future<NativeDocumentInfo> _sameCheckpoint(
    NativeDocumentInfo expected,
  ) async {
    final actual = await document.refreshInfo();
    if (actual.documentId != expected.documentId ||
        actual.revision != expected.revision ||
        actual.fontGeneration != expected.fontGeneration) {
      throw const NativeCallException(
        'documentChanged',
        'The document changed during the pending decision; it was preserved.',
      );
    }
    return actual;
  }

  Future<NativeSaveResult?> _save({
    required NativeDocumentInfo expected,
    String? destination,
  }) async {
    var path = destination ?? files.sourcePath;
    if (path == null) {
      path = await chooseSavePath();
      if (path == null) return null;
    }
    await _sameCheckpoint(expected);
    return files.save(
      destination: path,
      approveLoss: approveLoss,
      approveOverwrite: approveOverwrite,
    );
  }

  Future<NativeDocumentInfo?> _transitionCheckpoint() async {
    final observed = await _settledInfo();
    if (!observed.isDirty) return observed;
    final decision = await confirmUnsaved(observed);
    if (decision == NativeUnsavedDecision.cancel) return null;
    await _sameCheckpoint(observed);
    if (decision == NativeUnsavedDecision.discard) return observed;
    final saved = await _save(expected: observed);
    if (saved == null || saved.cancelled) return null;
    final current = await document.refreshInfo();
    if (current.documentId != observed.documentId || current.isDirty) {
      throw const NativeCallException(
        'unsavedChanges',
        'Edits accepted during the save remain unsaved; the document was kept.',
      );
    }
    return current;
  }

  Future<bool> newDocument() => _run(() async {
    final checkpoint = await _transitionCheckpoint();
    if (checkpoint == null) return false;
    await document.newDocument(replacing: checkpoint);
    return true;
  });

  Future<bool> openPath(String path, {bool recovery = false}) => _run(() async {
    final checkpoint = await _transitionCheckpoint();
    if (checkpoint == null) return false;
    if (recovery) {
      await files.openRecovery(path, replacing: checkpoint);
    } else {
      await files.openPath(path, replacing: checkpoint);
    }
    return true;
  });

  Future<bool> save({String? destination}) => _run(() async {
    final observed = await _settledInfo();
    final saved = await _save(expected: observed, destination: destination);
    return saved != null && !saved.cancelled;
  });

  /// Writes recovery bytes without acknowledging a user save or clearing dirty.
  Future<String?> writeRecovery(String directory) async {
    String? path;
    await _run(() async {
      await _settledInfo();
      path = await files.writeRecovery(directory);
      return true;
    });
    return path;
  }

  /// A cancelled decision or unconfirmed viewport retirement keeps the owner.
  /// Once document.dispose starts, an owner-close error cannot resume editing.
  ///
  /// Repeated calls share the in-progress result. A cancelled close can be
  /// attempted again; only successful retirement and owner close are final.
  Future<bool> close() {
    if (_closed) return _closeFuture ?? Future<bool>.value(true);
    if (_closeFuture != null) return _closeFuture!;
    final future = _run(() async {
      final checkpoint = await _transitionCheckpoint();
      if (checkpoint == null) return false;
      if (await retireViewport?.call() == false) {
        throw const NativeCallException(
          'retirementPending',
          'Texture retirement is unconfirmed; keep its view and owner alive.',
        );
      }
      await files.whenIdle;
      await _sameCheckpoint(checkpoint);
      await document.dispose();
      _closed = true;
      return true;
    });
    _closeFuture = future;
    unawaited(
      future.then<void>(
        (_) {
          if (!_closed) _closeFuture = null;
        },
        onError: (Object _, StackTrace _) {
          _closeFuture = null;
        },
      ),
    );
    return future;
  }

  /// Releases listeners only. Hosts must call and await close while mounted.
  @override
  void dispose() {
    _disposed = true;
    unawaited(_changes.cancel());
    super.dispose();
  }
}
