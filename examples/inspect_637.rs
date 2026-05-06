//! Task #637 Stage 1 분석 도구
//!
//! aift.hwp 페이지 2/3 (cover-style) 미표시 메커니즘 분석.
//! 페이지 host paragraph 의 raw header bytes + control_mask 비교 +
//! 다른 샘플의 cover-style 패턴 enumerate.

use std::fs;
use rhwp::model::control::Control;
use rhwp::model::paragraph::ColumnBreakType;

fn dump_para(label: &str, sec_idx: usize, para_idx: usize, doc: &rhwp::model::document::Document) {
    let sec = match doc.sections.get(sec_idx) {
        Some(s) => s,
        None => { println!("  {}: section {} 없음", label, sec_idx); return; }
    };
    let para = match sec.paragraphs.get(para_idx) {
        Some(p) => p,
        None => { println!("  {}: para {}.{} 없음", label, sec_idx, para_idx); return; }
    };
    let ctrls: Vec<&str> = para.controls.iter().map(|c| match c {
        Control::SectionDef(_) => "SectionDef",
        Control::ColumnDef(_) => "ColumnDef",
        Control::Table(_) => "Table",
        Control::PageNumberPos(_) => "PageNumberPos",
        Control::PageHide(_) => "PageHide",
        Control::NewNumber(_) => "NewNumber",
        Control::Header(_) => "Header",
        Control::Footer(_) => "Footer",
        Control::Footnote(_) => "Footnote",
        Control::Endnote(_) => "Endnote",
        Control::Picture(_) => "Picture",
        Control::Shape(_) => "Shape",
        _ => "Other",
    }).collect();
    println!("\n=== {} ({}.{}): cc={} mask=0x{:08X} ps_id={} style={} break_raw=0x{:02X} ===",
        label, sec_idx, para_idx, para.char_count, para.control_mask,
        para.para_shape_id, para.style_id, para.raw_break_type);
    println!("  text_len={} controls={:?}", para.text.chars().count(), ctrls);
    println!("  raw_header_extra ({} bytes): {:02X?}", para.raw_header_extra.len(), &para.raw_header_extra);
}

fn enumerate_cover_pages(doc: &rhwp::model::document::Document) -> Vec<(usize, usize, u32, u32, u32)> {
    // 반환: (sec_idx, para_idx, table_height_hu, table_width_hu, body_height_estimate)
    // body_height_estimate 는 sec.section_def.page_def 기준
    let mut results = Vec::new();
    for (si, sec) in doc.sections.iter().enumerate() {
        let pd = &sec.section_def.page_def;
        let body_h = if pd.landscape {
            pd.width - pd.margin_top - pd.margin_bottom - pd.margin_header - pd.margin_footer
        } else {
            pd.height - pd.margin_top - pd.margin_bottom - pd.margin_header - pd.margin_footer
        };
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            // single Table 만 있는 paragraph 찾기 (controls 안에 SectionDef/ColumnDef 도 카운트하나
            // 시각적 항목은 Table 만)
            let table_count = para.controls.iter().filter(|c| matches!(c, Control::Table(_))).count();
            let visual_count = para.controls.iter().filter(|c| matches!(c,
                Control::Table(_) | Control::Picture(_) | Control::Shape(_)
            )).count();
            if table_count == 1 && visual_count == 1 && para.text.chars().count() == 0 {
                // 표가 단일 시각 항목, paragraph 텍스트 없음
                if let Some(Control::Table(tbl)) = para.controls.iter().find(|c| matches!(c, Control::Table(_))) {
                    let th = tbl.common.height as u32;
                    let tw = tbl.common.width as u32;
                    let ratio = if body_h > 0 { (th as f64 / body_h as f64) * 100.0 } else { 0.0 };
                    if ratio >= 50.0 && !tbl.common.treat_as_char {
                        // 50% 이상 차지하는 tac=false (block-level) cover-style 후보
                        // 추가 조건: paragraph 가 [쪽나누기] (= 페이지 시작) 인지 (=> items=1 가능성 높음)
                        let pg_break = matches!(para.column_type, ColumnBreakType::Page | ColumnBreakType::Section);
                        results.push((si, pi, th, tw, body_h as u32));
                        println!("  cover-candidate: sec={} para={} tbl_h={} body_h={} ratio={:.1}% tac=false wrap={:?} pg_break={} break_raw=0x{:02X}",
                            si, pi, th, body_h, ratio, tbl.common.text_wrap, pg_break, para.raw_break_type);
                    }
                }
            }
        }
    }
    results
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or("samples/aift.hwp");
    let data = fs::read(path).unwrap_or_else(|e| panic!("read {}: {}", path, e));
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();

    println!("=== File: {} ===", path);
    println!("  sections: {}", doc.sections.len());
    for (si, sec) in doc.sections.iter().enumerate() {
        let pd = &sec.section_def.page_def;
        println!("  sec[{}]: paragraphs={}, page=({}x{}HU={:.0}x{:.0}mm), margin_top={} body_top={}",
            si, sec.paragraphs.len(), pd.width, pd.height,
            pd.width as f64 / 7200.0 * 25.4, pd.height as f64 / 7200.0 * 25.4,
            pd.margin_top, pd.margin_top + pd.margin_header);
    }

    if path.contains("aift") {
        // aift.hwp 의 핵심 paragraphs raw 비교
        dump_para("p1 host (page 1, 표시)", 0, 0, doc);
        dump_para("p2 host (page 2, 미표시) ★", 0, 1, doc);
        dump_para("p3 host (page 3, 미표시) ★", 1, 0, doc);
        dump_para("p6 host (page 6, 표시)", 2, 57, doc);
    }

    println!("\n=== cover-style candidates (single-table-only paragraph, table covers >= 50% body) ===");
    let candidates = enumerate_cover_pages(doc);
    println!("  total: {}", candidates.len());
}
