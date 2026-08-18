//! 문서 상태를 변경하지 않는 CLI 진단 조회 어댑터.
//!
//! Stage 2에서는 기존 binary-local load 및 단위 변환 seam을 보존한다. service 계층 이행은
//! Stage 3의 책임이다.

use std::fs;

use rhwp::model::footnote::FootnoteShape;

use crate::{hu_to_mm_i, load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn dump_note_shape(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-note-shape <파일.hwp|파일.hwpx>");
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let sections: Vec<serde_json::Value> = doc
        .document()
        .sections
        .iter()
        .enumerate()
        .map(|(idx, section)| {
            serde_json::json!({
                "section": idx,
                "footnoteShape": note_shape_json(&section.section_def.footnote_shape),
                "endnoteShape": note_shape_json(&section.section_def.endnote_shape),
            })
        })
        .collect();

    let value = serde_json::json!({
        "file": file_path,
        "sections": sections,
    });
    match serde_json::to_string_pretty(&value) {
        Ok(text) => {
            println!("{}", text);
            EXIT_OK
        }
        Err(e) => {
            eprintln!("오류: JSON 생성 실패 - {}", e);
            EXIT_RUNTIME
        }
    }
}

fn note_shape_json(shape: &FootnoteShape) -> serde_json::Value {
    serde_json::json!({
        "raw": {
            "attr": shape.attr,
            "numberFormat": format!("{:?}", shape.number_format),
            "userChar": shape.user_char.to_string(),
            "prefixChar": shape.prefix_char.to_string(),
            "suffixChar": shape.suffix_char.to_string(),
            "startNumber": shape.start_number,
            "separatorLength": hu_json(shape.separator_length as i32),
            "separatorMarginTop": hu_json(shape.separator_margin_top as i32),
            "separatorMarginBottom": hu_json(shape.separator_margin_bottom as i32),
            "noteSpacing": hu_json(shape.note_spacing as i32),
            "separatorLineType": shape.separator_line_type,
            "separatorLineWidth": shape.separator_line_width,
            "separatorColor": format!("0x{:08x}", shape.separator_color),
            "numbering": format!("{:?}", shape.numbering),
            "placement": format!("{:?}", shape.placement),
            "numberCodeSuperscript": shape.number_code_superscript,
            "printInlineAfterText": shape.print_inline_after_text,
            "rawUnknown": hu_json(shape.raw_unknown as i32),
        },
        "ui": {
            "separatorAbove": hu_json(shape.separator_above_margin_hu() as i32),
            "separatorBelow": hu_json(shape.separator_below_margin_hu() as i32),
            "betweenNotes": hu_json(shape.between_notes_margin_hu() as i32),
        },
    })
}

fn hu_json(hu: i32) -> serde_json::Value {
    serde_json::json!({
        "hu": hu,
        "mm": rounded_mm(hu),
    })
}

fn rounded_mm(hu: i32) -> f64 {
    (hu_to_mm_i(hu) * 1000.0).round() / 1000.0
}
