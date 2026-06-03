# Task #1129 Stage 9 - 쪽 클립 코너와 하단 격자 범위 보정

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 8 커밋 후 사용자 수동 비교에서 한컴오피스 기준과 다른 점이 남았다.

비교 결과:

- `쪽` 클립 좌상단 코너 표시가 보이지 않는다.
- 격자가 페이지 하단 경계를 넘어 표시된다.

## 판단

Stage 8에서 `쪽` 기준 clip을 1px 안쪽으로 단순 보정했지만, 한컴오피스 화면의 쪽 클립 표시는 보존해야 하고 격자 표시 영역은 하단 쪽 경계 안에서 끝나야 한다.

현재 overlay는 canvas 위 sibling으로 표시되므로, 쪽 클립 코너 표시가 overlay보다 앞에 있어야 한다. 또한 clip 하단 inset은 쪽 경계와 페이지 표시 경계의 실제 차이를 더 엄격하게 반영해야 한다.

추가 확인 결과 `PageRenderer.drawMarginGuides()`는 쪽 클립 코너를 다음 기준으로 그린다.

- 좌: `marginLeft`
- 상: `marginTop + marginHeader`
- 우: `marginRight`
- 하: `marginBottom + marginFooter`

Stage 8의 격자 clip은 `pageBorder*` 기준이라 쪽 클립 코너보다 바깥쪽에 격자가 표시되고 있었다.

## 수정 방향

- `쪽` 기준 clip은 `drawMarginGuides()`와 같은 본문 클립 기준으로 맞춘다.
- `쪽` 기준 격자 하단은 `marginBottom + marginFooter` 기준으로 잘리도록 보정한다.
- `samples/hwp3-sample16-hwp5.hwp` 기준으로 `쪽/10mm/0,0` 자동 검증을 수행한다.

## 수정 내용

- `rhwp-studio/src/view/grid-overlay.ts`
  - `getPageGridAreaPx()`를 `pageBorder*` 기준에서 본문 클립 코너 기준으로 변경했다.
  - `PageRenderer.drawMarginGuides()`와 같은 계산을 사용하도록 주석을 추가했다.

## 검증 계획

- 로컬 Playwright 기능 검증
  - `samples/hwp3-sample16-hwp5.hwp` 로드
  - HWPX 비표준 감지 모달에서 `그대로 보기`
  - `쪽/10mm/0,0` 설정
  - overlay `clip-path`, `background-position`, `background-size`, `background-image`, `opacity` 기록
  - 좌상단/하단 보조 표시 DOM 여부 기록
- `npm run build`
- `cargo fmt --all -- --check && git diff --check`

## 검증 결과

- 로컬 Playwright 기능 검증 통과
  - 샘플: `samples/hwp3-sample16-hwp5.hwp`
  - 모달: `그대로 보기`
  - 설정: `쪽/10mm/0,0`
  - 기준: `marginLeft / marginTop+marginHeader / marginBottom+marginFooter`
  - 예상 guide px:
    - left: `62.8650623661333`
    - top: `83.82008315484438`
    - right: `62.8650623661333`
    - bottom: `83.82008315484438`
  - overlay 수: `2`
  - `background-size`: `41.9048px 41.9048px`
  - `background-position`: `62.8651px 83.8201px`
  - `clip-path`: `inset(84.8201px 63.8651px)`
  - `opacity`: `1`
- `npm run build` 통과
  - 기존 큰 chunk 경고만 표시됨
- `cargo fmt --all -- --check && git diff --check` 통과

## 대기

자동 검증 후 커밋하고, 최종 정합 여부는 작업지시자의 수동 비교를 기다린다.
