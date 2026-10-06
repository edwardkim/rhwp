# Task 1293 Stage 72: boundary TAC 이후 남은 미주 흐름 drift 분리

## 목적

Stage71에서 같은 `line_seg` 경계의 TAC 수식이 이전 줄 끝과 다음 줄 시작에 중복 emit되는
문제를 제거했다. 자동 sweep 결과 `equation_text_overlap_pages`와 `frame_overflow_pages`는
비었지만, `question_marker_flow_drift`, `line_band_drift`, `large_ink_region_drift`가 남아 있다.

이번 단계는 남은 drift를 다음 축으로 분리한다.

- 문단 내부 `lineSegArray -> Paragraph.line_segs -> ComposedLine` 흐름이 한컴과 다른지
- 미주 모양 값(구분선 위/아래/미주 사이)이 페이지네이션에서 잘못 적용되는지
- 미주 위치(문서의 끝/구역의 끝)와 단 나눔/rewind 로직이 한컴과 다른지

## 대상

- `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
  - `output/task1293_stage71_boundary_tac/.../compare_012.png`
  - 12쪽: RHWP는 문16~문20 흐름이 PDF보다 위/아래 재배치되어 있음.
- `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp`
  - `output/task1293_stage71_boundary_tac/.../compare_011.png`
  - 11쪽: 문9~문13 흐름이 PDF와 다름.

## 계획

1. `summary.json`/`metrics.json`에서 Stage71 이후 남은 주요 후보를 page 단위로 정리한다.
2. `dump-pages`와 `dump-endnote-lines`를 같이 사용해 page item source와 원본 미주 문단의
   `line_seg` 누적 위치를 대조한다.
3. 수식 중복이 아닌 흐름 drift라면, 미주 pagination/height cursor 쪽에서 한컴의
   구분선 위/아래/미주 사이 적용 지점이 잘못된 곳을 찾는다.
4. 문서명/문항 번호 조건 없이, 미주 shape 값과 line_seg 구조 조건으로만 보정한다.

## 중간 판단

- 문단 단위가 아니라 `lineSegArray -> Paragraph.line_segs -> ComposedLine` 순서로 좁혀
  봐야 한다. 특히 미주 풀이에는 텍스트가 `\n\n\n`뿐인데 글자처럼 취급하는 수식 TAC가
  textRun 위치를 차지해 여러 `line_seg`에 배치되는 문단이 있다.
- 렌더러(`paragraph_layout.rs`)는 이미 같은 `char_start`의 수식 TAC를 줄 후보에 분배하고,
  선행 guide 줄은 제외하는 규칙을 가지고 있다. 페이지네이터(`typeset.rs`)가 다른 char 범위
  규칙으로 TAC 높이를 측정하면 layout과 pagination이 갈라진다.
- 실험: 선행 guide 줄을 무조건 실제 수식 줄로 보존하면
  `issue_1256_2022_sep_page10_question12_keeps_between_notes_gap`가 실패했다. 따라서 빈 runs
  `line_seg`가 항상 수식 textRun 줄이라는 가정은 틀리고, 렌더러의 guide 제외 규칙과 같은
  소유권을 페이지네이터에 적용해야 한다.
- Stage72의 코드 변경은 `typeset.rs`에서 줄별 TAC 소유권을 렌더러와 동일하게 계산하도록
  맞추는 보조 수정이다. 이 변경만으로 남은 `question_marker_flow_drift`는 줄어들지 않았으므로
  남은 원인은 미주 묶음의 column/page advance 판단(`compute_en_metrics`,
  `advance_for_fit`, `advance_for_new_endnote`) 쪽으로 계속 좁힌다.

## 검증

- `cargo fmt --all`
- `cargo build --bin rhwp`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 52 passed
- 부정 실험 후 복구 검증:
  - 선행 guide 줄을 실제 줄로 보존하는 실험은 `issue_1256_2022_sep_page10_question12...` 실패.
  - guide 제외 규칙 복구 후 focused test 52개 다시 통과.
