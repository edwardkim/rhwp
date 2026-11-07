import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native.dart';

import 'support/fake_transport.dart';

void main() {
  test(
    'new/info/page/render returns owned bytes after native release',
    () async {
      final transport = FakeTransport();
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      final opened = await document.newDocument();
      expect((await document.refreshInfo()).documentId, opened.documentId);
      expect((await document.pageSize(0)).width, 4);
      final view = document.beginView();
      final frame = await document.renderRegion(
        page: 0,
        x: 0,
        y: 0,
        width: 4,
        height: 4,
        scale: 1,
        viewGeneration: view,
      );
      expect(frame!.pixels.length, 64);
      expect(transport.released, [42]);
      expect(document.isCurrent(frame.stamp), isTrue);
      expect(transport.commands, [
        'new',
        'info',
        'pageSize',
        'renderTiledRegion',
      ]);
    },
  );

  test(
    'viewport cancellation retains FIFO until actual cancelled completion',
    () async {
      final transport = FakeTransport()..autoRender = false;
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      final render = document.renderRegion(
        page: 0,
        x: 0,
        y: 0,
        width: 4,
        height: 4,
        scale: 1,
      );
      await waitFor(() => transport.commands.contains('renderTiledRegion'));
      final renderId = transport.serial;
      document.beginView();
      final info = document.refreshInfo();
      await waitFor(() => transport.commands.contains('cancel'));
      expect(transport.commands.where((value) => value == 'info'), isEmpty);
      transport.completions.add({
        'ok': true,
        'kind': 'cancelled',
        'requestId': renderId,
      });
      expect(await render, isNull);
      await info;
      expect(transport.commands, [
        'new',
        'renderTiledRegion',
        'cancel',
        'info',
      ]);
    },
  );

  for (final stale in ['view', 'font', 'revision']) {
    test('stale $stale frames are released without a pixel copy', () async {
      final transport = FakeTransport()..autoRender = false;
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      final render = document.renderRegion(
        page: 0,
        x: 0,
        y: 0,
        width: 4,
        height: 4,
        scale: 1,
      );
      await waitFor(() => transport.commands.contains('renderTiledRegion'));
      if (stale == 'view') document.beginView();
      transport.completions.add(
        transport.frame(
          font: stale == 'font' ? 1 : 0,
          rev: stale == 'revision' ? 1 : 0,
        ),
      );
      expect(await render, isNull);
      expect(transport.copied, isEmpty);
      expect(transport.released, [42]);
    });
  }

  test('view changes during copy reject the late owned frame', () async {
    final transport = FakeTransport()..copyGate = Completer<Uint8List>();
    final document = await NativeDocument.connect(transport);
    addTearDown(document.dispose);
    await document.newDocument();
    final render = document.renderRegion(
      page: 0,
      x: 0,
      y: 0,
      width: 4,
      height: 4,
      scale: 1,
    );
    await waitFor(() => transport.copied.isNotEmpty);
    document.beginView();
    transport.copyGate!.complete(Uint8List(64));
    expect(await render, isNull);
    expect(transport.released, [42]);
  });

  test(
    'queued open snapshots caller bytes and runs after the previous render',
    () async {
      final transport = FakeTransport()..autoRender = false;
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      final oldId = document.info!.documentId;
      final render = document.renderRegion(
        page: 0,
        x: 0,
        y: 0,
        width: 4,
        height: 4,
        scale: 1,
      );
      await waitFor(() => transport.commands.contains('renderTiledRegion'));
      final request = transport.serial;
      final source = Uint8List.fromList([1, 2, 3]);
      final open = document.openBytes(source);
      source.fillRange(0, source.length, 9);
      expect(transport.opened, isEmpty);
      transport.completions.add({
        'ok': true,
        'kind': 'skipped',
        'requestId': request,
      });
      expect(await render, isNull);
      expect((await open).documentId, isNot(oldId));
      expect(transport.opened.single, [1, 2, 3]);
    },
  );

  test(
    'invalid frame metadata releases the native buffer before allocation',
    () async {
      final transport = FakeTransport()..autoRender = false;
      final document = await NativeDocument.connect(transport);
      addTearDown(document.dispose);
      await document.newDocument();
      final render = document.renderRegion(
        page: 0,
        x: 0,
        y: 0,
        width: 4,
        height: 4,
        scale: 1,
      );
      await waitFor(() => transport.commands.contains('renderTiledRegion'));
      transport.completions.add({
        ...transport.frame(),
        'byteLength': maxReferenceBytes + 1,
      });
      await expectLater(render, throwsA(isA<NativeCallException>()));
      expect(transport.copied, isEmpty);
      expect(transport.released, [42]);
    },
  );

  test(
    'dispose rejects requests immediately but waits for owner cleanup',
    () async {
      final transport = FakeTransport()
        ..autoRender = false
        ..closeGate = Completer<void>();
      final document = await NativeDocument.connect(transport);
      await document.newDocument();
      final render = document.renderRegion(
        page: 0,
        x: 0,
        y: 0,
        width: 4,
        height: 4,
        scale: 1,
      );
      final rejected = expectLater(
        render,
        throwsA(isA<NativeDisposedException>()),
      );
      await waitFor(() => transport.commands.contains('renderTiledRegion'));
      final close = document.dispose();
      expect(document.isDisposed, isTrue);
      expect(
        () => document.refreshInfo(),
        throwsA(isA<NativeDisposedException>()),
      );
      await rejected;
      var finished = false;
      unawaited(
        close.then((_) {
          finished = true;
        }),
      );
      await Future<void>.delayed(Duration.zero);
      expect(finished, isFalse);
      transport.closeGate!.complete();
      await close;
      expect(transport.closes, 1);
      await document.dispose();
      expect(transport.closes, 1);
    },
  );

  test(
    'completed or queued requests cannot grow past the bounded FIFO',
    () async {
      final transport = FakeTransport()..autoComplete = false;
      final document = await NativeDocument.connect(transport);
      final futures = <Future<Object?>>[];
      futures.add(
        document.newDocument().then<Object?>(
          (value) => value,
          onError: (Object error) => error,
        ),
      );
      for (var index = 0; index < 63; index++) {
        futures.add(
          document.refreshInfo().then<Object?>(
            (value) => value,
            onError: (Object error) => error,
          ),
        );
      }
      expect(() => document.refreshInfo(), throwsA(isA<NativeCallException>()));
      await document.dispose();
      expect(
        (await Future.wait(futures))
            .whereType<NativeDisposedException>()
            .length,
        64,
      );
    },
  );
}
