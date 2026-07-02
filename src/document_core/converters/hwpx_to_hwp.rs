//! HWPX → HWP IR 매핑 어댑터
//!
//! HWPX 파서가 채운 IR 을 HWP 직렬화기가 받아들이는 형태로 정규화한다.
//!
//! ## 핵심 원칙
//!
//! - **HWP 직렬화기 0줄 수정**: `serializer/cfb_writer.rs`, `body_text.rs`,
//!   `control.rs` 등은 변경하지 않는다.
//! - **IR 만 만진다**: 진입점은 `&mut Document` 이며, 출력은 IR 필드 갱신뿐.
//! - **idempotent**: 같은 IR 에 두 번 호출해도 같은 결과.
//! - **HWP 출처 보호**: `source_format == Hwpx` 일 때만 동작. HWP 출처는 no-op.
//!
//! ## 매핑 명세서
//!
//! HWP 직렬화기가 IR 에서 무엇을 읽는지가 단 하나의 명세서 (구현계획서 §1.3 참조).
//!
//! Stage 1 (현재): 진입점만 노출. 영역별 매핑은 Stage 2~ 에서 추가.

use std::collections::BTreeSet;

use crate::model::bin_data::{BinDataContent, BinDataStatus, BinDataType};
use crate::model::control::Control;
use crate::model::document::{Document, Section, SectionDef};
use crate::model::image::Picture;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{common_obj_offsets, ShapeObject, TextBox};
use crate::model::style::{BorderFill, BorderLineType, Fill, FillType};
use crate::model::table::{Cell, Table, TablePageBreak};
use crate::parser::FileFormat;

use super::common_obj_attr_writer::{pack_common_attr_bits, serialize_common_obj_attr};

/// 어댑터 실행 보고서.
///
/// 각 영역별로 변환된 항목 수를 누적한다. 진단 도구와 단계별 회귀 측정에 사용.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AdapterReport {
    /// 변환을 건너뛴 사유 (HWP 출처 등). None 이면 정상 적용.
    pub skipped_reason: Option<String>,
    /// `table.raw_ctrl_data` 합성 횟수 (Stage 2)
    pub tables_ctrl_data_synthesized: u32,
    /// `table.attr` 재구성 횟수 (Stage 2)
    pub tables_attr_packed: u32,
    /// HWPX 표의 page_break 를 한컴 HWP 저장 관례에 맞춰 보강한 횟수
    pub tables_page_break_materialized: u32,
    /// 표 outer_margin 을 CommonObjAttr.margin 으로 승격한 횟수
    pub tables_outer_margin_materialized: u32,
    /// HWPX 표 CTRL_HEADER attr 중 한컴 HWP 저장 관례 비트 보강 횟수
    pub table_ctrl_header_attr_materialized: u32,
    /// HWPX 표 TABLE record attr 중 한컴 저장 관례 비트 보강 횟수
    pub table_record_attr_materialized: u32,
    /// HWPX 표 TABLE record row-size payload 를 행별 셀 수로 보강한 횟수
    pub table_record_row_sizes_materialized: u32,
    /// HWPX 표 TABLE record trailing zone/count payload 를 한컴 저장 관례로 보강한 횟수
    pub table_record_extra_materialized: u32,
    /// `cell.list_attr bit 16` 보강 횟수 (Stage 3)
    pub cells_list_attr_bit16_set: u32,
    /// HWPX 출처 셀 LIST_HEADER width_ref/raw_list_extra materialize 횟수
    pub cells_list_header_contract_materialized: u32,
    /// paragraph/char shape 참조 BorderFill 무채움 정규화 횟수
    pub border_fills_no_fill_normalized: u32,
    /// HWPX 출처 FileHeader를 HWP5 compressed 저장 관례로 보정한 횟수
    pub file_header_compression_normalized: u32,
    /// HWPX 출처 DocProperties.section_count 보정 횟수
    pub doc_properties_section_count_normalized: u32,
    /// HWPX embedded BinData metadata 보정 횟수
    pub bin_data_metadata_normalized: u32,
    /// HWPX OLE Storage 포함 문서의 HWP5 BinData 순서/참조 materialize 횟수
    pub bin_data_order_materialized: u32,
    /// `Control::SectionDef` 컨트롤 삽입 횟수 (Stage 4 — 섹션 개수)
    pub section_def_controls_inserted: u32,
    /// HWPX `hp:pic@href` 를 HWP CTRL_DATA ParameterSet 으로 materialize한 횟수
    pub picture_href_ctrl_data_materialized: u32,
    /// HWPX 3x2 row-break table의 HWP5 layout CTRL_DATA ParameterSet materialize 횟수
    pub table_layout_ctrl_data_materialized: u32,
    /// HWPX drawText TextBox LIST_HEADER tail materialize 횟수
    pub text_box_list_header_tail_materialized: u32,
    /// HWPX drawText 내부 paragraph PARA_HEADER tail materialize 횟수
    pub text_box_para_header_tail_materialized: u32,
    /// HWPX 출처 일반 paragraph PARA_HEADER tail materialize 횟수
    pub para_header_tail_materialized: u32,
    /// HWPX 수식(Equation) CTRL_HEADER attr 중 한컴 저장 관례 비트 보강 횟수 (Task #1061)
    pub equation_ctrl_header_attr_materialized: u32,
    /// HWPX 수식(Equation) EQEDIT 의 font_name/version_info 정답지 정합 정정 횟수 (Task #1061 Stage 2)
    pub equation_font_version_normalized: u32,
    /// HWPX 바탕쪽 포함 SectionDef CTRL_HEADER 확장 tail materialize 횟수
    pub section_def_master_page_tail_materialized: u32,
    /// HWPX 후속 구역 첫 문단 break_type 을 한컴 HWP 저장 관례로 보정한 횟수
    pub following_section_break_type_materialized: u32,
    /// HWPX SectionDef masterPageCnt=1 flags 를 한컴 HWP 저장 관례로 보정한 횟수
    pub section_def_single_master_page_flags_materialized: u32,
    /// HWPX SectionDef masterPageCnt=2 flags 를 한컴 HWP 저장 관례로 보정한 횟수
    pub section_def_multi_master_page_flags_materialized: u32,
    /// HWPX SectionDef hide_empty_line bool 을 HWP5 flags bit 19로 동기화한 횟수
    pub section_def_hide_empty_line_flag_materialized: u32,
    /// HWPX AutoNumber 뒤 fixed-width space 문단의 HWP5 PARA_RANGE_TAG materialize 횟수
    pub autonum_fwspace_range_tag_materialized: u32,
    /// HWPX AutoNumber 뒤 fixed-width space 문단의 char shape start_pos 보정 횟수
    pub autonum_fwspace_char_shape_offsets_materialized: u32,
    /// HWPX fixed-width space를 HWP5 fixed blank control로 보존한 횟수
    pub header_footer_fwspace_control_materialized: u32,
    /// HWPX 바탕쪽 AutoNumber-only 문단의 placeholder space 제거 횟수
    pub master_page_autonum_placeholder_removed: u32,
    /// HWPX 바탕쪽 line shape rendering matrix를 HWP5 size ratio contract로 보정한 횟수
    pub master_page_line_rendering_size_ratio_materialized: u32,
}

impl AdapterReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn no_op(mut self, reason: impl Into<String>) -> Self {
        self.skipped_reason = Some(reason.into());
        self
    }

    /// 어댑터가 실제로 무언가를 변경했는지 여부.
    pub fn changed_anything(&self) -> bool {
        self.skipped_reason.is_none()
            && (self.tables_ctrl_data_synthesized
                + self.tables_attr_packed
                + self.tables_page_break_materialized
                + self.tables_outer_margin_materialized
                + self.table_ctrl_header_attr_materialized
                + self.table_record_attr_materialized
                + self.table_record_row_sizes_materialized
                + self.table_record_extra_materialized
                + self.cells_list_attr_bit16_set
                + self.cells_list_header_contract_materialized
                + self.border_fills_no_fill_normalized
                + self.file_header_compression_normalized
                + self.doc_properties_section_count_normalized
                + self.bin_data_metadata_normalized
                + self.bin_data_order_materialized
                + self.section_def_controls_inserted
                + self.picture_href_ctrl_data_materialized
                + self.table_layout_ctrl_data_materialized
                + self.text_box_list_header_tail_materialized
                + self.text_box_para_header_tail_materialized
                + self.para_header_tail_materialized
                + self.equation_ctrl_header_attr_materialized
                + self.equation_font_version_normalized
                + self.section_def_master_page_tail_materialized
                + self.following_section_break_type_materialized
                + self.section_def_single_master_page_flags_materialized
                + self.section_def_multi_master_page_flags_materialized
                + self.section_def_hide_empty_line_flag_materialized
                + self.autonum_fwspace_range_tag_materialized
                + self.autonum_fwspace_char_shape_offsets_materialized
                + self.header_footer_fwspace_control_materialized
                + self.master_page_autonum_placeholder_removed
                + self.master_page_line_rendering_size_ratio_materialized)
                > 0
    }
}

/// HWPX 출처 IR 을 HWP 직렬화기가 기대하는 형태로 정규화한다.
///
/// HWP 출처에는 no-op (idempotent + 보호).
///
/// ## 실행 영역
///
/// - **SectionDef 컨트롤 삽입** (Stage 4) — `Section.section_def` 를 첫 문단의 `controls`
///   시작 위치에 `Control::SectionDef` 로 삽입. HWPX 파서가 만들지 않으므로 PAGE_DEF 누락
///   → 재로드 시 페이지 크기 0 이 되는 결손 보강.
/// - **표 raw_ctrl_data + attr 합성** (Stage 2)
/// - **셀 list_attr bit 16 합성** (Stage 3)
///
/// ## lineseg vpos 가 본 어댑터에 없는 이유
///
/// HWPX 로드 시점에 `DocumentCore::from_bytes` 가 `reflow_zero_height_paragraphs`
/// (`document_core/commands/document.rs:208-318`) 를 호출하여 IR 의 `line_segs[].vertical_pos`
/// 를 in-place 로 갱신한다. 이 갱신은 메모리상 IR 에 영구 반영되므로, 어댑터 시점에는 이미
/// 정확한 vpos 가 채워져 있어 추가 사전계산이 불필요. 직렬화 → 재로드 시에도 vpos 가 그대로
/// 보존된다 (정수 필드 라운드트립).
pub fn convert_hwpx_to_hwp_ir(doc: &mut Document) -> AdapterReport {
    let mut report = AdapterReport::new();

    normalize_file_header_for_hwp(doc, &mut report);
    normalize_doc_properties_for_hwp(doc, &mut report);
    materialize_hwp5_bin_data_order(doc, &mut report);
    normalize_bin_data_for_hwp(doc, &mut report);

    // Stage 4: SectionDef 컨트롤 삽입 (HWPX 파서가 만들지 않으므로 직렬화기가 PAGE_DEF 출력 못 함)
    for (section_idx, section) in doc.sections.iter_mut().enumerate() {
        adapt_section_def(&mut section.section_def, &mut report);
        insert_section_def_control(section, &mut report);
        materialize_following_section_break_type(section_idx, section, &mut report);
    }

    normalize_paragraph_char_border_fills(doc, &mut report);

    // Stage 2/3: 표 ctrl_data + 셀 list_attr (raw_ctrl_data 합성)
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            adapt_paragraph(para, &mut report);
        }
    }

    report
}

/// HWPX embedded BinData를 한컴 HWP 저장 관례에 맞춰 materialize한다.
///
/// HWPX parser는 `content.hpf`의 BinData 항목을 모델에 등록하지만 HWP `BIN_DATA`
/// record 전용 attr/status 값은 비워 둔다. 한컴 HWP 로더는 일반 embedded image의
/// `BIN_DATA` record에서 `attr=0x0101` + Success 상태를 허용하지만, OLE storage가 함께
/// 있는 문서에서는 한컴 저장본이 image/OLE 모두 NotAccessed 계약(`0x0001`/`0x0002`)을
/// 사용한다. HWP 저장 직전에 HWPX 출처 모델을 이 계약으로 명시적으로 보정한다.
fn normalize_bin_data_for_hwp(doc: &mut Document, report: &mut AdapterReport) {
    let mut changed = false;
    let has_storage = doc
        .doc_info
        .bin_data_list
        .iter()
        .any(|bin_data| bin_data.data_type == BinDataType::Storage);

    for bin_data in &mut doc.doc_info.bin_data_list {
        if !matches!(
            bin_data.data_type,
            BinDataType::Embedding | BinDataType::Storage
        ) {
            continue;
        }

        let expected_attr = match bin_data.data_type {
            BinDataType::Embedding if has_storage => 0x0001,
            BinDataType::Embedding => 0x0101,
            BinDataType::Storage => 0x0002,
            BinDataType::Link => continue,
        };
        if bin_data.attr != expected_attr {
            bin_data.attr = expected_attr;
            changed = true;
        }

        let expected_status = match bin_data.data_type {
            BinDataType::Embedding if has_storage => BinDataStatus::NotAccessed,
            BinDataType::Embedding => BinDataStatus::Success,
            BinDataType::Storage => BinDataStatus::NotAccessed,
            BinDataType::Link => continue,
        };
        if bin_data.status != expected_status {
            bin_data.status = expected_status;
            changed = true;
        }

        if bin_data.raw_data.is_some() {
            bin_data.raw_data = None;
            changed = true;
        }
    }

    if changed {
        report.bin_data_metadata_normalized += 1;
        doc.doc_info.raw_stream_dirty = true;
    }
}

/// HWPX manifest 순서는 HWP5 저장 시 한컴이 기대하는 BinData 순서와 다를 수 있다.
///
/// 특히 OLE 차트가 포함된 HWPX 변환본은 `content.hpf`에 배경 이미지 → 본문 그림 → OLE
/// 순서로 적히는 경우가 있다. HWP5의 `bin_data_id`는 `BIN_DATA` 레코드 순번을 가리키므로,
/// 한컴 정답지처럼 본문 컨트롤에서 먼저 등장하는 그림/OLE를 앞에 두고 DocInfo 전용 배경
/// 이미지는 뒤로 보내야 한다. 이때 모든 참조 ID와 `BinDataContent.id`도 함께 remap한다.
fn materialize_hwp5_bin_data_order(doc: &mut Document, report: &mut AdapterReport) {
    let bin_count = doc.doc_info.bin_data_list.len();
    if bin_count <= 1
        || !doc
            .doc_info
            .bin_data_list
            .iter()
            .any(|bd| bd.data_type == BinDataType::Storage)
    {
        return;
    }

    let mut order = Vec::with_capacity(bin_count);
    let mut seen = BTreeSet::new();

    for section in &doc.sections {
        collect_bin_order_from_paragraphs(&section.paragraphs, bin_count, &mut order, &mut seen);
        for master_page in &section.section_def.master_pages {
            collect_bin_order_from_paragraphs(
                &master_page.paragraphs,
                bin_count,
                &mut order,
                &mut seen,
            );
        }
    }

    collect_bin_order_from_doc_info(doc, bin_count, &mut order, &mut seen);

    for id in 1..=bin_count as u16 {
        push_bin_order(id, bin_count, &mut order, &mut seen);
    }

    let identity: Vec<u16> = (1..=bin_count as u16).collect();
    if order == identity {
        return;
    }

    let mut remap = vec![0u16; bin_count + 1];
    for (new_idx, old_id) in order.iter().enumerate() {
        remap[*old_id as usize] = (new_idx + 1) as u16;
    }

    let old_bin_data = doc.doc_info.bin_data_list.clone();
    let mut new_bin_data = Vec::with_capacity(old_bin_data.len());
    for old_id in &order {
        let Some(old) = old_bin_data.get((*old_id as usize).saturating_sub(1)) else {
            continue;
        };
        let mut bin_data = old.clone();
        bin_data.storage_id = remap[*old_id as usize];
        new_bin_data.push(bin_data);
    }
    if new_bin_data.len() == old_bin_data.len() {
        doc.doc_info.bin_data_list = new_bin_data;
    }

    let old_content = doc.bin_data_content.clone();
    let mut new_content = Vec::with_capacity(old_content.len());
    for old_id in &order {
        if let Some(content) = old_content.iter().find(|content| content.id == *old_id) {
            let mut content = content.clone();
            content.id = remap[*old_id as usize];
            new_content.push(content);
        }
    }
    for content in old_content {
        if content.id == 0 || content.id as usize > bin_count {
            new_content.push(content);
        }
    }
    doc.bin_data_content = new_content;

    remap_bin_refs_in_doc(doc, &remap);
    doc.doc_info.raw_stream_dirty = true;
    report.bin_data_order_materialized += 1;
}

fn push_bin_order(id: u16, bin_count: usize, order: &mut Vec<u16>, seen: &mut BTreeSet<u16>) {
    if id == 0 || id as usize > bin_count || !seen.insert(id) {
        return;
    }
    order.push(id);
}

fn collect_bin_order_from_doc_info(
    doc: &Document,
    bin_count: usize,
    order: &mut Vec<u16>,
    seen: &mut BTreeSet<u16>,
) {
    for border_fill in &doc.doc_info.border_fills {
        if let Some(image) = &border_fill.fill.image {
            push_bin_order(image.bin_data_id, bin_count, order, seen);
        }
    }
}

fn collect_bin_order_from_paragraphs(
    paragraphs: &[Paragraph],
    bin_count: usize,
    order: &mut Vec<u16>,
    seen: &mut BTreeSet<u16>,
) {
    for para in paragraphs {
        for ctrl in &para.controls {
            collect_bin_order_from_control(ctrl, bin_count, order, seen);
        }
    }
}

fn collect_bin_order_from_control(
    ctrl: &Control,
    bin_count: usize,
    order: &mut Vec<u16>,
    seen: &mut BTreeSet<u16>,
) {
    match ctrl {
        Control::Picture(pic) => {
            push_bin_order(pic.image_attr.bin_data_id, bin_count, order, seen);
        }
        Control::Shape(shape) => collect_bin_order_from_shape(shape, bin_count, order, seen),
        Control::Table(table) => {
            for cell in &table.cells {
                collect_bin_order_from_paragraphs(&cell.paragraphs, bin_count, order, seen);
            }
        }
        Control::Header(header) => {
            collect_bin_order_from_paragraphs(&header.paragraphs, bin_count, order, seen);
        }
        Control::Footer(footer) => {
            collect_bin_order_from_paragraphs(&footer.paragraphs, bin_count, order, seen);
        }
        Control::Footnote(footnote) => {
            collect_bin_order_from_paragraphs(&footnote.paragraphs, bin_count, order, seen);
        }
        Control::Endnote(endnote) => {
            collect_bin_order_from_paragraphs(&endnote.paragraphs, bin_count, order, seen);
        }
        _ => {}
    }
}

fn collect_bin_order_from_shape(
    shape: &ShapeObject,
    bin_count: usize,
    order: &mut Vec<u16>,
    seen: &mut BTreeSet<u16>,
) {
    match shape {
        ShapeObject::Picture(pic) => {
            push_bin_order(pic.image_attr.bin_data_id, bin_count, order, seen);
        }
        ShapeObject::Ole(ole) => {
            if let Ok(id) = u16::try_from(ole.bin_data_id) {
                push_bin_order(id, bin_count, order, seen);
            }
        }
        ShapeObject::Group(group) => {
            for child in &group.children {
                collect_bin_order_from_shape(child, bin_count, order, seen);
            }
        }
        _ => {}
    }

    if let Some(drawing) = shape.drawing() {
        if let Some(image) = &drawing.fill.image {
            push_bin_order(image.bin_data_id, bin_count, order, seen);
        }
        if let Some(text_box) = &drawing.text_box {
            collect_bin_order_from_paragraphs(&text_box.paragraphs, bin_count, order, seen);
        }
    }
}

fn remap_bin_refs_in_doc(doc: &mut Document, remap: &[u16]) {
    for border_fill in &mut doc.doc_info.border_fills {
        remap_bin_ref_in_fill(&mut border_fill.fill, remap);
    }

    for section in &mut doc.sections {
        remap_bin_refs_in_paragraphs(&mut section.paragraphs, remap);
        for master_page in &mut section.section_def.master_pages {
            remap_bin_refs_in_paragraphs(&mut master_page.paragraphs, remap);
        }
    }
}

fn remap_bin_refs_in_paragraphs(paragraphs: &mut [Paragraph], remap: &[u16]) {
    for para in paragraphs {
        for ctrl in &mut para.controls {
            remap_bin_refs_in_control(ctrl, remap);
        }
    }
}

fn remap_bin_refs_in_control(ctrl: &mut Control, remap: &[u16]) {
    match ctrl {
        Control::Picture(pic) => {
            pic.image_attr.bin_data_id = remap_bin_ref(pic.image_attr.bin_data_id, remap);
        }
        Control::Shape(shape) => remap_bin_refs_in_shape(shape, remap),
        Control::Table(table) => {
            for cell in &mut table.cells {
                remap_bin_refs_in_paragraphs(&mut cell.paragraphs, remap);
            }
        }
        Control::Header(header) => remap_bin_refs_in_paragraphs(&mut header.paragraphs, remap),
        Control::Footer(footer) => remap_bin_refs_in_paragraphs(&mut footer.paragraphs, remap),
        Control::Footnote(footnote) => {
            remap_bin_refs_in_paragraphs(&mut footnote.paragraphs, remap)
        }
        Control::Endnote(endnote) => remap_bin_refs_in_paragraphs(&mut endnote.paragraphs, remap),
        _ => {}
    }
}

fn remap_bin_refs_in_shape(shape: &mut ShapeObject, remap: &[u16]) {
    match shape {
        ShapeObject::Picture(pic) => {
            pic.image_attr.bin_data_id = remap_bin_ref(pic.image_attr.bin_data_id, remap);
        }
        ShapeObject::Ole(ole) => {
            if let Ok(id) = u16::try_from(ole.bin_data_id) {
                ole.bin_data_id = remap_bin_ref(id, remap) as u32;
            }
        }
        ShapeObject::Group(group) => {
            for child in &mut group.children {
                remap_bin_refs_in_shape(child, remap);
            }
        }
        _ => {}
    }

    if let Some(drawing) = shape.drawing_mut() {
        remap_bin_ref_in_fill(&mut drawing.fill, remap);
        if let Some(text_box) = &mut drawing.text_box {
            remap_bin_refs_in_paragraphs(&mut text_box.paragraphs, remap);
        }
    }
}

fn remap_bin_ref_in_fill(fill: &mut Fill, remap: &[u16]) {
    if let Some(image) = &mut fill.image {
        image.bin_data_id = remap_bin_ref(image.bin_data_id, remap);
    }
}

fn remap_bin_ref(id: u16, remap: &[u16]) -> u16 {
    remap
        .get(id as usize)
        .copied()
        .filter(|new_id| *new_id != 0)
        .unwrap_or(id)
}

/// HWPX 출처 문서를 HWP5 저장 관례에 맞춰 압축 문서로 보정한다.
///
/// HWPX 파서는 HWP `FileHeader` 원본이 없기 때문에 `compressed=false`, `flags=0`인
/// 임시 헤더를 만든다. 그러나 HWP 저장기는 이 값을 그대로 사용해 DocInfo/BodyText/BinData
/// 스트림 압축 여부를 결정한다. Stage30 probe의 공통 기준선도 압축 플래그를 켠 상태였으므로,
/// HWPX -> HWP 저장 adapter는 HWP5 compressed 헤더를 명시적으로 materialize해야 한다.
fn normalize_file_header_for_hwp(doc: &mut Document, report: &mut AdapterReport) {
    let mut changed = false;

    if !doc.header.compressed {
        doc.header.compressed = true;
        changed = true;
    }

    if doc.header.flags & 0x01 == 0 {
        doc.header.flags |= 0x01;
        changed = true;
    }

    if doc.header.raw_data.is_some() {
        doc.header.raw_data = None;
        changed = true;
    }

    if changed {
        report.file_header_compression_normalized += 1;
    }
}

/// HWP `DOCUMENT_PROPERTIES`의 구역 개수를 실제 BodyText 섹션 수와 동기화한다.
///
/// HWPX header.xml 파싱 경로는 `DocProperties.section_count`를 기본값 1로 남길 수 있다.
/// 한컴 HWP 로더는 이 값을 BodyText 섹션 스트림 해석의 상한으로 사용하므로, 실제 섹션이
/// 2개 이상인 문서에서는 마지막 섹션이 렌더링되지 않는다.
fn normalize_doc_properties_for_hwp(doc: &mut Document, report: &mut AdapterReport) {
    let section_count = doc.sections.len().min(u16::MAX as usize) as u16;
    let changed =
        doc.doc_properties.section_count != section_count || doc.doc_properties.raw_data.is_some();

    doc.doc_properties.section_count = section_count;
    doc.doc_properties.raw_data = None;

    if changed {
        report.doc_properties_section_count_normalized += 1;
        doc.doc_info.raw_stream_dirty = true;
    }
}

/// 섹션의 `section_def` 를 첫 문단의 `controls` 시작 위치에 `Control::SectionDef` 로 삽입한다.
///
/// ## 배경
///
/// HWPX 파서는 `<hp:secPr>` 정보를 `Section.section_def` 필드와
/// `Control::SectionDef` 컨트롤에 함께 반영한다. 단, 예전 파서 산출물이나 외부 생성 IR처럼
/// `section_def` 필드만 있고 문단 control stream 에 `Control::SectionDef` 가 빠진 문서를
/// HWP로 저장할 수 있으므로, 어댑터는 fallback 으로 이 컨트롤을 보강한다.
/// HWP 직렬화기 (`serializer/control.rs:40 + 171-241`) 는 `paragraph.controls` 를
/// 순회하면서 `Control::SectionDef` 를 만나야 PAGE_DEF / FOOTNOTE_SHAPE / PAGE_BORDER_FILL
/// 레코드를 출력한다. 이 컨트롤이 없으면 직렬화 결과의 PAGE_DEF 가 누락되어 재로드 시
/// `page_def.width = 0` 등 페이지 크기 손상으로 페이지 폭주 발생.
///
/// ## 동작
///
/// 1. 섹션의 첫 문단에 `Control::SectionDef` 가 이미 있으면 `section.section_def` 의 최신
///    값을 반영한다. HWPX package-level masterpage 는 section XML 파싱 뒤에 붙기 때문에,
///    기존 컨트롤 복사본이 오래된 상태일 수 있다.
/// 2. 없으면 `Control::SectionDef(Box::new(section.section_def.clone()))` 를 첫 문단의
///    `controls[0]` 위치에 삽입
///
/// ## 한컴 영향
///
/// 한컴은 `<secd>` CTRL_HEADER 와 PAGE_DEF 를 정상 인식. HWP 출처에서는 이미 컨트롤이
/// 있으므로 idempotent 가드에 막혀 변경 없음.
fn insert_section_def_control(section: &mut Section, report: &mut AdapterReport) {
    if section.paragraphs.is_empty() {
        return;
    }
    let first_para = &mut section.paragraphs[0];
    if let Some(Control::SectionDef(section_def)) = first_para
        .controls
        .iter_mut()
        .find(|c| matches!(c, Control::SectionDef(_)))
    {
        **section_def = section.section_def.clone();
        return;
    }
    first_para.controls.insert(
        0,
        Control::SectionDef(Box::new(section.section_def.clone())),
    );
    report.section_def_controls_inserted += 1;
}

fn materialize_following_section_break_type(
    section_idx: usize,
    section: &mut Section,
    report: &mut AdapterReport,
) {
    if section_idx == 0 {
        return;
    }

    let Some(first_para) = section.paragraphs.first_mut() else {
        return;
    };

    let has_section_def = first_para
        .controls
        .iter()
        .any(|control| matches!(control, Control::SectionDef(_)));
    if !has_section_def || first_para.raw_break_type != 0 {
        return;
    }

    // HWPX parser가 pageBreak/columnBreak/secPr/colPr를 HWP5 break flag로 합성한다.
    // 이 adapter는 과거 IR처럼 raw_break_type이 완전히 비어 있는 경우에만 최소 section
    // break를 보강한다. 이미 materialize된 0x03/0x07 같은 조합을 덮어쓰면 한컴이
    // 후속 section의 바탕쪽/머리말 layout을 다르게 해석한다.
    first_para.raw_break_type = 0x01;
    report.following_section_break_type_materialized += 1;
}

fn normalize_paragraph_char_border_fills(doc: &mut Document, report: &mut AdapterReport) {
    let para_char_refs = collect_paragraph_char_border_fill_refs(doc);
    if para_char_refs.is_empty() {
        return;
    }

    let object_refs = collect_object_border_fill_refs(doc);
    for id in para_char_refs {
        if id == 0 || object_refs.contains(&id) {
            continue;
        }

        let Some(border_fill) = doc
            .doc_info
            .border_fills
            .get_mut(id.saturating_sub(1) as usize)
        else {
            continue;
        };

        if is_transparent_paragraph_no_fill_candidate(border_fill) {
            border_fill.fill.fill_type = FillType::None;
            border_fill.fill.solid = None;
            border_fill.fill.gradient = None;
            border_fill.fill.image = None;
            border_fill.fill.alpha = 0;
            border_fill.raw_data = None;
            report.border_fills_no_fill_normalized += 1;
        }
    }
}

fn collect_paragraph_char_border_fill_refs(doc: &Document) -> std::collections::HashSet<u16> {
    let mut refs = std::collections::HashSet::new();
    for para_shape in &doc.doc_info.para_shapes {
        if para_shape.border_fill_id > 0 {
            refs.insert(para_shape.border_fill_id);
        }
    }
    for char_shape in &doc.doc_info.char_shapes {
        if char_shape.border_fill_id > 0 {
            refs.insert(char_shape.border_fill_id);
        }
    }
    refs
}

fn collect_object_border_fill_refs(doc: &Document) -> std::collections::HashSet<u16> {
    let mut refs = std::collections::HashSet::new();
    for section in &doc.sections {
        if section.section_def.page_border_fill.border_fill_id > 0 {
            refs.insert(section.section_def.page_border_fill.border_fill_id);
        }
        for page_border_fill in &section.section_def.extra_page_border_fills {
            if page_border_fill.border_fill_id > 0 {
                refs.insert(page_border_fill.border_fill_id);
            }
        }
        for para in &section.paragraphs {
            collect_object_border_fill_refs_from_paragraph(para, &mut refs);
        }
    }
    refs
}

fn collect_object_border_fill_refs_from_paragraph(
    para: &Paragraph,
    refs: &mut std::collections::HashSet<u16>,
) {
    for ctrl in &para.controls {
        match ctrl {
            Control::Table(table) => collect_table_border_fill_refs(table, refs),
            Control::Shape(shape) => collect_object_border_fill_refs_from_shape(shape, refs),
            _ => {}
        }
    }
}

fn collect_object_border_fill_refs_from_shape(
    shape: &ShapeObject,
    refs: &mut std::collections::HashSet<u16>,
) {
    if let Some(drawing) = shape.drawing() {
        if let Some(text_box) = &drawing.text_box {
            for para in &text_box.paragraphs {
                collect_object_border_fill_refs_from_paragraph(para, refs);
            }
        }
    }

    if let ShapeObject::Group(group) = shape {
        for child in &group.children {
            collect_object_border_fill_refs_from_shape(child, refs);
        }
    }
}

fn collect_table_border_fill_refs(table: &Table, refs: &mut std::collections::HashSet<u16>) {
    if table.border_fill_id > 0 {
        refs.insert(table.border_fill_id);
    }
    for zone in &table.zones {
        if zone.border_fill_id > 0 {
            refs.insert(zone.border_fill_id);
        }
    }
    for cell in &table.cells {
        if cell.border_fill_id > 0 {
            refs.insert(cell.border_fill_id);
        }
        for para in &cell.paragraphs {
            collect_object_border_fill_refs_from_paragraph(para, refs);
        }
    }
}

fn is_transparent_paragraph_no_fill_candidate(border_fill: &BorderFill) -> bool {
    if !border_fill
        .borders
        .iter()
        .all(|border| matches!(border.line_type, BorderLineType::None))
    {
        return false;
    }

    if !matches!(border_fill.fill.fill_type, FillType::Solid) {
        return false;
    }

    let Some(solid) = border_fill.fill.solid else {
        return false;
    };

    border_fill.fill.alpha == 0 && solid.background_color == 0xffff_ffff
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParagraphContext {
    Body,
    HeaderFooter,
    MasterPage,
}

fn adapt_paragraph(para: &mut Paragraph, report: &mut AdapterReport) {
    adapt_paragraph_with_context(para, report, ParagraphContext::Body);
}

fn adapt_paragraph_with_context(
    para: &mut Paragraph,
    report: &mut AdapterReport,
    context: ParagraphContext,
) {
    materialize_master_page_autonum_placeholder(para, report, context);
    materialize_autonum_fwspace_range_tag(para, report);
    materialize_autonum_fwspace_char_shape_offsets(para, report);
    materialize_fixed_width_space_control(para, report, context);
    materialize_para_header_tail(para, report);

    if para.ctrl_data_records.len() < para.controls.len() {
        para.ctrl_data_records
            .resize_with(para.controls.len(), || None);
    }

    let controls = &mut para.controls;
    let ctrl_data_records = &mut para.ctrl_data_records;
    for (idx, ctrl) in controls.iter_mut().enumerate() {
        match ctrl {
            Control::Table(table) => {
                adapt_table_with_context(table, report, context);
                adapt_table_layout_ctrl_data(table, &mut ctrl_data_records[idx], report);
            }
            Control::Header(header) => adapt_paragraphs_with_context(
                &mut header.paragraphs,
                report,
                ParagraphContext::HeaderFooter,
            ),
            Control::Footer(footer) => adapt_paragraphs_with_context(
                &mut footer.paragraphs,
                report,
                ParagraphContext::HeaderFooter,
            ),
            Control::Picture(pic) => {
                adapt_picture_href_ctrl_data(pic, &mut ctrl_data_records[idx], report)
            }
            Control::Shape(shape) => adapt_shape_with_context(shape, report, context),
            Control::Equation(eq) => adapt_equation(eq, report),
            _ => {}
        }
    }
}

fn materialize_autonum_fwspace_range_tag(para: &mut Paragraph, report: &mut AdapterReport) {
    const HWP5_AUTONUM_FWSPACE_TRAILING_TAG: u32 = 0x0100_0023;

    if para
        .range_tags
        .iter()
        .any(|tag| tag.tag == HWP5_AUTONUM_FWSPACE_TRAILING_TAG)
    {
        return;
    }

    if !para
        .controls
        .iter()
        .any(|ctrl| matches!(ctrl, Control::AutoNumber(_)))
    {
        return;
    }

    let mut chars = para.text.chars();
    if chars.next() != Some(' ') || chars.next() != Some('\u{2007}') {
        return;
    }

    if para.text.chars().count() <= 2 || para.char_offsets.len() != para.text.chars().count() {
        return;
    }

    let Some(last_char) = para.text.chars().last() else {
        return;
    };
    let Some(&start) = para.char_offsets.last() else {
        return;
    };

    let end = start + last_char.len_utf16() as u32;
    if end <= start {
        return;
    }

    // Hancom's HWPX->HWP save materializes an otherwise implicit range tag on
    // the final visible character in paragraphs shaped as:
    //   AutoNumber placeholder + fixed-width space + visible heading text.
    //
    // Without this tag the file is structurally readable by rhwp, but Hancom
    // reports corruption around the first AutoNumber/PageHide boundary in
    // exam_social.hwpx.
    para.range_tags.push(crate::model::paragraph::RangeTag {
        start,
        end,
        tag: HWP5_AUTONUM_FWSPACE_TRAILING_TAG,
    });
    report.autonum_fwspace_range_tag_materialized += 1;
}

fn materialize_autonum_fwspace_char_shape_offsets(
    para: &mut Paragraph,
    report: &mut AdapterReport,
) {
    if !para
        .controls
        .iter()
        .any(|ctrl| matches!(ctrl, Control::AutoNumber(_)))
    {
        return;
    }

    if !para.text.starts_with(" \u{2007}") {
        return;
    }

    if !para
        .char_shapes
        .iter()
        .any(|char_shape| (2..9).contains(&char_shape.start_pos))
    {
        return;
    }

    // HWPX positions are based on logical characters:
    //   placeholder space(1) + fixed-width space(1) + visible text...
    // HWP5 stores the AutoNumber as an 8-code-unit extended control and the
    // fixed-width space as 0x001f, so style boundaries after the placeholder
    // move by +7 code units. This matches Hancom's HWPX->HWP output.
    for char_shape in &mut para.char_shapes {
        if char_shape.start_pos >= 2 {
            char_shape.start_pos += 7;
        }
    }
    report.autonum_fwspace_char_shape_offsets_materialized += 1;
}

fn adapt_paragraphs(paragraphs: &mut [Paragraph], report: &mut AdapterReport) {
    adapt_paragraphs_with_context(paragraphs, report, ParagraphContext::Body);
}

fn adapt_paragraphs_with_context(
    paragraphs: &mut [Paragraph],
    report: &mut AdapterReport,
    context: ParagraphContext,
) {
    for para in paragraphs {
        adapt_paragraph_with_context(para, report, context);
    }
}

fn materialize_fixed_width_space_control(
    para: &mut Paragraph,
    report: &mut AdapterReport,
    context: ParagraphContext,
) {
    const HWP5_FIXED_WIDTH_SPACE_MASK: u32 = 1u32 << 0x001f;

    if context != ParagraphContext::Body || !para.text.contains('\u{2007}') {
        return;
    }

    if para.control_mask & HWP5_FIXED_WIDTH_SPACE_MASK != 0 {
        return;
    }

    // Hancom's HWPX->HWP path stores body fixed-width blanks as HWP5
    // control char 0x001f in the affected exam_social body paragraphs.
    // Header/footer and master page paragraphs keep literal U+2007 because
    // page-number placeholder replacement depends on that visible spacer.
    para.control_mask |= HWP5_FIXED_WIDTH_SPACE_MASK;
    report.header_footer_fwspace_control_materialized += 1;
}

fn materialize_para_header_tail(para: &mut Paragraph, report: &mut AdapterReport) {
    if para.raw_header_extra.len() >= 12 {
        return;
    }

    if para.raw_header_extra.len() >= 10 {
        para.raw_header_extra.resize(12, 0);
    } else {
        let mut extra = vec![0; 12];
        let char_shape_count = para.char_shapes.len().max(1).min(u16::MAX as usize) as u16;
        let range_tag_count = para.range_tags.len().min(u16::MAX as usize) as u16;
        let line_seg_count = para.line_segs.len().min(u16::MAX as usize) as u16;

        extra[0..2].copy_from_slice(&char_shape_count.to_le_bytes());
        extra[2..4].copy_from_slice(&range_tag_count.to_le_bytes());
        extra[4..6].copy_from_slice(&line_seg_count.to_le_bytes());

        para.raw_header_extra = extra;
    }

    report.para_header_tail_materialized += 1;
}

fn adapt_section_def(section_def: &mut SectionDef, report: &mut AdapterReport) {
    materialize_section_def_hide_empty_line_flag(section_def, report);
    materialize_single_master_page_flags(section_def, report);
    materialize_multi_master_page_flags(section_def, report);
    materialize_section_def_master_page_tail(section_def, report);

    for master_page in &mut section_def.master_pages {
        adapt_paragraphs_with_context(
            &mut master_page.paragraphs,
            report,
            ParagraphContext::MasterPage,
        );
    }
}

fn materialize_section_def_hide_empty_line_flag(
    section_def: &mut SectionDef,
    report: &mut AdapterReport,
) {
    const HIDE_EMPTY_LINE_FLAG: u32 = 0x0008_0000;

    let old_flags = section_def.flags;
    if section_def.hide_empty_line {
        section_def.flags |= HIDE_EMPTY_LINE_FLAG;
    } else {
        section_def.flags &= !HIDE_EMPTY_LINE_FLAG;
    }

    if section_def.flags != old_flags {
        report.section_def_hide_empty_line_flag_materialized += 1;
    }
}

fn materialize_master_page_autonum_placeholder(
    para: &mut Paragraph,
    report: &mut AdapterReport,
    context: ParagraphContext,
) {
    // [Task #1113] 바탕쪽(MasterPage) 뿐 아니라 머리말/꼬리말(HeaderFooter) 글상자
    // 안의 AutoNumber-only 페이지번호 문단도 동일하게 처리한다.
    if !matches!(
        context,
        ParagraphContext::MasterPage | ParagraphContext::HeaderFooter
    ) {
        return;
    }

    if para.text != " "
        || para.char_offsets.as_slice() != [0]
        || para.controls.len() != 1
        || !matches!(para.controls.first(), Some(Control::AutoNumber(_)))
    {
        return;
    }

    // HWPX emits an empty <hp:t/> after PAGE AutoNumber controls. The generic
    // HWPX parser synthesizes a visible placeholder space (U+0020) for the
    // AutoNumber, but Hancom's HWPX->HWP save stores the page-number paragraph
    // as AutoNumber-only: no leading U+0020 before the control.
    //
    // [Task #1113] 머리말 홀수쪽 글상자(폭 4252)에서 이 잉여 U+0020 이 한컴
    // 에디터의 페이지번호 줄나눔/글상자 높이 증가를 유발. 정답지처럼
    // AutoNumber-only 로 정규화한다. (짝수쪽은 fwSpace+텍스트+autoNum 이라
    // `text != " "` 에서 자동 제외 → 회귀 없음)
    para.text.clear();
    para.char_offsets.clear();
    para.char_count = 9;
    para.has_para_text = true;
    report.master_page_autonum_placeholder_removed += 1;
}

fn materialize_single_master_page_flags(section_def: &mut SectionDef, report: &mut AdapterReport) {
    const HWPX_SINGLE_MASTER_PAGE_FLAGS: u32 = 0x4000_0000;
    const HANCOM_SINGLE_MASTER_PAGE_FLAGS: u32 = 0x2000_0000;
    const MASTER_PAGE_FLAGS_MASK: u32 = 0xe000_0000;

    if section_def.master_pages.len() != 1
        || section_def.flags & MASTER_PAGE_FLAGS_MASK != HWPX_SINGLE_MASTER_PAGE_FLAGS
    {
        return;
    }

    section_def.flags =
        (section_def.flags & !MASTER_PAGE_FLAGS_MASK) | HANCOM_SINGLE_MASTER_PAGE_FLAGS;
    report.section_def_single_master_page_flags_materialized += 1;
}

fn materialize_multi_master_page_flags(section_def: &mut SectionDef, report: &mut AdapterReport) {
    const HWPX_TWO_MASTER_PAGE_FLAGS: u32 = 0x8000_0000;
    const HANCOM_MULTI_MASTER_PAGE_FLAGS: u32 = 0xC000_0000;
    const MASTER_PAGE_FLAGS_MASK: u32 = 0xe000_0000;

    if section_def.master_pages.len() < 2
        || section_def.flags & MASTER_PAGE_FLAGS_MASK != HWPX_TWO_MASTER_PAGE_FLAGS
    {
        return;
    }

    section_def.flags =
        (section_def.flags & !MASTER_PAGE_FLAGS_MASK) | HANCOM_MULTI_MASTER_PAGE_FLAGS;
    report.section_def_multi_master_page_flags_materialized += 1;
}

fn materialize_section_def_master_page_tail(
    section_def: &mut SectionDef,
    report: &mut AdapterReport,
) {
    if section_def.master_pages.is_empty() || !section_def.raw_ctrl_extra.is_empty() {
        return;
    }

    // HWPX 출처 SectionDef는 HWP 원본 CTRL_HEADER tail이 없지만, 한컴이 HWPX를
    // HWP5로 저장한 정답지는 바탕쪽이 있는 구역에서 대표Language(0) 뒤에
    // 17 byte 확장 영역을 붙여 총 43 byte ctrl_data (CTRL_HEADER 47 byte)를 만든다.
    //
    // 관찰된 계약:
    // - exam_kor: masterPageCnt=3 -> 0x0001 marker + 15 byte zero
    // - exam_social-p1: 단일 Both 바탕쪽 -> 17 byte zero
    // - exam_social section1: Both + Odd 2개 바탕쪽 -> 17 byte zero
    let mut extra = vec![0; 19];
    extra[0..2].copy_from_slice(&0u16.to_le_bytes());
    if section_def.master_pages.len() >= 3 {
        extra[2..4].copy_from_slice(&1u16.to_le_bytes());
    }
    section_def.raw_ctrl_extra = extra;
    report.section_def_master_page_tail_materialized += 1;
}

/// [Task #1061] HWPX 수식 control 의 한컴 호환 contract 정정.
///
/// 정답지 (samples/math-001.hwp) vs 저장본 (saved/111math-001.hwp) record-level diff:
/// - common.attr 의 bit 27 (0x08000000) 누락 — 정답지 0x0C2A2211 vs 저장본 0x042A2211
/// - HWPX 의 `font` 속성을 HWP5 EQEDIT 의 font_name 자리에 매핑한 결과 정답지와 자리값 swap
///   → Stage 2 에서 parser 직접 정정 (정답지: version_info="Equation Version 60", font_name="")
///
/// 본 함수는 Stage 1 의 attr 재구성 (enum 필드 → bit 합성 + bit 27 보강) + raw_ctrl_data clear
/// (직렬화기가 common 으로 재합성).
fn adapt_equation(eq: &mut crate::model::control::Equation, report: &mut AdapterReport) {
    const HWPX_EQUATION_NUMBERING_BIT: u32 = 0x0800_0000;

    let before = eq.common.attr;
    // HWPX 출처는 attr=0 으로 IR 생성 → pack_common_attr_bits 로 enum 필드들에서 재합성 후
    // bit 27 보강. 표 어댑터 (materialize_table_ctrl_header_attr) 와 동일 패턴.
    eq.common.attr = pack_common_attr_bits(&eq.common) | HWPX_EQUATION_NUMBERING_BIT;

    // raw_ctrl_data 가 보존되어 있으면 직렬화기가 raw 우선 사용 → attr 갱신 무효화.
    // clear 하여 직렬화기가 common 으로 재합성하도록 함.
    let raw_was_present = !eq.raw_ctrl_data.is_empty();
    eq.raw_ctrl_data.clear();

    if eq.common.attr != before || raw_was_present {
        report.equation_ctrl_header_attr_materialized += 1;
    }
}

fn adapt_shape(shape: &mut ShapeObject, report: &mut AdapterReport) {
    adapt_shape_with_context(shape, report, ParagraphContext::Body);
}

fn adapt_shape_with_context(
    shape: &mut ShapeObject,
    report: &mut AdapterReport,
    context: ParagraphContext,
) {
    if context == ParagraphContext::MasterPage {
        if let ShapeObject::Line(line) = shape {
            materialize_master_page_line_rendering_size_ratio(line, report);
        }
    }

    if let Some(drawing) = shape.drawing_mut() {
        if let Some(text_box) = &mut drawing.text_box {
            materialize_text_box_hwp5_envelope(text_box, report);
            adapt_paragraphs_with_context(&mut text_box.paragraphs, report, context);
        }
    }

    if let ShapeObject::Group(group) = shape {
        for child in &mut group.children {
            adapt_shape_with_context(child, report, context);
        }
    }
}

fn materialize_master_page_line_rendering_size_ratio(
    line: &mut crate::model::shape::LineShape,
    report: &mut AdapterReport,
) {
    const COUNT_SIZE: usize = 2;
    const MATRIX_SIZE: usize = 6 * 8;
    const SCALE_START: usize = COUNT_SIZE + MATRIX_SIZE;
    const ROTATION_START: usize = SCALE_START + MATRIX_SIZE;
    const MIN_RAW_LEN: usize = ROTATION_START + MATRIX_SIZE;
    const EPSILON: f64 = 0.01;

    let attr = &mut line.drawing.shape_attr;
    if attr.raw_rendering.len() < MIN_RAW_LEN
        || attr.original_width == 0
        || attr.original_height == 0
    {
        return;
    }

    let count = u16::from_le_bytes([attr.raw_rendering[0], attr.raw_rendering[1]]);
    if count == 0 {
        return;
    }

    let exact_sx = attr.current_width as f64 / attr.original_width as f64;
    let exact_sy = attr.current_height as f64 / attr.original_height as f64;
    let Some(raw_sx) = read_raw_rendering_f64(&attr.raw_rendering, SCALE_START) else {
        return;
    };
    let Some(raw_sy) = read_raw_rendering_f64(&attr.raw_rendering, SCALE_START + 4 * 8) else {
        return;
    };

    if (raw_sx - exact_sx).abs() > EPSILON || (raw_sy - exact_sy).abs() > EPSILON {
        return;
    }

    let mut changed = false;
    changed |= write_raw_rendering_f64(&mut attr.raw_rendering, SCALE_START, exact_sx);
    changed |= write_raw_rendering_f64(&mut attr.raw_rendering, SCALE_START + 4 * 8, exact_sy);

    if matches!(
        read_raw_rendering_f64(&attr.raw_rendering, ROTATION_START + 8),
        Some(value) if value == 0.0
    ) {
        changed |= write_raw_rendering_f64(&mut attr.raw_rendering, ROTATION_START + 8, -0.0);
    }

    if changed {
        attr.render_sx = exact_sx;
        attr.render_sy = exact_sy;
        report.master_page_line_rendering_size_ratio_materialized += 1;
    }
}

fn read_raw_rendering_f64(raw: &[u8], offset: usize) -> Option<f64> {
    let bytes = raw.get(offset..offset + 8)?;
    Some(f64::from_le_bytes(bytes.try_into().ok()?))
}

fn write_raw_rendering_f64(raw: &mut [u8], offset: usize, value: f64) -> bool {
    let Some(target) = raw.get_mut(offset..offset + 8) else {
        return false;
    };
    let bytes = value.to_le_bytes();
    if target == bytes {
        return false;
    }
    target.copy_from_slice(&bytes);
    true
}

fn materialize_text_box_hwp5_envelope(text_box: &mut TextBox, report: &mut AdapterReport) {
    if !is_draw_text_hwp5_envelope_candidate(text_box) {
        return;
    }

    if text_box.raw_list_header_extra.is_empty() {
        text_box.raw_list_header_extra = vec![0; 13];
        report.text_box_list_header_tail_materialized += 1;
    }

    for para in &mut text_box.paragraphs {
        if para.raw_header_extra.len() >= 12 {
            continue;
        }

        let mut extra = vec![0; 12];
        let char_shape_count = para.char_shapes.len().max(1).min(u16::MAX as usize) as u16;
        let range_tag_count = para.range_tags.len().min(u16::MAX as usize) as u16;
        let line_seg_count = para.line_segs.len().min(u16::MAX as usize) as u16;

        extra[0..2].copy_from_slice(&char_shape_count.to_le_bytes());
        extra[2..4].copy_from_slice(&range_tag_count.to_le_bytes());
        extra[4..6].copy_from_slice(&line_seg_count.to_le_bytes());
        extra[6..10].copy_from_slice(&0x8000_0000_u32.to_le_bytes());

        para.raw_header_extra = extra;
        report.text_box_para_header_tail_materialized += 1;
    }
}

fn is_draw_text_hwp5_envelope_candidate(text_box: &TextBox) -> bool {
    text_box.paragraphs.iter().any(|para| {
        para.controls
            .iter()
            .any(|control| matches!(control, Control::Picture(_)))
    })
}

fn adapt_picture_href_ctrl_data(
    pic: &Picture,
    ctrl_data_slot: &mut Option<Vec<u8>>,
    report: &mut AdapterReport,
) {
    let Some(href) = pic.href.as_deref().filter(|value| !value.is_empty()) else {
        return;
    };

    let ctrl_data = build_picture_href_ctrl_data(href);
    if ctrl_data_slot.as_deref() == Some(ctrl_data.as_slice()) {
        return;
    }

    *ctrl_data_slot = Some(ctrl_data);
    report.picture_href_ctrl_data_materialized += 1;
}

fn build_picture_href_ctrl_data(href: &str) -> Vec<u8> {
    let hwp_href = normalize_picture_href_for_hwp_ctrl_data(href);
    let utf16: Vec<u16> = hwp_href.encode_utf16().collect();

    let mut data = Vec::with_capacity(22 + utf16.len() * 2);
    data.extend_from_slice(&0x021b_u16.to_le_bytes());
    data.extend_from_slice(&1_u16.to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());
    data.extend_from_slice(&0x026f_u16.to_le_bytes());
    data.extend_from_slice(&0x8000_u16.to_le_bytes());
    data.extend_from_slice(&0x026f_u16.to_le_bytes());
    data.extend_from_slice(&1_u16.to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());
    data.extend_from_slice(&0x0265_u16.to_le_bytes());
    data.extend_from_slice(&0x0001_u16.to_le_bytes());
    data.extend_from_slice(&(utf16.len().min(u16::MAX as usize) as u16).to_le_bytes());
    for ch in utf16.into_iter().take(u16::MAX as usize) {
        data.extend_from_slice(&ch.to_le_bytes());
    }
    data
}

fn normalize_picture_href_for_hwp_ctrl_data(href: &str) -> String {
    if href.contains("\\://") {
        href.to_string()
    } else {
        href.replace("://", "\\://")
    }
}

fn adapt_table_layout_ctrl_data(
    table: &Table,
    ctrl_data_slot: &mut Option<Vec<u8>>,
    report: &mut AdapterReport,
) {
    if ctrl_data_slot.is_some() || !table_requires_layout_ctrl_data(table) {
        return;
    }

    *ctrl_data_slot = Some(build_table_layout_ctrl_data());
    report.table_layout_ctrl_data_materialized += 1;
}

fn table_requires_layout_ctrl_data(table: &Table) -> bool {
    table.row_count == 3
        && table.col_count == 2
        && table.repeat_header
        && matches!(table.page_break, TablePageBreak::RowBreak)
}

fn build_table_layout_ctrl_data() -> Vec<u8> {
    // 한컴 HWPX→HWP 변환본에서 3x2 선택지 표 뒤에 붙는 Table CTRL_DATA.
    // 공식 5.0 문서에는 의미가 정리되어 있지 않지만, #1064/#1099 정답지 모두
    // 같은 104바이트 ParameterSet(0x021b → 0x0242)을 사용한다.
    const VALUES: [u32; 11] = [3826, 1048, 28346, 8475, 708, 0, 2, 9, 0, 59528, 84188];

    let mut data = Vec::with_capacity(104);
    data.extend_from_slice(&0x021b_u16.to_le_bytes());
    data.extend_from_slice(&1_u16.to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());
    data.extend_from_slice(&0x0242_u16.to_le_bytes());
    data.extend_from_slice(&0x8000_u16.to_le_bytes());
    data.extend_from_slice(&0x0242_u16.to_le_bytes());
    data.extend_from_slice(&(VALUES.len() as u16).to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());
    for (idx, value) in VALUES.iter().enumerate() {
        data.extend_from_slice(&(0x4000_u16 + idx as u16).to_le_bytes());
        data.extend_from_slice(&0x0004_u16.to_le_bytes());
        data.extend_from_slice(&value.to_le_bytes());
    }
    data
}

fn adapt_table(table: &mut Table, report: &mut AdapterReport) {
    adapt_table_with_context(table, report, ParagraphContext::Body);
}

fn adapt_table_with_context(
    table: &mut Table,
    report: &mut AdapterReport,
    context: ParagraphContext,
) {
    // 1. raw_ctrl_data 합성 (HWPX 출처는 비어있음)
    if table.raw_ctrl_data.is_empty() {
        materialize_table_outer_margin(table, report);
        materialize_table_record_attr(table, report);
        materialize_table_record_row_sizes(table, report);
        materialize_table_record_extra(table, report);
        materialize_table_ctrl_header_attr(table, report);

        table.raw_ctrl_data = serialize_common_obj_attr(&table.common);
        report.tables_ctrl_data_synthesized += 1;

        if table.raw_ctrl_data.len() >= common_obj_offsets::FLAGS.end {
            let packed = u32::from_le_bytes(
                table.raw_ctrl_data[common_obj_offsets::FLAGS]
                    .try_into()
                    .unwrap(),
            );
            if table.attr != packed {
                table.attr = packed;
                report.tables_attr_packed += 1;
            }
        }
    }

    // 셀별 보강 + 내부 문단 재귀 (중첩 표 대응)
    let use_cell_width_ref = table_requires_cell_width_ref_contract(table);
    let table_padding = table.padding;
    for cell in &mut table.cells {
        adapt_cell_list_attr(cell, report);
        materialize_cell_list_header_contract(cell, use_cell_width_ref, &table_padding, report);
        for cpara in &mut cell.paragraphs {
            adapt_paragraph_with_context(cpara, report, context);
        }
    }
}

fn table_requires_cell_width_ref_contract(table: &Table) -> bool {
    // HWPX 조직도류 표는 많은 논리 열로 셀 폭을 쪼개어 만든 micro-grid 형태다.
    // 이 계열은 LIST_HEADER width_ref bit가 없으면 한컴이 셀 내부 줄나눔 폭을 너무 좁게 잡는다.
    //
    // 반대로 mel-001의 8x12 인원 현황 표는 같은 bit를 세우면 한컴이 병합 셀 높이를 과도하게
    // 계산했다. 따라서 raw_list_extra는 모든 셀에 materialize하되 width_ref bit는
    // 고열 수 micro-grid 표에만 적용한다.
    table.col_count >= 30
}

fn materialize_cell_list_header_contract(
    cell: &mut Cell,
    use_width_ref: bool,
    table_padding: &crate::model::Padding,
    report: &mut AdapterReport,
) {
    let before_width_ref = cell.list_header_width_ref;
    let before_extra_len = cell.raw_list_extra.len();

    // [#1809] micro-grid 계약으로 width_ref bit0(=aim)을 켤 때, aim=false 셀의
    // 유효 안 여백(effective_padding — 표 기본 폴백 포함)을 셀 padding 에 물질화한다.
    // 재파싱 시 aim=true 가 되면 측정/레이아웃의 aim=true 원값 존중 경로(#493 시멘틱)가
    // raw cell padding 을 그대로 쓰므로, 물질화 없이는 padding 0 셀의 행높이가
    // 원본(HWPX, 표 기본 여백)과 어긋난다 (admrul_0296 행 32.37→31.60, 표 3.87px).
    if use_width_ref && !cell.apply_inner_margin {
        cell.padding = cell.effective_padding(table_padding);
    }

    if use_width_ref || cell.apply_inner_margin {
        cell.list_header_width_ref |= 0x0001;
    } else {
        cell.list_header_width_ref &= !0x0001;
    }

    if cell.raw_list_extra.is_empty() {
        let mut extra = vec![0u8; 13];
        extra[0..4].copy_from_slice(&cell.width.to_le_bytes());
        cell.raw_list_extra = extra;
    }

    if cell.list_header_width_ref != before_width_ref
        || cell.raw_list_extra.len() != before_extra_len
    {
        report.cells_list_header_contract_materialized += 1;
    }
}

fn materialize_table_outer_margin(table: &mut Table, report: &mut AdapterReport) {
    let changed = table.common.margin.left != table.outer_margin_left
        || table.common.margin.right != table.outer_margin_right
        || table.common.margin.top != table.outer_margin_top
        || table.common.margin.bottom != table.outer_margin_bottom;
    if changed {
        table.common.margin.left = table.outer_margin_left;
        table.common.margin.right = table.outer_margin_right;
        table.common.margin.top = table.outer_margin_top;
        table.common.margin.bottom = table.outer_margin_bottom;
        report.tables_outer_margin_materialized += 1;
    }
}

fn materialize_table_record_attr(table: &mut Table, report: &mut AdapterReport) {
    let mut attr = match table.page_break {
        TablePageBreak::CellBreak => 0x01,
        TablePageBreak::RowBreak => 0x02,
        TablePageBreak::None => 0,
    };
    if table.repeat_header {
        attr |= 0x04;
    }
    if (table.attr | table.raw_table_record_attr) & 0x08 != 0 {
        attr |= 0x08;
    }
    // HWPX inMargin 값만 쓰면 한컴 에디터의 "셀 안쪽 여백 지정"이 꺼진
    // 상태로 저장된다. HWP5 TABLE attr bit 26을 함께 켜야 해당 여백이
    // 조판 계약에 참여한다. 공식 5.0 문서에는 이 상위 비트가 명시되어
    // 있지 않아 aift 정답 HWP와의 교차 검증으로 보존한다.
    if table.padding.left != 0
        || table.padding.right != 0
        || table.padding.top != 0
        || table.padding.bottom != 0
    {
        attr |= 0x0400_0000;
    }

    if table.raw_table_record_attr != attr {
        table.raw_table_record_attr = attr;
        report.table_record_attr_materialized += 1;
    }
}

fn materialize_table_record_row_sizes(table: &mut Table, report: &mut AdapterReport) {
    let mut row_sizes = vec![0i16; table.row_count as usize];
    for cell in &table.cells {
        let row = cell.row as usize;
        if row < row_sizes.len() {
            row_sizes[row] = row_sizes[row].saturating_add(1);
        }
    }

    if row_sizes.is_empty() || row_sizes.iter().all(|&count| count == 0) {
        return;
    }

    if table.row_sizes != row_sizes {
        table.row_sizes = row_sizes;
        report.table_record_row_sizes_materialized += 1;
    }
}

fn materialize_table_record_extra(table: &mut Table, report: &mut AdapterReport) {
    if table.raw_table_record_extra.is_empty() {
        table.raw_table_record_extra = vec![0, 0];
        report.table_record_extra_materialized += 1;
    }
}

fn materialize_table_ctrl_header_attr(table: &mut Table, report: &mut AdapterReport) {
    const HWPX_TABLE_FLOW_WITH_TEXT_BIT: u32 = 0x0000_2000;
    const HWPX_TABLE_NUMBERING_BIT: u32 = 0x0800_0000;
    const HWP5_TABLE_CAPTION_COMMON_ATTR_BIT: u32 = 0x2000_0000;

    let before = table.common.attr;
    let mut attr = pack_common_attr_bits(&table.common)
        | HWPX_TABLE_FLOW_WITH_TEXT_BIT
        | HWPX_TABLE_NUMBERING_BIT;
    if table.caption.is_some() {
        attr |= HWP5_TABLE_CAPTION_COMMON_ATTR_BIT;
    }
    table.common.attr = attr;

    if table.common.attr != before {
        report.table_ctrl_header_attr_materialized += 1;
    }
}

/// 셀 `apply_inner_margin` → LIST_HEADER width_ref bit 0 합성 (Stage 3, 보수적).
///
/// ## 배경
///
/// `serializer/control.rs` 가 작성하는 셀 LIST_HEADER 의 앞 8바이트:
/// ```text
/// n_para: u16
/// list_attr: u32
/// width_ref/property: u16
/// ```
///
/// HWPX 출처 셀에서 `apply_inner_margin = true` 인 경우, 직렬화 시 `width_ref bit 0` 이
/// 0 으로 떨어지면 한컴이 셀 안 여백을 표 기본값으로 대체한다.
///
/// ## 합성 방식
///
/// `apply_inner_margin == true`인 경우 `list_header_width_ref |= 0x0001`을 적용한다.
fn adapt_cell_list_attr(cell: &mut Cell, report: &mut AdapterReport) {
    if cell.apply_inner_margin && cell.list_header_width_ref & 0x0001 == 0 {
        cell.list_header_width_ref |= 0x0001;
        report.cells_list_attr_bit16_set += 1;
    }
}

/// `source_format` 검사 후 어댑터를 호출하는 보조 함수.
///
/// 호출자: `DocumentCore::export_hwp_with_adapter()` (Stage 5 에서 추가).
pub fn convert_if_hwpx_source(doc: &mut Document, source_format: FileFormat) -> AdapterReport {
    if !matches!(source_format, FileFormat::Hwpx | FileFormat::Hwp3) {
        return AdapterReport::new().no_op("source_format != Hwpx/Hwp3");
    }
    convert_hwpx_to_hwp_ir(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::CharShapeRef;

    #[test]
    fn empty_doc_normalizes_file_header_once() {
        let mut doc = Document::default();
        let report = convert_hwpx_to_hwp_ir(&mut doc);
        assert!(report.changed_anything());
        assert!(report.skipped_reason.is_none());
        assert_eq!(report.file_header_compression_normalized, 1);
        assert!(doc.header.compressed);
        assert_eq!(doc.header.flags & 0x01, 0x01);
        assert!(doc.header.raw_data.is_none());
    }

    #[test]
    fn hwp_source_no_op_via_filter() {
        let mut doc = Document::default();
        let report = convert_if_hwpx_source(&mut doc, FileFormat::Hwp);
        assert_eq!(
            report.skipped_reason.as_deref(),
            Some("source_format != Hwpx/Hwp3")
        );
    }

    #[test]
    fn table_axis_materializes_hancom_record_contract() {
        use crate::model::shape::{CommonObjAttr, HorzRelTo, TextWrap, VertRelTo};
        use crate::model::Padding;

        let mut table = Table {
            row_count: 1,
            col_count: 3,
            padding: Padding {
                left: 141,
                right: 141,
                top: 141,
                bottom: 141,
            },
            cells: (0..3)
                .map(|col| Cell {
                    col,
                    row: 0,
                    col_span: 1,
                    row_span: 1,
                    ..Default::default()
                })
                .collect(),
            page_break: TablePageBreak::RowBreak,
            repeat_header: true,
            attr: 0x08,
            common: CommonObjAttr {
                treat_as_char: true,
                text_wrap: TextWrap::TopAndBottom,
                vert_rel_to: VertRelTo::Para,
                horz_rel_to: HorzRelTo::Para,
                width: 47697,
                height: 3525,
                z_order: 26,
                ..Default::default()
            },
            outer_margin_left: 283,
            outer_margin_right: 283,
            outer_margin_top: 283,
            outer_margin_bottom: 283,
            border_fill_id: 3,
            ..Default::default()
        };

        let mut report = AdapterReport::new();
        adapt_table(&mut table, &mut report);

        assert_eq!(table.raw_table_record_attr, 0x0400_000e);
        assert_eq!(table.row_sizes, vec![3]);
        assert_eq!(table.raw_table_record_extra, vec![0, 0]);
        assert_eq!(
            u32::from_le_bytes(
                table.raw_ctrl_data[common_obj_offsets::FLAGS]
                    .try_into()
                    .unwrap(),
            ),
            0x082a_2311
        );
        assert_eq!(
            (
                i16::from_le_bytes(
                    table.raw_ctrl_data[common_obj_offsets::MARGIN_LEFT]
                        .try_into()
                        .unwrap(),
                ),
                i16::from_le_bytes(
                    table.raw_ctrl_data[common_obj_offsets::MARGIN_RIGHT]
                        .try_into()
                        .unwrap(),
                ),
                i16::from_le_bytes(
                    table.raw_ctrl_data[common_obj_offsets::MARGIN_TOP]
                        .try_into()
                        .unwrap(),
                ),
                i16::from_le_bytes(
                    table.raw_ctrl_data[common_obj_offsets::MARGIN_BOTTOM]
                        .try_into()
                        .unwrap(),
                ),
            ),
            (283, 283, 283, 283)
        );
    }

    #[test]
    fn table_break_materializes_hwp5_cell_break_bit() {
        let mut table = Table {
            row_count: 1,
            col_count: 1,
            cells: vec![Cell {
                col: 0,
                row: 0,
                col_span: 1,
                row_span: 1,
                ..Default::default()
            }],
            page_break: TablePageBreak::CellBreak,
            repeat_header: true,
            ..Default::default()
        };

        let mut report = AdapterReport::new();
        adapt_table(&mut table, &mut report);

        assert_eq!(table.raw_table_record_attr & 0x03, 0x01);
        assert_eq!(table.raw_table_record_attr & 0x04, 0x04);
    }

    #[test]
    fn captioned_table_materializes_hancom_caption_common_attr_bit() {
        use crate::model::shape::{
            Caption, CaptionDirection, CommonObjAttr, HorzRelTo, TextWrap, VertRelTo,
        };
        use crate::model::Padding;

        let mut table = Table {
            row_count: 12,
            col_count: 5,
            padding: Padding {
                left: 141,
                right: 141,
                top: 141,
                bottom: 141,
            },
            cells: (0..5)
                .map(|col| Cell {
                    col,
                    row: 0,
                    col_span: 1,
                    row_span: 1,
                    ..Default::default()
                })
                .collect(),
            page_break: TablePageBreak::CellBreak,
            repeat_header: true,
            attr: 0x08,
            common: CommonObjAttr {
                treat_as_char: true,
                text_wrap: TextWrap::TopAndBottom,
                vert_rel_to: VertRelTo::Para,
                horz_rel_to: HorzRelTo::Para,
                width: 47152,
                height: 14976,
                z_order: 6,
                ..Default::default()
            },
            outer_margin_left: 141,
            outer_margin_right: 141,
            outer_margin_top: 141,
            outer_margin_bottom: 141,
            border_fill_id: 97,
            caption: Some(Caption {
                direction: CaptionDirection::Top,
                width: 8504,
                spacing: 283,
                max_width: 47152,
                paragraphs: vec![Paragraph::default()],
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut report = AdapterReport::new();
        adapt_table(&mut table, &mut report);

        assert_eq!(
            u32::from_le_bytes(
                table.raw_ctrl_data[common_obj_offsets::FLAGS]
                    .try_into()
                    .unwrap(),
            ),
            0x282a_2311
        );
    }

    #[test]
    fn cell_list_header_contract_materializes_width_ref_and_extra() {
        let mut cell = Cell {
            width: 2266,
            list_header_width_ref: 0,
            raw_list_extra: Vec::new(),
            ..Default::default()
        };
        let mut report = AdapterReport::new();

        materialize_cell_list_header_contract(
            &mut cell,
            true,
            &crate::model::Padding::default(),
            &mut report,
        );

        assert_eq!(cell.list_header_width_ref & 0x0001, 0x0001);
        assert_eq!(cell.raw_list_extra.len(), 13);
        assert_eq!(
            u32::from_le_bytes(cell.raw_list_extra[0..4].try_into().unwrap()),
            2266
        );
        assert!(cell.raw_list_extra[4..].iter().all(|&byte| byte == 0));
        assert_eq!(report.cells_list_header_contract_materialized, 1);

        materialize_cell_list_header_contract(
            &mut cell,
            true,
            &crate::model::Padding::default(),
            &mut report,
        );
        assert_eq!(report.cells_list_header_contract_materialized, 1);
    }

    #[test]
    fn cell_list_header_contract_keeps_width_ref_clear_for_normal_tables() {
        let mut cell = Cell {
            width: 2266,
            list_header_width_ref: 0x0001,
            raw_list_extra: Vec::new(),
            ..Default::default()
        };
        let mut report = AdapterReport::new();

        materialize_cell_list_header_contract(
            &mut cell,
            false,
            &crate::model::Padding::default(),
            &mut report,
        );

        assert_eq!(cell.list_header_width_ref & 0x0001, 0);
        assert_eq!(cell.raw_list_extra.len(), 13);
        assert_eq!(
            u32::from_le_bytes(cell.raw_list_extra[0..4].try_into().unwrap()),
            2266
        );
        assert_eq!(report.cells_list_header_contract_materialized, 1);
    }

    #[test]
    fn idempotent_when_called_twice() {
        let mut doc = Document::default();
        let r1 = convert_hwpx_to_hwp_ir(&mut doc);
        let r2 = convert_hwpx_to_hwp_ir(&mut doc);
        assert_eq!(r1.file_header_compression_normalized, 1);
        // 두 번째 호출은 변경 없음 (이미 정규화됨).
        assert_eq!(r2.tables_ctrl_data_synthesized, 0);
        assert_eq!(r2.file_header_compression_normalized, 0);
        assert!(!r2.changed_anything());
    }

    #[test]
    fn following_section_first_paragraph_break_type_preserves_materialized_flags() {
        let mut first_para = Paragraph {
            raw_break_type: 0x01,
            ..Default::default()
        };
        first_para
            .controls
            .push(Control::SectionDef(Box::<SectionDef>::default()));

        let mut following_para = Paragraph {
            raw_break_type: 0x03,
            ..Default::default()
        };
        following_para
            .controls
            .push(Control::SectionDef(Box::<SectionDef>::default()));
        following_para
            .controls
            .push(Control::ColumnDef(Default::default()));

        let mut doc = Document {
            sections: vec![
                Section {
                    paragraphs: vec![first_para],
                    ..Default::default()
                },
                Section {
                    paragraphs: vec![following_para],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let report = convert_hwpx_to_hwp_ir(&mut doc);

        assert_eq!(doc.sections[0].paragraphs[0].raw_break_type, 0x01);
        assert_eq!(doc.sections[1].paragraphs[0].raw_break_type, 0x03);
        assert_eq!(report.following_section_break_type_materialized, 0);

        let second = convert_hwpx_to_hwp_ir(&mut doc);
        assert_eq!(second.following_section_break_type_materialized, 0);
    }

    #[test]
    fn following_section_first_paragraph_break_type_fills_missing_section_flag() {
        let mut following_para = Paragraph {
            raw_break_type: 0,
            ..Default::default()
        };
        following_para
            .controls
            .push(Control::SectionDef(Box::<SectionDef>::default()));

        let mut doc = Document {
            sections: vec![
                Section {
                    paragraphs: vec![Paragraph::default()],
                    ..Default::default()
                },
                Section {
                    paragraphs: vec![following_para],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let report = convert_hwpx_to_hwp_ir(&mut doc);

        assert_eq!(doc.sections[1].paragraphs[0].raw_break_type, 0x01);
        assert_eq!(report.following_section_break_type_materialized, 1);
    }

    #[test]
    fn picture_href_ctrl_data_matches_hancom_parameter_set_shape() {
        let data = build_picture_href_ctrl_data("http://www.korea.kr;1;0;0;");
        assert_eq!(data.len(), 76);
        assert_eq!(&data[0..2], &0x021b_u16.to_le_bytes());
        assert_eq!(&data[2..4], &1_u16.to_le_bytes());
        assert_eq!(&data[6..8], &0x026f_u16.to_le_bytes());
        assert_eq!(&data[8..10], &0x8000_u16.to_le_bytes());
        assert_eq!(&data[10..12], &0x026f_u16.to_le_bytes());
        assert_eq!(&data[16..18], &0x0265_u16.to_le_bytes());
        assert_eq!(&data[18..20], &0x0001_u16.to_le_bytes());
        assert_eq!(&data[20..22], &27_u16.to_le_bytes());

        let text: Vec<u16> = data[22..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        assert_eq!(
            String::from_utf16(&text).unwrap(),
            "http\\://www.korea.kr;1;0;0;"
        );
    }

    #[test]
    fn picture_href_ctrl_data_materializes_on_matching_control_slot() {
        let mut para = Paragraph::default();
        let pic = Picture {
            href: Some("http://www.korea.kr;1;0;0;".to_string()),
            ..Default::default()
        };
        para.controls.push(Control::Picture(Box::new(pic)));

        let mut report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut report);
        assert_eq!(report.picture_href_ctrl_data_materialized, 1);
        assert_eq!(para.ctrl_data_records.len(), 1);
        assert_eq!(para.ctrl_data_records[0].as_ref().unwrap().len(), 76);

        let mut second = AdapterReport::new();
        adapt_paragraph(&mut para, &mut second);
        assert_eq!(second.picture_href_ctrl_data_materialized, 0);
    }

    #[test]
    fn table_layout_ctrl_data_materializes_for_three_by_two_row_break_table() {
        let mut para = Paragraph::default();
        para.controls.push(Control::Table(Box::new(Table {
            row_count: 3,
            col_count: 2,
            page_break: TablePageBreak::RowBreak,
            repeat_header: true,
            ..Default::default()
        })));

        let mut report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut report);

        assert_eq!(report.table_layout_ctrl_data_materialized, 1);
        assert_eq!(para.ctrl_data_records.len(), 1);
        let data = para.ctrl_data_records[0].as_ref().unwrap();
        assert_eq!(data.len(), 104);
        assert_eq!(&data[0..2], &0x021b_u16.to_le_bytes());
        assert_eq!(&data[6..8], &0x0242_u16.to_le_bytes());
        assert_eq!(&data[10..12], &0x0242_u16.to_le_bytes());
        assert_eq!(&data[12..14], &11_u16.to_le_bytes());

        let mut second = AdapterReport::new();
        adapt_paragraph(&mut para, &mut second);
        assert_eq!(second.table_layout_ctrl_data_materialized, 0);
    }

    #[test]
    fn table_layout_ctrl_data_does_not_materialize_for_other_table_shapes() {
        let mut para = Paragraph::default();
        para.controls.push(Control::Table(Box::new(Table {
            row_count: 3,
            col_count: 3,
            page_break: TablePageBreak::RowBreak,
            repeat_header: true,
            ..Default::default()
        })));

        let mut report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut report);

        assert_eq!(report.table_layout_ctrl_data_materialized, 0);
        assert!(para.ctrl_data_records[0].is_none());
    }

    #[test]
    fn single_master_page_flags_materialize_hancom_save_contract() {
        let mut section_def = SectionDef {
            flags: 0x4000_0000,
            master_pages: vec![Default::default()],
            ..Default::default()
        };

        let mut report = AdapterReport::new();
        adapt_section_def(&mut section_def, &mut report);

        assert_eq!(section_def.flags, 0x2000_0000);
        assert_eq!(report.section_def_single_master_page_flags_materialized, 1);

        let mut second = AdapterReport::new();
        adapt_section_def(&mut section_def, &mut second);
        assert_eq!(second.section_def_single_master_page_flags_materialized, 0);
    }

    #[test]
    fn two_master_page_flags_materialize_hancom_save_contract() {
        let mut section_def = SectionDef {
            flags: 0x8000_0000,
            master_pages: vec![Default::default(), Default::default()],
            ..Default::default()
        };

        let mut report = AdapterReport::new();
        adapt_section_def(&mut section_def, &mut report);

        assert_eq!(section_def.flags, 0xC000_0000);
        assert_eq!(report.section_def_multi_master_page_flags_materialized, 1);

        let mut second = AdapterReport::new();
        adapt_section_def(&mut section_def, &mut second);
        assert_eq!(second.section_def_multi_master_page_flags_materialized, 0);
    }

    #[test]
    fn task1654_hide_empty_line_flag_materializes_before_section_def_control_copy() {
        let mut section = Section::default();
        section.section_def.hide_empty_line = true;
        section.section_def.flags &= !0x0008_0000;
        section.paragraphs.push(Paragraph::default());

        let mut doc = Document {
            sections: vec![section],
            ..Default::default()
        };

        let report = convert_hwpx_to_hwp_ir(&mut doc);
        assert_eq!(report.section_def_hide_empty_line_flag_materialized, 1);
        assert_ne!(doc.sections[0].section_def.flags & 0x0008_0000, 0);

        let Control::SectionDef(section_def) = &doc.sections[0].paragraphs[0].controls[0] else {
            panic!("SectionDef 컨트롤이 삽입되어야 함");
        };
        assert!(section_def.hide_empty_line);
        assert_ne!(section_def.flags & 0x0008_0000, 0);
    }

    #[test]
    fn section_def_master_page_tail_marker_depends_on_master_page_count() {
        let mut single = SectionDef {
            master_pages: vec![Default::default()],
            ..Default::default()
        };
        let mut report = AdapterReport::new();
        materialize_section_def_master_page_tail(&mut single, &mut report);
        assert_eq!(single.raw_ctrl_extra.len(), 19);
        assert_eq!(&single.raw_ctrl_extra[0..4], &[0, 0, 0, 0]);
        assert_eq!(report.section_def_master_page_tail_materialized, 1);

        let mut pair = SectionDef {
            master_pages: vec![Default::default(), Default::default()],
            ..Default::default()
        };
        let mut report = AdapterReport::new();
        materialize_section_def_master_page_tail(&mut pair, &mut report);
        assert_eq!(pair.raw_ctrl_extra.len(), 19);
        assert_eq!(&pair.raw_ctrl_extra[0..4], &[0, 0, 0, 0]);
        assert_eq!(report.section_def_master_page_tail_materialized, 1);

        let mut triple = SectionDef {
            master_pages: vec![Default::default(), Default::default(), Default::default()],
            ..Default::default()
        };
        let mut report = AdapterReport::new();
        materialize_section_def_master_page_tail(&mut triple, &mut report);
        assert_eq!(triple.raw_ctrl_extra.len(), 19);
        assert_eq!(&triple.raw_ctrl_extra[0..4], &[0, 0, 1, 0]);
        assert_eq!(report.section_def_master_page_tail_materialized, 1);
    }

    #[test]
    fn header_footer_nested_tables_are_materialized() {
        use crate::model::header_footer::{Footer, Header};

        fn make_table_para() -> Paragraph {
            let table = Table {
                row_count: 1,
                col_count: 1,
                cells: vec![Cell {
                    col: 0,
                    row: 0,
                    col_span: 1,
                    row_span: 1,
                    ..Default::default()
                }],
                repeat_header: true,
                ..Default::default()
            };

            let mut para = Paragraph::default();
            para.controls.push(Control::Table(Box::new(table)));
            para
        }

        let mut para = Paragraph::default();
        para.controls.push(Control::Header(Box::new(Header {
            paragraphs: vec![make_table_para()],
            ..Default::default()
        })));
        para.controls.push(Control::Footer(Box::new(Footer {
            paragraphs: vec![make_table_para()],
            ..Default::default()
        })));

        let mut report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut report);

        for control in &para.controls {
            let nested = match control {
                Control::Header(header) => &header.paragraphs[0].controls[0],
                Control::Footer(footer) => &footer.paragraphs[0].controls[0],
                _ => continue,
            };
            let Control::Table(table) = nested else {
                panic!("expected nested table");
            };

            assert!(!table.raw_ctrl_data.is_empty());
            assert_eq!(table.raw_table_record_attr & 0x04, 0x04);
            assert_eq!(table.row_sizes, vec![1]);
            assert_eq!(table.raw_table_record_extra, vec![0, 0]);
            assert_eq!(table.cells[0].raw_list_extra.len(), 13);
        }

        assert_eq!(report.tables_ctrl_data_synthesized, 2);
        assert_eq!(report.table_record_extra_materialized, 2);
        assert_eq!(report.cells_list_header_contract_materialized, 2);
    }

    #[test]
    fn picture_href_ctrl_data_materializes_inside_shape_text_box() {
        use crate::model::shape::{DrawingObjAttr, RectangleShape, TextBox};

        let mut nested_para = Paragraph::default();
        nested_para
            .controls
            .push(Control::Picture(Box::new(Picture {
                href: Some("http://www.korea.kr;1;0;0;".to_string()),
                ..Default::default()
            })));

        let mut shape = ShapeObject::Rectangle(RectangleShape {
            drawing: DrawingObjAttr {
                text_box: Some(TextBox {
                    paragraphs: vec![nested_para],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        });

        let mut report = AdapterReport::new();
        adapt_shape(&mut shape, &mut report);
        assert_eq!(report.picture_href_ctrl_data_materialized, 1);

        let ShapeObject::Rectangle(rect) = shape else {
            panic!("expected rectangle");
        };
        let text_box = rect.drawing.text_box.unwrap();
        let ctrl_data = text_box.paragraphs[0].ctrl_data_records[0]
            .as_ref()
            .unwrap();
        assert_eq!(ctrl_data.len(), 76);
    }

    #[test]
    fn hwpx_h_03_href_ctrl_data_from_source_contract() {
        let data = std::fs::read("samples/hwpx/hwpx-h-03.hwpx").expect("sample exists");
        let mut core = crate::document_core::DocumentCore::from_bytes(&data).expect("parse hwpx");

        assert_eq!(
            count_ctrl_data_records_in_sections(&core.document.sections),
            0
        );
        let report = convert_hwpx_to_hwp_ir(&mut core.document);

        assert_eq!(report.picture_href_ctrl_data_materialized, 1);
        assert_eq!(
            count_ctrl_data_records_in_sections(&core.document.sections),
            1
        );
    }

    #[test]
    fn hwpx_h_03_rect_draw_text_contract_from_source() {
        let data = std::fs::read("samples/hwpx/hwpx-h-03.hwpx").expect("sample exists");
        let core = crate::document_core::DocumentCore::from_bytes(&data).expect("parse hwpx");

        let shape = find_shape_by_description(&core.document, "사각형입니다.")
            .expect("hp:rect shapeComment must survive into CommonObjAttr.description");
        let drawing = shape.drawing().expect("hp:rect must have DrawingObjAttr");
        let text_box = drawing
            .text_box
            .as_ref()
            .expect("hp:rect/drawText must survive as TextBox");

        assert_eq!(shape.common().instance_id, 1875692958);
        assert_eq!(drawing.inst_id, 801951135);
        assert_eq!(
            text_box.list_attr & (0b11 << 5),
            1 << 5,
            "drawText subList vertAlign=CENTER must materialize LIST_HEADER list_attr bit 5"
        );
        assert_eq!(text_box.max_width, 25698);
        assert_eq!(text_box.paragraphs.len(), 1);

        let pictures: Vec<_> = text_box.paragraphs[0]
            .controls
            .iter()
            .filter_map(|control| match control {
                Control::Picture(pic) => Some(pic.as_ref()),
                _ => None,
            })
            .collect();
        assert_eq!(pictures.len(), 2);
        assert_eq!(pictures[0].common.instance_id, 1875692960);
        assert_eq!(pictures[0].instance_id, 801951137);
        assert_eq!(pictures[1].common.instance_id, 1875692962);
        assert_eq!(pictures[1].instance_id, 801951139);
    }

    #[test]
    fn hwpx_h_03_draw_text_envelope_materializes_with_id_instid_contract() {
        let data = std::fs::read("samples/hwpx/hwpx-h-03.hwpx").expect("sample exists");
        let mut core = crate::document_core::DocumentCore::from_bytes(&data).expect("parse hwpx");

        let report = convert_hwpx_to_hwp_ir(&mut core.document);
        assert!(report.text_box_list_header_tail_materialized > 0);
        assert!(report.text_box_para_header_tail_materialized > 0);

        let shape = find_shape_by_description(&core.document, "사각형입니다.")
            .expect("hp:rect shapeComment must survive into CommonObjAttr.description");
        let drawing = shape.drawing().expect("hp:rect must have DrawingObjAttr");
        let text_box = drawing
            .text_box
            .as_ref()
            .expect("hp:rect/drawText must survive as TextBox");

        assert_eq!(shape.common().instance_id, 1875692958);
        assert_eq!(drawing.inst_id, 801951135);
        assert_eq!(text_box.raw_list_header_extra, vec![0; 13]);
        assert_eq!(text_box.paragraphs.len(), 1);
        assert_eq!(text_box.paragraphs[0].raw_header_extra.len(), 12);
        assert_eq!(
            &text_box.paragraphs[0].raw_header_extra[6..10],
            &[0, 0, 0, 0x80]
        );

        let pictures: Vec<_> = text_box.paragraphs[0]
            .controls
            .iter()
            .filter_map(|control| match control {
                Control::Picture(pic) => Some(pic.as_ref()),
                _ => None,
            })
            .collect();
        assert_eq!(pictures.len(), 2);
        assert_eq!(pictures[0].common.instance_id, 1875692960);
        assert_eq!(pictures[0].instance_id, 801951137);
        assert_eq!(pictures[1].common.instance_id, 1875692962);
        assert_eq!(pictures[1].instance_id, 801951139);
    }

    fn find_shape_by_description<'a>(
        doc: &'a Document,
        description: &str,
    ) -> Option<&'a ShapeObject> {
        doc.sections
            .iter()
            .flat_map(|section| section.paragraphs.iter())
            .find_map(|para| find_shape_by_description_in_paragraph(para, description))
    }

    fn find_shape_by_description_in_paragraph<'a>(
        para: &'a Paragraph,
        description: &str,
    ) -> Option<&'a ShapeObject> {
        para.controls
            .iter()
            .find_map(|control| find_shape_by_description_in_control(control, description))
    }

    fn find_shape_by_description_in_control<'a>(
        control: &'a Control,
        description: &str,
    ) -> Option<&'a ShapeObject> {
        match control {
            Control::Shape(shape) => find_shape_by_description_in_shape(shape, description),
            Control::Table(table) => table
                .cells
                .iter()
                .flat_map(|cell| cell.paragraphs.iter())
                .find_map(|para| find_shape_by_description_in_paragraph(para, description)),
            _ => None,
        }
    }

    fn find_shape_by_description_in_shape<'a>(
        shape: &'a ShapeObject,
        description: &str,
    ) -> Option<&'a ShapeObject> {
        if shape.common().description == description {
            return Some(shape);
        }

        if let ShapeObject::Group(group) = shape {
            return group
                .children
                .iter()
                .find_map(|child| find_shape_by_description_in_shape(child, description));
        }

        shape
            .drawing()
            .and_then(|drawing| drawing.text_box.as_ref())
            .and_then(|text_box| {
                text_box
                    .paragraphs
                    .iter()
                    .find_map(|para| find_shape_by_description_in_paragraph(para, description))
            })
    }

    fn count_ctrl_data_records_in_sections(sections: &[Section]) -> usize {
        sections
            .iter()
            .map(|section| count_ctrl_data_records_in_paragraphs(&section.paragraphs))
            .sum()
    }

    fn count_ctrl_data_records_in_paragraphs(paragraphs: &[Paragraph]) -> usize {
        paragraphs
            .iter()
            .map(|para| {
                let own = para
                    .ctrl_data_records
                    .iter()
                    .filter(|data| data.is_some())
                    .count();
                own + para
                    .controls
                    .iter()
                    .map(count_ctrl_data_records_in_control)
                    .sum::<usize>()
            })
            .sum()
    }

    fn count_ctrl_data_records_in_control(control: &Control) -> usize {
        match control {
            Control::Table(table) => table
                .cells
                .iter()
                .map(|cell| count_ctrl_data_records_in_paragraphs(&cell.paragraphs))
                .sum(),
            Control::Shape(shape) => count_ctrl_data_records_in_shape(shape),
            _ => 0,
        }
    }

    fn count_ctrl_data_records_in_shape(shape: &ShapeObject) -> usize {
        let text_box_count = shape
            .drawing()
            .and_then(|drawing| drawing.text_box.as_ref())
            .map(|text_box| count_ctrl_data_records_in_paragraphs(&text_box.paragraphs))
            .unwrap_or(0);

        let child_count = match shape {
            ShapeObject::Group(group) => group
                .children
                .iter()
                .map(count_ctrl_data_records_in_shape)
                .sum(),
            _ => 0,
        };

        text_box_count + child_count
    }

    // ============================================================
    // Stage 3 — cell.list_attr bit 16 보강 단위 테스트
    // ============================================================

    fn make_cell_with_inner_margin(apply: bool, text_dir: u8) -> Cell {
        let mut cell = Cell::default();
        cell.apply_inner_margin = apply;
        cell.text_direction = text_dir;
        cell
    }

    #[test]
    fn stage3_cell_with_inner_margin_gets_width_ref_bit0() {
        let mut cell = make_cell_with_inner_margin(true, 0);
        let mut report = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut report);
        assert_eq!(
            cell.list_header_width_ref & 0x0001,
            0x0001,
            "셀 안쪽 여백 지정은 LIST_HEADER width_ref bit 0으로 저장되어야 함"
        );
        assert_eq!(report.cells_list_attr_bit16_set, 1);
    }

    #[test]
    fn stage3_cell_with_width_ref_bit0_already_set_no_change() {
        let mut cell = make_cell_with_inner_margin(true, 0);
        cell.list_header_width_ref = 0x0001;
        let mut report = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut report);
        assert_eq!(cell.list_header_width_ref & 0x0001, 0x0001);
        assert_eq!(report.cells_list_attr_bit16_set, 0);
    }

    #[test]
    fn stage3_no_inner_margin_no_change() {
        let mut cell = make_cell_with_inner_margin(false, 0);
        let mut report = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut report);
        assert_eq!(cell.list_header_width_ref & 0x0001, 0);
        assert_eq!(report.cells_list_attr_bit16_set, 0);
    }

    #[test]
    fn stage3_list_header_width_ref_layout_has_apply_inner_margin_bit_after_adapter() {
        let mut cell = make_cell_with_inner_margin(true, 0);
        let mut report = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut report);

        assert_eq!(
            cell.list_header_width_ref & 0x0001,
            0x0001,
            "LIST_HEADER bytes 6-7의 bit 0 = 1"
        );
        let recovered_apply_inner_margin = cell.list_header_width_ref & 0x0001 != 0;
        assert!(
            recovered_apply_inner_margin,
            "재파싱 시 apply_inner_margin 회복"
        );
    }

    #[test]
    fn stage3_idempotent_does_not_double_or() {
        let mut cell = make_cell_with_inner_margin(true, 0);
        let mut r1 = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut r1);
        // 1차 호출 후 width_ref bit0=1, apply_inner_margin=true
        assert_eq!(cell.list_header_width_ref & 0x0001, 0x0001);

        let mut r2 = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut r2);
        // 2차 호출은 width_ref bit0이 이미 1이므로 변경 없음
        assert_eq!(cell.list_header_width_ref & 0x0001, 0x0001);
        assert_eq!(r2.cells_list_attr_bit16_set, 0);
    }

    #[test]
    fn autonum_fwspace_materializes_hancom_range_tag_once() {
        let mut para = Paragraph {
            text: " \u{2007}(사회·문화)".to_string(),
            char_offsets: vec![0, 8, 9, 10, 11, 12, 13, 14, 15],
            char_count: 17,
            controls: vec![Control::AutoNumber(Default::default())],
            ..Default::default()
        };

        let mut report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut report);

        assert_eq!(report.autonum_fwspace_range_tag_materialized, 1);
        assert_eq!(para.range_tags.len(), 1);
        assert_eq!(para.range_tags[0].start, 15);
        assert_eq!(para.range_tags[0].end, 16);
        assert_eq!(para.range_tags[0].tag, 0x0100_0023);

        let mut second_report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut second_report);
        assert_eq!(second_report.autonum_fwspace_range_tag_materialized, 0);
        assert_eq!(para.range_tags.len(), 1);
    }

    #[test]
    fn hwp5_save_fwspace_marks_fixed_blank_control() {
        const HWP5_FIXED_WIDTH_SPACE_MASK: u32 = 1u32 << 0x001f;

        let mut header_para = Paragraph {
            text: "사회탐구\u{2007}영역".to_string(),
            char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            char_count: 10,
            ..Default::default()
        };
        let mut report = AdapterReport::new();
        adapt_paragraph_with_context(
            &mut header_para,
            &mut report,
            ParagraphContext::HeaderFooter,
        );

        assert_eq!(report.header_footer_fwspace_control_materialized, 0);
        assert_eq!(header_para.control_mask & HWP5_FIXED_WIDTH_SPACE_MASK, 0);

        let mut body_para = Paragraph {
            text: "사회탐구\u{2007}영역".to_string(),
            char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            char_count: 10,
            ..Default::default()
        };
        let mut body_report = AdapterReport::new();
        adapt_paragraph_with_context(&mut body_para, &mut body_report, ParagraphContext::Body);

        assert_eq!(body_report.header_footer_fwspace_control_materialized, 1);
        assert_ne!(body_para.control_mask & HWP5_FIXED_WIDTH_SPACE_MASK, 0);

        let mut master_para = Paragraph {
            text: "사회탐구\u{2007}영역".to_string(),
            char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            char_count: 10,
            ..Default::default()
        };
        let mut master_report = AdapterReport::new();
        adapt_paragraph_with_context(
            &mut master_para,
            &mut master_report,
            ParagraphContext::MasterPage,
        );

        assert_eq!(master_report.header_footer_fwspace_control_materialized, 0);
        assert_eq!(master_para.control_mask & HWP5_FIXED_WIDTH_SPACE_MASK, 0);
    }

    #[test]
    fn master_page_autonum_removes_parser_placeholder_space() {
        let mut master_page_para = Paragraph {
            text: " ".to_string(),
            char_offsets: vec![0],
            char_count: 9,
            controls: vec![Control::AutoNumber(Default::default())],
            has_para_text: true,
            ..Default::default()
        };

        let mut report = AdapterReport::new();
        adapt_paragraph_with_context(
            &mut master_page_para,
            &mut report,
            ParagraphContext::MasterPage,
        );

        assert_eq!(report.master_page_autonum_placeholder_removed, 1);
        assert!(master_page_para.text.is_empty());
        assert!(master_page_para.char_offsets.is_empty());
        assert_eq!(master_page_para.char_count, 9);

        let mut body_para = Paragraph {
            text: " ".to_string(),
            char_offsets: vec![0],
            char_count: 9,
            controls: vec![Control::AutoNumber(Default::default())],
            has_para_text: true,
            ..Default::default()
        };
        let mut body_report = AdapterReport::new();
        adapt_paragraph_with_context(&mut body_para, &mut body_report, ParagraphContext::Body);

        assert_eq!(body_report.master_page_autonum_placeholder_removed, 0);
        assert_eq!(body_para.text, " ");
        assert_eq!(body_para.char_offsets, vec![0]);
    }

    #[test]
    fn master_page_line_rendering_uses_exact_size_ratio() {
        use crate::model::shape::{LineShape, ShapeComponentAttr};

        fn matrix(values: [f64; 6]) -> Vec<u8> {
            let mut bytes = Vec::with_capacity(48);
            for value in values {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes
        }

        let mut raw_rendering = Vec::new();
        raw_rendering.extend_from_slice(&1_u16.to_le_bytes());
        raw_rendering.extend_from_slice(&matrix([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]));
        raw_rendering.extend_from_slice(&matrix([
            f64::from(0.01_f32),
            0.0,
            0.0,
            0.0,
            f64::from(924.09_f32),
            0.0,
        ]));
        raw_rendering.extend_from_slice(&matrix([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]));

        let mut shape = ShapeObject::Line(LineShape {
            drawing: crate::model::shape::DrawingObjAttr {
                shape_attr: ShapeComponentAttr {
                    original_width: 100,
                    original_height: 100,
                    current_width: 1,
                    current_height: 92409,
                    raw_rendering,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        });
        let mut report = AdapterReport::new();

        adapt_shape_with_context(&mut shape, &mut report, ParagraphContext::MasterPage);

        let line = match shape {
            ShapeObject::Line(line) => line,
            other => panic!("expected line, got {:?}", other),
        };
        assert_eq!(report.master_page_line_rendering_size_ratio_materialized, 1);
        let raw = &line.drawing.shape_attr.raw_rendering;
        assert_eq!(read_raw_rendering_f64(raw, 2 + 48), Some(0.01));
        assert_eq!(read_raw_rendering_f64(raw, 2 + 48 + 4 * 8), Some(924.09));
        assert_eq!(
            read_raw_rendering_f64(raw, 2 + 48 + 48 + 8)
                .unwrap()
                .to_bits(),
            (-0.0_f64).to_bits()
        );
    }

    #[test]
    fn autonum_fwspace_materializes_char_shape_offsets_once() {
        let mut para = Paragraph {
            text: " \u{2007}(사회·문화)".to_string(),
            char_offsets: vec![0, 8, 9, 10, 11, 12, 13, 14, 15],
            char_count: 17,
            controls: vec![Control::AutoNumber(Default::default())],
            char_shapes: vec![
                CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 63,
                },
                CharShapeRef {
                    start_pos: 2,
                    char_shape_id: 74,
                },
                CharShapeRef {
                    start_pos: 9,
                    char_shape_id: 76,
                },
            ],
            ..Default::default()
        };

        let mut report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut report);

        assert_eq!(report.autonum_fwspace_char_shape_offsets_materialized, 1);
        assert_eq!(para.char_shapes[0].start_pos, 0);
        assert_eq!(para.char_shapes[1].start_pos, 9);
        assert_eq!(para.char_shapes[2].start_pos, 16);

        let mut second_report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut second_report);
        assert_eq!(
            second_report.autonum_fwspace_char_shape_offsets_materialized,
            0
        );
        assert_eq!(para.char_shapes[1].start_pos, 9);
        assert_eq!(para.char_shapes[2].start_pos, 16);
    }
}
