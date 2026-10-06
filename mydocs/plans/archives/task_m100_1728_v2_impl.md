# Task #1728 v2 구현계획서 — continuation 상단 space-before 복원

## 단계 1 — 엔진 토글 + spacing_before 게이트
- `src/renderer/layout.rs`: 엔진 struct 에 `keep_continuation_column_top_spacing_before:
  Cell<bool>` 추가(기본 false), 생성자 초기화. (기존 `use_hwp3_origin_flow_spacing_before`
  관용과 동일)
- `src/renderer/layout/paragraph_layout.rs:1564`: column-top spacing_before 블록에서
  `!is_column_top || keep_continuation_spacing_before` 이면 전량 적용하도록 게이트 확장.

## 단계 2 — table_partial continuation 첫 문단 토글
- `src/renderer/layout/table_partial.rs:967` 호출 직전:
  `keep_spacing = cut_units.su>0 && !has_preceding_text && start_line==0
   && !(row_count==1 && col_count==1)` 일 때 토글 set, 호출 직후 reset.
- **1×1 linear 셀 제외(검증 중 발견)**: page-spanning 1×1 컨테이너 셀
  (`preserve_linear_single_cell_vpos` 계열)의 continuation 은 자연 흐름으로 이미 정합하고
  textbox/shape 를 품을 수 있어, spacing 추가 시 shape 가 표 프레임 밖으로 밀려
  `issue_rowbreak_chart_overlap::rowbreak_page17_split_table_covers_visible_textbox_shape`
  회귀(table=[..1015.17], rect bottom 1021.06). giant 는 5×1 이라 대상, page17 은 1×1 이라 제외.

## 단계 3 — 검증
- **재측정 (96 DPI, vs 한글 2024 PDF)**: 아래 표.
- **페이지 수 불변**: giant 42p 유지(높이 모델 desync/cascade 없음 확인).
- **scattered 무영향**: inter-row continuation(cp_idx==0)은 게이트 밖 → 불변 확인.
- **전체 `cargo test`** + `svg_snapshot`(ktx/aift/table_text 등 골든).

### 측정 결과
| 페이지 | Δtop 수정 전 | Δtop 수정 후 | 비고 |
|--------|-----------:|-----------:|------|
| giant p5 | −18 | **−4** | 목표 달성 |
| giant p6 | −18 | **−4** | 목표 달성 |
| scattered p5 | −2 | −2 | 불변(무영향) |
| scattered p6 top | −2 | −2 | 불변 |

- 잔차 −4px = 전역 오프셋(scattered −2 와 동류) + pad/폰트 메트릭 잔차. 추가 튜닝은
  회귀 위험 대비 이득 미미하여 보류.
- spacing_before=1000 HU(13.3px) 복원이 −18→−4 개선의 실체.

## 비범위 재확인
- scattered p6 하단 +13px over-fill(#1718 grace) 는 본 v2 미포함. 별도 후속 이슈로.
