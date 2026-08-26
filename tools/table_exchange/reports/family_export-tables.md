# export-tables 케이스

37건. 기존 CLI 계약만.

| id | sample | table | size | exit | writes | next |
|---|---|---:|---|---:|---|---|
| E-missing-file | samples/does-not-exist.hwp | 0 | 0×0 | 1 | no | 경로를 고친다 |
| E-occ-block_2x2 | synthetic/merge-pattern.hwpx | 0 | 4×4 | 0 | no | edit set-cell |
| E-occ-block_2x3 | synthetic/merge-pattern.hwpx | 0 | 5×5 | 0 | no | edit set-cell |
| E-occ-block_3x2 | synthetic/merge-pattern.hwpx | 0 | 5×4 | 0 | no | edit set-cell |
| E-occ-checker_safe | synthetic/merge-pattern.hwpx | 0 | 4×4 | 0 | no | edit set-cell |
| E-occ-colspan2_last_row | synthetic/merge-pattern.hwpx | 0 | 4×3 | 0 | no | edit set-cell |
| E-occ-colspan2_r0c0 | synthetic/merge-pattern.hwpx | 0 | 3×4 | 0 | no | edit set-cell |
| E-occ-colspan2_r0c1 | synthetic/merge-pattern.hwpx | 0 | 3×4 | 0 | no | edit set-cell |
| E-occ-colspan3_header | synthetic/merge-pattern.hwpx | 0 | 4×4 | 0 | no | edit set-cell |
| E-occ-colspan3_mid | synthetic/merge-pattern.hwpx | 0 | 5×5 | 0 | no | edit set-cell |
| E-occ-colspan4_full_row | synthetic/merge-pattern.hwpx | 0 | 3×4 | 0 | no | edit set-cell |
| E-occ-corner_l | synthetic/merge-pattern.hwpx | 0 | 5×5 | 0 | no | edit set-cell |
| E-occ-first_col_stack | synthetic/merge-pattern.hwpx | 0 | 6×3 | 0 | no | edit set-cell |
| E-occ-header_plus_note | synthetic/merge-pattern.hwpx | 0 | 4×5 | 0 | no | edit set-cell |
| E-occ-hwp_table_test_t0 | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | no | table-to-csv --table N |
| E-occ-issue2007_t0 | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 0 | 2×1 | 0 | no | table-to-csv --table N |
| E-occ-issue2007_t1 | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 1 | 2×3 | 0 | no | table-to-csv --table N |
| E-occ-last_cell_span | synthetic/merge-pattern.hwpx | 0 | 3×3 | 0 | no | edit set-cell |
| E-occ-many_small | synthetic/merge-pattern.hwpx | 0 | 8×6 | 0 | no | edit set-cell |
| E-occ-rowspan2_r0c0 | synthetic/merge-pattern.hwpx | 0 | 4×3 | 0 | no | edit set-cell |
| E-occ-rowspan2_r1c2 | synthetic/merge-pattern.hwpx | 0 | 4×3 | 0 | no | edit set-cell |
| E-occ-rowspan3_note | synthetic/merge-pattern.hwpx | 0 | 5×4 | 0 | no | edit set-cell |
| E-occ-table001_header | samples/table-001.hwp | 0 | 19×9 | 0 | no | edit set-cell |
| E-occ-tall_col0 | synthetic/merge-pattern.hwpx | 0 | 8×4 | 0 | no | edit set-cell |
| E-occ-triple_header | synthetic/merge-pattern.hwpx | 0 | 3×6 | 0 | no | edit set-cell |
| E-occ-two_colspan3 | synthetic/merge-pattern.hwpx | 0 | 4×7 | 0 | no | edit set-cell |
| E-occ-wide_row0 | synthetic/merge-pattern.hwpx | 0 | 6×8 | 0 | no | edit set-cell |
| E-scan-chujin | samples/추진일정.hwp | 0 | 4×3 | 0 | no | containerPath 없는 표의 index 로 --table |
| E-scan-hwp_table_test_t0 | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | no | containerPath 없는 표의 index 로 --table |
| E-scan-hwpx_basic_01 | samples/hwpx/basic-table-01.hwpx | 0 | 2×2 | 0 | no | containerPath 없는 표의 index 로 --table |
| E-scan-inner_table_outer | samples/inner-table-01.hwp | 0 | 4×4 | 0 | no | containerPath 없는 표의 index 로 --table |
| E-scan-issue2007_t0 | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 0 | 2×1 | 0 | no | containerPath 없는 표의 index 로 --table |
| E-scan-jichi_header_zero | samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 0 | 2×3 | 0 | no | containerPath 없는 표의 index 로 --table |
| E-scan-table_001 | samples/table-001.hwp | 0 | 19×9 | 0 | no | containerPath 없는 표의 index 로 --table |
| E-scan-treatise_header | samples/basic/treatise sample.hwp | 2 | 2×2 | 0 | no | containerPath 없는 표의 index 로 --table |
| E-scan-wrapper_1x1 | samples/복학원서.hwp | 0 | 1×1 | 0 | no | containerPath 없는 표의 index 로 --table |
| E-usage-two-files | samples/hwp_table_test.hwp | 0 | 0×0 | 2 | no | 인자 조립을 고친다 |
