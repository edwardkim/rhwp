//! Task #574 진단 — exam_science.hwp 머리말 표 셀의 paragraph 별 char_shapes 풀 덤프
//!
//! 검증 가설:
//! - A: HWP CharShape 자체가 bold + black + 33pt (저작 결함, rhwp 무결)
//! - B: parse_char_shape 결함 (color/bold 비트 위치 오류)
//! - C: composed run 의 char_shape_id 누락 (renderer 결함)
//! - D: \u{0015} 마커에 char_shape_id 미할당 (parser/marker)
//!
//! 본 스크립트는 머리말 표 셀의 paragraph 별 char_shapes / 텍스트 / CharShape 속성을
//! 출력해 가설 A/B/C/D 중 본질을 확정한다.
//!
//! 실행: cargo run --release --example inspect_574

use rhwp::model::control::Control;
use rhwp::model::style::CharShape;
use std::fs;

fn main() {
    let path = "samples/exam_science.hwp";
    let data = fs::read(path).unwrap_or_else(|e| panic!("read {} failed: {}", path, e));
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();

    println!("=== exam_science.hwp 머리말 표 셀 CharShape 진단 ===\n");
    println!("CharShape 총 개수: {}", doc.doc_info.char_shapes.len());

    // 본문 default CharShape 비교 (id=9 — section setup paragraph 0.0 의 본문 CS)
    println!("\n--- [쪽번호 후보] CharShape id=0 (TextBox 사각형 내 \"1\") ---");
    if let Some(cs) = doc.doc_info.char_shapes.get(0) {
        print_char_shape(0, cs);
    }
    println!("\n--- [본문 비교] CharShape id=9 (본문 default ratio=95%) ---");
    if let Some(cs) = doc.doc_info.char_shapes.get(9) {
        print_char_shape(9, cs);
    }
    println!("\n--- [본문 비교] CharShape id=32 ---");
    if let Some(cs) = doc.doc_info.char_shapes.get(32) {
        print_char_shape(32, cs);
    }

    // 본문 표 셀 paragraph 0 의 Shape (사각형, InFrontOfText) 내부 TextBox 검사
    println!("\n=== 본문 [6] 표 셀 paragraph 의 TextBox 검사 ===");
    let setup_para0 = &doc.sections[0].paragraphs[0];
    if let Some(rhwp::model::control::Control::Table(t)) = setup_para0.controls.iter()
        .find(|c| matches!(c, rhwp::model::control::Control::Table(_)))
    {
        for (cell_idx, cell) in t.cells.iter().enumerate() {
            for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                for (ci, ctrl) in cp.controls.iter().enumerate() {
                    if let rhwp::model::control::Control::Shape(s) = ctrl {
                        if let Some(d) = s.drawing() {
                            if let Some(tb) = d.text_box.as_ref() {
                                println!("  셀[{}] p[{}] ctrl[{}] Shape({}) TextBox paras={}",
                                    cell_idx, cpi, ci, s.shape_name(), tb.paragraphs.len());
                                for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                                    let tp_text: String = tp.text.chars().take(50).collect();
                                    println!("    tb_p[{}]: cc={} text=\"{}\" char_shapes={}",
                                        tpi, tp.char_count, tp_text, tp.char_shapes.len());
                                    for csref in &tp.char_shapes {
                                        if let Some(cs) = doc.doc_info.char_shapes.get(csref.char_shape_id as usize) {
                                            println!("      pos={} cs_id={} → size={}({:.1}pt) bold={} color=#{:06X} ratio[0]={}% font_id[0]={}",
                                                csref.start_pos, csref.char_shape_id,
                                                cs.base_size, cs.base_size as f64 / 100.0,
                                                cs.bold, cs.text_color & 0x00FF_FFFF,
                                                cs.ratios[0], cs.font_ids[0]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 바탕쪽 (master page) 검사
    let sd = &doc.sections[0].section_def;
    println!("\n=== 구역 0 / 바탕쪽 (master_pages={}) ===", sd.master_pages.len());
    for (mi, mp) in sd.master_pages.iter().enumerate() {
        println!("\n>>> 바탕쪽[{}] apply_to={:?} is_ext={} overlap={} ext_flags=0x{:04X}",
            mi, mp.apply_to, mp.is_extension, mp.overlap, mp.ext_flags);
        for (mppi, mpp) in mp.paragraphs.iter().enumerate() {
            inspect_hf_paragraph(doc, "바탕p", mppi, mpp, 1);
        }
    }

    // 구역 0 / paragraph 0.0 의 controls 에서 머리말 추출
    let setup_para = &doc.sections[0].paragraphs[0];
    println!("\n=== 구역 0 / paragraph 0.0 (controls={}) ===", setup_para.controls.len());

    for (ci, ctrl) in setup_para.controls.iter().enumerate() {
        match ctrl {
            Control::Header(h) => {
                println!("\n>>> controls[{}] 머리말 (apply_to={:?}, paragraphs={})",
                    ci, h.apply_to, h.paragraphs.len());
                for (hpi, hp) in h.paragraphs.iter().enumerate() {
                    inspect_hf_paragraph(doc, "머리말", hpi, hp, 0);
                }
            }
            Control::Footer(f) => {
                println!("\n>>> controls[{}] 꼬리말 (apply_to={:?}, paragraphs={})",
                    ci, f.apply_to, f.paragraphs.len());
                for (fpi, fp) in f.paragraphs.iter().enumerate() {
                    inspect_hf_paragraph(doc, "꼬리말", fpi, fp, 0);
                }
            }
            Control::AutoNumber(an) => {
                println!("\n>>> controls[{}] AutoNumber (type={:?}, format={}, num={})",
                    ci, an.number_type, an.format, an.assigned_number);
            }
            _ => {}
        }
    }
}

fn inspect_hf_paragraph(
    doc: &rhwp::model::document::Document,
    label: &str,
    pi: usize,
    para: &rhwp::model::paragraph::Paragraph,
    indent_level: usize,
) {
    let pad = "  ".repeat(indent_level);
    println!("{}[{} p[{}]] cc={} text_len={} controls={} char_shapes={}",
        pad, label, pi, para.char_count, para.text.chars().count(),
        para.controls.len(), para.char_shapes.len());

    // raw text bytes (control char 가시화)
    let text_bytes: Vec<String> = para.text.chars().map(|c| {
        let cu = c as u32;
        if cu < 0x20 || (0x10..=0x18).contains(&cu) {
            format!("\\u{{{:04X}}}", cu)
        } else {
            c.to_string()
        }
    }).collect();
    println!("{}  text=[{}]", pad, text_bytes.join(""));

    println!("{}  char_shapes:", pad);
    for cs_ref in &para.char_shapes {
        let cs = doc.doc_info.char_shapes.get(cs_ref.char_shape_id as usize);
        let summary = cs.map(|c| format!(
            "size={}({:.1}pt) bold={} color=#{:06X} ratio[0]={}% font_id[0]={}",
            c.base_size, c.base_size as f64 / 100.0,
            c.bold,
            c.text_color & 0x00FF_FFFF,
            c.ratios[0],
            c.font_ids[0],
        )).unwrap_or_else(|| "(invalid id)".to_string());
        println!("{}    pos={} cs_id={} → {}", pad, cs_ref.start_pos, cs_ref.char_shape_id, summary);
    }

    // 표 컨트롤 안 셀 paragraph 재귀
    for (ci, ctrl) in para.controls.iter().enumerate() {
        match ctrl {
            Control::Table(t) => {
                println!("{}  controls[{}] 표 ({}행×{}열, cells={})",
                    pad, ci, t.row_count, t.col_count, t.cells.len());
                for (cell_idx, cell) in t.cells.iter().enumerate() {
                    let cell_text: String = cell.paragraphs.iter()
                        .map(|p| p.text.clone()).collect::<Vec<_>>().join("|");
                    println!("{}    셀[{}] r={} c={} paras={} text=\"{}\"",
                        pad, cell_idx, cell.row, cell.col,
                        cell.paragraphs.len(),
                        cell_text.chars().take(60).collect::<String>());
                    for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                        inspect_hf_paragraph(doc, "셀p", cpi, cp, indent_level + 3);
                    }
                }
            }
            Control::Shape(_) => {
                println!("{}  controls[{}] Shape", pad, ci);
            }
            Control::AutoNumber(an) => {
                println!("{}  controls[{}] AutoNumber type={:?} fmt={} num={}",
                    pad, ci, an.number_type, an.format, an.assigned_number);
            }
            _ => {
                println!("{}  controls[{}] {:?}", pad, ci, std::mem::discriminant(ctrl));
            }
        }
    }
}

fn print_char_shape(id: usize, cs: &CharShape) {
    println!("  CharShape[{}]:", id);
    println!("    base_size={} ({:.1}pt)", cs.base_size, cs.base_size as f64 / 100.0);
    println!("    bold={}, italic={}", cs.bold, cs.italic);
    println!("    text_color=#{:08X} (rgb=#{:06X})", cs.text_color, cs.text_color & 0x00FF_FFFF);
    println!("    ratios={:?}", cs.ratios);
    println!("    spacings={:?}", cs.spacings);
    println!("    relative_sizes={:?}", cs.relative_sizes);
    println!("    font_ids={:?}", cs.font_ids);
    println!("    attr=0x{:08X}", cs.attr);
    println!("    underline={:?}, outline={}, shadow={}", cs.underline_type, cs.outline_type, cs.shadow_type);
}
