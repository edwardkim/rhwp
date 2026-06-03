# Task #1129 Stage 13 - 격자 토글 복구와 쪽 클립/상하 위치 재분석

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

Stage 12 이후 사용자 수동 비교에서 다음 문제가 확인됐다.

- `격자 보기` 버튼 클릭이 동작하지 않는다.
- 쪽 클립 표시가 사라졌다.
- 한컴오피스와 비교할 때 페이지 외곽 테두리 기준으로 상단 여백이 너무 좁고 하단 여백이 너무 넓게 표시되는 것으로 보인다.

첨부 비교 순서:

1. rhwp-studio
2. 한컴오피스

## 분석 대상

- 격자 토글 명령이 실제로 실행되는지
- overlay가 생성되는데 보이지 않는지, 또는 생성 자체가 안 되는지
- Stage 12의 하단 clip 보정이 쪽 클립/격자 표시를 과하게 잘라냈는지
- canvas shadow 제거가 페이지 외곽 기준 인식을 바꾸었는지
- rhwp-studio의 page canvas 표시 크기/위치가 한컴오피스보다 상단에 치우쳐 보이는지

## 수정 방향

자동 재현 결과를 먼저 확인한 뒤 수정한다.

- 격자 토글이 실패하면 command/event/grid settings 경로를 복구한다.
- overlay가 있으나 안 보이면 z-index/clip/color/position을 복구한다.
- 쪽 클립은 한컴 기준처럼 페이지 테두리 안쪽에서 보이도록, 하단만 과도하게 자르는 Stage 12 보정은 재검토한다.
- 외곽선은 shadow 제거와 별개로 한컴처럼 얇게 유지한다.

## 재현 결과

- 툴바 `격자 보기` 버튼 클릭은 명령까지 정상 전달된다.
  - 클릭 후 overlay 수: `2`
  - active 상태: `true`
- Stage 12의 실제 문제는 버튼 이벤트가 아니라 표시 상태다.
  - 하단 clip이 `marginBottom` 기준으로 바뀌어 상/하 기준이 비대칭이었다.
  - 쪽 클립 코너 표시가 제거되어 한컴의 기준점이 사라져 보였다.

## 수정 내용

- `rhwp-studio/src/view/grid-overlay.ts`
  - `쪽` 하단 기준을 다시 `pageBorderBottom` 기준으로 복구했다.
  - 짧고 연한 `page-grid-clip-corners` overlay를 추가했다.
  - 점 색/크기 조정은 Stage 12 상태를 유지했다.
- `rhwp-studio/src/view/canvas-view.ts`
  - 격자 overlay와 함께 clip corner overlay를 렌더링한다.
- `rhwp-studio/src/styles/editor.css`
  - canvas 기본 z-index를 `0`으로 명시해 grid overlay가 안정적으로 위에 오도록 했다.

## 검증 계획

- 로컬 Playwright 기능 검증
  - `samples/hwp3-sample16-hwp5.hwp` 로드
  - HWPX 비표준 감지 모달에서 `그대로 보기`
  - 툴바 `격자 보기` 버튼 클릭
  - overlay 생성 수, active 상태, grid settings 상태 기록
  - `쪽/3mm/0,0` 설정 후 overlay CSS 기록
- `npm run build`
- `wasm-pack build --target web --out-dir pkg`
- `cargo fmt --all -- --check && git diff --check`

## 검증 결과

- 로컬 Playwright 기능 검증 통과
  - 샘플: `samples/hwp3-sample16-hwp5.hwp`
  - 모달: `그대로 보기`
  - 툴바 `격자 보기` 클릭:
    - overlay 수: `2`
    - clip corner overlay 수: `2`
    - active 상태: `true`
  - `쪽/3mm/0,0` 설정:
    - `background-image`: `radial-gradient(circle, rgba(0, 22, 135, 0.95) 0px, rgba(0, 22, 135, 0.95) 0.42px, rgba(0, 0, 0, 0) 0.6px)`
    - `background-size`: `12.5714px 12.5714px`
    - `background-position`: `20.955px 20.955px`
    - `clip-path`: `inset(21.955px)`
    - overlay `z-index`: `1`
    - canvas `z-index`: `0`
    - canvas `box-shadow`: `none`
    - clip corner `z-index`: `2`
  - 쪽 기준 px:
    - top: `20.955020788711096`
    - bottom: `20.955020788711096`
    - marginBottom: `41.91004157742219`
- `npm run build` 통과
  - 기존 큰 chunk 경고만 표시됨
- `wasm-pack build --target web --out-dir pkg` 통과
  - 추적 산출물 변경 없음
- `cargo fmt --all -- --check && git diff --check` 통과

## 대기

자동 검증 후 커밋하고, 최종 정합 여부는 작업지시자의 수동 비교를 기다린다.
