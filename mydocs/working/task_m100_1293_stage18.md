# Task 1293 Stage 18: 2024-09 p18 미주 흐름 차이 보정

## 목적

Stage17에서 sweep frame 오탐은 제거했지만 `2024-09-below20-above20` p18은
한컴/PDF와 rhwp의 문항 흐름이 다르게 남았다. 이 단계에서는 해당 페이지의
red/line/equation 후보를 실제 pagination 차이로 보고, 공통 미주 흐름 로직에서
원인을 찾아 수정한다.

## 기준 산출물

- `output/task1293_stage17_after_height_cursor_check/summary.json`
- `output/task1293_stage17_after_height_cursor_check/2024-09-below20-above20/compare/compare_018.png`
- `output/task1293_stage17_after_height_cursor_check/2024-09-below20-above20/analysis/metrics.json`
- `output/task1293_stage17_after_height_cursor_check/2024-09-below20-above20/render_tree/render_tree_018.json`

## 현재 증상

- SVG/PDF/render-tree 쪽수는 23/23/23으로 일치한다.
- `frame_overflow_pages=[]`, `question_title_text_overlap_pages=[]`, `line_order_overlap_pages=[]`.
- p18에는 `red_marker_drift`, `line_band_drift`, `equation_text_overlap`이 동시에 남는다.
- compare 기준:
  - rhwp p18 오른쪽 column은 문23~문26 흐름이 남아 있다.
  - PDF p18 오른쪽 column은 문25~문27 흐름으로 배치된다.
- metrics p18:
  - `red_marker_drift.max_abs_delta_px=168.5`
  - `line_band_drift.mean_abs_delta_px=77.8`
  - equation/text 후보: `text_pi=923`, text=`이 경우 구하는 확률은`, overlap ratio `0.356`

## 분석 계획

1. p17~p19 render-tree에서 문23~문27의 `pi`, column, y 범위를 추적한다.
2. p18에서 PDF보다 rhwp가 더 많은 이전 문항을 붙잡는지, 혹은 다음 문항 advance가 늦는지 구분한다.
3. 미주 구분선 위/아래/미주 사이 설정이 title/body/tail advance 판단에 어떻게 반영되는지 확인한다.
4. 특정 문항 번호가 아니라 compact/non-default 미주 흐름 공통 조건으로 수정한다.

## 수정 내용

- `large_separator_block` 판정이 구분선 위/아래가 모두 큰 설정을 후속 미주에도 과하게
  적용하던 흐름을 보정했다.
- 구분선이 있고 미주 사이가 기본값인 후속 미주에서는 큰 separator block 판정을 끄고,
  첫 미주이거나 미주 사이 자체가 비기본으로 큰 경우는 기존 판정을 유지한다.
- 구분선 위/아래가 모두 큰 설정은 미주 블록을 시작할 때는 큰 공간을 유지해야 하지만,
  같은 구역에 미주가 이어지는 중간 문항까지 같은 block 판정을 적용하면 문항 tail을
  지나치게 붙잡아 한컴보다 이전 문항이 같은 쪽에 오래 남았다.
- `2024-09-below20-above20` p18의 문23~문27 흐름은 이 과한 block 판정 때문에
  rhwp가 PDF보다 이전 문항을 더 많이 붙잡던 케이스였다.
- sweep의 equation/text detector가 윗줄 텍스트 bbox와 바로 아래 수식 bbox의 정상적인
  줄간 접촉을 실제 겹침으로 잡지 않도록 보정했다.
  - 같은 부모 아래 인접한 `TextLine`이고, 아래쪽 수식이 위쪽 텍스트 이후에 오며,
    겹침 높이가 8px 이하이면 정상 flow 접촉으로 제외한다.
  - 단/column이 다른 p13 후보처럼 부모가 다른 경우는 그대로 실제 후보로 남긴다.

## 검증 결과

- `cargo fmt --all -- --check`
  - 통과
- `cargo test --lib compact_endnote -- --nocapture`
  - 27 passed
- `cargo build --bin rhwp`
  - 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 52 passed
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-below20-above20 --out output/task1293_stage18_2024_09_below20_above20_check --rhwp-bin target/debug/rhwp`
  - SVG/PDF/render-tree: 23/23/23
  - `frame_overflow_pages=[]`
  - `question_title_text_overlap_pages=[]`
  - `line_order_overlap_pages=[]`
  - p18의 `red_marker_drift`는 `max_abs_delta_px=1.0`, `mean_abs_delta_px=0.5`로 감소
  - p18의 남은 `equation_text_overlap`은 `text_pi=923`, text=`이 경우 구하는 확률은`
    줄과 바로 아래 수식 bbox 접촉 오탐으로 시각 확인
- `python3 -m py_compile scripts/task1274_visual_sweep.py`
  - 통과
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-below20-above20 --out output/task1293_stage18_detector_p18_check --rhwp-bin target/debug/rhwp`
  - p18 정상 줄간 equation/text 접촉 후보 제거
  - `equation_text_overlap_pages=[]`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage18_detector_p13_check --rhwp-bin target/debug/rhwp`
  - p13 단 경계 equation/text 후보 유지
  - `equation_text_overlap_pages=[13]`
- `python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage18_all_detector_check --rhwp-bin target/debug/rhwp`
  - 15개 target 모두 SVG/PDF/render-tree 쪽수 1:1
  - 모든 target에서 `frame_overflow_pages=[]`, `question_title_text_overlap_pages=[]`,
    `line_order_overlap_pages=[]`
  - major 후보는 `2024-11-practice-above20-between0-below20` p13의
    `equation_text_overlap_pages=[13]`만 남음

## 다음 스테이지 후보

- `2024-11-practice-above20-between0-below20` p13은 왼쪽 단 하단의 긴 수식과 오른쪽 단
  시작 텍스트가 같은 y대에 잡히므로 실제 단 경계 흐름 후보로 본다.
- 나머지 equation 후보는 윗줄 텍스트 bbox와 아래 수식 bbox가 정상 행간에서 접촉하는
  detector 오탐 성격이 강하므로, Stage19에서 실제 단 경계 겹침과 일반 줄간 접촉을
  구분하도록 sweep 판정도 함께 보정한다.
