//! `<hp:tbl>` 표 직렬화.
//!
//! Stage 3 (#182): `Control::Table` IR → `<hp:tbl>` + `<hp:tr>` + `<hp:tc>` + `<hp:subList>` + 문단 재귀.
//!
//! 속성·자식 순서는 한컴 OWPML 공식 (hancom-io/hwpx-owpml-model, Apache 2.0)
//! `Class/Para/TableType.cpp` 의 `WriteElement()`, `InitMap()` 기준:
//!
//! ### `<hp:tbl>` 속성 순서 (부모 AbstractShapeObjectType + 자신)
//! id, zOrder, numberingType, textWrap, textFlow, lock, dropcapstyle,
//! pageBreak, repeatHeader, rowCnt, colCnt, cellSpacing, borderFillIDRef, noAdjust
//!
//! ### `<hp:tbl>` 자식 순서
//! sz, pos, outMargin, (caption, shapeComment, parameterset, metaTag — 옵셔널),
//! inMargin, (cellzoneList — 옵셔널), tr (루프), (label — 옵셔널)
//!
//! ### `<hp:tc>` 속성 순서
//! name, header, hasMargin, protect, editable, dirty, borderFillIDRef
//!
//! ### `<hp:tc>` 자식 순서
//! subList, cellAddr, cellSpan, cellSz, cellMargin
//!
//! ## 중요: table.attr 비트 연산 금지
//!
//! HWPX에서 `table.attr` 는 0인 경우가 많으므로 비트 연산으로 `textWrap/textFlow/pageBreak` 등을
//! 추출하면 안 된다. 반드시 `table.common.text_wrap`, `table.page_break` 등 파싱된 IR 필드를 사용.

use std::io::Write;

use quick_xml::Writer;

use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, TextFlow, TextWrap, VertAlign, VertRelTo,
};
use crate::model::table::{Cell, Table, TablePageBreak, VerticalAlign};

use super::context::SerializeContext;
use super::section::{render_hp_p_open, render_paragraph_parts};
use super::utils::{empty_tag, end_tag, start_tag, start_tag_attrs};
use super::SerializeError;

/// `<hp:tbl>` 직렬화.
pub fn write_table<W: Write>(
    w: &mut Writer<W>,
    table: &Table,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    // borderFillIDRef 참조 등록 (assert_all_refs_resolved 검증 대상)
    ctx.border_fill_ids.reference(table.border_fill_id);
    for zone in &table.zones {
        ctx.border_fill_ids.reference(zone.border_fill_id);
    }
    for cell in &table.cells {
        ctx.border_fill_ids.reference(cell.border_fill_id);
    }

    // --- <hp:tbl> 시작 태그 + 속성 ---
    let id_str = table.common.instance_id.to_string();
    let z_order = table.common.z_order.to_string();
    let text_wrap = text_wrap_str(table.common.text_wrap);
    let text_flow = text_flow_str(table.common.text_flow);
    let lock = bool01(false);
    let page_break = table_page_break_str(table.page_break);
    let repeat_header = bool01(table.repeat_header);
    let row_cnt = table.row_count.to_string();
    let col_cnt = table.col_count.to_string();
    let cell_spacing = table.cell_spacing.to_string();
    let border_fill_id_ref = table.border_fill_id.to_string();
    let no_adjust = bool01((table.attr | table.raw_table_record_attr) & 0x08 != 0);

    start_tag_attrs(
        w,
        "hp:tbl",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", "TABLE"),
            ("textWrap", text_wrap),
            ("textFlow", text_flow),
            ("lock", lock),
            ("dropcapstyle", "None"),
            ("pageBreak", page_break),
            ("repeatHeader", repeat_header),
            ("rowCnt", &row_cnt),
            ("colCnt", &col_cnt),
            ("cellSpacing", &cell_spacing),
            ("borderFillIDRef", &border_fill_id_ref),
            ("noAdjust", no_adjust),
        ],
    )?;

    // --- 자식: sz, pos, outMargin, (caption — 옵셔널), inMargin, tr[] ---
    write_sz(w, &table.common)?;
    write_pos(w, &table.common)?;
    write_out_margin(w, table)?;
    if let Some(caption) = &table.caption {
        write_caption(w, caption, ctx)?;
    }
    write_in_margin(w, table)?;
    // cellzoneList: inMargin 다음, tr 앞 (OWPML 자식 순서). 셀 영역 테두리/배경
    // 오버레이를 정의한다. 파서가 table.zones 로 보존하므로 그대로 되살린다.
    write_cellzone_list(w, table)?;

    // tr[]: 행 단위 반복. 각 행에 속한 셀 (cell.row == r) 을 col 오름차순으로 출력.
    for row_idx in 0..table.row_count {
        start_tag(w, "hp:tr")?;
        let mut row_cells: Vec<&Cell> = table.cells.iter().filter(|c| c.row == row_idx).collect();
        row_cells.sort_by_key(|c| c.col);
        for cell in row_cells {
            write_cell(w, cell, ctx)?;
        }
        end_tag(w, "hp:tr")?;
    }

    end_tag(w, "hp:tbl")?;
    Ok(())
}

fn write_sz<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let width = c.width.to_string();
    let height = c.height.to_string();
    empty_tag(
        w,
        "hp:sz",
        &[
            ("width", &width),
            ("widthRelTo", "ABSOLUTE"),
            ("height", &height),
            ("heightRelTo", "ABSOLUTE"),
            ("protect", "0"),
        ],
    )
}

fn write_pos<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let treat = bool01(c.treat_as_char);
    let vert_offset = c.vertical_offset.to_string();
    let horz_offset = c.horizontal_offset.to_string();
    // [#1594] holdAnchorAndSO 는 IR(prevent_page_break)을 보존한다. 종전 "0" 하드코딩은
    // 페이지 하단 앵커 개체(발신명의 footer 등)의 1→0 드롭으로 한글 페이지 붕괴를 유발했다.
    let hold = bool01(c.prevent_page_break != 0);
    empty_tag(
        w,
        "hp:pos",
        &[
            ("treatAsChar", treat),
            ("affectLSpacing", "0"),
            // [#1637] flowWithText 는 IR(flow_with_text)을 보존한다. 종전 "1" 하드코딩은
            // treatAsChar 표의 1→0 케이스에서 0→1 드롭으로 표 partial-split 임계를 흔들어
            // 페이지네이션이 달라졌다(IR-invisible). #1594 holdAnchorAndSO 와 동형.
            ("flowWithText", bool01(c.flow_with_text)),
            ("allowOverlap", "0"),
            ("holdAnchorAndSO", hold),
            ("vertRelTo", vert_rel_to_str(c.vert_rel_to)),
            ("horzRelTo", horz_rel_to_str(c.horz_rel_to)),
            ("vertAlign", vert_align_str(c.vert_align)),
            ("horzAlign", horz_align_str(c.horz_align)),
            ("vertOffset", &vert_offset),
            ("horzOffset", &horz_offset),
        ],
    )
}

fn write_out_margin<W: Write>(w: &mut Writer<W>, t: &Table) -> Result<(), SerializeError> {
    let left = t.outer_margin_left.to_string();
    let right = t.outer_margin_right.to_string();
    let top = t.outer_margin_top.to_string();
    let bottom = t.outer_margin_bottom.to_string();
    empty_tag(
        w,
        "hp:outMargin",
        &[
            ("left", &left),
            ("right", &right),
            ("top", &top),
            ("bottom", &bottom),
        ],
    )
}

fn write_in_margin<W: Write>(w: &mut Writer<W>, t: &Table) -> Result<(), SerializeError> {
    let left = t.padding.left.to_string();
    let right = t.padding.right.to_string();
    let top = t.padding.top.to_string();
    let bottom = t.padding.bottom.to_string();
    empty_tag(
        w,
        "hp:inMargin",
        &[
            ("left", &left),
            ("right", &right),
            ("top", &top),
            ("bottom", &bottom),
        ],
    )
}

/// `<hp:cellzoneList>` + `<hp:cellzone>` 자식들. 셀 영역(범위) 단위 테두리/배경
/// 오버레이. `table.zones` 가 비어 있으면 원본에 없었던 것이므로 미방출.
/// 속성 순서는 OWPML 관찰 순서: startRowAddr, startColAddr, endRowAddr,
/// endColAddr, borderFillIDRef. borderFillIDRef 는 셀과 동일하게 raw 값(오프셋
/// 없음)으로 쓴다 (parser `parse_u16`, 헤더 borderFill id 와 1:1).
fn write_cellzone_list<W: Write>(w: &mut Writer<W>, t: &Table) -> Result<(), SerializeError> {
    if t.zones.is_empty() {
        return Ok(());
    }
    start_tag(w, "hp:cellzoneList")?;
    for zone in &t.zones {
        let start_row = zone.start_row.to_string();
        let start_col = zone.start_col.to_string();
        let end_row = zone.end_row.to_string();
        let end_col = zone.end_col.to_string();
        let border_fill = zone.border_fill_id.to_string();
        empty_tag(
            w,
            "hp:cellzone",
            &[
                ("startRowAddr", &start_row),
                ("startColAddr", &start_col),
                ("endRowAddr", &end_row),
                ("endColAddr", &end_col),
                ("borderFillIDRef", &border_fill),
            ],
        )?;
    }
    end_tag(w, "hp:cellzoneList")
}

fn write_cell<W: Write>(
    w: &mut Writer<W>,
    cell: &Cell,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    let name = cell.field_name.as_deref().unwrap_or("");
    let header = bool01(cell.is_header);
    let has_margin = bool01(cell.apply_inner_margin);
    let protect = bool01(cell.cell_protect());
    let editable = bool01(cell.editable_in_form());
    let border_ref = cell.border_fill_id.to_string();

    start_tag_attrs(
        w,
        "hp:tc",
        &[
            ("name", name),
            ("header", header),
            ("hasMargin", has_margin),
            ("protect", protect),
            ("editable", editable),
            ("dirty", "0"),
            ("borderFillIDRef", &border_ref),
        ],
    )?;

    // 자식 순서: subList, cellAddr, cellSpan, cellSz, cellMargin
    write_sub_list(w, cell, ctx)?;
    write_cell_addr(w, cell)?;
    write_cell_span(w, cell)?;
    write_cell_sz(w, cell)?;
    write_cell_margin(w, cell)?;

    end_tag(w, "hp:tc")?;
    Ok(())
}

fn write_sub_list<W: Write>(
    w: &mut Writer<W>,
    cell: &Cell,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    start_tag_attrs(
        w,
        "hp:subList",
        &[
            ("id", ""),
            (
                "textDirection",
                if cell.text_direction == 1 {
                    "VERTICAL"
                } else {
                    "HORIZONTAL"
                },
            ),
            ("lineWrap", "BREAK"),
            ("vertAlign", cell_vert_align_str(cell.vertical_align)),
            ("linkListIDRef", "0"),
            ("linkListNextIDRef", "0"),
            ("textWidth", "0"),
            ("textHeight", "0"),
            ("hasTextRef", "0"),
            ("hasNumRef", "0"),
        ],
    )?;

    write_sub_list_paragraphs(w, &cell.paragraphs, ctx)?;

    end_tag(w, "hp:subList")?;
    Ok(())
}

/// subList 내부 문단 시퀀스 방출 — 셀(#1379)과 표 캡션(#1387)이 공유.
///
/// 본문과 동일한 공유 직렬화 경로(render_paragraph_parts)로 컨트롤 슬롯(표 재귀
/// 포함) 방출 + run 분할 + lineseg IR 보존/fallback (#1379 2단계).
/// sub_list_depth: subList 경로 한정 colPr 인라인 방출 스코프 (#1379 3단계).
fn write_sub_list_paragraphs<W: Write>(
    w: &mut Writer<W>,
    paragraphs: &[crate::model::paragraph::Paragraph],
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    ctx.sub_list_depth += 1;
    let mut vert_cursor: u32 = 0;
    for para in paragraphs {
        ctx.para_shape_ids.reference(para.para_shape_id);
        ctx.style_ids.reference(para.style_id as u16);

        let (runs, linesegs, advance) = render_paragraph_parts(para, vert_cursor, ctx);
        vert_cursor = advance;
        let mut p_xml = render_hp_p_open(para, ctx.next_para_id());
        p_xml.push_str(&runs);
        p_xml.push_str(&linesegs);
        p_xml.push_str("</hp:p>");
        w.get_mut()
            .write_all(p_xml.as_bytes())
            .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    }
    ctx.sub_list_depth -= 1;
    Ok(())
}

/// `<hp:caption>` 직렬화 (#1387) — 자식 순서상 outMargin 과 inMargin 사이 (모듈 doc).
///
/// 속성 순서는 한컴 실물(ta-pic-001-r) 기준: side, fullSz, width, gap, lastWidth.
/// 캡션 subList 속성은 파서가 적재하지 않으며 samples/hwpx 전수 17건 동일
/// (vertAlign=TOP lineWrap=BREAK textDirection=HORIZONTAL — 1단계 측정) — 실물 고정값 방출.
/// 그림/도형/묶음 캡션(#1403)도 동일 경로를 공유한다 (pub(super)).
pub(super) fn write_caption<W: Write>(
    w: &mut Writer<W>,
    caption: &crate::model::shape::Caption,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    use crate::model::shape::CaptionDirection;

    let side = match caption.direction {
        CaptionDirection::Left => "LEFT",
        CaptionDirection::Right => "RIGHT",
        CaptionDirection::Top => "TOP",
        CaptionDirection::Bottom => "BOTTOM",
    };
    let full_sz = bool01(caption.include_margin);
    let width = caption.width.to_string();
    let gap = caption.spacing.to_string();
    let last_width = caption.max_width.to_string();
    start_tag_attrs(
        w,
        "hp:caption",
        &[
            ("side", side),
            ("fullSz", full_sz),
            ("width", &width),
            ("gap", &gap),
            ("lastWidth", &last_width),
        ],
    )?;
    start_tag_attrs(
        w,
        "hp:subList",
        &[
            ("id", ""),
            ("textDirection", "HORIZONTAL"),
            ("lineWrap", "BREAK"),
            ("vertAlign", "TOP"),
            ("linkListIDRef", "0"),
            ("linkListNextIDRef", "0"),
            ("textWidth", "0"),
            ("textHeight", "0"),
            ("hasTextRef", "0"),
            ("hasNumRef", "0"),
        ],
    )?;
    write_sub_list_paragraphs(w, &caption.paragraphs, ctx)?;
    end_tag(w, "hp:subList")?;
    end_tag(w, "hp:caption")?;
    Ok(())
}

fn write_cell_addr<W: Write>(w: &mut Writer<W>, cell: &Cell) -> Result<(), SerializeError> {
    let col = cell.col.to_string();
    let row = cell.row.to_string();
    empty_tag(w, "hp:cellAddr", &[("colAddr", &col), ("rowAddr", &row)])
}

fn write_cell_span<W: Write>(w: &mut Writer<W>, cell: &Cell) -> Result<(), SerializeError> {
    let cs = cell.col_span.max(1).to_string();
    let rs = cell.row_span.max(1).to_string();
    empty_tag(w, "hp:cellSpan", &[("colSpan", &cs), ("rowSpan", &rs)])
}

fn write_cell_sz<W: Write>(w: &mut Writer<W>, cell: &Cell) -> Result<(), SerializeError> {
    let w_s = cell.width.to_string();
    let h_s = cell.height.to_string();
    empty_tag(w, "hp:cellSz", &[("width", &w_s), ("height", &h_s)])
}

fn write_cell_margin<W: Write>(w: &mut Writer<W>, cell: &Cell) -> Result<(), SerializeError> {
    let l = cell.padding.left.to_string();
    let r = cell.padding.right.to_string();
    let t = cell.padding.top.to_string();
    let b = cell.padding.bottom.to_string();
    empty_tag(
        w,
        "hp:cellMargin",
        &[("left", &l), ("right", &r), ("top", &t), ("bottom", &b)],
    )
}

// ---------- enum 변환 헬퍼 ----------

fn bool01(b: bool) -> &'static str {
    if b {
        "1"
    } else {
        "0"
    }
}

fn text_wrap_str(w: TextWrap) -> &'static str {
    use TextWrap::*;
    match w {
        Square => "SQUARE",
        Tight => "TIGHT",
        Through => "THROUGH",
        TopAndBottom => "TOP_AND_BOTTOM",
        BehindText => "BEHIND_TEXT",
        InFrontOfText => "IN_FRONT_OF_TEXT",
    }
}

fn text_flow_str(f: TextFlow) -> &'static str {
    match f {
        TextFlow::BothSides => "BOTH_SIDES",
        TextFlow::LeftOnly => "LEFT_ONLY",
        TextFlow::RightOnly => "RIGHT_ONLY",
        TextFlow::LargestOnly => "LARGEST_ONLY",
    }
}

fn table_page_break_str(pb: TablePageBreak) -> &'static str {
    use TablePageBreak::*;
    // HWPX 문자열 매핑은 파서(parser/hwpx/section.rs `pageBreak`)의 정확한 역이어야
    // HWPX→HWPX 왕복이 충실하다. 파서는 한컴 관찰 기준으로
    // `"TABLE"→CellBreak`, `"CELL"/"ROW"→RowBreak` 로 매핑하므로 (한컴 HWPX
    // "CELL" = HWP5 row-break bit 1 = RowBreak), 직렬화도 그 역으로 방출한다.
    // (이전 구현은 CellBreak→"CELL", RowBreak→"TABLE" 로 파서와 불일치해 왕복 시
    //  CELL↔TABLE 이 뒤바뀌었다.) enum→HWP5 bit 매핑(hwpx_to_hwp.rs)은 enum 을
    // 직접 쓰므로 이 변경의 영향을 받지 않는다.
    match pb {
        None => "NONE",
        CellBreak => "TABLE",
        RowBreak => "CELL",
    }
}

fn vert_rel_to_str(v: VertRelTo) -> &'static str {
    use VertRelTo::*;
    match v {
        Paper => "PAPER",
        Page => "PAGE",
        Para => "PARA",
    }
}

fn horz_rel_to_str(h: HorzRelTo) -> &'static str {
    use HorzRelTo::*;
    match h {
        Paper => "PAPER",
        Page => "PAGE",
        Column => "COLUMN",
        Para => "PARA",
    }
}

fn vert_align_str(v: VertAlign) -> &'static str {
    use VertAlign::*;
    match v {
        Top => "TOP",
        Center => "CENTER",
        Bottom => "BOTTOM",
        Inside => "INSIDE",
        Outside => "OUTSIDE",
    }
}

fn horz_align_str(h: HorzAlign) -> &'static str {
    use HorzAlign::*;
    match h {
        Left => "LEFT",
        Center => "CENTER",
        Right => "RIGHT",
        Inside => "INSIDE",
        Outside => "OUTSIDE",
    }
}

fn cell_vert_align_str(v: VerticalAlign) -> &'static str {
    use VerticalAlign::*;
    match v {
        Top => "TOP",
        Center => "CENTER",
        Bottom => "BOTTOM",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::Document;
    use crate::model::paragraph::Paragraph;
    use crate::model::table::{Cell, Table};
    use crate::serializer::hwpx::context::SerializeContext;

    fn empty_table(rows: u16, cols: u16) -> Table {
        let mut t = Table::default();
        t.row_count = rows;
        t.col_count = cols;
        for r in 0..rows {
            for c in 0..cols {
                let mut cell = Cell::default();
                cell.col = c;
                cell.row = r;
                cell.col_span = 1;
                cell.row_span = 1;
                cell.width = 1000;
                cell.height = 300;
                cell.paragraphs.push(Paragraph::default());
                t.cells.push(cell);
            }
        }
        t.rebuild_grid();
        t
    }

    fn serialize(table: &Table) -> String {
        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_table(&mut w, table, &mut ctx).expect("write_table");
        String::from_utf8(w.into_inner()).unwrap()
    }

    fn cs(start_pos: u32, char_shape_id: u32) -> crate::model::paragraph::CharShapeRef {
        crate::model::paragraph::CharShapeRef {
            start_pos,
            char_shape_id,
        }
    }

    // ---------- #1594: holdAnchorAndSO 보존 ----------

    #[test]
    fn task1594_hold_anchor_preserved() {
        // [#1594] holdAnchorAndSO 는 IR(common.prevent_page_break)을 보존해야 한다.
        // 현재 직렬화가 "0" 하드코딩 → 1→0 드롭(페이지 하단 앵커 개체에서 페이지 붕괴 원인). RED.
        let mut t = empty_table(1, 1);
        t.common.prevent_page_break = 1;
        let xml = serialize(&t);
        assert!(
            xml.contains(r#"holdAnchorAndSO="1""#),
            "holdAnchorAndSO 가 IR 값(1)으로 방출돼야 한다(현재 0 하드코딩): {}",
            &xml[..xml.len().min(500)]
        );
    }

    #[test]
    fn task1594_hold_anchor_zero_when_unset() {
        // prevent_page_break=0 이면 holdAnchorAndSO="0" (기존 동작 보존).
        let t = empty_table(1, 1);
        let xml = serialize(&t);
        assert!(
            xml.contains(r#"holdAnchorAndSO="0""#),
            "기본 0: {}",
            &xml[..xml.len().min(300)]
        );
    }

    // ---------- #1387: write_caption — 표 캡션 직렬화 ----------

    fn caption_with_text(text: &str) -> crate::model::shape::Caption {
        let mut para = Paragraph::default();
        para.text = text.to_string();
        let mut caption = crate::model::shape::Caption::default();
        caption.width = 8504;
        caption.spacing = 850;
        caption.max_width = 47624;
        caption.paragraphs.push(para);
        caption
    }

    #[test]
    fn task1387_caption_attrs_reflect_ir() {
        // 속성 5종 역매핑 + 한컴 실물 순서(side, fullSz, width, gap, lastWidth).
        let mut t = empty_table(1, 1);
        t.caption = Some(caption_with_text("캡션"));
        let xml = serialize(&t);
        assert!(
            xml.contains(
                r#"<hp:caption side="BOTTOM" fullSz="0" width="8504" gap="850" lastWidth="47624">"#
            ),
            "caption 속성이 IR 값·실물 순서로 방출되어야 함: {}",
            xml
        );
        assert!(
            xml.contains("<hp:t>캡션</hp:t>"),
            "캡션 subList 문단 텍스트가 방출되어야 함"
        );
        // 자식 순서: outMargin → caption → inMargin (모듈 doc).
        let om = xml.find("<hp:outMargin").unwrap();
        let cp = xml.find("<hp:caption").unwrap();
        let im = xml.find("<hp:inMargin").unwrap();
        assert!(om < cp && cp < im, "caption 은 outMargin 과 inMargin 사이");
    }

    #[test]
    fn task1387_caption_side_reflects_direction() {
        use crate::model::shape::CaptionDirection;
        for (dir, expected) in [
            (CaptionDirection::Left, r#"side="LEFT""#),
            (CaptionDirection::Right, r#"side="RIGHT""#),
            (CaptionDirection::Top, r#"side="TOP""#),
            (CaptionDirection::Bottom, r#"side="BOTTOM""#),
        ] {
            let mut t = empty_table(1, 1);
            let mut c = caption_with_text("c");
            c.direction = dir;
            t.caption = Some(c);
            let xml = serialize(&t);
            assert!(xml.contains(expected), "{:?} → {}", dir, expected);
        }
    }

    #[test]
    fn task1387_caption_paragraph_controls_emitted() {
        // 캡션 문단 내 인라인 컨트롤(autoNum — ta-pic-001-r 실물 패턴)이 공유 경로로 방출.
        let mut para = Paragraph::default();
        para.text = "\u{0}그림".to_string(); // 확장 컨트롤 문자 위치 0 + 텍스트
        para.char_offsets = vec![0, 1, 2];
        let auto_num = crate::model::control::AutoNumber {
            number_type: crate::model::control::AutoNumberType::Picture,
            ..Default::default()
        };
        para.controls
            .push(crate::model::control::Control::AutoNumber(auto_num));
        let mut caption = crate::model::shape::Caption::default();
        caption.paragraphs.push(para);
        let mut t = empty_table(1, 1);
        t.caption = Some(caption);
        let xml = serialize(&t);
        assert!(
            xml.contains("<hp:autoNum"),
            "캡션 내 autoNum 컨트롤이 방출되어야 함: {}",
            xml
        );
        assert!(
            xml.contains(r#"numType="PICTURE""#),
            "그림 번호는 한컴 실물 표기 PICTURE 로 방출 — FIGURE 는 한컴 미인식(#1387 판정): {}",
            xml
        );
    }

    #[test]
    fn task1387_no_caption_no_emission() {
        let t = empty_table(1, 1);
        let xml = serialize(&t);
        assert!(
            !xml.contains("<hp:caption"),
            "caption 없는 표는 hp:caption 미방출 (기존 동작 무변화)"
        );
    }

    #[test]
    fn tbl_root_attrs_in_canonical_order() {
        let t = empty_table(2, 3);
        let xml = serialize(&t);
        assert!(xml.contains("<hp:tbl "), "should emit <hp:tbl>: {}", xml);
        // id → zOrder → numberingType → textWrap → textFlow → lock → dropcapstyle →
        // pageBreak → repeatHeader → rowCnt → colCnt → cellSpacing → borderFillIDRef → noAdjust
        let ip = xml.find("id=").unwrap();
        let zp = xml.find("zOrder=").unwrap();
        let nt = xml.find("numberingType=").unwrap();
        let tw = xml.find("textWrap=").unwrap();
        let tf = xml.find("textFlow=").unwrap();
        let rc = xml.find("rowCnt=").unwrap();
        let cc = xml.find("colCnt=").unwrap();
        let bf = xml.find("borderFillIDRef=").unwrap();
        let na = xml.find("noAdjust=").unwrap();
        assert!(
            ip < zp && zp < nt && nt < tw && tw < tf && tf < rc && rc < cc && cc < bf && bf < na
        );
    }

    #[test]
    fn tr_count_matches_row_count() {
        let t = empty_table(4, 2);
        let xml = serialize(&t);
        assert_eq!(xml.matches("<hp:tr>").count(), 4);
    }

    #[test]
    fn cellzone_list_omitted_when_no_zones() {
        // 셀 영역이 없으면(원본에 cellzoneList 부재) 출력하지 않는다.
        let t = empty_table(2, 2);
        assert!(t.zones.is_empty());
        let xml = serialize(&t);
        assert!(
            !xml.contains("cellzone"),
            "zones 없음 → cellzoneList 미출력: {xml}"
        );
    }

    #[test]
    fn cellzone_list_serialized_between_inmargin_and_tr() {
        // [Finding 13] 셀 영역 테두리/배경(cellzone)이 inMargin 다음, 첫 tr 앞에
        // 정확한 속성 순서로 복원되어야 한다(종전엔 IR 에만 있고 직렬화 누락).
        use crate::model::table::TableZone;
        let mut t = empty_table(15, 12);
        t.zones.push(TableZone {
            start_row: 2,
            start_col: 0,
            end_row: 14,
            end_col: 11,
            border_fill_id: 5,
        });
        let xml = serialize(&t);
        assert!(
            xml.contains(
                r#"<hp:cellzoneList><hp:cellzone startRowAddr="2" startColAddr="0" endRowAddr="14" endColAddr="11" borderFillIDRef="5"/></hp:cellzoneList>"#
            ),
            "cellzone 정확 복원 실패: {xml}"
        );
        // 자식 순서: inMargin < cellzoneList < 첫 tr
        let im = xml.find("<hp:inMargin").expect("inMargin");
        let cz = xml.find("<hp:cellzoneList>").expect("cellzoneList");
        let tr = xml.find("<hp:tr>").expect("tr");
        assert!(
            im < cz && cz < tr,
            "순서 inMargin<cellzoneList<tr 위반: {xml}"
        );
    }

    #[test]
    fn tc_count_matches_cell_count() {
        let t = empty_table(2, 3);
        let xml = serialize(&t);
        assert_eq!(xml.matches("<hp:tc ").count(), 6);
    }

    #[test]
    fn cells_have_canonical_child_order() {
        let t = empty_table(1, 1);
        let xml = serialize(&t);
        // subList → cellAddr → cellSpan → cellSz → cellMargin
        let sl = xml.find("<hp:subList ").unwrap();
        let ca = xml.find("<hp:cellAddr ").unwrap();
        let cs = xml.find("<hp:cellSpan ").unwrap();
        let cz = xml.find("<hp:cellSz ").unwrap();
        let cm = xml.find("<hp:cellMargin ").unwrap();
        assert!(sl < ca && ca < cs && cs < cz && cz < cm);
    }

    #[test]
    fn cell_addr_reflects_coordinates() {
        let t = empty_table(2, 2);
        let xml = serialize(&t);
        assert!(xml.contains(r#"<hp:cellAddr colAddr="0" rowAddr="0"/>"#));
        assert!(xml.contains(r#"<hp:cellAddr colAddr="1" rowAddr="0"/>"#));
        assert!(xml.contains(r#"<hp:cellAddr colAddr="0" rowAddr="1"/>"#));
        assert!(xml.contains(r#"<hp:cellAddr colAddr="1" rowAddr="1"/>"#));
    }

    #[test]
    fn cell_span_defaults_to_one() {
        let t = empty_table(1, 1);
        let xml = serialize(&t);
        assert!(xml.contains(r#"<hp:cellSpan colSpan="1" rowSpan="1"/>"#));
    }

    #[test]
    fn border_fill_id_ref_registered_in_ctx() {
        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut t = empty_table(1, 1);
        t.border_fill_id = 99;
        t.cells[0].border_fill_id = 99;
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_table(&mut w, &t, &mut ctx).unwrap();
        // 99 는 등록되지 않은 borderFill → unresolved
        assert!(ctx.border_fill_ids.unresolved().contains(&99u16));
    }

    #[test]
    fn text_flow_default_is_both_sides() {
        let t = empty_table(1, 1);
        let xml = serialize(&t);
        assert!(xml.contains(r#"textFlow="BOTH_SIDES""#), "{}", xml);
    }

    #[test]
    fn text_flow_left_only_serialized() {
        let mut t = empty_table(1, 1);
        t.common.text_flow = TextFlow::LeftOnly;
        let xml = serialize(&t);
        assert!(xml.contains(r#"textFlow="LEFT_ONLY""#), "{}", xml);
    }

    #[test]
    fn text_flow_right_only_serialized() {
        let mut t = empty_table(1, 1);
        t.common.text_flow = TextFlow::RightOnly;
        let xml = serialize(&t);
        assert!(xml.contains(r#"textFlow="RIGHT_ONLY""#), "{}", xml);
    }

    #[test]
    fn text_flow_largest_only_serialized() {
        let mut t = empty_table(1, 1);
        t.common.text_flow = TextFlow::LargestOnly;
        let xml = serialize(&t);
        assert!(xml.contains(r#"textFlow="LARGEST_ONLY""#), "{}", xml);
    }

    #[test]
    fn cell_paragraph_ids_are_globally_unique() {
        // 2×2 표 = 셀 4개, 각 셀에 문단 1개 → id="0", id="1", id="2", id="3"
        let t = empty_table(2, 2);
        let xml = serialize(&t);
        assert_eq!(
            xml.matches(r#"<hp:p id="0""#).count(),
            1,
            "셀 문단 id=0 이 중복됨: {}",
            &xml[..xml.len().min(400)]
        );
        for expected_id in 0..4u32 {
            assert!(
                xml.contains(&format!(r#"<hp:p id="{}""#, expected_id)),
                "id={} 가 없음",
                expected_id
            );
        }
    }

    #[test]
    fn cell_para_ids_continue_from_context_counter() {
        // 컨텍스트 카운터를 5 앞당긴 뒤 직렬화하면 셀 문단 id가 5부터 시작해야 함.
        // 본문 문단이 먼저 id 0~4를 소비한 상황을 모사한다.
        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        for _ in 0..5 {
            ctx.next_para_id();
        }
        let t = empty_table(1, 2); // 셀 2개 × 문단 1개 → id=5, id=6 이어야 함
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_table(&mut w, &t, &mut ctx).unwrap();
        let xml = String::from_utf8(w.into_inner()).unwrap();
        assert!(
            xml.contains(r#"<hp:p id="5""#),
            "첫 셀 문단은 id=5 여야 함 (카운터 오프셋 5): {}",
            &xml[..xml.len().min(600)]
        );
        assert!(
            xml.contains(r#"<hp:p id="6""#),
            "두 번째 셀 문단은 id=6 여야 함"
        );
    }

    #[test]
    fn two_sequential_tables_have_no_para_id_collision() {
        // 같은 ctx로 표 두 개를 연달아 직렬화 — 두 번째 표가 카운터를 초기화하면
        // id=0 이 2번 나타나므로 회귀를 탐지할 수 있다.
        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_table(&mut w, &empty_table(2, 2), &mut ctx).unwrap(); // id 0-3
        write_table(&mut w, &empty_table(2, 2), &mut ctx).unwrap(); // id 4-7
        let xml = String::from_utf8(w.into_inner()).unwrap();

        assert_eq!(
            xml.matches(r#"<hp:p id="0""#).count(),
            1,
            "id=0 이 중복 — 두 번째 표가 카운터를 재사용했을 가능성"
        );
        for expected_id in 0..8u32 {
            assert!(
                xml.contains(&format!(r#"<hp:p id="{}""#, expected_id)),
                "id={} 가 없음 (총 8개 문단이어야 함)",
                expected_id
            );
        }
    }

    #[test]
    fn multi_para_cells_all_get_unique_ids() {
        // 셀당 문단 3개, 2×2 표 → 총 12개 문단, id=0..11 전부 1회씩
        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut t = empty_table(2, 2);
        for cell in &mut t.cells {
            cell.paragraphs.push(Paragraph::default());
            cell.paragraphs.push(Paragraph::default());
        }
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_table(&mut w, &t, &mut ctx).unwrap();
        let xml = String::from_utf8(w.into_inner()).unwrap();

        let p_count = xml.matches("<hp:p ").count();
        assert_eq!(p_count, 12, "문단 수가 12여야 함: {}", p_count);

        for expected_id in 0..12u32 {
            assert_eq!(
                xml.matches(&format!(r#"<hp:p id="{}""#, expected_id))
                    .count(),
                1,
                "id={} 가 없거나 중복됨",
                expected_id
            );
        }
    }

    #[test]
    fn cell_text_serializes_tab_and_line_break_as_hwpx_inline_elements() {
        let mut t = empty_table(1, 1);
        let para = &mut t.cells[0].paragraphs[0];
        para.text = "A\tB\nC".to_string();
        para.tab_extended = vec![[2000, 0, 0x0100, 0, 0, 0, 0]];

        let xml = serialize(&t);

        assert!(
            xml.contains(
                r#"<hp:t>A<hp:tab width="2000" leader="0" type="1"/>B<hp:lineBreak/>C</hp:t>"#
            ),
            "cell text must emit hp:tab/hp:lineBreak instead of raw control chars: {}",
            xml
        );
    }

    #[test]
    fn task1378_cell_paragraph_multi_run_split() {
        // 셀 문단 다중 char_shapes → 경계 기준 다중 run 분할 (#1378 3단계).
        let mut t = empty_table(1, 1);
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.text = "abcd".to_string();
            para.char_offsets = vec![0, 1, 2, 3];
            para.char_count = 5;
            para.char_shapes = vec![cs(0, 1), cs(2, 2)];
        }
        let xml = serialize(&t);
        assert!(
            xml.contains(
                r#"<hp:run charPrIDRef="1"><hp:t>ab</hp:t></hp:run><hp:run charPrIDRef="2"><hp:t>cd</hp:t></hp:run>"#
            ),
            "셀 문단이 경계에서 2 run 으로 분할되어야 함: {}",
            xml
        );
    }

    #[test]
    fn task1378_cell_boundary_with_control_gap_offsets() {
        // IR 내 컨트롤(8 유닛 갭)이 있어도 char_offsets 매핑으로 경계 위치가
        // 어긋나지 않는다 (컨트롤 자체의 출력은 #1379 범위).
        let mut t = empty_table(1, 1);
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.text = "abcd".to_string();
            para.char_offsets = vec![0, 1, 10, 11];
            para.char_count = 13;
            para.char_shapes = vec![cs(0, 1), cs(10, 2)];
        }
        let xml = serialize(&t);
        assert!(
            xml.contains(
                r#"<hp:run charPrIDRef="1"><hp:t>ab</hp:t></hp:run><hp:run charPrIDRef="2"><hp:t>cd</hp:t></hp:run>"#
            ),
            "경계(pos=10)가 컨트롤 갭 뒤 'c' 앞에 떨어져야 함: {}",
            xml
        );
    }

    /// bin_data_id=1 을 참조하는 Picture 컨트롤 — `serialize_with_bin` 과 함께 사용.
    fn picture_control() -> crate::model::control::Control {
        let mut pic = crate::model::image::Picture::default();
        pic.image_attr.bin_data_id = 1;
        crate::model::control::Control::Picture(Box::new(pic))
    }

    /// BinDataContent(id=1) 등록 문서 기준으로 직렬화 — hp:pic 방출 테스트용.
    fn serialize_with_bin(table: &Table) -> String {
        let mut doc = Document::default();
        doc.bin_data_content
            .push(crate::model::bin_data::BinDataContent {
                id: 1,
                data: vec![0u8; 4],
                extension: "png".to_string(),
            });
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_table(&mut w, table, &mut ctx).expect("write_table");
        String::from_utf8(w.into_inner()).unwrap()
    }

    #[test]
    fn task1379_cell_paragraph_emits_picture_control() {
        // 셀 문단의 Picture 컨트롤이 hp:pic 으로 방출되어야 함 (#1379 2단계).
        let mut t = empty_table(1, 1);
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.char_count = 9; // 슬롯 1개(8 유닛) + 종단 1
            para.controls.push(picture_control());
        }
        let xml = serialize_with_bin(&t);
        assert!(
            xml.contains("<hp:pic "),
            "셀 문단의 Picture 가 hp:pic 으로 방출되어야 함: {}",
            xml
        );
    }

    #[test]
    fn task1379_nested_table_in_cell_recurses_with_unique_para_ids() {
        // 셀 안 표 재귀 — 중첩 hp:tbl 방출 + next_para_id 채번 무충돌.
        let mut outer = empty_table(1, 1);
        {
            let para = &mut outer.cells[0].paragraphs[0];
            para.char_count = 9;
            para.controls
                .push(crate::model::control::Control::Table(Box::new(
                    empty_table(1, 1),
                )));
        }
        let xml = serialize(&outer);
        assert_eq!(
            xml.matches("<hp:tbl ").count(),
            2,
            "중첩 hp:tbl 이 방출되어야 함: {}",
            xml
        );
        assert_eq!(xml.matches("<hp:p ").count(), 2, "문단 2개여야 함");
        for id in 0..2u32 {
            assert_eq!(
                xml.matches(&format!(r#"<hp:p id="{}""#, id)).count(),
                1,
                "id={} 가 없거나 중복 — 재귀 채번 충돌",
                id
            );
        }
    }

    #[test]
    fn task1379_cell_control_slot_position_between_text() {
        // char_offsets 8 유닛 갭 위치에서 컨트롤이 정확히 방출되어야 함.
        let mut t = empty_table(1, 1);
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.text = "ab".to_string();
            para.char_offsets = vec![0, 9]; // a=0, 슬롯=1..9, b=9
            para.char_count = 11;
            para.controls.push(picture_control());
        }
        let xml = serialize_with_bin(&t);
        let a = xml.find("<hp:t>a</hp:t>").expect("a 텍스트");
        let p = xml.find("<hp:pic ").expect("hp:pic");
        let b = xml.find("<hp:t>b</hp:t>").expect("b 텍스트");
        assert!(a < p && p < b, "슬롯이 a 와 b 사이에 와야 함: {}", xml);
    }

    #[test]
    fn page_break_str_is_exact_inverse_of_parser() {
        // 직렬화 문자열은 파서(parser/hwpx/section.rs pageBreak)의 정확한 역이어야
        // HWPX→HWPX 왕복이 충실하다. 파서: "TABLE"→CellBreak, "CELL"/"ROW"→RowBreak.
        // 따라서 직렬화: CellBreak→"TABLE", RowBreak→"CELL". (뒤바뀌면 한컴 HWPX 의
        // CELL↔TABLE 의미가 왕복 시 스왑된다.)
        assert_eq!(table_page_break_str(TablePageBreak::None), "NONE");
        assert_eq!(table_page_break_str(TablePageBreak::CellBreak), "TABLE");
        assert_eq!(table_page_break_str(TablePageBreak::RowBreak), "CELL");
    }

    #[test]
    fn task1379_cell_char_shape_boundary_after_control_restored() {
        // #1378 게이트의 경계 8×컨트롤수 시프트 해소 — 경계 (8,77) 이 컨트롤 뒤에 복원.
        let mut t = empty_table(1, 1);
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.text = "ab".to_string();
            para.char_offsets = vec![8, 9]; // 슬롯=0..8, a=8, b=9
            para.char_count = 11;
            para.char_shapes = vec![cs(0, 1), cs(8, 77)];
            para.controls.push(picture_control());
        }
        let xml = serialize_with_bin(&t);
        assert!(
            xml.contains(r#"<hp:run charPrIDRef="1"><hp:pic "#),
            "run1 은 컨트롤만 포함해야 함: {}",
            xml
        );
        assert!(
            xml.contains(r#"<hp:run charPrIDRef="77"><hp:t>ab</hp:t></hp:run>"#),
            "경계(pos=8)가 컨트롤 뒤 텍스트 앞에 복원되어야 함: {}",
            xml
        );
    }

    #[test]
    fn task1379_cell_lineseg_preserved_from_ir() {
        // 합성 lineseg 제거 — IR line_segs 가 있으면 값 그대로 보존 (#177 정렬).
        let mut t = empty_table(1, 1);
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.line_segs.push(crate::model::paragraph::LineSeg {
                text_start: 0,
                vertical_pos: 1234,
                line_height: 900,
                text_height: 900,
                baseline_distance: 765,
                line_spacing: 540,
                column_start: 0,
                segment_width: 7777,
                tag: 0x60000,
            });
        }
        let xml = serialize(&t);
        assert!(
            xml.contains(
                r#"<hp:lineseg textpos="0" vertpos="1234" vertsize="900" textheight="900" baseline="765" spacing="540" horzpos="0" horzsize="7777" flags="393216"/>"#
            ),
            "IR lineseg 값이 그대로 방출되어야 함: {}",
            xml
        );
    }

    #[test]
    fn task1379_cell_char_overlap_emitted_as_compose() {
        // 셀 내 글자겹침 — render_control_slot CharOverlap arm (mel-001 양상).
        let mut t = empty_table(1, 1);
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.char_count = 9;
            para.controls
                .push(crate::model::control::Control::CharOverlap(
                    crate::model::control::CharOverlap {
                        chars: vec!['장'],
                        border_type: 0,
                        inner_char_size: -3,
                        expansion: 1,
                        char_shape_ids: vec![37, u32::MAX],
                    },
                ));
        }
        let xml = serialize(&t);
        assert!(
            xml.contains(
                r#"<hp:compose circleType="CHAR" charSz="-3" composeType="OVERLAP" charPrCnt="2" composeText="장">"#
            ),
            "compose 속성이 원본 형태로 방출되어야 함: {}",
            xml
        );
        assert!(
            xml.contains(
                r#"<hp:charPr prIDRef="37"/><hp:charPr prIDRef="4294967295"/></hp:compose>"#
            ),
            "charPr 목록이 미설정(u32::MAX) 포함 그대로 방출되어야 함: {}",
            xml
        );
    }

    #[test]
    fn task1387_ta_pic_001_r_roundtrip_preserves_caption() {
        // 이슈 #1387 대표 샘플 — roundtrip 후 표 캡션(문단 텍스트 포함) 보존.
        fn first_table_caption(doc: &Document) -> Option<crate::model::shape::Caption> {
            doc.sections
                .iter()
                .flat_map(|s| &s.paragraphs)
                .flat_map(|p| &p.controls)
                .find_map(|c| match c {
                    crate::model::control::Control::Table(t) => t.caption.clone(),
                    _ => None,
                })
        }
        let bytes = std::fs::read("samples/hwpx/ta-pic-001-r.hwpx").expect("샘플 읽기");
        let doc1 = crate::parser::hwpx::parse_hwpx(&bytes).expect("파싱");
        let cap1 = first_table_caption(&doc1).expect("픽스처 전제: 원본 표에 캡션 존재");
        assert!(
            cap1.paragraphs[0].text.contains("의정활동"),
            "원본 캡션 텍스트 확인: {:?}",
            cap1.paragraphs[0].text
        );
        let out = crate::serializer::hwpx::serialize_hwpx(&doc1).expect("직렬화");
        let doc2 = crate::parser::hwpx::parse_hwpx(&out).expect("재파싱");
        let cap2 = first_table_caption(&doc2).expect("roundtrip 후 캡션이 보존되어야 함");
        assert_eq!(cap1.paragraphs.len(), cap2.paragraphs.len());
        assert_eq!(cap1.width, cap2.width);
        assert_eq!(cap1.spacing, cap2.spacing);
        assert_eq!(cap1.max_width, cap2.max_width);
        assert_eq!(cap1.direction, cap2.direction);
        assert_eq!(cap1.include_margin, cap2.include_margin);
        // #1382 해소(autoNum 폭 축 일관화) 후 텍스트 완전 일치로 승격 —
        // 종전에는 placeholder 변위로 trim_end 비교만 가능했다.
        assert_eq!(
            cap2.paragraphs[0].text, cap1.paragraphs[0].text,
            "캡션 텍스트 완전 일치 (#1382 해소 승격)"
        );
        assert_eq!(
            cap2.paragraphs[0].char_offsets, cap1.paragraphs[0].char_offsets,
            "캡션 char_offsets 보존 — autoNum 8-jump 포함 (#1382)"
        );
        assert_eq!(
            cap1.paragraphs[0].controls.len(),
            cap2.paragraphs[0].controls.len(),
            "캡션 내 컨트롤(autoNum) 수 보존"
        );
    }

    #[test]
    fn task1382_ta_pic_caption_autonum_slot_at_midtext() {
        // 캡션 autoNum 슬롯 위치 회귀 테스트 — RT XML 에서 ctrl 이 한컴 원본 동형의
        // mid-text 위치(`<hp:t>…그림 </hp:t><hp:ctrl>` 패턴)에 방출되는지 고정.
        // (#1387 1차 한컴 판정의 "번호 문장 끝 밀림" 재발 방지)
        let bytes = std::fs::read("samples/hwpx/ta-pic-001-r.hwpx").expect("샘플 읽기");
        let doc = crate::parser::hwpx::parse_hwpx(&bytes).expect("파싱");
        let out = crate::serializer::hwpx::serialize_hwpx(&doc).expect("직렬화");
        // section0.xml 추출 없이 zip 바이트에서 직접 패턴 탐색은 불가 — 재파싱 IR 로
        // 위치를 검증하고, XML 패턴은 단위 테스트(task1382_autonum_slot_emitted_at_placeholder)가 고정.
        let doc2 = crate::parser::hwpx::parse_hwpx(&out).expect("재파싱");
        let cap = doc2
            .sections
            .iter()
            .flat_map(|s| &s.paragraphs)
            .flat_map(|p| &p.controls)
            .find_map(|c| match c {
                crate::model::control::Control::Table(t) => t.caption.clone(),
                _ => None,
            })
            .expect("RT 캡션");
        let p = &cap.paragraphs[0];
        // placeholder(공백)가 mid-text 위치(인덱스 4)에 있고 직후 offset 이 +8 jump —
        // ctrl 이 끝으로 밀렸다면 jump 가 끝에 가 있거나 소실된다.
        assert_eq!(p.char_offsets.get(4), Some(&4), "placeholder 위치 4");
        assert_eq!(p.char_offsets.get(5), Some(&12), "autoNum 8-jump 직후 12");
    }

    #[test]
    fn task1387_mel_001_roundtrip_caption_text_exact() {
        // autoNum 없는 캡션(mel-001, side=TOP) — 텍스트 완전 일치 + side 역매핑 검증.
        fn first_table_caption(doc: &Document) -> Option<crate::model::shape::Caption> {
            doc.sections
                .iter()
                .flat_map(|s| &s.paragraphs)
                .flat_map(|p| &p.controls)
                .find_map(|c| match c {
                    crate::model::control::Control::Table(t) => t.caption.clone(),
                    _ => None,
                })
        }
        let bytes = std::fs::read("samples/hwpx/mel-001.hwpx").expect("샘플 읽기");
        let doc1 = crate::parser::hwpx::parse_hwpx(&bytes).expect("파싱");
        let cap1 = first_table_caption(&doc1).expect("픽스처 전제: 원본 표에 캡션 존재");
        assert_eq!(cap1.direction, crate::model::shape::CaptionDirection::Top);
        let out = crate::serializer::hwpx::serialize_hwpx(&doc1).expect("직렬화");
        let doc2 = crate::parser::hwpx::parse_hwpx(&out).expect("재파싱");
        let cap2 = first_table_caption(&doc2).expect("roundtrip 후 캡션 보존");
        assert_eq!(cap1.paragraphs[0].text, cap2.paragraphs[0].text);
        assert_eq!(cap1.direction, cap2.direction);
    }

    #[test]
    fn task1379_ta_pic_001_r_roundtrip_preserves_cell_pictures() {
        // 이슈 #1379 대표 샘플 — roundtrip 후 셀 내 Picture 전수(실측 2개) 보존.
        fn count_cell_pictures(doc: &Document) -> usize {
            doc.sections
                .iter()
                .flat_map(|s| &s.paragraphs)
                .flat_map(|p| &p.controls)
                .filter_map(|c| match c {
                    crate::model::control::Control::Table(t) => Some(t),
                    _ => None,
                })
                .flat_map(|t| &t.cells)
                .flat_map(|c| &c.paragraphs)
                .flat_map(|p| &p.controls)
                .filter(|c| matches!(c, crate::model::control::Control::Picture(_)))
                .count()
        }
        let bytes = std::fs::read("samples/hwpx/ta-pic-001-r.hwpx").expect("샘플 읽기");
        let doc1 = crate::parser::hwpx::parse_hwpx(&bytes).expect("파싱");
        let n1 = count_cell_pictures(&doc1);
        // 원본 section0.xml 실측 hp:pic 2개 (전부 셀 내부) — 이슈 본문의 "4개" 는
        // 부정확 수치로 확인됨 (stage2 보고서 참조).
        assert_eq!(n1, 2, "원본 셀 내 Picture 는 2개여야 함");
        let out = crate::serializer::hwpx::serialize_hwpx(&doc1).expect("직렬화");
        let doc2 = crate::parser::hwpx::parse_hwpx(&out).expect("재파싱");
        assert_eq!(
            count_cell_pictures(&doc2),
            n1,
            "roundtrip 후 셀 내 Picture 수가 보존되어야 함"
        );
    }

    #[test]
    fn task1379_cell_column_def_emits_col_pr() {
        // 셀 문단의 ColumnDef 가 hp:ctrl/hp:colPr 인라인으로 방출되어야 함 (#1379 3단계).
        let mut t = empty_table(1, 1);
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.char_count = 9; // 슬롯 1개(8 유닛) + 종단 1
            let mut cd = crate::model::page::ColumnDef::default();
            cd.column_count = 1;
            cd.same_width = true;
            para.controls
                .push(crate::model::control::Control::ColumnDef(cd));
        }
        let xml = serialize(&t);
        assert!(
            xml.contains(
                r#"<hp:ctrl><hp:colPr id="" type="NEWSPAPER" layout="LEFT" colCount="1" sameSz="1" sameGap="0"/></hp:ctrl>"#
            ),
            "셀 문단의 ColumnDef 가 hp:colPr 로 방출되어야 함: {}",
            xml
        );
    }

    #[test]
    fn task1379_cell_column_def_col_line_emitted_when_separator() {
        // separator_type≠0 인 경우 hp:colLine 자식 방출.
        let mut t = empty_table(1, 1);
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.char_count = 9;
            let mut cd = crate::model::page::ColumnDef::default();
            cd.column_count = 2;
            cd.same_width = true;
            cd.spacing = 1134;
            cd.separator_type = 2; // DASH
            cd.separator_width = 1; // 0.12 mm
            para.controls
                .push(crate::model::control::Control::ColumnDef(cd));
        }
        let xml = serialize(&t);
        assert!(
            xml.contains(r##"<hp:colLine type="DASH" width="0.12 mm" color="#000000"/>"##),
            "separator 있는 ColumnDef 는 hp:colLine 을 방출해야 함: {}",
            xml
        );
    }

    #[test]
    fn task1378_cell_tab_in_split_runs() {
        // 탭 포함 셀 문단 run 분할 — 탭은 첫 run 에, 분할 텍스트는 새 run 에.
        let mut t = empty_table(1, 1);
        {
            let para = &mut t.cells[0].paragraphs[0];
            para.text = "a\tb".to_string();
            para.char_offsets = vec![0, 1, 2];
            para.char_count = 4;
            para.char_shapes = vec![cs(0, 1), cs(2, 2)];
            para.tab_extended = vec![[2000, 0, 0x0100, 0, 0, 0, 0]];
        }
        let xml = serialize(&t);
        assert!(
            xml.contains(
                r#"<hp:run charPrIDRef="1"><hp:t>a<hp:tab width="2000" leader="0" type="1"/></hp:t></hp:run><hp:run charPrIDRef="2"><hp:t>b</hp:t></hp:run>"#
            ),
            "탭 포함 분할: run1=a+tab, run2=b 여야 함: {}",
            xml
        );
    }
}
