# Task M100 #1209 Stage 2

## 목적

`3-11월_실전_통합_2022.hwp` 17쪽 `문29)` 미주 간격 불일치를 재확인하고, task 1139와 task 1189에서 누적된 미주 간격 보정이 `미주 모양`의 공통 `미주 사이` 설정으로 통합 가능한지 분석한다.

## 시작 기준

- 이슈: [#1209](https://github.com/edwardkim/rhwp/issues/1209)
- 작업 브랜치: `local/task_m100_1209`
- 선행 커밋: `0c503873` (`task 1209: Stage1 미주 사이 간격 보정`)
- 대상 문서: `samples/3-11월_실전_통합_2022.hwp`
- 우선 대상 페이지: 17쪽 `문29)`
- 관련 누적 작업: task 1139 Stage 8/9/15/16/19/23/24/26/27, task 1189 Stage 2~7

## 분석 질문

1. `FootnoteShape.raw_unknown`으로 보존되는 한컴 UI `미주 사이` 값을 페이지네이터와 렌더러가 같은 방식으로 참조하고 있는가?
2. 현재 `typeset.rs`의 compact endnote 분배와 `height_cursor.rs`의 렌더 VPOS 보정이 서로 다른 규칙으로 같은 간격을 중복/누락하고 있는가?
3. `문29)`처럼 앞선 미주 본문과 다음 제목/본문 사이 간격이 흔들리는 경우를 문항별 예외 없이 공통 정책으로 표현할 수 있는가?
4. 공통화가 가능하다면 최소 변경 지점은 `typeset`의 미주 분배 단계인지, `HeightCursor`의 렌더 보정 단계인지, 아니면 shared helper가 필요한지 판단한다.

## 현재 판단

- Stage1까지의 수정은 `문27)`, `문28)` 앞 간격에는 효과가 있었지만 `문29)` 이상을 해결했다고 보지 않는다.
- 기존 히스토리상 기본 `미주 사이 7mm`는 원본 `LINE_SEG`/VPOS 흐름에 이미 상당 부분 포함되어 있어 무조건 더하면 페이지 수가 증가한다.
- 반대로 일부 split/rewind/stale-forward 흐름에서는 같은 7mm가 렌더 또는 분배 단계에서 사라져 보존해야 한다.
- 따라서 필요한 것은 값 자체를 전역 가산하는 방식이 아니라, “이미 반영된 흐름”과 “손실된 흐름”을 같은 판정 함수로 나누는 공통 정책으로 보인다.

## 진행 계획

1. task 1139/1189 관련 문서와 현재 `typeset.rs`, `height_cursor.rs`의 미주 간격 분기들을 인벤토리화한다.
2. `3-11월_실전_통합_2022.hwp` 17쪽 `문29)`의 현재 SVG/PDF 차이를 수치화한다.
3. `FootnoteShape.raw_unknown` 기반의 공통 helper 후보를 설계한다.
4. 구현이 필요하면 작업지시자 승인 후 Stage2 구현으로 진행한다.

## 현재 상태

- 2026-06-01: 작업지시자가 Stage1 커밋 후 새 스테이지에서 미주 간격 공통화 가능성 확인을 지시했다.
- 2026-06-01: Stage1 커밋 `0c503873` 이후 새 스테이지 문서를 만들고 분석을 시작했다.
- 2026-06-01: 현재 산출물 기준 17쪽 `문29)` 제목(`pi=812`)은 RHWP debug overlay top `y=791.7px`, rsvg-convert red row `y=792..802px`에 잡힌다. PDF 96dpi 추출본의 같은 제목 red row는 `y=813..823px`라서 현재 RHWP가 약 `21px` 위에 있다.
- 2026-06-01: 작업지시자가 Stage2 진행을 승인했다.
- 2026-06-01: `layout.rs`의 Stage1 partial-only 보정을 compact 미주 제목 공통 gap 보존으로 확장했다. 직전 미주 paragraph 또는 이어진 partial의 마지막 `line_spacing`을 후보 gap으로 잡고, 현재 제목의 VPOS 보정이 실제로 아래 방향으로 적용됐거나 직전 항목이 이어진 partial일 때만 최소 gap으로 보존한다.
- 2026-06-01: 처음에는 full paragraph 뒤 제목에도 무조건 `y_before_vpos + line_spacing`을 적용했으나, 14쪽 `문23)`처럼 이미 시각 간격이 있는 흐름에서 gap이 중복 적용됐다. 따라서 full paragraph 뒤에는 `HeightCursor`가 title을 아래로 보정한 경우에만 손실된 gap을 보존하도록 좁혔다.
- 2026-06-01: 수정 후 17쪽 `문29)` red row는 RHWP rsvg-convert 기준 `y=812..821px`, PDF 기준 `y=813..822px`로 맞춰졌다.

## 인벤토리

### 공통 설정 확인

- `src/renderer/typeset.rs`의 `endnote_between_notes_margin()`은 `FootnoteShape.raw_unknown`을 한컴 UI `미주 사이`로 해석한다.
- task 1139 Stage15에서 HWP5 실제 레코드를 확인했다.
  - 기본 파일: `raw_unknown≈1984HU`(`7mm`)
  - `미주사이20` 기준 파일: `raw_unknown≈5669HU`(`20mm`)
- 같은 Stage15에서 `7mm`를 모든 미주 사이에 전역 가산하면 23쪽 문서가 24쪽 또는 25쪽으로 밀리는 것을 확인했다.
- 따라서 `미주 사이` 값은 공통 설정이지만, 기본 `7mm`는 원본 `LINE_SEG`/VPOS 흐름에 이미 들어 있는 경우가 많다. 필요한 처리는 전역 가산이 아니라 누락된 전환에서만 보존하는 판정이다.

### 현재 코드의 개별 처리 지점

- `typeset.rs` 페이지네이터 쪽
  - `default_late_question_group_tail`, `current_default_late_question_title`, `allow_default_late_question_tail`에서 `matches!(en_ref.number, 29 | 30)`가 직접 들어간다.
  - `late_question_intro_tail`, `late_question_continuation_tail`은 `en_ref.number == 29`에 묶여 있다.
  - `new_endnote_between_notes_px`는 `미주 사이`를 쓰지만, `suppress_late_question_gap_for_fit`처럼 문항 번호 기반 예외가 앞에서 간격 적용 여부를 바꾼다.
- `height_cursor.rs` 렌더 쪽
  - `compact_endnote_question_title`, `compact_endnote_new_note_jump`, `compact_endnote_stale_note_gap`, `compact_endnote_tac_picture_gap`, `compact_endnote_deep_backtrack`, `compact_endnote_title_tail_backtrack`처럼 VPOS 패턴별 플래그가 누적되어 있다.
  - 이 계층은 `FootnoteShape.raw_unknown`을 직접 받지 않으므로 실제 `미주 사이` 값 대신 직전 `line_spacing`과 현재/직전 paragraph 패턴으로 간격을 추정한다.
- `layout.rs` Stage1 보정
  - 앞쪽 쪽/단에서 이어진 `PartialParagraph` 뒤 새 문제 제목이 오면 직전 partial의 마지막 `line_spacing`을 보존하도록 추가했다.
  - 이 보정은 비교적 공통 성격이지만, 여전히 `HeightCursor`의 다른 compact endnote 플래그들과 별도로 동작한다.

## 판단

- 작업지시자 지적대로 현재 구조는 케이스 바이 케이스가 맞다.
- 다만 단순히 `미주 모양.raw_unknown`을 모든 문항 사이에 적용하는 방식은 이미 실패한 실험이다. 기본 7mm가 LINE_SEG에 들어 있는 정상 흐름과, split/rewind/stale-forward 때문에 손실되는 비정상 흐름을 구분해야 한다.
- 공통화는 가능해 보인다. 방향은 문항 번호 예외를 늘리는 것이 아니라 `EndnoteSpacingPolicy` 같은 공통 판정으로 아래 정보를 한곳에서 계산하는 것이다.
  1. `between_notes_hu = FootnoteShape.raw_unknown`
  2. `base_flow_hu = 1984HU` 기본 7mm
  3. pagination에서는 `extra_hu = max(0, between_notes_hu - base_flow_hu)`만 별도 advance 후보로 사용
  4. renderer에서는 직전 `line_spacing`/split partial의 trailing spacing을 `preserved_gap_px`로 보고, 이 gap이 손실된 전환에만 복원
  5. 전환 종류는 문항 번호가 아니라 `new endnote title`, `continued partial 뒤 title`, `local/internal vpos rewind`, `stale forward vpos`, `TAC object only`, `column bottom fit` 같은 구조적 상태로 판정
- 최소 변경 후보는 두 단계다.
  1. 먼저 `typeset.rs` 안에서 번호 기반 `29 | 30` / `== 29` 조건을 `default_between_notes_gap + last column + fit 가능 + title/tail 구조` 조건으로 대체할 수 있는지 실험한다.
  2. 이후 `height_cursor.rs`와 `layout.rs`에 흩어진 gap 보존 로직을 같은 정책 이름으로 묶고, 가능하면 `FootnoteShape.raw_unknown` 또는 계산된 `preserved_gap_px`를 렌더 컨텍스트에 전달한다.

## 다음 검증 후보

- `3-11월_실전_통합_2022.hwp` 17쪽 `pi=811 -> pi=812` (`문29)`) 현재 gap과 PDF gap.
- task 1189에서 보정한 10쪽/11쪽/12쪽/14쪽/17쪽 회귀 조건.
- task 1139에서 보정한 `3-09월_교육_통합_2022.hwp` 12쪽/13쪽/23쪽, 그리고 `미주사이20` 24쪽 분기.
- 공통화 실험은 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`를 먼저 통과시키고, 커밋 전 전체 `cargo test --tests`로 확인한다.

## 검증 기록

- `cargo test issue_1209_2022_nov_page17_question29_keeps_hancom_gap_after_full_para --test issue_1139_inline_picture_duplicate -- --nocapture` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(37 passed, 0 failed).
- `cargo run --quiet --bin rhwp -- export-svg samples/3-11월_실전_통합_2022.hwp -p 16 -o output/task1209_stage2_page17_analysis/fixed --debug-overlay` 통과.
- `rsvg-convert -f png -o output/task1209_stage2_page17_analysis/fixed/rhwp_page17.png output/task1209_stage2_page17_analysis/fixed/3-11월_실전_통합_2022_017.svg` 통과.
- `pdftoppm -f 17 -l 17 -singlefile -r 96 -png pdf/3-11월_실전_통합_2022.pdf output/task1209_stage2_page17_analysis/fixed/pdf_page17` 통과.
- `cargo run --quiet --bin rhwp -- export-svg samples/3-11월_실전_통합_2022.hwp -p 13 -o output/task1209_stage2_page14_analysis/fixed --debug-overlay` 통과.
- `rsvg-convert -f png -o output/task1209_stage2_page14_analysis/fixed/rhwp_page14.png output/task1209_stage2_page14_analysis/fixed/3-11월_실전_통합_2022_014.svg` 통과.
- `pdftoppm -f 14 -l 14 -singlefile -r 96 -png pdf/3-11월_실전_통합_2022.pdf output/task1209_stage2_page14_analysis/fixed/pdf_page14` 통과.
- `cargo fmt --all --check` 통과.
- `git diff --check` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
