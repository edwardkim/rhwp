//! `edit` command dispatch and shared runtime boundary.

use crate::{
    edit_add_bookmark, edit_apply_cell_style, edit_apply_char_format,
    edit_apply_char_format_in_cell, edit_apply_endnote_shape, edit_apply_hf_template,
    edit_apply_para_format, edit_apply_para_format_in_cell, edit_apply_para_format_in_footnote,
    edit_apply_para_format_in_hf, edit_apply_style, edit_delete_bookmark, edit_delete_control,
    edit_delete_footnote, edit_delete_header_footer, edit_delete_hf_text, edit_delete_paragraph,
    edit_delete_text, edit_delete_text_in_footnote, edit_insert_column_break, edit_insert_endnote,
    edit_insert_field_in_hf, edit_insert_footnote, edit_insert_footnote_text,
    edit_insert_header_footer, edit_insert_header_footer_text, edit_insert_page_break,
    edit_insert_paragraph, edit_insert_text, edit_merge_paragraph, edit_merge_paragraph_in_cell,
    edit_merge_paragraph_in_footnote, edit_merge_paragraph_in_hf, edit_rename_bookmark,
    edit_set_column_def, edit_set_header_footer_text, edit_set_hf_picture,
    edit_set_numbering_restart, edit_set_page_def, edit_set_page_hide, edit_set_section_def,
    edit_split_paragraph, edit_split_paragraph_in_cell, edit_split_paragraph_in_footnote,
    edit_split_paragraph_in_hf, edit_toggle_hide_hf, EXIT_USAGE,
};

mod cells;
mod document_objects;
mod equations;
mod fields;
mod media;
mod privacy;
pub(crate) mod runtime;
mod shapes;
mod tables;
mod text;

use cells::{
    edit_delete_text_in_cell, edit_insert_text_in_cell, edit_set_cell, edit_set_cell_props,
};
pub(crate) use cells::{
    measure_cell_overflow, recolor_cell_text_black, set_cell_control_char_rejection,
};
use document_objects::{
    edit_insert_number, edit_set_chart_data, edit_set_form_value, edit_set_form_value_in_cell,
    edit_set_page_border_fill,
};
use equations::{edit_delete_equation, edit_insert_equation, edit_set_equation_properties};
use fields::edit_fill_fields;
pub(crate) use fields::{fill_fields_core, parse_field_key};
use media::{edit_delete_picture, edit_insert_image, edit_insert_picture, edit_set_picture};
use privacy::{edit_redact, edit_sanitize};
use shapes::{edit_delete_shape, edit_group_shapes, edit_insert_shape, edit_ungroup_shape};
use tables::{
    edit_delete_col, edit_delete_row, edit_delete_table, edit_fit_table, edit_insert_col,
    edit_insert_row, edit_insert_table, edit_merge_cells, edit_merge_table, edit_move_table,
    edit_resize_table, edit_resize_table_cell, edit_set_column_widths, edit_set_table_props,
    edit_split_cell, edit_split_cell_into, edit_split_table, edit_transpose_table,
};
pub(crate) use tables::{resolve_table_cell, CellResolveError};
use text::edit_replace_text;

/// `edit` — 문서 편집 명령군 (로드맵 #2659 Stage 3).
///
/// 공통 규약: `--dry-run`(변경 요약만 출력, 파일 무변경), 결과 리포트 JSON,
/// **실패 시 원본 불변**(하나라도 실패하면 출력 파일을 쓰지 않는다).
pub(crate) fn run(args: &[String]) -> i32 {
    const USAGE: &str =
        "사용법: rhwp edit <fill-fields|replace-text|set-cell|insert-text-in-cell|delete-text-in-cell|insert-text|delete-text|insert-paragraph|delete-paragraph|merge-paragraph|insert-page-break|insert-column-break|insert-table|set-numbering-restart|insert-row|insert-col|delete-row|delete-col|merge-cells|split-cell|split-cell-into|split-table|fit-table|resize-table|resize-table-cell|set-cell-props|set-table-props|move-table|merge-table|set-column-widths|insert-footnote|insert-endnote|insert-equation|delete-footnote|delete-text-in-footnote|insert-footnote-text|split-paragraph-in-footnote|merge-paragraph-in-footnote|apply-para-format-in-footnote|add-bookmark|delete-bookmark|delete-table|rename-bookmark|delete-header-footer|insert-header-footer-text|set-header-footer-text|delete-hf-text|set-hf-picture|apply-hf-template|split-paragraph-in-hf|merge-paragraph-in-hf|apply-para-format-in-hf|toggle-hide-hf|split-paragraph-in-cell|merge-paragraph-in-cell|apply-char-format|apply-para-format|apply-style|apply-cell-style|apply-para-format-in-cell|apply-char-format-in-cell|delete-control|insert-header-footer|insert-field-in-hf|set-column-def|delete-equation|split-paragraph|set-page-hide|transpose-table|set-equation-properties|insert-image|group-shapes|set-page-def|set-section-def|apply-endnote-shape|insert-picture|delete-picture|set-picture|set-page-border-fill|redact|sanitize|set-chart-data|insert-number|insert-shape|delete-shape|set-form-value|set-form-value-in-cell|ungroup-shape> <파일.hwp|파일.hwpx> [옵션] (rhwp --help 참조)";

    match args.first().map(String::as_str) {
        Some("fill-fields") => edit_fill_fields(&args[1..]),
        Some("replace-text") => edit_replace_text(&args[1..]),
        Some("set-cell") => edit_set_cell(&args[1..]),
        Some("insert-text-in-cell") => edit_insert_text_in_cell(&args[1..]),
        Some("delete-text-in-cell") => edit_delete_text_in_cell(&args[1..]),
        Some("insert-text") => edit_insert_text(&args[1..]),
        Some("delete-text") => edit_delete_text(&args[1..]),
        Some("insert-paragraph") => edit_insert_paragraph(&args[1..]),
        Some("delete-paragraph") => edit_delete_paragraph(&args[1..]),
        Some("merge-paragraph") => edit_merge_paragraph(&args[1..]),
        Some("insert-page-break") => edit_insert_page_break(&args[1..]),
        Some("insert-column-break") => edit_insert_column_break(&args[1..]),
        Some("insert-table") => edit_insert_table(&args[1..]),
        Some("insert-row") => edit_insert_row(&args[1..]),
        Some("insert-col") => edit_insert_col(&args[1..]),
        Some("delete-row") => edit_delete_row(&args[1..]),
        Some("delete-col") => edit_delete_col(&args[1..]),
        Some("merge-cells") => edit_merge_cells(&args[1..]),
        Some("split-cell") => edit_split_cell(&args[1..]),
        Some("split-cell-into") => edit_split_cell_into(&args[1..]),
        Some("split-table") => edit_split_table(&args[1..]),
        Some("fit-table") => edit_fit_table(&args[1..]),
        Some("resize-table") => edit_resize_table(&args[1..]),
        Some("resize-table-cell") => edit_resize_table_cell(&args[1..]),
        Some("set-cell-props") => edit_set_cell_props(&args[1..]),
        Some("set-table-props") => edit_set_table_props(&args[1..]),
        Some("move-table") => edit_move_table(&args[1..]),
        Some("merge-table") => edit_merge_table(&args[1..]),
        Some("set-column-widths") => edit_set_column_widths(&args[1..]),
        Some("insert-footnote") => edit_insert_footnote(&args[1..]),
        Some("insert-endnote") => edit_insert_endnote(&args[1..]),
        Some("insert-equation") => edit_insert_equation(&args[1..]),
        Some("delete-footnote") => edit_delete_footnote(&args[1..]),
        Some("insert-footnote-text") => edit_insert_footnote_text(&args[1..]),
        Some("delete-text-in-footnote") => edit_delete_text_in_footnote(&args[1..]),
        Some("split-paragraph-in-footnote") => edit_split_paragraph_in_footnote(&args[1..]),
        Some("merge-paragraph-in-footnote") => edit_merge_paragraph_in_footnote(&args[1..]),
        Some("apply-para-format-in-footnote") => edit_apply_para_format_in_footnote(&args[1..]),
        Some("add-bookmark") => edit_add_bookmark(&args[1..]),
        Some("delete-bookmark") => edit_delete_bookmark(&args[1..]),
        Some("rename-bookmark") => edit_rename_bookmark(&args[1..]),
        Some("delete-header-footer") => edit_delete_header_footer(&args[1..]),
        Some("insert-header-footer-text") => edit_insert_header_footer_text(&args[1..]),
        Some("set-header-footer-text") => edit_set_header_footer_text(&args[1..]),
        Some("delete-hf-text") => edit_delete_hf_text(&args[1..]),
        Some("set-hf-picture") => edit_set_hf_picture(&args[1..]),
        Some("apply-hf-template") => edit_apply_hf_template(&args[1..]),
        Some("split-paragraph-in-hf") => edit_split_paragraph_in_hf(&args[1..]),
        Some("merge-paragraph-in-hf") => edit_merge_paragraph_in_hf(&args[1..]),
        Some("apply-para-format-in-hf") => edit_apply_para_format_in_hf(&args[1..]),
        Some("toggle-hide-hf") => edit_toggle_hide_hf(&args[1..]),
        Some("split-paragraph-in-cell") => edit_split_paragraph_in_cell(&args[1..]),
        Some("merge-paragraph-in-cell") => edit_merge_paragraph_in_cell(&args[1..]),
        Some("apply-char-format") => edit_apply_char_format(&args[1..]),
        Some("apply-para-format") => edit_apply_para_format(&args[1..]),
        Some("apply-style") => edit_apply_style(&args[1..]),
        Some("set-numbering-restart") => edit_set_numbering_restart(&args[1..]),
        Some("apply-cell-style") => edit_apply_cell_style(&args[1..]),
        Some("apply-para-format-in-cell") => edit_apply_para_format_in_cell(&args[1..]),
        Some("apply-char-format-in-cell") => edit_apply_char_format_in_cell(&args[1..]),
        Some("delete-control") => edit_delete_control(&args[1..]),
        Some("delete-table") => edit_delete_table(&args[1..]),
        Some("insert-header-footer") => edit_insert_header_footer(&args[1..]),
        Some("insert-field-in-hf") => edit_insert_field_in_hf(&args[1..]),
        Some("set-column-def") => edit_set_column_def(&args[1..]),
        Some("delete-equation") => edit_delete_equation(&args[1..]),
        Some("split-paragraph") => edit_split_paragraph(&args[1..]),
        Some("set-page-hide") => edit_set_page_hide(&args[1..]),
        Some("transpose-table") => edit_transpose_table(&args[1..]),
        Some("set-equation-properties") => edit_set_equation_properties(&args[1..]),
        Some("insert-image") => edit_insert_image(&args[1..]),
        Some("group-shapes") => edit_group_shapes(&args[1..]),
        Some("set-page-def") => edit_set_page_def(&args[1..]),
        Some("set-section-def") => edit_set_section_def(&args[1..]),
        Some("apply-endnote-shape") => edit_apply_endnote_shape(&args[1..]),
        Some("insert-picture") => edit_insert_picture(&args[1..]),
        Some("delete-picture") => edit_delete_picture(&args[1..]),
        Some("set-picture") => edit_set_picture(&args[1..]),
        Some("set-chart-data") => edit_set_chart_data(&args[1..]),
        Some("insert-number") => edit_insert_number(&args[1..]),
        Some("insert-shape") => edit_insert_shape(&args[1..]),
        Some("delete-shape") => edit_delete_shape(&args[1..]),
        Some("set-form-value") => edit_set_form_value(&args[1..]),
        Some("set-form-value-in-cell") => edit_set_form_value_in_cell(&args[1..]),
        Some("ungroup-shape") => edit_ungroup_shape(&args[1..]),
        Some("set-page-border-fill") => edit_set_page_border_fill(&args[1..]),
        // [#3719 §6-11] 공개 전 정리 — 개인정보 마스킹 / 메타데이터 제거.
        Some("redact") => edit_redact(&args[1..]),
        Some("sanitize") => edit_sanitize(&args[1..]),
        Some(other) => {
            eprintln!("오류: 알 수 없는 edit 하위 명령 - {}", other);
            eprintln!("{USAGE}");
            EXIT_USAGE
        }
        None => {
            eprintln!("오류: edit 하위 명령을 지정해주세요.");
            eprintln!("{USAGE}");
            EXIT_USAGE
        }
    }
}
