//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_016

#[path = "../diag_1042_used_breakdown.rs"]
mod diag_1042_used_breakdown;

#[path = "../disclose_contract.rs"]
mod disclose_contract;

#[path = "../harness_contract.rs"]
mod harness_contract;

#[path = "../issue_1086.rs"]
mod issue_1086;

#[path = "../issue_1138.rs"]
mod issue_1138;

#[path = "../issue_1195_cell_table_empty_line.rs"]
mod issue_1195_cell_table_empty_line;

#[path = "../issue_1251_ole_chart_contents.rs"]
mod issue_1251_ole_chart_contents;

#[path = "../issue_1352_table_cell_tac_picture_text.rs"]
mod issue_1352_table_cell_tac_picture_text;

#[path = "../issue_1549.rs"]
mod issue_1549;

#[path = "../issue_1623_cellzone_diagonal.rs"]
mod issue_1623_cellzone_diagonal;

#[path = "../issue_2071_cell_anchor_picture_valign.rs"]
mod issue_2071_cell_anchor_picture_valign;

#[path = "../issue_2136_neartop_reset_sb2500.rs"]
mod issue_2136_neartop_reset_sb2500;

#[path = "../issue_3460_svg_picture_render.rs"]
mod issue_3460_svg_picture_render;

#[path = "../issue_3930_hwpx_hwp_save_layout.rs"]
mod issue_3930_hwpx_hwp_save_layout;

#[path = "../issue_595.rs"]
mod issue_595;

#[path = "../issue_cli_test_caption_no_panic.rs"]
mod issue_cli_test_caption_no_panic;

#[path = "../password_crypto_multiformat_contract.rs"]
mod password_crypto_multiformat_contract;

#[path = "../scan_contract.rs"]
mod scan_contract;
