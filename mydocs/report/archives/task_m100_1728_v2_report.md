# Task #1728 v2 최종 보고서 — RowBreak 셀-내 continuation 상단 space-before 복원

## 요약
#1728(부분) 보고서가 남긴 갈래 **"continuation 상단 space-before 트림"**을 수정.
거대 셀(5×1 표의 row 4, 654문단)이 페이지 분할될 때 continuation 페이지 첫 heading
문단의 앞 간격(spacing_before=1000 HU=13.3px)이 트림되어 본문이 18px 위로 뜨던 문제를
복원. **giant p5/p6 Δtop −18px → −4px.**

## 근본 원인
- `layout_composed_paragraph`(`paragraph_layout.rs:1564`)는 column-top 문단 중
  `para_index>0` 인 것의 spacing_before 를 전량 트림한다(Task #853, 본문 page-break 정합).
- 5×1 표의 거대 셀이 **셀-내(intra-cell) 분할**되면 continuation 첫 문단이 셀-상단이면서
  셀-상대 인덱스>0 → spacing_before 가 잘못 트림된다. 한컴 PDF 는 유지.
- scattered(260×9)는 **행 단위(inter-row) 분할**이라 셀 첫 문단 `cp_idx==0` → 정상.

## 수정 (`src/renderer/`)
1. `layout.rs`: 엔진에 `keep_continuation_column_top_spacing_before: Cell<bool>` 추가
   (기존 `use_hwp3_origin_flow_spacing_before` 관용).
2. `paragraph_layout.rs`: column-top spacing_before 블록에 토글 우회 게이트 추가
   (`!is_column_top || keep_continuation_spacing_before` → 전량 적용).
3. `table_partial.rs`: continuation 조각(`cut su>0`)의 첫 가시·문단-선두 문단에서 토글 set,
   호출 후 reset. **단 1×1 linear 셀은 제외**.

### 1×1 linear 셀 제외 (검증 중 발견)
page-spanning 1×1 컨테이너 셀(`preserve_linear_single_cell_vpos` 계열)의 continuation 은
자연 흐름으로 이미 정합하고 textbox/shape 를 품을 수 있어, spacing 추가 시 shape 가 표
프레임 밖으로 밀린다 →
`issue_rowbreak_chart_overlap::rowbreak_page17_split_table_covers_visible_textbox_shape`
회귀(table=[..1015.17] < rect bottom 1021.06). 게이트에 `!(row_count==1 && col_count==1)`
추가로 해소. giant(5×1)=대상, page17(1×1)=제외.

## 검증 (96 DPI PNG dark-pixel top vs 한글 2024 PDF)
| 페이지 | Δtop 전 | Δtop 후 | 비고 |
|--------|-------:|-------:|------|
| giant p5 | −18 | **−4** | 목표 달성 |
| giant p6 | −18 | **−4** | 목표 달성 |
| scattered p5/p6 top | −2 | −2 | 불변(무영향) |

- giant 페이지 수 42 불변(높이 모델 desync/cascade 없음).
- **전체 `cargo test`: 2738 passed / 0 failed.** `svg_snapshot` 8/8(골든 불변). `cargo fmt --check` OK.
- 잔차 −4px = 전역 오프셋(scattered −2 동류) + pad/폰트 메트릭 잔차, 추가 튜닝 보류.

## 범위 / 비범위
- **범위**: giant continuation 상단 space-before 복원.
- **비범위**: scattered p6 하단 +13px over-fill(#1718 grace 계열). 회귀 위험이 높아 별도
  후속 이슈로 분리(#1728 원 관찰의 "PDF 하단" 갈래).
