//! `tests/suites/manifest.json`에서 자동 생성된 integration test harness다.
//! 직접 수정하지 말고 suite manifest 생성기를 사용한다.
//! suite: regression_suite_019

#[path = "../edit_format_preserve_contract.rs"]
mod edit_format_preserve_contract;

#[path = "../extract_data_contract.rs"]
mod extract_data_contract;

#[path = "../hwp5_roundtrip_baseline.rs"]
mod hwp5_roundtrip_baseline;

#[path = "../issue_1145.rs"]
mod issue_1145;

#[path = "../issue_1436_size_protect_properties.rs"]
mod issue_1436_size_protect_properties;

#[path = "../issue_1486_hwpx_partial_tac_table.rs"]
mod issue_1486_hwpx_partial_tac_table;

#[path = "../issue_2098_page_bottom_fixed_anchor_vpos0.rs"]
mod issue_2098_page_bottom_fixed_anchor_vpos0;

#[path = "../issue_2146_no_ls_label_cell_declared_height.rs"]
mod issue_2146_no_ls_label_cell_declared_height;

#[path = "../issue_2220_tac_host_line_outer_margin.rs"]
mod issue_2220_tac_host_line_outer_margin;

#[path = "../issue_2291_rowspan_declared_residual.rs"]
mod issue_2291_rowspan_declared_residual;

#[path = "../issue_2527_empty_lineseg_reflow.rs"]
mod issue_2527_empty_lineseg_reflow;

#[path = "../issue_3413_structure_equation_text.rs"]
mod issue_3413_structure_equation_text;

#[path = "../issue_3751_vpos_reset_midparagraph_fit.rs"]
mod issue_3751_vpos_reset_midparagraph_fit;

#[path = "../issue_3820_tac_caption_first_text_owner.rs"]
mod issue_3820_tac_caption_first_text_owner;

#[path = "../issue_3865_search_text_in_table_cells.rs"]
mod issue_3865_search_text_in_table_cells;

#[path = "../issue_825.rs"]
mod issue_825;

#[path = "../issue_852_hwpx_to_hwp_contract_streams.rs"]
mod issue_852_hwpx_to_hwp_contract_streams;

#[path = "../llm_export_contract.rs"]
mod llm_export_contract;

#[path = "../wmf_emf_metafile_dos.rs"]
mod wmf_emf_metafile_dos;
