# #1999 Stage 3 - 14~16쪽 그림/표 흐름 불일치 원인 분석

## 목적

Stage 2에서 PDF/HWP/HWPX 쪽수는 모두 115쪽으로 맞았지만, 14~16쪽 시각 비교는 아직 통과하지 못했다.
특히 p14~p15에서 그림 2개와 뒤따르는 표/본문이 기준 PDF보다 한 페이지 앞에 배치된다.

이 스테이지에서는 코드를 바로 고치지 않고, 다음 자료를 먼저 대조한다.

- `output/task1999_issue1999_p14_16_hwp_final/.../render_tree/`
- `output/task1999_issue1999_p14_16_hwpx_final/.../render_tree/`
- `review_014.png`, `review_015.png`, `review_016.png`
- 관련 표 셀의 `CellUnit`, non-inline control flow height, RowBreak 컷 위치

## 검증 기준

- 특정 샘플명/페이지 번호/이슈 번호에 의존한 분기는 만들지 않는다.
- 그림 두 개가 같은 문단의 `TopAndBottom` flow control로 처리되는지, 서로 다른 flow group인지 문서 속성으로 확인한다.
- 기준 PDF에서 p14 하단에 그림이 밀려나고 p15 상단에 남는 이유가 페이지 예산, 그림 flow height, 문단 trailing height 중 무엇인지 구분한다.

## 진행 기록

### Stage 3 시작 상태

- 선행 커밋:
  - `0f2cce6f2 task 1999: RowBreak 거대 셀 쪽수 정합 보정`
  - `0675de6b3 task 1999: HWP 저장본 115쪽 회귀 가드 추가`
- 현재 작업트리: clean
- Stage 2 결론: 쪽수 parity는 통과, 14~16쪽 시각 배치는 미통과

### pi=310 문단 속성 확인

임시 probe로 RowBreak 표의 `cell[2]` 내부 문단 300~325를 확인했다.

- 대상 표: `page_break=RowBreak`, `row_count=3`, `col_count=1`
- `cell[2]`: 2507개 문단
- pi=310:
  - 텍스트: 공백만 있는 문단
  - `LineSeg`: `vpos=57600`, `line_height=1200`, `line_spacing=720`
  - 그림 2개:
    - 둘 다 `wrap=TopAndBottom`
    - 둘 다 `flow_with_text=true`
    - 둘 다 `treat_as_char=false`
    - 둘 다 `vert_rel_to=Para`, `vert_align=Top`, `horz_rel_to=Column`
    - 그림 1: `vertical_offset=1790`, `height=13924`, `margin.top=852`, `margin.bottom=852`
    - 그림 2: `vertical_offset=2202`, `height=13517`, `margin.top=852`, `margin.bottom=852`

### 1차 보정 시도

기존 표 셀 TopAndBottom 예약 높이는 `vertical_offset + height`만 사용했다. 일반 pagination과
`paragraph_layout::calc_sibling_topandbottom_reserved_hu`는 그림/도형 예약 높이에 margin을 포함하므로,
표 셀 flow 높이도 `vertical_offset + margin.top + height + margin.bottom`으로 보정했다.

검증:

- `cargo fmt --check`
- `env CARGO_INCREMENTAL=0 cargo test row_cut_tests --lib`
  - 결과: 18 passed
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1949_giant_cell_render_perf`
  - 결과: 1 passed
- PDF/HWPX/HWP 쪽수: 115 / 115 / 115

visual sweep:

- HWP: `output/task1999_issue1999_p14_16_hwp_stage3/summary.json`
- HWPX: `output/task1999_issue1999_p14_16_hwpx_stage3/summary.json`
- 둘 다 SVG/PDF/render-tree 115 / 115 / 115쪽, 선택 페이지 14~16쪽, `flagged=0/3`

실제 시각 판정:

- 실패. p14에는 여전히 pi=310의 그림 2개가 하단에 남아 있다.
- 기준 PDF에서는 같은 그림 2개가 p15 상단에 남아 있다.
- 따라서 margin 포함은 필요한 정합성 보정일 가능성은 있지만, p14~p15 흐름 불일치의 직접 원인은 아니다.

다음 확인:

- p14의 `start_cut=[503]`, `end_cut=[535]` 구간에서 pi=310 그림 유닛이 어느 인덱스인지 확인한다.
- 한컴이 p14에 물리적으로 들어갈 수 있는 그림 유닛을 왜 다음 페이지로 넘기는지, RowBreak 컷의
  hard break/vpos reset/orphan 처리 조건을 문서 속성 기반으로 좁힌다.

### p14 CellUnit 컷 확인

임시 probe로 p14에 해당하는 RowBreak 셀 컷을 직접 확인했다.

- 입력 컷: `start_cut=[503]`
- 페이지 예산: `avail=1009.1px`
- 결과: `end_cut=[534]`, `hit_hard_break=false`, `fully_consumed=false`, `consumed=984.31px`

주요 유닛:

- `unit 503` ~ `unit 531`: 일반 가시 본문 줄
- `unit 532`: pi=310의 공백 텍스트 줄, 높이 `25.60px`
- `unit 533`: pi=310의 non-inline TopAndBottom 그림 flow 유닛, 높이 `216.31px`
- `unit 534`: 뒤따르는 빈 spacer, 높이 `25.60px`
- `unit 535`: 다음 가시 본문 줄

해석:

- 현재 컷은 `unit 533` 그림 유닛까지 p14에 넣고, 바로 뒤 `unit 534` 빈 spacer는 다음 쪽으로 넘긴다.
- `unit 533`만 넣으면 `984.31px`로 페이지 예산 안에 들어간다.
- `unit 533 + unit 534`를 함께 넣으면 `1009.91px`로 예산 `1009.1px`을 넘는다.
- 기준 PDF는 이 그림 묶음을 p15 상단으로 넘긴다.

따라서 직접 원인은 그림 높이가 모자라서가 아니라, RowBreak 컷이 `TopAndBottom` 그림 flow 유닛과
뒤따르는 빈 spacer를 분리해 그림만 페이지 하단에 고립시키는 것이다. 다음 스테이지에서는 특정 샘플이나
페이지 번호가 아니라 `non-inline TopAndBottom flow unit + following empty spacer`라는 문서 구조에 근거해
둘을 같은 쪽에 둘 수 없으면 그림 유닛부터 다음 쪽으로 넘기는지 검증한다.

Stage 3에서 시도한 margin 포함 변경은 아직 단독 해결로 확정하지 않는다. 다음 스테이지에서 구조적 orphan
보정과 함께 유지 여부를 다시 검증한다.
