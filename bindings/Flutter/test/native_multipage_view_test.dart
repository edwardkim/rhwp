// 합성 계약: 관계형 viewport/소유권·고정형 자원 한도와 resize cover 검사.
// 주석 갱신: 2026-11-08T09:10:21+09:00; 신설 파일이므로 직전 파일 commit 없음.
// 초기 geometry·worker·Flutter raster 8개에 lifecycle editor 관측 전달을 추가했다.
// resize clear를 생략한 격리 사본은 이전 검정 pixel을 노출해 실패했다.
// 실제 문서의 절대 조판 좌표나 독립 한컴 출력 측정이 아니므로 raw/전쪽 TSV는 해당하지 않는다.
// 근거: bindings/Flutter/README.md의 다중 페이지 CPU 참조 viewport(P105).
import 'dart:async';
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

import 'support/fake_transport.dart';
import 'native_editor_view_test.dart' show ViewTransport;
import 'native_document_view_test.dart' show pumpUntil;

class PagedTransport extends FakeTransport {
  int pages = 3;
  final regions = <Map<String, Object?>>[];
  final buffers = <int, Uint8List>{};
  final bufferGates = <Completer<void>>[];
  bool gateNextBuffer = false;
  @override
  Map<String, Object?> get info => {...super.info, 'pageCount': pages};
  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) async {
    final command = request['command'] as Map?;
    final op = command?['op'];
    if (op == 'pageSize') {
      commands.add('pageSize');
      final id = ++serial;
      completions.add({
        'ok': true,
        'requestId': id,
        'kind': 'pageSize',
        'size': {'width': 300.0, 'height': 200.0},
      });
      return {'ok': true, 'requestId': id};
    }
    if (op == 'renderRegion' || op == 'renderTiledRegion') {
      commands.add(op! as String);
      regions.add(Map<String, Object?>.from(command!));
      final id = ++serial;
      final region = command['region']! as Map;
      final scale = command['scale']! as double;
      final left = ((region['x']! as double) * scale).floor();
      final top = ((region['y']! as double) * scale).floor();
      final right =
          (((region['x']! as double) + (region['width']! as double)) * scale)
              .ceil();
      final bottom =
          (((region['y']! as double) + (region['height']! as double)) * scale)
              .ceil();
      final width = right - left, height = bottom - top;
      final buffer = id + 100;
      buffers[buffer] = Uint8List.fromList(
        List.generate(
          width * height * 4,
          (i) => i % 4 == 3 ? 255 : (command['page']! as int) * 50,
        ),
      );
      completions.add({
        'ok': true,
        'requestId': id,
        'kind': 'frame',
        'buffer': buffer,
        'width': width,
        'height': height,
        'stride': width * 4,
        'byteLength': width * height * 4,
        'originX': left,
        'originY': top,
        'revision': revision,
        'fontGeneration': fontGeneration,
      });
      return {'ok': true, 'requestId': id};
    }
    return super.hostRequest(request, data: data);
  }

  @override
  Future<Uint8List> copyAndReleaseBuffer(int buffer, int byteLength) async {
    copied.add(buffer);
    try {
      if (gateNextBuffer) {
        gateNextBuffer = false;
        final gate = Completer<void>();
        bufferGates.add(gate);
        await gate.future;
      }
      return buffers[buffer]!;
    } finally {
      released.add(buffer);
      buffers.remove(buffer);
    }
  }

  @override
  Future<void> releaseBuffer(int buffer) async {
    released.add(buffer);
    buffers.remove(buffer);
  }
}

Future<void> advance(WidgetTester tester, [int frames = 80]) async {
  for (var i = 0; i < frames; i++) {
    await tester.pump(const Duration(milliseconds: 8));
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 1)),
    );
  }
}

void main() {
  test('variable page placement maps visible regions and pointers with one transform', () {
    final layout = NativeDocumentViewport(
      pages: const [
        NativePageSize(300, 200),
        NativePageSize(240, 300),
        NativePageSize(300, 100),
      ],
      viewport: const Size(300, 200),
      scrollOffset: 166,
      deviceScale: 2,
    );
    expect(layout.visiblePages.keys, [0, 1]);
    expect(layout.regionFor(0), const Rect.fromLTWH(0, 166, 300, 34));
    expect(layout.regionFor(1), const Rect.fromLTWH(0, 0, 240, 150));
    final page = layout.visiblePages[1]!;
    expect(page.origin, const Offset(30, 50));
    expect(page.toNative(const Offset(50, 80)), const Offset(20, 30));
    expect(
      page.caret({'x': 20, 'y': 30, 'height': 12}),
      const Rect.fromLTWH(50, 80, 1, 12),
    );
    expect(layout.rasterBytes, (300 * 34 + 240 * 150) * 4 * 4);
  });
  test('zoom and horizontal scroll preserve logical coordinates independently of DPR', () {
    final one = NativeDocumentViewport(
      pages: const [NativePageSize(300, 500)],
      viewport: const Size(300, 200),
      zoom: 2,
      scrollOffset: 100,
      horizontalOffset: 50,
    );
    final two = NativeDocumentViewport(
      pages: const [NativePageSize(300, 500)],
      viewport: const Size(300, 200),
      zoom: 2,
      scrollOffset: 100,
      horizontalOffset: 50,
      deviceScale: 2,
    );
    expect(one.visiblePages[0]!.origin, const Offset(-50, -100));
    expect(one.regionFor(0), const Rect.fromLTWH(25, 50, 150, 100));
    expect(
      one.visiblePages[0]!.toNative(const Offset(10, 20)),
      const Offset(30, 60),
    );
    expect(two.regionFor(0), one.regionFor(0));
    expect(two.rasterBytes, one.rasterBytes * 4);
  });
  test(
    'invalid and excessive visibility is rejected before worker admission',
    () {
      for (final dpr in [0.0, double.infinity, 9.0]) {
        expect(
          () => NativeDocumentViewport(
            pages: const [NativePageSize(300, 200)],
            viewport: const Size(300, 200),
            deviceScale: dpr,
          ),
          throwsArgumentError,
        );
      }
      expect(
        () => NativeDocumentViewport(
          pages: List.filled(257, const NativePageSize(1, 1)),
          viewport: const Size(100, 100),
        ),
        throwsArgumentError,
      );
      expect(
        () => NativeDocumentViewport(
          pages: List.filled(10, const NativePageSize(100, 10)),
          viewport: const Size(100, 400),
        ),
        throwsA(isA<NativeCallException>()),
      );
      expect(
        () => NativeDocumentViewport(
          pages: const [NativePageSize(3000, 3000)],
          viewport: const Size(3000, 3000),
          deviceScale: 2,
        ),
        throwsA(isA<NativeCallException>()),
      );
    },
  );
  testWidgets('a queued jump resets when the document owner is replaced', (
    tester,
  ) async {
    final first = await NativeDocument.connect(PagedTransport());
    final secondTransport = PagedTransport()..pages = 1;
    final second = await NativeDocument.connect(secondTransport);
    final firstOpen = first.newDocument(), secondOpen = second.newDocument();
    await pumpUntil(tester, () => first.info != null && second.info != null);
    await firstOpen;
    await secondOpen;
    final controller = NativePagedViewportController();
    Widget view(NativeDocument document) => MaterialApp(
      home: Center(
        child: SizedBox(
          width: 300,
          height: 200,
          child: NativePagedDocumentView(
            document: document,
            controller: controller,
            editable: false,
          ),
        ),
      ),
    );
    await tester.pumpWidget(view(first));
    await advance(tester);
    expect(controller.pageCount, 3);
    controller.jumpToPage(2);
    await tester.pumpWidget(view(second));
    await advance(tester);
    expect(controller.pageCount, 1);
    expect(controller.currentPage, 0);
    expect(secondTransport.regions, isNotEmpty);
    expect(secondTransport.regions.every((r) => r['page'] == 0), isTrue);
    await tester.pumpWidget(const SizedBox());
    await first.dispose();
    await second.dispose();
    controller.dispose();
    await tester.pump();
  });

  testWidgets(
    'a queued jump follows fresh page count after a same-owner shrink',
    (tester) async {
      final transport = PagedTransport();
      final document = await NativeDocument.connect(transport);
      final opened = document.newDocument();
      await pumpUntil(tester, () => document.info != null);
      await opened;
      final controller = NativePagedViewportController();
      await tester.pumpWidget(
        MaterialApp(
          home: Center(
            child: SizedBox(
              width: 300,
              height: 200,
              child: NativePagedDocumentView(
                document: document,
                controller: controller,
                editable: false,
              ),
            ),
          ),
        ),
      );
      await advance(tester);
      expect(controller.pageCount, 3);
      controller.jumpToPage(2);
      transport.pages = 1;
      final refreshed = document.refreshInfo();
      await pumpUntil(tester, () => document.info!.pageCount == 1);
      await refreshed;
      await advance(tester);
      expect(controller.pageCount, 1);
      expect(controller.currentPage, 0);
      await tester.pumpWidget(const SizedBox());
      await document.dispose();
      controller.dispose();
      await tester.pump();
    },
  );

  testWidgets('paged viewport forwards its owned editor lifecycle observer', (
    tester,
  ) async {
    final transport = ViewTransport();
    final document = await NativeDocument.connect(transport);
    final opened = document.newDocument();
    await pumpUntil(tester, () => document.info != null);
    await opened;
    NativeEditorController? editor;
    await tester.pumpWidget(
      MaterialApp(
        home: Center(
          child: SizedBox(
            width: 24,
            height: 24,
            child: NativePagedDocumentView(
              document: document,
              onControllerChanged: (value) => editor = value,
            ),
          ),
        ),
      ),
    );
    await pumpUntil(tester, () => editor?.ready == true);
    await tester.pumpWidget(const SizedBox());
    expect(editor, isNull);
    await document.dispose();
    await tester.pump();
  });

  testWidgets(
    'one generation renders only visible pages, navigation has no competing page views',
    (tester) async {
      final transport = PagedTransport();
      final document = await NativeDocument.connect(transport);
      final opened = document.newDocument();
      await pumpUntil(tester, () => document.info != null);
      await opened;
      final controller = NativePagedViewportController();
      await tester.pumpWidget(
        MaterialApp(
          home: SizedBox(
            width: 300,
            height: 200,
            child: Center(
              child: SizedBox(
                width: 300,
                height: 200,
                child: NativePagedDocumentView(
                  document: document,
                  controller: controller,
                  editable: false,
                ),
              ),
            ),
          ),
        ),
      );
      await advance(tester);
      expect(controller.pageCount, 3);
      expect(transport.regions.map((r) => r['page']).toSet(), {0});
      expect(find.byType(NativeDocumentView), findsNothing);
      controller.scrollTo(166);
      await advance(tester);
      expect(transport.regions.takeLast(2).map((r) => r['page']), [0, 1]);
      final requested = transport.regions.last['region']! as Map;
      expect(requested['y'], 0);
      expect(requested['height'], 150);
      controller.jumpToPage(2);
      await advance(tester);
      expect(controller.currentPage, 2);
      expect(transport.regions.last['page'], 2);
      expect(transport.buffers, isEmpty);
      expect(transport.copied.length, transport.released.length);
      await tester.pumpWidget(const SizedBox());
      await advance(tester, 2);
      await document.dispose();
      controller.dispose();
    },
  );
  testWidgets(
    'rapid navigation cancels an old copy and publishes the latest request only',
    (tester) async {
      final transport = PagedTransport()..gateNextBuffer = true;
      final document = await NativeDocument.connect(transport);
      final opened = document.newDocument();
      await pumpUntil(tester, () => document.info != null);
      await opened;
      final controller = NativePagedViewportController();
      await tester.pumpWidget(
        MaterialApp(
          home: Center(
            child: SizedBox(
              width: 300,
              height: 200,
              child: NativePagedDocumentView(
                document: document,
                controller: controller,
                editable: false,
              ),
            ),
          ),
        ),
      );
      await advance(tester, 20);
      expect(transport.bufferGates, hasLength(1));
      controller.jumpToPage(1);
      controller.jumpToPage(2);
      await advance(tester, 2);
      transport.bufferGates.single.complete();
      await advance(tester);
      expect(controller.currentPage, 2);
      expect(transport.regions.last['page'], 2);
      expect(transport.regions.map((r) => r['page']), isNot(contains(1)));
      expect(transport.buffers, isEmpty);
      await tester.pumpWidget(const SizedBox());
      await advance(tester, 2);
      await document.dispose();
      controller.dispose();
    },
  );
  testWidgets(
    'large documents report a bounded limit without loading page metadata',
    (tester) async {
      final transport = PagedTransport()..pages = 257;
      final document = await NativeDocument.connect(transport);
      final opened = document.newDocument();
      await pumpUntil(tester, () => document.info != null);
      await opened;
      await tester.pumpWidget(
        MaterialApp(
          home: NativePagedDocumentView(document: document, editable: false),
        ),
      );
      await advance(tester, 20);
      expect(find.textContaining('pageLimit'), findsOneWidget);
      expect(transport.commands.where((c) => c == 'pageSize'), isEmpty);
      expect(transport.regions, isEmpty);
      await tester.pumpWidget(const SizedBox());
      await document.dispose();
    },
  );
  testWidgets(
    'one editor routes a second-page click through its placed transform',
    (tester) async {
      final transport = ViewTransport();
      final document = await NativeDocument.connect(transport);
      final opened = document.newDocument();
      await pumpUntil(tester, () => document.info != null);
      await opened;
      await tester.pumpWidget(
        MaterialApp(
          home: Center(
            child: SizedBox(
              width: 300,
              height: 200,
              child: NativeEditorView(
                document: document,
                pageViewports: {
                  1: NativePageViewport.placed(
                    const NativePageSize(300, 200),
                    const Size(300, 200),
                    zoom: .5,
                    origin: const Offset(30, 50),
                  ),
                },
                backgroundBuilder: (_) => const ColoredBox(color: Colors.white),
              ),
            ),
          ),
        ),
      );
      await pumpUntil(
        tester,
        () => transport.requests.any((r) => r['op'] == 'accessibility'),
      );
      final box = tester.getTopLeft(find.byType(NativeEditorView));
      await tester.tapAt(box + const Offset(50, 80));
      await advance(tester, 20);
      final points = transport.requests.where((r) => r['op'] == 'hitTest');
      expect(points, isNotEmpty);
      expect(points.first['page'], 1);
      expect(points.first['x'], 40);
      expect(points.first['y'], 60);
      await tester.pumpWidget(const SizedBox());
      await advance(tester, 20);
      await document.dispose();
    },
  );
  testWidgets(
    'the resize frame covers old pixels before deferred rendering starts',
    (tester) async {
      final transport = PagedTransport();
      final document = await NativeDocument.connect(transport);
      final opened = document.newDocument();
      await pumpUntil(tester, () => document.info != null);
      await opened;
      final boundaryKey = GlobalKey();
      var ready = 0;
      Widget surface(double height) => MaterialApp(
        home: Center(
          child: SizedBox(
            width: 300,
            height: height,
            child: RepaintBoundary(
              key: boundaryKey,
              child: NativePagedDocumentView(
                document: document,
                editable: false,
                onViewportReady: (_) => ready++,
              ),
            ),
          ),
        ),
      );
      await tester.pumpWidget(surface(200));
      await pumpUntil(tester, () => ready > 0);
      await tester.pump();
      transport.gateNextBuffer = true;
      await tester.pumpWidget(surface(100));
      // No second framework frame is pumped: this observes the resize frame,
      // not the subsequent asynchronous clear/render frame.
      final boundary =
          boundaryKey.currentContext!.findRenderObject()!
              as RenderRepaintBoundary;
      final image = await tester.runAsync(
        () => boundary.toImage(pixelRatio: 1),
      );
      final bytes = await tester.runAsync(() => image!.toByteData());
      image!.dispose();
      expect(bytes!.buffer.asUint8List().sublist(0, 4), [221, 221, 221, 255]);
      await pumpUntil(tester, () => transport.bufferGates.isNotEmpty);
      transport.bufferGates.single.complete();
      await advance(tester, 30);
      await tester.pumpWidget(const SizedBox());
      await tester.pump();
      await document.dispose();
    },
  );
}

extension<T> on Iterable<T> {
  Iterable<T> takeLast(int count) => skip(math.max(0, length - count));
}
