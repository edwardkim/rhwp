# Task 1293 Stage 41: 구분선 0 샘플 잔여 overflow 분석

## 배경

Stage40 focused sweep에서 `2024-11-practice-above0-between0-below0`
(`3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`)만
render-tree overflow가 남았다.

이 샘플은 한컴 미주 모양에서 다음 값이 모두 0mm인 기준 샘플이다.

- 구분선 위: 0mm
- 미주 사이: 0mm
- 구분선 아래: 0mm

공식 미주 모양 접근자로는 section 0 endnote 값이 모두 0mm로 노출된다. 따라서
남은 overflow는 단순 수치 보정이 아니라, separator/gap이 없는 미주에서 저장 vpos,
문단 line advance, 단/쪽 넘김 조건이 어떻게 결합되는지 확인해야 한다.

## 현재 sweep 결과

`output/task1293_stage40_focused/summary.json` 기준:

- SVG/PDF/render tree 페이지 수: `21/21/21`
- frame overflow: 0
- render-tree overflow: 14

대표 위치:

| page | paragraph | 유형 | overflow |
|---:|---:|---|---:|
| 9 | `pi=510` | FullParagraph | 32.2px |
| 10 | `pi=537` | Shape | 31.3px |
| 10 | `pi=538` | FullParagraph | 67.2px |
| 12 | `pi=616` | FullParagraph | 3.2px |
| 13 | `pi=691` | FullParagraph | 3.9px |
| 13 | `pi=693` | PartialParagraph | 58.0px |
| 16 | `pi=853` | PartialParagraph | 51.6px |
| 17 | `pi=914` | FullParagraph | 29.0px |

## 확인 계획

1. overflow가 큰 page 10 `pi=537~538`, page 13 `pi=693`, page 16 `pi=853`부터
   `dump-pages`, `dump`, render-tree JSON을 대조한다.
2. 각 후보가 다음 단/쪽으로 넘어가야 하는지, 아니면 renderer y 계산만
   pagination보다 낮아진 것인지 분리한다.
3. 구분선/gap이 0인 샘플에서도 문단 전체 line advance가 누락되는 공통 구조가
   있는지 확인한다.
4. 수정은 개별 page/paragraph 특례가 아니라 미주 공식 설정과 flow 구조를 기준으로
   적용한다.

## 분석 결과

Stage41 중간 실험에서 보정 범위를 너무 넓히면 기존 7mm/8mm 계열이 즉시 회귀했다.

- `visible_separator_saved_vpos_tail_outside`를 보이는 구분선 전체 profile에 적용하면
  `2024-11-practice-above0-between0-below0`은 일부 개선되지만 page 14에 새 overflow가 생겼다.
- `internal_rewind_head_overflows_current_column`을 전체 compact 미주에 적용하면
  `2024-11-practice-above20-between7-below2`가 기존 0건에서 57건으로 회귀했다.
- `allow_default_late_question_tail`을 exact 7mm에만 열면 0mm profile의 일부 문제는 줄지만,
  `between=0`이면서 구분선 위/아래가 있는 기존 target도 회귀할 수 있다.
- 비TAC 그림/도형 높이를 모든 profile의 `en_advance`에 반영하면 0/0/0의 `pi=538`은 줄지만
  기존 7mm target에 `pi=571` overflow가 생겼다.

따라서 이번 단계의 정식 변경은 한컴 미주 모양 UI 의미값 기준으로 다음 profile에만 한정했다.

- 구분선 위: 0mm
- 미주 사이: 0mm
- 구분선 아래: 0mm

이 profile에서는 구분선 주변에 흡수될 여백이 없으므로, 저장 `vpos` tail이 단 하단 밖으로
예측되는 경우와 internal rewind 머리가 현 단에 들어갈 수 없는 경우를 현재 단에 남기지 않는다.
또한 0/0/0 profile의 비TAC 그림/도형 전용 미주 문단은 renderer가 실제 객체 높이를 그리므로,
advance 후 다음 미주 시작 기준에도 객체 높이를 반영한다.

## 구현 내용

- `typeset.rs`에 `zero_endnote_spacing_profile` 판정을 추가했다.
  - 정규화 접근자 `separator_above_margin_hu()`,
    `endnote_between_notes_margin()`, `endnote_separator_below_margin()`이 모두 0인지 본다.
- 기본 7mm 미주 tail 허용은 유지하되, 0/0/0 profile에서는 제외했다.
  - 0/0/0은 구분선 주변 여백이 없어 기본 7mm profile처럼 하단 tail을 허용하면 saved-vpos tail이
    frame 밖으로 남는다.
- 보이는 구분선 saved-vpos tail advance와 internal rewind head overflow advance는
  `zero_endnote_spacing_profile`에서만 동작하도록 제한했다.
- 0/0/0 profile의 비TAC 그림/도형 전용 미주 문단은 advance 후 `en_advance`가 실제 객체 높이보다
  작아지지 않도록 보정했다.

## 검증 결과

- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between7-below2 --target 2024-11-practice-above20-between0-below20 --target 2024-11-practice-shape987 --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage41_zero_profile --rhwp-bin target/debug/rhwp`
- `cargo test --lib compact_endnote -- --nocapture`
- `cargo fmt --all -- --check`
- `git diff --check`

Focused sweep 결과:

| target | page count | overflow_lines | 판단 |
|---|---:|---:|---|
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 0 | 기존 7mm 계열 회귀 없음 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 구분선 위/아래가 있는 0mm 계열 회귀 없음 |
| `2024-11-practice-shape987` | 21/21/21 | 0 | 9/8/7 설정 회귀 없음 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 6 | 14건에서 6건으로 감소 |

남은 0/0/0 overflow:

- page 9 `pi=510` line 3: 32.2px
- page 10 `pi=537` Shape: 31.3px
- page 12 `pi=616`: 3.2px
- page 14 `pi=712`: 3.0px
- page 14 `pi=713`: 21.0px

추가로 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`를 실행했다.
현재 HEAD `86c9b584`를 별도 worktree로 체크아웃해 같은 테스트를 수행한 결과도 동일하게
5개 실패가 재현되었다. 따라서 아래 실패는 Stage41 변경으로 새로 생긴 회귀가 아니라
현재 브랜치 baseline 실패로 분류한다.

- `issue_1139_endnote_spacing_reference_files_match_hancom_page_counts`
- `issue_1189_2022_oct_page11_endnote_question_gaps_match_pdf`
- `issue_1209_2022_nov_page14_question22_keeps_hancom_endnote_gap`
- `issue_1274_2022_oct_page11_question20_equation_tail_stays_in_frame`
- `issue_1274_2022_oct_page16_question30_title_keeps_first_line`

## 상태

Stage41은 0/0/0 profile에 한정한 무회귀 개선으로 정리한다.

다음 단계에서는 남은 6건을 처리한다. 특히 `pi=510`, `pi=537`, `pi=616`, `pi=712~713`은
PDF 대비 페이지 흐름도 아직 맞지 않으므로, 단순 overflow 로그 제거가 아니라 한컴/PDF와
같은 단/쪽 분기인지 계속 확인해야 한다.
