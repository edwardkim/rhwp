# Task M100 #6107 — 1단계 완료 보고서

- **이슈**: [#6107](https://github.com/edwardkim/rhwp/issues/6107)
- **단계**: 활성 페이지와 2D 페이지 이동 계약
- **기준 commit**: `upstream/devel` `70ebacc4c9589e8c778907e179a6dab18cce8eb0`
- **완료일**: 2026-08-26 KST

## 완료 내용

### 활성 페이지 resolver

- `rhwp-studio/src/view/active-page.ts`에 DOM·WASM 비의존 순수 resolver를 추가했다.
- 보이는 편집 페이지를 우선하고, 편집 페이지가 화면 밖이면 뷰포트 기준 페이지로 전환한다.
- 뷰포트 기준 페이지가 빈 슬롯·범위 밖이면 첫 실제 가시 페이지를 사용한다.
- 빈 문서와 가시 페이지가 없는 상태는 `null`로 명시한다.
- 결과는 `pageIndex`와 `editing | viewport` 출처를 함께 보존해 후속 CanvasView·눈금자 단계가
  문단 핀 표시 여부를 구분할 수 있게 했다.

### 가로·세로 PageUp/PageDown

- `PageScrollResult`를 단일 Y `delta`에서 `deltaX`·`deltaY`로 확장했다.
- 세로 이동은 기존 행 시작 목록, 페이지 경계, 뷰포트 높이와 #2560 그리드 계약을 유지한다.
- 가로 이동은 실제 페이지 왼쪽 경계, 전체 문서 너비, 뷰포트 너비와 `scrollLeft` 한계를 사용한다.
- 페이지가 뷰포트보다 넓으면 한 화면씩 이동하되 모든 페이지 왼쪽 경계를 빠짐없이 지난다.
- 문서 처음·끝에서는 X/Y 실제 브라우저 clamp 결과가 0이면 `moved: false`를 반환한다.

### 캐럿 화면 위치 보존

- `input-handler-keyboard.ts`가 페이지 이동의 X/Y delta를 모두 받아 hit-test 좌표를 보정하게 했다.
- 기존 세로 이동은 `deltaX = 0`, 가로 이동은 `deltaY = 0`이므로 한 경로에서 두 방향을 처리한다.
- 머리말/꼬리말·각주·개체/셀 선택의 화면 전용 이동 예외는 변경하지 않았다.

## 변경 파일

- `rhwp-studio/src/view/active-page.ts`
- `rhwp-studio/src/view/page-scroll.ts`
- `rhwp-studio/src/engine/input-handler-keyboard.ts`
- `rhwp-studio/tests/active-page.test.ts`
- `rhwp-studio/tests/page-scroll-step.test.ts`

## 검증

```text
$ cd rhwp-studio
$ node --test tests/active-page.test.ts tests/page-scroll-step.test.ts
tests 18, pass 18, fail 0

$ npx tsc --noEmit
exit 0
```

focused 테스트에는 다음 계약이 포함된다.

- 보이는 편집 페이지 우선과 화면 밖 편집 페이지 fallback
- 범위 밖/빈 슬롯/빈 문서 처리
- 기존 세로 단일 열, 큰 페이지, 그리드, 맞쪽 이동 무회귀
- 가로 PageDown/PageUp의 모든 실제 페이지 경계 통과
- 가로·세로 이동의 한 화면 상한, 문서 처음·끝 clamp, 실제 X/Y delta

## 계획 대비 차이

- 없음. CanvasView 이벤트 통합, cursor payload의 `pageIndex`, document-agent 가시성 전달은 계획대로
  2단계에 남겼다.
- 실제 브라우저 PageUp/PageDown과 눈금자 시각 검증은 CanvasView·Ruler 연결 뒤 4단계에서 수행한다.

## 다음 단계

2단계에서 캐럿 페이지와 2D 가시 집합을 CanvasView의 활성 페이지 snapshot으로 통합하고,
상태 표시줄과 document-agent 갱신이 같은 판정을 사용하게 한다.
