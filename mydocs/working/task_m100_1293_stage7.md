# Task 1293 Stage 7: 2024-11 미주 모양 샘플 페이지 수 정합

## 목적

Stage6에서 `3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7` HWP/HWPX/PDF 샘플을
검증 세트에 추가했다. 이 샘플은 미주 모양 UI 의미값이 `구분선 위 9mm / 미주 사이 8mm /
구분선 아래 7mm`로 정규화되지만, rhwp 렌더는 22쪽이고 한컴 PDF는 21쪽이다.

이번 스테이지에서는 이 페이지 수 불일치가 특정 문항 overflow인지, 미주 전체 flow 누적 오차인지,
또는 TAC/수식 문단 높이 과대 산정인지 분리한다.

## 초기 관찰

- HWP `dump-note-shape`
  - raw `separatorMarginTop=0mm`
  - raw `separatorMarginBottom=8.999mm`
  - raw `noteSpacing=6.999mm`
  - raw `rawUnknown=7.997mm`
- HWPX `dump-note-shape`
  - raw `separatorMarginTop=8.999mm`
  - raw `separatorMarginBottom=0mm`
  - raw `noteSpacing=6.999mm`
  - raw `rawUnknown=7.997mm`
- 양쪽 UI 의미값은 모두 `위 8.999mm / 아래 6.999mm / 사이 7.997mm`로 같다.
- `task1274_visual_sweep.py --target 2024-11-practice-shape987`
  - SVG/render tree 22쪽
  - PDF 21쪽
  - `flagged=20/21`
  - red marker drift: 10~21쪽
  - line band drift: 9~21쪽

## 분석 계획

1. 8~10쪽의 dump와 compare PNG를 기준으로 최초 drift 시작점을 찾는다.
2. HWP와 HWPX의 페이지 분배가 같은지 확인해 raw 슬롯 차이가 아니라 정규화 이후 flow 문제인지 확정한다.
3. 21쪽 PDF 마지막 내용과 rhwp 21/22쪽 내용을 대조해 어느 paragraph가 한 페이지 늦어졌는지 찾는다.
4. `render_tree` bbox에서 equation/text overlap 후보가 실제 늦어짐의 원인인지, 결과인지 분리한다.
5. 수치 보정이 아니라 공식 미주 모양 흐름 규칙으로 설명 가능한 공통 조건만 구현한다.

## 검증 대기

- `cargo fmt --all -- --check`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-shape987 --out output/task1293_stage7_2024_11_shape987 --rhwp-bin target/debug/rhwp`
- 필요 시 `2024-09-between20`, `2024-09-below20-above20` 회귀 sweep 재확인

## 추가 샘플 반영

작업지시자가 2024-11 실전 통합 기준으로 미주 모양 분리 검증 샘플 7종을 추가했다. 모두
같은 basename의 `.hwp`, `.hwpx`, `.pdf`가 존재한다.

| target | 한컴 UI 의미값 | PDF 쪽수 | rhwp 쪽수 | 판정 |
|---|---:|---:|---:|---|
| `2024-11-practice-above0-between0-below0` | 위 0 / 사이 0 / 아래 0 | 21 | 21 | 쪽수 일치 |
| `2024-11-practice-above0-between7-below2` | 위 0 / 사이 7 / 아래 2 | 21 | 21 | 쪽수 일치 |
| `2024-11-practice-above0-between7-below20` | 위 0 / 사이 7 / 아래 20 | 21 | 21 | 쪽수 일치 |
| `2024-11-practice-above20-between7-below2` | 위 20 / 사이 7 / 아래 2 | 21 | 21 | 쪽수 일치 |
| `2024-11-practice-shape987` | 위 9 / 사이 8 / 아래 7 | 21 | 22 | 남은 mismatch |
| `2024-11-practice-above0-between20-below2` | 위 0 / 사이 20 / 아래 2 | 22 | 21 | 남은 mismatch |
| `2024-11-practice-above20-between0-below20` | 위 20 / 사이 0 / 아래 20 | 21 | 20 | 남은 mismatch |
| `2024-11-practice-no-separator-above20-between20-below20` | 구분선 없음 / 위 20 / 사이 20 / 아래 20 | 23 | 21 | 남은 mismatch |

이 표에서 확인되는 점:

- `구분선 위`, `구분선 아래`, `미주 사이`는 HWP/HWPX 모두 공식 UI 의미값으로 정규화된다.
- `구분선 없음` 샘플은 raw `separatorLineType=0`, `separatorLineWidth=0`, `separatorLength=0`이지만
  한컴 PDF는 위/아래/사이 20mm 설정의 영향을 받는다.
- 따라서 renderer는 선을 그리지 않는 조건과 위/아래 여백을 소비하는 조건을 분리해야 한다.
- 20mm `미주 사이`가 포함된 2024-11 샘플들은 기존 2024-09 `미주사이20`보다 여전히 짧게
  pagination된다. 이 문제는 Stage8에서 vpos 누적/미주 사이 소비 위치를 분리해 분석한다.

## Stage7 구현

1. `scripts/task1274_visual_sweep.py`
   - 신규 2024-11 미주 모양 샘플 7종을 target으로 추가했다.
   - target key는 `above20-between7-below2`, `no-separator-above20-between20-below20`처럼
     설정값이 그대로 보이게 했다.
2. `src/renderer/typeset.rs`
   - `endnote_separator_height_px()`에서 구분선 선분이 없어도 `구분선 위`와 `구분선 아래`
     여백은 pagination 높이에 포함하도록 했다.
   - 선분 높이는 `separatorLineType`, `separatorLineWidth`, `separatorLength` 중 하나라도
     실제 값이 있을 때만 더한다.
3. `src/renderer/layout.rs`
   - `EndnoteSeparator` item을 layout할 때 선분이 없는 경우 `Line` render node를 만들지 않는다.
   - 그러나 `margin_above`와 `margin_below`는 그대로 y 흐름에 반영한다.

## Stage7 검증

- `cargo fmt --all -- --check` — 통과
- `python3 -m py_compile scripts/task1274_visual_sweep.py` — 통과
- `cargo build --bin rhwp` — 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` — 52 passed
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-shape987 --out output/task1293_stage7_validation --rhwp-bin target/debug/rhwp`
  - SVG/render tree 22쪽, PDF 21쪽
  - `flagged=20/21`, frame `[11, 13, 17]`, red `10~21`, line `9~21`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage7_validation_between20 --rhwp-bin target/debug/rhwp`
  - SVG/render tree 21쪽, PDF 22쪽
  - `flagged=20/21`, frame `[11, 12, 15, 18, 19]`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage7_validation_no_separator --rhwp-bin target/debug/rhwp`
  - SVG/render tree 21쪽, PDF 23쪽
  - `compare_010.png` 기준 rhwp에는 초록 구분선이 그려지지 않으며, 여백은 적용된다.
  - 전체 미주 흐름은 아직 PDF보다 앞서므로 Stage8에서 계속 분석한다.

## Stage7 결론

새 샘플 7종을 검증 세트에 편입했고, 공식 미주 모양 중 `구분선 없음`의 선분 표시와 여백
소비를 분리했다. 다만 새 샘플들은 `미주 사이 20mm`, 후반 TAC 그림/수식 vpos 누적, 문단 간
되감기 해석에서 여전히 PDF와 다른 페이지 분배를 드러낸다. 다음 스테이지는 이 잔여 mismatch를
쪽수/시각 기준으로 하나씩 줄이는 단계로 진행한다.
