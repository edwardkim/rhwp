# Task 1274 Stage 7

## 대상

- Stage6 전체 sweep 이후 남은 overflow:
  - `2024-09-between20`: 39줄
  - `2022-10`: 12줄
- 가장 작은 단일 재현:
  - `2022-10` 11쪽 오른쪽 단 `pi=588`
  - 로그:
    - `LAYOUT_OVERFLOW_DRAW: section=0 pi=588 line=0 y=1116.7 col_bottom=1092.3 overflow=24.5px`
    - `LAYOUT_OVERFLOW: page=10, sec=0, col=1, para=588, type=FullParagraph, first=false, y=1116.7, bottom=1092.3, overflow=24.5px`

## 관찰

- `dump-pages` 기준 `pi=588`은 미주 가상 문단이며 텍스트가 없는 빈 문단이다.
- Stage6에서 본문 빈 spacer 문단의 하단 오탐은 제거했지만, 미주 가상 문단은 원본 `Paragraph` 조건이 달라 같은 공통 helper에 걸리지 않았다.
- `2022-10`의 다음 페이지는 `pi=589`, `pi=590` 빈 미주 문단으로 시작한다.

## 목적

- 미주 가상 문단의 빈 spacer 줄도 본문 빈 spacer 줄과 같은 공통 판단으로 처리한다.
- 실제 컨트롤/TAC/수식/그림이 있는 줄은 제외한다.
- 문서/페이지/문항 하드코딩 없이 공통 레이아웃 조건만 사용한다.

## 검증 계획

- 진행 중 자동 테스트는 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`만 사용한다.
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
- 필요 시 전체 sweep으로 페이지 수와 overflow 수를 다시 확인한다.
- 전체 CI급 테스트는 전체 목표 마지막에만 수행한다.

## 원인

- Stage6 helper는 빈 줄, TAC 없음, 원본 문단 control 없음 조건을 모두 요구했다.
- 미주 가상 문단은 본문과 달리 endnote paragraph copy로 렌더링되며, 빈 미주 줄이라도 원본 control 조건만으로 실제 줄의 가시성을 판정하기 어렵다.
- `pi=588`은 줄 안에 표시할 텍스트나 TAC가 없는 미주 spacer 줄이지만, helper의 본문 문단 전용 control guard 때문에 draw/item overflow 집계에 남았다.

## 수정

- 빈 spacer 줄 판정에 `is_endnote_virtual_para` 인자를 추가했다.
- 줄 조건은 기존과 동일하게 유지했다.
  - line run이 모두 공백이다.
  - 해당 줄에 TAC offset이 없다.
- 본문 문단은 기존처럼 문단 control이 없을 때만 blank spacer로 본다.
- 미주 가상 문단은 줄 자체에 TAC가 없으면 문단 전체 control 유무와 무관하게 blank spacer로 처리한다.
- 이에 따라 실제 inline 수식/그림이 있는 줄은 계속 overflow 검출 대상이고, 미주 끝의 빈 spacer 줄만 제외된다.

## 검증 결과

- `cargo fmt`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 통과
  - 기존에 테스트 출력에 남던 `2022-10` `pi=588` overflow 로그가 사라졌다.
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
  - SVG/PDF/비교 PNG 18쪽 유지
  - `2022-10` overflow 12줄에서 10줄로 감소
  - `pi=588` draw/item overflow 2줄 제거
- `python3 scripts/task1274_visual_sweep.py`

전체 sweep 결과:

- `2022-09`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2023-09`: SVG/PDF/비교 PNG 20쪽, overflow 없음
- `2024-09-below20`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2024-09-between20`: SVG/PDF/비교 PNG 24쪽, overflow 38줄
- `2022-10`: SVG/PDF/비교 PNG 18쪽, overflow 10줄
- `2022-11-practice`: SVG/PDF/비교 PNG 21쪽, overflow 없음

Stage7에서 제거된 로그:

- `2022-10` `pi=588` draw/item overflow 2줄
- `2024-09-between20` `pi=804` item overflow 1줄

## 시각 확인

- `output/task1274/2022-10/compare/compare_011.png`
- `output/task1274/2022-10/compare/compare_012.png`
- 11쪽/12쪽의 문20 흐름은 PDF와 같은 페이지 수를 유지하며, 빈 spacer 제외로 실제 텍스트나 수식이 사라지지 않았다.

## 다음 후보

- `2022-10` 다음 남은 첫 재현은 12쪽 왼쪽 단 `pi=627`, `pi=628`이다.
- `2024-09-between20`은 14쪽 전후의 `pi=714` 이후 문단군 overflow가 가장 큰 묶음으로 남아 있다.
