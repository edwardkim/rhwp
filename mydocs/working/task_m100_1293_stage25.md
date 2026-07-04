# Task 1293 Stage 25: 미주 사이 20mm와 구분선 없음 overflow 분석

## 목적

Stage24 전체 sweep에서 page count는 모두 SVG/render tree/PDF 1:1로 맞았지만, 일부 미주 모양
설정 샘플에서 renderer `LAYOUT_OVERFLOW`가 남았다. 이번 단계는 가장 큰 두 target을 기준으로
공식 `미주 모양` 값이 번호 경계와 separator block에서 어떻게 소비되어야 하는지 확인하고,
문항/문서별 수치 보정이 아니라 공통 미주 flow 로직으로 수정 가능한 지점을 찾는 것이다.

## 우선 분석 대상

1. `2024-11-practice-above0-between20-below2`
   - 설정: 구분선 위 0mm, 미주 사이 20mm, 구분선 아래 2mm
   - Stage24 overflow: 51건
   - 우선 페이지/문단: page 9~11, `pi=497`, `pi=543` 이후 overflow chain
2. `2024-11-practice-no-separator-above20-between20-below20`
   - 설정: 구분선 없음, 구분선 위 20mm, 미주 사이 20mm, 구분선 아래 20mm
   - Stage24 overflow: 38건
   - 우선 페이지/문단: page 9, `pi=464` 이후 overflow chain

## 분석 계획

- `summary.json`의 overflow line을 page/para별로 그룹화한다.
- `dump-pages`로 overflow 직전/직후 page의 column item 배치를 확인한다.
- `render_tree_XXX.json`에서 해당 `pi`의 bbox, text, column 위치를 확인한다.
- `note_shape.json`의 정규화 값을 확인해 UI 설정과 pagination 계산식이 같은 의미를 쓰는지 대조한다.
- 원인이 `미주 사이` 예약 부족인지, separator 없음 처리 누락인지, 저장 LINE_SEG 내부 vpos rewind인지 분리한다.

## 분석 결과

### `2024-11-practice-above0-between20-below2`

- `note_shape.json` 기준 정규화 값은 UI와 일치한다.
  - `separatorAbove=0mm`
  - `betweenNotes=19.999mm`
  - `separatorBelow=1.997mm`
- Stage24 기준 첫 overflow chain은 page 10 표시분의 `pi=497`(`문9`)에서 시작했다.
- compare PNG 기준 rhwp는 PDF보다 새 문항 제목을 단 하단에 더 많이 남긴다.
- 원인은 새 미주 제목이 sequential height 기준으로는 들어간다고 판단되지만, 실제 렌더는 저장
  `vpos`와 큰 `미주 사이` gap을 함께 복원해 frame 아래로 내려가는 데 있었다.
- 따라서 새 미주 제목 tail 판단에서 제목 한 줄만 보지 않고 `미주 사이 gap + 제목 첫 줄`이
  현재 단에 들어가는지 함께 보도록 했다.
- 같은 미주 내부 `vpos` 되감김도 `미주 사이`가 기본값보다 크고 흡수되지 않는 경우에는 단 하단
  전환 기준을 0.85에서 0.80으로 낮췄다.

### `2024-11-practice-no-separator-above20-between20-below20`

- `note_shape.json` 기준 정규화 값은 UI와 일치한다.
  - `separatorLength=0`, `separatorLineType=0`
  - `separatorAbove=19.999mm`
  - `betweenNotes=19.999mm`
  - `separatorBelow=19.999mm`
- page 10 표시분에서 rhwp는 `문4` 제목/본문을 좌측 단 하단에 남기지만 PDF는 우측 단 상단으로
  넘긴다.
- 이번 보정으로 `pi=497` chain은 사라졌지만, `pi=464~466`은 여전히 남았다.
- 따라서 구분선이 없을 때 `구분선 위/아래`가 separator line 없는 block에서 어떻게 소비되는지는
  아직 별도 후속 분석이 필요하다.

## 수정 내용

- `src/renderer/typeset.rs`
  - 큰 `미주 사이`가 흡수되지 않는 문서의 local vpos rewind 단 전환 기준을 별도 계산했다.
  - 새 미주 제목 tail 허용 조건에 `미주 사이 gap + 제목 첫 줄` fit 검사를 추가했다.
  - `cleared_single_line_internal_rewind_split`이 없는 큰 `미주 사이` 문서도 새 제목이 하단으로
    내려가는 경우 단/쪽 전환 후보로 보도록 확장했다.

## 검증

- `cargo fmt --all -- --check`: 통과
- `cargo test --lib compact_endnote -- --nocapture`: 28개 통과
- `cargo build --bin rhwp`: 통과
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between20-below2 --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage25_gap_target_final --rhwp-bin target/debug/rhwp`
  - `2024-11-practice-above0-between20-below2`
    - page count: 22/22/22 유지
    - overflow: 51건 -> 38건
    - flagged pages: 21 -> 19
    - `pi=497`, `pi=543~546`, `pi=595`, `pi=650` overflow chain 제거
  - `2024-11-practice-no-separator-above20-between20-below20`
    - page count: 23/23/23 유지
    - overflow: 38건 -> 37건
    - `pi=497` overflow chain 제거
    - `pi=464~466`, `pi=543~544`, `pi=593~597` 등은 잔여
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 47개 통과, 5개 실패
  - `issue_1139_page17_endnote_question30_starts_on_right_column`
  - `issue_1189_2022_oct_page11_endnote_question_gaps_match_pdf`
  - `issue_1274_2022_oct_page11_question20_equation_tail_stays_in_frame`
  - `issue_1274_2022_oct_page16_question30_title_keeps_first_line`
  - `issue_1274_2022_sep_page18_question26_equation_paragraph_reserves_height`
  - 실패 항목은 Stage24에서 이미 남아 있던 기존 교육 통합/실전 샘플 overflow 축과 연결되어 있으므로,
    이번 단계는 부분 개선으로 기록하고 다음 스테이지에서 계속 해결한다.

## 남은 문제

- `above0-between20-below2`는 page 13 표시분의 `pi=593` TAC 그림/도형과 후반 page 19/22
  overflow chain이 남아 있다.
- `no-separator`는 구분선 없음 상태의 `구분선 위/아래` block 소비 방식이 아직 한컴/PDF와
  다르다. 다음 스테이지에서는 separator line이 없는 미주 block의 시작 간격과 첫 column 전환
  조건을 우선 분석한다.
