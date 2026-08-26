# Task M100 #6107 — 2단계 완료 보고서

- **이슈**: [#6107](https://github.com/edwardkim/rhwp/issues/6107)
- **단계**: CanvasView 활성 페이지 snapshot과 document-agent 2D 가시성 통합
- **선행 commit**: `0e1facfea`
- **완료일**: 2026-08-26 KST

## 완료 내용

### 단일 활성 페이지 발행 경로

- `CanvasView`가 최근 편집 페이지, 현재 2D 가시 페이지 집합과 viewport fallback을 보관한다.
- 캐럿·텍스트 선택은 `cursor-rect-updated.pageIndex`, 그림·표 개체 선택은
  `editing-page-changed`로 실제 편집 페이지를 전달한다.
- 스크롤과 편집 focus 변경은 모두 `resolveActivePage()`와 `updateActivePageSnapshot()`을 거쳐
  `active-page-changed` snapshot을 발행한다.
- 기존 `current-page-changed` 상태 표시줄 이벤트도 같은 snapshot의 `pageIndex`에서 파생한다.
- snapshot은 `pageIndex`와 `editing | viewport` 출처가 모두 같을 때만 중복 발행을 생략한다.
  상태 표시줄 이벤트는 pagination으로 전체 쪽 수·구역 쪽번호가 바뀌는 경우를 위해 가시 페이지
  갱신마다 유지한다.

### 편집 페이지 전달

- 일반 캐럿, IME 조합 중 캐럿과 드래그 selection focus가 공유하는 두 cursor rect 발행 경로에
  `pageIndex`를 추가했다.
- 그림·직선·다중 그림 선택은 실제 선택 bbox 페이지를 전달한다.
- 여러 페이지에 걸친 표 선택은 현재 cursor page가 표 bbox 집합에 있으면 그 페이지를 우선하고,
  그렇지 않으면 첫 실제 표 bbox 페이지를 사용한다.

### 2D viewport fallback과 #2560 보존

- 가로 이동 fallback은 viewport X/Y 중심의 실제 페이지를 사용한다.
- 세로 이동 fallback은 기존 #2560 계약대로 viewport 중심 Y가 속한 행의 첫 실제 페이지를 사용한다.
- 두 경우 모두 resolver가 편집 페이지를 우선하므로 한 행의 다른 페이지를 클릭하면 스크롤 없이
  활성 페이지와 상태 표시줄이 그 페이지로 전환된다.
- 편집 페이지가 화면 밖으로 나가면 위 fallback으로 전환된다.

### document-agent strict render

- `refreshDocumentAgentMutation()`의 실제 가시 페이지 확인에 `scrollY`, `viewportHeight`,
  `scrollX`, `viewportWidth` 네 값을 모두 전달한다.
- 가로 이동에서 화면 밖이지만 같은 Y행에 있는 페이지를 잘못 검사하던 경로를 제거했다.

## 변경 파일

- `rhwp-studio/src/view/canvas-view.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/engine/input-handler-picture.ts`
- `rhwp-studio/tests/active-page-integration.test.ts`

## 검증

```text
$ cd rhwp-studio
$ node --test \
    tests/active-page.test.ts \
    tests/active-page-integration.test.ts \
    tests/page-scroll-step.test.ts \
    tests/virtual-scroll-page-arrangement.test.ts \
    tests/render-backend.test.ts \
    tests/viewport-manager-smooth-zoom.test.ts \
    tests/canvas-view-blank-page-placeholder.test.ts
tests 101, pass 101, fail 0

$ npx tsc --noEmit
exit 0
```

focused 회귀에는 다음 계약이 포함된다.

- 일반·드래그 캐럿의 page index 전달
- 그림·표 개체 선택의 실제 페이지 전달
- 활성 페이지 resolver 단일 관문과 상태 표시줄 파생
- document-agent의 X/Y 가시성 검사
- #2560 세로 그리드 행 기준, 가로 페이지 배치·프리페치
- zoom/scroll coalescing과 기존 CanvasView 렌더 경로

## 계획 대비 차이

- 구현 계획의 일반적인 “viewport 중심 실제 페이지” 표현을 이슈 수용 기준에 맞게 구체화했다.
  가로 이동은 X/Y 중심 페이지, 세로 이동은 #2560의 행 첫 페이지를 fallback으로 사용한다.
- 눈금자는 아직 `active-page-changed`를 소비하지 않는다. 계획대로 3단계에서 snapshot의 페이지와
  출처를 이용해 좌표·용지 정보·핀 표시 및 commit 대상을 함께 정렬한다.

## 다음 단계

3단계에서 Ruler가 `active-page-changed`를 구독하고, 가로·세로 눈금과 여백 핀이 활성 페이지 한 쪽의
X/Y 위치·용지 속성·구역을 일관되게 사용하도록 변경한다.
