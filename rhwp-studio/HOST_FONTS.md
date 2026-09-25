# 호스트 글꼴 공급 — #7403

Studio 페이지와 같은 JavaScript realm에서 `window.rhwpStudio.fonts.setProvider(provider)`를
호출한다. 반환 Promise는 목록 준비가 끝나면 완료된다. 화면 갱신 완료를 뜻하지 않는다.
연결 해제는 `setProvider(null)`이다. `getState()`는 활성 여부·세대·목록 수·목록 조회 오류를 반환한다.
공급자 타입은 `src/core/host-font-provider.ts`에 있다. iframe 부모용 RPC는 제공하지 않는다.

```ts
await window.rhwpStudio.fonts.setProvider({
  async getSnapshot(signal) {
    return { revision: 'catalog-1', faces: [{
      id: 'face-regular', family: 'Example', fullName: 'Example Regular',
      postscriptName: 'Example-Regular', style: 'Regular', weight: 400,
      slant: 'normal', aliases: ['예제 글꼴'],
    }] };
  },
  async readFace(id, revision, signal) {
    // 호스트가 요청한 revision의 승인된 face를 읽는다. 경로는 Studio에 전달하지 않는다.
    return { bytes: await hostRead(id, revision, signal), faceIndex: 0 };
  },
  subscribe(onChange) {
    return hostSubscribe(onChange); // 구독 해제 함수를 반환한다.
  },
});
```

호스트는 OS 권한·보관함·동명 파일 선택을 처리하고 사용 가능한 목록을 제공한다.
목록은 메타데이터만 포함하며 파일·선택 결과가 바뀌면 이름과 개수가 같아도 revision을 바꾼다.
동일 revision의 데이터를 제공할 수 없으면 읽기를 거절한다. 변경을 게시한 뒤 구독자에게 알린다.
TTC 데이터는 선택할 `faceIndex`를 지정한다. standalone SFNT는 0이다.

화면 지원은 CanvasKit과 Canvas2D의 `textRun`·`charOverlap` face 경로다.
문서 run의 family와 bold/italic에서 실제 face를 고르고 선택한 ID를 바이트 요청까지 유지한다.
PostScript/full name으로 지정한 face는 문서 bold 플래그 때문에 다른 face로 바꾸지 않는다.
기울임은 정확한 slant를 우선하고, 없으면 유일한 Italic/Oblique 대체 face를 선택한다.
굵기는 요청값과 일치해야 한다. 후보가 없거나 여러 개면 기존 renderer fallback을 사용한다.
지정한 호스트 공급자와 브라우저 목록의 우선순위를 자동 병합하지 않는다.

CanvasKit은 선택한 face를 native Typeface로 준비한다. Canvas2D는 같은 face 데이터를 FontFace로
읽은 뒤 문서별 내부 별칭으로 등록한다. TTC는 선택한 face를 standalone SFNT로 변환한다.
Canvas2D의 보조 글자 폭 측정과 실제 paint는 같은 별칭을 사용한다. 동기 paint 호출 범위에만
별칭을 적용하며, 원래 문서 글꼴명과 전역 CSS 대체 목록은 변경하지 않는다.
Canvas2D의 CSS 설치 글꼴 판정·영속 감지 snapshot에도 호스트 목록을 넣지 않는다.

공급자 교체·변경·해제는 이전 요청을 취소하고 연결 세대까지 검사하여 늦은 결과를 폐기한다.
호스트 목록과 바이트는 browser storage에 저장하지 않는다. 각 renderer는 성공·실패·pending과
native Typeface/FontFace를 문서 자원 경계에서 정리하고 새 페이지를 준비한다. Canvas2D는 등록한
FontFace를 `document.fonts`에서 제거하고 측정 캐시도 무효화한다. 실패한 face는 같은 세대에서
반복 읽지 않으며 새 revision 또는 재연결로 재시도한다. 문서 교체 뒤 도착한 응답은 등록하지 않는다.
이 갱신은 문서 모델과 dirty/undo 상태를 변경하지 않는다. 내부 ID·cache key·FontFace 별칭은
HWP/HWPX 저장 글꼴명이나 `getPageSvg` 결과에 들어가지 않는다.

`getPageSvg`는 기존 portable SVG 경로를 유지하므로 화면의 호스트 FontFace를 내장하지 않는다.
별도 출력 환경에 글꼴을 공급하고 PDF/인쇄를 구성하는 것은 임베딩 호스트의 책임이다.
네이티브 IPC, 출력 snapshot 고정, Rust layout, 임베디드 glyph resource 교체는 이 API의 범위가
아니다. 자동 backend 선택의 기존 적합성 검사를 우회하지 않으며, 호스트 목록만으로 CanvasKit
eligibility를 보장하지 않는다. 직접 수용 검증은 명시적 CanvasKit·Canvas2D 화면에서 한다.

목록은 최대 25,000 faces, face당 별칭 32개, 문자열 1,024자, 읽기당 64 MiB로 제한한다.
이 제한은 Studio의 메모리 경계이며 호스트의 전송·권한 정책을 대신하지 않는다.
