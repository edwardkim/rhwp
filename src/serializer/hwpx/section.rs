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
    AutoNumber, AutoNumberType, CharOverlap, Control, Equation, Field, NewNumber, PageHide,
    PageNumberPos, Ruby,
};
use crate::model::document::{Document, Section, SectionDef};
use crate::model::footnote::{Endnote, Footnote};
use crate::model::header_footer::{Footer, Header, HeaderFooterApply};
use crate::model::page::{ColumnDef, ColumnDirection, ColumnType};
use crate::model::paragraph::{ColumnBreakType, LineSeg, OrphanFieldEnd, Paragraph};
use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, ShapeObject, TextWrap, VertAlign, VertRelTo,
};

use super::context::SerializeContext;
use super::field::{write_bookmark, write_field_begin, write_field_end, write_field_end_full};
use super::utils::xml_escape;
use super::SerializeError;
use super::{picture, table};

const EMPTY_SECTION_XML: &str = include_str!("templates/empty_section0.xml");

/// MEMO subList 여는 태그 (#1391) — 실물(aift) 고정 속성.
const SUB_LIST_OPEN: &str = r#"<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">"#;
const LINESEG_SLOT_OPEN: &str = "<hp:linesegarray>";
const LINESEG_SLOT_CLOSE: &str = "</hp:linesegarray>";
const PARA_CLOSE: &str = "</hp:p></hs:sec>";

// 템플릿 내 첫 <hp:p> 태그의 실제 문자열 (id="3121190098" 랜덤 해시 포함).
// 템플릿은 정적이므로 이 문자열이 고정 위치에 있음이 보장됨.
const TEMPLATE_FIRST_P_TAG: &str = r#"<hp:p id="3121190098" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0">"#;
// 템플릿 첫 문단의 텍스트 run 전체 — 템플릿 내 유일하게 1회 등장 (#1378).
const TEMPLATE_TEXT_RUN: &str = r#"<hp:run charPrIDRef="0"><hp:t/></hp:run>"#;
// 템플릿 첫 run(secPr/colPr 전용)의 여는 태그 + secPr 시작 — run id 정비용 anchor (#1378).
const TEMPLATE_SECPR_RUN_OPEN: &str = r#"<hp:run charPrIDRef="0"><hp:secPr "#;

// [#1407] 템플릿의 하드코딩 본문 colPr(단 정의) — 단일 단 기본값. 첫 문단 IR 에
// ColumnDef 가 있으면 이 anchor 를 IR 값으로 치환한다 (#1388 secPr 동형).
const TEMPLATE_BODY_COL_PR: &str = r#"<hp:ctrl><hp:colPr id="" type="NEWSPAPER" layout="LEFT" colCount="1" sameSz="1" sameGap="0"/></hp:ctrl>"#;

// [#1637] 템플릿의 하드코딩 secPr visibility — 전부 기본값(미숨김/SHOW_ALL). 원본의
// visibility 를 IR(SectionDef) 값으로 치환하지 않으면 hideFirstEmptyLine 등이 드롭되어
// (특히 ="1" → "0") 선두 빈줄이 가시화되며 본문이 밀려 페이지네이션이 달라진다 (#1388 동형).
const TEMPLATE_VISIBILITY: &str = r#"<hp:visibility hideFirstHeader="0" hideFirstFooter="0" hideFirstMasterPage="0" border="SHOW_ALL" fill="SHOW_ALL" hideFirstPageNum="0" hideFirstEmptyLine="0" showLineNumber="0"/>"#;

/// SectionDef 의 visibility 플래그를 `<hp:visibility>` 요소로 직렬화.
///
/// 파서가 IR 에 보존하는 6필드(hideFirstHeader/Footer/MasterPage·border·fill·
/// hideFirstEmptyLine)만 치환하고, IR 미보존 필드(hideFirstPageNum·showLineNumber)는
/// 템플릿 기본값("0")을 유지한다.
fn render_visibility(sd: &SectionDef) -> String {
    let b = |v: bool| if v { "1" } else { "0" };
    let bf = |v: bool| if v { "HIDE_ALL" } else { "SHOW_ALL" };
    format!(
        r#"<hp:visibility hideFirstHeader="{}" hideFirstFooter="{}" hideFirstMasterPage="{}" border="{}" fill="{}" hideFirstPageNum="0" hideFirstEmptyLine="{}" showLineNumber="0"/>"#,
        b(sd.hide_header),
        b(sd.hide_footer),
        b(sd.hide_master_page),
        bf(sd.hide_border),
        bf(sd.hide_fill),
        b(sd.hide_empty_line),
    )
}

/// 템플릿의 visibility 고정 문자열을 IR 기반 값으로 1회 치환.
fn replace_visibility(xml: &str, sd: &SectionDef) -> String {
    xml.replacen(TEMPLATE_VISIBILITY, &render_visibility(sd), 1)
}

/// 레퍼런스 기준 줄 레이아웃 파라미터.
const VERT_STEP: u32 = 1600; // vertsize(1000) + spacing(600)
/// 탭 기본 폭 (한컴이 열면서 재계산하지만 초기값으로 필요).
const TAB_DEFAULT_WIDTH: u32 = 4000;

/// Stage 2 진입점. `ctx` 는 Stage 3+ 에서 파라미터 검증에 사용.
pub fn write_section(
    section: &Section,
    doc: &Document,
    index: usize,
    ctx: &mut SerializeContext,
) -> Result<Vec<u8>, SerializeError> {
    let mut vert_cursor: u32 = 0;

    let first_para = section.paragraphs.first();
    // [#1584] 첫 문단 렌더 직전 set — 본문 첫 ColumnDef(섹션 템플릿 흡수분)의 인라인
    // XML 방출만 1회 억제한다(슬롯 위치는 보존). 렌더 직후 reset 하여 추가 문단 누설 방지.
    ctx.body_coldef_template_pending = true;
    let (first_runs, first_linesegs, first_advance) = match first_para {
        Some(p) => render_paragraph_parts(p, vert_cursor, ctx),
        // 문단이 없는 섹션(비파싱 IR) — linesegarray 방출 생략 (#1380)
        None => (String::new(), String::new(), vert_cursor),
    };
    ctx.body_coldef_template_pending = false;
    vert_cursor = first_advance;

    // 치환은 모두 pristine 템플릿의 고정 anchor 에 대해 수행한다 (#1378):
    // linesegarray 치환을 콘텐츠(run 시퀀스) 삽입보다 먼저 두어, 콘텐츠에 포함된
    // 중첩 linesegarray(각주·머리말 등)가 anchor 탐색을 오염시키지 않도록 한다.
    let mut out = replace_first_linesegs(EMPTY_SECTION_XML, &first_linesegs);
    out = replace_page_pr(&out, &section.section_def.page_def);
    // 쪽 테두리/배경 — 템플릿의 하드코딩 borderFillIDRef="1"(테두리 없음)을 IR 값으로
    // 치환한다. 누락 시 문서의 쪽 테두리가 소실되어 외곽 4선 노드가 사라진다(#1388 동형).
    out = replace_page_border_fill(&out, &section.section_def);
    // [#1637] secPr visibility — 템플릿 고정값을 IR 값으로 치환(hideFirstEmptyLine 등 보존).
    out = replace_visibility(&out, &section.section_def);

    // 바탕쪽(masterPage) — secPr 의 masterPageCnt 치환 + secPr 내부 끝에 idRef 참조 삽입.
    // 누락 시 라운드트립에서 바탕쪽 전체(그 안의 그림/표/문단 노드 포함)가 소실된다.
    // id 인덱스는 전 섹션 누적 전역값으로, mod.rs 의 파일 생성 인덱스와 정합한다.
    let master_pages = &section.section_def.master_pages;
    if !master_pages.is_empty() {
        let base: usize = doc.sections[..index]
            .iter()
            .map(|s| s.section_def.master_pages.len())
            .sum();
        let ids: Vec<String> = (0..master_pages.len())
            .map(|k| format!("masterpage{}", base + k))
            .collect();
        out = out.replacen(
            r#"masterPageCnt="0""#,
            &format!(r#"masterPageCnt="{}""#, master_pages.len()),
            1,
        );
        let refs = super::master_page::render_master_page_refs(&ids);
        out = out.replacen("</hp:secPr>", &format!("{refs}</hp:secPr>"), 1);
    }

    // [#1407] 본문 단 정의(colPr) — 첫 문단 IR 의 ColumnDef 를 템플릿 하드코딩
    // colPr(colCount=1)에 치환한다. 본문(depth 0) ColumnDef 는 render_runs 인라인
    // 슬롯에서 제외되므로(#1379) 여기서 받지 않으면 단 정의가 손실된다(2단→1단 →
    // 페이지 넘침). #1388 secPr 여백 치환과 동형.
    if let Some(p) = first_para {
        if let Some(Control::ColumnDef(cd)) = p
            .controls
            .iter()
            .find(|c| matches!(c, Control::ColumnDef(_)))
        {
            out = out.replacen(TEMPLATE_BODY_COL_PR, &render_col_pr_ctrl(cd), 1);
        }
    }

    if let Some(p) = first_para {
        // 첫 문단 `<hp:p>` 태그를 IR 기반 속성으로 교체
        let new_p_tag = render_hp_p_open(p, ctx.next_para_id());
        out = out.replacen(TEMPLATE_FIRST_P_TAG, &new_p_tag, 1);

        // 섹션 첫 run(secPr/colPr 전용)의 charPrIDRef 를 첫 텍스트 run id 와 일치시킨다.
        // 파서는 이 run 시작에서도 (0, id) 를 기록하므로, id 가 다르면 재파싱 시
        // 가짜 (0, 0) entry 가 생긴다 (#1378 stage1 양상 ②).
        let first_cs = first_run_char_shape_id(p);
        if first_cs != 0 {
            let new_secpr_run = format!(r#"<hp:run charPrIDRef="{}"><hp:secPr "#, first_cs);
            out = out.replacen(TEMPLATE_SECPR_RUN_OPEN, &new_secpr_run, 1);
        }

        // 템플릿의 텍스트 run 전체를 문단의 run 시퀀스(다중 run 분할 포함)로 1회 치환.
        out = out.replacen(TEMPLATE_TEXT_RUN, &first_runs, 1);
    }

    // 추가 문단: `</hp:p></hs:sec>` 직전에 `<hp:p>` 요소를 삽입.
    if section.paragraphs.len() > 1 {
        let mut extra = String::new();
        for p in section.paragraphs.iter().skip(1) {
            let (runs, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
            vert_cursor = advance;
            extra.push_str(&render_hp_p_open(p, ctx.next_para_id()));
            extra.push_str(&runs);
            extra.push_str(&linesegs);
            extra.push_str("</hp:p>");
        }
        out = out.replacen(PARA_CLOSE, &format!("</hp:p>{}</hs:sec>", extra), 1);
    }

    Ok(out.into_bytes())
}

/// IR의 Paragraph를 기반으로 `<hp:p>` 시작 태그를 생성.
///
/// `id` 는 문단 순서 기반(0, 1, 2, ...)로 할당한다. 한컴 샘플은 랜덤 해시도 쓰지만
/// 파서는 id 를 무시하므로 순차값으로 충분.
pub(crate) fn render_hp_p_open(p: &Paragraph, id: u32) -> String {
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

/// Paragraph 하나를 (완전한 `<hp:run>` 시퀀스 XML, `<hp:linesegarray>` 요소 XML,
/// 다음 vert_cursor)로 변환.
///
/// `<hp:lineseg>` 출력 원칙 (#177, #1380):
/// - `para.line_segs` 가 비어있지 않으면 **IR 값 그대로** `<hp:linesegarray>` 요소로 출력
/// - 비어있으면 **요소 자체를 방출 생략** (빈 문자열 반환) — 원본에 linesegarray 가
///   없는 문단의 보존 + rhwp 는 lineseg 를 새로 생산하지 않음. 한컴은 열 때 재계산
pub(crate) fn render_paragraph_parts(
    para: &Paragraph,
    vert_start: u32,
    ctx: &mut SerializeContext,
) -> (String, String, u32) {
    let runs_xml = render_runs(para, ctx);

    if !para.line_segs.is_empty() {
        // IR 기반 출력 — 원본 lineseg 값 보존 (#177)
        let linesegs = format!(
            "{}{}{}",
            LINESEG_SLOT_OPEN,
            render_lineseg_array_from_ir(&para.line_segs),
            LINESEG_SLOT_CLOSE
        );
        let vert_end = next_vert_cursor_from_ir(&para.line_segs, vert_start);
        (runs_xml, linesegs, vert_end)
    } else {
        // IR 에 line_segs 없음 — linesegarray 방출 생략 (#1380)
        (runs_xml, String::new(), vert_start)
    }
}

/// `<hp:t>...</hp:t>` 본문 생성 — 탭/소프트브레이크/XML escape 포함.
///
/// `tab_extended`: IR의 탭 확장 정보 목록. `tab_idx`를 통해 탭 문자마다 순서대로 참조.
/// 항목이 없으면 폴백(width=TAB_DEFAULT_WIDTH, leader=0, type=1)을 사용.
pub(crate) fn render_hp_t_content(
    text: &str,
    tab_extended: &[[u16; 7]],
    tab_idx: &mut usize,
) -> String {
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

/// 문단 콘텐츠를 `char_shapes` 경계 기준 다중 `<hp:run>` 으로 분할 출력하는 빌더 (#1378).
///
/// 파서(`src/parser/hwpx/section.rs`)는 각 `<hp:run charPrIDRef>` 시작 위치에서
/// `(utf16_pos, char_shape_id)` 를 기록하고 연속 동일 id 를 dedup 한다.
/// 이 빌더는 그 역방향: `segs[i].0` (i ≥ 1) 위치에서 run 을 닫고 새 run 을 연다.
///
/// 경계 규칙 (구현계획서 1.2):
/// 1. 경계와 슬롯/문자가 같은 위치면 경계 먼저 — 해당 콘텐츠는 새 run 소속 (`cut_before`)
/// 2. 연속 동일 id 경계는 출력 시 skip (IR 은 이미 dedup 상태이나 방어적으로 유지)
/// 3. `char_shapes` 가 비어있으면 단일 run `charPrIDRef="0"`
/// 4. `segs[0].start_pos > 0` 인 비정상 IR 도 첫 run 은 위치 0 부터 시작 (관용 처리)
/// 5. 빈 세그먼트는 `<hp:t></hp:t>` 로 방출 — 재파싱 시 run 시작 entry 위치 보존
struct RunSplitter {
    /// `(start_pos, char_shape_id)` — `[0]` 은 첫 run, `[1..]` 은 cut 경계.
    segs: Vec<(u32, u32)>,
    /// 다음 적용할 경계 인덱스.
    next: usize,
    /// 완성된 run 시퀀스.
    runs: String,
    /// 현재 run 의 내부 콘텐츠 버퍼.
    content: String,
}

impl RunSplitter {
    fn new(para: &Paragraph) -> Self {
        let mut segs: Vec<(u32, u32)> = Vec::new();
        for cs in &para.char_shapes {
            if let Some(&(_, last_id)) = segs.last() {
                if last_id == cs.char_shape_id {
                    continue; // 규칙 2
                }
            }
            segs.push((cs.start_pos, cs.char_shape_id));
        }
        if segs.is_empty() {
            segs.push((0, 0)); // 규칙 3
        }
        Self {
            segs,
            next: 1,
            runs: String::new(),
            content: String::new(),
        }
    }

    /// 경계가 없는 단일 run 문단인지.
    fn single_run(&self) -> bool {
        self.segs.len() == 1
    }

    /// `pos` 위치의 콘텐츠 방출 전에 run 경계 적용이 필요한지 (flush 판단용).
    fn needs_cut(&self, pos: u32) -> bool {
        self.next < self.segs.len() && self.segs[self.next].0 <= pos
    }

    /// `pos` 이하 위치의 경계를 모두 적용 — 규칙 1 (경계 먼저, 콘텐츠는 새 run 소속).
    fn cut_before(&mut self, pos: u32) {
        while self.needs_cut(pos) {
            self.close_run();
            self.next += 1;
        }
    }

    /// 현재 run 을 `<hp:run charPrIDRef>` 로 감싸 완성 목록에 추가.
    fn close_run(&mut self) {
        self.runs.push_str(&format!(
            r#"<hp:run charPrIDRef="{}">"#,
            self.segs[self.next - 1].1
        ));
        if self.content.is_empty() {
            // 규칙 5 — 빈 run 도 <hp:t></hp:t> 로 방출해 재파싱 시 entry 보존
            self.runs.push_str("<hp:t></hp:t>");
        } else {
            self.runs.push_str(&self.content);
            self.content.clear();
        }
        self.runs.push_str("</hp:run>");
    }

    /// 잔여 경계(콘텐츠 끝 이후 시작)를 빈 run 으로 방출하고 마지막 run 을 닫는다.
    fn finish(mut self) -> String {
        while self.next < self.segs.len() {
            self.close_run();
            self.next += 1;
        }
        self.close_run();
        self.runs
    }
}

/// `<hp:ctrl><hp:fieldEnd beginIDRef=".."/></hp:ctrl>` 방출 공통 경로.
fn emit_field_end(out: &mut String, para: &Paragraph, control_idx: usize) {
    if let Some(Control::Field(f)) = para.controls.get(control_idx) {
        if let Ok(xml) = writer_to_string(|w| write_field_end(w, f.field_id)) {
            out.push_str("<hp:ctrl>");
            out.push_str(&xml);
            out.push_str("</hp:ctrl>");
        }
    }
}

/// 고아(다단락) fieldEnd 를 `<hp:ctrl><hp:fieldEnd .../></hp:ctrl>` 로 방출 (Task #1556).
fn emit_orphan_field_end(out: &mut String, ofe: &OrphanFieldEnd) {
    if let Ok(xml) = writer_to_string(|w| write_field_end_full(w, ofe.begin_id_ref, ofe.field_id)) {
        out.push_str("<hp:ctrl>");
        out.push_str(&xml);
        out.push_str("</hp:ctrl>");
    }
}

/// 문단 텍스트 전체를 char_shapes 경계로 분할하며 `splitter` 에 누적한다.
///
/// `char_offsets` 로 문자 idx → UTF-16 위치를 매핑하므로 IR 내 컨트롤(8 유닛 갭)이
/// 있어도 경계 위치가 어긋나지 않는다.
fn split_text_into(splitter: &mut RunSplitter, para: &Paragraph, tab_idx: &mut usize) {
    let mut text_buf = String::new();
    let mut running_pos = 0u32;
    for (idx, c) in para.text.chars().enumerate() {
        let char_pos = para.char_offsets.get(idx).copied().unwrap_or(running_pos);
        if splitter.needs_cut(char_pos) {
            flush_text_fragment(
                &mut splitter.content,
                &mut text_buf,
                &para.tab_extended,
                tab_idx,
            );
            splitter.cut_before(char_pos);
        }
        text_buf.push(c);
        running_pos = char_pos
            .max(running_pos)
            .saturating_add(char_utf16_width(c));
    }
    flush_text_fragment(
        &mut splitter.content,
        &mut text_buf,
        &para.tab_extended,
        tab_idx,
    );
}

/// Paragraph 본문을 완전한 `<hp:run>` 시퀀스로 직렬화한다 (#1378 다중 run 분할).
fn render_runs(para: &Paragraph, ctx: &mut SerializeContext) -> String {
    // ID 참조 무결성 (구현계획서 1.5): 실제 char_shapes entry 만 reference.
    // 빈 IR 의 fallback 0 은 제외 — char_shapes 미등록 문서(`Document::default()`)의
    // 직렬화를 깨지 않도록.
    for cs in &para.char_shapes {
        ctx.char_shape_ids.reference(cs.char_shape_id);
    }

    // [#1592] 완전 빈 문단(원본에 <hp:run> 없음)은 run 을 방출하지 않는다. char_shapes=[]
    // 인 문단에 RunSplitter 가 기본 (0,0) 세그먼트로 charPrIDRef="0" 빈 run 을 추가하면,
    // 재파싱 시 spurious (0,0) char_shape 가 생긴다(원본은 run 없어 char_shapes=[]).
    // char_shapes 가 있으면(예: [(0,0)] 명시) 종전대로 run 을 방출한다(linesegarray 는 별도).
    if para.text.is_empty()
        && para.char_shapes.is_empty()
        && para.controls.is_empty()
        && para.field_ranges.is_empty()
        && para.orphan_field_ends.is_empty()
    {
        return String::new();
    }

    let mut splitter = RunSplitter::new(para);

    let slot_count = inferred_control_slot_count(para);
    // slots 와 각 slot 의 para.controls 인덱스(slot_ctrl_indices)를 병행 수집 —
    // [Task #1627] empty-text 문단의 bookmark in-order 방출에서 slot 사이 위치 계산에 사용.
    let (slots, slot_ctrl_indices): (Vec<&Control>, Vec<usize>) =
        if slot_count == para.controls.len() {
            // 전 컨트롤이 위치 슬롯인 경로. 본문 첫 문단의 첫 ColumnDef 도 슬롯으로 남겨
            // char-offset 정합을 보존하고, 그 XML 만 render_control_slot 의 consume-once
            // 플래그로 억제한다(템플릿이 이미 방출 — 중복 방지). 2번째+ 는 인라인 방출.
            (
                para.controls.iter().collect(),
                (0..para.controls.len()).collect(),
            )
        } else {
            // [Task #1379] 셀·글상자 subList(depth>0) 경로에서는 ColumnDef 도 인라인 슬롯으로
            // 취급한다 (원본 XML 에 <hp:ctrl><hp:colPr/></hp:ctrl> 인라인 존재).
            // [Task #1584] 본문(depth 0) 경로에서도 ColumnDef 를 인라인 슬롯에 포함하되,
            // 첫 문단의 첫 ColumnDef(섹션 템플릿 흡수분)는 슬롯에서 기본 제외한다.
            // 2번째+ 본문 ColumnDef 는 포함하여 드롭을 방지한다.
            let suppress_first_col = ctx.sub_list_depth == 0 && ctx.body_coldef_template_pending;
            let mut col_seen = 0u32;
            let mut s: Vec<&Control> = Vec::new();
            let mut si: Vec<usize> = Vec::new();
            // 제외된 hidden 슬롯 후보(SectionDef·템플릿 흡수 첫 ColumnDef)의 인덱스.
            let mut hidden: Vec<usize> = Vec::new();
            for (i, c) in para.controls.iter().enumerate() {
                let keep = if matches!(c, Control::ColumnDef(_)) {
                    col_seen += 1;
                    if suppress_first_col && col_seen == 1 {
                        hidden.push(i);
                        false
                    } else {
                        true
                    }
                } else if matches!(c, Control::SectionDef(_)) {
                    hidden.push(i);
                    false
                } else {
                    is_hwpx_inline_slot(c)
                };
                if keep {
                    s.push(c);
                    si.push(i);
                }
            }
            // [Task #1591 v2] hidden 슬롯 정합: cc 증거(slot_count)가 hidden 후보
            // (secPr/템플릿 흡수 colPr — 원본 XML 에서 첫 run 이 점유하는 8유닛 슬롯)의
            // 점유를 보여주면 위치 슬롯으로 편입한다. XML 은 방출되지 않고
            // (SectionDef: 방출 arm 없음, 첫 ColumnDef: consume-once 억제) 위치 축만
            // 8유닛씩 전진 — 첫 문단이 mismatch 폴백 대신 메인 경로(위치 정확)로
            // 진입해 후위 슬롯(pageNum 등)·fieldEnd 가 char-offset 위치에 방출된다
            // (Class C1 +8 시프트·C2 fieldEnd 드롭의 근원 교정). 증거 불일치(합성 IR,
            // hidden 이 cc 를 점유하지 않는 문서)는 종전 동작 그대로.
            if !hidden.is_empty() && slot_count == s.len() + hidden.len() {
                let mut s2: Vec<&Control> = Vec::new();
                let mut si2: Vec<usize> = Vec::new();
                let mut keep_iter = si.iter().peekable();
                for (i, c) in para.controls.iter().enumerate() {
                    let kept = keep_iter.peek() == Some(&&i);
                    if kept {
                        keep_iter.next();
                    }
                    if kept || hidden.contains(&i) {
                        s2.push(c);
                        si2.push(i);
                    }
                }
                // 첫 ColumnDef 를 슬롯으로 되살렸으므로 consume-once 플래그를 유지해
                // render_control_slot 이 XML 방출만 1회 건너뛰게 한다(첫 분기와 동형).
                (s2, si2)
            } else {
                if suppress_first_col {
                    // 템플릿 흡수분을 슬롯에서 제외한 채 진행하므로, render_control_slot 의
                    // consume-once 억제가 2번째 ColumnDef 를 잘못 건너뛰지 않도록 플래그 해제.
                    ctx.body_coldef_template_pending = false;
                }
                (s, si)
            }
        };

    // Bookmark는 zero-width 라 slot char-position 축에 안 잡힌다.
    // - [Task #1627] empty-text(객체-only) 문단: 문단 시작 강제는 원본 컨트롤 순서(예:
    //   Table·PageNumberPos 뒤의 Bookmark)를 깨고 roundtrip char_shape 오프셋을 shift시킨다
    //   → para.controls 순서대로 slot 사이에 in-order 방출.
    // - [Task #1591 v2] 슬롯이 있는 비-empty 문단도 in-order 로 통일(1라운드 hoist 제거
    //   편입 — 표/pageNum 뒤 북마크의 원본 순서 보존).
    // - 비-empty·무슬롯 문단: 종전대로 문단 시작(첫 run)에 배치(순서 증거 없음, 종전 유지).
    let bm_inorder = para.text.is_empty() || !slots.is_empty();
    if !bm_inorder {
        for ctrl in &para.controls {
            if let Control::Bookmark(bm) = ctrl {
                if let Ok(xml) = writer_to_string(|w| write_bookmark(w, bm)) {
                    splitter.content.push_str("<hp:ctrl>");
                    splitter.content.push_str(&xml);
                    splitter.content.push_str("</hp:ctrl>");
                }
            }
        }
    }

    let mut tab_idx = 0usize;

    // fast path: 슬롯·필드·고아 fieldEnd·경계 없음 — 텍스트 전체를 단일 run 으로
    if slots.is_empty()
        && para.field_ranges.is_empty()
        && para.orphan_field_ends.is_empty()
        && splitter.single_run()
    {
        let t = render_hp_t_content(&para.text, &para.tab_extended, &mut tab_idx);
        splitter.content.push_str(&t);
        return splitter.finish();
    }

    // mismatch 경로: 슬롯 위치 추정 불가 — 텍스트(경계 분할 포함) 후 슬롯 일괄 방출
    if slot_count != slots.len() {
        split_text_into(&mut splitter, para, &mut tab_idx);
        let mut bm_done = vec![false; para.controls.len()];
        for (si, slot) in slots.iter().enumerate() {
            // [Task #1627] empty-text 문단: 이 slot 앞(controls 순서)의 bookmark 를 먼저 방출.
            if bm_inorder {
                emit_inorder_bookmarks(
                    &mut splitter.content,
                    para,
                    &mut bm_done,
                    slot_ctrl_indices[si],
                );
            }
            render_control_slot(&mut splitter.content, slot, ctx);
        }
        if bm_inorder {
            // 마지막 slot 뒤에 오는 trailing bookmark.
            emit_inorder_bookmarks(&mut splitter.content, para, &mut bm_done, usize::MAX);
        }
        // [Task #1591 v2] 균형 field_ranges 의 닫는 fieldEnd 도 말미 복원 — 이 경로는
        // fieldBegin(슬롯)만 방출하고 fieldEnd 방출 코드가 없어 same-para 균형 필드가
        // 1/0 으로 깨지며 cc −8 (#1593). #1556 고아 복원과 동형(위치 대신 말미 일괄).
        for fr in &para.field_ranges {
            emit_field_end(&mut splitter.content, para, fr.control_idx);
        }
        // [Task #1556] 위치 추정 불가 경로에서도 고아 fieldEnd 의 8유닛 슬롯은 복원한다
        // (정확한 위치 대신 말미 일괄 — 최소한 char_count 보존).
        for ofe in &para.orphan_field_ends {
            emit_orphan_field_end(&mut splitter.content, ofe);
        }
        return splitter.finish();
    }

    // 메인 경로 — UTF-16 위치 축 위에서 슬롯/필드/문자/경계를 함께 처리
    let mut text_buf = String::new();
    let mut slot_idx = 0usize;
    let mut expected_utf16_pos = 0u32;
    let mut field_end_emitted = vec![false; para.field_ranges.len()];
    // [Task #1556] 고아 fieldEnd 방출 추적.
    let mut orphan_emitted = vec![false; para.orphan_field_ends.len()];
    let text_char_count = para.text.chars().count();

    // [Task #1591 v2] 메인 경로 bookmark in-order 방출 추적 (mismatch 경로와 동일 기제).
    // 메인 경로는 종전엔 hoist 전담이라 방출 지점이 없었다 — 첫 문단(hidden 슬롯 정합)이
    // 메인 경로로 진입하면서 슬롯 사이 in-order 방출이 필요해졌다.
    let mut bm_done = vec![false; para.controls.len()];

    // 빈 문단(text == "")의 0-length 필드: 메인 루프가 실행되지 않아
    // pre-char 검사를 통과하지 못하므로 루프 전에 slots → fieldEnd 순으로 방출한다.
    if para.text.is_empty() {
        while slot_idx < slots.len() {
            splitter.cut_before(expected_utf16_pos);
            if bm_inorder {
                emit_inorder_bookmarks(
                    &mut splitter.content,
                    para,
                    &mut bm_done,
                    slot_ctrl_indices[slot_idx],
                );
            }
            render_control_slot(&mut splitter.content, slots[slot_idx], ctx);
            slot_idx += 1;
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
        }
        for (i, fr) in para.field_ranges.iter().enumerate() {
            if fr.start_char_idx == fr.end_char_idx && !field_end_emitted[i] {
                splitter.cut_before(expected_utf16_pos);
                emit_field_end(&mut splitter.content, para, fr.control_idx);
                expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                field_end_emitted[i] = true;
            }
        }
        // [Task #1556] 빈 문단의 고아 fieldEnd (char_idx == 0).
        for (i, ofe) in para.orphan_field_ends.iter().enumerate() {
            if ofe.char_idx == 0 && !orphan_emitted[i] {
                splitter.cut_before(expected_utf16_pos);
                emit_orphan_field_end(&mut splitter.content, ofe);
                expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                orphan_emitted[i] = true;
            }
        }
    }

    for (idx, c) in para.text.chars().enumerate() {
        let char_pos = para
            .char_offsets
            .get(idx)
            .copied()
            .unwrap_or(expected_utf16_pos);
        // [#1407] 이 idx 위치에서 닫혀야 할(미방출) fieldEnd 가 있으면, 그 8유닛 갭은
        // 슬롯이 아니라 fieldEnd 소유다. 슬롯 방출을 양보해 텍스트-끝 슬롯(newNum 등)이
        // fieldEnd 자리를 가로채지 못하게 한다 (0-length 필드는 아래 pre-char 경로가 처리).
        while slot_idx < slots.len() && char_pos >= expected_utf16_pos.saturating_add(8) {
            flush_text_fragment(
                &mut splitter.content,
                &mut text_buf,
                &para.tab_extended,
                &mut tab_idx,
            );
            // 슬롯 시작 위치의 경계 — 슬롯은 새 run 소속 (규칙 1)
            splitter.cut_before(expected_utf16_pos);
            if bm_inorder {
                emit_inorder_bookmarks(
                    &mut splitter.content,
                    para,
                    &mut bm_done,
                    slot_ctrl_indices[slot_idx],
                );
            }
            render_control_slot(&mut splitter.content, slots[slot_idx], ctx);
            slot_idx += 1;
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
        }

        // [Task #1556] 고아 fieldEnd (char_idx == idx): 문자 push 전에 8유닛 슬롯 방출.
        for (i, ofe) in para.orphan_field_ends.iter().enumerate() {
            if ofe.char_idx == idx && !orphan_emitted[i] {
                flush_text_fragment(
                    &mut splitter.content,
                    &mut text_buf,
                    &para.tab_extended,
                    &mut tab_idx,
                );
                splitter.cut_before(expected_utf16_pos);
                emit_orphan_field_end(&mut splitter.content, ofe);
                expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                orphan_emitted[i] = true;
            }
        }

        // 0-length 필드(start == end == idx): fieldBegin 방출 직후, 문자 push 전에 fieldEnd 방출.
        // post-char 검사(next_idx 기준)는 end-1 번째 문자 처리 후 방출하므로 0-length 필드에서
        // fieldEnd가 fieldBegin 앞에 나오거나 텍스트 뒤로 밀리는 문제가 생긴다.
        for (i, fr) in para.field_ranges.iter().enumerate() {
            if fr.start_char_idx == fr.end_char_idx
                && fr.end_char_idx == idx
                && !field_end_emitted[i]
            {
                flush_text_fragment(
                    &mut splitter.content,
                    &mut text_buf,
                    &para.tab_extended,
                    &mut tab_idx,
                );
                splitter.cut_before(expected_utf16_pos);
                emit_field_end(&mut splitter.content, para, fr.control_idx);
                field_end_emitted[i] = true;
            }
        }

        // AutoNumber 슬롯 (#1382): placeholder 공백이 슬롯 8유닛의 첫 유닛을 점유
        // (HWP5/HWPX 파서 공통 규약 — placeholder at P, 다음 문자 P+8). 슬롯 위치
        // (char_pos == expected)의 placeholder 에서 ctrl 을 방출하고, placeholder 는
        // 파서가 IR 에 합성한 문자이므로 텍스트로 내보내지 않는다 (한컴 원본 XML 동형:
        // `<hp:ctrl><hp:autoNum/></hp:ctrl>` 뒤에 실제 텍스트만 이어진다).
        // placeholder 판별: 일반 공백과 구분하기 위해 직후 offset 의 +8 jump 를 본다
        // (마지막 문자면 jump 가 char_offsets 에 없으므로 placeholder 로 간주).
        let is_autonum_placeholder = c == ' '
            && char_pos == expected_utf16_pos
            && para
                .char_offsets
                .get(idx + 1)
                .copied()
                .unwrap_or_else(|| char_pos.saturating_add(8))
                >= char_pos.saturating_add(8);
        if slot_idx < slots.len()
            && matches!(slots[slot_idx], Control::AutoNumber(_))
            && is_autonum_placeholder
        {
            flush_text_fragment(
                &mut splitter.content,
                &mut text_buf,
                &para.tab_extended,
                &mut tab_idx,
            );
            splitter.cut_before(expected_utf16_pos);
            render_control_slot(&mut splitter.content, slots[slot_idx], ctx);
            slot_idx += 1;
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
            continue;
        }

        // 문자 위치의 경계 — 문자는 새 run 소속 (규칙 1)
        if splitter.needs_cut(char_pos) {
            flush_text_fragment(
                &mut splitter.content,
                &mut text_buf,
                &para.tab_extended,
                &mut tab_idx,
            );
            splitter.cut_before(char_pos);
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
                flush_text_fragment(
                    &mut splitter.content,
                    &mut text_buf,
                    &para.tab_extended,
                    &mut tab_idx,
                );
                splitter.cut_before(expected_utf16_pos);
                emit_field_end(&mut splitter.content, para, fr.control_idx);
                // [#1407] fieldEnd 는 8유닛 슬롯을 소비한다. expected 를 +8 진행하지
                // 않으면 다음 idx 에서 텍스트-끝 슬롯(newNum 등)이 이 8유닛 갭을
                // 가로채 텍스트가 +8 밀린다 (143E 문단 0.14: char_offsets[3] 27→35).
                expected_utf16_pos = expected_utf16_pos.saturating_add(8);
                field_end_emitted[i] = true;
            }
        }
    }

    flush_text_fragment(
        &mut splitter.content,
        &mut text_buf,
        &para.tab_extended,
        &mut tab_idx,
    );

    // end_char_idx >= text.len() 인 경우 루프에서 감지되지 않으므로 루프 후에 처리
    for (i, fr) in para.field_ranges.iter().enumerate() {
        if !field_end_emitted[i] {
            splitter.cut_before(expected_utf16_pos);
            emit_field_end(&mut splitter.content, para, fr.control_idx);
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
        }
    }

    // [Task #1556] 텍스트 끝(char_idx == text_char_count) 의 고아 fieldEnd — para 0.16 케이스.
    for (i, ofe) in para.orphan_field_ends.iter().enumerate() {
        if !orphan_emitted[i] {
            debug_assert!(
                ofe.char_idx >= text_char_count,
                "미방출 고아 fieldEnd 는 텍스트 끝이어야 함"
            );
            splitter.cut_before(expected_utf16_pos);
            emit_orphan_field_end(&mut splitter.content, ofe);
            expected_utf16_pos = expected_utf16_pos.saturating_add(8);
            orphan_emitted[i] = true;
        }
    }

    while slot_idx < slots.len() {
        splitter.cut_before(expected_utf16_pos);
        if bm_inorder {
            emit_inorder_bookmarks(
                &mut splitter.content,
                para,
                &mut bm_done,
                slot_ctrl_indices[slot_idx],
            );
        }
        render_control_slot(&mut splitter.content, slots[slot_idx], ctx);
        slot_idx += 1;
        expected_utf16_pos = expected_utf16_pos.saturating_add(8);
    }
    if bm_inorder {
        // 마지막 슬롯 뒤(컨트롤 순서 후미)의 trailing bookmark.
        emit_inorder_bookmarks(&mut splitter.content, para, &mut bm_done, usize::MAX);
    }

    splitter.finish()
}

fn inferred_control_slot_count(para: &Paragraph) -> usize {
    // [#1382] AutoNumber 는 placeholder 공백이 슬롯 8유닛의 첫 유닛을 점유한다
    // (HWP5 body_text / HWPX offsets 조립 공통 규약). placeholder 가 텍스트 폭에
    // 이미 1유닛으로 집계되므로 두 축 모두에서 autoNum 1개당 잉여가 8이 아닌 7로
    // 측정된다 → autoNum 수만큼 더해 보정한 뒤 8로 나눈다. placeholder 없는
    // 합성 IR(편집기 생성)은 잉여 0이라 보정해도 0 — 기존 mismatch 경로 유지.
    let autonum_count = para
        .controls
        .iter()
        .filter(|c| matches!(c, Control::AutoNumber(_)))
        .count() as u32;

    let text_units: u32 = para.text.chars().map(char_utf16_width).sum();
    let from_char_count = para
        .char_count
        .saturating_sub(1)
        .saturating_sub(text_units)
        .saturating_add(autonum_count)
        / 8;

    let mut offsets_gap = 0u32;
    let mut expected = 0u32;
    for (idx, c) in para.text.chars().enumerate() {
        let pos = para.char_offsets.get(idx).copied().unwrap_or(expected);
        if pos > expected {
            offsets_gap += pos - expected;
        }
        expected = pos.max(expected).saturating_add(char_utf16_width(c));
    }
    let from_offsets = offsets_gap.saturating_add(autonum_count) / 8;

    // fieldEnd는 8 code unit 슬롯이지만 para.controls[]에 대응 컨트롤이 없다.
    // field_ranges.len()이 fieldEnd 수와 정확히 일치하므로 빼서 보정한다.
    // [Task #1556] 고아(다단락) fieldEnd 도 컨트롤 없는 8유닛 슬롯이므로 동일하게 차감.
    from_char_count
        .max(from_offsets)
        .saturating_sub(para.field_ranges.len() as u32)
        .saturating_sub(para.orphan_field_ends.len() as u32) as usize
}

pub(crate) fn is_hwpx_inline_slot(control: &Control) -> bool {
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

/// [Task #1627] empty-text 문단에서 bookmark 를 para.controls 순서대로 in-order 방출한다.
/// `upto_ctrl_idx` 미만 인덱스의 미방출 bookmark 만 방출(usize::MAX = 나머지 전부).
/// bookmark 는 zero-width 라 char-position 을 점유하지 않으므로 slot 사이에 끼워도 위치 불변.
fn emit_inorder_bookmarks(
    content: &mut String,
    para: &Paragraph,
    bm_done: &mut [bool],
    upto_ctrl_idx: usize,
) {
    for (i, ctrl) in para.controls.iter().enumerate() {
        if i >= upto_ctrl_idx {
            break;
        }
        if bm_done[i] {
            continue;
        }
        if let Control::Bookmark(bm) = ctrl {
            if let Ok(xml) = writer_to_string(|w| write_bookmark(w, bm)) {
                content.push_str("<hp:ctrl>");
                content.push_str(&xml);
                content.push_str("</hp:ctrl>");
            }
            bm_done[i] = true;
        }
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
            out.push_str("<hp:ctrl>");
            let generated_params = generated_field_parameters(f);
            let has_params = f.raw_parameters_xml.is_some() || generated_params.is_some();
            let has_memo = f.field_type == crate::model::control::FieldType::Memo
                && !f.memo_paragraphs.is_empty();
            if has_params || has_memo {
                // [#1391] 자식(parameters / memo subList)이 있으면 start/end 태그.
                out.push_str(&super::field::field_begin_open_tag(f));
                out.push('>');
                if let Some(params) = f
                    .raw_parameters_xml
                    .as_deref()
                    .or(generated_params.as_deref())
                {
                    out.push_str(params);
                }
                if has_memo {
                    out.push_str(SUB_LIST_OPEN);
                    let mut vert_cursor: u32 = 0;
                    for para in &f.memo_paragraphs {
                        ctx.para_shape_ids.reference(para.para_shape_id);
                        ctx.style_ids.reference(para.style_id as u16);
                        let (runs, linesegs, advance) =
                            render_paragraph_parts(para, vert_cursor, ctx);
                        vert_cursor = advance;
                        out.push_str(&render_hp_p_open(para, ctx.next_para_id()));
                        out.push_str(&runs);
                        out.push_str(&linesegs);
                        out.push_str("</hp:p>");
                    }
                    out.push_str("</hp:subList>");
                }
                out.push_str("</hp:fieldBegin>");
            } else {
                match writer_to_string(|w| write_field_begin(w, f)) {
                    Ok(xml) => out.push_str(&xml),
                    Err(e) => eprintln!("[hwpx] Field 직렬화 실패: {e}"),
                }
            }
            out.push_str("</hp:ctrl>");
        }
        Control::PageHide(ph) => out.push_str(&render_page_hiding(ph)),
        Control::PageNumberPos(pn) => out.push_str(&render_page_num(pn)),
        Control::NewNumber(nn) => out.push_str(&render_new_num(nn)),
        Control::Header(h) => out.push_str(&render_header(h, ctx)),
        Control::Footer(f) => out.push_str(&render_footer(f, ctx)),
        Control::AutoNumber(an) => out.push_str(&render_autonum(an)),
        Control::Form(form) => match writer_to_string(|w| super::form::write_form(w, form)) {
            // 폼은 <hp:run> 직접 자식 (Table/Picture와 동일, <hp:ctrl> 비포장)
            Ok(xml) => out.push_str(&xml),
            Err(e) => eprintln!("[hwpx] Form 직렬화 실패: {e}"),
        },
        Control::CharOverlap(co) => out.push_str(&render_compose(co)),
        // [Task #1587] 덧말(Ruby) 인라인 방출. is_hwpx_inline_slot 에 등록돼 슬롯 위치는
        // 자동이나 종전 방출 arm 부재로 드롭됐다. parse_dutmal 의 역매핑.
        Control::Ruby(r) => out.push_str(&render_dutmal(r)),
        // [Task #1379/#1584] 인라인 colPr 방출.
        // - subList(depth>0): 전부 인라인 방출(원본 XML 인라인 존재).
        // - 본문(depth 0): 첫 문단의 첫 ColumnDef 1개는 섹션 템플릿 colPr 앵커가 이미
        //   방출했으므로(중복 방지) consume-once 플래그로 XML 만 건너뛴다. 슬롯 자체는
        //   상위(render_runs)에서 유지하므로 char-offset 위치 정합은 보존된다.
        Control::ColumnDef(cd) => {
            if ctx.sub_list_depth == 0 && ctx.body_coldef_template_pending {
                ctx.body_coldef_template_pending = false;
            } else {
                out.push_str(&render_col_pr_ctrl(cd));
            }
        }
        _ => {}
    }
}

/// 덧말(Ruby) `<hp:dutmal>` 직렬화 (#1587). `parse_dutmal` 의 역매핑.
/// 속성 순서는 한컴 실측(posType szRatio option styleIDRef align)을 따른다.
fn render_dutmal(r: &Ruby) -> String {
    let pos_type = match r.pos_type {
        1 => "BOTTOM",
        _ => "TOP",
    };
    let align = match r.align {
        1 => "RIGHT",
        2 => "CENTER",
        _ => "LEFT",
    };
    format!(
        r#"<hp:dutmal posType="{}" szRatio="{}" option="{}" styleIDRef="{}" align="{}"><hp:mainText>{}</hp:mainText><hp:subText>{}</hp:subText></hp:dutmal>"#,
        pos_type,
        r.sz_ratio,
        r.option,
        r.style_id_ref,
        align,
        xml_escape(&r.main_text),
        xml_escape(&r.ruby_text),
    )
}

fn generated_field_parameters(field: &Field) -> Option<String> {
    if field.command.is_empty() || field.raw_parameters_xml.is_some() {
        return None;
    }
    Some(format!(
        r#"<hp:parameters cnt="1" name=""><hp:stringParam name="Command">{}</hp:stringParam></hp:parameters>"#,
        xml_escape(&field.command),
    ))
}

/// 셀·글상자 subList 인라인 `<hp:ctrl><hp:colPr .../></hp:ctrl>` (#1379 3단계).
/// `parse_col_pr` / `parse_col_line`(parser/hwpx/section.rs)의 역매핑.
fn render_col_pr_ctrl(cd: &ColumnDef) -> String {
    let col_type = match cd.column_type {
        ColumnType::Distribute => "BalancedNewspaper",
        ColumnType::Parallel => "Parallel",
        ColumnType::Normal => "NEWSPAPER",
    };
    let layout = match cd.direction {
        ColumnDirection::RightToLeft => "RIGHT",
        ColumnDirection::LeftToRight => "LEFT",
    };
    let mut out = format!(
        r#"<hp:ctrl><hp:colPr id="" type="{}" layout="{}" colCount="{}" sameSz="{}" sameGap="{}""#,
        col_type, layout, cd.column_count, cd.same_width as u8, cd.spacing,
    );
    if cd.separator_type != 0 {
        out.push('>');
        out.push_str(&format!(
            r#"<hp:colLine type="{}" width="{} mm" color="{}"/>"#,
            col_line_type_str(cd.separator_type),
            line_width_mm(cd.separator_width),
            super::shape::color_to_hex(cd.separator_color),
        ));
        out.push_str("</hp:colPr></hp:ctrl>");
    } else {
        out.push_str("/></hp:ctrl>");
    }
    out
}

/// `parse_hwpx_line_type` 역매핑 (colLine type).
fn col_line_type_str(t: u8) -> &'static str {
    match t {
        2 => "DASH",
        3 => "DOT",
        4 => "DASH_DOT",
        5 => "DASH_DOT_DOT",
        6 => "LONG_DASH",
        7 => "CIRCLE",
        _ => "SOLID",
    }
}

/// HWP 선 굵기 인덱스 → mm 수치 문자열 (`parse_hwpx_line_width` 역매핑).
fn line_width_mm(w: u8) -> &'static str {
    match w {
        0 => "0.1",
        1 => "0.12",
        2 => "0.15",
        3 => "0.2",
        4 => "0.25",
        5 => "0.3",
        6 => "0.4",
        7 => "0.5",
        8 => "0.6",
        9 => "0.7",
        10 => "1.0",
        11 => "1.5",
        12 => "2.0",
        13 => "3.0",
        14 => "4.0",
        _ => "5.0",
    }
}

/// `<hp:compose>` 글자겹침(CharOverlap) — `<hp:run>` 직접 자식 (<hp:ctrl> 비포장).
/// `parse_compose`(parser/hwpx/section.rs)의 역매핑. charPr 목록은 미설정(u32::MAX)
/// 항목 포함 원본 그대로 방출한다 (ID 참조 등록 비대상).
fn render_compose(co: &CharOverlap) -> String {
    let circle_type = match co.border_type {
        1 => "SHAPE_CIRCLE",
        2 => "SHAPE_REVERSAL_CIRCLE",
        3 => "SHAPE_RECTANGLE",
        4 => "SHAPE_REVERSAL_RECTANGLE",
        5 => "SHAPE_TRIANGLE",
        6 => "SHAPE_REVERSAL_TIRANGLE",
        _ => "CHAR",
    };
    let compose_type = if co.expansion == 1 {
        "OVERLAP"
    } else {
        "SPREAD"
    };
    let text: String = co.chars.iter().collect();
    let mut out = format!(
        r#"<hp:compose circleType="{}" charSz="{}" composeType="{}" charPrCnt="{}" composeText="{}">"#,
        circle_type,
        co.inner_char_size,
        compose_type,
        co.char_shape_ids.len(),
        xml_escape(&text),
    );
    for id in &co.char_shape_ids {
        out.push_str(&format!(r#"<hp:charPr prIDRef="{}"/>"#, id));
    }
    out.push_str("</hp:compose>");
    out
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
            r#"<hp:ctrl><hp:{tag} id="{id}" applyPageType="{apply}">"#,
            r#"<hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" "#,
            r#"linkListIDRef="0" linkListNextIDRef="0" textWidth="{tw}" textHeight="{th}" "#,
            r#"hasTextRef="{tr}" hasNumRef="{nr}">"#
        ),
        tag = tag,
        id = h.id,
        apply = apply_page_type_to_str(h.apply_to),
        tw = h.text_width,
        th = h.text_height,
        tr = h.text_ref,
        nr = h.num_ref,
    );
    let mut vert_cursor: u32 = 0;
    for p in h.paragraphs.iter() {
        let (runs, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
        vert_cursor = advance;
        out.push_str(&render_hp_p_open(p, ctx.next_para_id()));
        out.push_str(&runs);
        out.push_str(&linesegs);
        out.push_str("</hp:p>");
    }
    out.push_str(&format!("</hp:subList></hp:{tag}></hp:ctrl>", tag = tag));
    out
}

/// render_header_footer 공통 인자 묶음 (Header/Footer가 동일 필드를 가짐).
struct HeaderFooterFields<'a> {
    id: u32,
    apply_to: HeaderFooterApply,
    text_width: u32,
    text_height: u32,
    text_ref: u8,
    num_ref: u8,
    paragraphs: &'a [Paragraph],
}

fn hwpx_header_footer_id(raw_ctrl_extra: &[u8]) -> u32 {
    raw_ctrl_extra
        .get(..4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .unwrap_or(0)
}

fn render_header(h: &Header, ctx: &mut SerializeContext) -> String {
    render_header_footer(
        "header",
        HeaderFooterFields {
            id: hwpx_header_footer_id(&h.raw_ctrl_extra),
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
            id: hwpx_header_footer_id(&f.raw_ctrl_extra),
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

/// 번호 종류 → HWPX `numType` 문자열. `parse_num_type`의 역매핑.
///
/// [#1387] Picture 는 "PICTURE" — 한컴 실물 표기. 종전 "FIGURE" 는 한컴 생산 파일에
/// 비실재(samples/hwpx 전수 0건)하고 한컴에디터가 그림 번호로 인식하지 못해 캡션
/// 번호가 미출력됐다 (한컴 판정 실증). 파서는 FIGURE/PICTURE 양쪽 수용 유지.
fn auto_number_type_to_str(t: AutoNumberType) -> &'static str {
    match t {
        AutoNumberType::Page => "PAGE",
        AutoNumberType::Footnote => "FOOTNOTE",
        AutoNumberType::Endnote => "ENDNOTE",
        AutoNumberType::Picture => "PICTURE",
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

fn render_shape(shape: &ShapeObject, ctx: &mut SerializeContext) -> String {
    // Rectangle: Writer-based serializer (drawText 포함)
    if let ShapeObject::Rectangle(r) = shape {
        return match writer_to_string(|w| super::shape::write_rect(w, r, ctx)) {
            Ok(xml) => xml,
            Err(e) => {
                eprintln!("[hwpx] Shape::Rectangle 직렬화 실패: {e}");
                String::new()
            }
        };
    }
    // Line: Writer-based serializer
    if let ShapeObject::Line(l) = shape {
        return match writer_to_string(|w| super::shape::write_line(w, l, ctx)) {
            Ok(xml) => xml,
            Err(e) => {
                eprintln!("[hwpx] Shape::Line 직렬화 실패: {e}");
                String::new()
            }
        };
    }
    if let ShapeObject::Group(g) = shape {
        let mut xml = match writer_to_string(|w| {
            super::shape::write_container_open(w, &g.common, &g.shape_attr)
        }) {
            Ok(xml) => xml,
            Err(e) => {
                eprintln!("[hwpx] Shape::Group 직렬화 실패: {e}");
                String::new()
            }
        };
        for child in &g.children {
            xml.push_str(&render_shape(child, ctx));
        }
        // 캡션 (#1403) 은 자식 도형 뒤에 방출 — 한컴 실물(aift.hwpx) 순서
        // 설명(#1392)은 캡션 직후 (write_container_close 내부)
        match writer_to_string(|w| {
            super::shape::write_container_close(w, g.caption.as_ref(), &g.common, ctx)
        }) {
            Ok(close) => xml.push_str(&close),
            Err(e) => eprintln!("[hwpx] Shape::Group 닫기 실패: {e}"),
        }
        return xml;
    }
    const NO_PTS: &[crate::model::Point] = &[];
    let (tag, c, caption, drawing, points): (
        _,
        _,
        _,
        Option<&crate::model::shape::DrawingObjAttr>,
        &[crate::model::Point],
    ) = match shape {
        ShapeObject::Rectangle(_) | ShapeObject::Line(_) => unreachable!(),
        ShapeObject::Ellipse(e) => (
            "ellipse",
            &e.common,
            &e.drawing.caption,
            Some(&e.drawing),
            NO_PTS,
        ),
        ShapeObject::Arc(a) => (
            "arc",
            &a.common,
            &a.drawing.caption,
            Some(&a.drawing),
            NO_PTS,
        ),
        ShapeObject::Polygon(p) => (
            "polygon",
            &p.common,
            &p.drawing.caption,
            Some(&p.drawing),
            &p.points,
        ),
        ShapeObject::Curve(cv) => (
            "curve",
            &cv.common,
            &cv.drawing.caption,
            Some(&cv.drawing),
            &cv.points,
        ),
        ShapeObject::Group(_) => unreachable!(),
        ShapeObject::Picture(pic) => {
            return match writer_to_string(|w| picture::write_picture(w, pic, ctx)) {
                Ok(xml) => xml,
                Err(e) => {
                    eprintln!("[hwpx] Shape::Picture 직렬화 실패: {e}");
                    String::new()
                }
            };
        }
        ShapeObject::Chart(ch) => ("chart", &ch.common, &ch.caption, None, NO_PTS),
        ShapeObject::Ole(o) => {
            return match writer_to_string(|w| super::shape::write_ole(w, o, ctx)) {
                Ok(xml) => xml,
                Err(e) => {
                    eprintln!("[hwpx] Shape::Ole 직렬화 실패: {e}");
                    String::new()
                }
            };
        }
    };
    // [Task #1598] ellipse / arc 전용 지오메트리(center/축/시작끝점) — 미방출 시 한글이
    // 타원/호를 다르게 렌더 → 누적 레이아웃 변동 → 페이지 붕괴(#1589 잔여). hc:pt(polygon/
    // curve) 와 상호배타이므로 동일 위치(shadow 직후, sz 직전)에 방출.
    let hc = |t: &str, p: &crate::model::Point| format!(r#"<hc:{t} x="{}" y="{}"/>"#, p.x, p.y);
    let geom_tail = match shape {
        ShapeObject::Ellipse(e) => format!(
            "{}{}{}{}{}{}{}",
            hc("center", &e.center),
            hc("ax1", &e.axis1),
            hc("ax2", &e.axis2),
            hc("start1", &e.start1),
            hc("end1", &e.end1),
            hc("start2", &e.start2),
            hc("end2", &e.end2),
        ),
        ShapeObject::Arc(a) => format!(
            "{}{}{}",
            hc("center", &a.center),
            hc("ax1", &a.axis1),
            hc("ax2", &a.axis2),
        ),
        _ => String::new(),
    };
    render_common_shape_xml(tag, c, caption, drawing, points, &geom_tail, ctx)
}

fn render_common_shape_xml(
    tag: &str,
    c: &CommonObjAttr,
    caption: &Option<crate::model::shape::Caption>,
    drawing: Option<&crate::model::shape::DrawingObjAttr>,
    points: &[crate::model::Point],
    geom_tail: &str,
    ctx: &mut SerializeContext,
) -> String {
    // 도형 좌표계 블록(offset/orgSz/curSz/flip/rotationInfo/renderingInfo) — 누락 시
    // 회전/뒤집힘이 소실되어 bbox 가 전치되는 등 렌더가 어긋난다(#1501 동류, polygon 등).
    let shape_block = drawing
        .map(|d| {
            writer_to_string(|w| super::shape::write_shape_component_block(w, &d.shape_attr))
                .unwrap_or_default()
        })
        .unwrap_or_default();
    // [#1596] 지오메트리(lineShape/fillBrush/shadow/꼭짓점) — 종전 드롭으로 도형 형상 소실 →
    // 페이지 붕괴(#1589 잔여). write_rect 와 동형 순서(shape_block 직후, sz 직전).
    let geometry = drawing
        .map(|d| {
            let ls = writer_to_string(|w| super::shape::write_line_shape(w, &d.border_line))
                .unwrap_or_default();
            let fb = writer_to_string(|w| super::shape::write_fill_brush(w, &d.fill, ctx))
                .unwrap_or_default();
            let sh = writer_to_string(|w| super::shape::write_shadow(w, d)).unwrap_or_default();
            let pts: String = points
                .iter()
                .map(|p| format!(r#"<hc:pt x="{}" y="{}"/>"#, p.x, p.y))
                .collect();
            // [#1598] ellipse/arc 전용 지오메트리(center/축/시작끝점)는 hc:pt 와 상호배타.
            format!("{ls}{fb}{sh}{pts}{geom_tail}")
        })
        .unwrap_or_default();
    // 태그 부수 속성 — numberingType/dropcapstyle/href/groupLevel/instid (rect/line 동형).
    let group_level = drawing.map(|d| d.shape_attr.group_level).unwrap_or(0);
    let instid = drawing
        .map(|d| d.inst_id)
        .filter(|&i| i != 0)
        .unwrap_or(c.instance_id);
    let mut out = format!(
        concat!(
            r#"<hp:{tag} id="{id}" zOrder="{zo}" numberingType="{nt}" textWrap="{tw}" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" href="" groupLevel="{gl}" instid="{iid}">"#,
            "{block}",
            "{geometry}",
            r#"<hp:sz width="{w}" height="{h}" widthRelTo="ABSOLUTE" heightRelTo="ABSOLUTE"/>"#,
            r#"<hp:pos treatAsChar="{tac}" affectLSpacing="0" flowWithText="{fwt}" allowOverlap="{ao}" holdAnchorAndSO="{hold}" vertRelTo="{vr}" vertAlign="{va}" horzRelTo="{hr}" horzAlign="{ha}" vertOffset="{vo}" horzOffset="{ho}"/>"#,
            r#"<hp:outMargin left="{ml}" right="{mr}" top="{mt}" bottom="{mb}"/>"#,
        ),
        tag = tag,
        block = shape_block,
        geometry = geometry,
        id = c.instance_id,
        zo = c.z_order,
        nt = super::shape::numbering_type_str(c.numbering_type),
        gl = group_level,
        iid = instid,
        tw = text_wrap_to_hwpx(c.text_wrap),
        tac = if c.treat_as_char { "1" } else { "0" },
        fwt = if c.flow_with_text { "1" } else { "0" },
        ao = if c.allow_overlap { "1" } else { "0" },
        hold = if c.prevent_page_break != 0 { "1" } else { "0" },
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
    );
    // 캡션 (#1403) — HWP5 파서는 모든 도형의 캡션을 적재하므로(parser/control/shape.rs)
    // legacy 경로(ellipse/arc/polygon/curve/chart/ole)도 방출해야 소실되지 않는다.
    if let Some(cap) = caption {
        match writer_to_string(|w| super::table::write_caption(w, cap, ctx)) {
            Ok(xml) => out.push_str(&xml),
            Err(e) => eprintln!("[hwpx] Shape({tag}) 캡션 직렬화 실패: {e}"),
        }
    }
    // 설명 (#1451) — caption 직후 (OWPML AbstractShapeObjectType: outMargin→caption→shapeComment).
    // picture.rs:104 선례와 동일 순서. legacy 경로(ellipse/arc/polygon/curve/chart/ole) 보존.
    // 빈 description 미방출은 write_shape_comment 내부 가드로 보장된다.
    match writer_to_string(|w| super::shape::write_shape_comment(w, c)) {
        Ok(xml) => out.push_str(&xml),
        Err(e) => eprintln!("[hwpx] Shape({tag}) shapeComment 직렬화 실패: {e}"),
    }
    out.push_str(&format!("</hp:{tag}>"));
    out
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
        let (runs, linesegs, advance) = render_paragraph_parts(p, vert_cursor, ctx);
        vert_cursor = advance;
        out.push_str(&render_hp_p_open(p, ctx.next_para_id()));
        out.push_str(&runs);
        out.push_str(&linesegs);
        out.push_str("</hp:p>");
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

    // 설명 (#1392) — outMargin 직후, 빈 description 은 미방출
    let shape_comment = if c.description.is_empty() {
        String::new()
    } else {
        format!(
            "<hp:shapeComment>{}</hp:shapeComment>",
            xml_escape(&c.description)
        )
    };

    // [#1594] holdAnchorAndSO 는 IR(prevent_page_break)을 보존(종전 "0" 하드코딩 제거).
    let hold = if c.prevent_page_break != 0 { "1" } else { "0" };

    format!(
        r#"<hp:equation id="{id}" zOrder="{z_order}" numberingType="EQUATION" textWrap="{}" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" instid="{id}" version="{version}" baseLine="{baseline}" textColor="{text_color}" baseUnit="{base_unit}" font="{font}"><hp:script>{script}</hp:script><hp:sz width="{width}" widthRelTo="ABSOLUTE" height="{height}" heightRelTo="ABSOLUTE"/><hp:pos treatAsChar="{treat}" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="{hold}" vertRelTo="{}" horzRelTo="{}" vertAlign="{}" horzAlign="{}" vertOffset="{vert_offset}" horzOffset="{horz_offset}"/><hp:outMargin left="{margin_left}" right="{margin_right}" top="{margin_top}" bottom="{margin_bottom}"/>{shape_comment}</hp:equation>"#,
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

fn flush_buf(t_xml: &mut String, buf: &mut String) {
    if !buf.is_empty() {
        t_xml.push_str(&xml_escape(buf));
        buf.clear();
    }
}

/// 템플릿의 첫 `<hp:linesegarray>...</hp:linesegarray>` **요소 전체**를
/// `new_element` 로 치환한다. `new_element` 가 빈 문자열이면 요소가 제거된다
/// (#1380 — line_segs 빈 문단의 linesegarray 방출 생략).
fn replace_first_linesegs(xml: &str, new_element: &str) -> String {
    let open = xml
        .find(LINESEG_SLOT_OPEN)
        .expect("template has linesegarray");
    let close_rel = xml[open..]
        .find(LINESEG_SLOT_CLOSE)
        .expect("template has closing linesegarray");
    let elem_end = open + close_rel + LINESEG_SLOT_CLOSE.len();
    let mut out = String::with_capacity(xml.len() + new_element.len());
    out.push_str(&xml[..open]);
    out.push_str(new_element);
    out.push_str(&xml[elem_end..]);
    out
}

/// [#1166] 템플릿 pagePr 의 고정 용지 속성(landscape/width/height)을 IR page_def
/// 값으로 치환한다. 종전엔 템플릿 하드코딩값(landscape="WIDELY" width=59528
/// height=84186)이 그대로 출력되어 HWPX 저장 시 가로/세로 + 용지 크기가 손실됐다.
///
/// OWPML landscape: WIDELY=세로(landscape=false), NARROWLY=가로(landscape=true).
/// width/height 는 짧은변/긴변 그대로 (HWP 바이너리 동일 규약).
///
/// [#1388] 확장: gutterType(제본 방향) + `<hp:margin>` 여백 7필드를 IR 값으로 치환.
/// 종전엔 템플릿 고정 여백(left/right=8504 등)이 그대로 출력되어 원본 여백이
/// 변형됐다 (samples/hwpx 전수 51/74 섹션 영향, 본문 +56.7px 시프트·페이지 수 변동).
/// gutterType ↔ binding 매핑은 parser/hwpx/section.rs `parse_page_pr` 의 역매핑.
fn replace_page_pr(xml: &str, page_def: &crate::model::page::PageDef) -> String {
    // 템플릿의 pagePr 여는 태그(고정 문자열) → IR 기반으로 교체.
    const TEMPLATE_PAGE_PR: &str =
        r#"<hp:pagePr landscape="WIDELY" width="59528" height="84186" gutterType="LEFT_ONLY">"#;
    let landscape = if page_def.landscape {
        "NARROWLY"
    } else {
        "WIDELY"
    };
    let gutter_type = match page_def.binding {
        crate::model::page::BindingMethod::SingleSided => "LEFT_ONLY",
        crate::model::page::BindingMethod::DuplexSided => "LEFT_RIGHT",
        crate::model::page::BindingMethod::TopFlip => "TOP_BOTTOM",
    };
    let new_page_pr = format!(
        r#"<hp:pagePr landscape="{}" width="{}" height="{}" gutterType="{}">"#,
        landscape, page_def.width, page_def.height, gutter_type,
    );
    let out = if xml.contains(TEMPLATE_PAGE_PR) {
        xml.replacen(TEMPLATE_PAGE_PR, &new_page_pr, 1)
    } else {
        // 템플릿이 변경됐거나 이미 치환된 경우 — 원본 유지(회귀 방지).
        xml.to_string()
    };

    // 템플릿의 hp:margin(고정 문자열) → IR 여백 7필드로 교체 (#1388).
    // write_section 은 콘텐츠 삽입 전에 호출하므로 템플릿 내 유일 1회 등장이 보장된다.
    const TEMPLATE_PAGE_MARGIN: &str = r#"<hp:margin header="4252" footer="4252" gutter="0" left="8504" right="8504" top="5668" bottom="4252"/>"#;
    let new_margin = format!(
        r#"<hp:margin header="{}" footer="{}" gutter="{}" left="{}" right="{}" top="{}" bottom="{}"/>"#,
        page_def.margin_header,
        page_def.margin_footer,
        page_def.margin_gutter,
        page_def.margin_left,
        page_def.margin_right,
        page_def.margin_top,
        page_def.margin_bottom,
    );
    if out.contains(TEMPLATE_PAGE_MARGIN) {
        out.replacen(TEMPLATE_PAGE_MARGIN, &new_margin, 1)
    } else {
        // 템플릿이 변경됐거나 이미 치환된 경우 — 원본 유지(회귀 방지).
        out
    }
}

/// 템플릿의 3개 `pageBorderFill`(BOTH/EVEN/ODD, 하드코딩 borderFillIDRef="1") 을 IR 값으로
/// 치환한다. 누락 시 문서의 실제 쪽 테두리가 소실된다. 파서는 첫 항목(BOTH)을
/// `page_border_fill`, 나머지(EVEN/ODD)를 `extra_page_border_fills` 에 위치 기반 저장한다.
fn replace_page_border_fill(xml: &str, sec_def: &crate::model::document::SectionDef) -> String {
    let entries: [(&str, &crate::model::page::PageBorderFill); 3] = [
        ("BOTH", &sec_def.page_border_fill),
        (
            "EVEN",
            sec_def
                .extra_page_border_fills
                .first()
                .unwrap_or(&sec_def.page_border_fill),
        ),
        (
            "ODD",
            sec_def
                .extra_page_border_fills
                .get(1)
                .unwrap_or(&sec_def.page_border_fill),
        ),
    ];
    let mut out = xml.to_string();
    for (ty, pbf) in entries {
        // 템플릿 고정 문자열 (empty_section0.xml 와 정확히 일치해야 함).
        let template = format!(
            r#"<hp:pageBorderFill type="{ty}" borderFillIDRef="1" textBorder="PAPER" headerInside="0" footerInside="0" fillArea="PAPER"><hp:offset left="1417" right="1417" top="1417" bottom="1417"/></hp:pageBorderFill>"#
        );
        if out.contains(&template) {
            out = out.replacen(&template, &render_page_border_fill(ty, pbf), 1);
        }
        // 미일치 시 원본 유지(회귀 방지) — replace_page_pr 패턴과 동형.
    }
    out
}

/// 단일 `pageBorderFill` 요소를 IR 에서 재구성한다. attr 비트 → textBorder/fillArea/
/// headerInside/footerInside (parser `page_border_fill_attr` 의 역).
fn render_page_border_fill(ty: &str, pbf: &crate::model::page::PageBorderFill) -> String {
    let text_border = if pbf.attr & 0x0000_0001 != 0 {
        "PAPER"
    } else {
        "CONTENT"
    };
    let fill_area = if pbf.attr & 0x0000_0008 != 0 {
        "PAGE"
    } else if pbf.attr & 0x0000_0010 != 0 {
        "BORDER"
    } else {
        "PAPER"
    };
    let header_inside = if pbf.attr & 0x0000_0002 != 0 {
        "1"
    } else {
        "0"
    };
    let footer_inside = if pbf.attr & 0x0000_0004 != 0 {
        "1"
    } else {
        "0"
    };
    format!(
        r#"<hp:pageBorderFill type="{ty}" borderFillIDRef="{}" textBorder="{}" headerInside="{}" footerInside="{}" fillArea="{}"><hp:offset left="{}" right="{}" top="{}" bottom="{}"/></hp:pageBorderFill>"#,
        pbf.border_fill_id,
        text_border,
        header_inside,
        footer_inside,
        fill_area,
        pbf.spacing_left,
        pbf.spacing_right,
        pbf.spacing_top,
        pbf.spacing_bottom,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::{CharShapeRef, Paragraph};

    /// [Task #1627] empty-text(객체-only) 문단에서 bookmark 는 문단 시작으로 끌려가지 않고
    /// para.controls 순서대로 in-order 방출되어야 한다(원본 컨트롤 순서 보존).
    #[test]
    fn task1627_empty_para_bookmark_serialized_after_preceding_table() {
        use crate::model::control::{Bookmark, Control};

        let mut para = Paragraph::default();
        // empty text, controls 순서 = [Table, Bookmark]
        para.controls = vec![
            Control::Table(Box::default()),
            Control::Bookmark(Bookmark {
                name: "BM_AFTER_TBL".to_string(),
            }),
        ];
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        let tbl_pos = xml.find("<hp:tbl");
        let bm_pos = xml.find("BM_AFTER_TBL");
        assert!(
            tbl_pos.is_some(),
            "table 직렬화 필요: {}",
            &xml[..400.min(xml.len())]
        );
        assert!(bm_pos.is_some(), "bookmark 직렬화 필요");
        assert!(
            tbl_pos < bm_pos,
            "bookmark 가 table 뒤(원본 순서)에 와야 함 — 문단 시작 강제 회귀 (#1627)"
        );
    }

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
    fn secpr_emits_tab_stop_val_and_unit() {
        // [Finding 14] 원본 secPr 의 tabStopVal="4000" tabStopUnit="HWPUNIT" (한컴
        // 기본 탭 폭 상수) 이 직렬화에서 누락되지 않아야 한다. 순서는
        // tabStop → tabStopVal → tabStopUnit → outlineShapeIDRef.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(
                r#"tabStop="8000" tabStopVal="4000" tabStopUnit="HWPUNIT" outlineShapeIDRef="1""#
            ),
            "secPr 에 tabStopVal/tabStopUnit 이 정확 순서로 있어야 함: {}",
            &xml[..600.min(xml.len())]
        );
    }

    #[test]
    fn section_root_declares_hwpunitchar_namespace() {
        // [Finding 15] hs:sec 루트의 xmlns:hwpunitchar 선언(원본에 존재, 미사용
        // 이지만 무손실 위해 보존)이 누락되지 않아야 한다.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"xmlns:hwpunitchar="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar""#),
            "hs:sec 에 xmlns:hwpunitchar 선언이 있어야 함"
        );
    }

    #[test]
    fn task1407_body_col_pr_reflects_ir_column_def() {
        // [#1407] 본문 첫 문단 IR 에 2단 ColumnDef 가 있으면 템플릿 하드코딩
        // colPr(colCount=1)이 IR 값(colCount=2)으로 치환돼야 한다. 미치환 시
        // 2단→1단 손실로 페이지 넘침(143E RT 1→2).
        use crate::model::page::{ColumnDef, ColumnType};
        let mut cd = ColumnDef::default();
        cd.column_type = ColumnType::Normal;
        cd.column_count = 2;
        cd.same_width = true;
        cd.spacing = 2268;

        let mut para = Paragraph::default();
        para.text = "x".to_string();
        para.controls.push(Control::ColumnDef(cd));

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"colCount="2""#) && xml.contains(r#"sameGap="2268""#),
            "본문 colPr 이 IR 2단 정의로 치환돼야 함: {}",
            &xml[..900.min(xml.len())]
        );
        assert!(
            !xml.contains(r#"colCount="1""#),
            "하드코딩 colCount=1 이 남으면 안 됨 (회귀 가드): {}",
            &xml[..900.min(xml.len())]
        );
    }

    #[test]
    fn task1407_single_column_doc_unaffected() {
        // ColumnDef IR 이 없는 문단(단일 단)은 템플릿 colCount=1 유지 — 회귀 없음.
        let mut para = Paragraph::default();
        para.text = "x".to_string();
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"colCount="1""#),
            "ColumnDef 없으면 템플릿 colCount=1 유지"
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

    // ---------- #1382: autoNum placeholder 슬롯 ----------

    /// autoNum IR 규약 문단: 텍스트 "가·placeholder·나", offsets [0,1,9].
    fn autonum_para() -> Paragraph {
        let mut para = Paragraph::default();
        para.text = "가 나".to_string(); // 가(0) + placeholder 공백(1) + 나(9)
        para.char_offsets = vec![0, 1, 9];
        para.char_count = 11; // offsets 축 총 10 + 끝 마커 1
        para.controls
            .push(crate::model::control::Control::AutoNumber(
                crate::model::control::AutoNumber {
                    number_type: crate::model::control::AutoNumberType::Footnote,
                    ..Default::default()
                },
            ));
        para
    }

    #[test]
    fn task1382_inference_counts_autonum_placeholder_slot() {
        // placeholder 가 8유닛 중 1유닛을 점유해 잉여가 7로 측정되는 패턴 —
        // autoNum 보정으로 슬롯 1개로 집계되어야 한다 (종전 0 → mismatch 경로).
        assert_eq!(inferred_control_slot_count(&autonum_para()), 1);
    }

    #[test]
    fn task1382_autonum_slot_emitted_at_placeholder() {
        // ctrl 이 placeholder 원위치(mid-text)에 방출되고 placeholder 는 텍스트로
        // 내보내지 않는다 (한컴 원본 XML 동형).
        let (doc, section) = make_doc_with_paragraph(autonum_para());
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains("<hp:t>가</hp:t><hp:ctrl><hp:autoNum"),
            "ctrl 은 placeholder 위치(가 직후)에 방출: {}",
            xml
        );
        assert!(
            xml.contains("</hp:ctrl><hp:t>나</hp:t>"),
            "placeholder 공백은 텍스트로 미방출, 후속 텍스트만 이어짐: {}",
            xml
        );
        assert!(
            !xml.contains("<hp:t>가 나</hp:t>"),
            "종전 결함(슬롯 끝 방출 + placeholder 텍스트 이중 방출) 재발 금지"
        );
    }

    #[test]
    fn task1382_synthetic_autonum_without_placeholder_keeps_legacy_path() {
        // 합성 IR(placeholder/offset 없는 편집기 생성 문단)은 보정 후에도 추론 0 —
        // 기존 mismatch 경로(끝 방출) 유지로 회귀 없음.
        let mut para = Paragraph::default();
        para.text = "가나".to_string();
        para.char_offsets = vec![0, 1];
        para.char_count = 3;
        para.controls
            .push(crate::model::control::Control::AutoNumber(
                crate::model::control::AutoNumber::default(),
            ));
        assert_eq!(inferred_control_slot_count(&para), 0);
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains("<hp:t>가나</hp:t><hp:ctrl><hp:autoNum"),
            "합성 IR 은 텍스트 후 끝 방출(기존 동작): {}",
            xml
        );
    }

    // ---------- #1388: replace_page_pr — secPr 페이지 여백 원본 보존 ----------

    #[test]
    fn task1388_page_margin_reflects_page_def() {
        let mut para = Paragraph::default();
        para.text = "m".to_string();
        let (doc, mut section) = make_doc_with_paragraph(para);
        let pd = &mut section.section_def.page_def;
        pd.width = 59527;
        pd.height = 84189;
        pd.margin_header = 4251;
        pd.margin_footer = 4251;
        pd.margin_gutter = 0;
        pd.margin_left = 7086;
        pd.margin_right = 14173;
        pd.margin_top = 4251;
        pd.margin_bottom = 4251;
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(
            xml.contains(
                r#"<hp:margin header="4251" footer="4251" gutter="0" left="7086" right="14173" top="4251" bottom="4251"/>"#
            ),
            "hp:margin must reflect IR PageDef 7 fields (온새미로 sec0 실측값)"
        );
        assert!(
            !xml.contains(r#"left="8504""#),
            "template margin value must not survive"
        );
        // #1166 동적화 회귀 없음 — width/height 동반 검증.
        assert!(xml.contains(r#"width="59527" height="84189""#));
    }

    #[test]
    fn task1388_gutter_type_reflects_binding() {
        use crate::model::page::BindingMethod;
        for (binding, expected) in [
            (BindingMethod::SingleSided, r#"gutterType="LEFT_ONLY""#),
            (BindingMethod::DuplexSided, r#"gutterType="LEFT_RIGHT""#),
            (BindingMethod::TopFlip, r#"gutterType="TOP_BOTTOM""#),
        ] {
            let mut para = Paragraph::default();
            para.text = "g".to_string();
            let (doc, mut section) = make_doc_with_paragraph(para);
            section.section_def.page_def.binding = binding;
            let mut ctx = SerializeContext::collect_from_document(&doc);
            let bytes = write_section(&section, &doc, 0, &mut ctx).unwrap();
            let xml = std::str::from_utf8(&bytes).unwrap();
            assert!(
                xml.contains(expected),
                "binding={:?} must emit {}",
                binding,
                expected
            );
        }
    }

    #[test]
    fn task1388_template_mismatch_keeps_original() {
        // 템플릿 anchor 가 없는 입력 — 원본 유지(silent no-op, 회귀 방지 정책).
        let mut pd = crate::model::page::PageDef::default();
        pd.margin_left = 1234;
        let xml = r#"<hp:pagePr landscape="WIDELY" width="1" height="2" gutterType="LEFT_ONLY"><hp:margin header="0" footer="0" gutter="0" left="9" right="9" top="9" bottom="9"/></hp:pagePr>"#;
        assert_eq!(replace_page_pr(xml, &pd), xml);
    }

    #[test]
    fn task1388_template_anchor_present_in_template() {
        // 템플릿 변경 시 silent no-op 으로 빠지지 않도록 anchor 존재를 직접 보장한다.
        let pd = crate::model::page::PageDef {
            margin_header: 1,
            margin_footer: 2,
            margin_gutter: 3,
            margin_left: 4,
            margin_right: 5,
            margin_top: 6,
            margin_bottom: 7,
            ..Default::default()
        };
        let out = replace_page_pr(EMPTY_SECTION_XML, &pd);
        assert!(
            out.contains(
                r#"<hp:margin header="1" footer="2" gutter="3" left="4" right="5" top="6" bottom="7"/>"#
            ),
            "EMPTY_SECTION_XML 의 margin anchor 치환이 실패 — 템플릿이 변경됐는지 확인"
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
    fn task1380_linesegarray_omitted_when_ir_empty() {
        // IR 의 line_segs 가 비어있으면 linesegarray 요소 자체를 방출 생략 (#1380).
        // 종전 fallback(vertsize=1000 합성)은 원본 무 → RT 유 비대칭을 만들었다.
        let mut para = Paragraph::default();
        para.text = "a\nb".to_string(); // 텍스트가 있어도 IR 에 lineseg 없으면 생략
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            !xml.contains("<hp:linesegarray"),
            "empty line_segs must omit linesegarray entirely: {}",
            xml
        );
        assert!(!xml.contains("<hp:lineseg "));
    }

    #[test]
    fn task1380_empty_section_omits_linesegarray() {
        // 문단이 없는 섹션(비파싱 IR)도 템플릿의 linesegarray 가 제거되어야 함 (#1380).
        let doc = crate::model::document::Document::default();
        let section = crate::model::document::Section::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(!xml.contains("<hp:linesegarray"));
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

    // ---------- #1556: 고아(다단락) fieldEnd 방출 ----------

    #[test]
    fn task1556_orphan_field_end_emitted_at_text_end() {
        // 다단락 필드의 end 문단(para 0.16 동형): 텍스트 "끝." 뒤에 고아 fieldEnd 8유닛.
        use crate::model::paragraph::{CharShapeRef, OrphanFieldEnd};
        let mut para = Paragraph::default();
        para.text = "끝.".to_string();
        para.char_offsets = vec![0, 1];
        para.char_count = 11; // 텍스트 2 + fieldEnd 8 + 끝마커 1
        para.char_shapes = vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 3,
            },
            CharShapeRef {
                start_pos: 10,
                char_shape_id: 30,
            },
        ];
        para.orphan_field_ends = vec![OrphanFieldEnd {
            char_idx: 2,
            begin_id_ref: 1_878_228_493,
            field_id: 627_272_811,
        }];
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"<hp:fieldEnd beginIDRef="1878228493" fieldid="627272811"/>"#),
            "고아 fieldEnd 가 attrs 와 함께 방출되어야 함: {xml}"
        );
        // 텍스트가 fieldEnd 보다 앞에 나온다 (run 말미 fieldEnd 패턴).
        let t_pos = xml.find("끝.").expect("텍스트");
        let fe_pos = xml.find("<hp:fieldEnd").expect("fieldEnd");
        assert!(t_pos < fe_pos, "텍스트가 fieldEnd 앞: {xml}");
    }

    #[test]
    fn task1556_multipara_field_parse_serialize_parse_roundtrip() {
        // 합성 다단락 필드(begin=문단0, end=문단1) → parse → serialize → re-parse.
        // end 문단의 char_count/text/char_offsets/orphan 이 보존되어야 한다 (IR diff=0).
        use crate::parser::hwpx::section::parse_hwpx_section;
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0"><hp:ctrl><hp:fieldBegin id="1878228493" type="CLICK_HERE" name="본문" fieldid="627272811"/></hp:ctrl><hp:t>본문</hp:t></hp:run>
  </hp:p>
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="3"><hp:t>끝.</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="1878228493" fieldid="627272811"/></hp:ctrl></hp:run>
    <hp:run charPrIDRef="30"><hp:t/></hp:run>
  </hp:p>
</hs:sec>"#;
        let sec1 = parse_hwpx_section(xml).unwrap();
        let mut doc = Document::default();
        doc.sections.push(sec1.clone());
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_section(&sec1, &doc, 0, &mut ctx).unwrap();
        let xml2 = String::from_utf8(bytes).unwrap();
        let sec2 = parse_hwpx_section(&xml2).unwrap();

        // 두 번째 문단(고아 fieldEnd 보유) IR 보존.
        let a = &sec1.paragraphs[1];
        let b = &sec2.paragraphs[1];
        assert_eq!(b.text, a.text, "text 보존");
        assert_eq!(
            b.char_count, a.char_count,
            "char_count 보존 (8유닛 소실 없음)"
        );
        assert_eq!(b.char_offsets, a.char_offsets, "char_offsets 보존");
        assert_eq!(
            b.char_shapes
                .iter()
                .map(|c| (c.start_pos, c.char_shape_id))
                .collect::<Vec<_>>(),
            a.char_shapes
                .iter()
                .map(|c| (c.start_pos, c.char_shape_id))
                .collect::<Vec<_>>(),
            "char_shape 경계 보존"
        );
        assert_eq!(b.orphan_field_ends.len(), 1, "고아 fieldEnd 재파싱 보존");
        assert_eq!(b.orphan_field_ends[0].begin_id_ref, 1_878_228_493);
    }

    #[test]
    fn task1556_orphan_field_end_zero_fieldid_omits_attr() {
        use crate::model::paragraph::OrphanFieldEnd;
        let mut para = Paragraph::default();
        para.text = "a".to_string();
        para.char_offsets = vec![0];
        para.char_count = 10; // 1 + 8 + 1
        para.orphan_field_ends = vec![OrphanFieldEnd {
            char_idx: 1,
            begin_id_ref: 42,
            field_id: 0,
        }];
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"<hp:fieldEnd beginIDRef="42"/>"#),
            "field_id 0 이면 fieldid 속성 생략: {xml}"
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
    fn task1391_memo_field_emits_parameters_and_sublist() {
        // MEMO 필드: parameters verbatim + subList 본문 방출, start/end 태그.
        let mut f = Field::default();
        f.field_type = FieldType::Memo;
        f.field_id = 7;
        f.raw_parameters_xml =
            Some(r#"<hp:parameters cnt="1" name=""><hp:stringParam name="ID">memo1</hp:stringParam></hp:parameters>"#.to_string());
        let mut memo_para = Paragraph::default();
        memo_para.text = "메모 본문".to_string();
        f.memo_paragraphs.push(memo_para);

        let mut para = Paragraph::default();
        para.text = "x".to_string();
        para.char_count = 18; // fieldBegin(8) + x(1) + fieldEnd(8) + end(1)
        para.char_offsets = vec![8];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 1,
            control_idx: 0,
        });

        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();

        assert!(
            xml.contains(r#"<hp:stringParam name="ID">memo1</hp:stringParam>"#),
            "parameters verbatim 방출: {xml}"
        );
        assert!(xml.contains("<hp:t>메모 본문</hp:t>"), "메모 본문 방출");
        assert!(
            xml.contains("</hp:fieldBegin>"),
            "자식 있으면 start/end 태그 (자기닫힘 금지)"
        );
        // 순서: parameters → subList
        let pp = xml.find("<hp:parameters").unwrap();
        let sl = xml.find("<hp:subList").unwrap();
        assert!(pp < sl, "parameters 가 subList 보다 먼저");
    }

    #[test]
    fn task1391_field_without_params_keeps_empty_tag() {
        // parameters/memo 없는 필드는 기존 empty_tag 자기닫힘 유지 (회귀 방지).
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 5;
        let mut para = Paragraph::default();
        para.text = "y".to_string();
        para.char_count = 18;
        para.char_offsets = vec![8];
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 1,
            control_idx: 0,
        });
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            !xml.contains("</hp:fieldBegin>"),
            "자식 없으면 자기닫힘 유지: {xml}"
        );
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
            xml.contains(r#"<hp:ctrl><hp:fieldBegin id="99" type="CLICK_HERE""#),
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

    // ---------- #1407: newNum(텍스트 끝 슬롯)이 fieldEnd 갭을 가로채지 않음 ----------

    #[test]
    fn task1407_field_end_not_stolen_by_newnum_slot() {
        // 143E 문단 0.14 모델: 하이퍼링크 필드가 "ABC"를 래핑하고 그 뒤 " DEF" 텍스트,
        // newNum 은 텍스트 끝. char_offsets 는 head(secPr) 16유닛 뒤로 시작:
        // fieldBegin(8) "ABC"(3) fieldEnd(8) " DEF"(4) newNum(8) end(1).
        // 핵심: fieldEnd(텍스트 중간 갭)가 후속 슬롯(newNum)에 가로채이면 " DEF"가
        // +8 밀린다. render_runs 방출 XML 의 컨트롤·텍스트 순서로 봉인.
        let mut f = Field::default();
        f.field_type = FieldType::Hyperlink;
        f.field_id = 42;

        let mut nn = NewNumber::default();
        nn.number = 2;
        nn.number_type = AutoNumberType::Page;

        let mut para = Paragraph::default();
        para.text = "ABC DEF".to_string();
        para.char_count = 8 + 3 + 8 + 4 + 8 + 1;
        para.char_offsets = vec![8, 9, 10, 19, 20, 21, 22];
        para.controls.push(Control::Field(f));
        para.controls.push(Control::NewNumber(nn));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 3, // "ABC" 래핑
            control_idx: 0,
        });

        let xml = runs_of(&para);

        let begin_pos = xml.find("fieldBegin").expect("fieldBegin");
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        let newnum_pos = xml.find("newNum").expect("newNum");
        // " DEF" 텍스트(<hp:t> DEF</hp:t>) 위치 — fieldEnd 닫힘 뒤의 'DEF'.
        let after_end = end_pos + xml[end_pos..].find('>').unwrap();
        let def_pos = xml[after_end..]
            .find("DEF")
            .map(|p| p + after_end)
            .expect("DEF");

        assert!(begin_pos < end_pos, "fieldBegin이 fieldEnd보다 앞: {xml}");
        assert!(
            end_pos < def_pos,
            "fieldEnd 가 ' DEF' 텍스트보다 앞 (갭이 가로채이지 않음): {xml}"
        );
        assert!(
            def_pos < newnum_pos,
            "newNum 은 텍스트 끝 — ' DEF' 뒤에 와야 한다 (#1407 회귀 가드): {xml}"
        );
    }

    // ---------- #1378: char_shapes 경계 기준 다중 run 분할 ----------

    fn cs(start_pos: u32, char_shape_id: u32) -> CharShapeRef {
        CharShapeRef {
            start_pos,
            char_shape_id,
        }
    }

    fn runs_of(para: &Paragraph) -> String {
        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);
        render_runs(para, &mut ctx)
    }

    #[test]
    fn task1378_two_run_split_mid_text() {
        // 경계가 텍스트 중간 — 텍스트 분할 (경계 케이스 2)
        let mut para = Paragraph::default();
        para.text = "abcdef".to_string();
        para.char_shapes = vec![cs(0, 1), cs(3, 2)];
        assert_eq!(
            runs_of(&para),
            r#"<hp:run charPrIDRef="1"><hp:t>abc</hp:t></hp:run><hp:run charPrIDRef="2"><hp:t>def</hp:t></hp:run>"#
        );
    }

    #[test]
    fn task1378_boundary_at_slot_position_slot_in_new_run() {
        // 경계와 컨트롤 슬롯이 같은 위치 — 경계 먼저 cut, 컨트롤은 새 run 소속 (경계 케이스 1)
        // 스트림: "ab"(0..2) + equation 슬롯(2..10) + "cd"(10..12), 경계 pos=2
        let mut para = Paragraph::default();
        para.text = "abcd".to_string();
        para.char_offsets = vec![0, 1, 10, 11];
        para.char_count = 13; // 4 + 8(슬롯) + 1
        para.controls.push(Control::Equation(Box::default()));
        para.char_shapes = vec![cs(0, 1), cs(2, 2)];
        let xml = runs_of(&para);
        assert!(
            xml.starts_with(r#"<hp:run charPrIDRef="1"><hp:t>ab</hp:t></hp:run><hp:run charPrIDRef="2"><hp:equation"#),
            "슬롯 위치의 경계는 슬롯보다 먼저 적용돼야 한다: {}",
            &xml[..200.min(xml.len())]
        );
        let run2 = xml.find(r#"<hp:run charPrIDRef="2">"#).unwrap();
        let cd = xml.find("<hp:t>cd</hp:t>").expect("cd 텍스트");
        assert!(run2 < cd, "cd 는 새 run 소속이어야 한다");
    }

    #[test]
    fn task1378_boundary_between_slot_and_text() {
        // 슬롯 이후 텍스트 중간 경계 — 슬롯은 이전 run, 분할은 텍스트에서
        // 스트림: equation 슬롯(0..8) + "ab"(8..10) + "cd"(10..12), 경계 pos=10
        let mut para = Paragraph::default();
        para.text = "abcd".to_string();
        para.char_offsets = vec![8, 9, 10, 11];
        para.char_count = 13;
        para.controls.push(Control::Equation(Box::default()));
        para.char_shapes = vec![cs(0, 1), cs(10, 2)];
        let xml = runs_of(&para);
        let run1_end = xml.find("</hp:run>").unwrap();
        let eq = xml.find("<hp:equation").expect("equation");
        let ab = xml.find("<hp:t>ab</hp:t>").expect("ab");
        assert!(
            eq < run1_end && ab < run1_end,
            "equation 과 ab 는 첫 run 소속"
        );
        assert!(
            xml.ends_with(r#"<hp:run charPrIDRef="2"><hp:t>cd</hp:t></hp:run>"#),
            "cd 만 새 run 으로 분할: {}",
            xml
        );
    }

    #[test]
    fn task1378_consecutive_same_id_boundary_skipped() {
        // 연속 동일 id 경계 skip — 파서 dedup 왕복 정합 (경계 케이스 3)
        let mut para = Paragraph::default();
        para.text = "abcdef".to_string();
        para.char_shapes = vec![cs(0, 5), cs(2, 5), cs(4, 6)];
        let xml = runs_of(&para);
        assert_eq!(
            xml,
            r#"<hp:run charPrIDRef="5"><hp:t>abcd</hp:t></hp:run><hp:run charPrIDRef="6"><hp:t>ef</hp:t></hp:run>"#,
            "동일 id 경계 (2,5) 는 skip 되고 (4,6) 만 분할해야 한다"
        );
    }

    #[test]
    fn task1378_tab_and_linebreak_in_split_runs() {
        // 탭(8 유닛)/lineBreak 를 포함한 run 분할 (경계 케이스 4)
        // 위치: a=0, \t=1..9, b=9, \n=10, c=11 — 경계 pos=9
        let mut para = Paragraph::default();
        para.text = "a\tb\nc".to_string();
        para.char_shapes = vec![cs(0, 1), cs(9, 2)];
        let xml = runs_of(&para);
        assert_eq!(
            xml,
            concat!(
                r#"<hp:run charPrIDRef="1"><hp:t>a<hp:tab width="4000" leader="0" type="1"/></hp:t></hp:run>"#,
                r#"<hp:run charPrIDRef="2"><hp:t>b<hp:lineBreak/>c</hp:t></hp:run>"#
            )
        );
    }

    #[test]
    fn task1378_empty_paragraph_single_run_id_zero() {
        // [#1592 갱신] 완전 빈 문단(text="", char_shapes=[], 컨트롤 없음)은 run 을 방출하지
        // 않는다. char_shapes=[] 는 "원본에 <hp:run> 없음"을 의미하므로(빈 run 이 있었다면
        // 파서가 [(0,0)] 을 산출), run 을 추가하면 재파싱 시 spurious (0,0) 가 생긴다(#1592).
        // 종전 #1378 은 빈 run(id 0)을 방출했으나, 이는 run 없던 빈 문단에 entry 를 가공했다.
        let para = Paragraph::default();
        assert_eq!(runs_of(&para), "");
    }

    #[test]
    fn task1378_field_end_stays_in_previous_run() {
        // 필드 begin/end 와 경계 교차 (경계 케이스 6)
        // 스트림: fieldBegin(0..8) + "abc"(8..11) + fieldEnd(11..19) + "de"(19..21)
        // 경계 pos=19 — fieldEnd 는 이전 run, "de" 는 새 run
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 11;
        let mut para = Paragraph::default();
        para.text = "abcde".to_string();
        para.char_offsets = vec![8, 9, 10, 19, 20];
        para.char_count = 22;
        para.controls.push(Control::Field(f));
        para.field_ranges.push(FieldRange {
            start_char_idx: 0,
            end_char_idx: 3,
            control_idx: 0,
        });
        para.char_shapes = vec![cs(0, 1), cs(19, 2)];
        let xml = runs_of(&para);
        let end_pos = xml.find("fieldEnd").expect("fieldEnd");
        let run2 = xml
            .find(r#"<hp:run charPrIDRef="2">"#)
            .expect("두 번째 run");
        assert!(
            end_pos < run2,
            "fieldEnd 는 이전 run 소속이어야 한다: {}",
            xml
        );
        assert!(
            xml.ends_with(r#"<hp:run charPrIDRef="2"><hp:t>de</hp:t></hp:run>"#),
            "de 는 새 run 소속이어야 한다: {}",
            xml
        );
    }

    #[test]
    fn task1378_trailing_boundary_emits_empty_run() {
        // 콘텐츠 끝 이후 경계 — 빈 run(<hp:t></hp:t>)으로 entry 보존 (규칙 5)
        let mut para = Paragraph::default();
        para.text = "abc".to_string();
        para.char_shapes = vec![cs(0, 1), cs(3, 2)];
        assert_eq!(
            runs_of(&para),
            r#"<hp:run charPrIDRef="1"><hp:t>abc</hp:t></hp:run><hp:run charPrIDRef="2"><hp:t></hp:t></hp:run>"#
        );
    }

    #[test]
    fn task1378_trailing_slot_in_new_run() {
        // 문단 끝 슬롯 위치의 경계 — trailing 슬롯도 새 run 소속 (규칙 1)
        // 스트림: "abc"(0..3) + equation 슬롯(3..11), 경계 pos=3
        let mut para = Paragraph::default();
        para.text = "abc".to_string();
        para.char_offsets = vec![0, 1, 2];
        para.char_count = 12; // 3 + 8(슬롯) + 1
        para.controls.push(Control::Equation(Box::default()));
        para.char_shapes = vec![cs(0, 1), cs(3, 2)];
        let xml = runs_of(&para);
        assert!(
            xml.starts_with(
                r#"<hp:run charPrIDRef="1"><hp:t>abc</hp:t></hp:run><hp:run charPrIDRef="2"><hp:equation"#
            ),
            "trailing 슬롯은 경계 적용 후 새 run 에 들어가야 한다: {}",
            &xml[..200.min(xml.len())]
        );
    }

    #[test]
    fn task1378_section_first_paragraph_secpr_run_id_follows_first_cs() {
        // 섹션 첫 문단 — secPr run id 와 텍스트 run id 의 dedup 상호작용 (경계 케이스 7)
        let mut para = Paragraph::default();
        para.text = "hi".to_string();
        para.char_shapes = vec![cs(0, 7)];
        let (doc, section) = make_doc_with_paragraph(para);
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_section(&section, &doc, 0, &mut ctx).unwrap()).unwrap();
        assert!(
            xml.contains(r#"<hp:run charPrIDRef="7"><hp:secPr "#),
            "secPr run id 는 첫 텍스트 run id 와 일치해야 한다 (재파싱 시 (0,0) 오염 방지)"
        );
        assert!(
            xml.contains(r#"<hp:run charPrIDRef="7"><hp:t>hi</hp:t></hp:run>"#),
            "텍스트 run 은 완전한 run 시퀀스로 치환돼야 한다"
        );
        assert!(
            !xml.contains(r#"<hp:run charPrIDRef="0">"#),
            "템플릿의 charPrIDRef=0 run 이 남아있으면 안 된다: {}",
            xml
        );
    }

    #[test]
    fn task1378_serialize_parse_roundtrip_preserves_char_shapes() {
        // serialize → parse 왕복 후 본문 char_shapes 시퀀스 보존.
        // 파서 정합 IR 로 구성: 섹션 첫 문단은 secPr(8)+colPr(8) 가 위치 축을 16 만큼
        // 선점하므로 char_offsets 가 16 부터 시작하고, 첫 entry 는 secPr run 시작(0)에
        // 기록된다 (stage1 양상 ② 메커니즘).
        use crate::model::style::CharShape;

        // 첫 문단: "abcdef", 경계 1개 — 원본 파스 결과는 [(0,1),(19,2)]
        let mut p0 = Paragraph::default();
        p0.text = "abcdef".to_string();
        p0.char_offsets = vec![16, 17, 18, 19, 20, 21];
        p0.char_count = 23;
        p0.char_shapes = vec![cs(0, 1), cs(19, 2)];

        // 추가 문단: 위치 축이 0 부터 — [(0,1),(3,2)]
        let mut p1 = Paragraph::default();
        p1.text = "abcdef".to_string();
        p1.char_offsets = vec![0, 1, 2, 3, 4, 5];
        p1.char_count = 7;
        p1.char_shapes = vec![cs(0, 1), cs(3, 2)];

        let mut section = Section::default();
        section.paragraphs.push(p0);
        section.paragraphs.push(p1);
        let mut doc = Document::default();
        doc.doc_info.char_shapes = vec![
            CharShape::default(),
            CharShape::default(),
            CharShape::default(),
        ];
        doc.sections.push(section);

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");
        let shapes_of = |i: usize| -> Vec<(u32, u32)> {
            doc2.sections[0].paragraphs[i]
                .char_shapes
                .iter()
                .map(|r| (r.start_pos, r.char_shape_id))
                .collect()
        };
        assert_eq!(shapes_of(0), vec![(0, 1), (19, 2)], "섹션 첫 문단");
        assert_eq!(shapes_of(1), vec![(0, 1), (3, 2)], "추가 문단");
    }

    // ---------- #1584: 본문 인라인 ColumnDef 드롭 회귀 가드 ----------

    #[test]
    fn task1584_body_first_para_two_columndefs_roundtrip() {
        // 본문 첫 문단에 ColumnDef 2개(섹션 단 정의 + 인라인 단 정의).
        // 섹션 템플릿은 첫 ColumnDef 1개만 흡수하고, 2번째 인라인 ColumnDef 는
        // 본문 인라인 슬롯에서 제외되어 드롭된다(controls 6→5 양상).
        // 수정 전: reparse 후 ColumnDef 1개만 → RED. 수정 후: 2개 보존 → GREEN.
        let mut p0 = Paragraph::default();
        p0.controls.push(Control::ColumnDef(ColumnDef::default()));
        p0.controls.push(Control::ColumnDef(ColumnDef::default()));

        let mut section = Section::default();
        section.paragraphs.push(p0);
        let mut doc = Document::default();
        doc.sections.push(section);

        let bytes = crate::serializer::hwpx::serialize_hwpx(&doc).expect("serialize");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");
        let coldef_count = doc2.sections[0].paragraphs[0]
            .controls
            .iter()
            .filter(|c| matches!(c, Control::ColumnDef(_)))
            .count();
        assert_eq!(
            coldef_count, 2,
            "본문 첫 문단의 ColumnDef 2개가 roundtrip 후 모두 보존돼야 한다 (템플릿1 + 인라인1): {coldef_count}"
        );
    }

    // ---------- #1596: generic-shape 지오메트리 직렬화 ----------

    #[test]
    fn task1596_polygon_geometry_serialized() {
        // [#1596] polygon 의 꼭짓점(hc:pt)·테두리(lineShape)·그림자(shadow)가 방출돼야 한다.
        // render_common_shape_xml 이 종전 이들을 드롭 → 도형 형상 소실 → 페이지 붕괴(#1589 잔여).
        use crate::model::shape::PolygonShape;
        use crate::model::Point;
        let mut poly = PolygonShape::default();
        poly.points = vec![
            Point { x: 0, y: 0 },
            Point { x: 100, y: 0 },
            Point { x: 100, y: 100 },
        ];
        poly.drawing.border_line.width = 50;
        poly.drawing.shadow_type = 1;
        let mut ctx = SerializeContext::collect_from_document(&Document::default());
        let xml = render_shape(&ShapeObject::Polygon(poly), &mut ctx);
        assert!(xml.contains("<hc:pt "), "폴리곤 꼭짓점(hc:pt) 방출: {xml}");
        assert!(xml.contains("<hp:lineShape"), "lineShape 방출: {xml}");
        assert!(xml.contains("<hp:shadow"), "shadow 방출: {xml}");
    }
}
