//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_015

#[path = "../dump_pages_json_contract.rs"]
mod dump_pages_json_contract;

#[path = "../export_doclang_json_contract.rs"]
mod export_doclang_json_contract;

#[path = "../ir_field_sweep_baseline.rs"]
mod ir_field_sweep_baseline;

#[path = "../issue_1052_footnote_in_textbox.rs"]
mod issue_1052_footnote_in_textbox;

#[path = "../issue_2007_nested_cell_pagination.rs"]
mod issue_2007_nested_cell_pagination;

#[path = "../issue_2097_rowbreak_midpage_declared_fits.rs"]
mod issue_2097_rowbreak_midpage_declared_fits;

#[path = "../issue_2214_page_local_repaint.rs"]
mod issue_2214_page_local_repaint;

#[path = "../issue_2277_stock.rs"]
mod issue_2277_stock;

#[path = "../issue_3307_outline_default_numbering.rs"]
mod issue_3307_outline_default_numbering;

#[path = "../issue_3308_nested_table_width.rs"]
mod issue_3308_nested_table_width;

#[path = "../issue_3744_structure_clause_confidence.rs"]
mod issue_3744_structure_clause_confidence;

#[path = "../issue_4272_nested_cell_text_selection.rs"]
mod issue_4272_nested_cell_text_selection;

#[path = "../issue_4709_metric_font_annotation.rs"]
mod issue_4709_metric_font_annotation;

#[path = "../issue_676_trailing_empty_para.rs"]
mod issue_676_trailing_empty_para;

#[path = "../render_p37_pdf_backend_cli.rs"]
mod render_p37_pdf_backend_cli;

#[path = "../split_document_tool_contract.rs"]
mod split_document_tool_contract;

#[path = "../suites/issue_regression_pilot/issue_3486_hancom_pua_display.rs"]
mod issue_3486_hancom_pua_display;

#[path = "../verify_contract.rs"]
mod verify_contract;
