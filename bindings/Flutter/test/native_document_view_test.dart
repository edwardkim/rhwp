import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'support/fake_transport.dart';

Future<void> pumpUntil(WidgetTester tester, bool Function() ready) async {
  for (var index = 0; index < 100 && !ready(); index++) {
    await tester.pump(const Duration(milliseconds: 10));
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 1)),
    );
  }
  expect(ready(), isTrue);
}

void main() {
  testWidgets(
    'CPU bytes decode into a Flutter image and dispose with the viewport',
    (tester) async {
      final transport = FakeTransport();
      final document = await NativeDocument.connect(transport);
      final opened = document.newDocument();
      await pumpUntil(tester, () => document.info != null);
      await opened;
      await tester.pumpWidget(
        MaterialApp(
          home: MediaQuery(
            data: const MediaQueryData(devicePixelRatio: 1),
            child: NativeDocumentView(document: document),
          ),
        ),
      );
      await pumpUntil(
        tester,
        () => find.byType(RawImage).evaluate().isNotEmpty,
      );
      final image = tester.widget<RawImage>(find.byType(RawImage)).image!;
      expect(image.width, 4);
      expect(transport.copied, [42]);
      expect(transport.released, [42]);
      await tester.pumpWidget(const SizedBox.shrink());
      await tester.pump();
      expect(image.debugDisposed, isTrue);
      await document.dispose();
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets('an unmounted viewport cannot publish a late native frame', (
    tester,
  ) async {
    final transport = FakeTransport()..autoRender = false;
    final document = await NativeDocument.connect(transport);
    final opened = document.newDocument();
    await pumpUntil(tester, () => document.info != null);
    await opened;
    await tester.pumpWidget(
      MaterialApp(
        home: MediaQuery(
          data: const MediaQueryData(devicePixelRatio: 1),
          child: NativeDocumentView(document: document),
        ),
      ),
    );
    await pumpUntil(
      tester,
      () => transport.commands.contains('renderTiledRegion'),
    );
    await tester.pumpWidget(const SizedBox.shrink());
    transport.completions.add(transport.frame());
    await pumpUntil(tester, () => transport.released.isNotEmpty);
    expect(transport.copied, isEmpty);
    expect(find.byType(RawImage), findsNothing);
    await document.dispose();
    expect(tester.takeException(), isNull);
  });

  testWidgets(
    'replacement workers with identical IDs cannot publish old frames',
    (tester) async {
      final oldTransport = FakeTransport()..autoRender = false;
      final newTransport = FakeTransport()..autoRender = false;
      final oldDocument = await NativeDocument.connect(oldTransport);
      final newDocument = await NativeDocument.connect(newTransport);
      final opened = [oldDocument.newDocument(), newDocument.newDocument()];
      await pumpUntil(
        tester,
        () => oldDocument.info != null && newDocument.info != null,
      );
      await Future.wait(opened);
      expect(oldDocument.info!.documentId, newDocument.info!.documentId);
      Widget viewport(NativeDocument document) => MaterialApp(
        home: MediaQuery(
          data: const MediaQueryData(devicePixelRatio: 1),
          child: NativeDocumentView(document: document),
        ),
      );
      await tester.pumpWidget(viewport(oldDocument));
      await pumpUntil(
        tester,
        () => oldTransport.commands.contains('renderTiledRegion'),
      );
      await tester.pumpWidget(viewport(newDocument));
      await pumpUntil(
        tester,
        () => newTransport.commands.contains('renderTiledRegion'),
      );
      oldTransport.completions.add(oldTransport.frame());
      await pumpUntil(tester, () => oldTransport.released.isNotEmpty);
      expect(oldTransport.copied, isEmpty);
      expect(find.byType(RawImage), findsNothing);
      newTransport.completions.add(newTransport.frame());
      await pumpUntil(
        tester,
        () => find.byType(RawImage).evaluate().isNotEmpty,
      );
      expect(newTransport.copied, [42]);
      await tester.pumpWidget(const SizedBox.shrink());
      await oldDocument.dispose();
      await newDocument.dispose();
      expect(tester.takeException(), isNull);
    },
  );
}
