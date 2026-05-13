//! Cross-document paragraph migration primitive.
//!
//! 두 개의 별도 Document IR 사이에서 paragraph를 안전하게 이동하는 primitive.
//! ID 참조(charPr/paraPr/style/borderFill)는 destination DocInfo 에 머지된다.
//!
//! ## Stage 1 범위
//!
//! - `IdRemap` / `MigrateReport` 타입 정의
//! - 4개 카테고리(borderFill/charPr/paraPr/style)에 대한 정의 머지 + 위상 정렬
//! - paragraph walker / 삽입 로직은 Stage 2~3 에서 추가
//!
//! ## Dedup 정책
//!
//! - `CharShape` / `ParaShape` : 명시적 `PartialEq` 보유 (raw_data 제외) → find-or-append
//! - `BorderFill` / `Style` : `PartialEq` 부재 → 항상 append (Stage 1 안전 default)
//!
//! 후속 태스크에서 `BorderFill`/`Style` 및 하위 타입 PartialEq 추가 시
//! `append_border_fill`/`append_style` 을 `find_or_append_*` 패턴으로 전환 가능.
//!
//! ## Note: Style 의 ID ref
//!
//! `Style` 구조는 `para_shape_id (u16)`, `char_shape_id (u16)` 만 ID ref로 가지며
//! `border_fill_id` 필드는 부재. 따라서 style 의 위상 의존은 paraShape/charShape 만.

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::document::{DocInfo, Document};
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::model::style::{BorderFill, CharShape, ParaShape, Style};
use crate::model::table::Table;
use std::collections::HashMap;
use std::ops::Range;

/// 4개 ID 카테고리에 대한 source → target 매핑.
///
/// `char_shape` 는 `CharShapeRef.char_shape_id (u32)` 와의 매칭을 위해 u32 키.
/// `Style.char_shape_id (u16)` 적용 시 `as u16` cast.
#[derive(Default, Debug, Clone)]
pub struct IdRemap {
    pub border_fill: HashMap<u16, u16>,
    pub char_shape: HashMap<u32, u32>,
    pub para_shape: HashMap<u16, u16>,
    pub style: HashMap<u8, u8>,
}

/// Cross-document paragraph migration 의 결과.
#[derive(Debug)]
pub struct MigrateReport {
    pub inserted_para_count: usize,
    pub last_para_idx: usize,
    pub last_char_offset: usize,
    pub id_remap: IdRemap,
}

/// 4개 카테고리에 대해 destination DocInfo 에 정의를 머지하고 remap 테이블을 반환.
///
/// 의존성 위상 순서: `borderFill → charShape → paraShape → style`.
/// 후행 카테고리는 선행 카테고리의 ID ref 를 미리 remap 한 뒤 비교/append.
pub(crate) fn remap_definitions(src: &DocInfo, dst: &mut DocInfo) -> IdRemap {
    let mut remap = IdRemap::default();

    // 1. borderFill (cross-ref to other categories: 없음)
    //    Stage 1: PartialEq 부재 → 항상 append.
    for (src_idx, src_bf) in src.border_fills.iter().enumerate() {
        let target = append_border_fill(dst, src_bf.clone());
        remap.border_fill.insert(src_idx as u16, target);
    }

    // 2. charShape (refers borderFill)
    //    PartialEq dedup 활성.
    for (src_idx, src_cs) in src.char_shapes.iter().enumerate() {
        let mut adjusted = src_cs.clone();
        adjusted.border_fill_id = remap_border_fill_id(&remap, adjusted.border_fill_id);
        let target = find_or_append_char_shape(dst, adjusted);
        remap.char_shape.insert(src_idx as u32, target);
    }

    // 3. paraShape (refers borderFill)
    //    PartialEq dedup 활성.
    //    tab_def_id, numbering_id remap 은 후속 (현재 fragment dataset 미사용).
    for (src_idx, src_ps) in src.para_shapes.iter().enumerate() {
        let mut adjusted = src_ps.clone();
        adjusted.border_fill_id = remap_border_fill_id(&remap, adjusted.border_fill_id);
        let target = find_or_append_para_shape(dst, adjusted);
        remap.para_shape.insert(src_idx as u16, target);
    }

    // 4. style (refers paraShape, charShape — borderFill ref 부재)
    //    Stage 1: PartialEq 부재 → 항상 append.
    for (src_idx, src_st) in src.styles.iter().enumerate() {
        let mut adjusted = src_st.clone();
        adjusted.para_shape_id = remap_para_shape_id(&remap, adjusted.para_shape_id);
        adjusted.char_shape_id = remap_char_shape_id_u16(&remap, adjusted.char_shape_id);
        let target = append_style(dst, adjusted);
        remap.style.insert(src_idx as u8, target);
    }

    remap
}

// ─── Append / find-or-append 헬퍼 ───────────────────────────────────────────

fn append_border_fill(dst: &mut DocInfo, src: BorderFill) -> u16 {
    dst.border_fills.push(src);
    (dst.border_fills.len() - 1) as u16
}

fn find_or_append_char_shape(dst: &mut DocInfo, src: CharShape) -> u32 {
    if let Some(idx) = dst.char_shapes.iter().position(|x| *x == src) {
        return idx as u32;
    }
    dst.char_shapes.push(src);
    (dst.char_shapes.len() - 1) as u32
}

fn find_or_append_para_shape(dst: &mut DocInfo, src: ParaShape) -> u16 {
    if let Some(idx) = dst.para_shapes.iter().position(|x| *x == src) {
        return idx as u16;
    }
    dst.para_shapes.push(src);
    (dst.para_shapes.len() - 1) as u16
}

fn append_style(dst: &mut DocInfo, src: Style) -> u8 {
    dst.styles.push(src);
    (dst.styles.len() - 1) as u8
}

// ─── Remap lookup 헬퍼 (없으면 원본 ID 그대로 유지) ─────────────────────────

fn remap_border_fill_id(remap: &IdRemap, src_id: u16) -> u16 {
    *remap.border_fill.get(&src_id).unwrap_or(&src_id)
}

fn remap_para_shape_id(remap: &IdRemap, src_id: u16) -> u16 {
    *remap.para_shape.get(&src_id).unwrap_or(&src_id)
}

/// `CharShapeRef.char_shape_id` 는 u32, `Style.char_shape_id` 는 u16.
/// remap 테이블은 u32 단일 키로 통합되어 있으므로 Style 적용 시 cast.
fn remap_char_shape_id_u16(remap: &IdRemap, src_id: u16) -> u16 {
    let new_u32 = *remap
        .char_shape
        .get(&(src_id as u32))
        .unwrap_or(&(src_id as u32));
    new_u32 as u16
}

// ─── Paragraph walker (Stage 2) ─────────────────────────────────────────────

/// `Paragraph` 의 ID ref 를 remap 결과에 따라 갈아끼운다.
/// `controls` 안의 Table/Header/Footer/Footnote/Endnote/CharOverlap 을 재귀 처리.
///
/// Stage 2 미지원 variant (Shape, Picture, Equation, Form, HiddenComment 등):
/// - Shape 자체에는 ID ref 가 없지만 `ShapeComponentAttr.text_box.paragraphs`
///   (TextBox 내부 문단) 가 paragraph IR 을 포함. 후속 태스크에서 `walk_shape` 추가.
/// - 그 외는 ID ref 가 없거나 미사용 영역.
pub(crate) fn walk_paragraph(p: &mut Paragraph, remap: &IdRemap) {
    if let Some(&new) = remap.para_shape.get(&p.para_shape_id) {
        p.para_shape_id = new;
    }
    if let Some(&new) = remap.style.get(&p.style_id) {
        p.style_id = new;
    }
    for cs in &mut p.char_shapes {
        if let Some(&new) = remap.char_shape.get(&cs.char_shape_id) {
            cs.char_shape_id = new;
        }
    }
    for ctrl in &mut p.controls {
        walk_control(ctrl, remap);
    }
}

fn walk_paragraphs(paragraphs: &mut [Paragraph], remap: &IdRemap) {
    for p in paragraphs {
        walk_paragraph(p, remap);
    }
}

fn walk_control(ctrl: &mut Control, remap: &IdRemap) {
    match ctrl {
        Control::Table(tbl) => walk_table(tbl, remap),
        Control::Header(h) => walk_paragraphs(&mut h.paragraphs, remap),
        Control::Footer(f) => walk_paragraphs(&mut f.paragraphs, remap),
        Control::Footnote(fn_) => walk_paragraphs(&mut fn_.paragraphs, remap),
        Control::Endnote(en) => walk_paragraphs(&mut en.paragraphs, remap),
        Control::CharOverlap(co) => {
            for id in &mut co.char_shape_ids {
                if let Some(&new) = remap.char_shape.get(id) {
                    *id = new;
                }
            }
        }
        // Shape/Picture/Equation/Form/HiddenComment/SectionDef/ColumnDef/AutoNumber/
        // NewNumber/PageNumberPos/Bookmark/Hyperlink/Ruby/PageHide/Field/Unknown:
        // Stage 2 미지원 (대부분 ID ref 없거나 Shape 내부 paragraphs 처리는 후속).
        _ => {}
    }
}

fn walk_table(t: &mut Table, remap: &IdRemap) {
    if let Some(&new) = remap.border_fill.get(&t.border_fill_id) {
        t.border_fill_id = new;
    }
    for zone in &mut t.zones {
        if let Some(&new) = remap.border_fill.get(&zone.border_fill_id) {
            zone.border_fill_id = new;
        }
    }
    for cell in &mut t.cells {
        if let Some(&new) = remap.border_fill.get(&cell.border_fill_id) {
            cell.border_fill_id = new;
        }
        walk_paragraphs(&mut cell.paragraphs, remap);
    }
    if let Some(caption) = &mut t.caption {
        walk_paragraphs(&mut caption.paragraphs, remap);
    }
}

// ─── Stage 4: build_mini_document_from_fragment + paste_hwpx_fragment ─────

const HWPX_NS_DECL: &str = " xmlns:hp=\"http://www.hancom.co.kr/hwpml/2011/paragraph\" \
                              xmlns:hh=\"http://www.hancom.co.kr/hwpml/2011/head\" \
                              xmlns:hc=\"http://www.hancom.co.kr/hwpml/2011/core\" \
                              xmlns:hs=\"http://www.hancom.co.kr/hwpml/2011/section\"";

/// HWPX fragment(byte-exact) + 4개 카테고리 source 정의 raw XML 을
/// in-memory mini Document 로 빌드한다.
///
/// 각 인자는 raw HWPX XML 스니펫 (1개 이상의 `<hh:*>` 또는 `<hp:p>`):
/// - `fragment_xml`: 1개 이상의 `<hp:p ...>...</hp:p>` 시퀀스
/// - `char_prs`: 1개 이상의 `<hh:charPr ...>...</hh:charPr>`
/// - `para_prs`: 1개 이상의 `<hh:paraPr ...>...</hh:paraPr>`
/// - `styles`: 1개 이상의 `<hh:style .../>`
/// - `border_fills`: 1개 이상의 `<hh:borderFill ...>...</hh:borderFill>`
///
/// 빈 문자열은 해당 카테고리에 정의 없음을 의미.
pub(crate) fn build_mini_document_from_fragment(
    fragment_xml: &str,
    char_prs: &str,
    para_prs: &str,
    styles: &str,
    border_fills: &str,
) -> Result<Document, HwpError> {
    use crate::parser::hwpx::{header::parse_hwpx_header, section::parse_hwpx_section};

    let header_xml = format!(
        "<hh:head{ns}><hh:refList>{bf}{cs}{ps}{st}</hh:refList></hh:head>",
        ns = HWPX_NS_DECL,
        bf = border_fills,
        cs = char_prs,
        ps = para_prs,
        st = styles,
    );
    let (doc_info, _) = parse_hwpx_header(&header_xml).map_err(HwpError::from)?;

    let section_xml = format!(
        "<hs:sec{ns}>{frag}</hs:sec>",
        ns = HWPX_NS_DECL,
        frag = fragment_xml,
    );
    let section = parse_hwpx_section(&section_xml).map_err(HwpError::from)?;

    Ok(Document {
        doc_info,
        sections: vec![section],
        ..Default::default()
    })
}

// ─── Stage 3: cross_document_migrate (DocumentCore impl) ───────────────────

impl DocumentCore {
    /// 외부 Document 의 paragraphs 를 caret 위치로 migrate 한다.
    /// DocInfo 정의(charPr/paraPr/style/borderFill)는 destination 에 머지된다.
    ///
    /// 삽입 패턴은 `paste_html_native` 의 has_controls 분기와 동일:
    /// - dst paragraph 를 caret 에서 split → 좌반 + 우반
    /// - 좌반이 비어있으면 첫 cloned paragraph 로 대체, 아니면 그 뒤에 모두 insert
    /// - 우반이 비어있지 않으면 마지막 cloned 뒤에 insert
    /// - reflow + recompose + paginate 후 `DocumentEvent::FragmentPasted` 발행
    pub fn cross_document_migrate(
        &mut self,
        src_doc: &Document,
        src_section: usize,
        src_paras: Range<usize>,
        dst_section: usize,
        dst_para: usize,
        dst_offset: usize,
    ) -> Result<MigrateReport, HwpError> {
        // 1. 검증
        if dst_section >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 {} 범위 초과",
                dst_section
            )));
        }
        if dst_para >= self.document.sections[dst_section].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 {} 범위 초과",
                dst_para
            )));
        }
        if src_section >= src_doc.sections.len() {
            return Err(HwpError::RenderError(format!(
                "src 구역 {} 범위 초과",
                src_section
            )));
        }
        let src_total = src_doc.sections[src_section].paragraphs.len();
        if src_paras.end > src_total || src_paras.start > src_paras.end {
            return Err(HwpError::RenderError(format!(
                "src paragraph 범위 {:?} 가 총 {} 초과",
                src_paras, src_total
            )));
        }

        // 2. ID 위상 정렬 + 정의 머지
        let remap = remap_definitions(&src_doc.doc_info, &mut self.document.doc_info);
        self.document.doc_info.raw_stream_dirty = true;

        // 3. fragment paragraphs deep-clone + walker
        let cloned: Vec<Paragraph> = src_doc.sections[src_section].paragraphs[src_paras.clone()]
            .iter()
            .map(|p| {
                let mut c = p.clone();
                walk_paragraph(&mut c, &remap);
                c
            })
            .collect();

        let inserted_count = cloned.len();
        if inserted_count == 0 {
            return Ok(MigrateReport {
                inserted_para_count: 0,
                last_para_idx: dst_para,
                last_char_offset: dst_offset,
                id_remap: remap,
            });
        }

        // 4. 삽입 (paste_html_native has_controls 패턴)
        self.document.sections[dst_section].raw_stream = None;

        let right_half =
            self.document.sections[dst_section].paragraphs[dst_para].split_at(dst_offset);

        let left_empty = self.document.sections[dst_section].paragraphs[dst_para]
            .text
            .is_empty();

        let insert_idx = if left_empty {
            // 좌반이 비어있으면 첫 cloned 로 대체
            self.document.sections[dst_section].paragraphs[dst_para] = cloned[0].clone();
            let idx = dst_para + 1;
            for i in 1..inserted_count {
                self.document.sections[dst_section]
                    .paragraphs
                    .insert(idx + i - 1, cloned[i].clone());
            }
            dst_para + inserted_count
        } else {
            // 좌반에 텍스트 → 그 뒤에 모든 cloned 삽입
            let idx = dst_para + 1;
            for i in 0..inserted_count {
                self.document.sections[dst_section]
                    .paragraphs
                    .insert(idx + i, cloned[i].clone());
            }
            dst_para + 1 + inserted_count
        };

        let last_para_idx;
        let last_char_offset;
        if !right_half.text.is_empty() {
            self.document.sections[dst_section]
                .paragraphs
                .insert(insert_idx, right_half);
            last_para_idx = insert_idx;
            last_char_offset = 0;
        } else {
            last_para_idx = insert_idx - 1;
            last_char_offset = self.document.sections[dst_section].paragraphs[last_para_idx]
                .text
                .chars()
                .count();
        }

        // 5. reflow + paginate
        // 주의: paste_html_native 는 `insert_composed_paragraph` 로 composed 캐시를
        // 점진 갱신하지만, paragraphs를 여러 개 insert 한 후의 composed 캐시 일관성을
        // 신뢰하기 어려우므로 cross-document migration 은 dirty mark 후 paginate 에 위임.
        for i in dst_para..=last_para_idx {
            self.reflow_paragraph(dst_section, i);
        }
        self.mark_section_dirty(dst_section);
        self.paginate_if_needed();

        // 6. event log
        self.event_log.push(DocumentEvent::FragmentPasted {
            section: dst_section,
            para: dst_para,
        });

        Ok(MigrateReport {
            inserted_para_count: inserted_count,
            last_para_idx,
            last_char_offset,
            id_remap: remap,
        })
    }

    /// HWPX fragment + 4개 source 정의를 받아 caret 위치에 paste 한다.
    /// `cross_document_migrate` 의 얇은 래퍼.
    ///
    /// 반환: `{"ok":true,"paraIdx":<idx>,"charOffset":<offset>,"insertedParaCount":<n>}`
    pub fn paste_hwpx_fragment_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        fragment_xml: &str,
        char_prs: &str,
        para_prs: &str,
        styles: &str,
        border_fills: &str,
    ) -> Result<String, HwpError> {
        let src = build_mini_document_from_fragment(
            fragment_xml,
            char_prs,
            para_prs,
            styles,
            border_fills,
        )?;
        let n = src.sections[0].paragraphs.len();
        let report =
            self.cross_document_migrate(&src, 0, 0..n, section_idx, para_idx, char_offset)?;
        Ok(format!(
            "{{\"ok\":true,\"paraIdx\":{},\"charOffset\":{},\"insertedParaCount\":{}}}",
            report.last_para_idx, report.last_char_offset, report.inserted_para_count
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_char_shape(font: u16) -> CharShape {
        let mut cs = CharShape::default();
        cs.font_ids = [font; 7];
        cs
    }

    fn make_para_shape(margin_left: i32) -> ParaShape {
        let mut ps = ParaShape::default();
        ps.margin_left = margin_left;
        ps
    }

    fn make_border_fill(attr: u16) -> BorderFill {
        let mut bf = BorderFill::default();
        bf.attr = attr;
        bf
    }

    #[test]
    fn test_remap_reuse_existing_char_shape() {
        // dst에 동일한 charPr 존재 → remap 이 기존 idx 재사용, 길이 변화 없음
        let cs = make_char_shape(7);

        let mut src = DocInfo::default();
        src.char_shapes.push(cs.clone());

        let mut dst = DocInfo::default();
        dst.char_shapes.push(cs.clone());
        let initial_len = dst.char_shapes.len();

        let remap = remap_definitions(&src, &mut dst);

        assert_eq!(
            dst.char_shapes.len(),
            initial_len,
            "기존 charShape 재사용, dst 길이 변화 없음"
        );
        assert_eq!(
            remap.char_shape.get(&0),
            Some(&0),
            "src[0] → dst[0] remap (기존 idx)"
        );
    }

    #[test]
    fn test_remap_append_new_border_fill() {
        // dst에 없는 borderFill → append, 새 idx 부여
        let mut src = DocInfo::default();
        src.border_fills.push(make_border_fill(42));

        let mut dst = DocInfo::default();
        let initial_len = dst.border_fills.len();

        let remap = remap_definitions(&src, &mut dst);

        assert_eq!(
            dst.border_fills.len(),
            initial_len + 1,
            "새 borderFill append (Stage 1: 항상 append)"
        );
        assert_eq!(
            remap.border_fill.get(&0),
            Some(&(initial_len as u16)),
            "src[0] → 새 idx"
        );
    }

    #[test]
    fn test_remap_topological_order_borderfill_first() {
        // src.char_shape 가 src.border_fill_id=2 참조.
        // dst가 비어있을 때 → border_fill 들이 먼저 append되고
        // char_shape 의 border_fill_id 가 dst의 새 idx로 갱신되어야 함.
        let mut src = DocInfo::default();
        src.border_fills.push(make_border_fill(0));
        src.border_fills.push(make_border_fill(1));
        src.border_fills.push(make_border_fill(2));

        let mut cs = make_char_shape(1);
        cs.border_fill_id = 2; // src 기준 idx 2 (bf2) 참조
        src.char_shapes.push(cs);

        let mut dst = DocInfo::default();
        let remap = remap_definitions(&src, &mut dst);

        let new_bf2_idx = *remap.border_fill.get(&2).expect("bf2 remap 존재");
        let new_cs_idx = *remap.char_shape.get(&0).expect("cs0 remap 존재");

        assert_eq!(
            dst.char_shapes[new_cs_idx as usize].border_fill_id, new_bf2_idx,
            "char_shape의 border_fill_id 가 위상 정렬 결과에 따라 갱신됨"
        );
    }

    #[test]
    fn test_remap_para_shape_dedup() {
        // 동일 ParaShape 가 dst에 있으면 재사용
        let ps = make_para_shape(1000);

        let mut src = DocInfo::default();
        src.para_shapes.push(ps.clone());

        let mut dst = DocInfo::default();
        dst.para_shapes.push(ps);
        let initial_len = dst.para_shapes.len();

        let remap = remap_definitions(&src, &mut dst);

        assert_eq!(dst.para_shapes.len(), initial_len, "동일 paraShape 재사용");
        assert_eq!(remap.para_shape.get(&0), Some(&0));
    }

    #[test]
    fn test_remap_style_always_appends() {
        // Stage 1: Style 은 PartialEq 부재 → 항상 append.
        let mut st = Style::default();
        st.local_name = "바탕글".into();
        st.style_type = 0;

        let mut src = DocInfo::default();
        src.styles.push(st.clone());

        let mut dst = DocInfo::default();
        dst.styles.push(st);
        let initial_len = dst.styles.len();

        let remap = remap_definitions(&src, &mut dst);

        assert_eq!(
            dst.styles.len(),
            initial_len + 1,
            "Stage 1: style 은 항상 append (dedup 비활성)"
        );
        assert_eq!(
            remap.style.get(&0),
            Some(&(initial_len as u8)),
            "src[0] → 새 idx (initial_len)"
        );
    }

    #[test]
    fn test_remap_style_id_refs_updated_after_topological_sort() {
        // src.style 이 src.para_shape_id=1 + src.char_shape_id=1 참조.
        // dst가 비어있으면 둘 다 append → style 의 ID ref 들이 새 idx 로 갱신
        let mut src = DocInfo::default();
        src.para_shapes.push(make_para_shape(0));
        src.para_shapes.push(make_para_shape(1000));
        src.char_shapes.push(make_char_shape(0));
        src.char_shapes.push(make_char_shape(1));

        let mut st = Style::default();
        st.para_shape_id = 1;
        st.char_shape_id = 1;
        src.styles.push(st);

        let mut dst = DocInfo::default();
        let remap = remap_definitions(&src, &mut dst);

        let new_ps1 = *remap.para_shape.get(&1).expect("ps1 remap 존재");
        let new_cs1 = *remap.char_shape.get(&1).expect("cs1 remap 존재");
        let new_st_idx = *remap.style.get(&0).expect("style remap 존재");

        let new_st = &dst.styles[new_st_idx as usize];
        assert_eq!(new_st.para_shape_id, new_ps1, "style.para_shape_id 갱신됨");
        assert_eq!(
            new_st.char_shape_id as u32, new_cs1,
            "style.char_shape_id 갱신됨 (u16↔u32 cast)"
        );
    }

    #[test]
    fn test_remap_empty_source_no_changes() {
        let src = DocInfo::default();
        let mut dst = DocInfo::default();
        dst.border_fills.push(make_border_fill(99));
        let initial = dst.border_fills.len();

        let remap = remap_definitions(&src, &mut dst);

        assert!(remap.border_fill.is_empty());
        assert!(remap.char_shape.is_empty());
        assert!(remap.para_shape.is_empty());
        assert!(remap.style.is_empty());
        assert_eq!(
            dst.border_fills.len(),
            initial,
            "빈 src 는 dst 를 변경하지 않음"
        );
    }

    // ─── Stage 2: Walker tests ─────────────────────────────────────────────

    use crate::model::control::CharOverlap;
    use crate::model::header_footer::Header;
    use crate::model::paragraph::CharShapeRef;
    use crate::model::table::{Cell, Table, TableZone};

    fn remap_with_char_shape(src_id: u32, dst_id: u32) -> IdRemap {
        let mut r = IdRemap::default();
        r.char_shape.insert(src_id, dst_id);
        r
    }

    fn remap_with_border_fill(src_id: u16, dst_id: u16) -> IdRemap {
        let mut r = IdRemap::default();
        r.border_fill.insert(src_id, dst_id);
        r
    }

    fn remap_with_para_shape(src_id: u16, dst_id: u16) -> IdRemap {
        let mut r = IdRemap::default();
        r.para_shape.insert(src_id, dst_id);
        r
    }

    #[test]
    fn test_walk_paragraph_text_only() {
        // Para with single CharShapeRef(char_shape_id=0); remap 0→5
        let mut p = Paragraph::default();
        p.para_shape_id = 0;
        p.style_id = 0;
        p.char_shapes.push(CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        });

        let mut remap = remap_with_char_shape(0, 5);
        remap.para_shape.insert(0, 9);
        remap.style.insert(0, 7);

        walk_paragraph(&mut p, &remap);

        assert_eq!(p.para_shape_id, 9, "para_shape_id remap");
        assert_eq!(p.style_id, 7, "style_id remap");
        assert_eq!(
            p.char_shapes[0].char_shape_id, 5,
            "char_shape_id (CharShapeRef) remap"
        );
    }

    #[test]
    fn test_walk_paragraph_no_remap_keeps_id() {
        // remap 에 없는 ID 는 그대로 유지
        let mut p = Paragraph::default();
        p.para_shape_id = 42;
        p.style_id = 3;

        let remap = IdRemap::default(); // 빈 remap
        walk_paragraph(&mut p, &remap);

        assert_eq!(p.para_shape_id, 42);
        assert_eq!(p.style_id, 3);
    }

    fn make_table_2x2_with_bf(table_bf: u16, cell_bf: u16) -> Table {
        let mut t = Table::default();
        t.row_count = 2;
        t.col_count = 2;
        t.border_fill_id = table_bf;
        for r in 0..2_u16 {
            for c in 0..2_u16 {
                let mut cell = Cell::default();
                cell.col = c;
                cell.row = r;
                cell.col_span = 1;
                cell.row_span = 1;
                cell.border_fill_id = cell_bf;
                t.cells.push(cell);
            }
        }
        t
    }

    #[test]
    fn test_walk_table_2x2() {
        let mut t = make_table_2x2_with_bf(2, 3);

        let mut remap = remap_with_border_fill(2, 12);
        remap.border_fill.insert(3, 13);

        walk_table(&mut t, &remap);

        assert_eq!(t.border_fill_id, 12, "table border_fill_id remap");
        for cell in &t.cells {
            assert_eq!(cell.border_fill_id, 13, "cell border_fill_id remap");
        }
        assert_eq!(t.cells.len(), 4);
    }

    #[test]
    fn test_walk_nested_table() {
        // outer.cells[0].paragraphs[0].controls[0] = Inner Table
        // Inner table.border_fill_id 도 remap 적용되어야 함
        let mut inner = make_table_2x2_with_bf(5, 6);
        let mut inner_para = Paragraph::default();
        inner_para.controls.push(Control::Table(Box::new(inner)));

        let mut outer = make_table_2x2_with_bf(2, 3);
        outer.cells[0].paragraphs.push(inner_para);

        let mut remap = IdRemap::default();
        remap.border_fill.insert(2, 12);
        remap.border_fill.insert(3, 13);
        remap.border_fill.insert(5, 15);
        remap.border_fill.insert(6, 16);

        walk_table(&mut outer, &remap);

        assert_eq!(outer.border_fill_id, 12, "outer table remap");
        let extracted_inner = match &outer.cells[0].paragraphs[0].controls[0] {
            Control::Table(t) => t,
            _ => panic!("expected inner Table"),
        };
        assert_eq!(
            extracted_inner.border_fill_id, 15,
            "inner table remap (재귀)"
        );
        for cell in &extracted_inner.cells {
            assert_eq!(cell.border_fill_id, 16, "inner cell remap");
        }
    }

    #[test]
    fn test_walk_char_overlap() {
        // CharOverlap.char_shape_ids 의 각 element 가 remap 적용
        let mut co = CharOverlap::default();
        co.char_shape_ids = vec![1, 2, 3];

        let mut remap = IdRemap::default();
        remap.char_shape.insert(1, 10);
        remap.char_shape.insert(2, 20);
        remap.char_shape.insert(3, 30);

        let mut ctrl = Control::CharOverlap(co);
        walk_control(&mut ctrl, &remap);

        if let Control::CharOverlap(co_out) = ctrl {
            assert_eq!(co_out.char_shape_ids, vec![10, 20, 30]);
        } else {
            panic!("expected CharOverlap");
        }
    }

    #[test]
    fn test_walk_table_zones() {
        let mut t = make_table_2x2_with_bf(0, 0);
        t.zones.push(TableZone {
            start_col: 0,
            start_row: 0,
            end_col: 1,
            end_row: 1,
            border_fill_id: 7,
        });

        let mut remap = IdRemap::default();
        remap.border_fill.insert(0, 100);
        remap.border_fill.insert(7, 17);

        walk_table(&mut t, &remap);

        assert_eq!(
            t.zones[0].border_fill_id, 17,
            "TableZone border_fill_id remap"
        );
    }

    #[test]
    fn test_walk_header_paragraphs() {
        // Header.paragraphs 재귀: header 안 paragraph 의 para_shape_id remap
        let mut header = Header::default();
        let mut p = Paragraph::default();
        p.para_shape_id = 1;
        header.paragraphs.push(p);

        let mut ctrl = Control::Header(Box::new(header));
        let remap = remap_with_para_shape(1, 5);
        walk_control(&mut ctrl, &remap);

        if let Control::Header(h_out) = ctrl {
            assert_eq!(
                h_out.paragraphs[0].para_shape_id, 5,
                "Header.paragraphs[0].para_shape_id remap (재귀)"
            );
        } else {
            panic!("expected Header");
        }
    }

    // ─── Stage 3: cross_document_migrate integration tests ────────────────

    use crate::model::document::{Document, Section};
    use crate::wasm_api::HwpDocument;

    /// blank2010.hwp 템플릿으로 정상 1-section 1-paragraph dst doc 생성.
    fn make_blank_doc() -> HwpDocument {
        let mut doc = HwpDocument::create_empty();
        doc.create_blank_document_native().expect("blank doc setup");
        doc
    }

    /// src 측 mini Document 빌드: paragraphs + doc_info 정의.
    fn make_src_doc_with_paragraphs(paragraphs: Vec<Paragraph>) -> Document {
        let mut doc = Document::default();
        let mut section = Section::default();
        section.paragraphs = paragraphs;
        doc.sections.push(section);
        doc
    }

    fn make_text_paragraph(text: &str, char_shape_id: u32, para_shape_id: u16) -> Paragraph {
        let mut p = Paragraph::default();
        p.text = text.to_string();
        p.char_count = text.chars().count() as u32;
        p.para_shape_id = para_shape_id;
        p.style_id = 0;
        p.char_shapes.push(CharShapeRef {
            start_pos: 0,
            char_shape_id,
        });
        p
    }

    #[test]
    fn test_cross_document_migrate_text_only() {
        // src: 단일 텍스트 paragraph (char_shape_id=0, para_shape_id=0)
        // src.doc_info: char_shape[0], para_shape[0]
        let mut src = make_src_doc_with_paragraphs(vec![make_text_paragraph("안녕", 0, 0)]);
        src.doc_info.char_shapes.push(make_char_shape(1));
        src.doc_info.para_shapes.push(make_para_shape(0));

        let mut doc = make_blank_doc();
        let initial_para_count = doc.document.sections[0].paragraphs.len();

        let report = doc
            .cross_document_migrate(&src, 0, 0..1, 0, 0, 0)
            .expect("migrate ok");

        assert_eq!(report.inserted_para_count, 1, "1 paragraph inserted");
        assert!(
            doc.document.sections[0].paragraphs.len() >= initial_para_count,
            "dst.paragraphs 길이 증가 또는 유지 (좌반 빈 → 대체 가능)"
        );
        // 이벤트 발행 확인
        let last_event = doc.event_log.last().expect("event_log non-empty");
        assert!(
            matches!(last_event, DocumentEvent::FragmentPasted { .. }),
            "FragmentPasted event 발행"
        );
    }

    #[test]
    fn test_cross_document_migrate_table_fragment() {
        // src: table 1개 가진 paragraph
        let table = make_table_2x2_with_bf(2, 3);
        let mut src_para = Paragraph::default();
        src_para.text = "\u{0002}".into(); // table marker (rhwp 관례)
        src_para.char_count = 1;
        src_para.controls.push(Control::Table(Box::new(table)));

        let mut src = make_src_doc_with_paragraphs(vec![src_para]);
        src.doc_info.border_fills.push(make_border_fill(2));
        src.doc_info.border_fills.push(make_border_fill(3));

        let mut doc = make_blank_doc();
        let report = doc
            .cross_document_migrate(&src, 0, 0..1, 0, 0, 0)
            .expect("migrate ok");

        assert_eq!(report.inserted_para_count, 1);
        // remap: src bf 0 (default) → 0, src bf 1 (default) → 1 (Stage 1: 항상 append)
        // 또는 dst doc_info 의 기존 보존된 borderFill 들과 idx 충돌 가능.
        // 검증: report.id_remap.border_fill 에 src idx 0, 1 매핑 존재
        assert!(report.id_remap.border_fill.contains_key(&0));
        assert!(report.id_remap.border_fill.contains_key(&1));
    }

    #[test]
    fn test_cross_document_migrate_caret_position() {
        // dst paragraph 0 에 미리 텍스트가 있고 caret offset > 0 일 때
        // split → 우반 → 마지막 cloned 뒤로 삽입
        let mut doc = make_blank_doc();
        // 빈 문서이므로 텍스트를 채워 caret middle 시뮬레이션
        let _ = doc.insert_text_native(0, 0, 0, "abcde");

        let src = make_src_doc_with_paragraphs(vec![make_text_paragraph("XYZ", 0, 0)]);

        // caret offset = 2 ("ab" 뒤)
        let report = doc
            .cross_document_migrate(&src, 0, 0..1, 0, 0, 2)
            .expect("migrate ok");

        assert_eq!(report.inserted_para_count, 1);
        // dst 측: 좌반 "ab" + cloned "XYZ" + 우반 "cde" 으로 분리됨
        // 정확한 paragraph 개수는 paste_html_native 패턴과 동일하게 3 (또는 우반 이동 시)
        let final_paras = doc.document.sections[0].paragraphs.len();
        assert!(final_paras >= 2, "최소 2 paragraph (split 결과)");
    }

    #[test]
    fn test_cross_document_migrate_out_of_range() {
        let src = make_src_doc_with_paragraphs(vec![make_text_paragraph("x", 0, 0)]);
        let mut doc = make_blank_doc();

        // dst_section 범위 초과
        assert!(doc.cross_document_migrate(&src, 0, 0..1, 99, 0, 0).is_err());
        // src_section 범위 초과
        assert!(doc.cross_document_migrate(&src, 99, 0..1, 0, 0, 0).is_err());
        // src_paras.end 초과
        assert!(doc.cross_document_migrate(&src, 0, 0..99, 0, 0, 0).is_err());
    }

    // ─── Stage 4: build_mini_document + paste_hwpx_fragment_native tests ──

    const TEST_FRAGMENT: &str = r#"<hp:p id="1" paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>x</hp:t></hp:run></hp:p>"#;

    #[test]
    fn test_build_mini_document_from_fragment_minimal() {
        // 가장 작은 fragment + 빈 정의 카테고리 → mini Document 빌드 성공
        let src = build_mini_document_from_fragment(TEST_FRAGMENT, "", "", "", "")
            .expect("build_mini_document ok");

        assert_eq!(src.sections.len(), 1, "1 section");
        assert_eq!(
            src.sections[0].paragraphs.len(),
            1,
            "1 paragraph from fragment"
        );
    }

    #[test]
    fn test_paste_hwpx_fragment_minimal_synthetic() {
        // 합성 fragment paste → JSON 응답 형식 검증
        let mut doc = make_blank_doc();

        let json = doc
            .paste_hwpx_fragment_native(0, 0, 0, TEST_FRAGMENT, "", "", "", "")
            .expect("paste_hwpx_fragment ok");

        assert!(json.contains(r#""ok":true"#), "ok flag: {}", json);
        assert!(json.contains(r#""paraIdx":"#), "paraIdx field: {}", json);
        assert!(
            json.contains(r#""charOffset":"#),
            "charOffset field: {}",
            json
        );
        assert!(
            json.contains(r#""insertedParaCount":1"#),
            "insertedParaCount=1: {}",
            json
        );
    }

    #[test]
    fn test_paste_hwpx_fragment_no_panic_on_malformed() {
        // 의미 없는 fragment 도 panic 없이 완료 (Ok 또는 Err 양쪽 허용)
        let mut doc = make_blank_doc();
        let _ = doc.paste_hwpx_fragment_native(0, 0, 0, "<x/>", "", "", "", "");
        // panic 없으면 통과
    }
}
