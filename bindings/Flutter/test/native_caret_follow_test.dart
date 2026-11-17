// Synthetic native geometry/selection and widget ownership contracts. These
// checks do not inject an OS IME or measure document layout/GPU output.
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'native_document_view_test.dart' show pumpUntil;
import 'native_multipage_view_test.dart' show PagedTransport;

class SelectionCaretTransport extends PagedTransport {
  SelectionCaretTransport() {
    pages = 2;
    capabilities['atomicImeEdits'] = true;
  }

  bool spansParagraphs = false;
  final editorRequests = <Map<String, Object?>>[];
  static const point = {
    'section': 0,
    'paragraph': 0,
    'scalarOffset': 1,
    'cellPath': <Object?>[],
  };

  @override
  Map<String, Object?> get info => {
    ...super.info,
    'editing': true,
    'compositionId': null,
    'selection': {
      'anchor': point,
      'focus': {...point, 'paragraph': spansParagraphs ? 1 : 0},
    },
  };

  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) async {
    final command = request['command'] as Map?;
    final op = command?['op'];
    if (!const {
      'imeEditContext',
      'caretRect',
      'selectionRects',
      'accessibility',
      'hitTest',
      'setSelection',
      'executeCommand',
    }.contains(op)) {
      return super.hostRequest(request, data: data);
    }
    editorRequests.add(Map<String, Object?>.from(command!));
    final id = ++serial;
    final result = <String, Object?>{'ok': true, 'requestId': id};
    switch (op) {
      case 'imeEditContext':
        if (spansParagraphs) {
          result.addAll({
            'ok': false,
            'code': 'unsupported',
            'error': 'The selection has no single-paragraph IME context.',
          });
        } else {
          result.addAll({
            'kind': 'imeEditContext',
            'context': {
              'revision': revision,
              'fontGeneration': fontGeneration,
              'compositionId': null,
              'preedit': null,
              'selection': info['selection'],
              'context': {
                'text': 'A',
                'cursorScalar': 1,
                'anchorScalar': 1,
                'position': point,
                'absoluteStartScalar': 0,
              },
            },
          });
        }
      case 'caretRect':
        result.addAll({
          'kind': 'caretRect',
          'caret': {
            'page': spansParagraphs ? 1 : 0,
            'x': 20,
            'y': spansParagraphs ? 160 : 20,
            'height': 12,
          },
        });
      case 'selectionRects':
        result.addAll({'kind': 'selectionRects', 'rects': const []});
      case 'accessibility':
        result.addAll({
          'kind': 'accessibility',
          'snapshot': {
            'revision': revision,
            'fontGeneration': fontGeneration,
            'nodes': const [
              {'text': 'A'},
            ],
            'complete': true,
          },
        });
      case 'hitTest':
        result.addAll({
          'kind': 'hitTest',
          'hit': {'position': point},
        });
      case 'setSelection':
        result.addAll({
          'kind': 'edited',
          'outcome': {'revision': revision},
        });
      case 'executeCommand':
        final action = command['command']! as Map;
        if (action['command'] == 'moveDown' &&
            action['extendSelection'] == true) {
          spansParagraphs = true;
        }
        result.addAll({
          'kind': 'commandExecuted',
          'outcome': {
            'outcome': {'revision': revision},
            'preferredX': null,
          },
        });
    }
    completions.add(result);
    return {'ok': true, 'requestId': id};
  }
}

Future<NativeDocument> openDocument(
  WidgetTester tester,
  SelectionCaretTransport transport,
) async {
  final document = await NativeDocument.connect(transport);
  final opening = document.newDocument();
  await pumpUntil(tester, () => document.info != null);
  await opening;
  return document;
}

void main() {
  testWidgets('native caret is reported without a paragraph IME context', (
    tester,
  ) async {
    final transport = SelectionCaretTransport();
    final document = await openDocument(tester, transport);
    NativeEditorController? editor;
    final carets = <Map<String, Object?>?>[];
    try {
      await tester.pumpWidget(
        MaterialApp(
          home: NativeEditorView(
            document: document,
            onControllerChanged: (value) => editor = value,
            onCaretChanged: carets.add,
          ),
        ),
      );
      await pumpUntil(tester, () => editor?.ready == true);
      await tester.pump();
      carets.clear();
      final selection = editor!.command('moveDown', extend: true);
      await pumpUntil(tester, () => editor!.pending == 0);
      await selection;
      expect(editor!.ready, isFalse);
      expect(editor!.caret, containsPair('page', 1));
      await tester.pump();
      expect(carets, isNotEmpty);
      expect(carets.last, containsPair('page', 1));
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pump(const Duration(milliseconds: 50));
      await tester.runAsync(document.dispose);
    }
  });

  testWidgets(
    'Shift selection follows an offscreen caret without IME context',
    (tester) async {
      final transport = SelectionCaretTransport();
      final document = await openDocument(tester, transport);
      final pages = NativePagedViewportController();
      NativeEditorController? editor;
      NativeDocumentViewport? visible;
      try {
        await tester.pumpWidget(
          MaterialApp(
            home: MediaQuery(
              data: const MediaQueryData(devicePixelRatio: 1),
              child: Center(
                child: SizedBox(
                  width: 300,
                  height: 160,
                  child: NativePagedDocumentView(
                    document: document,
                    controller: pages,
                    onControllerChanged: (value) => editor = value,
                    onViewportReady: (value) => visible = value,
                  ),
                ),
              ),
            ),
          ),
        );
        await pumpUntil(tester, () => editor?.ready == true && visible != null);
        await tester.tapAt(
          tester.getTopLeft(find.byType(NativeEditorView)) +
              const Offset(40, 40),
        );
        await pumpUntil(tester, () => editor!.pending == 0);
        expect(pages.scrollOffset, 0);
        await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
        await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
        await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
        await pumpUntil(
          tester,
          () => transport.spansParagraphs && editor!.pending == 0,
        );
        expect(editor!.ready, isFalse);
        expect(editor!.caret, containsPair('page', 1));
        await pumpUntil(tester, () => pages.scrollOffset > 0);
        await pumpUntil(tester, () => visible!.visiblePages.containsKey(1));
        final caret = visible!.geometryFor(1).caret(editor!.caret!);
        expect(caret.top, greaterThanOrEqualTo(-.001));
        expect(
          caret.bottom,
          lessThanOrEqualTo(visible!.viewport.height + .001),
        );
      } finally {
        await tester.pumpWidget(const SizedBox.shrink());
        await tester.pump(const Duration(milliseconds: 50));
        await tester.runAsync(document.dispose);
        pages.dispose();
      }
    },
  );

  testWidgets('a scheduled caret cannot notify a replacement document owner', (
    tester,
  ) async {
    final first = await openDocument(tester, SelectionCaretTransport());
    final second = await openDocument(tester, SelectionCaretTransport());
    var current = first;
    late StateSetter updateHost;
    NativeEditorController? editor;
    final firstCarets = <Map<String, Object?>?>[];
    final replacementCarets = <Map<String, Object?>?>[];
    var switched = false;
    try {
      await tester.pumpWidget(
        MaterialApp(
          home: StatefulBuilder(
            builder: (context, setState) {
              updateHost = setState;
              return NativeEditorView(
                document: current,
                onControllerChanged: (value) => editor = value,
                onCaretChanged: identical(current, first)
                    ? firstCarets.add
                    : replacementCarets.add,
              );
            },
          ),
        ),
      );
      await pumpUntil(tester, () => editor?.ready == true);
      await tester.pump();
      final previous = editor!;
      void replaceAfterInput() {
        if (previous.pending != 0 || switched) return;
        previous.removeListener(replaceAfterInput);
        updateHost(() {
          current = second;
          switched = true;
        });
      }

      previous.addListener(replaceAfterInput);
      final moving = previous.command('moveRight');
      await pumpUntil(tester, () => switched);
      await moving;
      expect(
        replacementCarets,
        isEmpty,
        reason: 'The old editor callback ran after the new owner was bound.',
      );
      await pumpUntil(
        tester,
        () => editor?.ready == true && identical(editor!.document, second),
      );
      await tester.pump();
      expect(replacementCarets, isNotEmpty);
      expect(replacementCarets.last, isNotNull);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pump(const Duration(milliseconds: 50));
      await tester.runAsync(first.dispose);
      await tester.runAsync(second.dispose);
    }
  });
}
