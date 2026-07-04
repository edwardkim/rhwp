# task 1293 stage84 - 0/0/0 미주 page10~14 흐름 보정

## 목적

stage83 이후 현재 sweep과 compare PNG 기준으로만
`3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`의 page10~14에 남은
시각 차이를 분석한다. WIP/HEAD 비교는 사용하지 않고, 현재 sweep과 한컴/PDF 시각 판단만
기준으로 삼는다.

## 대상

- HWP: `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
- PDF: `pdf/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.pdf`
- 최초 비교 sweep: `output/task1293_stage83_after_tail_split_sweep`
- 최종 검증 sweep: `output/task1293_stage84_after_zero_title_boundary_sweep`
- target: `2024-11-practice-above0-between0-below0`

## 최초 sweep 후보

- 페이지 수: SVG/render-tree/PDF `21/21/21`
- flagged: `5/21`
- 후보 페이지:
  - page10: `render_tree_frame_tail_overflow`, `question_marker_drift`, `line_band_drift`,
    `column_line_band_drift`, `large_ink_region_drift`
  - page11: `question_marker_drift`, `line_band_drift`, `large_ink_region_drift`
  - page12: `question_marker_drift`, `line_band_drift`, `column_line_band_drift`,
    `large_ink_region_drift`
  - page13: `question_marker_drift`, `line_band_drift`, `column_line_band_drift`,
    `large_ink_region_drift`
  - page14: `render_tree_frame_tail_overflow`, `question_marker_drift`, `line_band_drift`,
    `column_line_band_drift`, `large_ink_region_drift`

## 원인 판단

- page10은 문1부터 약 8px 아래에서 시작하고, 문6까지 약 40px까지 누적 drift가 생겼다.
- `dump-pages`와 `VPOS_CORR` 로그 기준 미주 문단 `line_seg.gap=452HU`가 반복되며,
  96dpi 환산 약 6px이다.
- `between_notes_marker_gap`의 RHWP/PDF 차이도 대부분 5.5~7.5px로 같은 크기다.
- 즉 0/0/0 미주에서 새 문항 제목 경계가 직전 문단 trailing `line_spacing` 1개를 계속
  간격으로 소비해 문항별 y가 누적 drift되는 문제로 판단했다.
- WIP/HEAD 비교는 판단 근거로 사용하지 않았다.

## 수정 내용

- `src/renderer/height_cursor.rs`
  - 0/0/0 미주에서 `lazy_base` 역산이 무효가 되는 페이지/단 시작부 새 문항 제목은
    순차 y에서 직전 trailing `line_spacing` 1개를 접는다.
  - 정상 vpos 경로에서도 새 문항 제목 경계가 raw vpos와 같은 위치로 들어오면,
    직전 콘텐츠 하단을 침범하지 않는 범위에서 같은 trailing gap을 접고 vpos base를 같이 이동한다.
  - page14 문27~문29처럼 앞 단 tail 뒤 title forward gap이 남는 경우도 0/0/0 미주 전용으로
    순차 흐름 기준을 유지한다.
- `src/renderer/typeset.rs`, `src/renderer/equation_tac_flow.rs`
  - 같은 `char_start`에 여러 TAC 수식이 있고 저장 `LINE_SEG`도 같은 수만큼 있으면 선행 빈 guide 줄을
    물리 수식 줄로 보존한다.
  - 0/0/0 미주 하단에서 텍스트/수식 tail이 다음 단 rewind 직전에 들어갈 수 있는 narrow fit 조건을 추가했다.
- `src/renderer/layout/paragraph_layout.rs`
  - 0/0/0 미주 boundary의 작은 equation-only 결과식 tail은 물리 흐름은 유지하되 표시 y만 한 줄 위로 붙여
    다음 문항 제목과 순서가 뒤집혀 보이지 않게 했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - page10 문7 제목/도입/수식 tail이 왼쪽 단 하단에 남고, 문11 제목/본문이 오른쪽 단 하단에 남는
    회귀 테스트를 추가했다.

## 최종 sweep/시각 확인

- sweep: `output/task1293_stage84_after_zero_title_boundary_sweep`
- 페이지 수: SVG/render-tree/PDF `21/21/21`
- flagged: `0/21`
- page10 compare:
  - RHWP 문1~문7 왼쪽 단, 문8~문11 오른쪽 단 흐름이 PDF와 맞는다.
  - 문7 tail과 문11 tail이 frame 밖 겹침 없이 보존된다.
- page14 compare:
  - 문23~문26 왼쪽 단, 문27~문29 오른쪽 단 흐름이 PDF와 맞는다.
  - 문29 하단 tail 후보가 사라졌다.

## 검증

- `cargo fmt`
- `cargo build`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage84_after_zero_title_boundary_sweep --rhwp-bin target/debug/rhwp`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1293_2024_zero_endnote_spacing_page10_question7_intro_stays_left_tail`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1256_2022_sep_page10_question12_keeps_between_notes_gap`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1274_2022_oct_page11_question20_equation_tail_stays_in_frame`
- `cargo test --test issue_1139_inline_picture_duplicate`

전체 CI 테스트는 PR 전 사용자 승인이 필요하므로 이번 stage에서는 수행하지 않았다.
