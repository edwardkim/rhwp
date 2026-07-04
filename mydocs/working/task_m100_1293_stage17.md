# Task 1293 Stage 17: refined sweep 잔여 후보 triage

## 목적

Stage16에서 `shape987` p19/p20의 하단 tail overflow와 sweep의 하단 glyph bleed 오탐은 정리했다.
이제 refined sweep 기준으로 여전히 남는 frame/equation/red/line 후보 중 실제 한컴/PDF와 다른
페이지를 선별하고, 공통 미주 흐름 로직으로 처리할 수 있는 후보를 이어서 수정한다.

## 현재 기준

- 직전 커밋: `510f8e4b task 1293: 미주 하단 tail sweep 판정 보정`
- 기준 산출물:
  - `output/task1293_stage16_all_post_commit_check/summary.json`
  - `output/task1293_stage17_candidate_recheck/summary.json`
  - `output/task1293_stage17_after_height_cursor_check/summary.json`
- 전체 sweep 14종 모두 SVG/PDF/render-tree 쪽수는 1:1이다.
- 모든 target에서 `question_title_text_overlap_pages=[]`, `line_order_overlap_pages=[]`이다.
- 하단 frame 후보는 3건, equation/text bbox 후보는 11개 target에 남았다.

## 남은 우선 후보

1. `2024-11-practice-above0-between7-below2` p12
   - `frame=[12]`
   - Stage16 이후 2024-11 신규 8종 중 유일한 frame 후보
2. `2022-11-practice` p12
   - `frame=[12]`
   - 같은 원본 흐름의 기본 미주 설정 기준 frame 후보
3. `2024-09-below20-above20` p9
   - `frame=[9]`
   - 2024-09 대표 회귀 중 남은 frame 후보
4. equation/text overlap 후보
   - `2022-09`: p17
   - `2024-09-below20`: p17
   - `2024-11-practice-above0-between20-below2`: p11, p12, p19, p22
   - `2024-11-practice-above20-between0-below20`: p11, p13, p21
   - `2024-11-practice-above20-between7-below2`: p18
   - `2024-11-practice-no-separator-above20-between20-below20`: p11, p12, p18, p21
   - `2024-09-between20`: p18
   - `2024-09-below20-above20`: p18
   - `2022-10`: p10

## 수정 내용

### frame 후보 오탐 보정

- `detect_frame`이 진한 검정 픽셀만 frame 후보로 잡으면 `rsvg-convert`의 0.5px 회색
  antialias frame 선을 놓칠 수 있었다.
- 이 경우 `2024-11-practice-above0-between7-below2` p12처럼 내부 표/구분선 row를
  하단 frame으로 오인해 실제 frame bottom보다 약 188px 위를 page frame으로 판정했다.
- frame 검출 전용 `is_frame_line_pixel`을 추가해 RGB 채널 차이가 작은 회색 선 픽셀까지
  frame 후보로 포함했다.
- 하단 5px 이내의 glyph bleed는 PDF의 bleed가 없더라도 본문 하단 위치가 같으면
  overflow 후보에서 제외하도록 했다. `2024-09-below20-above20` p9처럼 실제 하단
  흐름이 같은 페이지를 frame overflow로 잡는 오탐을 줄이기 위한 보정이다.

### compact 미주 rewind 침범 방지

- `HeightCursor::vpos_adjust`에서 compact 미주 vpos rewind가 직전 줄의 실제 콘텐츠 하단보다
  위로 되돌아가면 저장 vpos가 같은 단의 재배치 흔적으로 남은 것으로 보고 순차 y를 유지한다.
- 기존에 안전한 rewind는 그대로 존중하고, 직전 콘텐츠를 침범하는 rewind만 차단하도록
  단위 테스트를 분리했다.

## 분석 결과

### 해결된 후보

`output/task1293_stage17_after_height_cursor_check/summary.json` 기준:

| target | SVG/PDF/tree | frame | equation | title | order |
|---|---:|---:|---:|---:|---:|
| `2024-11-practice-above0-between7-below2` | 21/21/21 | `[]` | `[]` | `[]` | `[]` |
| `2022-11-practice` | 21/21/21 | `[]` | `[]` | `[]` | `[]` |
| `2024-09-below20-above20` | 23/23/23 | `[]` | `[18]` | `[]` | `[]` |

### 다음 스테이지로 분리할 실제 후보

- `2024-09-below20-above20` p18은 frame 오탐이 아니라 실제 흐름 차이가 남아 있다.
  - `compare_018.png`에서 rhwp는 p18 오른쪽 column에 문23~문26 흐름이 남고,
    PDF는 문25~문27 흐름으로 배치된다.
  - 새 binary 기준 `metrics.json` p18: `red_marker_drift` max `168.5px`,
    `line_band_drift` mean `77.8px`.
  - `equation_text_overlap` 후보는 `text_pi=923`, text=`이 경우 구하는 확률은`,
    overlap ratio `0.356`이다.
- 이 후보는 sweep 판정 문제가 아니라 미주 pagination/column flow 자체의 차이이므로
  Stage18에서 공통 미주 흐름 로직으로 수정한다.

## 검증 예정

- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between7-below2 --target 2022-11-practice --target 2024-09-below20-above20 --out output/task1293_stage17_candidate_recheck --rhwp-bin target/debug/rhwp`
- `cargo fmt --all -- --check`
- `cargo test compact_endnote_bottom_rewind --lib -- --nocapture`
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between7-below2 --target 2022-11-practice --target 2024-09-below20-above20 --out output/task1293_stage17_after_height_cursor_check --rhwp-bin target/debug/rhwp`
- `python3 -m py_compile scripts/task1274_visual_sweep.py`
