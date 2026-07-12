# Task M100 #2217 Stage 5 - 스크롤 렌더 대기 제거

## 목표

`samples/issue2217/20200830.hwp`를 Chrome에서 연 뒤 스크롤하거나 캐럿을
이동할 때, 이미지 재렌더와 프리페치가 스크롤 이벤트와 같은 메인 스레드를
장시간 점유하지 않도록 한다.

## 재현 및 분석

- 실제 Chrome Console에서 `viewport-manager.ts`의 scroll handler가 최대
  186초, `page-renderer.ts` 이미지 재렌더 timer가 최대 79초를 점유했다.
- 현재 구현은 이미지 페이지마다 200/600/1500ms timer 3개와 image prefetch
  완료 재렌더 1개를 모두 등록한다.
- `updateVisiblePages()`가 현재 보이지 않는 프리페치 페이지도 scroll event
  처리 안에서 동기 렌더한다.

## 변경 범위

1. scroll event를 animation frame마다 하나로 합친다.
2. 현재 보이는 페이지를 우선 렌더하고, 인접 프리페치는 idle callback으로
   미룬다.
3. 이미지 decode 재렌더는 decode 완료 또는 fallback 중 한 번만 실행한다.
4. Canvas2D에서는 로컬 글꼴 감지 뒤의 불필요한 전체 문서 재로딩을 제거하고,
   초기 본문 렌더에서 정적 이미지 layer를 분리한다.
5. 일반 flow image는 DOM image layer로 표시해 Canvas2D의 동기 image 합성을 피하고,
   OLE/raw SVG는 기존 Canvas fallback을 유지한다.

## 검증 계획

- source-level 회귀 테스트로 scroll coalescing, idle prefetch, 단일 image
  rerender 보장을 확인한다.
- WASM build 후 `20200830.hwp` 브라우저 로드 및 스크롤 시나리오를 확인한다.

## 검증 결과

- `node --test tests/render-backend.test.ts tests/document-initialization-order.test.ts`:
  37 passed.
- `npm run build`: passed.
- Chrome headless, DPR 2, `20200830.hwp`:
  - 초기 완료 `118.9ms`, 입력 textarea 활성 상태 확인.
  - flow image DOM layer가 표시되고, 문서 첫 페이지의 악보 이미지와 crop 결과를
    시각적으로 확인.
  - 연속 스크롤 뒤 page 2/3의 `flow`만 각각 약 42ms/27ms 렌더했다.
  - `flow-static` Canvas 재렌더는 0회였다.
