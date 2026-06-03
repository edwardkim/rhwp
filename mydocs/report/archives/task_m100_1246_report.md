# 최종 결과 보고서 — Task #1246: 미주 between-notes margin (HeightCursor min-gap)

- **이슈**: #1246 (M100 / v1.0.0) — #1238(미주 between-notes margin 누락) 흡수
- **브랜치**: `feature/issue-1246-endnote-vpos-anchor` (base: `stream/devel`)
- **작성일**: 2026-06-02
- **결과**: 완료 (closes #1238, #1246)

## 1. 문제

미주(답안) 영역에서 **다줄 풀이로 끝나는 문제 다음의 제목**이 직전 줄에 붙었다(문22 외 문21·23·
27·28·29). 한컴 2022 기준 미주 사이 7mm(between-notes) 간격이 누락(문22 above-gap 0px).

## 2. 원인 (조사 경과)

#1238 → #1246 으로 이어진 조사로 확정:

- between-notes 는 **가산(additive)이 아니라 min-gap(max)** 모델 (한컴 2022 PDF 검증).
- render 는 미주 문단을 `HeightCursor.vpos_adjust`(#1027 Stage C)로 **vpos→y 매핑**하며, compact
  endnote 사이 gap 특례(forward-cap/backtrack/safe-vpos-backtrack #1209)를 이미 보유.
- **유일한 사각지대**: stored vpos gap 이 거의 0 인 경우(다줄 풀이 마지막 줄 trailing 이 render
  에서 누락 → gap≈0, 문22) 끌어올리는 **min-gap 로직 부재**.
- #1238 의 render-클램프 시도는 vpos 위치 시스템(#1209 backtrack 등)과 충돌해 폐기.

## 3. 해결

`HeightCursor.vpos_adjust` 말미에 **min-gap 케이스** 추가:

- 조건: 새 미주 제목 & forward 흐름 & 다줄 prev & **stored vpos gap ∈ [-0.5, 4.0)px**.
- 동작: `y_offset + prev_line_spacing_px`(직전 문단에 주입된 between-notes 줄간격)로 끌어올림.
- 의도적 소-gap(문13 ~10px)·backtrack(문12 #1209)·단일줄 prev 는 **제외** → over-lift/회귀 회피.
- 보정값이 `prev_line_spacing_px`(= pagination `compute_en_metrics` 가 `bottom_with_spacing` 으로
  이미 예약한 trailing)와 동일 → render·pagination 정합, overflow 없음.

배선: `typeset` 이 섹션 between-notes 마진(HU)을 `PaginationResult` 로 전달 → `layout` 셋업 시
HeightCursor 에 주입(미주 흐름 컬럼만).

## 4. 검증

| 항목 | 결과 |
|------|------|
| **문22 above-gap** (3-11월 page14) | **0 → 26.5px (=7mm)** — 한컴 2022 PDF 시각 정합 확인 |
| `issue_1189_2022_nov_pages10_12` (pi=475 overflow) | ✅ |
| `issue_1189_2022_nov_page17` (문28 수식) | ✅ |
| `issue_1209_2022_sep_page10_question12` (문12 safe-vpos-backtrack) | ✅ |
| `issue_1139` 미주군, 3-09·10·24월 회귀군 | ✅ |
| HeightCursor 단위테스트 (신규 3 + 기존 26) | ✅ 29 |
| **전체 `cargo test`** | ✅ **1945 passed, 0 failed** |

시각: `pdf/3-11월_실전_통합_2022.pdf` page14 ↔ `rhwp export-svg -p 13` 크롭 대조 — 문22 위 7mm
gap 동일, 붙음 해소.

## 5. #1238 흡수 종결

#1238 의 검증된 모델(min-gap)·PDF 근거·전면 조사를 #1246 에서 회귀 없이 구현 완료. #1238 의
render-클램프(별 접근)는 폐기. 본 PR 로 #1238, #1246 동시 종결.

## 6. 산출물

- 소스: `height_cursor.rs`(min-gap + 단위테스트 3), `layout.rs`/`typeset.rs`/`pagination.rs`/
  `pagination/engine.rs`/`rendering.rs`/`paragraph_layout.rs`(배선·클램프 제거).
- 문서: `plans/task_m100_1246{,_impl}.md`, `tech/endnote_vpos_anchor_1246.md`,
  `working/task_m100_1246_stage{1,2}.md`, `tech/between_notes_multiline_1238.md §7`,
  `working/task_m100_1238_stage2.md`.
