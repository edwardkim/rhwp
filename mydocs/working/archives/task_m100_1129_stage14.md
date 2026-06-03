# Task #1129 Stage 14 - 점 격자 가시성 복구와 페이지 외곽 상하 위치 확인

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 13 이후 사용자 수동 비교에서 다음 문제가 확인됐다.

- `격자 보기`를 눌러도 점 격자가 표시되지 않고 쪽 클립만 보인다.
- 한컴오피스와 비교할 때 페이지 외곽선 기준 상단 여백이 좁고 하단 여백이 넓게 보인다.

첨부 비교 순서:

1. rhwp-studio
2. 한컴오피스

## 분석

Stage 13 자동 검증에서 버튼 클릭과 overlay 생성은 확인됐다.

- overlay 수: `2`
- active 상태: `true`
- clip corner overlay 수: `2`

따라서 현재 문제는 버튼 이벤트 자체가 아니라 점 격자 가시성이다. Stage 12에서 점 반경을 `0.42px`로 줄였고, 실제 화면에서는 점이 거의 사라져 쪽 클립만 보이는 결과가 되었다.

페이지 외곽선 상/하 여백 문제는 grid overlay와 별개로 실제 렌더된 페이지 테두리 위치와 `PageInfo.pageBorderTop/Bottom` 기준을 비교해야 한다.

## 수정 방향

- 점 격자 반경을 다시 사람이 보이는 수준으로 복구한다.
- 점 색상은 한컴 기준에 맞게 선명하되 과하게 두껍지 않게 조정한다.
- `쪽` clip은 `pageBorder*` 기준 대칭을 유지한다.
- 페이지 외곽선 상/하 위치는 자동 검증에서 `PageInfo`와 overlay 기준 값을 기록하고, 별도 렌더 코어 수정이 필요한지 판단한다.

## 수정 내용

- `rhwp-studio/src/view/grid-overlay.ts`
  - 점 색상을 `rgba(0, 32, 150, 0.9)`로 조정했다.
  - 점 반경을 `0.42px`에서 `0.65px`로 복구했다.
  - transparent 경계를 `0.6px`에서 `0.85px`로 복구했다.

## 검증 계획

- 로컬 Playwright 기능 검증
  - `samples/hwp3-sample16-hwp5.hwp` 로드
  - HWPX 비표준 감지 모달에서 `그대로 보기`
  - 툴바 `격자 보기` 버튼 클릭
  - `쪽/3mm/0,0` 설정
  - overlay CSS와 clip corner overlay 기록
  - canvas/pageInfo/pageBorder 상하 기준 기록
- `npm run build`
- `wasm-pack build --target web --out-dir pkg`
- `cargo fmt --all -- --check && git diff --check`

## 검증 결과

- 로컬 Playwright 기능 검증 통과
  - 샘플: `samples/hwp3-sample16-hwp5.hwp`
  - 모달: `그대로 보기`
  - 설정: `쪽/3mm/0,0`
  - overlay 수: `2`
  - clip corner overlay 수: `2`
  - active 상태: `true`
  - `background-image`: `radial-gradient(circle, rgba(0, 32, 150, 0.9) 0px, rgba(0, 32, 150, 0.9) 0.65px, rgba(0, 0, 0, 0) 0.85px)`
  - `background-size`: `12.5714px 12.5714px`
  - `background-position`: `20.955px 20.955px`
  - `clip-path`: `inset(21.955px)`
  - page border top/bottom CSS px:
    - top: `20.955020788711096`
    - bottom: `20.955020788711096`
  - canvas:
    - `z-index`: `0`
    - `box-shadow`: `none`
- 판단
  - `PageInfo.pageBorderTop/Bottom` 기준은 자동 검증상 대칭이다.
  - 한컴 대비 페이지 외곽선 상/하 여백 차이는 grid overlay보다 실제 WASM 렌더된 페이지 테두리 위치 또는 화면 배율/스크롤 표시 기준을 별도 단계에서 비교해야 한다.
- `npm run build` 통과
  - 기존 큰 chunk 경고만 표시됨
- `wasm-pack build --target web --out-dir pkg` 통과
  - 추적 산출물 변경 없음
- `cargo fmt --all -- --check && git diff --check` 통과

## 대기

자동 검증 후 커밋하고, 최종 정합 여부는 작업지시자의 수동 비교를 기다린다.
