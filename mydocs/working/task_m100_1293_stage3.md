# Task 1293 Stage 3: 미주 모양 샘플 계측과 sweep 메타데이터

## 목적

Stage2에서 공식 의미 접근자를 추가했으므로, 이번 단계에서는 샘플별 미주 모양 값과
렌더 차이를 같은 산출물에서 확인할 수 있게 한다.

## 확인 대상

- `samples/3-09월_교육_통합_2024-구분선아래20.hwp`
- `samples/3-09월_교육_통합_2024-미주사이20.hwp`
- `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`

`구분선아래20구분선위20` 샘플은 #1292 head(`jangster77:task_m100_1284`)에 이미
포함되어 있었으므로 Stage3에서 HWP/HWPX/PDF 세 파일만 현재 브랜치로 가져와 계측 대상에
포함한다. #1292의 레이아웃 코드 변경은 가져오지 않는다.

## 작업 계획

1. `export-render-tree` 또는 별도 CLI 출력에 미주 모양 정규화 값을 포함할 수 있는지 확인한다.
2. `scripts/task1274_visual_sweep.py`의 target summary에 미주 모양 메타데이터를 추가한다.
3. summary/metrics에서 `구분선 위`, `구분선 아래`, `미주 사이` 값을 mm 단위로 확인할 수 있게 한다.
4. `2024-09-below20`, `2024-09-between20`, `2024-09-below20-above20` sweep을 재실행해
   페이지 수와 flags 변화를 확인한다.

## 구현 전 판단

이번 단계의 목표는 layout 보정이 아니라 계측이다. 공식 미주 모양 값이 sweep 산출물에
나오지 않으면 이후 시각 불일치가 설정 문제인지 LINE_SEG 재구성 문제인지 분리하기 어렵다.

## 구현 내용

1. #1292 head에서 다음 샘플만 현재 브랜치로 가져왔다.
   - `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`
   - `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx`
   - `pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf`
2. `rhwp dump-note-shape <파일>` CLI를 추가했다.
   - 구역별 `footnoteShape`, `endnoteShape`의 raw 슬롯과 공식 UI 의미값을 JSON으로 출력한다.
   - `raw.noteSpacing`은 한컴 UI `구분선 아래`, `raw.rawUnknown`은 `미주 사이`로 노출한다.
3. HWP5 조합 샘플에서 `구분선 위 20mm`가 `separator_margin_bottom` raw 슬롯에 들어오는
   것을 확인했다. HWPX 조합 샘플은 같은 값이 `aboveLine` → `separator_margin_top`으로 들어온다.
4. `FootnoteShape::separator_above_margin_hu()`가 HWPX `separator_margin_top`을 우선 사용하고,
   없으면 HWP5 `separator_margin_bottom`을 공식 `구분선 위` fallback으로 쓰도록 정규화했다.
5. `scripts/task1274_visual_sweep.py`에 다음을 추가했다.
   - target `2024-09-below20-above20`
   - 각 target의 `analysis/note_shape.json`
   - `summary.json`의 compact `note_shape` 값

## 계측 결과

| target | 페이지 수(SVG/render tree/PDF) | 미주 구분선 위 | 미주 구분선 아래 | 미주 사이 |
|---|---:|---:|---:|---:|
| `2024-09-below20` | 23/23/23 | 0.0mm | 19.999mm | 6.999mm |
| `2024-09-between20` | 24/24/24 | 0.0mm | 2.032mm | 19.999mm |
| `2024-09-below20-above20` | 23/23/23 | 19.999mm | 19.999mm | 6.999mm |

`2024-09-below20-above20`의 HWP/HWPX는 raw 슬롯은 다르지만 공식 UI 값은 같다.

- HWP: `separatorMarginTop=0.0mm`, `separatorMarginBottom=19.999mm`
- HWPX: `separatorMarginTop=19.999mm`, `separatorMarginBottom=0.0mm`
- 정규화 후 둘 다 `separatorAbove=19.999mm`

## 검증

- `cargo fmt --all -- --check`
- `python3 -m py_compile scripts/task1274_visual_sweep.py`
- `cargo build --bin rhwp`
- `target/debug/rhwp dump-note-shape samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`
- `target/debug/rhwp dump-note-shape samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_spacing_reference_files_match_hancom_page_counts -- --nocapture`
- `cargo test --test issue_1050_footnote_serialize -- --nocapture` — 7 passed
- `cargo test --lib parse_endnote -- --nocapture` — 3 passed
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` — 51 passed
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-below20-above20 --out output/task1293_stage3 --rhwp-bin target/debug/rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-below20 --out output/task1293_stage3_below20 --rhwp-bin target/debug/rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20 --out output/task1293_stage3_between20 --rhwp-bin target/debug/rhwp`

## 남은 판단

현재 devel 기준 sweep은 `2024-09-below20-above20`에서 visual flags가 남는다. 이 단계에서는
공식 미주 모양 값이 정확히 계측되는지 확인했고, 남은 frame/red/line 후보는 다음 레이아웃
단계에서 공식 미주 모양 모델을 기준으로 분석한다.
