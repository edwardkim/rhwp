import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'native_document_view_test.dart' show pumpUntil;
import 'native_editor_view_test.dart' show ViewTransport;

class _ComposingTransport extends ViewTransport {
  _ComposingTransport() {
    text = '';
    cursor = 0;
  }

  int? composition;
  String baseline = '';
  Completer<void>? commitGate;
  bool failContext = false;
  final settlements = <String>[];

  @override
  Map<String, Object?> get info => {
    ...super.info,
    'compositionId': composition,
  };

  @override
  Map<String, Object?> get ime => {
    ...super.ime,
    'compositionId': composition,
    'preedit': composition == null ? null : {'start': 0, 'end': text.length},
  };

  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) async {
    final command = request['command'];
    final op = command is Map ? command['op'] : null;
    if (op == 'imeEditContext' && failContext) {
      throw const NativeCallException(
        'contextUnavailable',
        'IME context failed.',
      );
    }
    if (op == 'applyImeEdit') {
      final operation = command as Map;
      expectSync((operation['expected']! as Map)['revision'], revision);
      requests.add(Map<String, Object?>.from(operation));
      final edit = operation['edit']! as Map;
      void replace(Map replacement) {
        final range = replacement['range']! as Map;
        text = text.replaceRange(
          range['start']! as int,
          range['end']! as int,
          replacement['text']! as String,
        );
      }

      replace(edit['replacement']! as Map);
      if (edit['kind'] == 'commit') composition = null;
      final preedit = edit['preedit'] as Map?;
      if (preedit != null) {
        baseline = text;
        replace(preedit['replacement']! as Map);
        composition = 71;
      }
      final selection = (preedit?['selection'] ?? edit['selection']) as Map;
      cursor = selection['focus']! as int;
      revision++;
      final id = ++serial;
      completions.add({
        'ok': true,
        'requestId': id,
        'kind': 'edited',
        'outcome': {'revision': revision},
      });
      return {'ok': true, 'requestId': id};
    }
    if (op != 'commitComposition' && op != 'cancelComposition') {
      return super.hostRequest(request, data: data);
    }
    final operation = command as Map;
    expectSync(operation['compositionId'], composition);
    settlements.add(op as String);
    requests.add(Map<String, Object?>.from(operation));
    final id = ++serial;
    if (op == 'commitComposition') await commitGate?.future;
    if (op == 'cancelComposition') text = baseline;
    composition = null;
    cursor = text.runes.length;
    revision++;
    completions.add({
      'ok': true,
      'requestId': id,
      'kind': 'edited',
      'outcome': {'revision': revision},
    });
    return {'ok': true, 'requestId': id};
  }
}

class _Files extends NativeFileStore {
  _Files(super.document, this.transport);
  final _ComposingTransport transport;
  final saved = <(String, int?)>[];

  @override
  Future<NativeSaveResult> save({
    String? destination,
    NativeLossApproval? approveLoss,
    NativeOverwriteApproval? approveOverwrite,
  }) async {
    saved.add((transport.text, transport.composition));
    return NativeSaveResult(
      info: await document.refreshInfo(),
      cancelled: false,
      path: destination,
      savedRevision: document.info!.revision,
    );
  }
}

Future<(NativeDocument, NativeEditorController, _ComposingTransport)>
_open() async {
  final transport = _ComposingTransport();
  final document = await NativeDocument.connect(transport);
  await document.newDocument();
  final editor = NativeEditorController(document);
  await editor.initialize();
  await editor.setEditingValue(
    const TextEditingValue(
      text: '한',
      selection: TextSelection.collapsed(offset: 1),
      composing: TextRange(start: 0, end: 1),
    ),
  );
  expect(editor.value.composing, const TextRange(start: 0, end: 1));
  return (document, editor, transport);
}

void main() {
  test('focus loss commits the last accepted Hangul preedit', () async {
    final (document, editor, transport) = await _open();
    addTearDown(document.dispose);
    addTearDown(editor.dispose);
    await editor.focusLost();
    expect(transport.settlements, ['commitComposition']);
    expect(editor.value.text, '한');
    expect(editor.value.composing, TextRange.empty);
    expect(document.info!.compositionId, isNull);
  });

  test(
    'focus settlement shares only an active commit at the queue tail',
    () async {
      final (document, editor, transport) = await _open();
      addTearDown(document.dispose);
      addTearDown(editor.dispose);
      final blur = editor.focusLost();
      final saveBarrier = editor.focusLost();
      final pendingAtBlur = editor.pending;

      final movement = editor.command('moveLeft');
      final afterMovement = editor.focusLost();
      final escape = editor.focusLost(cancel: true);
      final pendingAfterMovement = editor.pending;
      await Future.wait([blur, saveBarrier, movement, afterMovement, escape]);
      expect(pendingAtBlur, 1);
      expect(pendingAfterMovement, 4);
      expect(transport.settlements, ['commitComposition']);

      final nextBlur = editor.focusLost();
      expect(editor.pending, 1);
      await nextBlur;
    },
  );

  test(
    'save waits for composition acknowledgement and preserves the syllable',
    () async {
      final (document, editor, transport) = await _open();
      addTearDown(document.dispose);
      addTearDown(editor.dispose);
      final gate = transport.commitGate = Completer<void>();
      final files = _Files(document, transport);
      final lifecycle = NativeDocumentLifecycle(
        document: document,
        files: files,
        settleInput: editor.focusLost,
        confirmUnsaved: (_) async => NativeUnsavedDecision.cancel,
        chooseSavePath: () async => 'document.hwpx',
      );
      addTearDown(lifecycle.dispose);
      final save = lifecycle.save();
      for (var i = 0; i < 200 && transport.settlements.isEmpty; i++) {
        await Future<void>.delayed(const Duration(milliseconds: 1));
      }
      expect(transport.settlements, ['commitComposition']);
      expect(files.saved, isEmpty);
      gate.complete();
      expect(await save, isTrue);
      expect(files.saved, [('한', null)]);
    },
  );

  test(
    'cancelling the save destination preserves committed unsaved text',
    () async {
      final (document, editor, transport) = await _open();
      addTearDown(document.dispose);
      addTearDown(editor.dispose);
      final files = _Files(document, transport);
      final lifecycle = NativeDocumentLifecycle(
        document: document,
        files: files,
        settleInput: editor.focusLost,
        confirmUnsaved: (_) async => NativeUnsavedDecision.cancel,
        chooseSavePath: () async => null,
      );
      addTearDown(lifecycle.dispose);
      expect(await lifecycle.save(), isFalse);
      expect(files.saved, isEmpty);
      expect(editor.value.text, '한');
      expect(document.info!.isDirty, isTrue);
    },
  );

  test(
    'Undo keeps cancelling an active preedit without undoing history',
    () async {
      final (document, editor, transport) = await _open();
      addTearDown(document.dispose);
      addTearDown(editor.dispose);
      await editor.command('undo');
      expect(transport.settlements, ['cancelComposition']);
      expect(
        transport.requests.any((request) => request['op'] == 'undo'),
        isFalse,
      );
      expect(editor.value.text, isEmpty);
      expect(editor.value.composing, TextRange.empty);
    },
  );

  test(
    'failed post-commit context aborts saving and preserves accepted text',
    () async {
      final (document, editor, transport) = await _open();
      addTearDown(document.dispose);
      addTearDown(editor.dispose);
      final files = _Files(document, transport);
      final lifecycle = NativeDocumentLifecycle(
        document: document,
        files: files,
        settleInput: editor.focusLost,
        confirmUnsaved: (_) async => NativeUnsavedDecision.cancel,
        chooseSavePath: () async => 'document.hwpx',
      );
      addTearDown(lifecycle.dispose);
      transport.failContext = true;
      await expectLater(
        lifecycle.save(),
        throwsA(
          isA<NativeCallException>().having(
            (error) => error.code,
            'code',
            'contextUnavailable',
          ),
        ),
      );
      expect(files.saved, isEmpty);
      expect(transport.settlements, ['commitComposition']);
      expect(transport.text, '한');
      expect(document.info!.compositionId, isNull);
      expect(document.info!.isDirty, isTrue);
    },
  );

  test(
    'save does not settle composition through a replaced editor owner',
    () async {
      final (document, editor, transport) = await _open();
      addTearDown(document.dispose);
      addTearDown(editor.dispose);
      final nextOwner = NativeEditorController(document);
      addTearDown(nextOwner.dispose);
      final files = _Files(document, transport);
      final lifecycle = NativeDocumentLifecycle(
        document: document,
        files: files,
        settleInput: editor.focusLost,
        confirmUnsaved: (_) async => NativeUnsavedDecision.cancel,
        chooseSavePath: () async => 'document.hwpx',
      );
      addTearDown(lifecycle.dispose);
      await expectLater(
        lifecycle.save(),
        throwsA(
          isA<NativeCallException>().having(
            (error) => error.code,
            'code',
            'staleDocument',
          ),
        ),
      );
      expect(files.saved, isEmpty);
      expect(transport.settlements, isEmpty);
      expect(transport.text, '한');
      expect(document.info!.compositionId, isNotNull);
    },
  );

  for (final escape in [false, true]) {
    testWidgets(
      escape
          ? 'Escape cancels the active preedit'
          : 'view focus loss commits the active preedit',
      (tester) async {
        final transport = _ComposingTransport();
        final document = await NativeDocument.connect(transport);
        final opened = document.newDocument();
        await pumpUntil(tester, () => document.info != null);
        await opened;
        NativeEditorController? editor;
        await tester.pumpWidget(
          MaterialApp(
            home: NativeEditorView(
              document: document,
              onControllerChanged: (value) => editor = value,
            ),
          ),
        );
        await pumpUntil(tester, () => editor?.ready == true);
        await tester.tapAt(tester.getCenter(find.byType(NativeEditorView)));
        await pumpUntil(tester, () => tester.testTextInput.hasAnyClients);
        await pumpUntil(tester, () => editor!.pending == 0);
        tester.testTextInput.updateEditingValue(
          const TextEditingValue(
            text: '한',
            selection: TextSelection.collapsed(offset: 1),
            composing: TextRange(start: 0, end: 1),
          ),
        );
        await pumpUntil(
          tester,
          () => document.info!.compositionId != null && editor!.pending == 0,
        );
        expect(editor!.value.composing, const TextRange(start: 0, end: 1));
        if (escape) {
          await tester.sendKeyEvent(LogicalKeyboardKey.escape);
        } else {
          FocusManager.instance.primaryFocus!.unfocus();
        }
        await pumpUntil(
          tester,
          () => transport.settlements.isNotEmpty && editor?.pending == 0,
        );
        expect(transport.settlements, [
          escape ? 'cancelComposition' : 'commitComposition',
        ]);
        expect(transport.text, escape ? '' : '한');
        await tester.pumpWidget(const SizedBox.shrink());
        await tester.pump(const Duration(milliseconds: 20));
        await tester.runAsync(document.dispose);
      },
    );
  }

  for (final deltaModel in [false, true]) {
    testWidgets(
      'save ignores detached ${deltaModel ? 'delta' : 'full-value'} input while commit waits',
      (tester) async {
        final transport = _ComposingTransport();
        final document = await NativeDocument.connect(transport);
        final opening = document.newDocument();
        await pumpUntil(tester, () => document.info != null);
        await opening;
        NativeEditorController? editor;
        final gate = transport.commitGate = Completer<void>();
        NativeDocumentLifecycle? lifecycle;
        var saveComplete = false;
        Future<bool>? saving;
        try {
          await tester.pumpWidget(
            MaterialApp(
              home: NativeEditorView(
                document: document,
                onControllerChanged: (value) => editor = value,
              ),
            ),
          );
          await pumpUntil(tester, () => editor?.ready == true);
          await tester.tapAt(tester.getCenter(find.byType(NativeEditorView)));
          await pumpUntil(tester, () => tester.testTextInput.hasAnyClients);
          await pumpUntil(tester, () => editor!.pending == 0);
          tester.testTextInput.updateEditingValue(
            const TextEditingValue(
              text: '한',
              selection: TextSelection.collapsed(offset: 1),
              composing: TextRange(start: 0, end: 1),
            ),
          );
          await pumpUntil(
            tester,
            () => document.info!.compositionId != null && editor!.pending == 0,
          );
          expect(editor!.value.text, '한');
          expect(editor!.value.composing, const TextRange(start: 0, end: 1));
          final queuedEdits = <Future<void>>[
            // Leave one queue slot for the synchronous blur settlement. The
            // save barrier must wait for that same operation and its edits.
            if (deltaModel)
              for (var index = 0; index < 31; index++)
                editor!.setEditingValue(
                  TextEditingValue(
                    text: index.isEven ? '한' : '하',
                    selection: const TextSelection.collapsed(offset: 1),
                    composing: const TextRange(start: 0, end: 1),
                  ),
                ),
          ];
          expect(editor!.pending, queuedEdits.length);
          final client = tester.state(find.byType(NativeEditorView));
          final files = _Files(document, transport);
          lifecycle = NativeDocumentLifecycle(
            document: document,
            files: files,
            settleInput: () async {
              final focus = FocusManager.instance;
              focus.primaryFocus?.unfocus();
              focus.applyFocusChangesIfNeeded();
              await editor!.focusLost();
            },
            confirmUnsaved: (_) async => NativeUnsavedDecision.cancel,
            chooseSavePath: () async => 'document.hwpx',
          );
          saving = lifecycle.save();
          unawaited(
            saving.then<void>(
              (_) => saveComplete = true,
              onError: (Object _, StackTrace _) => saveComplete = true,
            ),
          );
          if (deltaModel) await tester.pump(const Duration(seconds: 2));
          await pumpUntil(tester, () => transport.settlements.isNotEmpty);
          await Future.wait(queuedEdits);
          expect(saveComplete, isFalse);
          expect(
            transport.requests.where(
              (request) => request['op'] == 'applyImeEdit',
            ),
            hasLength(1 + queuedEdits.length),
          );
          expect(tester.testTextInput.hasAnyClients, isFalse);
          expect(files.saved, isEmpty);
          final pending = editor!.pending;
          if (deltaModel) {
            (client as DeltaTextInputClient).updateEditingValueWithDeltas([
              const TextEditingDeltaInsertion(
                oldText: '한',
                textInserted: '글',
                insertionOffset: 1,
                selection: TextSelection.collapsed(offset: 2),
                composing: TextRange(start: 1, end: 2),
              ),
            ]);
          } else {
            (client as TextInputClient).updateEditingValue(
              const TextEditingValue(
                text: '한글',
                selection: TextSelection.collapsed(offset: 2),
                composing: TextRange(start: 1, end: 2),
              ),
            );
          }
          expect(
            editor!.pending,
            pending,
            reason: 'A detached input client must not admit more editor work.',
          );
          (client as TextInputClient).performAction(TextInputAction.newline);
          (client as TextInputClient).performSelector('insertNewline:');
          expect(editor!.pending, pending);
          gate.complete();
          await pumpUntil(tester, () => saveComplete);
          expect(await saving, isTrue);
          expect(files.saved, [('한', null)]);
          expect(editor!.error, isNull);
        } finally {
          if (!gate.isCompleted) gate.complete();
          if (saving != null) {
            await pumpUntil(tester, () => saveComplete);
            await saving.catchError((Object _) => false);
          }
          lifecycle?.dispose();
          await tester.pumpWidget(const SizedBox.shrink());
          await tester.pump(const Duration(milliseconds: 50));
          await tester.runAsync(document.dispose);
        }
      },
    );
  }
}
