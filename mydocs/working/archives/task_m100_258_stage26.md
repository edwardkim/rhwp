# Task M100-258 Stage 26 — 인접 누름틀 실제 선택 표시 보정

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `8cf7de06` (`task 258: 인접 누름틀 경계 선택 보정`)

## 1. 문제

Stage25는 `abc[123][123]`의 field range가 `3..6`, `6..9`로 분리되고
키보드 선택 range가 `3..9`가 되는지 검증했다.

하지만 작업지시자 시각 판정 기준에서는 한컴처럼 두 누름틀 값 `123123` 전체가
하나의 연속 선택 배경으로 보여야 한다. 현재 rhwp-studio 실제 화면에서는 두 누름틀
동시 선택 표시가 한컴 기준과 다르거나, 실제 마우스/키보드 조작으로 그 상태까지
도달하지 못하는 문제가 남아 있다.

## 2. 조사 방향

- 실제 마우스 드래그로 `abc[123][123]`의 첫 누름틀 시작부터 두 번째 누름틀 끝까지 선택되는지 확인한다.
- `Shift+ArrowRight` 선택과 마우스 드래그 선택의 selection range/rect/화면 표시 차이를 비교한다.
- selection renderer가 canvas 텍스트 위에 그리는 선택 배경의 색상, 폭, z-index, field marker와의 겹침을 확인한다.
- Stage25의 field range 분리 보정은 유지한다.

조사 결과:

- 실제 마우스 드래그는 selection range `3..9`, selection rect 폭 `44px`를 정상 생성했다.
- 문제는 range 계산이 아니라 시각 표시였다.
- 기존 `SelectionRenderer`는 파란 반투명 사각형을 canvas 위에 올려 한컴의 검은 반전 선택과 달랐다.
- 선택 중에도 누름틀 field marker `「 」`가 표시되어 선택 영역 위에 붉은 마커가 남았다.

## 3. 수정 방향

- 선택 range가 `3..9`로 계산되는데 시각 표시만 한컴 기준과 다르면 selection renderer를 보정한다.
- 실제 조작으로 `3..9` range가 만들어지지 않으면 input-handler mouse/keyboard boundary 처리를 보정한다.
- 기존 Stage24/Stage25의 단일 누름틀 경계 이동, 빈 누름틀 진입/이탈, 인접 field range 분리 회귀를 유지한다.

수정:

- `SelectionRenderer` 레이어에 `mix-blend-mode: difference`를 적용하고,
  선택 rect는 흰색 배경으로 렌더링해 canvas의 검은 글자가 흰색으로 반전되게 했다.
- 선택 range가 있을 때는 `updateFieldMarkers`가 누름틀 marker를 숨기고 active field를 해제하도록 했다.

## 4. 검증 계획

- `http://localhost:7700/` Browser/Playwright 실제 조작 검증
  - `abc[123][123]` 구성
  - 마우스 드래그 또는 `Shift+ArrowRight`로 `123123` 전체 선택
  - 화면 선택 배경이 두 누름틀 전체를 연속으로 덮는지 확인
- `cargo test --test issue_258_clickhere_form_mode`
- `cd rhwp-studio && npm run build`
- `wasm-pack build --target web --out-dir pkg` 필요 시 수행
- `cargo fmt --check`
- `git diff --check`

## 5. 검증 결과

- Browser plugin: `http://localhost:7700/` 로드, title `rhwp-studio`, console error/warn 없음 확인
  - 단, Browser 탭은 개발용 `__wasm/__inputHandler` 전역이 없어 내부 상태 검증은 Playwright로 수행
- Playwright 실제 마우스 드래그 검증 통과
  - text `abc123123`
  - field ranges `3..6`, `6..9`
  - selection range `3..9`
  - selection rect 폭 `44px`
  - selection layer `mix-blend-mode: difference`
  - field marker `display:none`
  - 확대 스크린샷에서 `123123` 전체가 검은 배경/흰 글자로 선택 표시됨
- `cargo test --test issue_258_clickhere_form_mode`: 통과 (`11 passed`)
- `cd rhwp-studio && npm run build`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과

스크린샷:

- 전체: `/tmp/rhwp-task258-stage26-after-fix2-drag.png`
- 확대: `/tmp/rhwp-task258-stage26-after-fix2-drag-crop.png`
