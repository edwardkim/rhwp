# task 1293 stage82 - 현재 sweep 기준 미주 흐름 회귀 보정

## 목적

stage81 이후 `3-09월_교육_통합_2024-미주사이20.hwp`에서 남은 미주 흐름 후보를
현재 sweep 산출물과 compare PNG 시각 판단 기준으로 다시 확인했다. 직전 커밋 대비
비교는 판단 기준으로 삼지 않았다.

## 수정 내용

### 13~14쪽 문18 흐름

- 13쪽 하단에는 PDF처럼 문18 제목 tail만 남고, 문18 첫 풀이 수식 `pi=713`은
  14쪽 첫머리에서 시작하도록 회귀 테스트를 보강했다.
- `issue_1284_2024_between20_page13_question_flow_matches_pdf`에 page14 dump 확인을
  추가해, 문18 본문이 13쪽 frame 아래에 억지로 남지 않도록 감시한다.

### 17쪽 문26 tail split

- 17쪽 왼쪽 단 하단에서 문26 `(ⅲ)` 첫 두 줄만 남는 후보를 확인했다.
- pagination 상으로 split head가 들어갈 수 있어도, 저장 vpos를 적용한 실제 render 위치가
  frame 하단을 넘으면 한컴/PDF처럼 문단 전체를 다음 단에서 시작하도록 `typeset.rs`에
  `large_between_split_head_render_overflows` 가드를 추가했다.
- `issue_1284_2024_between20_page17_question26_tail_starts_next_column`을 추가해
  `pi=872`가 왼쪽 단 하단 partial로 남지 않고 오른쪽 단 상단에서 시작하는지 확인한다.

### 15쪽 문22 그래프 잘림

- 15쪽 문22에서 본문 중간 저장 vpos가 순차 y보다 약 한 미주 간격 이상 앞으로 튀어,
  첫 그래프 `pi=795`가 frame 하단에서 잘리던 문제를 확인했다.
- 큰 `미주 사이` 문서의 visible body에서 발생하는 stale forward jump는 순차 흐름을
  유지하고 lazy base만 보정하도록 `height_cursor.rs`에
  `compact_endnote_large_gap_body_stale_forward`를 추가했다.
- `issue_1284_2024_between20_page15_question22_graph_stays_in_frame`을 추가해
  문22 그래프 안내 문장과 첫 그래프가 PDF 근처 위치에서 frame 안에 남는지 확인한다.

## 검증

### 자동 검증

- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture`
  - 7개 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1256_2022_sep_page10_question12_keeps_between_notes_gap -- --nocapture`
  - 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1261_2024_sep_page10_question8_stays_below_previous_equation -- --nocapture`
  - 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2022_oct_page15_question28_formula_does_not_overlap_case_label -- --nocapture`
  - 통과
- `cargo build`
  - 통과

### sweep

명령:

```bash
python3 scripts/task1274_visual_sweep.py --target 2024-09-between20 --out output/task1293_stage82_between20_after_body_stale --rhwp-bin target/debug/rhwp
```

결과:

```text
analysis: 2024-09-between20 flagged=1/24 frame=[] red=[] qflow=[] line=[11] column=[11] sep=[] eq=[] title=[] order=[11] tail=[] question=[] large=[11]
```

- SVG/PDF/render-tree 페이지 수는 모두 24쪽이다.
- 13쪽, 14쪽, 15쪽, 17쪽은 수정 대상 후보 flag가 사라졌다.
- 11쪽은 `line_order_overlap`, `line_band_drift`, `column_line_band_drift`,
  `large_ink_region_drift` 후보가 남았다. `rhwp_outside_frame_pixels=0`,
  `pdf_outside_frame_pixels=0`, `content_bottom_delta_px=6.0`이고 compare PNG 기준 실제
  하단 overflow나 문항/수식 겹침은 보이지 않는다. 도형/수식 덩어리를 band 단위로
  매칭하면서 남은 sweep 휴리스틱 후보로 본다.

### 시각 확인 산출물

- 13쪽: `output/task1293_stage82_between20_after_body_stale/2024-09-between20/compare/compare_013.png`
  - 문18 제목 tail만 13쪽 하단에 남는다.
- 14쪽: `output/task1293_stage82_between20_after_body_stale/2024-09-between20/compare/compare_014.png`
  - 문18 첫 풀이 수식이 14쪽 상단에서 이어진다.
- 15쪽: `output/task1293_stage82_between20_after_body_stale/2024-09-between20/compare/compare_015.png`
  - 문22 첫 그래프가 frame 하단에서 잘리지 않는다.
- 17쪽: `output/task1293_stage82_between20_after_body_stale/2024-09-between20/compare/compare_017.png`
  - 문26 `(ⅲ)` tail이 왼쪽 단 아래에 걸리지 않고 오른쪽 단에서 시작한다.
- 11쪽: `output/task1293_stage82_between20_after_body_stale/2024-09-between20/analysis/annotated_011.png`
  - 남은 후보 위치를 확인했으나 실제 overflow나 제목/본문 겹침은 보이지 않는다.

## 판단

현재 stage82 변경은 page13/14 문18 흐름, page15 문22 그래프 잘림, page17 문26 tail split
회귀를 공통 미주 흐름 규칙으로 보정했다. 남은 page11 sweep 후보는 현 시점에서 실제
오버플로우가 아닌 자동 탐지 휴리스틱 후보로 기록하고, 추가 수정 대상으로 확정하지 않는다.

PR CI 전체 테스트는 작업지시자 별도 승인 전에는 수행하지 않았다.
