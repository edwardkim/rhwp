# Task #1129 Stage 11 - 쪽 테두리 중복 표시와 외곽선 두께 보정

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 10 커밋 후 사용자 수동 비교에서 다음 문제가 확인됐다.

- `쪽` 테두리 위치가 한컴오피스 기준과 다르게 보인다.
- rhwp-studio 쪽 외곽선이 한컴오피스 대비 너무 넓고 두껍게 보인다.

첨부 비교는 `3mm` 격자 기준이다.

## 판단

Stage 10은 점 개수 기준을 맞추기 위해 격자 영역을 `pageBorder*` 기준으로 복구했다. 이 방향은 유지한다.

다만 `page-grid-corners`를 별도로 추가하면서 canvas가 이미 렌더링하는 문서 테두리/코너와 중첩될 수 있다. 이 중복 표시가 외곽선을 더 두껍게 보이게 만든다.

또한 rhwp-studio의 page canvas CSS `box-shadow`가 한컴오피스 화면보다 넓은 외곽 음영을 만든다.

## 수정 방향

- 점 격자 기준 영역은 `pageBorder*` 기준을 유지한다.
- 별도 `page-grid-corners` overlay는 제거한다.
- 격자 overlay의 `clip-path`가 테두리 선을 덮지 않도록 `pageBorder* + 1px` clip은 유지한다.
- page canvas 외곽 shadow를 한컴오피스에 더 가깝게 얇게 줄인다.

## 수정 내용

- `rhwp-studio/src/view/grid-overlay.ts`
  - `createGridCornerOverlay()`와 corner line 생성 코드를 제거했다.
  - 격자 영역은 `pageBorder*` 기준으로 유지했다.
- `rhwp-studio/src/view/canvas-view.ts`
  - corner overlay 렌더링 호출을 제거했다.
- `rhwp-studio/src/styles/editor.css`
  - canvas shadow를 `0 2px 8px rgba(0, 0, 0, 0.15)`에서 `0 1px 2px rgba(0, 0, 0, 0.12)`로 줄였다.

## 검증 계획

- 로컬 Playwright 기능 검증
  - `samples/hwp3-sample16-hwp5.hwp` 로드
  - HWPX 비표준 감지 모달에서 `그대로 보기`
  - `쪽/3mm/0,0` 설정
  - overlay `clip-path`, `background-position`, `background-size`, `opacity` 기록
  - corner overlay가 생성되지 않는지 확인
  - canvas `box-shadow` 기록
- `npm run build`
- `cargo fmt --all -- --check && git diff --check`

## 검증 결과

- 로컬 Playwright 기능 검증 통과
  - 샘플: `samples/hwp3-sample16-hwp5.hwp`
  - 모달: `그대로 보기`
  - 설정: `쪽/3mm/0,0`
  - overlay 수: `2`
  - corner overlay 수: `0`
  - `background-size`: `12.5714px 12.5714px`
  - `background-position`: `20.955px 20.955px`
  - `clip-path`: `inset(21.955px)`
  - `opacity`: `1`
  - canvas `box-shadow`: `rgba(0, 0, 0, 0.12) 0px 1px 2px 0px`
  - 쪽 테두리 기준 예상 점 개수: `67 x 96`
- `npm run build` 통과
  - 기존 큰 chunk 경고만 표시됨
- `cargo fmt --all -- --check && git diff --check` 통과

## 대기

자동 검증 후 커밋하고, 최종 정합 여부는 작업지시자의 수동 비교를 기다린다.
