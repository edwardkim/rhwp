// 관계형: 실제 Native CPU 영역과 Flutter viewport pixels의 동일성 검사.
// 주석 갱신: 2026-11-08T09:10:21+09:00; 신설 파일이므로 직전 파일 commit 없음.
// 2026-11-08 실행: 공개 radial-corner-print-contract.hwpx 6쪽 중 2쪽,
// DPR 1의 300x200 viewport RGBA 240,000 bytes가 direct Native CPU와 완전히 일치했다.
// 한컴 독립 절대 좌표/golden을 고정하지 않으며 이 검사는 Hancom raw 측정이 아니다.
// 동일 입력의 기존 Native/fresh WASM 전6쪽 raw100% 자료는
// mydocs/pr/assets/seo-rii-20261107/visual/radial-native.tsv 및 radial-wasm.tsv의 통합 이력이다.
// 이 브랜치에서 전쪽 Visual Sweep을 새로 실행한 결과로 간주하지 않는다.
// 근거: bindings/Flutter/README.md의 다중 페이지 CPU 참조 viewport(P105).
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native_flutter.dart';

Future<void> pumpUntil(WidgetTester tester, bool Function() ready) async {
  for (var i = 0; i < 1000 && !ready(); ++i) {
    await tester.pump(const Duration(milliseconds: 10));
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 5)),
    );
  }
  expect(ready(), isTrue);
}

const _ownerLibrary = String.fromEnvironment('RHWP_OWNER_LIBRARY');
const _nativeLibrary = String.fromEnvironment('RHWP_NATIVE_LIBRARY');
const _fixture = String.fromEnvironment(
  'RHWP_MULTIPAGE_DOCUMENT',
  defaultValue:
      '../../tests/fixtures/gradient/radial-corner-print-contract.hwpx',
);

void main() {
  testWidgets(
    'actual six-page Native fixture scrolls and matches direct CPU pixels',
    (tester) async {
      final document = await tester.runAsync(() async {
        final transport = await NativeOwnerTransport.connect(
          dispatcherLibrary: _ownerLibrary,
          nativeLibrary: _nativeLibrary,
        );
        final document = await NativeDocument.connect(transport);
        await document.openBytes(await File(_fixture).readAsBytes());
        return document;
      });
      expect(document!.info!.pageCount, 6);
      final controller = NativePagedViewportController();
      final boundaryKey = GlobalKey();
      NativeDocumentViewport? visible;
      try {
        await tester.pumpWidget(
          MaterialApp(
            home: MediaQuery(
              data: const MediaQueryData(devicePixelRatio: 1),
              child: Center(
                child: SizedBox(
                  width: 300,
                  height: 200,
                  child: RepaintBoundary(
                    key: boundaryKey,
                    child: NativePagedDocumentView(
                      document: document,
                      controller: controller,
                      editable: false,
                      onViewportReady: (layout) => visible = layout,
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
        await pumpUntil(tester, () => visible != null);
        controller.jumpToPage(1);
        visible = null;
        await pumpUntil(
          tester,
          () => visible?.visiblePages.containsKey(1) == true,
        );
        await tester.pump();
        final layout = visible!;
        final boundary =
            boundaryKey.currentContext!.findRenderObject()!
                as RenderRepaintBoundary;
        final image = await tester.runAsync(
          () => boundary.toImage(pixelRatio: 1),
        );
        final captured = await tester.runAsync(() => image!.toByteData());
        image!.dispose();
        expect(captured, isNotNull);
        final page = layout.visiblePages.keys.single;
        expect(page, 1);
        final region = layout.regionFor(page);
        final reference = await tester.runAsync(
          () => document.renderRegion(
            page: page,
            x: region.left,
            y: region.top,
            width: region.width,
            height: region.height,
            scale: layout.scale,
          ),
        );
        final actual = captured!.buffer.asUint8List();
        final frame = reference!;
        final source = layout.visiblePages[page]!.origin;
        expect(source, Offset.zero);
        expect(frame.width, greaterThanOrEqualTo(300));
        expect(frame.height, greaterThanOrEqualTo(200));
        for (var y = 0; y < 200; y++) {
          expect(
            actual.sublist(y * 300 * 4, (y + 1) * 300 * 4),
            frame.pixels.sublist(y * frame.stride, y * frame.stride + 300 * 4),
            reason: 'visible row $y',
          );
        }
      } finally {
        await tester.pumpWidget(const SizedBox());
        await tester.pump();
        await tester.runAsync(document.dispose);
        controller.dispose();
      }
    },
    skip: _ownerLibrary.isEmpty || _nativeLibrary.isEmpty,
  );
}
