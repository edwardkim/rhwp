# Task m100-2901: HWPX 표 row_sizes 계약 위반 수정

- 이슈: #3060
- 브랜치: `task/m100-2901-table-cnt-attrs`

## 문제

`src/parser/hwpx/section.rs`의 `parse_table`이 `Table::row_sizes`를 "행별 최대 셀 높이"로
채우고 있었다. 그러나 이 필드의 실제 계약은 "행별 셀 수"(HWP5 스펙 `UINT16[NRows]`)이며,
다음 모든 생산자가 이 계약을 따른다.

- `src/parser/control.rs:274` (HWP5 네이티브 파서, 스펙 주석 명시)
- `src/model/table.rs::rebuild_row_sizes()`
- `src/document_core/html_table_import.rs:538`
- `src/document_core/commands/object_ops/table.rs` (신규 표 생성)
- `src/document_core/converters/hwpx_to_hwp.rs::materialize_table_record_row_sizes`
  (HWPX→HWP 저장 직전, "행별 셀 수"로 강제 재계산 — 이 안전망 때문에 HWP 변환 경로는
  증상이 가려져 있었다)

HWPX 파서만 유일하게 "높이"를 채워, 순수 HWPX IR을 그대로 소비하는 경로
(예: 렌더러의 cellzone 폴백 계산)에서 필드 의미가 어긋난다.

## 수정

`parse_table` 말미의 row_sizes 계산을 "행별 셀 수" 카운트로 통일했다 (다른 생산자와
동일한 `(0..row_count).map(|r| cells.iter().filter(|c| c.row == r).count())` 패턴).

## 검증

- `cargo check --lib`: 통과
- 신규 테스트 `test_parse_table_row_sizes_is_cell_count_not_height` (2행×2열, 행별 높이를
  다르게 주어 카운트(2,2)와 높이가 혼동되지 않음을 확인): 통과
- `cargo test --lib table` (332개 표 관련 테스트, hwpx_to_hwp 어댑터 회귀 포함): 전부 통과
