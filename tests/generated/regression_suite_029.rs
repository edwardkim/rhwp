//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_029

#[path = "../cases/bookmarks_contract.rs"]
mod bookmarks_contract;

#[path = "../cases/transpose_table_contract.rs"]
mod transpose_table_contract;

#[path = "../changed_pages_contract.rs"]
mod changed_pages_contract;

#[path = "../cli_password_stdin_command_parity_contract.rs"]
mod cli_password_stdin_command_parity_contract;

#[path = "../diag_1042_vpos_distribution.rs"]
mod diag_1042_vpos_distribution;

#[path = "../issue_1070_tac_table_post_text_overflow.rs"]
mod issue_1070_tac_table_post_text_overflow;

#[path = "../issue_1868_export_hwpx_cli.rs"]
mod issue_1868_export_hwpx_cli;

#[path = "../issue_1898_tac_image_line_advance.rs"]
mod issue_1898_tac_image_line_advance;

#[path = "../issue_1939.rs"]
mod issue_1939;

#[path = "../issue_2032_picture_offpage_restrict_loss.rs"]
mod issue_2032_picture_offpage_restrict_loss;

#[path = "../issue_2099_araea_pua.rs"]
mod issue_2099_araea_pua;

#[path = "../issue_2430_cell_rewrap_threshold.rs"]
mod issue_2430_cell_rewrap_threshold;

#[path = "../issue_258_clickhere_form_mode.rs"]
mod issue_258_clickhere_form_mode;

#[path = "../issue_3216_hf_field_display_space.rs"]
mod issue_3216_hf_field_display_space;

#[path = "../issue_3542_line_startpt_namespace.rs"]
mod issue_3542_line_startpt_namespace;

#[path = "../issue_3837_stored_vpos_rewind_page_break.rs"]
mod issue_3837_stored_vpos_rewind_page_break;

#[path = "../issue_4141_hwp3_relative_size_contract.rs"]
mod issue_4141_hwp3_relative_size_contract;

#[path = "../issue_712.rs"]
mod issue_712;

#[path = "../issue_824.rs"]
mod issue_824;

#[path = "../ontology_contract.rs"]
mod ontology_contract;

#[path = "../run_plan_contract.rs"]
mod run_plan_contract;
