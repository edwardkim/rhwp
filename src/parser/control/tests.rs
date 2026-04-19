use super::*;
use crate::parser::tags;

fn make_record(tag_id: u16, level: u16, data: Vec<u8>) -> Record {
    Record {
        tag_id,
        level,
        size: data.len() as u32,
        data,
    }
}

fn make_para_header_data(char_count: u32) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&char_count.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes()); // control_mask
    data.extend_from_slice(&0u16.to_le_bytes()); // para_shape_id
    data.push(0); // style_id
    data.push(0); // break_type
    data
}

fn make_para_text_data(text: &str) -> Vec<u8> {
    let mut data = Vec::new();
    for ch in text.encode_utf16() {
        data.extend_from_slice(&ch.to_le_bytes());
    }
    data.extend_from_slice(&0x000Du16.to_le_bytes());
    data
}

#[test]
fn test_parse_table_basic() {
    // TABLE 레코드 데이터: 2×2 표
    let mut table_data = Vec::new();
    table_data.extend_from_slice(&0u32.to_le_bytes()); // attr
    table_data.extend_from_slice(&2u16.to_le_bytes()); // row_count
    table_data.extend_from_slice(&2u16.to_le_bytes()); // col_count
    table_data.extend_from_slice(&0i16.to_le_bytes()); // cell_spacing
    table_data.extend_from_slice(&0i16.to_le_bytes()); // padding_left
    table_data.extend_from_slice(&0i16.to_le_bytes()); // padding_right
    table_data.extend_from_slice(&0i16.to_le_bytes()); // padding_top
    table_data.extend_from_slice(&0i16.to_le_bytes()); // padding_bottom
    // row heights
    table_data.extend_from_slice(&500i16.to_le_bytes());
    table_data.extend_from_slice(&500i16.to_le_bytes());
    table_data.extend_from_slice(&1u16.to_le_bytes()); // border_fill_id

    // LIST_HEADER (cell 0,0) 데이터
    let mut cell_data = Vec::new();
    cell_data.extend_from_slice(&1u16.to_le_bytes()); // n_paragraphs
    cell_data.extend_from_slice(&0u32.to_le_bytes()); // list_attr
    cell_data.extend_from_slice(&0u16.to_le_bytes()); // unknown (텍스트 영역 폭)
    cell_data.extend_from_slice(&0u16.to_le_bytes()); // col
    cell_data.extend_from_slice(&0u16.to_le_bytes()); // row
    cell_data.extend_from_slice(&1u16.to_le_bytes()); // col_span
    cell_data.extend_from_slice(&1u16.to_le_bytes()); // row_span
    cell_data.extend_from_slice(&10000u32.to_le_bytes()); // width
    cell_data.extend_from_slice(&5000u32.to_le_bytes()); // height
    cell_data.extend_from_slice(&0i16.to_le_bytes()); // paddings
    cell_data.extend_from_slice(&0i16.to_le_bytes());
    cell_data.extend_from_slice(&0i16.to_le_bytes());
    cell_data.extend_from_slice(&0i16.to_le_bytes());
    cell_data.extend_from_slice(&1u16.to_le_bytes()); // border_fill_id

    let child_records = vec![
        make_record(tags::HWPTAG_TABLE, 2, table_data),
        make_record(tags::HWPTAG_LIST_HEADER, 2, cell_data),
        make_record(
            tags::HWPTAG_PARA_HEADER,
            3,
            make_para_header_data(5),
        ),
        make_record(
            tags::HWPTAG_PARA_TEXT,
            4,
            make_para_text_data("test"),
        ),
    ];

    let ctrl = parse_table_control(&[], &child_records);
    if let Control::Table(table) = ctrl {
        assert_eq!(table.row_count, 2);
        assert_eq!(table.col_count, 2);
        assert_eq!(table.cells.len(), 1);
        assert_eq!(table.cells[0].width, 10000);
        assert_eq!(table.cells[0].paragraphs.len(), 1);
        assert_eq!(table.cells[0].paragraphs[0].text, "test");
    } else {
        panic!("Expected Table control");
    }
}

#[test]
fn test_parse_header_control() {
    let mut ctrl_data = Vec::new();
    ctrl_data.extend_from_slice(&0u32.to_le_bytes()); // attr: Both

    // LIST_HEADER + paragraph
    let mut list_data = Vec::new();
    list_data.extend_from_slice(&1u16.to_le_bytes()); // n_paragraphs
    list_data.extend_from_slice(&0u32.to_le_bytes()); // list_attr

    let child_records = vec![
        make_record(tags::HWPTAG_LIST_HEADER, 2, list_data),
        make_record(
            tags::HWPTAG_PARA_HEADER,
            3,
            make_para_header_data(6),
        ),
        make_record(
            tags::HWPTAG_PARA_TEXT,
            4,
            make_para_text_data("머리말"),
        ),
    ];

    let ctrl = parse_header_control(&ctrl_data, &child_records);
    if let Control::Header(header) = ctrl {
        assert_eq!(header.apply_to, HeaderFooterApply::Both);
        assert_eq!(header.paragraphs.len(), 1);
        assert_eq!(header.paragraphs[0].text, "머리말");
    } else {
        panic!("Expected Header control");
    }
}

#[test]
fn test_parse_footer_control() {
    let mut ctrl_data = Vec::new();
    ctrl_data.extend_from_slice(&1u32.to_le_bytes()); // attr: Even

    let child_records = vec![
        make_record(tags::HWPTAG_LIST_HEADER, 2, vec![0; 6]),
    ];

    let ctrl = parse_footer_control(&ctrl_data, &child_records);
    if let Control::Footer(footer) = ctrl {
        assert_eq!(footer.apply_to, HeaderFooterApply::Even);
    } else {
        panic!("Expected Footer control");
    }
}

#[test]
fn test_parse_footnote_control() {
    let mut ctrl_data = Vec::new();
    ctrl_data.extend_from_slice(&3u16.to_le_bytes()); // number = 3

    let child_records = vec![
        make_record(tags::HWPTAG_LIST_HEADER, 2, vec![0; 6]),
        make_record(
            tags::HWPTAG_PARA_HEADER,
            3,
            make_para_header_data(5),
        ),
        make_record(
            tags::HWPTAG_PARA_TEXT,
            4,
            make_para_text_data("각주"),
        ),
    ];

    let ctrl = parse_footnote_control(&ctrl_data, &child_records);
    if let Control::Footnote(fn_) = ctrl {
        assert_eq!(fn_.number, 3);
        assert_eq!(fn_.paragraphs.len(), 1);
        assert_eq!(fn_.paragraphs[0].text, "각주");
    } else {
        panic!("Expected Footnote control");
    }
}

#[test]
fn test_parse_auto_number() {
    let mut ctrl_data = Vec::new();
    ctrl_data.extend_from_slice(&0x04u32.to_le_bytes()); // Table type

    let ctrl = parse_auto_number(&ctrl_data);
    if let Control::AutoNumber(an) = ctrl {
        assert_eq!(an.number_type, AutoNumberType::Table);
    } else {
        panic!("Expected AutoNumber control");
    }
}

#[test]
fn test_parse_bookmark() {
    let mut ctrl_data = Vec::new();
    // HWP string: length=4, "test"
    ctrl_data.extend_from_slice(&4u16.to_le_bytes());
    for ch in "test".encode_utf16() {
        ctrl_data.extend_from_slice(&ch.to_le_bytes());
    }

    let ctrl = parse_bookmark(&ctrl_data);
    if let Control::Bookmark(bm) = ctrl {
        assert_eq!(bm.name, "test");
    } else {
        panic!("Expected Bookmark control");
    }
}

#[test]
fn test_parse_page_hide() {
    let mut ctrl_data = Vec::new();
    ctrl_data.extend_from_slice(&0x07u32.to_le_bytes()); // hide header+footer+master

    let ctrl = parse_page_hide(&ctrl_data);
    if let Control::PageHide(ph) = ctrl {
        assert!(ph.hide_header);
        assert!(ph.hide_footer);
        assert!(ph.hide_master_page);
        assert!(!ph.hide_border);
    } else {
        panic!("Expected PageHide control");
    }
}

#[test]
fn test_parse_hidden_comment() {
    let child_records = vec![
        make_record(tags::HWPTAG_LIST_HEADER, 2, vec![0; 6]),
        make_record(
            tags::HWPTAG_PARA_HEADER,
            3,
            make_para_header_data(5),
        ),
        make_record(
            tags::HWPTAG_PARA_TEXT,
            4,
            make_para_text_data("메모"),
        ),
    ];

    let ctrl = parse_hidden_comment_control(&child_records);
    if let Control::HiddenComment(comment) = ctrl {
        assert_eq!(comment.paragraphs.len(), 1);
        assert_eq!(comment.paragraphs[0].text, "메모");
    } else {
        panic!("Expected HiddenComment control");
    }
}

#[test]
fn test_parse_control_dispatch() {
    let ctrl = parse_control(0x12345678, &[], &[]);
    assert!(matches!(ctrl, Control::Unknown(u) if u.ctrl_id == 0x12345678));
}

#[test]
fn test_parse_char_overlap() {
    // 표 152: WORD(len=2) + WCHAR['A','B'] + border_type(1) + inner_size(0) + expansion(0) + cs_count(0)
    let mut data = Vec::new();
    data.extend_from_slice(&2u16.to_le_bytes()); // len = 2
    data.extend_from_slice(&0x0041u16.to_le_bytes()); // 'A'
    data.extend_from_slice(&0x0042u16.to_le_bytes()); // 'B'
    data.push(1); // border_type = 원
    data.push(0i8 as u8); // inner_char_size = 0 (기본)
    data.push(0); // expansion
    data.push(0); // cs_count

    let ctrl = parse_char_overlap(&data);
    if let Control::CharOverlap(co) = ctrl {
        assert_eq!(co.chars, vec!['A', 'B']);
        assert_eq!(co.border_type, 1);
        assert_eq!(co.inner_char_size, 0);
        assert_eq!(co.char_shape_ids.len(), 0);
    } else {
        panic!("Expected CharOverlap control");
    }
}

#[test]
fn test_char_dup_sample_parsing() {
    let path = std::path::Path::new("samples/char-dup.hwp");
    if !path.exists() {
        eprintln!("samples/char-dup.hwp 없음 — 건너뜀");
        return;
    }
    let data = std::fs::read(path).unwrap();
    let doc = crate::parser::parse_hwp(&data).expect("parse");

    let mut char_overlap_count = 0;
    for section in &doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::CharOverlap(co) = ctrl {
                    char_overlap_count += 1;
                    eprintln!("CharOverlap: chars={:?}, border={}, size={}", co.chars, co.border_type, co.inner_char_size);
                }
                // 표 셀 내부도 탐색
                if let Control::Table(table) = ctrl {
                    for cell in &table.cells {
                        for cp in &cell.paragraphs {
                            for cc in &cp.controls {
                                if let Control::CharOverlap(co) = cc {
                                    char_overlap_count += 1;
                                    eprintln!("CharOverlap(cell): chars={:?}, border={}, size={}, cs_ids={:?}", co.chars, co.border_type, co.inner_char_size, co.char_shape_ids);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("Total CharOverlap controls found: {}", char_overlap_count);
    assert!(char_overlap_count > 0, "char-dup.hwp에서 CharOverlap 컨트롤을 찾지 못함");
}

#[test]
fn test_parse_common_obj_attr() {
    let mut data = Vec::new();
    data.extend_from_slice(&0x01u32.to_le_bytes()); // attr: treat_as_char
    data.extend_from_slice(&1000u32.to_le_bytes()); // vertical_offset
    data.extend_from_slice(&2000u32.to_le_bytes()); // horizontal_offset
    data.extend_from_slice(&5000u32.to_le_bytes()); // width
    data.extend_from_slice(&3000u32.to_le_bytes()); // height
    data.extend_from_slice(&1i32.to_le_bytes()); // z_order
    data.extend_from_slice(&0i16.to_le_bytes()); // margins
    data.extend_from_slice(&0i16.to_le_bytes());
    data.extend_from_slice(&0i16.to_le_bytes());
    data.extend_from_slice(&0i16.to_le_bytes());
    data.extend_from_slice(&42u32.to_le_bytes()); // instance_id

    let common = parse_common_obj_attr(&data);
    assert!(common.treat_as_char);
    assert_eq!(common.width, 5000);
    assert_eq!(common.height, 3000);
    assert_eq!(common.instance_id, 42);
}

#[test]
fn debug_border_data() {
    let path = std::path::Path::new("samples/k-water-rfp.hwp");
    if !path.exists() {
        eprintln!("샘플 파일 없음 — 건너뜀");
        return;
    }
    let data = std::fs::read(path).unwrap();
    let doc = crate::parser::parse_hwp(&data).expect("parse");

    eprintln!("\n=== BorderFill entries (total: {}) ===", doc.doc_info.border_fills.len());
    for (i, bf) in doc.doc_info.border_fills.iter().enumerate().take(10) {
        eprintln!("  BF[{}]: L=({:?}, w={}, c={:?}) R=({:?}, w={}, c={:?}) T=({:?}, w={}, c={:?}) B=({:?}, w={}, c={:?}) fill={:?}",
            i,
            bf.borders[0].line_type, bf.borders[0].width, bf.borders[0].color,
            bf.borders[1].line_type, bf.borders[1].width, bf.borders[1].color,
            bf.borders[2].line_type, bf.borders[2].width, bf.borders[2].color,
            bf.borders[3].line_type, bf.borders[3].width, bf.borders[3].color,
            bf.fill,
        );
    }

    eprintln!("\n=== Tables in section 0 (first 30 paragraphs) ===");
    for (pi, para) in doc.sections[0].paragraphs.iter().enumerate().take(30) {
        for ctrl in &para.controls {
            if let crate::model::control::Control::Table(table) = ctrl {
                eprintln!("\nP{} Table: {}rows x {}cols border_fill_id={}", pi, table.row_count, table.col_count, table.border_fill_id);
                let bf_idx = table.border_fill_id as usize;
                if bf_idx > 0 && bf_idx <= doc.doc_info.border_fills.len() {
                    let tbf = &doc.doc_info.border_fills[bf_idx - 1];
                    eprintln!("  -> Table BF[{}]: L=({:?},w={},c={:?}) R=({:?},w={},c={:?}) T=({:?},w={},c={:?}) B=({:?},w={},c={:?})",
                        bf_idx - 1,
                        tbf.borders[0].line_type, tbf.borders[0].width, tbf.borders[0].color,
                        tbf.borders[1].line_type, tbf.borders[1].width, tbf.borders[1].color,
                        tbf.borders[2].line_type, tbf.borders[2].width, tbf.borders[2].color,
                        tbf.borders[3].line_type, tbf.borders[3].width, tbf.borders[3].color,
                    );
                } else {
                    eprintln!("  -> INVALID border_fill_id: {} (max={})", bf_idx, doc.doc_info.border_fills.len());
                }
                for (ci, cell) in table.cells.iter().enumerate().take(6) {
                    eprintln!("  Cell[{}] r={} c={} colspan={} rowspan={} bfid={}", ci, cell.row, cell.col, cell.col_span, cell.row_span, cell.border_fill_id);
                    let cbf_idx = cell.border_fill_id as usize;
                    if cbf_idx > 0 && cbf_idx <= doc.doc_info.border_fills.len() {
                        let cbf = &doc.doc_info.border_fills[cbf_idx - 1];
                        eprintln!("    -> Cell BF[{}]: L=({:?},w={},c={:?}) R=({:?},w={},c={:?}) T=({:?},w={},c={:?}) B=({:?},w={},c={:?}) fill={:?}",
                            cbf_idx - 1,
                            cbf.borders[0].line_type, cbf.borders[0].width, cbf.borders[0].color,
                            cbf.borders[1].line_type, cbf.borders[1].width, cbf.borders[1].color,
                            cbf.borders[2].line_type, cbf.borders[2].width, cbf.borders[2].color,
                            cbf.borders[3].line_type, cbf.borders[3].width, cbf.borders[3].color,
                            cbf.fill,
                        );
                    } else {
                        eprintln!("    -> INVALID border_fill_id: {} (max={})", cbf_idx, doc.doc_info.border_fills.len());
                    }
                }
            }
        }
    }
}

#[test]
fn dump_bookreview_section1_controls() {
    let path = std::path::Path::new("samples/basic/BookReview.hwp");
    if !path.exists() {
        eprintln!("samples/basic/BookReview.hwp 없음 — 건너뜀");
        return;
    }
    let data = std::fs::read(path).unwrap();
    let doc = crate::parser::parse_hwp(&data).expect("BookReview.hwp parse failed");

    eprintln!("\n========== BookReview.hwp section count: {} ==========", doc.sections.len());

    // Dump all sections
    for sec_idx in 0..doc.sections.len() {
    let section = &doc.sections[sec_idx];
    eprintln!("\n=== Section {} : {} paragraphs ===", sec_idx, section.paragraphs.len());

    for (pi, para) in section.paragraphs.iter().enumerate() {
        let text_preview: String = para.text.chars().take(60).collect();
        eprintln!("\n  Para[{}]: text_len={}, controls={}, text={:?}",
            pi, para.text.len(), para.controls.len(), text_preview);

        for (ci, ctrl) in para.controls.iter().enumerate() {
            match ctrl {
                Control::SectionDef(_) => eprintln!("    Ctrl[{}]: SectionDef", ci),
                Control::ColumnDef(_) => eprintln!("    Ctrl[{}]: ColumnDef", ci),
                Control::Table(table) => {
                    eprintln!("    Ctrl[{}]: Table ({}x{}, {} cells)",
                        ci, table.row_count, table.col_count, table.cells.len());
                    for (cell_i, cell) in table.cells.iter().enumerate() {
                        let cell_text: String = cell.paragraphs.iter()
                            .map(|p| p.text.as_str())
                            .collect::<Vec<_>>()
                            .join("|");
                        let cell_text_preview: String = cell_text.chars().take(40).collect();
                        eprintln!("      Cell[{}] (r={},c={}): {} paras, text={:?}",
                            cell_i, cell.row, cell.col, cell.paragraphs.len(), cell_text_preview);
                        // Dump controls inside cell paragraphs
                        for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                            if !cp.controls.is_empty() {
                                eprintln!("        CellPara[{}]: {} controls", cpi, cp.controls.len());
                                for (cci, cc) in cp.controls.iter().enumerate() {
                                    dump_control_brief(cc, cci, 10);
                                }
                            }
                        }
                    }
                }
                Control::Shape(shape) => {
                    let shape_type = match shape.as_ref() {
                        crate::model::shape::ShapeObject::Line(_) => "Line",
                        crate::model::shape::ShapeObject::Rectangle(_) => "Rectangle",
                        crate::model::shape::ShapeObject::Ellipse(_) => "Ellipse",
                        crate::model::shape::ShapeObject::Arc(_) => "Arc",
                        crate::model::shape::ShapeObject::Polygon(_) => "Polygon",
                        crate::model::shape::ShapeObject::Curve(_) => "Curve",
                        crate::model::shape::ShapeObject::Group(_) => "Group",
                        crate::model::shape::ShapeObject::Picture(_) => "Picture",
                        crate::model::shape::ShapeObject::Chart(_) => "Chart",
                        crate::model::shape::ShapeObject::Ole(_) => "Ole",
                    };
                    let common = shape.common();
                    let has_textbox = shape.drawing().and_then(|d| d.text_box.as_ref());
                    if let Some(tb) = has_textbox {
                        eprintln!("    Ctrl[{}]: Shape({}) size={}x{} treat_as_char={} — TextBox: {} paras",
                            ci, shape_type, common.width, common.height, common.treat_as_char, tb.paragraphs.len());
                        for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                            let tp_preview: String = tp.text.chars().take(50).collect();
                            eprintln!("      TBPara[{}]: text_len={}, controls={}, text={:?}",
                                tpi, tp.text.len(), tp.controls.len(), tp_preview);
                            // Recurse: controls inside textbox paragraphs
                            for (tci, tc) in tp.controls.iter().enumerate() {
                                dump_control_brief(tc, tci, 8);
                            }
                        }
                    } else {
                        eprintln!("    Ctrl[{}]: Shape({}) size={}x{} treat_as_char={} — no textbox",
                            ci, shape_type, common.width, common.height, common.treat_as_char);
                    }
                }
                Control::Picture(pic) => {
                    eprintln!("    Ctrl[{}]: Picture size={}x{} treat_as_char={}",
                        ci, pic.common.width, pic.common.height, pic.common.treat_as_char);
                }
                Control::Header(_) => eprintln!("    Ctrl[{}]: Header", ci),
                Control::Footer(_) => eprintln!("    Ctrl[{}]: Footer", ci),
                Control::Footnote(f) => eprintln!("    Ctrl[{}]: Footnote (num={}, {} paras)", ci, f.number, f.paragraphs.len()),
                Control::Endnote(e) => eprintln!("    Ctrl[{}]: Endnote (num={}, {} paras)", ci, e.number, e.paragraphs.len()),
                Control::AutoNumber(a) => eprintln!("    Ctrl[{}]: AutoNumber({:?}, num={})", ci, a.number_type, a.assigned_number),
                Control::NewNumber(_) => eprintln!("    Ctrl[{}]: NewNumber", ci),
                Control::PageNumberPos(_) => eprintln!("    Ctrl[{}]: PageNumberPos", ci),
                Control::Bookmark(b) => eprintln!("    Ctrl[{}]: Bookmark(name={:?})", ci, b.name),
                Control::Hyperlink(h) => eprintln!("    Ctrl[{}]: Hyperlink(url={:?})", ci, h.url),
                Control::Ruby(r) => eprintln!("    Ctrl[{}]: Ruby(text={:?})", ci, r.ruby_text),
                Control::CharOverlap(_) => eprintln!("    Ctrl[{}]: CharOverlap", ci),
                Control::PageHide(_) => eprintln!("    Ctrl[{}]: PageHide", ci),
                Control::HiddenComment(hc) => eprintln!("    Ctrl[{}]: HiddenComment ({} paras)", ci, hc.paragraphs.len()),
                Control::Equation(eq) => eprintln!("    Ctrl[{}]: Equation(script={:?})", ci, eq.script),
                Control::Field(f) => eprintln!("    Ctrl[{}]: Field({:?})", ci, f.field_type),
                Control::Form(f) => eprintln!("    Ctrl[{}]: Form({:?}, name={})", ci, f.form_type, f.name),
                Control::Unknown(u) => eprintln!("    Ctrl[{}]: Unknown(ctrl_id=0x{:08X})", ci, u.ctrl_id),
            }
        }
    }
    } // end for sec_idx

    // Also dump all sections summary
    eprintln!("\n=== All sections summary ===");
    for (si, sec) in doc.sections.iter().enumerate() {
        let total_controls: usize = sec.paragraphs.iter().map(|p| p.controls.len()).sum();
        eprintln!("  Section[{}]: {} paragraphs, {} top-level controls", si, sec.paragraphs.len(), total_controls);
    }
}

/// Helper to print a control briefly at given indent level
fn dump_control_brief(ctrl: &Control, idx: usize, indent: usize) {
    let pad = " ".repeat(indent);
    match ctrl {
        Control::SectionDef(_) => eprintln!("{}Ctrl[{}]: SectionDef", pad, idx),
        Control::ColumnDef(_) => eprintln!("{}Ctrl[{}]: ColumnDef", pad, idx),
        Control::Table(t) => eprintln!("{}Ctrl[{}]: Table ({}x{}, {} cells)", pad, idx, t.row_count, t.col_count, t.cells.len()),
        Control::Shape(s) => {
            let stype = match s.as_ref() {
                crate::model::shape::ShapeObject::Line(_) => "Line",
                crate::model::shape::ShapeObject::Rectangle(_) => "Rectangle",
                crate::model::shape::ShapeObject::Ellipse(_) => "Ellipse",
                crate::model::shape::ShapeObject::Arc(_) => "Arc",
                crate::model::shape::ShapeObject::Polygon(_) => "Polygon",
                crate::model::shape::ShapeObject::Curve(_) => "Curve",
                crate::model::shape::ShapeObject::Group(_) => "Group",
                crate::model::shape::ShapeObject::Picture(_) => "Picture",
                crate::model::shape::ShapeObject::Chart(_) => "Chart",
                crate::model::shape::ShapeObject::Ole(_) => "Ole",
            };
            let has_tb = s.drawing().and_then(|d| d.text_box.as_ref());
            if let Some(tb) = has_tb {
                eprintln!("{}Ctrl[{}]: Shape({}) — TextBox: {} paras", pad, idx, stype, tb.paragraphs.len());
                for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                    let preview: String = tp.text.chars().take(40).collect();
                    eprintln!("{}  TBPara[{}]: text_len={}, text={:?}", pad, tpi, tp.text.len(), preview);
                }
            } else {
                eprintln!("{}Ctrl[{}]: Shape({}) — no textbox", pad, idx, stype);
            }
        }
        Control::Picture(_) => eprintln!("{}Ctrl[{}]: Picture", pad, idx),
        Control::Header(_) => eprintln!("{}Ctrl[{}]: Header", pad, idx),
        Control::Footer(_) => eprintln!("{}Ctrl[{}]: Footer", pad, idx),
        Control::Footnote(f) => eprintln!("{}Ctrl[{}]: Footnote ({} paras)", pad, idx, f.paragraphs.len()),
        Control::Endnote(e) => eprintln!("{}Ctrl[{}]: Endnote ({} paras)", pad, idx, e.paragraphs.len()),
        Control::AutoNumber(a) => eprintln!("{}Ctrl[{}]: AutoNumber({:?})", pad, idx, a.number_type),
        Control::NewNumber(_) => eprintln!("{}Ctrl[{}]: NewNumber", pad, idx),
        Control::PageNumberPos(_) => eprintln!("{}Ctrl[{}]: PageNumberPos", pad, idx),
        Control::Bookmark(b) => eprintln!("{}Ctrl[{}]: Bookmark({:?})", pad, idx, b.name),
        Control::Hyperlink(h) => eprintln!("{}Ctrl[{}]: Hyperlink({:?})", pad, idx, h.url),
        Control::Ruby(_) => eprintln!("{}Ctrl[{}]: Ruby", pad, idx),
        Control::CharOverlap(_) => eprintln!("{}Ctrl[{}]: CharOverlap", pad, idx),
        Control::PageHide(_) => eprintln!("{}Ctrl[{}]: PageHide", pad, idx),
        Control::HiddenComment(hc) => eprintln!("{}Ctrl[{}]: HiddenComment ({} paras)", pad, idx, hc.paragraphs.len()),
        Control::Equation(eq) => eprintln!("{}Ctrl[{}]: Equation(script={:?})", pad, idx, eq.script),
        Control::Field(f) => eprintln!("{}Ctrl[{}]: Field({:?})", pad, idx, f.field_type),
        Control::Form(f) => eprintln!("{}Ctrl[{}]: Form({:?}, name={})", pad, idx, f.form_type, f.name),
        Control::Unknown(u) => eprintln!("{}Ctrl[{}]: Unknown(0x{:08X})", pad, idx, u.ctrl_id),
    }
}
