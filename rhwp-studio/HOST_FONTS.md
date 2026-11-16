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
기본 호출은 Rust 조판 선택에 참여하지 않는다. 아래 명시적 옵션으로 그 범위를 확장한다.
네이티브 IPC, 출력 snapshot 고정, 임베디드 glyph resource 교체는 이 API의 범위가 아니다. 자동 backend 선택의 기존 적합성 검사를 우회하지 않으며, 호스트 목록만으로 CanvasKit
eligibility를 보장하지 않는다. 직접 수용 검증은 명시적 CanvasKit·Canvas2D 화면에서 한다.

목록은 최대 25,000 faces, face당 별칭 32개, 문자열 1,024자, 읽기당 64 MiB로 제한한다.
이 제한은 Studio의 메모리 경계이며 호스트의 전송·권한 정책을 대신하지 않는다.

## 명시적 Canvas2D 조판 참여

호스트는 `setProvider(provider, { layout: true })`로 원본 선언에 대응하는 실제
프로그램의 조판 참여를 선택한다. 옵션 생략과 `{ layout: false }`는 기존 화면
face 공급 계약이다. 옵션은 공급자 연결 단위이며 문서 교체에도 유지된다.
`setProvider`를 다시 호출하면 그 호출의 옵션으로 결정하고, null 연결 해제 시
비활성화한다. 목록의 revision 변경은 연결 옵션을 유지한다.

활성화한 Canvas2D에서는 문서의 원본·대체 후보에 대응하는 승인된 SFNT/TTC를
읽고 선택 face의 로드 성공을 확인한 뒤 공통 문서 글꼴 환경에 전달한다.
프로그램 ID는 Native와 같은 SHA-256(원본 파일 바이트 + big-endian u32 faceIndex)이며,
TTC도 원본 컨테이너와 선택 index를 Rust에 함께 전달한다. 선택한 바이트를 해당
공급자 세대의 측정·화면 준비에서 재사용한다. 문서 교체와 공급자 갱신은
준비 중인 결과의 적용 여부를 문서·공급자 세대로 검사한다.

조판 참여 시 정적 OpenType의 측정용 표는 원본 신원과 함께 보관한다. CFF 윤곽은
각 화면 paint에 필요한 프로그램만 등록하고 호출 종료 시 해제한다. 전체 문서의
미사용 쪽 윤곽으로 현재 쪽의 자원 한도를 소진하지 않으며, 기존 읽기·세션 한도는
유지한다. 이 수명 관리는 문서의 원본 메트릭 정책과 별개다.

자동 환경의 `preserveSourceMetrics` 정책을 유지한다. 이 옵션은
`preserveSourceMetrics: false`를 요청하지 않는다. 문서별 사용자 선택은
기존 `replaceSourceMetricsFor` 계약으로 적용한다. 저장 LineSeg·폭·프레임은
기존 유효성·편집 재조판 계약에 따라 사용한다.

목록 제거와 옵션 비활성화는 기능이 소유한 자동 환경과 자원을 해제한다.
공급자 연결을 해제하면 브라우저 로컬의 현재 유효한 프로그램 준비를 다시 수행한다.
사용자 명시 환경은 자동 환경의 소유권 검사로 보호한다. CanvasKit은 이 옵션의
조판 선택 대상에서 제외되며 기존 화면 공급 경로를 사용한다.

`setProvider`의 Promise는 계속 목록 준비까지만 보장한다. 화면은 문서 환경과
renderer 준비를 마친 뒤 갱신된다. 자동 환경은 저장 문서의 글꼴 선언과
dirty/undo 기록을 편집하지 않는다. portable SVG는 문서 환경의 선택을 소비하며
호스트 FontFace의 내부 별칭과 바이트는 내장하지 않는다. 호스트가 출력에 필요한
동일 글꼴을 별도로 제공해야 한다.
