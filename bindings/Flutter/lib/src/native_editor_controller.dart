import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import 'native_document.dart';
import 'native_ffi.dart';
import 'native_ime.dart';

/// Serialized editor input over one NativeDocument. The native worker owns all
/// text, history, layout, hit testing and geometry; Flutter holds snapshots only.
class NativeEditorController extends ChangeNotifier {
  NativeEditorController(this.document)
    : documentId = document.info?.documentId,
      editorGeneration = document.beginEditor() {
    if (documentId == null) {
      throw StateError('Open a document before attaching an editor.');
    }
  }
  final NativeDocument document;
  final int? documentId;
  final int editorGeneration;
  int _optimisticBatches = 0;
  Future<void> _tail = Future<void>.value();
  int _pending = 0;
  int _inputEpoch = 0;
  bool _disposed = false;
  NativeImeSnapshot? _ime;
  TextEditingValue value = TextEditingValue.empty;
  Object? error;
  Map<String, Object?>? caret;
  List<Map<String, Object?>> rectangles = const [];
  Map<String, Object?>? semantics;
  double? preferredX;
  int get pending => _pending;
  bool get ready =>
      !_disposed && _ime != null && document.info?.documentId == documentId;

  void _guard() {
    if (_disposed || document.isDisposed) throw const NativeDisposedException();
    if (document.editorGeneration != editorGeneration ||
        document.info?.documentId != documentId) {
      throw const NativeCallException(
        'staleDocument',
        'Editor belongs to a replaced document.',
      );
    }
  }

  Future<void> _run(Future<void> Function() action) {
    _guard();
    if (_pending >= 32) {
      throw const NativeCallException(
        'inputLimit',
        'Editor input queue is full.',
      );
    }
    final epoch = _inputEpoch;
    final done = Completer<void>();
    _pending++;
    _tail = _tail.then((_) async {
      try {
        _guard();
        if (epoch != _inputEpoch) {
          throw const NativeCallException(
            'staleInput',
            'Input context was refreshed.',
          );
        }
        await action();
        _guard();
        error = null;
        done.complete();
      } catch (failure, stack) {
        _inputEpoch++;
        error = failure;
        caret = null;
        rectangles = const [];
        // Querying the authority after a failed/stale delta never retries the
        // mutation or replaces the user's bytes with an optimistic platform copy.
        if (!_disposed &&
            !document.isDisposed &&
            document.editorGeneration == editorGeneration &&
            epoch == _inputEpoch - 1) {
          try {
            await document.refreshInfo();
            _guard();
            await _readIme();
            value = _ime!.value;
          } catch (_) {
            _ime = null;
          }
        }
        done.completeError(failure, stack);
      } finally {
        _pending--;
        if (!_disposed) notifyListeners();
      }
    });
    return done.future;
  }

  Future<void> initialize() => _run(() async {
    if (document.transport.capabilities['atomicImeEdits'] != true) {
      throw const NativeCallException(
        'unsupportedIme',
        'Native library has no atomic IME ABI.',
      );
    }
    await document.refreshInfo();
    await _settleComposition(cancel: true);
    if (!document.info!.editing) {
      const point = {
        'section': 0,
        'paragraph': 0,
        'scalarOffset': 0,
        'cellPath': <Object?>[],
      };
      await document.request('beginEdit', {
        'selection': {'anchor': point, 'focus': point},
      });
    }
    await _synchronize();
  });

  Future<void> _readIme() async {
    final result = await document.request('imeEditContext');
    _guard();
    final snapshot = NativeImeSnapshot(
      Map<String, Object?>.from(result['context']! as Map),
    );
    if (snapshot.expected['revision'] != document.info!.revision ||
        snapshot.expected['fontGeneration'] != document.info!.fontGeneration) {
      throw const NativeCallException(
        'staleContext',
        'Native IME context changed before receipt.',
      );
    }
    _ime = snapshot;
    if (_pending <= 1 && _optimisticBatches == 0) value = snapshot.value;
  }

  Future<void> _synchronize() async {
    try {
      await _readIme();
    } on NativeCallException catch (failure) {
      if (failure.code != 'invalidArgument' &&
          failure.code != 'unsupported' &&
          failure.code != 'limitExceeded') {
        rethrow;
      }
      _ime =
          null; // Multi-paragraph/object selections remain native and copyable.
    }
    _guard();
    final info = document.info!;
    final selection = info.selection;
    caret = null;
    rectangles = const [];
    if (selection != null) {
      try {
        final c = await document.request('caretRect', {
          'position': selection['focus'],
        });
        final r = await document.request('selectionRects', {
          'selection': selection,
        });
        if (document.info!.revision == info.revision &&
            document.info!.fontGeneration == info.fontGeneration) {
          caret = c['caret'] == null
              ? null
              : Map<String, Object?>.from(c['caret']! as Map);
          rectangles = List.unmodifiable(
            (r['rects']! as List)
                .take(64)
                .map((item) => Map<String, Object?>.from(item as Map)),
          );
        }
      } on NativeCallException {
        /* Native geometry may be unavailable for controls. */
      }
    }
    final a = await document.request('accessibility', {
      'request': {
        'geometry': false,
        'maxNodes': 64,
        'maxTextBytes': 32768,
        'maxDepth': 4,
        'maxGeometryQueries': 0,
        // Native ABI 1 requires a positive cap even with geometry disabled.
        'maxSelectionRects': 1,
      },
    });
    final snapshot = Map<String, Object?>.from(a['snapshot']! as Map);
    if (snapshot['revision'] == document.info!.revision &&
        snapshot['fontGeneration'] == document.info!.fontGeneration) {
      semantics = Map.unmodifiable(snapshot);
    }
  }

  Future<void> applyDeltas(List<TextEditingDelta> deltas) {
    if (deltas.isEmpty) return Future<void>.value();
    if (deltas.length > 32) {
      throw const NativeCallException('inputLimit', 'Too many IME deltas.');
    }
    final changes = <(TextEditingValue, TextEditingValue)>[];
    var next = value;
    for (final delta in deltas) {
      if (delta.oldText != next.text) {
        throw const FormatException('Stale platform oldText.');
      }
      final after = delta.apply(next);
      checkNativeText(after.text);
      changes.add((next, after));
      next = after;
    }
    final future = _run(() async {
      for (final (before, after) in changes) {
        await _readIme();
        final snapshot = _ime!;
        final edit = nativeImeDelta(snapshot, before, after);
        await document.request('applyImeEdit', {
          'expected': snapshot.expected,
          'edit': edit,
        });
        await _readIme();
      }
      await _synchronize();
    });
    _optimisticBatches++;
    value = next;
    return future.whenComplete(() {
      _optimisticBatches--;
      if (!_disposed && _optimisticBatches == 0 && _ime != null) {
        value = _ime!.value;
        notifyListeners();
      }
    });
  }

  Future<void> setEditingValue(TextEditingValue next) {
    checkNativeText(next.text);
    final difference = nativeTextDifference(value.text, next.text);
    return applyDeltas([
      TextEditingDeltaReplacement(
        oldText: value.text,
        replacementText: difference.text,
        replacedRange: difference.range,
        selection: next.selection,
        composing: next.composing,
      ),
    ]);
  }

  Future<void> command(String command, {bool extend = false}) => _run(() async {
    if ((command == 'undo' || command == 'redo') &&
        document.info?.compositionId != null) {
      await _settleComposition(cancel: true);
      await _synchronize();
      return;
    }
    await _settleComposition();
    if (command == 'undo' || command == 'redo') {
      await document.request(command);
    } else {
      final args = <String, Object?>{'command': command};
      if ([
        'moveLeft',
        'moveRight',
        'moveUp',
        'moveDown',
        'home',
        'end',
      ].contains(command)) {
        args['extendSelection'] = extend;
      }
      if (command == 'moveUp' || command == 'moveDown') {
        args['preferredX'] = preferredX;
      }
      final result = await document.request('executeCommand', {
        'command': args,
      });
      final outcome = result['outcome']! as Map;
      preferredX = (outcome['preferredX'] as num?)?.toDouble();
    }
    await _synchronize();
  });

  Future<void> _settleComposition({bool cancel = false}) async {
    final id = document.info?.compositionId;
    if (id != null) {
      await document.request(
        cancel ? 'cancelComposition' : 'commitComposition',
        {'compositionId': id},
      );
    }
  }

  Future<void> focusLost() => _run(() async {
    await _settleComposition(cancel: true);
    await _synchronize();
  });

  Future<void> point(int page, double x, double y, {bool extend = false}) =>
      _run(() async {
        await _settleComposition();
        final hit = await document.request('hitTest', {
          'page': page,
          'x': x,
          'y': y,
        });
        final position = hit['hit']! as Map;
        final focus = Map<String, Object?>.from(position['position']! as Map);
        final currentSelection = document.info!.selection;
        final anchor = extend && currentSelection != null
            ? (currentSelection['anchor'] ?? focus)
            : focus;
        await document.request('setSelection', {
          'selection': {'anchor': anchor, 'focus': focus},
        });
        preferredX = null;
        await _synchronize();
      });

  Future<void> setLocalSelection(TextSelection selection) {
    final snapshot = _ime;
    if (snapshot == null) {
      throw const NativeCallException(
        'unsupportedSelection',
        'No bounded paragraph context.',
      );
    }
    checkNativeTextBoundary(snapshot.value.text, selection.baseOffset);
    checkNativeTextBoundary(snapshot.value.text, selection.extentOffset);
    final context = snapshot.expected['context']! as Map;
    Map<String, Object?> position(int offset) => {
      ...Map<String, Object?>.from(context['position']! as Map),
      'scalarOffset':
          (context['absoluteStartScalar']! as int) +
          snapshot.value.text.substring(0, offset).runes.length,
    };
    final native = {
      'anchor': position(selection.baseOffset),
      'focus': position(selection.extentOffset),
    };
    return _run(() async {
      await _settleComposition();
      if (_ime == null ||
          jsonEncode(_ime!.expected) != jsonEncode(snapshot.expected)) {
        throw const FormatException('Stale semantic selection.');
      }
      await document.request('setSelection', {'selection': native});
      await _synchronize();
    });
  }

  Future<void> copy({bool cut = false}) => _run(() async {
    await _settleComposition();
    final result = await document.request('copySelection');
    final payload = result['payload']! as Map;
    await Clipboard.setData(
      ClipboardData(text: payload['plainText']! as String),
    );
    if (cut) {
      await document.request('cutSelection');
    }
    await _synchronize();
  });

  Future<void> paste() => _run(() async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    if (data?.text == null) return;
    await _settleComposition();
    await document.request('pasteText', {'text': data!.text!});
    await _synchronize();
  });

  @override
  void dispose() {
    document.endEditor(editorGeneration);
    _disposed = true;
    _inputEpoch++;
    _ime = null;
    caret = null;
    rectangles = const [];
    super.dispose();
  }
}
