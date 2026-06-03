# Task M100 #1209 Stage 1

## 목적

Stage0에서 확인한 `3-11월_실전_통합_2022.hwp` 14쪽 `문22)` 미주 간격 불일치를 보정하고, 추가 비교 요청된 `문23)` 및 17쪽 `문27)`, `문28)` 시작 간격도 함께 확인한다.

## 시작 기준

- 이슈: [#1209](https://github.com/edwardkim/rhwp/issues/1209)
- 작업 브랜치: `local/task_m100_1209`
- 선행 커밋: `6f6a5e71` (`task 1209: Stage0 원인 분석 기록`)
- 대상 문서: `samples/3-11월_실전_통합_2022.hwp`
- 대상 페이지: 14쪽, 17쪽
- 대상 구간: 14쪽 미주 왼쪽 단 `pi=631 -> pi=632`, `pi=643 -> pi=644`; 17쪽 `pi=786 -> pi=787`, `pi=800 -> pi=801`

## 구현 판단

1. `pi=632`의 원문 VPOS는 현재 위치보다 지나치게 아래를 가리키므로 그대로 적용하면 안 된다.
2. 하지만 직전 문단 `pi=631`의 trailing `line_spacing=1984HU`는 `문22)` 앞의 실제 미주 간격으로 보인다.
3. 따라서 compact 미주 질문 제목에서 큰 stale forward가 발생하면 원문 점프 전체는 억제하되, 직전 line spacing만 보존하는 별도 보정 경로를 둔다.
4. 보정 경로는 기존 compact gap 처리와 같이 VPOS base도 함께 이동해야 다음 문단에서 억제한 큰 점프가 다시 누적되지 않는다.
5. 추가 비교한 `문23)`은 직전 빈 spacer 문단이 이미 시각 간격을 만들고 있는데도 compact deep-backtrack이 새 제목을 위로 되감는 문제가 별도로 있었다.
6. compact 미주 질문 제목이 빈 spacer 문단 뒤에 올 때는 spacer가 만든 간격을 유지하도록 deep-backtrack 보정을 제외한다.
7. 17쪽 `문27)`, `문28)`은 이전 쪽/단에서 이어진 미주 조각 뒤 새 문제 제목이 오면서 직전 `line_spacing=1984HU`에 해당하는 미주 사이 간격이 사라졌다.
8. 이 경우는 문항별 특수 보정이 아니라 compact 미주 공통 정책으로 처리한다. 이어진 `PartialParagraph(start_line > 0)` 뒤에 새 `문N)` 제목이 오면 이전 partial 마지막 line spacing을 최소 간격으로 보존하고, 후속 항목도 같은 시각 기준을 따르도록 VPOS base를 함께 이동한다.

## 진행 계획

1. `src/renderer/height_cursor.rs`의 compact endnote question title 경로를 좁게 수정한다.
2. `tests/issue_1139_inline_picture_duplicate.rs`에 `3-11월_실전_통합_2022.hwp` 14쪽 `문22)` 간격 조건을 추가한다.
3. 추가 요청된 `문23)` 간격 조건을 같은 회귀 테스트에 포함한다.
4. 수정 후 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`로 대상 회귀 테스트를 확인한다.
5. PDF/SVG/rsvg-convert 산출물로 14쪽과 17쪽을 다시 비교하고 Stage1 기록을 갱신한다.

## 현재 상태

- 2026-06-01: 작업지시자가 Stage0 분석 결과를 승인했다. Stage0 변경분을 커밋한 뒤 Stage1을 시작한다.
- 2026-06-01: `src/renderer/height_cursor.rs`에 compact 미주 질문 제목의 큰 stale forward 보정 경로를 추가했다. `pi=632`처럼 단 중간의 새 문제 제목에서 저장 VPOS가 페이지 하단 근처로 크게 튀는 경우, 절대 VPOS는 억제하되 직전 `line_spacing`만 보존한다.
- 2026-06-01: 보정 경로에서 `vpos_lazy_base`를 함께 이동하도록 유지했다. 수정 후 `pi=633`은 새 base `415404`를 사용해 `stale_forward=false`, `applied=true`로 이어진다.
- 2026-06-01: `tests/issue_1139_inline_picture_duplicate.rs`에 `issue_1209_2022_nov_page14_question22_keeps_hancom_endnote_gap` 회귀 테스트를 추가했다.
- 2026-06-01: 수정 후 SVG 기준 `문22)` baseline이 `467.4667px`에서 `493.92px`로 내려가 `33이다.` 직후 간격이 보존됐다. 최신 산출물은 `output/task1209_stage1_3-11_page14/fixed/rhwp_page14.png`이다.
- 2026-06-01: 작업지시자가 `문23)` 비교를 추가 요청했다. PDF 기준 `f(4)=13` 아래에서 `문23)`까지 약 `25.9px` 간격인데, 현재 SVG에서는 `문23)` 제목이 `1030.28px` 근처까지 위로 당겨져 있었다.
- 2026-06-01: `pi=644`의 VPOS 로그에서 `compact_deep_backtrack=true`가 확인됐다. 직전 `pi=643`은 빈 spacer 문단이므로, compact 미주 질문 제목이 빈 spacer 뒤에 올 때 deep-backtrack을 적용하지 않도록 보정했다.
- 2026-06-01: 수정 후 SVG 기준 `문23)` 제목 y가 `1030.28px`에서 `1050.7067px`로 내려가 PDF/한컴 쪽 간격에 가까워졌다. 회귀 테스트에도 `pi=643 -> pi=644` 간격 조건을 추가했다.
- 2026-06-01: 작업지시자가 17쪽 미주 간격도 확인하고 공통 로직 가능성을 질의했다. `output/task1209_stage1_3-11_page17_compare/current/`에 현재 SVG/PNG와 PDF 17쪽 PNG를 생성해 비교했다.
- 2026-06-01: 최초 판단에서는 17쪽 위치 이탈을 작게 보았으나, 작업지시자 재지적으로 오판을 정정했다. PDF bbox 96dpi 환산 기준 `문27)`, `문28)` 제목은 현재 출력보다 약 20~30px 아래여야 했다.
- 2026-06-01: `RHWP_VPOS_DEBUG`에서 `pi=787`은 page-path VPOS가 6px 정도만 보정했고, `pi=801`은 stale forward page-path라 기존 compact gap 보정에 들어가지 않았다. 두 경우 모두 이전 partial의 마지막 `line_spacing=1984HU`가 새 미주 제목 앞 간격으로 보존되지 않았다.
- 2026-06-01: `layout.rs`에 이어진 `PartialParagraph` 뒤 compact 미주 질문 제목이 올 때 직전 partial 마지막 `line_spacing`을 공통 최소 간격으로 적용하는 정책을 추가했다.
- 2026-06-01: `height_cursor.rs`에는 렌더 y를 아래로 밀었을 때 후속 VPOS 기준도 같은 방향으로 따라오도록 `shift_vpos_base_for_rendered_delta`를 추가했다.
- 2026-06-01: 수정 후 SVG 기준 17쪽 `문27)` 제목 baseline은 `237.04px`에서 `257.4667px`, `문28)` 제목 baseline은 `201.16px`에서 `219.1867px`로 내려가 PDF/rsvg-convert 기준 간격에 가까워졌다. 최신 산출물은 `output/task1209_stage1_3-11_page17_compare/split_gap_fix/rhwp_page17.png`이다.
- 2026-06-01: 작업지시자가 17쪽 `문29)` 위치도 여전히 이상하다고 지적했다. Stage1은 `문27)`, `문28)` 앞 간격 보정까지 커밋하고, `문29)` 및 task 1139/1189에서 누적된 미주 간격 예외를 공통 `미주 모양/미주 사이` 정책으로 통합 가능한지 새 스테이지에서 분석한다.

## 검증 기록

- `cargo test compact_endnote_question_title_preserves_spacing_on_stale_forward_jump -- --nocapture` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(35 passed, 0 failed).
- `cargo test compact_endnote_deep_backtrack -- --nocapture` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 재통과(35 passed, 0 failed).
- `cargo run --quiet --bin rhwp -- export-svg samples/3-11월_실전_통합_2022.hwp -p 13 -o output/task1209_stage1_3-11_page14/fixed` 통과.
- `rsvg-convert -f png -o output/task1209_stage1_3-11_page14/fixed/rhwp_page14.png output/task1209_stage1_3-11_page14/fixed/3-11월_실전_통합_2022_014.svg` 통과.
- `cargo run --quiet --bin rhwp -- export-svg samples/3-11월_실전_통합_2022.hwp -p 16 -o output/task1209_stage1_3-11_page17_compare/current` 통과.
- `rsvg-convert -f png -o output/task1209_stage1_3-11_page17_compare/current/rhwp_page17.png output/task1209_stage1_3-11_page17_compare/current/3-11월_실전_통합_2022_017.svg` 통과.
- `pdftoppm -f 17 -l 17 -singlefile -r 96 -png pdf/3-11월_실전_통합_2022.pdf output/task1209_stage1_3-11_page17_compare/current/pdf_page17` 통과.
- `cargo run --quiet --bin rhwp -- export-svg samples/3-11월_실전_통합_2022.hwp -p 16 -o output/task1209_stage1_3-11_page17_compare/split_gap_fix` 통과.
- `rsvg-convert -f png -o output/task1209_stage1_3-11_page17_compare/split_gap_fix/rhwp_page17.png output/task1209_stage1_3-11_page17_compare/split_gap_fix/3-11월_실전_통합_2022_017.svg` 통과.
- `cargo test issue_1189_2022_nov_page17_internal_rewind_keeps_formula_tail_on_next_page --test issue_1139_inline_picture_duplicate -- --nocapture` 통과.
- `cargo test issue_1209_2022_nov_page17_split_endnote_titles_keep_hancom_gap --test issue_1139_inline_picture_duplicate -- --nocapture` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(36 passed, 0 failed).
- `cargo test --tests` 통과.
- `cargo fmt --all --check` 통과.
- `git diff --check` 통과.
