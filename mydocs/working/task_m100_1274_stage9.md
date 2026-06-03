# Task 1274 Stage 9

## 대상

- Stage8 전체 sweep 이후 남은 overflow:
  - `2024-09-between20`: 38줄
  - `2022-10`: 6줄
- `2022-10`의 다음 실제 내용 overflow:
  - 15쪽 왼쪽 단 `pi=805`, `pi=806`
  - `pi=805`: `문28）   ②`
  - `pi=806`: 문28 풀이 첫 문단 일부

## 관찰

- `dump-pages` 기준 15쪽 왼쪽 단에는 `pi=805` 문28 제목과 `pi=806` 첫 줄이 남고, 오른쪽 단은 `pi=806`의 나머지부터 시작한다.
- PDF 비교 PNG에서는 문28 제목이 오른쪽 단 상단에서 시작해야 한다.
- Stage8의 새 미주 단 넘김 가드는 내부 vpos rewind 미주 제목에 대한 보정이었고, 이번 케이스는 제목 직후 본문이 분할되는 문제다.

## 목적

- compact 미주 흐름에서 문항 제목 직후 첫 풀이 문단이 단 하단에서 쪼개지는 경우, 제목과 풀이 머리 부분을 다음 단으로 같이 넘기는 공통 조건을 찾는다.
- 문서/페이지/문항 번호 하드코딩 없이, endnote flow/문항 제목/다음 문단 fit 조건으로 처리한다.
- 기존 문8/문12/문28 보정과 충돌하지 않게 단일 허용 테스트로 회귀를 확인한다.

## 검증 계획

- 진행 중 자동 테스트는 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`만 사용한다.
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
- 필요 시 전체 sweep으로 페이지 수와 overflow 수를 다시 확인한다.
- 전체 CI급 테스트는 전체 목표 마지막에만 수행한다.

## 원인

- `pi=805`는 문28 미주 제목이고 `pi=806`은 바로 뒤 첫 풀이 문단이다.
- 기존 fit 판단은 새 미주의 제목 한 줄만 보면 왼쪽 단에 들어간다고 판단했다.
- 그러나 다음 `pi=806` 첫 줄은 남은 공간에 일부만 들어가고 곧바로 split되어, 한컴/PDF와 다르게 문항 제목과 풀이 머리가 단 하단에서 갈라졌다.
- 렌더 로그에서도 `pi=805` 제목 자체가 content bottom 기준 overflow로 남고, `pi=806` 첫 줄이 draw/item overflow를 만들었다.

## 수정

- 내부 vpos rewind가 있는 새 미주 제목에 대해, 다음 미주 문단의 첫 줄 advance를 함께 계산한다.
- 제목 advance와 다음 첫 줄 advance를 더했을 때 단 하단 여유가 `ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX` 수준이면 새 미주 시작부터 다음 단으로 넘긴다.
- Stage8의 단순 “제목이 하단에 너무 가까움” 조건은 유지하고, 이번에는 “제목과 첫 풀이 줄이 같이 들어가지 않음” 조건을 추가했다.
- 문서/페이지/문항 번호 하드코딩은 추가하지 않았다.

## 검증 결과

- `cargo fmt`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 통과
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
  - SVG/PDF/비교 PNG 18쪽 유지
  - `2022-10` overflow 6줄에서 3줄로 감소
  - `pi=805`, `pi=806` overflow 3줄 제거
  - `dump-pages` 기준 15쪽 왼쪽 단은 `pi=804`까지, 오른쪽 단은 `pi=805` 문28부터 시작한다.
- `python3 scripts/task1274_visual_sweep.py`

전체 sweep 결과:

- `2022-09`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2023-09`: SVG/PDF/비교 PNG 20쪽, overflow 없음
- `2024-09-below20`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2024-09-between20`: SVG/PDF/비교 PNG 24쪽, overflow 38줄
- `2022-10`: SVG/PDF/비교 PNG 18쪽, overflow 3줄
- `2022-11-practice`: SVG/PDF/비교 PNG 21쪽, overflow 없음

## 시각 확인

- `output/task1274/2022-10/compare/compare_015.png`
- `2022-10` 15쪽에서 문28이 PDF와 같이 오른쪽 단 상단에서 시작한다.
- 페이지 수는 18쪽으로 PDF와 1:1을 유지한다.

## 다음 후보

- `2022-10` 다음 남은 재현은 16쪽 왼쪽 단 `pi=841/842`이다.
- `2024-09-between20`은 14쪽 전후의 `pi=714` 이후 overflow 묶음이 계속 남아 있다.
