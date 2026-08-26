# dry-run 케이스

24건. 기존 CLI 계약만.

| id | sample | table | size | exit | writes | next |
|---|---|---:|---|---:|---|---|
| R-chujin-ctrl-tab | samples/추진일정.hwp | 0 | 4×3 | 2 | no | LF/TAB 을 공백으로 치환 후 --dry-run |
| R-chujin-identical | samples/추진일정.hwp | 0 | 4×3 | 0 | no | csv-to-table --verify |
| R-chujin-preview | samples/추진일정.hwp | 0 | 4×3 | 0 | no | csv-to-table --verify |
| R-hwp_table_test_t0-ctrl-lf | samples/hwp_table_test.hwp | 0 | 4×3 | 2 | no | LF/TAB 을 공백으로 치환 후 --dry-run |
| R-hwp_table_test_t0-ctrl-tab | samples/hwp_table_test.hwp | 0 | 4×3 | 2 | no | LF/TAB 을 공백으로 치환 후 --dry-run |
| R-hwp_table_test_t0-identical | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | no | csv-to-table --verify |
| R-hwp_table_test_t0-preview | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | no | csv-to-table --verify |
| R-hwpx_basic_01-identical | samples/hwpx/basic-table-01.hwpx | 0 | 2×2 | 0 | no | csv-to-table --verify |
| R-hwpx_basic_01-preview | samples/hwpx/basic-table-01.hwpx | 0 | 2×2 | 0 | no | csv-to-table --verify |
| R-issue2007_t1-ctrl-lf | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 1 | 2×3 | 2 | no | LF/TAB 을 공백으로 치환 후 --dry-run |
| R-issue2007_t1-identical | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 1 | 2×3 | 0 | no | csv-to-table --verify |
| R-issue2007_t1-preview | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 1 | 2×3 | 0 | no | csv-to-table --verify |
| R-jichi_body_12-identical | samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 12 | 5×4 | 0 | no | csv-to-table --verify |
| R-jichi_body_12-preview | samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 12 | 5×4 | 0 | no | csv-to-table --verify |
| R-recipe02-edited | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | no | csv-to-table --verify |
| R-shape_3x4_m0-ctrl-crlf_in_quotes | synthetic/shape_3x4_m0.hwp | 0 | 3×4 | 2 | no | LF/TAB 을 공백으로 치환 후 --dry-run |
| R-shape_3x4_m0-identical | synthetic/shape_3x4_m0.hwp | 0 | 3×4 | 0 | no | csv-to-table --verify |
| R-shape_3x4_m0-preview | synthetic/shape_3x4_m0.hwp | 0 | 3×4 | 0 | no | csv-to-table --verify |
| R-shape_5x5_m0-identical | synthetic/shape_5x5_m0.hwp | 0 | 5×5 | 0 | no | csv-to-table --verify |
| R-shape_5x5_m0-preview | synthetic/shape_5x5_m0.hwp | 0 | 5×5 | 0 | no | csv-to-table --verify |
| R-shape_8x4_m0-identical | synthetic/shape_8x4_m0.hwp | 0 | 8×4 | 0 | no | csv-to-table --verify |
| R-shape_8x4_m0-preview | synthetic/shape_8x4_m0.hwp | 0 | 8×4 | 0 | no | csv-to-table --verify |
| R-shape_9x2_m0-identical | synthetic/shape_9x2_m0.hwp | 0 | 9×2 | 0 | no | csv-to-table --verify |
| R-shape_9x2_m0-preview | synthetic/shape_9x2_m0.hwp | 0 | 9×2 | 0 | no | csv-to-table --verify |
