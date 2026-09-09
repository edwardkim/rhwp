# Task M100 #6107 — 3단계 완료 보고서

- **이슈**: [#6107](https://github.com/edwardkim/rhwp/issues/6107)
- **단계**: 선택 페이지 기준 가로·세로 눈금자와 핀 정렬
- **선행 commit**: `a0b142ceb`
- **완료일**: 2026-08-26 KST

## 완료 내용

### 활성 페이지 snapshot 소비

- `Ruler`가 `active-page-changed`를 구독해 `pageIndex`와 `editing | viewport` 출처를 보관한다.
- 문서 교체 시 `CanvasView.reset()`이 이전 snapshot을 `null`로 발행해 새 문서가 준비되기 전
  눈금자가 이전 페이지를 잠시 재사용하지 않게 했다.
- snapshot이 없거나 현재 문서 범위를 벗어나면 눈금과 핀을 그리지 않는다.

### 가로 눈금자

- 페이지 화면 X를 0번 페이지가 아니라
  `getPageLeftResolved(activePageIndex, totalWidth) - scrollX`에서 계산한다.
- 용지 너비, 여백, 제본 여백, 맞쪽 뒤바꿈과 다단 정보도 같은 활성 페이지의 `PageInfo`를 사용한다.
- 두 쪽·맞쪽·여러 쪽 배치에서 기존의 핀 전체 숨김을 제거하고 활성 페이지 한 쪽의 좌우
  쪽 여백 핀만 표시한다.
- `source === viewport`일 때는 화면 밖 캐럿의 문단·셀·다단 정보를 다른 페이지 속성처럼
  표시하지 않는다. 페이지 소유인 좌우 여백 핀은 유지하고 문단 들여쓰기 핀만 숨긴다.

### 세로 눈금자

- `getVisiblePages()` 반복과 모든 가시 페이지의 중복 눈금·핀 생성을 제거했다.
- 활성 페이지의 `getPageOffset(pageIndex) - scrollY`, 높이와 위·아래 여백만 사용한다.
- 같은 행에 여러 페이지가 있어도 위·아래 여백 핀은 활성 페이지 인덱스를 가진 한 쌍만 만든다.

### 핀 commit 대상 고정

- 가로 핀도 세로 핀처럼 드래그 시작 시 `pageIdx`를 저장한다.
- 드래그 중 활성 페이지가 바뀌어도 잡은 페이지를 계속 그리며, mouseup commit은 저장한
  `pageIdx`를 사용한다.
- 따라서 화면 좌표, `PageInfo`, 미리보기와 실제 `setPageMargin` 대상이 한 페이지로 유지된다.

## 변경 파일

- `rhwp-studio/src/view/ruler.ts`
- `rhwp-studio/src/view/canvas-view.ts`
- `rhwp-studio/tests/ruler-active-page.test.ts`
- `rhwp-studio/tests/active-page-integration.test.ts`
- `rhwp-studio/tests/zoom-anchor.test.ts`

## 검증

```text
$ cd rhwp-studio
$ node --test \
    tests/ruler-active-page.test.ts \
    tests/ruler-pin-geometry.test.ts \
    tests/active-page.test.ts \
    tests/active-page-integration.test.ts \
    tests/page-scroll-step.test.ts \
    tests/virtual-scroll-page-arrangement.test.ts \
    tests/zoom-anchor.test.ts \
    tests/canvas-view-blank-page-placeholder.test.ts
tests 57, pass 57, fail 0

$ npx tsc --noEmit
exit 0
```

focused 회귀에는 다음 계약이 포함된다.

- 활성 페이지 snapshot 구독과 문서 교체 시 초기화
- 활성 페이지의 X/Y 좌표와 `PageInfo` 단일 사용
- 세로 눈금의 가시 페이지 반복 제거
- 여러 쪽 배치의 활성 페이지 여백 핀
- viewport fallback의 문단 핀 억제
- 드래그 시작 페이지와 commit `pageIdx` 일치
- 기존 쪽 여백·들여쓰기 좌표 왕복, 제본·맞쪽 처리와 zoom anchor

## 계획 대비 차이

- 문서 교체 중 Ruler가 이전 snapshot을 재사용하지 않도록 `CanvasView.reset()`의 null 발행을
  함께 추가했다. 활성 페이지 소비처를 안전하게 연결하기 위한 3단계 범위의 lifecycle 보완이다.
- 실제 페이지 클릭·가로/세로 이동·핀 드래그의 브라우저 시각 검증과 증적은 계획대로
  4단계에서 수행한다.

## 다음 단계

4단계에서 전체 Studio 테스트·빌드 회귀를 수행하고, 실제 브라우저에서 세로·가로·두 쪽·맞쪽·
여러 쪽 배치와 페이지별 용지/여백 차이를 점검해 최종 검증 보고서를 작성한다.
