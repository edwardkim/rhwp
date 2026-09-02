# skip 정직 표

각 행은 `apply_existing_step` 이 skip 하는 이유와 1:1 이다.
새 편집 API 로 skip 을 메우지 않는다.

| fixture | action | reason | expected |
|---|---|---|---|
| `ref_text_hwpx` | `fill_fields` | `field_missing` | skip |
| `ref_text_hwpx` | `replace_text` | `no_hits` | skip |
| `ref_text_hwpx` | `replace_text` | `occurrence_oob` | skip |
| `ref_text_hwpx` | `replace_text` | `empty_find` | skip |
| `ref_text_hwpx` | `set_cell` | `table_missing` | skip |
| `ref_text_hwpx` | `set_cell` | `nested_table` | skip_if_nested |
| `ref_text_hwpx` | `set_checkbox` | `checkbox_missing` | skip |
| `ref_text_hwpx` | `*` | `all_steps_skipped` | reject |
| `ref_table_hwpx` | `fill_fields` | `field_missing` | skip |
| `ref_table_hwpx` | `replace_text` | `no_hits` | skip |
| `ref_table_hwpx` | `replace_text` | `empty_find` | skip |
| `ref_table_hwpx` | `set_cell` | `table_missing` | skip |
| `ref_table_hwpx` | `set_cell` | `cell_missing` | skip |
| `ref_table_hwpx` | `set_cell` | `cell_control_char` | skip |
| `ref_table_hwpx` | `set_cell` | `nested_table` | skip_if_nested |
| `ref_table_hwpx` | `set_checkbox` | `checkbox_missing` | skip |
| `ref_table_hwpx` | `*` | `all_steps_skipped` | reject |
| `para001_hwp5` | `fill_fields` | `field_missing` | skip |
| `para001_hwp5` | `replace_text` | `no_hits` | skip |
| `para001_hwp5` | `replace_text` | `occurrence_oob` | skip |
| `para001_hwp5` | `replace_text` | `empty_find` | skip |
| `para001_hwp5` | `set_cell` | `table_missing` | skip |
| `para001_hwp5` | `set_cell` | `nested_table` | skip_if_nested |
| `para001_hwp5` | `set_checkbox` | `checkbox_missing` | skip |
| `para001_hwp5` | `*` | `all_steps_skipped` | reject |
| `table001_hwp5` | `fill_fields` | `field_missing` | skip |
| `table001_hwp5` | `replace_text` | `no_hits` | skip |
| `table001_hwp5` | `replace_text` | `occurrence_oob` | skip |
| `table001_hwp5` | `replace_text` | `empty_find` | skip |
| `table001_hwp5` | `set_cell` | `table_missing` | skip |
| `table001_hwp5` | `set_cell` | `cell_missing` | skip |
| `table001_hwp5` | `set_cell` | `cell_control_char` | skip |
| `table001_hwp5` | `set_cell` | `nested_table` | skip_if_nested |
| `table001_hwp5` | `set_checkbox` | `checkbox_missing` | skip |
| `table001_hwp5` | `*` | `all_steps_skipped` | reject |
| `ref_empty_hwpx` | `fill_fields` | `field_missing` | skip |
| `ref_empty_hwpx` | `replace_text` | `no_hits` | skip |
| `ref_empty_hwpx` | `replace_text` | `empty_find` | skip |
| `ref_empty_hwpx` | `set_cell` | `table_missing` | skip |
| `ref_empty_hwpx` | `set_cell` | `nested_table` | skip_if_nested |
| `ref_empty_hwpx` | `set_checkbox` | `checkbox_missing` | skip |
| `ref_empty_hwpx` | `*` | `all_steps_skipped` | reject |
| `ref_mixed_hwpx` | `fill_fields` | `unclaimed_capability` | skip |
| `ref_mixed_hwpx` | `replace_text` | `unclaimed_capability` | skip |
| `ref_mixed_hwpx` | `set_cell` | `unclaimed_capability` | skip |
| `ref_mixed_hwpx` | `set_checkbox` | `unclaimed_capability` | skip |
