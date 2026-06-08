//! Contents/section{N}.xml — Section 본문 직렬화
//!
//! Stage 2 (#182): 기존 템플릿 기반 구조를 유지하되, `<hp:p>` 와 `<hp:run>` 의 속성을
//! IR에서 가져와 동적으로 생성한다. `secPr`/`pagePr`/`grid` 등 섹션 정의는 템플릿 보존
//! (IR에 대응 필드가 더 담길 때까지 점진적으로 동적화 예정).
//!
//! Stage #177 (2026-04-18): `<hp:lineseg>` 직렬화를 IR 기반으로 전환.
//! `Paragraph.line_segs` 의 9개 필드(textpos, vertpos, vertsize, textheight, baseline,
//! spacing, horzpos, horzsize, flags)를 그대로 출력하여 **원본 lineseg 값 보존**.
//! rhwp 는 자신의 문서에서 새로 부정확한 값을 생산하지 않는다.
//!
//! IR 매핑 관행:
//!   - `section.paragraphs` 여러 개 = 하드 문단 경계 (`<hp:p>` 여러 개)
//!   - `paragraph.text` 내 `\n` = 소프트 라인브레이크 (`<hp:lineBreak/>`, 같은 문단 내)
//!   - `paragraph.text` 내 `\t` = 탭 (`<hp:tab width=... leader="0" type="1"/>`)
//!   - `paragraph.para_shape_id` → `<hp:p paraPrIDRef>`
//!   - `paragraph.style_id` → `<hp:p styleIDRef>`
//!   - `paragraph.column_type` → `<hp:p pageBreak/columnBreak>`
//!   - `paragraph.char_shapes[0].char_shape_id` → 첫 `<hp:run charPrIDRef>`
//!   - `paragraph.line_segs[i]` → 각 `<hp:lineseg>` 속성 (9개 필드 그대로 출력)

use quick_xml::Writer;

use crate::model::control::{
    AutoNumber, AutoNumberType, Control, Equation, NewNumber, PageHide, PageNumberPos,
};
use crate::model::document::{Document, Section};
use crate::model::footnote::{Endnote, Footnote};
use crate::model::header_footer::{Footer, Header, HeaderFooterApply};
use crate::model::paragraph::{ColumnBreakType, LineSeg, Paragraph};
use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, ShapeObject, TextWrap, VertAlign, VertRelTo,
};

use super::context::SerializeContext;
use super::field::{write_bookmark, write_field_begin, write_field_end};
use super::utils::xml_escape;
use super::SerializeError;
use super::{picture, table};

const EMPTY_SECTION_XML: &str = include_str!("templates/empty_section0.xml");
const TEXT_SLOT: &str = "<hp:t/>";
const LINESEG_SLOT_OPEN: &str = "<hp:linesegarray>";
const LINESEG_SLOT_CLOSE: &str = "</hp:linesegarray>";
const PARA_CLOSE: &str = "</hp:p></hs:sec>";

// 템플릿 내 첫 <hp:p> 태그의 실제 문자열 (id="3121190098" 랜덤 해시 포함).
// 템플릿은 정적이므로 이 문자열이 고정 위치에 있음이 보장됨.
const TEMPLATE_FIRST_P_TAG: &str = r#"<hp:p id="3121190098" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">"#;
// 템플릿 내 <hp:run charPrIDRef="0"> 직후에 TEXT_SLOT 이 오는 패턴.
const TEMPLATE_RUN_BEFORE_TEXT: &str = r#"<hp:run charPrIDRef="0"><hp:t/>"#;

/// 레퍼런스 기준 줄 레이아웃 파라미터.
const VERT_STEP: u32 = 1600; // vertsize(1000) + spacing(600)
const LINE_FLAGS: u32 = LineSeg::TAG_SINGLE_SEGMENT_LINE;
const HORZ_SIZE: u32 = 42520;
/// 탭 기본 폭 (한컴이 열면서 재계산하지만 초기값으로 필요).
const TAB_DEFAULT_WIDTH: u32 = 4000;

/// Stage 2 진입점. `ctx` 는 Stage 3+ 에서 파라미터 검증에 사용.
pub fn write_section(
    section: &Section,
    _doc: &Document,
    _index: usize,
    ctx: &mut SerializeContext,
) -> Result<Vec<u8>, SerializeError> {
    let mut vert_cursor: u32 = 0;

    let first_para = section.paragraphs.first();
    let (first_t, first_linesegs, first_advance) = match first_para {
        Some(p) => render_paragraph_parts(p, vert_cursor, ctx),
        None => render_paragraph_parts_for_text("", vert_cursor),
    };
    vert_cursor = first_advance;

    let mut out = EMPTY_SECTION_XML.replacen(TEXT_SLOT, &first_t, 1);
    out = replace_first_linesegs(&out, &first_linesegs);
    out = replace_page_pr(&out, &section.section_def.page_def);

    // 첫 문단 `<hp:p>` 태그를 IR 기반 속성으로 교체
    if let Some(p) = first_para {
        let new_p_tag = render_hp_p_open(p, ctx.next_para_id());
        out = out.replacen(TEMPLATE_FIRST_P_TAG, &new_p_tag, 1);

        // 첫 문단의 텍스트용 <hp:run> 의 charPrIDRef 를 IR 기반으로 교체
        // 템플릿에서 TEXT_SLOT 이 있던 자리 바로 앞의 <hp:run charPrIDRef="0"> 패턴.
        let first_run_cs = first_run_char_shape_id(p);
        let new_run = format!(r#"<hp:run charPrIDRef="{}">"#, first_run_cs);
        let replacement = format!("{}{}", new_run, &first_t);
        // 이미 first_t 는 out 에 들어갔으므로 그 직전의 <hp:run charPrIDRef="0"> 만 변경
        let anchor = format!("{}{}", r#"<hp:run charPrIDRef="0">"#, &first_t);
        if out.contains(&anchor) {
            out = out.replacen(&anchor, &replacement, 1);
        }
    }

    // 추가 문단: `</hp:p></hs:sec>` 직전에 `<hp:p>` 요소를 삽입.
    if section.paragraphs.len() > 1 {
        let mut extra = String::new();
        for p in section.paragraphs.iter().skip(1) {
            let (t, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
            vert_cursor = advance;
            let cs = first_run_char_shape_id(p);
            extra.push_str(&render_hp_p_open(p, ctx.next_para_id()));
            extra.push_str(&format!(r#"<hp:run charPrIDRef="{}">"#, cs));
            extra.push_str(&t);
            extra.push_str(r#"</hp:run><hp:linesegarray>"#);
            extra.push_str(&linesegs);
            extra.push_str(r#"</hp:linesegarray></hp:p>"#);
        }
        out = out.replacen(PARA_CLOSE, &format!("</hp:p>{}</hs:sec>", extra), 1);
    }

    Ok(out.into_bytes())
}

/// IR의 Paragraph를 기반으로 `<hp:p>` 시작 태그를 생성.
///
/// `id` 는 문단 순서 기반(0, 1, 2, ...)로 할당한다. 한컴 샘플은 랜덤 해시도 쓰지만
/// 파서는 id 를 무시하므로 순차값으로 충분.
fn render_hp_p_open(p: &Paragraph, id: u32) -> String {
    let page_break = if matches!(p.column_type, ColumnBreakType::Page) {
        1
    } else {
        0
    };
    let column_break = if matches!(p.column_type, ColumnBreakType::Column) {
        1
    } else {
        0
    };
    format!(
        r#"<hp:p id="{}" paraPrIDRef="{}" styleIDRef="{}" pageBreak="{}" columnBreak="{}" merged="0">"#,
        id, p.para_shape_id, p.style_id, page_break, column_break,
    )
}

/// 문단 첫 run 의 charPrIDRef. IR의 `char_shapes[0].char_shape_id` 사용.
/// 비어있으면 0 (기본 글자모양) 반환.
fn first_run_char_shape_id(p: &Paragraph) -> u32 {
    p.char_shapes.first().map(|r| r.char_shape_id).unwrap_or(0)
}

/// Paragraph 하나를 (`<hp:t>` XML, lineseg XML, 다음 vert_cursor)로 변환.
///
/// `<hp:lineseg>` 출력 원칙 (#177):
/// - `para.line_segs` 가 비어있지 않으면 **IR 값 그대로 출력**
/// - 비어있을 때만 텍스트 내 `\n` 기반으로 fallback 생성 (빈 문단·`Document::default()` 호환)
fn render_paragraph_parts(
    para: &Paragraph,
    vert_start: u32,
    ctx: &mut SerializeContext,
) -> (String, String, u32) {
    let t_xml = render_run_content(para, ctx);

    if !para.line_segs.is_empty() {
        // IR 기반 출력 — 원본 lineseg 값 보존 (#177)
        let linesegs = render_lineseg_array_from_ir(&para.line_segs);
        let vert_end = next_vert_cursor_from_ir(&para.line_segs, vert_start);
        (t_xml, linesegs, vert_end)
    } else {
        // Fallback — IR에 line_segs 가 없으면 기존 생성 로직 유지
        let (linesegs, vert_end) = render_lineseg_array_fallback(&para.text, vert_start);
        (t_xml, linesegs, vert_end)
    }
}

/// IR 없이 텍스트만 있을 때 `<hp:t>` 와 fallback lineseg 생성.
/// `write_section` 이 `first_para == None` 인 경우를 위해 유지.
fn render_paragraph_parts_for_text(text: &str, vert_start: u32) -> (String, String, u32) {
    let t_xml = render_hp_t_content(text, &[], &mut 0);
    let (linesegs, vert_end) = render_lineseg_array_fallback(text, vert_start);
    (t_xml, linesegs, vert_end)
}

/// `<hp:t>...</hp:t>` 본문 생성 — 탭/소프트브레이크/XML escape 포함.
///
/// `tab_extended`: IR의 탭 확장 정보 목록. `tab_idx`를 통해 탭 문자마다 순서대로 참조.
/// 항목이 없으면 폴백(width=TAB_DEFAULT_WIDTH, leader=0, type=1)을 사용.
fn render_hp_t_content(text: &str, tab_extended: &[[u16; 7]], tab_idx: &mut usize) -> String {
    let mut t_xml = String::from("<hp:t>");
    let mut buf = String::new();
    for c in text.chars() {
        match c {
            '\t' => {
                flush_buf(&mut t_xml, &mut buf);
                let (width, leader, tab_type) = if let Some(ext) = tab_extended.get(*tab_idx) {
                    *tab_idx += 1;
                    (ext[0] as u32, ext[2] & 0x00ff, (ext[2] >> 8) & 0x00ff)
                } else {
                    (TAB_DEFAULT_WIDTH, 0u16, 1u16)
                };
                t_xml.push_str(&format!(
                    r#"<hp:tab width="{}" leader="{}" type="{}"/>"#,
                    width, leader, tab_type
                ));
            }
            '\n' => {
                flush_buf(&mut t_xml, &mut buf);
                t_xml.push_str("<hp:lineBreak/>");
            }
            c if (c as u32) < 0x20 => { /* 기타 제어문자 무시 */ }
            c => buf.push(c),
        }
    }
    flush_buf(&mut t_xml, &mut buf);
    t_xml.push_str("</hp:t>");
    t_xml
}

/// Paragraph의 본문 run 콘텐츠를 `<hp:t>`와 인라인 컨트롤 XML로 직렬화한다.
fn render_run_content(para: &Paragraph, ctx: &mut SerializeContext) -> String {
    // Bookmark는 IR에 위치 정보가 없어 문단 시작에 배치한다.
    // (HWPX 파서가 char_count에 포함하지 않아 slot 시스템이 위치를 추적할 수 없음)
    let mut prefix = String::new();
    for ctrl in &para.controls {
        if let Control::Bookmark(bm) = ctrl {
            if let Ok(xml) = writer_to_string(|w| write_bookmark(w, bm)) {
                prefix.push_str("<hp:ctrl>");
                prefix.push_str(&xml);
                prefix.push_str("</hp:ctrl>");
            }
        }
    }

    let slot_count = inferred_control_slot_count(para);
    let slots: Vec<&Control> = if slot_count == para.controls.len() {
        para.controls.iter().collect()
    } else {
        para.controls
            .iter()
            .filter(|c| is_hwpx_inline_slot(c))
            .collect()
    };

    let mut tab_idx = 0usize;

    if slots.is_empty() && para.field_ranges.is_empty() {
        let mut out = prefix;
        out.push_str(&render_hp_t_content(
            &para.text,
            &para.tab_extended,
            &mut tab_idx,
        ));
        return out;
    }

    if slot_count != slots.len() {
        let mut out = prefix;
        out.push_str(&render_hp_t_content(
            &para.text,
            &para.tab_extended,
            &mut tab_idx,
        ));
        for slot in &slots {
            render_control_slot(&mut out, slot, ctx);
        }
        return out;
    }

    let mut out = prefix;
    let mut text_buf = String::new();
    let mut slot_idx = 0usize;
    let mut expected_utf16_pos = 0u32;
    let mut field_end_emitted = vec![false; para.field_ranges.len()];

    // 빈 문단(text == "")의 0-length 필드: 메인 루프가 실행되지 않아
    // pre-char 검사를 통과하지 못하므로 루프 전에 slots → fieldEnd 순으로 방출한다.
    if para.text.is_empty() {
        while slot_idx < slots.len() {
            render_control_slot(&mut out, slots[slot_idx], ctx);
            slot_idx += 1;
        }
        for (i, fr) in para.field_ranges.iter().enumerate() {
            if fr.start_char_idx == fr.end_char_idx && !field_end_emitted[i] {
                if let Some(Control::Field(f)) = para.controls.get(fr.control_idx) {
                    if let Ok(xml) = writer_to_string(|w| write_field_end(w, f.field_id)) {
                        out.push_str("<hp:ctrl>");
                        out.push_str(&xml);
                        out.push_str("</hp:ctrl>");
                    }
                }
                field_end_emitted[i] = true;
            }
        }
    }

    for (idx, c) in para.text.chars().enumerate() {
        let char_pos = para
            .char_offsets
            .get(idx)
            .copied()
            .unwrap_or(expected_utf16_pos);
        while slot_idx < slots.len() && char_pos >= expected_utf16_pos.saturating_add(8) {
            flush_text_fragment(&mut out, &mut text_buf, &para.tab_extended, &mut tab_idx);
            render_control_slot(&mut out, slots[slot_idx], ctx);
            slot_idx += 1;
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
        }

        // 0-length 필드(start == end == idx): fieldBegin 방출 직후, 문자 push 전에 fieldEnd 방출.
        // post-char 검사(next_idx 기준)는 end-1 번째 문자 처리 후 방출하므로 0-length 필드에서
        // fieldEnd가 fieldBegin 앞에 나오거나 텍스트 뒤로 밀리는 문제가 생긴다.
        for (i, fr) in para.field_ranges.iter().enumerate() {
            if fr.start_char_idx == fr.end_char_idx
                && fr.end_char_idx == idx
                && !field_end_emitted[i]
            {
                flush_text_fragment(&mut out, &mut text_buf, &para.tab_extended, &mut tab_idx);
                if let Some(Control::Field(f)) = para.controls.get(fr.control_idx) {
                    if let Ok(xml) = writer_to_string(|w| write_field_end(w, f.field_id)) {
                        out.push_str("<hp:ctrl>");
                        out.push_str(&xml);
                        out.push_str("</hp:ctrl>");
                    }
                }
                field_end_emitted[i] = true;
            }
        }

        text_buf.push(c);
        let width = char_utf16_width(c);
        if char_pos >= expected_utf16_pos {
            expected_utf16_pos = char_pos.saturating_add(width);
        } else {
            expected_utf16_pos = expected_utf16_pos.saturating_add(width);
        }

        // end_char_idx는 미포함(exclusive): 현재 문자가 필드 범위의 마지막이면 fieldEnd 삽입.
        // 0-length 필드(start == end)는 위의 pre-char 검사에서 처리하므로 제외한다.
        let next_idx = idx + 1;
        for (i, fr) in para.field_ranges.iter().enumerate() {
            if fr.end_char_idx == next_idx
                && !field_end_emitted[i]
                && fr.start_char_idx < fr.end_char_idx
            {
                flush_text_fragment(&mut out, &mut text_buf, &para.tab_extended, &mut tab_idx);
                if let Some(Control::Field(f)) = para.controls.get(fr.control_idx) {
                    if let Ok(xml) = writer_to_string(|w| write_field_end(w, f.field_id)) {
                        out.push_str("<hp:ctrl>");
                        out.push_str(&xml);
                        out.push_str("</hp:ctrl>");
                    }
                }
                field_end_emitted[i] = true;
            }
        }
    }

    flush_text_fragment(&mut out, &mut text_buf, &para.tab_extended, &mut tab_idx);

    // end_char_idx >= text.len() 인 경우 루프에서 감지되지 않으므로 루프 후에 처리
    for (i, fr) in para.field_ranges.iter().enumerate() {
        if !field_end_emitted[i] {
            if let Some(Control::Field(f)) = para.controls.get(fr.control_idx) {
                if let Ok(xml) = writer_to_string(|w| write_field_end(w, f.field_id)) {
                    out.push_str("<hp:ctrl>");
                    out.push_str(&xml);
                    out.push_str("</hp:ctrl>");
                }
            }
        }
    }

    while slot_idx < slots.len() {
        render_control_slot(&mut out, slots[slot_idx], ctx);
        slot_idx += 1;
    }

    if out.is_empty() {
        render_hp_t_content("", &para.tab_extended, &mut tab_idx)
    } else {
        out
    }
}

fn inferred_control_slot_count(para: &Paragraph) -> usize {
    let text_units: u32 = para.text.chars().map(char_utf16_width).sum();
    let from_char_count = para.char_count.saturating_sub(1).saturating_sub(text_units) / 8;

    let mut from_offsets = 0u32;
    let mut expected = 0u32;
    for (idx, c) in para.text.chars().enumerate() {
        let pos = para.char_offsets.get(idx).copied().unwrap_or(expected);
        if pos > expected {
            from_offsets += (pos - expected) / 8;
        }
        expected = pos.max(expected).saturating_add(char_utf16_width(c));
    }

    // fieldEnd는 8 code unit 슬롯이지만 para.controls[]에 대응 컨트롤이 없다.
    // field_ranges.len()이 fieldEnd 수와 정확히 일치하므로 빼서 보정한다.
    from_char_count
        .max(from_offsets)
        .saturating_sub(para.field_ranges.len() as u32) as usize
}

fn is_hwpx_inline_slot(control: &Control) -> bool {
    matches!(
        control,
        Control::Table(_)
            | Control::Shape(_)
            | Control::Picture(_)
            | Control::CharOverlap(_)
            | Control::Ruby(_)
            | Control::Equation(_)
            | Control::Field(_)
            | Control::Form(_)
            | Control::Footnote(_)
            | Control::Endnote(_)
            | Control::PageHide(_)
            | Control::PageNumberPos(_)
            | Control::NewNumber(_)
            | Control::Header(_)
            | Control::Footer(_)
            | Control::AutoNumber(_)
    )
}

fn flush_text_fragment(
    out: &mut String,
    text_buf: &mut String,
    tab_extended: &[[u16; 7]],
    tab_idx: &mut usize,
) {
    if !text_buf.is_empty() {
        out.push_str(&render_hp_t_content(text_buf, tab_extended, tab_idx));
        text_buf.clear();
    }
}

fn render_control_slot(out: &mut String, control: &Control, ctx: &mut SerializeContext) {
    match control {
        Control::Equation(eq) => {
            out.push_str(&render_equation(eq));
        }
        Control::Table(tbl) => match writer_to_string(|w| table::write_table(w, tbl, ctx)) {
            Ok(xml) => out.push_str(&xml),
            Err(e) => eprintln!("[hwpx] Table 직렬화 실패: {e}"),
        },
        Control::Picture(pic) => match writer_to_string(|w| picture::write_picture(w, pic, ctx)) {
            Ok(xml) => out.push_str(&xml),
            Err(e) => eprintln!("[hwpx] Picture 직렬화 실패: {e}"),
        },
        Control::Shape(shape) => {
            out.push_str(&render_shape(shape, ctx));
        }
        Control::Footnote(note) => {
            out.push_str(&render_footnote(note, ctx));
        }
        Control::Endnote(note) => {
            out.push_str(&render_endnote(note, ctx));
        }
        Control::Field(f) => {
            // fieldBegin은 <hp:ctrl>...</hp:ctrl>로 감싸야 함 (Table/Picture와 달리)
            match writer_to_string(|w| write_field_begin(w, f)) {
                Ok(xml) => {
                    out.push_str("<hp:ctrl>");
                    out.push_str(&xml);
                    out.push_str("</hp:ctrl>");
                }
                Err(e) => eprintln!("[hwpx] Field 직렬화 실패: {e}"),
            }
        }
        Control::PageHide(ph) => out.push_str(&render_page_hiding(ph)),
        Control::PageNumberPos(pn) => out.push_str(&render_page_num(pn)),
        Control::NewNumber(nn) => out.push_str(&render_new_num(nn)),
        Control::Header(h) => out.push_str(&render_header(h, ctx)),
        Control::Footer(f) => out.push_str(&render_footer(f, ctx)),
        Control::AutoNumber(an) => out.push_str(&render_autonum(an)),
        _ => {}
    }
}

/// 장식 문자(userChar/prefixChar/suffixChar)용 속성값. '\0'(미설정)은 빈 문자열.
fn ctrl_char_attr(c: char) -> String {
    if c == '\0' {
        String::new()
    } else {
        xml_escape(&c.to_string())
    }
}

/// `<hp:ctrl><hp:autoNum num=".." numType=".."><hp:autoNumFormat .../></hp:autoNum></hp:ctrl>`
/// 자동 번호(AutoNumber) 컨트롤. format은 pageNum formatType과 동일한 코드→문자열 매핑.
fn render_autonum(an: &AutoNumber) -> String {
    format!(
        concat!(
            r#"<hp:ctrl><hp:autoNum num="{num}" numType="{nt}">"#,
            r#"<hp:autoNumFormat type="{ty}" userChar="{u}" prefixChar="{p}" "#,
            r#"suffixChar="{s}" supscript="{sup}"/></hp:autoNum></hp:ctrl>"#
        ),
        num = an.number,
        nt = auto_number_type_to_str(an.number_type),
        ty = page_num_format_to_str(an.format),
        u = ctrl_char_attr(an.user_symbol),
        p = ctrl_char_attr(an.prefix_char),
        s = ctrl_char_attr(an.suffix_char),
        sup = an.superscript as u8,
    )
}

/// 머리말/꼬리말 적용 범위 → HWPX `applyPageType`. `parse_apply_page_type`의 역매핑.
fn apply_page_type_to_str(a: HeaderFooterApply) -> &'static str {
    match a {
        HeaderFooterApply::Both => "BOTH",
        HeaderFooterApply::Even => "EVEN",
        HeaderFooterApply::Odd => "ODD",
    }
}

/// `<hp:ctrl><hp:{header|footer} applyPageType=".."><hp:subList ...>문단들</hp:subList>...`
/// 머리말/꼬리말은 중첩 문단(subList)을 가진다 — render_note_sublist와 동일한 문단 직렬화
/// 경로(render_paragraph_parts)를 쓰되, subList 텍스트 영역 속성은 IR 보존값을 사용한다.
fn render_header_footer(
    tag: &str,
    h: HeaderFooterFields<'_>,
    ctx: &mut SerializeContext,
) -> String {
    let mut out = format!(
        concat!(
            r#"<hp:ctrl><hp:{tag} id="0" applyPageType="{apply}">"#,
            r#"<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" "#,
            r#"linkListIDRef="0" linkListNextIDRef="0" textWidth="{tw}" textHeight="{th}" "#,
            r#"hasTextRef="{tr}" hasNumRef="{nr}">"#
        ),
        tag = tag,
        apply = apply_page_type_to_str(h.apply_to),
        tw = h.text_width,
        th = h.text_height,
        tr = h.text_ref,
        nr = h.num_ref,
    );
    let mut vert_cursor: u32 = 0;
    for p in h.paragraphs.iter() {
        let (t, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
        vert_cursor = advance;
        let cs = first_run_char_shape_id(p);
        out.push_str(&render_hp_p_open(p, ctx.next_para_id()));
        out.push_str(&format!(r#"<hp:run charPrIDRef="{}">"#, cs));
        out.push_str(&t);
        out.push_str(r#"</hp:run><hp:linesegarray>"#);
        out.push_str(&linesegs);
        out.push_str(r#"</hp:linesegarray></hp:p>"#);
    }
    out.push_str(&format!("</hp:subList></hp:{tag}></hp:ctrl>", tag = tag));
    out
}

/// render_header_footer 공통 인자 묶음 (Header/Footer가 동일 필드를 가짐).
struct HeaderFooterFields<'a> {
    apply_to: HeaderFooterApply,
    text_width: u32,
    text_height: u32,
    text_ref: u8,
    num_ref: u8,
    paragraphs: &'a [Paragraph],
}

fn render_header(h: &Header, ctx: &mut SerializeContext) -> String {
    render_header_footer(
        "header",
        HeaderFooterFields {
            apply_to: h.apply_to,
            text_width: h.text_width,
            text_height: h.text_height,
            text_ref: h.text_ref,
            num_ref: h.num_ref,
            paragraphs: &h.paragraphs,
        },
        ctx,
    )
}

fn render_footer(f: &Footer, ctx: &mut SerializeContext) -> String {
    render_header_footer(
        "footer",
        HeaderFooterFields {
            apply_to: f.apply_to,
            text_width: f.text_width,
            text_height: f.text_height,
            text_ref: f.text_ref,
            num_ref: f.num_ref,
            paragraphs: &f.paragraphs,
        },
        ctx,
    )
}

/// `<hp:ctrl><hp:pageHiding .../></hp:ctrl>` — 감추기(PageHide) 컨트롤.
/// `parse_page_hiding_attrs`의 역매핑. bool → "0"/"1" (한컴 정합).
fn render_page_hiding(ph: &PageHide) -> String {
    format!(
        concat!(
            r#"<hp:ctrl><hp:pageHiding hideHeader="{}" hideFooter="{}" "#,
            r#"hideMasterPage="{}" hideBorder="{}" hideFill="{}" hidePageNum="{}"/></hp:ctrl>"#
        ),
        ph.hide_header as u8,
        ph.hide_footer as u8,
        ph.hide_master_page as u8,
        ph.hide_border as u8,
        ph.hide_fill as u8,
        ph.hide_page_num as u8,
    )
}

/// 쪽 번호 위치 코드(표 150) → HWPX `pos` 문자열. `parse_page_num_attrs`의 역매핑.
fn page_num_pos_to_str(pos: u8) -> &'static str {
    match pos {
        0 => "NONE",
        1 => "TOP_LEFT",
        2 => "TOP_CENTER",
        3 => "TOP_RIGHT",
        4 => "BOTTOM_LEFT",
        5 => "BOTTOM_CENTER",
        6 => "BOTTOM_RIGHT",
        7 => "OUTSIDE_TOP",
        8 => "OUTSIDE_BOTTOM",
        9 => "INSIDE_TOP",
        10 => "INSIDE_BOTTOM",
        _ => "BOTTOM_CENTER",
    }
}

/// 번호 형식 코드(표 134) → HWPX `formatType` 문자열. `parse_page_num_attrs`의 역매핑.
fn page_num_format_to_str(fmt: u8) -> &'static str {
    match fmt {
        0 => "DIGIT",
        1 => "CIRCLE_DIGIT",
        2 => "ROMAN_CAPITAL",
        3 => "ROMAN_SMALL",
        4 => "LATIN_CAPITAL",
        5 => "LATIN_SMALL",
        6 => "HANGUL",
        7 => "HANJA",
        _ => "DIGIT",
    }
}

/// `<hp:ctrl><hp:pageNum .../></hp:ctrl>` — 쪽 번호 위치(PageNumberPos) 컨트롤.
fn render_page_num(pn: &PageNumberPos) -> String {
    // dash_char 기본값은 '-' (모델: 항상 '-'); '\0'이면 '-'로 폴백.
    let side = if pn.dash_char == '\0' {
        '-'
    } else {
        pn.dash_char
    };
    format!(
        r#"<hp:ctrl><hp:pageNum pos="{}" formatType="{}" sideChar="{}"/></hp:ctrl>"#,
        page_num_pos_to_str(pn.position),
        page_num_format_to_str(pn.format),
        xml_escape(&side.to_string()),
    )
}

/// 번호 종류 → HWPX `numType` 문자열. `parse_num_type`의 역매핑(Picture→FIGURE).
fn auto_number_type_to_str(t: AutoNumberType) -> &'static str {
    match t {
        AutoNumberType::Page => "PAGE",
        AutoNumberType::Footnote => "FOOTNOTE",
        AutoNumberType::Endnote => "ENDNOTE",
        AutoNumberType::Picture => "FIGURE",
        AutoNumberType::Table => "TABLE",
        AutoNumberType::Equation => "EQUATION",
    }
}

/// `<hp:ctrl><hp:newNum .../></hp:ctrl>` — 새 번호 지정(NewNumber) 컨트롤.
fn render_new_num(nn: &NewNumber) -> String {
    format!(
        r#"<hp:ctrl><hp:newNum num="{}" numType="{}"/></hp:ctrl>"#,
        nn.number,
        auto_number_type_to_str(nn.number_type),
    )
}

fn writer_to_string<F>(f: F) -> Result<String, SerializeError>
where
    F: FnOnce(&mut Writer<Vec<u8>>) -> Result<(), SerializeError>,
{
    let mut writer = Writer::new(Vec::new());
    f(&mut writer)?;
    let bytes = writer.into_inner();
    String::from_utf8(bytes)
        .map_err(|e| SerializeError::XmlError(format!("invalid UTF-8 from XML writer: {e}")))
}

fn render_shape(shape: &ShapeObject, ctx: &SerializeContext) -> String {
    // Rectangle: Writer-based serializer (drawText 포함)
    if let ShapeObject::Rectangle(r) = shape {
        return match writer_to_string(|w| super::shape::write_rect(w, r)) {
            Ok(xml) => xml,
            Err(e) => {
                eprintln!("[hwpx] Shape::Rectangle 직렬화 실패: {e}");
                String::new()
            }
        };
    }
    // Line: Writer-based serializer
    if let ShapeObject::Line(l) = shape {
        return match writer_to_string(|w| super::shape::write_line(w, l)) {
            Ok(xml) => xml,
            Err(e) => {
                eprintln!("[hwpx] Shape::Line 직렬화 실패: {e}");
                String::new()
            }
        };
    }
    let (tag, c) = match shape {
        ShapeObject::Rectangle(_) | ShapeObject::Line(_) => unreachable!(),
        ShapeObject::Ellipse(e) => ("ellipse", &e.common),
        ShapeObject::Arc(a) => ("arc", &a.common),
        ShapeObject::Polygon(p) => ("polygon", &p.common),
        ShapeObject::Curve(cv) => ("curve", &cv.common),
        ShapeObject::Group(g) => ("container", &g.common),
        ShapeObject::Picture(pic) => {
            return match writer_to_string(|w| picture::write_picture(w, pic, ctx)) {
                Ok(xml) => xml,
                Err(e) => {
                    eprintln!("[hwpx] Shape::Picture 직렬화 실패: {e}");
                    String::new()
                }
            };
        }
        ShapeObject::Chart(ch) => ("chart", &ch.common),
        ShapeObject::Ole(o) => ("ole", &o.common),
    };
    render_common_shape_xml(tag, c)
}

fn render_common_shape_xml(tag: &str, c: &CommonObjAttr) -> String {
    format!(
        concat!(
            r#"<hp:{tag} id="{id}" zOrder="{zo}" textWrap="{tw}" textFlow="BOTH_SIDES" lock="0">"#,
            r#"<hp:sz width="{w}" height="{h}" widthRelTo="ABSOLUTE" heightRelTo="ABSOLUTE"/>"#,
            r#"<hp:pos treatAsChar="{tac}" vertRelTo="{vr}" vertAlign="{va}" horzRelTo="{hr}" horzAlign="{ha}" vertOffset="{vo}" horzOffset="{ho}"/>"#,
            r#"<hp:outMargin left="{ml}" right="{mr}" top="{mt}" bottom="{mb}"/>"#,
            r#"</hp:{tag}>"#,
        ),
        tag = tag,
        id = c.instance_id,
        zo = c.z_order,
        tw = text_wrap_to_hwpx(c.text_wrap),
        tac = if c.treat_as_char { "1" } else { "0" },
        w = c.width,
        h = c.height,
        vr = vert_rel_to_hwpx(c.vert_rel_to),
        va = vert_align_to_hwpx(c.vert_align),
        hr = horz_rel_to_hwpx(c.horz_rel_to),
        ha = horz_align_to_hwpx(c.horz_align),
        vo = c.vertical_offset,
        ho = c.horizontal_offset,
        ml = c.margin.left,
        mr = c.margin.right,
        mt = c.margin.top,
        mb = c.margin.bottom,
    )
}

fn render_note_sublist(
    tag: &str,
    number: u16,
    paragraphs: &[Paragraph],
    ctx: &mut SerializeContext,
) -> String {
    let mut out = format!(
        r#"<hp:ctrl><hp:{tag} number="{num}"><hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">"#,
        tag = tag,
        num = number,
    );
    let mut vert_cursor: u32 = 0;
    for p in paragraphs.iter() {
        let (t, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
        vert_cursor = advance;
        let cs = first_run_char_shape_id(p);
        out.push_str(&render_hp_p_open(p, ctx.next_para_id()));
        out.push_str(&format!(r#"<hp:run charPrIDRef="{}">"#, cs));
        out.push_str(&t);
        out.push_str(r#"</hp:run><hp:linesegarray>"#);
        out.push_str(&linesegs);
        out.push_str(r#"</hp:linesegarray></hp:p>"#);
    }
    out.push_str(&format!("</hp:subList></hp:{tag}></hp:ctrl>", tag = tag));
    out
}

fn render_footnote(note: &Footnote, ctx: &mut SerializeContext) -> String {
    render_note_sublist("footNote", note.number, &note.paragraphs, ctx)
}

fn render_endnote(note: &Endnote, ctx: &mut SerializeContext) -> String {
    render_note_sublist("endNote", note.number, &note.paragraphs, ctx)
}

fn render_equation(eq: &Equation) -> String {
    let c = &eq.common;
    let id = c.instance_id.to_string();
    let z_order = c.z_order.to_string();
    let version = xml_escape(&eq.version_info);
    let baseline = eq.baseline.to_string();
    let text_color = color_ref_to_hwpx(eq.color);
    let base_unit = eq.font_size.to_string();
    let font = xml_escape(&eq.font_name);
    let script = xml_escape(&eq.script);
    let width = c.width.to_string();
    let height = c.height.to_string();
    let treat = if c.treat_as_char { "1" } else { "0" };
    let vert_offset = c.vertical_offset.to_string();
    let horz_offset = c.horizontal_offset.to_string();
    let margin_left = c.margin.left.to_string();
    let margin_right = c.margin.right.to_string();
    let margin_top = c.margin.top.to_string();
    let margin_bottom = c.margin.bottom.to_string();

    format!(
        r#"<hp:equation id="{id}" zOrder="{z_order}" numberingType="EQUATION" textWrap="{}" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" instid="{id}" version="{version}" baseLine="{baseline}" textColor="{text_color}" baseUnit="{base_unit}" font="{font}"><hp:script>{script}</hp:script><hp:sz width="{width}" widthRelTo="ABSOLUTE" height="{height}" heightRelTo="ABSOLUTE"/><hp:pos treatAsChar="{treat}" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="{}" horzRelTo="{}" vertAlign="{}" horzAlign="{}" vertOffset="{vert_offset}" horzOffset="{horz_offset}"/><hp:outMargin left="{margin_left}" right="{margin_right}" top="{margin_top}" bottom="{margin_bottom}"/></hp:equation>"#,
        text_wrap_to_hwpx(c.text_wrap),
        vert_rel_to_hwpx(c.vert_rel_to),
        horz_rel_to_hwpx(c.horz_rel_to),
        vert_align_to_hwpx(c.vert_align),
        horz_align_to_hwpx(c.horz_align),
    )
}

fn char_utf16_width(c: char) -> u32 {
    if c == '\t' {
        8
    } else if (c as u32) > 0xFFFF {
        2
    } else {
        1
    }
}

fn color_ref_to_hwpx(color: u32) -> String {
    if color == 0xFFFFFFFF {
        return "none".to_string();
    }

    let a = (color >> 24) & 0xFF;
    let r = color & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = (color >> 16) & 0xFF;
    if a == 0 {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{a:02X}{r:02X}{g:02X}{b:02X}")
    }
}

fn text_wrap_to_hwpx(wrap: TextWrap) -> &'static str {
    match wrap {
        TextWrap::Square => "SQUARE",
        TextWrap::Tight => "TIGHT",
        TextWrap::Through => "THROUGH",
        TextWrap::TopAndBottom => "TOP_AND_BOTTOM",
        TextWrap::BehindText => "BEHIND_TEXT",
        TextWrap::InFrontOfText => "IN_FRONT_OF_TEXT",
    }
}

fn vert_rel_to_hwpx(rel: VertRelTo) -> &'static str {
    match rel {
        VertRelTo::Paper => "PAPER",
        VertRelTo::Page => "PAGE",
        VertRelTo::Para => "PARA",
    }
}

fn horz_rel_to_hwpx(rel: HorzRelTo) -> &'static str {
    match rel {
        HorzRelTo::Paper => "PAPER",
        HorzRelTo::Page => "PAGE",
        HorzRelTo::Column => "COLUMN",
        HorzRelTo::Para => "PARA",
    }
}

fn vert_align_to_hwpx(align: VertAlign) -> &'static str {
    match align {
        VertAlign::Top => "TOP",
        VertAlign::Center => "CENTER",
        VertAlign::Bottom => "BOTTOM",
        VertAlign::Inside => "INSIDE",
        VertAlign::Outside => "OUTSIDE",
    }
}

fn horz_align_to_hwpx(align: HorzAlign) -> &'static str {
    match align {
        HorzAlign::Left => "LEFT",
        HorzAlign::Center => "CENTER",
        HorzAlign::Right => "RIGHT",
        HorzAlign::Inside => "INSIDE",
        HorzAlign::Outside => "OUTSIDE",
    }
}

/// IR의 `line_segs` 를 그대로 XML로 직렬화 (9개 필드 전부 IR 값 사용).
///
/// rhwp 는 자신의 문서에서 비표준 lineseg 를 **새로 생산하지 않는다**.
/// 원본 한컴 파일의 lineseg 값이 파서에 의해 `Paragraph.line_segs` 에 담겼다면,
/// 저장 시 그 값을 훼손 없이 보존한다.
fn render_lineseg_array_from_ir(segs: &[LineSeg]) -> String {
    let mut out = String::new();
    for seg in segs {
        out.push_str(&format!(
            r#"<hp:lineseg textpos="{}" vertpos="{}" vertsize="{}" textheight="{}" baseline="{}" spacing="{}" horzpos="{}" horzsize="{}" flags="{}"/>"#,
            seg.text_start,
            seg.vertical_pos,
            seg.line_height,
            seg.text_height,
            seg.baseline_distance,
            seg.line_spacing,
            seg.column_start,
            seg.segment_width,
            seg.tag,
        ));
    }
    out
}

/// IR 기반 다음 문단의 vert_start 계산 — 마지막 lineseg 의 vpos + lh 사용.
fn next_vert_cursor_from_ir(segs: &[LineSeg], vert_start: u32) -> u32 {
    if let Some(last) = segs.last() {
        // vertical_pos 는 섹션 시작 기준 절대값일 수도, 문단 기준 상대값일 수도 있음.
        // 현재 rhwp 는 섹션 절대값이므로 그대로 + lh 로 다음 커서 산출.
        let next = (last.vertical_pos as i64) + (last.line_height.max(0) as i64);
        if next > vert_start as i64 {
            next as u32
        } else {
            vert_start + VERT_STEP
        }
    } else {
        vert_start + VERT_STEP
    }
}

/// Fallback — IR 에 line_segs 가 없는 경우에만 사용 (예: `Document::default()`).
/// 과거 동작을 보존하기 위해 기존 정적값으로 lineseg 생성.
fn render_lineseg_array_fallback(text: &str, vert_start: u32) -> (String, u32) {
    let mut linesegs = String::new();
    push_lineseg_static(&mut linesegs, 0, vert_start);
    let mut utf16_pos: u32 = 0;
    let mut lines_in_para: u32 = 0;
    for c in text.chars() {
        let u16_len = c.len_utf16() as u32;
        match c {
            '\t' | '\n' => {
                utf16_pos += u16_len;
                if c == '\n' {
                    lines_in_para += 1;
                    push_lineseg_static(
                        &mut linesegs,
                        utf16_pos,
                        vert_start + lines_in_para * VERT_STEP,
                    );
                }
            }
            c if (c as u32) < 0x20 => {}
            _ => utf16_pos += u16_len,
        }
    }
    let vert_end = vert_start + (lines_in_para + 1) * VERT_STEP;
    (linesegs, vert_end)
}

fn flush_buf(t_xml: &mut String, buf: &mut String) {
    if !buf.is_empty() {
        t_xml.push_str(&xml_escape(buf));
        buf.clear();
    }
}

/// Fallback 전용 static lineseg 생성기 — IR에 값이 없을 때만 사용.
/// 주: 이 함수의 출력은 "명세 상 정확한 값" 이 아닌 정적 자리표이므로,
/// 호출 후 문서는 `DocumentCore::from_bytes` 의 `reflow_zero_height_paragraphs`
/// 또는 사용자의 `reflow_linesegs_on_demand` 로 재계산되어야 한다.
fn push_lineseg_static(out: &mut String, textpos: u32, vertpos: u32) {
    out.push_str(&format!(
        r#"<hp:lineseg textpos="{}" vertpos="{}" vertsize="1000" textheight="1000" baseline="850" spacing="600" horzpos="0" horzsize="{}" flags="{}"/>"#,
        textpos, vertpos, HORZ_SIZE, LINE_FLAGS,
    ));
}

fn replace_first_linesegs(xml: &str, new_inner: &str) -> String {
    let open = xml
        .find(LINESEG_SLOT_OPEN)
        .expect("template has linesegarray");
    let inner_start = open + LINESEG_SLOT_OPEN.len();
    let close_rel = xml[inner_start..]
        .find(LINESEG_SLOT_CLOSE)
        .expect("template has closing linesegarray");
    let inner_end = inner_start + close_rel;
    let mut out = String::with_capacity(xml.len() + new_inner.len());
    out.push_str(&xml[..inner_start]);
    out.push_str(new_inner);
    out.push_str(&xml[inner_end..]);
    out
}

/// [#1166] 템플릿 pagePr 의 고정 용지 속성(landscape/width/height)을 IR page_def
/// 값으로 치환한다. 종전엔 템플릿 하드코딩값(landscape="WIDELY" width=59528
/// height=84186)이 그대로 출력되어 HWPX 저장 시 가로/세로 + 용지 크기가 손실됐다.
///
/// OWPML landscape: WIDELY=세로(landscape=false), NARROWLY=가로(landscape=true).
/// width/height 는 짧은변/긴변 그대로 (HWP 바이너리 동일 규약).
fn replace_page_pr(xml: &str, page_def: &crate::model::page::PageDef) -> String {
    // 템플릿의 pagePr 여는 태그(고정 문자열) → IR 기반으로 교체.
    const TEMPLATE_PAGE_PR: &str =
        r#"<hp:pagePr landscape="WIDELY" width="59528" height="84186" gutterType="LEFT_ONLY">"#;
    let landscape = if page_def.landscape {
        "NARROWLY"
    } else {
        "WIDELY"
    };
    let new_page_pr = format!(
        r#"<hp:pagePr landscape="{}" width="{}" height="{}" gutterType="LEFT_ONLY">"#,
        landscape, page_def.width, page_def.height,
    );
    if xml.contains(TEMPLATE_PAGE_PR) {
        xml.replacen(TEMPLATE_PAGE_PR, &new_page_pr, 1)
    } else {
        // 템플릿이 변경됐거나 이미 치환된 경우 — 원본 유지(회귀 방지).
        xml.to_string()
    }
}

// `TEMPLATE_RUN_BEFORE_TEXT` 는 패턴 인식용 상수로만 쓰이므로 명시 참조.
#[allow(dead_code)]
fn _template_anchor_hint() {
    let _ = TEMPLATE_RUN_BEFORE_TEXT;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::{CharShapeRef, Paragraph};

    fn make_doc_with_paragraph(para: Paragraph) -> (Document, Section) {
        let mut section = Section::default();
        section.paragraphs.push(para);
        let mut doc = Document::default();
        doc.sections.push(section.clone());
        (doc, section)
    }

    #[test]
    fn hp_p_attrs_reflect_para_shape_id_and_style_id() {
        let mut para = Paragraph::default();
        para.para_shape_id = 7;
        para.style_id = 3;
        para.text = "hi".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains(r#"paraPrIDRef="7""#),
            "<hp:p> must reflect para_shape_id=7: {}",
            &xml[..200.min(xml.len())]
        );
        assert!(
            xml.contains(r#"styleIDRef="3""#),
            "<hp:p> must reflect style_id=3"
        );
    }

    #[test]
    fn hp_run_reflects_first_char_shape_id() {
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.char_shapes.push(CharShapeRef {
            start_pos: 0,
            char_shape_id: 42,
        });
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains(r#"<hp:run charPrIDRef="42"><hp:t>hello</hp:t>"#),
            "first run must use char_shape_id 42, xml excerpt around <hp:t>: {:?}",
            xml.find("<hp:t>")
                .map(|i| &xml[i.saturating_sub(50)..(i + 50).min(xml.len())])
        );
    }

    #[test]
    fn page_break_paragraph_emits_attr() {
        let mut para = Paragraph::default();
        para.text = "p1".to_string();
        para.column_type = crate::model::paragraph::ColumnBreakType::Page;
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains(r#"pageBreak="1""#),
            "pageBreak must be 1 for Page column_type"
        );
        assert!(xml.contains(r#"columnBreak="0""#));
    }

    #[test]
    fn default_paragraph_keeps_zero_attrs() {
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(xml.contains(r#"paraPrIDRef="0""#));
        assert!(xml.contains(r#"styleIDRef="0""#));
        // char_shapes 가 비어있으면 fallback 0
        assert!(xml.contains(r#"<hp:run charPrIDRef="0">"#));
    }

    #[test]
    fn additional_paragraphs_use_their_own_char_shape() {
        let mut p1 = Paragraph::default();
        p1.text = "first".to_string();
        p1.char_shapes.push(CharShapeRef {
            start_pos: 0,
            char_shape_id: 5,
        });
        let mut p2 = Paragraph::default();
        p2.text = "second".to_string();
        p2.para_shape_id = 2;
        p2.char_shapes.push(CharShapeRef {
            start_pos: 0,
            char_shape_id: 6,
        });
        let mut section = Section::default();
        section.paragraphs.push(p1);
        section.paragraphs.push(p2);
        let mut doc = Document::default();
        doc.sections.push(section.clone());
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        // 두 번째 문단: paraPrIDRef=2, charPrIDRef=6
        assert!(xml.contains(r#"paraPrIDRef="2""#));
        assert!(
            xml.matches(r#"charPrIDRef="6""#).count() >= 1,
            "second paragraph must emit charPrIDRef=6"
        );
    }

    // ---------- #177 Stage 2: IR 기반 lineseg 출력 ----------

    use crate::model::paragraph::LineSeg;

    #[test]
    fn task177_lineseg_reflects_ir_values() {
        // IR에 담긴 lineseg 값이 XML 속성에 그대로 반영되는지 확인.
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.line_segs.push(LineSeg {
            text_start: 0,
            vertical_pos: 5000,
            line_height: 1200,
            text_height: 1100,
            baseline_distance: 900,
            line_spacing: 700,
            column_start: 100,
            segment_width: 50000,
            tag: 999,
        });
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(xml.contains(r#"<hp:lineseg textpos="0" vertpos="5000" vertsize="1200" textheight="1100" baseline="900" spacing="700" horzpos="100" horzsize="50000" flags="999"/>"#),
            "lineseg must reflect IR values exactly, got XML: {}",
            &xml[xml.find("<hp:lineseg").unwrap_or(0)..(xml.find("<hp:lineseg").unwrap_or(0) + 200).min(xml.len())]);
    }

    #[test]
    fn task177_multiple_linesegs_preserved_in_order() {
        let mut para = Paragraph::default();
        para.text = "three\nlines\nhere".to_string();
        for (i, (tp, vp, lh)) in [(0u32, 0i32, 1000), (6, 1500, 1200), (12, 3100, 1100)]
            .iter()
            .enumerate()
        {
            let _ = i;
            para.line_segs.push(LineSeg {
                text_start: *tp,
                vertical_pos: *vp,
                line_height: *lh,
                text_height: *lh,
                baseline_distance: 850,
                line_spacing: 600,
                column_start: 0,
                segment_width: 42520,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            });
        }
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        // 3개 lineseg 모두 출력되고 각각의 vertsize 값이 IR 값과 일치
        assert_eq!(xml.matches("<hp:lineseg ").count(), 3);
        assert!(xml.contains(r#"textpos="0" vertpos="0" vertsize="1000""#));
        assert!(xml.contains(r#"textpos="6" vertpos="1500" vertsize="1200""#));
        assert!(xml.contains(r#"textpos="12" vertpos="3100" vertsize="1100""#));
    }

    #[test]
    fn task177_fallback_used_when_ir_empty() {
        // IR 의 line_segs 가 비어있으면 fallback 경로로 정적 값 출력.
        let mut para = Paragraph::default();
        para.text = "a\nb".to_string(); // 소프트브레이크 1개 → fallback 은 lineseg 2개 생성
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        // 정적 fallback: vertsize=1000, textheight=1000, baseline=850, spacing=600
        assert!(xml.contains(r#"vertsize="1000""#));
        assert!(xml.contains(r#"baseline="850""#));
    }

    #[test]
    fn task177_ir_lineseg_takes_precedence_over_text() {
        // text 의 \n 개수가 2개(lineseg 3개 기대)이지만 IR의 line_segs 는 1개만 있음.
        // IR 기반 출력이 우선 — 1개만 출력돼야 함.
        let mut para = Paragraph::default();
        para.text = "a\nb\nc".to_string(); // 3줄
        para.line_segs.push(LineSeg {
            text_start: 0,
            vertical_pos: 0,
            line_height: 2000, // IR 값
            text_height: 2000,
            baseline_distance: 1700,
            line_spacing: 300,
            column_start: 0,
            segment_width: 40000,
            tag: 0,
        });
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        // IR 에 1개만 있으므로 lineseg 도 1개만 출력 (rhwp 는 원본 보존)
        assert_eq!(xml.matches("<hp:lineseg ").count(), 1);
        assert!(
            xml.contains(r#"vertsize="2000""#),
            "IR value 2000 must be used, not fallback 1000"
        );
    }

    // ---------- #1289: Bookmark / Field dispatcher 연결 ----------

    use crate::model::control::{Bookmark, Control, Field, FieldType};
    use crate::model::paragraph::FieldRange;

    #[test]
    fn task1289_bookmark_emits_ctrl_wrapper() {
        // Bookmark는 슬롯 시스템이 위치를 추적할 수 없으므로 문단 시작에 배치한다.
        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.char_count = 6; // "hello"(5) + para_end(1)
        para.controls.push(Control::Bookmark(Bookmark {
            name: "test_bm".to_string(),
        }));
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"<hp:ctrl><hp:bookmark name="test_bm"/></hp:ctrl>"#),
            "bookmark must be wrapped in <hp:ctrl>: {}",
            &xml[..300.min(xml.len())]
        );
        assert!(xml.contains("hello"), "text must still be present");
    }

    #[test]
    fn task1289_field_begin_end_roundtrip() {
        // HWPX 파서가 생성하는 구조 시뮬레이션:
        // fieldBegin(8 cu) + "hello"(5 cu) + fieldEnd(8 cu) + para_end(1 cu) = 22
        // para.text 에는 "hello"만 있고 char_offsets 가 +8 오프셋으로 시작한다.
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 99;

        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.char_count = 22;
        para.char_offsets = vec![8, 9, 10, 11, 12];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 5,
            control_idx: 0,
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:ctrl><hp:fieldBegin id="99" type="CLICKHERE""#),
            "fieldBegin must be emitted: {}",
            &xml[..500.min(xml.len())]
        );
        assert!(
            xml.contains(r#"<hp:ctrl><hp:fieldEnd beginIDRef="99"/></hp:ctrl>"#),
            "fieldEnd must be emitted: {}",
            &xml[..500.min(xml.len())]
        );
        assert!(xml.contains("hello"), "field text must be present");

        // 순서 검증: fieldBegin < "hello" < fieldEnd
        let begin_pos = xml.find("fieldBegin").expect("fieldBegin");
        let hello_pos = xml.find("hello").expect("hello");
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        assert!(begin_pos < hello_pos, "fieldBegin must precede text");
        assert!(hello_pos < end_pos, "text must precede fieldEnd");
    }

    #[test]
    fn task1289_field_end_at_para_boundary() {
        // end_char_idx == text.len() 인 경우: 루프 내 감지 불가 → 루프 후 처리
        let mut f = Field::default();
        f.field_type = FieldType::Date;
        f.field_id = 7;

        let mut para = Paragraph::default();
        para.text = "abc".to_string();
        para.char_count = 20; // fieldBegin(8) + "abc"(3) + fieldEnd(8) + para_end(1)
        para.char_offsets = vec![8, 9, 10];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 3, // == text.len() → 루프 후 처리 경로
            control_idx: 0,
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:fieldEnd beginIDRef="7"/>"#),
            "fieldEnd must be emitted even when end_char_idx == text.len(): {}",
            &xml[..400.min(xml.len())]
        );
    }

    // ---------- #1298: 0-length field range fieldBegin/fieldEnd 인터리빙 ----------

    #[test]
    fn task1298_zero_length_field_at_para_start() {
        // 0-length 필드 at position 0 (start=0, end=0):
        // HWP stream: fieldBegin(8cu) fieldEnd(8cu) "hello"(5cu) para_end(1cu) = 22cu
        // char_offsets: [16, 17, 18, 19, 20] (fieldBegin+fieldEnd 갭 16 이후 텍스트)
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 55;

        let mut para = Paragraph::default();
        para.text = "hello".to_string();
        para.char_count = 22;
        para.char_offsets = vec![16, 17, 18, 19, 20];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 0, // 0-length
            control_idx: 0,
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:ctrl><hp:fieldBegin id="55""#),
            "fieldBegin must be emitted: {}",
            &xml[..500.min(xml.len())]
        );
        assert!(
            xml.contains(r#"<hp:ctrl><hp:fieldEnd beginIDRef="55"/></hp:ctrl>"#),
            "fieldEnd must be emitted: {}",
            &xml[..500.min(xml.len())]
        );
        assert!(xml.contains("hello"), "text must still be present");

        // 순서 검증: fieldBegin < fieldEnd < "hello"
        let begin_pos = xml.find("fieldBegin").expect("fieldBegin");
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        let hello_pos = xml.find("hello").expect("hello");
        assert!(begin_pos < end_pos, "fieldBegin must precede fieldEnd");
        assert!(
            end_pos < hello_pos,
            "fieldEnd must precede text for 0-length field"
        );
    }

    #[test]
    fn task1298_zero_length_field_mid_text() {
        // 0-length 필드 at position 3 (start=3, end=3), text="ABCDE":
        // HWP stream: A B C fieldBegin(8cu) fieldEnd(8cu) D E para_end
        // char_offsets: [0,1,2, 19,20] (D 앞에 16cu 갭)
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 77;

        let mut para = Paragraph::default();
        para.text = "ABCDE".to_string();
        para.char_count = 5 + 8 + 8 + 1; // text + fieldBegin + fieldEnd + para_end
        para.char_offsets = vec![0, 1, 2, 19, 20];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 3,
            end_char_idx: 3, // 0-length mid-text
            control_idx: 0,
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains("ABCDE") || (xml.contains("ABC") && xml.contains("DE")),
            "all text must be present: {}",
            &xml[..500.min(xml.len())]
        );

        // 순서 검증: "ABC" < fieldBegin < fieldEnd < "DE"
        let begin_pos = xml.find("fieldBegin").expect("fieldBegin");
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        // ABC는 fieldBegin 앞에
        let abc_pos = xml.find('A').expect("A");
        // DE는 fieldEnd 뒤에 (fieldEnd 태그 닫힘 이후)
        let field_end_close =
            xml.find("fieldEnd").unwrap() + xml[xml.find("fieldEnd").unwrap()..].find('>').unwrap();
        let de_pos = xml[field_end_close..]
            .find('D')
            .map(|p| p + field_end_close)
            .expect("D after fieldEnd");

        assert!(abc_pos < begin_pos, "ABC must precede fieldBegin");
        assert!(begin_pos < end_pos, "fieldBegin must precede fieldEnd");
        assert!(end_pos < de_pos, "fieldEnd must precede DE");
    }

    // ---------- #1321: 빈 문단(text == "")의 0-length field 순서 ----------

    #[test]
    fn task1321_zero_length_field_in_empty_paragraph() {
        // 빈 문단(text="")에 0-length 필드:
        // HWP stream: fieldBegin(8cu) + fieldEnd(8cu) + para_end(1cu) = 17cu
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 99;

        let mut para = Paragraph::default();
        para.text = "".to_string();
        para.char_count = 17;
        para.char_offsets = vec![];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 0,
            control_idx: 0,
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:fieldBegin id="99""#),
            "fieldBegin must be emitted: {}",
            &xml[..400.min(xml.len())]
        );
        assert!(
            xml.contains(r#"<hp:fieldEnd beginIDRef="99"/>"#),
            "fieldEnd must be emitted: {}",
            &xml[..400.min(xml.len())]
        );

        let begin_pos = xml.find("fieldBegin").expect("fieldBegin");
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        assert!(
            begin_pos < end_pos,
            "빈 문단에서도 fieldBegin이 fieldEnd보다 앞에 와야 한다: {}",
            &xml[..400.min(xml.len())]
        );
    }
}
