# task 1274 stage25: 2022-10 11쪽 문20 line order overlap 수정

## 배경

- 작업지시자가 `3-10월_교육_통합_2022.hwp` 11쪽 `문20)`의 본문/수식 겹침이 아직 남아 있다고 다시 지적했다.
- stage24는 이 현상을 `line_order_overlap`으로 검출하도록 sweep를 정밀화했지만, 배치 자체는 수정하지 않았다.
- 이제 `pi=586` 본문 line과 `pi=587` 수식 line이 같은 세로 위치로 겹치는 원인을 찾아 공통 레이아웃 로직에서 수정한다.

## 현재 관찰

- 대상: `samples/3-10월_교육_통합_2022.hwp`
- 페이지: 11쪽 (`0-based page=10`)
- 위치: 오른쪽 단 하단 `문20） 226`
- render tree 기준:
  - `pi=586`: `이차식 [EQ]에 대하여 [EQ]라 하자.`, bbox y=`1035.1`, h=`12.0`
  - `pi=587`: 다음 수식 line `[EQ]`, bbox y=`1032.7`, h=`28.8`
  - `pi=587`이 `pi=586`보다 위로 올라가 `overlap_ratio=1.0`이다.

## 수정 방향

1. `RHWP_VPOS_DEBUG=1 export-render-tree -p 10`으로 `pi=586/587`의 `vpos_adjust` 결과를 확인한다.
2. 다음 line이 이전 line 위로 되감기는 경우가 compact 미주 하단에서 허용되는 조건인지 판별한다.
3. 문항 내부의 일반 본문 뒤 수식 line은 저장 vpos backtrack이 이전 line과 겹치면 적용하지 않도록 보정한다.
4. 수정 후 `line_order_overlap_pages`에서 11쪽 문20이 사라지는지 확인한다.

## 검증 계획

- `cargo test --lib <추가 단위 테스트> -- --nocapture`
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
  - 11쪽 문20 `line_order_overlap_candidates`가 사라지는지 확인한다.
  - `compare_011.png`와 `annotated_011.png`에서 본문/수식이 분리되는지 확인한다.
- 필요 시 `python3 scripts/task1274_visual_sweep.py --target all`

## 상태

- stage25 구현 및 집중 검증 완료.

## 원인

- `pi=587` 수식-only line은 page-path compact 미주 하단 tail로 분류되어 저장 `vpos`를 따랐다.
- 기존 backtrack은 frame 안착에는 유리했지만 이전 텍스트 line의 실제 하단보다 위로 올라갈 수 있어 `pi=586` 본문 line과 `pi=587` 수식 line이 겹쳤다.
- 단순히 `pi=587`을 아래로만 밀면 다음 `pi=588` 수식 tail이 `issue_1274_2022_oct_page11_question20_equation_tail_stays_in_frame` 기준을 벗어나 frame 아래로 내려갔다.

## 수정 내용

- `src/renderer/height_cursor.rs`
  - page-path compact 미주 하단 tail backtrack 결과가 이전 line 하단보다 위로 올라가지 않도록 클램프했다.
  - tall inline 뒤의 일반 텍스트 line은 저장 `vpos`가 안전한 위치를 가리키면 이전 콘텐츠 하단까지 당겨 뒤 수식 line 공간을 확보한다.
  - 수식-only tail fit은 page-path compact 하단에서 저장 `end_y`가 이전 추정 하단보다 몇 px 위인 경우를 허용하고, frame 안착을 우선하되 이전 line과 과도하게 겹치지 않도록 4px tolerance를 둔다.
- `scripts/task1274_visual_sweep.py`
  - stage24에서 추가한 `line_order_overlap` 지표로 stage25 수정 전/후 문20 후보 소거 여부를 확인했다.

## 검증 결과

- `cargo test --lib compact_endnote_page_tail -- --nocapture`
  - `compact_endnote_page_tail_backtrack_keeps_previous_content_bottom`: 통과
  - `compact_endnote_page_tail_text_after_tall_line_backtracks_to_previous_bottom`: 통과
- `cargo build --bin rhwp`: 통과
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`: 통과
  - `analysis: 2022-10 ... frame=[] ... title=[] order=[]`
  - 11쪽 `line_order_overlap_candidates=[]`
  - 문20 직접 겹침 후보였던 `pi=586`/`pi=587` 후보가 사라졌다.
  - 11쪽 render tree:
    - `pi=586` bbox y=`1029.1`, h=`12.0`
    - `pi=587` bbox y=`1041.1`, h=`28.8`
    - `pi=588` bbox y=`1065.9`, h=`28.8`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1274_2022_oct_page11_question20_equation_tail_stays_in_frame -- --nocapture`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1189_2022_oct_page11_endnote_question_gaps_match_pdf -- --nocapture`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과
  - 51개 테스트 모두 통과.
- `python3 scripts/task1274_visual_sweep.py --target all`: 통과
  - `2022-09`: SVG/PDF/render tree 23/23/23쪽, `order=[]`, `frame=[]`
  - `2023-09`: SVG/PDF/render tree 20/20/20쪽, `order=[]`, 기존 full sweep 후보 `frame=[19]`
  - `2024-09-below20`: SVG/PDF/render tree 23/23/23쪽, `order=[]`, `frame=[]`
  - `2024-09-between20`: SVG/PDF/render tree 24/24/24쪽, `order=[]`, 기존 full sweep 후보 `frame=[12]`
  - `2022-10`: SVG/PDF/render tree 18/18/18쪽, `order=[]`, `frame=[]`
  - `2022-11-practice`: SVG/PDF/render tree 21/21/21쪽, `order=[]`, 기존 full sweep 후보 `frame=[12, 19]`
  - stage24에서 잡혔던 line-order 후보(`2022-10` 9쪽/11쪽, `2022-11-practice` 18쪽)는 stage25 공통 보정 후 모두 소거됐다.

## 산출물

- `output/task1274/2022-10/compare/compare_011.png`
- `output/task1274/2022-10/analysis/annotated_011.png`
- `output/task1274/2022-10/render_tree/render_tree_011.json`
