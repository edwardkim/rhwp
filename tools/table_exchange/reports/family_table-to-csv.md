# table-to-csv 케이스

36건. 기존 CLI 계약만.

| id | sample | table | size | exit | writes | next |
|---|---|---:|---|---:|---|---|
| T-block_2x2-bom | synthetic/merge-pattern.hwpx | 0 | 4×4 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-block_2x2-extract | synthetic/merge-pattern.hwpx | 0 | 4×4 | 0 | yes | edit set-cell |
| T-chujin-bom | samples/추진일정.hwp | 0 | 4×3 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-chujin-extract | samples/추진일정.hwp | 0 | 4×3 | 0 | yes | 외부 편집 후 csv-to-table --dry-run |
| T-colspan2_r0c0-bom | synthetic/merge-pattern.hwpx | 0 | 3×4 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-colspan2_r0c0-extract | synthetic/merge-pattern.hwpx | 0 | 3×4 | 0 | yes | edit set-cell |
| T-header_plus_note-bom | synthetic/merge-pattern.hwpx | 0 | 4×5 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-header_plus_note-extract | synthetic/merge-pattern.hwpx | 0 | 4×5 | 0 | yes | edit set-cell |
| T-hwp_table_test_t0-bom | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-hwp_table_test_t0-extract | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | yes | 외부 편집 후 csv-to-table --dry-run |
| T-hwpx_basic_01-bom | samples/hwpx/basic-table-01.hwpx | 0 | 2×2 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-hwpx_basic_01-extract | samples/hwpx/basic-table-01.hwpx | 0 | 2×2 | 0 | yes | 외부 편집 후 csv-to-table --dry-run |
| T-issue2007_t0-bom | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 0 | 2×1 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-issue2007_t0-extract | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 0 | 2×1 | 0 | yes | 외부 편집 후 csv-to-table --dry-run |
| T-issue2007_t1-bom | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 1 | 2×3 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-issue2007_t1-extract | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 1 | 2×3 | 0 | yes | 외부 편집 후 csv-to-table --dry-run |
| T-jichi_body_12-bom | samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 12 | 5×4 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-jichi_body_12-extract | samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 12 | 5×4 | 0 | yes | 외부 편집 후 csv-to-table --dry-run |
| T-many_small-bom | synthetic/merge-pattern.hwpx | 0 | 8×6 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-many_small-extract | synthetic/merge-pattern.hwpx | 0 | 8×6 | 0 | yes | edit set-cell |
| T-multi-folder | samples/multi-table-001.hwp | 0 | 2×2 | 0 | yes | export-tables 로 index 확인 후 하나만 csv-to-table |
| T-recipe02-rfc4180 | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | no | 판독기가 "" 를 한 따옴표로 되돌리는지 본다 |
| T-rowspan3_note-bom | synthetic/merge-pattern.hwpx | 0 | 5×4 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-rowspan3_note-extract | synthetic/merge-pattern.hwpx | 0 | 5×4 | 0 | yes | edit set-cell |
| T-shape_3x4_m0-bom | synthetic/shape_3x4_m0.hwp | 0 | 3×4 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-shape_3x4_m0-extract | synthetic/shape_3x4_m0.hwp | 0 | 3×4 | 0 | yes | 외부 편집 후 csv-to-table --dry-run |
| T-shape_5x5_m0-bom | synthetic/shape_5x5_m0.hwp | 0 | 5×5 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-shape_5x5_m0-extract | synthetic/shape_5x5_m0.hwp | 0 | 5×5 | 0 | yes | 외부 편집 후 csv-to-table --dry-run |
| T-table001_header-bom | samples/table-001.hwp | 0 | 19×9 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-table001_header-extract | samples/table-001.hwp | 0 | 19×9 | 0 | yes | edit set-cell |
| T-table_001-bom | samples/table-001.hwp | 0 | 19×9 | 0 | yes | 엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라. |
| T-table_001-extract | samples/table-001.hwp | 0 | 19×9 | 0 | yes | edit set-cell |
| T-unknown-table-99999 | samples/hwp_table_test.hwp | 99999 | 0×0 | 1 | no | export-tables --json |
| T-usage-csv-no-args | samples/hwp_table_test.hwp | 0 | 0×0 | 2 | no | 플래그를 고친다 |
| T-usage-csv-no-csv-flag | samples/hwp_table_test.hwp | 0 | 0×0 | 2 | no | 플래그를 고친다 |
| T-usage-no-file | samples/hwp_table_test.hwp | 0 | 0×0 | 2 | no | 플래그를 고친다 |
