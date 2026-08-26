# verify 케이스

31건. 기존 CLI 계약만.

| id | sample | table | size | exit | writes | next |
|---|---|---:|---|---:|---|---|
| V-chujin-exit3-diff3 | samples/추진일정.hwp | 0 | 4×3 | 3 | yes | csv-to-table --verify |
| V-chujin-identical | samples/추진일정.hwp | 0 | 4×3 | 0 | yes | csv-to-table --verify |
| V-chujin-write-no-verify-flag | samples/추진일정.hwp | 0 | 4×3 | 0 | yes | csv-to-table --verify |
| V-chujin-write-ok | samples/추진일정.hwp | 0 | 4×3 | 0 | yes | csv-to-table --verify |
| V-hwp_table_test_t0-exit3-diff2 | samples/hwp_table_test.hwp | 0 | 4×3 | 3 | yes | csv-to-table --verify |
| V-hwp_table_test_t0-identical | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | yes | csv-to-table --verify |
| V-hwp_table_test_t0-write-no-verify-flag | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | yes | csv-to-table --verify |
| V-hwp_table_test_t0-write-ok | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | yes | csv-to-table --verify |
| V-hwpx_basic_01-exit3-diff1 | samples/hwpx/basic-table-01.hwpx | 0 | 2×2 | 3 | yes | csv-to-table --verify |
| V-hwpx_basic_01-identical | samples/hwpx/basic-table-01.hwpx | 0 | 2×2 | 0 | yes | csv-to-table --verify |
| V-hwpx_basic_01-write-no-verify-flag | samples/hwpx/basic-table-01.hwpx | 0 | 2×2 | 0 | yes | csv-to-table --verify |
| V-hwpx_basic_01-write-ok | samples/hwpx/basic-table-01.hwpx | 0 | 2×2 | 0 | yes | csv-to-table --verify |
| V-issue2007_t1-exit3-diff1 | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 1 | 2×3 | 3 | yes | csv-to-table --verify |
| V-issue2007_t1-identical | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 1 | 2×3 | 0 | yes | csv-to-table --verify |
| V-issue2007_t1-write-no-verify-flag | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 1 | 2×3 | 0 | yes | csv-to-table --verify |
| V-issue2007_t1-write-ok | samples/basic/issue2007_nested_cell_pagination_42065.hwp | 1 | 2×3 | 0 | yes | csv-to-table --verify |
| V-jichi_body_12-exit3-diff2 | samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 12 | 5×4 | 3 | yes | csv-to-table --verify |
| V-jichi_body_12-identical | samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 12 | 5×4 | 0 | yes | csv-to-table --verify |
| V-jichi_body_12-write-no-verify-flag | samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 12 | 5×4 | 0 | yes | csv-to-table --verify |
| V-jichi_body_12-write-ok | samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 12 | 5×4 | 0 | yes | csv-to-table --verify |
| V-recipe02-verify-ok | samples/hwp_table_test.hwp | 0 | 4×3 | 0 | yes | csv-to-table --verify |
| V-shape_3x4_m0-identical | synthetic/shape_3x4_m0.hwp | 0 | 3×4 | 0 | yes | csv-to-table --verify |
| V-shape_3x4_m0-write-no-verify-flag | synthetic/shape_3x4_m0.hwp | 0 | 3×4 | 0 | yes | csv-to-table --verify |
| V-shape_3x4_m0-write-ok | synthetic/shape_3x4_m0.hwp | 0 | 3×4 | 0 | yes | csv-to-table --verify |
| V-shape_5x5_m0-exit3-diff4 | synthetic/shape_5x5_m0.hwp | 0 | 5×5 | 3 | yes | csv-to-table --verify |
| V-shape_5x5_m0-identical | synthetic/shape_5x5_m0.hwp | 0 | 5×5 | 0 | yes | csv-to-table --verify |
| V-shape_5x5_m0-write-no-verify-flag | synthetic/shape_5x5_m0.hwp | 0 | 5×5 | 0 | yes | csv-to-table --verify |
| V-shape_5x5_m0-write-ok | synthetic/shape_5x5_m0.hwp | 0 | 5×5 | 0 | yes | csv-to-table --verify |
| V-shape_8x4_m0-identical | synthetic/shape_8x4_m0.hwp | 0 | 8×4 | 0 | yes | csv-to-table --verify |
| V-shape_8x4_m0-write-no-verify-flag | synthetic/shape_8x4_m0.hwp | 0 | 8×4 | 0 | yes | csv-to-table --verify |
| V-shape_8x4_m0-write-ok | synthetic/shape_8x4_m0.hwp | 0 | 8×4 | 0 | yes | csv-to-table --verify |
