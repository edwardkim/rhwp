# task 1293 stage81 - 1310 rebase 후 미주 회귀 방지

## 목적

`upstream/devel`에 merge된 #1310을 기준으로 `local/task_m100_1293`를 rebase한 뒤
1293 미주 수정이 #1310 또는 기존 1261/1284 검증을 회귀시키지 않도록 원인을 좁혀 수정한다.

## rebase 상태

- #1310은 `upstream/devel`의 `f8d2ad39`까지 반영되어 있다.
- `local/task_m100_1293`는 `upstream/devel` 기준으로 rebase 완료했다.
- rebase 전 WIP는 stash 후 pop했으며, 실패한 중간 구현은 제거했다.

## 발견한 회귀

`HEAD`/WIP 비교는 판단 근거로 삼지 않고, 현재 렌더 결과와 sweep 산출물, PDF compare PNG를
기준으로 다시 확인했다. Focused test 기준으로 다음 계열이 실패하거나 시각 차이가 남았다.

- #1310 본문인 `issue_1256_2022_sep_page10_question12_keeps_between_notes_gap`는 통과했다.
- `issue_1261_2024_sep_page10_question8_stays_below_previous_equation`는 문8 제목과 직전 수식
  하단 간격이 부족했다.
- `issue_1284_2024_between20_page13_question_flow_matches_pdf`는 문18이 13쪽 우측 하단에
  오지 않아 page flow가 devel보다 크게 아래로 밀렸다.
- `issue_1284_2022_oct_page15_question28_formula_does_not_overlap_case_label`는 문28 제목이
  좌측 하단 tail에 남아 우측 컬럼 첫 줄로 넘어가야 할 풀이가 늦어졌다.
- `issue_1284_2024_between20_page22_23_question_tail_matches_pdf`와 current sweep compare에서
  `3-09월_교육_통합_2024-미주사이20.hwp` 22~23쪽 문28/문29 흐름이 PDF와 달랐다.
  22쪽 오른쪽 단 마지막 `a=-3/4` 수식 tail이 23쪽 왼쪽 상단으로 밀리거나, 반대로
  23쪽 문29 마지막 정사영 tail이 오른쪽 단 첫 줄로 넘어가지 않는 상태가 번갈아 나타났다.

## 원인 후보

1. `typeset.rs`에서 compact 구분선도 `sep_height`를 항상 pagination 높이에 더하고 있었다.
   `3-09월_교육_통합_2024-미주사이20.hwp`와 `3-10월_교육_통합_2022.hwp`는 UI 기준
   구분선 아래가 약 2mm인 compact 구분선이다. devel은 이 경우 구분선을 그리되 높이는
   소비하지 않으므로, 현재 branch의 page13/page15 하강은 이 차이와 맞다.
2. 수식 문단 가시는 #1310/devel처럼 `treat_as_char=true` 수식만 visible equation으로 본다.
   사용자가 제공한 `수식-문자처럼취급-아님.hwp` 기준으로도 모든 수식을 TAC로 간주하면 안 된다.
3. compact/default 문항 제목 tail 허용은 남은 회귀가 있으면 devel과 비교해서 더 좁힌다.

## 수정

- compact 구분선에서는 `PageItem::EndnoteSeparator`는 유지하되, `st.current_height`에는
  더하지 않도록 devel 동작을 복원한다.
- `para_has_visible_text_or_equation`는 TAC 수식만 visible equation으로 간주한다.
- 20mm `미주 사이` 문서에서 마지막 단 하단의 다줄 문단 split은 frame bottom bleed 허용치를
  반영해 마지막 줄 직전에서 쪼개도록 보정했다. 2024-09 미주사이20 18쪽 문25/문26,
  2022-10 15쪽 문28 수식 overlap 회귀를 막기 위한 공통 처리다.
- 20mm `미주 사이` 문서의 마지막 단에서 비가시 TAC 수식 tail이 실제로 들어가는 경우,
  다음 미주와의 boundary gap 때문에 tail을 다음 쪽으로 밀지 않는다. 2024-09 미주사이20
  22쪽 문28 마지막 `a=-3/4` 수식이 PDF처럼 오른쪽 단 하단에 남도록 했다.
- 같은 문서의 23쪽 문29에서는 짧은 visible text + 작은 TAC 수식 + vpos rewind로 끝나는
  마지막 tail 묶음을 오른쪽 단에서 시작하도록 보정했다. 이에 따라 문29의 마지막 정사영
  tail은 오른쪽 단 첫 줄로 넘어가고, 문30은 그 뒤에 이어진다.
- 현재 sweep 기준 16쪽 문23은 `문22` 마지막 빈 spacer 문단이 만든 한 줄 높이에
  20mm `미주 사이` trailing line spacing이 한 번 더 소비되어 PDF보다 약 80px 아래로
  밀렸다. 이 경우는 단 하단의 빈 spacer 뒤 새 문항 제목이고 저장 vpos가 stale-forward인
  케이스로 좁혀, 빈 줄 높이는 유지하되 주입된 trailing만 접도록 했다.

## 검증

- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture`
  - 5개 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1261_2024_sep_page10_question8_stays_below_previous_equation -- --nocapture`
  - 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1139_page17_endnote_question30_starts_on_right_column -- --nocapture`
  - 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2022_oct_page15_question28_formula_does_not_overlap_case_label -- --nocapture`
  - 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1256_2022_sep_page10_question12_keeps_between_notes_gap -- --nocapture`
  - 통과
- `cargo test --lib renderer::height_cursor::tests::compact_endnote_large_empty_spacer_collapses_trailing_gap_at_bottom -- --nocapture`
  - 통과
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20 --out output/task1293_stage81_between20_current --rhwp-bin target/debug/rhwp`
  - page 수 `SVG/PDF/render_tree = 24/24/24`
  - flagged page `7/24`에서 `6/24`로 감소
  - frame overflow, equation overlap, title overlap 후보 없음
  - page23은 flagged 목록에서 제거됨
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20 --out output/task1293_stage81_between20_after_empty_spacer --rhwp-bin target/debug/rhwp`
  - page 수 `SVG/PDF/render_tree = 24/24/24`
  - flagged page `5/24`
  - page16은 flag 없음. `question_marker_drift_candidates`, `tail_transition_candidates`,
    `column_region_drift_candidates` 모두 빈 값이다.
  - page22/page23은 flag 없음. 문28/문29 tail 보정 회귀 없음.

## 시각 확인

- `output/task1293_stage81_between20_current/2024-09-between20/compare/compare_022.png`
  - 문28 마지막 `a=-3/4` 수식이 PDF처럼 page22 오른쪽 단 하단에 남는다.
- `output/task1293_stage81_between20_current/2024-09-between20/compare/compare_023.png`
  - 문29가 page23 왼쪽 상단에서 시작하고, 마지막 정사영 tail은 PDF처럼 오른쪽 단 상단에서
    시작한 뒤 문30으로 이어진다.
- `output/task1293_stage81_between20_after_empty_spacer/2024-09-between20/compare/compare_016.png`
  - 문23 제목이 PDF처럼 16쪽 하단 tail 위치에서 시작한다. 이전처럼 빈 spacer 뒤
    20mm `미주 사이`가 중복 소비되어 한 note 간격만큼 내려가는 현상은 사라졌다.
- `output/task1293_stage81_between20_after_empty_spacer/2024-09-between20/compare/compare_022.png`
  - 문28 마지막 수식 tail은 여전히 PDF처럼 page22 오른쪽 단 하단에 남는다.
- `output/task1293_stage81_between20_after_empty_spacer/2024-09-between20/compare/compare_023.png`
  - 문29/문30 흐름은 기존 보정 상태를 유지한다.

## 남은 current sweep 후보

아래 후보는 이전 커밋과 비교하지 않고, 현재 sweep/PNG 기준으로 다음 스테이지에서 별도
확인한다.

- page11: `line_order_overlap`, `line_band_drift`, `column_line_band_drift`,
  `large_ink_region_drift`
- page13: 문18 tail `render_tree_frame_tail_overflow`
- page14: 문20 `question_marker_drift` (`y_delta_px=-50.9`) 및 red marker drift
- page15: `content_bottom_drift`
- page17: 문26 tail `render_tree_frame_tail_overflow`

## PR CI 주의

이 stage의 focused test와 target sweep은 계속 수행하되, PR 직전 전체 CI 검증은 작업지시자
별도 승인 전에는 실행하지 않는다. `/Goal` 자동 승인 지시는 PR CI 승인으로 간주하지 않는다.
