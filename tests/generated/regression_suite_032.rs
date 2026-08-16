//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_032

#[path = "../agent_codex_contract.rs"]
mod agent_codex_contract;

#[path = "../agent_toolkit_contract.rs"]
mod agent_toolkit_contract;

#[path = "../diag_1042_pi162_attr1.rs"]
mod diag_1042_pi162_attr1;

#[path = "../genpreview_json_contract.rs"]
mod genpreview_json_contract;

#[path = "../info_title_contract.rs"]
mod info_title_contract;

#[path = "../ir_diff_table_cells.rs"]
mod ir_diff_table_cells;

#[path = "../issue_1061_equation_serialize.rs"]
mod issue_1061_equation_serialize;

#[path = "../issue_1161_image_cellpath.rs"]
mod issue_1161_image_cellpath;

#[path = "../issue_1329_bullet_caret.rs"]
mod issue_1329_bullet_caret;

#[path = "../issue_2278_chart_3d_ofpie.rs"]
mod issue_2278_chart_3d_ofpie;

#[path = "../issue_2724_passthrough_invalidation_guard.rs"]
mod issue_2724_passthrough_invalidation_guard;

#[path = "../issue_3504_hwp3_autonumber_units.rs"]
mod issue_3504_hwp3_autonumber_units;

#[path = "../issue_3637_para_topbottom_vpos_base.rs"]
mod issue_3637_para_topbottom_vpos_base;

#[path = "../issue_4180_caret_stamp_roundtrip.rs"]
mod issue_4180_caret_stamp_roundtrip;

#[path = "../issue_4690_indent_over_stored_column_start.rs"]
mod issue_4690_indent_over_stored_column_start;

#[path = "../issue_986.rs"]
mod issue_986;

#[path = "../replace_occurrence_contract.rs"]
mod replace_occurrence_contract;

#[path = "../search_json_contract.rs"]
mod search_json_contract;

#[path = "../suites/issue_regression_pilot/issue_2006_1790387_prep_pagination_pin.rs"]
mod issue_2006_1790387_prep_pagination_pin;
