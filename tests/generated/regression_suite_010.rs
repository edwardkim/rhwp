//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_010

#[path = "../agent_context_cost_contract.rs"]
mod agent_context_cost_contract;

#[path = "../diag_1042_height_calc.rs"]
mod diag_1042_height_calc;

#[path = "../hwp3_charcount_convention.rs"]
mod hwp3_charcount_convention;

#[path = "../issue_1152_intra_para_vpos_reset.rs"]
mod issue_1152_intra_para_vpos_reset;

#[path = "../issue_1219_equation_line_hangul_advance.rs"]
mod issue_1219_equation_line_hangul_advance;

#[path = "../issue_1440_onsamiro_picture_wrap.rs"]
mod issue_1440_onsamiro_picture_wrap;

#[path = "../issue_1459_topbottom_picture_reflow.rs"]
mod issue_1459_topbottom_picture_reflow;

#[path = "../issue_1880_takeplace_host_before.rs"]
mod issue_1880_takeplace_host_before;

#[path = "../issue_1898.rs"]
mod issue_1898;

#[path = "../issue_2083_hide_fill_page_background.rs"]
mod issue_2083_hide_fill_page_background;

#[path = "../issue_2164_cell_enter_overlap.rs"]
mod issue_2164_cell_enter_overlap;

#[path = "../issue_3593_nested_host_cell_height.rs"]
mod issue_3593_nested_host_cell_height;

#[path = "../issue_4155_hwp3_char_shade_contract.rs"]
mod issue_4155_hwp3_char_shade_contract;

#[path = "../issue_4395_style_zero_hwpx_export.rs"]
mod issue_4395_style_zero_hwpx_export;

#[path = "../issue_4586_gym_t12_contract.rs"]
mod issue_4586_gym_t12_contract;

#[path = "../provenance_contract.rs"]
mod provenance_contract;

#[path = "../run_plan_dry_run_contract.rs"]
mod run_plan_dry_run_contract;

#[path = "../suites/issue_regression_pilot/issue_2097_1730000_real_doc_pin.rs"]
mod issue_2097_1730000_real_doc_pin;
