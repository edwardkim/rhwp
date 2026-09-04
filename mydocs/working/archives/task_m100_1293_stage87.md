# task 1293 stage87 - 전체 sweep PR 가능 수준 보정

## 목적

`output/task1293_rebase_full_sweep/summary.json` 기준으로 남은 전체 sweep 실패를 공통 원인으로
분류하고, PR 가능한 수준까지 미주 흐름을 보정한다. 개별 페이지별 수치 보정 대신 한컴 미주 모양
설정(`구분선 위`, `미주 사이`, `구분선 아래`, 구분선 표시 여부)에 맞는 공통 분기를 적용한다.

## 기준

- 브랜치: `local/task_m100_1293`
- 기준: `upstream/devel` (`f19c6a06`)
- 현재 작업 트리: clean
- 전체 sweep: `output/task1293_rebase_full_sweep/summary.json`

## 전체 sweep 관찰

- 기존 기본 샘플 중 `2022-10`은 `flagged=0/18`이다.
- `2024-11-practice-shape987`은 SVG/render tree 22쪽, PDF 21쪽으로 페이지 수부터 다르다.
- `2024-11-practice-no-separator-above20-between20-below20`은 SVG/render tree 22쪽, PDF 23쪽으로
  stage86 targeted sweep의 `23/23/23`, `flagged=0/23` 결과가 전체 sweep 기준에서 회귀했다.
- `above0-between20-below2`, `above20-between0-below20`, `above20-between7-below2`는 중후반 페이지에
  문항 marker drift, tail overflow, 수식/제목 overlap 후보가 반복된다.

## 우선 가설

1. visible separator가 있는 큰 `구분선 위` 샘플에서 제목/본문 경계 rewind를 단순히 현재 단
   80% 이상 조건으로 넘기면, `미주 사이`/`구분선 아래` 조합별로 과도한 단 이동 또는 부족한
   단 이동이 번갈아 생긴다.
2. no-separator 20/20/20은 targeted sweep과 full sweep의 page count가 달라졌다. full sweep 대상의
   입력/출력 파일 또는 stage86 이후 분기 영향이 다른 matrix 샘플에 의해 드러난 것으로 보고
   render tree page count와 핵심 boundary tail을 재확인한다.
3. page 수 불일치가 있는 샘플은 PR blocker다. 먼저 page count가 다른 두 샘플을 고치고,
   그 다음 flagged page 수가 큰 visible separator matrix를 줄인다.

## 검증 계획

- `cargo build --bin rhwp`
- focused test:
  `cargo test --test issue_1139_inline_picture_duplicate issue_1293_ -- --nocapture`
- targeted sweep:
  - `2024-11-practice-shape987`
  - `2024-11-practice-no-separator-above20-between20-below20`
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above20-between7-below2`
- 마지막에 전체 sweep 재실행 후 PR 가능 여부 재판정.
