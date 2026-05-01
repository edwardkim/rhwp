# Task #429 Stage 1 — 베이스라인 측정

## 결함 정황 정합

`samples/aift.hwp` 페이지 41 의 표 pi=496 (2x1, RowBreak, TAC):

| 셀 | r,c | bf_id | fill_type | 정합 |
|----|-----|-------|-----------|------|
| 셀[0] | 0,0 | **175** | **Image (bin_id=14)** | ❌ rhwp 미렌더 (image fill 미구현) |
| 셀[1] | 1,0 | 1 | Solid (#FFFFFF) | ✅ 정상 (캡션 텍스트) |

**SVG 출력 (정정 전)**: `<image>` 갯수 = **0** (셀 배경 그림 누락).

## image fill BorderFill 정합

aift.hwp 의 image fill BorderFill: `bf=174 (bin_id=13)`, `bf=175 (bin_id=14)`, `bf=176 (bin_id=15)` — 3개.

다른 샘플 (synam-001, k-water-rfp, exam_kor, 2022 국립국어원) 의 image fill BorderFill: 0개. 본 결함은 aift.hwp 에 집중.

## 정정 영역 호출부 점검

| # | 파일 | 라인 | bin_data_content 사용 가능 |
|---|------|------|--------------------------|
| 1 | `table_layout.rs` | 264 (표 전체 배경) | ✅ |
| 2 | `table_layout.rs` | 301 (zone 배경) | ✅ — image_fill 별도 처리 (제거 대상) |
| 3 | `table_layout.rs` | 1154 (일반 셀) | ✅ |
| 4 | `table_partial.rs` | 240 (분할 표 전체 배경) | ✅ |
| 5 | `table_partial.rs` | 340 (분할 표 셀) | ✅ |
| 6 | `table_cell_content.rs` | 472 (표 배경 — 추가 발견) | ✅ |

총 **6개소** 정정 (Stage 2 에서 적용).

## 다음 단계

Stage 2: `render_cell_background` 시그니처 확장 + image_fill 분기 추가 + 호출부 6개소 정정.
