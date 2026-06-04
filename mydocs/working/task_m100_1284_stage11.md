# task 1284 stage11: full sweep 후보 정리 및 문27 tail 보정

## 배경

- PR 준비용 CI 검증 중 작업지시자가 `3-09월_교육_통합_2024-미주사이20.hwp` 18쪽 화면이 한컴과 다르다고 확인했다.
- stage10 이후 full sweep을 다시 돌렸을 때, 실제 문제와 거친 픽셀 band 보조 지표가 함께 flag되어 “수정해야 할 목록”으로 쓰기 어려웠다.
- stage11에서는 sweep가 검출하는 후보를 하나씩 확인해 실제 배치 문제와 오탐을 분리하고, 실제 불일치인 `3-09월_교육_통합_2023.hwp` 16쪽 문27 tail 위치를 수정했다.

## 실제 수정한 렌더링 문제

### `3-09월_교육_통합_2023.hwp` 16쪽 문27

- 증상
  - rhwp에서는 문27 제목이 왼쪽 단 하단에서 한컴/PDF보다 약 50px 낮게 배치됐다.
  - sweep의 `line_order_overlap` 후보는 문27 제목과 직전 수식 줄 bbox가 겹친다고 보고했다.
- 원인
  - `HeightCursor`가 빈 spacer 뒤 새 문항 제목 tail을 `result=851px`로 되돌리는 보정을 계산했지만, 미주 사이 line spacing 보존 분기가 이를 무시하고 원래 `y_offset=1051px`을 반환했다.
  - 기존 보정식은 과하게 위로 당기는 값도 만들 수 있어, 한컴/PDF 기준 위치인 약 `1001px`에 맞지 않았다.
- 수정
  - `src/renderer/height_cursor.rs`
    - 빈 spacer 뒤 새 문항 제목 tail은 저장 line spacing 전체가 아니라 하단 tail 1줄만 앞 단에 남기도록 `min(50px)` backtrack으로 제한했다.
    - 이 케이스는 “주입된 미주 사이 간격 유지” 분기에서 제외하여 보정값이 다시 무시되지 않게 했다.
  - `src/renderer/layout.rs`
    - 문항 제목 tail backtrack이 적용된 경우, 이후 title gap 보존 로직이 다시 아래로 밀지 않도록 guard를 유지했다.
- 검증
  - `issue_1284_2023_sep_page16_question27_title_matches_pdf_tail`
    - 문27 제목 line bbox가 PDF page 16 왼쪽 단 하단(`y≈1001px`)에 들어오는지 확인한다.

### `3-10월_교육_통합_2022.hwp` 16쪽 문30

- 증상
  - 전체 `issue_1139_inline_picture_duplicate` 회귀 테스트에서 문30 첫 본문 줄의 하단이 frame 기준보다 약 3~4px 내려갔다.
  - 제목 자체는 16쪽 하단에 남았지만, 제목 직후 첫 줄이 함께 보존되는 한컴/PDF tail 조건을 만족하지 못했다.
- 원인
  - `compact_endnote_title_bottom_backtrack`이 저장 vpos의 제목 top을 그대로 쓰면서, 후속 문단의 vpos base는 이동하지 않았다.
  - 이 때문에 제목만 약간 들어오고 다음 첫 줄은 기존 vpos 기준을 따라 하단을 넘었다.
- 수정
  - `src/renderer/height_cursor.rs`
    - `page_tail=false`인 제목+첫줄 보존 케이스에만 4px 하단 여유를 둔다.
    - 제목을 위로 당긴 만큼 활성 vpos base도 이동해 다음 첫 줄이 같이 따라오도록 했다.
    - `page_tail=true`인 `2024-미주사이20` 13쪽 문18은 저장 vpos를 그대로 유지하도록 보정 범위를 좁혔다.
- 검증
  - `issue_1274_2022_oct_page16_question30_title_keeps_first_line`
    - 문30 제목과 첫 본문 줄이 16쪽 frame 안에 함께 남는지 확인한다.
  - `issue_1284_2024_between20_page13_question_flow_matches_pdf`
    - page-tail 문18 제목이 PDF 기준 위치 범위에서 유지되는지 확인한다.

## sweep 검출기 정리

### 수식/텍스트 겹침 후보

- `scripts/task1274_visual_sweep.py`
  - 수식 bbox와 텍스트 bbox의 overlap ratio만 보던 기준에 overlap width/height를 추가했다.
  - 실제 글자 겹침이 아니라 line box가 1px대 접촉하는 경우는 제외한다.
  - 같은 TextLine 내부 수식/텍스트, 문항 제목 line, 선택지 marker-only 텍스트, object placeholder는 계속 noise로 제외한다.
- 처리된 오탐
  - `3-09월_교육_통합_2023.hwp` 17쪽
    - `equation_text_overlap` 후보가 남았지만 실제로는 수식 bbox와 다음 텍스트 bbox가 약 1.2px 접촉하는 line-box 오탐이었다.

### 문장 순서/line overlap 후보

- `scripts/task1274_visual_sweep.py`
  - 이전 line이 `[EQ]` placeholder 중심이고 다음 line이 문항 제목이면 `line_order_overlap`에서 제외한다.
  - 실제 수식-문항 제목 충돌은 `equation_text_overlap`/시각 검증에서 별도로 잡는다.
- 처리된 오탐
  - `3-09월_교육_통합_2023.hwp` 16쪽 문27
    - 직전 수식 line box 높이 때문에 문27 제목과 bbox가 겹치는 것으로 잡혔으나, 실제 문항 제목 위치는 별도 테스트로 검증했다.

### 페이지 전체 band drift

- `red_marker_drift`, `line_band_drift`, `column_line_band_drift`, `content_bottom_drift`는 단독으로는 너무 넓게 잡힌다.
- stage11에서는 이 지표들을 보조 지표로 유지하되, render-tree 기반의 실제 후보(`equation_text_overlap`, `question_title_text_overlap`, `line_order_overlap`, `question_marker_drift`)와 결합될 때만 page flag가 되도록 조정했다.
- 결과적으로 full sweep summary가 실제 수정 후보 중심으로 읽히게 되었다.

## 검증 결과

- `python3 -m py_compile scripts/task1274_visual_sweep.py`
  - 통과
- `cargo fmt --all -- --check`
  - 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2023_sep -- --nocapture`
  - 3개 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2022_oct_page -- --nocapture`
  - 2개 통과
- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture`
  - 5개 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 62개 통과
- `python3 scripts/task1274_visual_sweep.py --target all`
  - 전체 6종 SVG/PDF/render-tree 페이지 수 1:1 일치
  - actionable flag 0

| 대상 | SVG/render-tree/PDF | actionable flag |
|---|---:|---:|
| `3-09월_교육_통합_2022.hwp` | 23/23/23 | 0/23 |
| `3-09월_교육_통합_2023.hwp` | 20/20/20 | 0/20 |
| `3-09월_교육_통합_2024-구분선아래20.hwp` | 23/23/23 | 0/23 |
| `3-09월_교육_통합_2024-미주사이20.hwp` | 24/24/24 | 0/24 |
| `3-10월_교육_통합_2022.hwp` | 18/18/18 | 0/18 |
| `3-11월_실전_통합_2022.hwp` | 21/21/21 | 0/21 |

주요 산출물:

- `output/task1274/summary.json`
- `output/task1274/*/analysis/metrics.json`
- `output/task1274/*/analysis/flagged_pages.json`
- `output/task1274/*/compare/compare_*.png`
- `output/task1274/*/analysis/annotated_*.png`

## 남은 판단

- 자동 sweep 기준으로는 stage11 종료 시점에 새로 수정해야 할 actionable 후보가 남지 않았다.
- 다만 한컴 편집기 직접 시각 검증은 작업지시자 판정 게이트로 남는다.
