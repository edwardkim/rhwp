# Task 1284 Stage 15: 2023-09 16쪽 문27 직전 수식 line box 겹침 재검출

## 배경

- PR 준비 CI 중 작업지시자가 `3-09월_교육_통합_2023.hwp` 16쪽 하단에서 문26 tail 수식과 문27 제목이 아직 겹쳐 보인다고 제보했다.
- stage14 전체 sweep은 `2023-09`를 `flagged=0/20`으로 보고했으나, 실제 page16 metrics 내부에는 큰 column drift와 suppressed tail 후보가 남아 있었다.
- PR 준비 CI는 중단했다. 완료된 로컬 CI 단계는 `cargo fmt --all -- --check`, `cargo build --verbose`, `cargo check --target wasm32-unknown-unknown --lib`, `cargo test --features native-skia skia --lib --verbose`까지다.

## 관찰

- 대상: `3-09월_교육_통합_2023.hwp` 16쪽 왼쪽 단 하단.
- 최초 render tree:
  - 문23 제목 `pi=812`: y=173.6, PDF y=147.2
  - 문24 제목 `pi=814`: y=280.9, PDF y=254.7
  - 문25 제목 `pi=820`: y=537.8, PDF y=496.2
  - 문26 제목 `pi=823`: y=665.9, PDF y=624.5
  - 문26 tail `pi=830`: y=981.5, h=43.2
  - 문27 제목 `pi=831`: y=1001.1, h=12.0
- PDF bbox:
  - 직전 수식 visible glyph는 대략 y=965px 부근에서 끝난다.
  - 문27 제목은 y≈1000.3px에 시작한다.
- 즉 PDF의 실제 ink는 겹치지 않지만, rhwp는 문23부터 미주 제목이 누적 하강해 문26 tail line box가 문27 제목 영역까지 내려왔다.

## 추가 관찰

- 첫 수정에서 문26만 PDF y≈625px로 당기자, 아직 내려가 있던 문25 tail 수식과 문26 제목이 겹쳤다.
- `RHWP_VPOS_DEBUG` 기준 문23은 입력 `y_in=147.19`가 이미 PDF와 맞았지만, `compact_stale_note_gap`이 직전 line spacing 26.45px을 다시 더해 `result=173.64`로 밀고 있었다.
- 문23 상단 stale-forward에서는 저장 vpos를 버리되 추가 gap 없이 실제 `y_in`을 유지하고, vpos base만 이동해야 뒤 문항 흐름이 같이 올라간다.
- 최종 render tree:
  - 문23 제목 `pi=812`: y=147.2
  - 문24 제목 `pi=814`: y=254.5
  - 문25 제목 `pi=820`: y=511.4
  - 문25 tail bottom `pi=822`: y≈613.0
  - 문26 제목 `pi=823`: y=623.0
  - 문26 tail `pi=830`: y=938.6, h=43.2
  - 문27 제목 `pi=831`: y=998.2

## 문제점

- `question_title_text_overlap_candidates`가 직전 수식/큰 line box와 다음 문항 제목의 overlap을 flag하지 못했다.
- `render_tree_frame_tail_overflow_suppressed_candidates`는 `pi=845` 오른쪽 단 tail만 suppressed로 기록하고, 왼쪽 단 문27 직전 수식/title overlap은 별도 candidate로 만들지 못했다.
- stage14의 `flagged=0`은 이 시각 문제를 놓친 결과다.
- page16처럼 상단 stale-forward가 제목을 한 번 밀어 전체 문항 흐름을 누적 하강시키는 경우, 마지막 제목/수식 overlap만 보는 sweep은 원인을 좁히지 못한다.
- sweep의 line-order 후보는 text가 빈 tall visual line을 제외해 문25 tail 수식과 문26 제목이 겹치는 중간 상태를 놓쳤다.

## 진행 계획

1. page16 `pi=812` 상단 stale-forward를 추가 gap 없이 유지하도록 공통 height cursor 분기를 조정한다.
2. tall 수식 뒤 문항 제목 mid-backtrack은 직전 수식 bottom + 10px을 하한으로 둬 문25 tail/문26 제목 겹침을 방지한다.
3. 문26 tail 뒤 문27 제목은 hard backtrack 임계값 바로 위에 놓일 때 10px soft backtrack으로 하단 제목 위치를 맞춘다.
4. `issue_1139_inline_picture_duplicate.rs`에 2023-09 page16 문23~문27 좌표와 문25→문26, 문26→문27 overlap 회귀 테스트를 추가한다.
5. sweep line-order 분석에서 tall visual-empty line 뒤 문항 제목 overlap을 후보에 포함한다.
6. focused test와 `python3 scripts/task1274_visual_sweep.py --target 2023-09`로 재검증한다.

## 검증 대기

- 현재 focused 테스트:
  - `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2023_sep_page16_question27_title_matches_pdf_tail -- --nocapture`
  - `cargo test --lib compact_endnote_question_title_top_stale_forward_keeps_sequential_y -- --nocapture`
  - `cargo test --lib compact_endnote_question_title_after_tall_mid_backtrack_shifts_lazy_base -- --nocapture`
  - `cargo test --lib compact_endnote_question_title_tail_soft_backtrack_after_equation_tail -- --nocapture`
  - `cargo test --test issue_1139_inline_picture_duplicate`
  - `python3 -m py_compile scripts/task1274_visual_sweep.py`
  - `python3 scripts/task1274_visual_sweep.py --target 2023-09`
- 구현 후에는 PR 준비 CI를 처음부터 다시 이어서 수행한다.
