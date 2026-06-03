# Task 1274 Stage 8

## 대상

- Stage7 전체 sweep 이후 남은 overflow:
  - `2024-09-between20`: 38줄
  - `2022-10`: 10줄
- 가장 작은 실제 내용 overflow:
  - `2022-10` 12쪽 왼쪽 단 `pi=627`, `pi=628`
  - `pi=627`: `문22）  82`
  - `pi=628`: `사차함수 가 에서만 극솟값을 갖는다고 하면`

## 관찰

- Stage7까지의 blank spacer 오탐 제거 뒤에도 `pi=627/628`은 실제 텍스트 문단으로 남아 있다.
- 렌더 로그 기준 왼쪽 단 하단은 `1092.3px`인데 `pi=627`의 첫 줄 하단은 `1124.2px`, `pi=628`은 `1142.3px`까지 내려간다.
- PDF 비교 PNG에서 문22 시작은 다음 단으로 넘어가야 하므로, 이번 문제는 draw 로그 오탐이 아니라 미주 흐름의 단 넘김 판단 문제다.

## 목적

- compact 미주 흐름에서 다음 문항 제목/머리 문단이 단 하단을 넘는 경우 같은 미주 묶음을 다음 단으로 넘기는 공통 조건을 확인한다.
- 문서/페이지/문항 번호 하드코딩 없이, endnote flow와 fit 판단 기준만 사용한다.
- 기존 문8/문12/문28 보정과 충돌하지 않게 `issue_1139_inline_picture_duplicate.rs` 단일 테스트로 회귀를 확인한다.

## 검증 계획

- 진행 중 자동 테스트는 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`만 사용한다.
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
- 필요 시 `2024-09-between20`과 전체 sweep으로 공통 효과와 페이지 수를 확인한다.
- 전체 CI급 테스트는 전체 목표 마지막에만 수행한다.

## 원인

- 문22 미주 묶음은 내부 vpos rewind를 가진다.
- 기존 새 미주 단 넘김 가드는 내부 vpos rewind가 있는 미주는 제외했다.
- 그 결과 `pi=627` 시점의 typeset `current_height`는 `980.7px`, 단 가용 높이는 `1001.6px`라서 숫자상으로는 제목 한 줄이 남는 것으로 판단했다.
- 그러나 렌더러의 HeightCursor vpos 정합 후 실제 줄 하단은 `1124.2px`로 단 하단 `1092.3px`을 넘었다.
- 즉 “내부 rewind 미주를 단 하단 근처에 남기면 렌더 vpos 보정으로 제목이 overflow될 수 있음”이 원인이다.

## 수정

- 새 미주 시작 문단이 내부 vpos rewind를 가지고 있어도, 단 하단 여유가 `ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX` 수준이면 기존 새 미주 단 넘김 가드에 다시 포함했다.
- 기존 예외는 유지했다.
  - 기본 7mm 문29/문30 late tail 허용은 그대로 둔다.
  - inline object 과대 추정 뒤의 짧은 제목 tail 허용도 그대로 둔다.
- 문서/페이지/문항 번호 하드코딩은 추가하지 않았다.

## 검증 결과

- `cargo fmt`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 통과
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
  - SVG/PDF/비교 PNG 18쪽 유지
  - `2022-10` overflow 10줄에서 6줄로 감소
  - `pi=627`, `pi=628` draw/item overflow 4줄 제거
  - `dump-pages` 기준 12쪽 왼쪽 단은 `pi=626`까지, 오른쪽 단은 `pi=627` 문22부터 시작한다.
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - SVG/PDF/비교 PNG 24쪽 유지
  - overflow 38줄 유지
- `python3 scripts/task1274_visual_sweep.py`

전체 sweep 결과:

- `2022-09`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2023-09`: SVG/PDF/비교 PNG 20쪽, overflow 없음
- `2024-09-below20`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2024-09-between20`: SVG/PDF/비교 PNG 24쪽, overflow 38줄
- `2022-10`: SVG/PDF/비교 PNG 18쪽, overflow 6줄
- `2022-11-practice`: SVG/PDF/비교 PNG 21쪽, overflow 없음

## 시각 확인

- `output/task1274/2022-10/compare/compare_012.png`
- `2022-10` 12쪽에서 문22가 PDF와 같이 오른쪽 단 상단에서 시작한다.
- 페이지 수는 18쪽으로 PDF와 1:1을 유지한다.

## 다음 후보

- `2022-10` 다음 남은 첫 재현은 15쪽 왼쪽 단 `pi=805/806`이다.
- `2024-09-between20`은 14쪽 전후의 `pi=714` 이후 overflow 묶음이 그대로 남아 있다.
