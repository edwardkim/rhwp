import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'support/fake_transport.dart';
import 'native_document_view_test.dart' show pumpUntil;

class ViewTransport extends FakeTransport {
  ViewTransport() {
    capabilities['atomicImeEdits'] = true;
  }
  String text = 'A';
  int cursor = 1;
  final requests = <Map<String, Object?>>[];
  static const point = {
    'section': 0,
    'paragraph': 0,
    'scalarOffset': 0,
    'cellPath': <Object?>[],
  };
  @override
  Map<String, Object?> get info => {
    ...super.info,
    'editing': true,
    'isDirty': true,
    'selection': {
      'anchor': {...point, 'scalarOffset': cursor},
      'focus': {...point, 'scalarOffset': cursor},
    },
    'compositionId': null,
  };
  Map<String, Object?> get ime => {
    'revision': revision,
    'fontGeneration': fontGeneration,
    'compositionId': null,
    'preedit': null,
    'selection': info['selection'],
    'context': {
      'text': text,
      'cursorScalar': cursor,
      'anchorScalar': cursor,
      'selectedText': '',
      'absoluteStartScalar': 0,
      'position': {...point, 'scalarOffset': cursor},
      'textTruncated': false,
      'selectionTruncated': false,
    },
  };
  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) async {
    final command = request['command'];
    if (command is! Map) return super.hostRequest(request, data: data);
    final op = command['op'];
    if ([
      'new',
      'info',
      'pageSize',
      'renderRegion',
      'renderTiledRegion',
    ].contains(op)) {
      return super.hostRequest(request, data: data);
    }
    requests.add(Map<String, Object?>.from(command));
    final id = ++serial;
    final result = <String, Object?>{'ok': true, 'requestId': id};
    switch (op) {
      case 'imeEditContext':
        result.addAll({'kind': 'imeEditContext', 'context': ime});
      case 'caretRect':
        result.addAll({
          'kind': 'caretRect',
          'caret': {'page': 0, 'x': 1, 'y': 1, 'height': 2},
        });
      case 'selectionRects':
        result.addAll({'kind': 'selectionRects', 'rects': const []});
      case 'accessibility':
        final options = command['request']! as Map;
        final selectionLimit = options['maxSelectionRects']! as int;
        // Native ABI 1 validates this positive cap even for geometry:false.
        if (selectionLimit < 1 || selectionLimit > 256) {
          result.addAll({
            'ok': false,
            'code': 'limitExceeded',
            'error': 'accessibility selection rectangle budget is invalid',
          });
          break;
        }
        result.addAll({
          'kind': 'accessibility',
          'snapshot': {
            'revision': revision,
            'fontGeneration': fontGeneration,
            'nodes': [
              {'text': text},
            ],
            'complete': true,
            'truncated': false,
            'limitations': const [],
          },
        });
      case 'hitTest':
        result.addAll({
          'kind': 'hitTest',
          'hit': {
            'position': {...point, 'scalarOffset': cursor},
          },
        });
      case 'applyImeEdit':
        text = 'AB';
        cursor = 2;
        revision++;
        result.addAll({
          'kind': 'edited',
          'outcome': {'revision': revision},
        });
      case 'setSelection':
        result.addAll({
          'kind': 'edited',
          'outcome': {'revision': revision},
        });
      case 'executeCommand':
        result.addAll({
          'kind': 'commandExecuted',
          'outcome': {
            'outcome': {'revision': revision},
            'preferredX': null,
          },
        });
      default:
        throw StateError('Unexpected editor request $op');
    }
    completions.add(result);
    return {'ok': true, 'requestId': id};
  }
}

void main() {
  test(
    'editor requests a valid bounded text-only accessibility snapshot',
    () async {
      final transport = ViewTransport();
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      final editor = NativeEditorController(document);
      addTearDown(editor.dispose);
      await editor.initialize();
      final queries = transport.requests.where(
        (request) => request['op'] == 'accessibility',
      );
      expect(queries, isNotEmpty);
      for (final query in queries) {
        final options = query['request']! as Map;
        expect(options['geometry'], isFalse);
        expect(options['maxGeometryQueries'], 0);
        expect(options['maxSelectionRects'], inInclusiveRange(1, 256));
      }
      expect(editor.ready, isTrue);
      expect(editor.semantics, isNotNull);
    },
  );
  testWidgets(
    'custom raster viewport receives the transform used for native hit testing',
    (tester) async {
      final transport = ViewTransport();
      final document = await NativeDocument.connect(transport);
      final opened = document.newDocument();
      await pumpUntil(tester, () => document.info != null);
      await opened;
      NativePageViewport? viewport;
      await tester.pumpWidget(
        MaterialApp(
          home: NativeEditorView(
            document: document,
            viewportBuilder:
                (context, boundDocument, page, geometry, cpuFallback) {
                  expect(boundDocument, same(document));
                  expect(page, 0);
                  viewport = geometry;
                  return cpuFallback(context);
                },
          ),
        ),
      );
      await pumpUntil(tester, () => viewport != null);
      final geometry = viewport!;
      expect(geometry.viewport, tester.getSize(find.byType(NativeEditorView)));
      final local = geometry.origin + const Offset(2, 3) * geometry.zoom;
      await tester.tapAt(
        tester.getTopLeft(find.byType(NativeEditorView)) + local,
      );
      await pumpUntil(
        tester,
        () => transport.requests.any((request) => request['op'] == 'hitTest'),
      );
      final hit = transport.requests.lastWhere(
        (request) => request['op'] == 'hitTest',
      );
      expect(hit['x'], closeTo(2, 0.000001));
      expect(hit['y'], closeTo(3, 0.000001));
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pump(const Duration(milliseconds: 20));
      await tester.runAsync(document.dispose);
    },
  );
  testWidgets(
    'reference editor connects input, native caret and semantic selection',
    (tester) async {
      final semantics = tester.ensureSemantics();
      final transport = ViewTransport();
      final document = await NativeDocument.connect(transport);
      final opened = document.newDocument();
      await pumpUntil(tester, () => document.info != null);
      await opened;
      await tester.pumpWidget(
        MaterialApp(home: NativeEditorView(document: document)),
      );
      await pumpUntil(
        tester,
        () => transport.requests.any((r) => r['op'] == 'accessibility'),
      );
      await tester.tapAt(tester.getCenter(find.byType(NativeEditorView)));
      await pumpUntil(tester, () => tester.testTextInput.hasAnyClients);
      expect(tester.testTextInput.isVisible, isTrue);
      await pumpUntil(
        tester,
        () => tester.testTextInput.log.any(
          (call) => call.method == 'TextInput.setCaretRect',
        ),
      );
      expect(find.bySemanticsLabel('현재 문단 입력'), findsOneWidget);
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: 'AB',
          selection: TextSelection.collapsed(offset: 2),
        ),
      );
      await pumpUntil(
        tester,
        () =>
            transport.requests.any((r) => r['op'] == 'applyImeEdit') &&
            document.info!.revision == 1,
      );
      final edit = transport.requests.firstWhere(
        (r) => r['op'] == 'applyImeEdit',
      );
      expect((edit['expected']! as Map)['revision'], 0);
      expect(((edit['edit']! as Map)['replacement']! as Map)['text'], 'B');
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pump(const Duration(milliseconds: 20));
      final close = document.dispose();
      await tester.runAsync(() => close);
      semantics.dispose();
    },
  );
  testWidgets(
    'replacement document reconnects input owner and old view disposes safely',
    (tester) async {
      final transport = ViewTransport();
      final document = await NativeDocument.connect(transport);
      final opened = document.newDocument();
      await pumpUntil(tester, () => document.info != null);
      await opened;
      await tester.pumpWidget(
        MaterialApp(home: NativeEditorView(document: document)),
      );
      await pumpUntil(
        tester,
        () => transport.requests.any((r) => r['op'] == 'accessibility'),
      );
      final generation = document.editorGeneration;
      final replacement = document.newDocument();
      await pumpUntil(tester, () => document.editorGeneration > generation);
      await replacement;
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pump(const Duration(milliseconds: 20));
      await tester.runAsync(document.dispose);
      expect(tester.takeException(), isNull);
    },
  );
}
