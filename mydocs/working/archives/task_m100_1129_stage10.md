# Task #1129 Stage 10 - 쪽 클립 표시 복구와 점 개수 기준 재검토

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 9 커밋 후 사용자 수동 비교에서 다음 문제가 확인됐다.

- `쪽` 클립 표시가 아예 보이지 않는다.
- 점 격자의 점 개수가 한컴오피스 기준과 같은지 비교해야 한다.

## 판단

Stage 9는 격자 영역을 `drawMarginGuides()`의 본문 클립 기준으로 좁혔다. 그러나 한컴오피스의 `쪽` 격자 기준 화면은 본문 클립보다 넓은 쪽 테두리 영역에 점이 표시된다.

따라서 Stage 9의 본문 클립 기준 변경은 한컴 기준 점 개수와 맞지 않는다.

첨부 비교 화면은 `3mm` 격자 기준이다.

점 개수 기준:

- sample16 첫 쪽 `PageInfo`
  - page width: `793.7px`
  - page height: `1122.5px`
  - pageBorder: `18.9px` = 약 `5mm`
  - marginLeft/right: `56.7px` = 약 `15mm`
  - marginTop/bottom/header/footer: 각 `37.8px` = 약 `10mm`
- 3mm 격자에서 쪽 테두리 기준 영역은 약 `200mm x 287mm`
  - 예상 점 개수: 약 `67 x 96`
- 3mm 격자에서 본문 클립 기준 영역은 약 `180mm x 257mm`
  - 예상 점 개수: 약 `60 x 86`

본문 클립 기준을 쓰면 가로와 세로 점 개수가 한컴보다 적어진다.

## 수정 방향

- `쪽` 격자 기준 영역은 다시 `pageBorder*` 기준으로 되돌린다.
- 격자 overlay 위에서도 `쪽` 클립 코너가 보이도록 별도 corner overlay를 만든다.
- corner overlay는 `pageBorder*`와 같은 쪽 테두리 좌표를 사용한다.
- 자동 검증에서는 쪽 테두리 기준 점 개수와 본문 클립 기준 점 개수를 함께 기록한다.

## 수정 내용

- `rhwp-studio/src/view/grid-overlay.ts`
  - `getPageGridAreaPx()`를 다시 `pageBorder*` 기준으로 복구했다.
  - `createGridCornerOverlay()`를 추가해 `쪽` 기준일 때 네 모서리 L 표시를 별도 overlay로 그린다.
- `rhwp-studio/src/view/canvas-view.ts`
  - 격자 overlay 뒤에 corner overlay를 추가 렌더링한다.

## 검증 계획

- 로컬 Playwright 기능 검증
  - `samples/hwp3-sample16-hwp5.hwp` 로드
  - HWPX 비표준 감지 모달에서 `그대로 보기`
  - `쪽/3mm/0,0` 설정
  - overlay `clip-path`, `background-position`, `background-size`, `opacity` 기록
  - corner overlay 표시 여부와 좌표 기록
  - 쪽 테두리 기준 예상 점 개수 기록
- `npm run build`
- `cargo fmt --all -- --check && git diff --check`

## 검증 결과

- 로컬 Playwright 기능 검증 통과
  - 샘플: `samples/hwp3-sample16-hwp5.hwp`
  - 모달: `그대로 보기`
  - 설정: `쪽/3mm/0,0`
  - overlay 수: `2`
  - corner overlay 수: `2`
  - `background-size`: `12.5714px 12.5714px`
  - `background-position`: `20.955px 20.955px`
  - `clip-path`: `inset(21.955px)`
  - `opacity`: `1`
  - corner z-index: `3`
  - 쪽 테두리 기준 영역:
    - `199.99854166666668mm x 286.9935416666666mm`
    - 예상 점 개수: `67 x 96`
  - 본문 클립 기준 영역:
    - `179.99604166666663mm x 256.9897916666667mm`
    - 예상 점 개수: `60 x 86`
- `npm run build` 통과
  - 기존 큰 chunk 경고만 표시됨
- `cargo fmt --all -- --check && git diff --check` 통과

## 대기

자동 검증 후 커밋하고, 최종 정합 여부는 작업지시자의 수동 비교를 기다린다.
