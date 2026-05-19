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

use crate::model::bin_data::{BinDataStatus, BinDataType};
use crate::model::control::Control;
use crate::model::document::{Document, Section};
use crate::model::image::Picture;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{ShapeObject, TextBox};
use crate::model::style::{BorderFill, BorderLineType, FillType};
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
    /// `Control::SectionDef` 컨트롤 삽입 횟수 (Stage 4 — 섹션 개수)
    pub section_def_controls_inserted: u32,
    /// HWPX `hp:pic@href` 를 HWP CTRL_DATA ParameterSet 으로 materialize한 횟수
    pub picture_href_ctrl_data_materialized: u32,
    /// HWPX drawText TextBox LIST_HEADER tail materialize 횟수
    pub text_box_list_header_tail_materialized: u32,
    /// HWPX drawText 내부 paragraph PARA_HEADER tail materialize 횟수
    pub text_box_para_header_tail_materialized: u32,
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
                + self.section_def_controls_inserted
                + self.picture_href_ctrl_data_materialized
                + self.text_box_list_header_tail_materialized
                + self.text_box_para_header_tail_materialized)
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
    normalize_bin_data_for_hwp(doc, &mut report);

    // Stage 4: SectionDef 컨트롤 삽입 (HWPX 파서가 만들지 않으므로 직렬화기가 PAGE_DEF 출력 못 함)
    for section in &mut doc.sections {
        insert_section_def_control(section, &mut report);
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
/// record 전용 attr/status 값은 비워 둔다. 한컴 HWP 로더는 embedded image의
/// `BIN_DATA` record에서 `attr=0x0101`, 접근 상태 success 형태를 기대하므로,
/// HWP 저장 직전에 HWPX 출처 모델을 명시적으로 보정한다.
fn normalize_bin_data_for_hwp(doc: &mut Document, report: &mut AdapterReport) {
    let mut changed = false;

    for bin_data in &mut doc.doc_info.bin_data_list {
        if !matches!(
            bin_data.data_type,
            BinDataType::Embedding | BinDataType::Storage
        ) {
            continue;
        }

        if bin_data.attr != 0x0101 {
            bin_data.attr = 0x0101;
            changed = true;
        }

        if !matches!(bin_data.status, BinDataStatus::Success) {
            bin_data.status = BinDataStatus::Success;
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
/// HWPX 파서는 `<hp:secPr>` 정보를 `Section.section_def` 필드로 채우지만,
/// `Control::SectionDef` 컨트롤을 첫 문단의 `controls` 에 삽입하지는 않는다.
/// HWP 직렬화기 (`serializer/control.rs:40 + 171-241`) 는 `paragraph.controls` 를
/// 순회하면서 `Control::SectionDef` 를 만나야 PAGE_DEF / FOOTNOTE_SHAPE / PAGE_BORDER_FILL
/// 레코드를 출력한다. 이 컨트롤이 없으면 직렬화 결과의 PAGE_DEF 가 누락되어 재로드 시
/// `page_def.width = 0` 등 페이지 크기 손상으로 페이지 폭주 발생.
///
/// ## 동작
///
/// 1. 섹션의 첫 문단에 `Control::SectionDef` 가 이미 있으면 no-op (idempotent)
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
    let already_has_section_def = first_para
        .controls
        .iter()
        .any(|c| matches!(c, Control::SectionDef(_)));
    if already_has_section_def {
        return;
    }
    first_para.controls.insert(
        0,
        Control::SectionDef(Box::new(section.section_def.clone())),
    );
    report.section_def_controls_inserted += 1;
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

fn adapt_paragraph(para: &mut Paragraph, report: &mut AdapterReport) {
    if para.ctrl_data_records.len() < para.controls.len() {
        para.ctrl_data_records
            .resize_with(para.controls.len(), || None);
    }

    let controls = &mut para.controls;
    let ctrl_data_records = &mut para.ctrl_data_records;
    for (idx, ctrl) in controls.iter_mut().enumerate() {
        match ctrl {
            Control::Table(table) => adapt_table(table, report),
            Control::Picture(pic) => {
                adapt_picture_href_ctrl_data(pic, &mut ctrl_data_records[idx], report)
            }
            Control::Shape(shape) => adapt_shape(shape, report),
            _ => {}
        }
    }
}

fn adapt_shape(shape: &mut ShapeObject, report: &mut AdapterReport) {
    if let Some(drawing) = shape.drawing_mut() {
        if let Some(text_box) = &mut drawing.text_box {
            materialize_text_box_hwp5_envelope(text_box, report);
            for para in &mut text_box.paragraphs {
                adapt_paragraph(para, report);
            }
        }
    }

    if let ShapeObject::Group(group) = shape {
        for child in &mut group.children {
            adapt_shape(child, report);
        }
    }
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

fn adapt_table(table: &mut Table, report: &mut AdapterReport) {
    // 1. raw_ctrl_data 합성 (HWPX 출처는 비어있음)
    if table.raw_ctrl_data.is_empty() {
        materialize_table_outer_margin(table, report);
        materialize_table_page_break(table, report);
        materialize_table_record_attr(table, report);
        materialize_table_record_row_sizes(table, report);
        materialize_table_record_extra(table, report);
        materialize_table_ctrl_header_attr(table, report);

        table.raw_ctrl_data = serialize_common_obj_attr(&table.common);
        report.tables_ctrl_data_synthesized += 1;

        if table.raw_ctrl_data.len() >= 4 {
            let packed = u32::from_le_bytes([
                table.raw_ctrl_data[0],
                table.raw_ctrl_data[1],
                table.raw_ctrl_data[2],
                table.raw_ctrl_data[3],
            ]);
            if table.attr != packed {
                table.attr = packed;
                report.tables_attr_packed += 1;
            }
        }
    }

    // 셀별 보강 + 내부 문단 재귀 (중첩 표 대응)
    let use_cell_width_ref = table_requires_cell_width_ref_contract(table);
    for cell in &mut table.cells {
        adapt_cell_list_attr(cell, report);
        materialize_cell_list_header_contract(cell, use_cell_width_ref, report);
        for cpara in &mut cell.paragraphs {
            adapt_paragraph(cpara, report);
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
    report: &mut AdapterReport,
) {
    let before_width_ref = cell.list_header_width_ref;
    let before_extra_len = cell.raw_list_extra.len();

    if use_width_ref {
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

fn materialize_table_page_break(table: &mut Table, report: &mut AdapterReport) {
    if matches!(table.page_break, TablePageBreak::CellBreak) {
        table.page_break = TablePageBreak::RowBreak;
        table.raw_table_record_attr = 0;
        report.tables_page_break_materialized += 1;
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
    if table.attr & 0x08 != 0 {
        attr |= 0x08;
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

/// 셀 `apply_inner_margin` → `list_attr bit 16` 합성 (Stage 3, 보수적).
///
/// ## 배경
///
/// `serializer/control.rs:429` 가 작성하는 LIST_HEADER 의 `list_attr`:
/// ```text
/// list_attr = (text_direction << 16) | (v_align << 21)
/// ```
///
/// HWPX 출처 셀에서 `apply_inner_margin = true` 인 경우, 직렬화 시 `list_attr bit 16` 이
/// 0 으로 떨어져 한컴이 셀 안 여백을 표 기본값으로 대체하는 손실 발생.
///
/// ## 합성 방식
///
/// `cell.text_direction == 0` (가로 = 99% 케이스) AND `apply_inner_margin == true` 일 때만
/// `text_direction |= 0x01` 합성. 이는 출력 LIST_HEADER 의 bit 16 = 1 을 만들어
/// 한컴이 `apply_inner_margin` 으로 인식하도록 함. 가로/세로 비트 자체에 영향이 있을 수 있으나,
/// `apply_inner_margin` 의미가 한컴에서 더 우선 (parser/control.rs:371 동일 로직).
///
/// 세로 셀 (`text_direction == 1`) 은 이미 bit 16 = 1 이므로 추가 합성 불필요.
///
/// ## 한계
///
/// 현재 디버그 샘플 3건 (hwpx-h-0[123].hwpx) 에는 `apply_inner_margin = true` 인 셀이 0건이므로,
/// 본 함수는 단위 테스트로만 동작 검증 (효과 측정은 후속 샘플에서).
fn adapt_cell_list_attr(cell: &mut Cell, report: &mut AdapterReport) {
    if cell.apply_inner_margin && cell.text_direction == 0 {
        cell.text_direction = 1; // bit 0 OR (출력 bit 16 = 1)
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
            page_break: TablePageBreak::CellBreak,
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

        assert_eq!(table.raw_table_record_attr, 0x0000_000e);
        assert_eq!(table.row_sizes, vec![3]);
        assert_eq!(table.raw_table_record_extra, vec![0, 0]);
        assert_eq!(
            u32::from_le_bytes(table.raw_ctrl_data[0..4].try_into().unwrap()),
            0x082a_2311
        );
        assert_eq!(
            (
                i16::from_le_bytes(table.raw_ctrl_data[24..26].try_into().unwrap()),
                i16::from_le_bytes(table.raw_ctrl_data[26..28].try_into().unwrap()),
                i16::from_le_bytes(table.raw_ctrl_data[28..30].try_into().unwrap()),
                i16::from_le_bytes(table.raw_ctrl_data[30..32].try_into().unwrap()),
            ),
            (283, 283, 283, 283)
        );
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
            u32::from_le_bytes(table.raw_ctrl_data[0..4].try_into().unwrap()),
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

        materialize_cell_list_header_contract(&mut cell, true, &mut report);

        assert_eq!(cell.list_header_width_ref & 0x0001, 0x0001);
        assert_eq!(cell.raw_list_extra.len(), 13);
        assert_eq!(
            u32::from_le_bytes(cell.raw_list_extra[0..4].try_into().unwrap()),
            2266
        );
        assert!(cell.raw_list_extra[4..].iter().all(|&byte| byte == 0));
        assert_eq!(report.cells_list_header_contract_materialized, 1);

        materialize_cell_list_header_contract(&mut cell, true, &mut report);
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

        materialize_cell_list_header_contract(&mut cell, false, &mut report);

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
    fn stage3_horizontal_cell_with_inner_margin_gets_bit16() {
        let mut cell = make_cell_with_inner_margin(true, 0);
        let mut report = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut report);
        assert_eq!(cell.text_direction, 1, "가로 셀에 bit 16 이 OR 되어야 함");
        assert_eq!(report.cells_list_attr_bit16_set, 1);
    }

    #[test]
    fn stage3_vertical_cell_already_has_bit16_no_change() {
        let mut cell = make_cell_with_inner_margin(true, 1);
        let mut report = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut report);
        // 세로 셀 (text_direction=1) 은 이미 bit 16 = 1 이므로 변경 불필요
        assert_eq!(cell.text_direction, 1);
        assert_eq!(report.cells_list_attr_bit16_set, 0);
    }

    #[test]
    fn stage3_no_inner_margin_no_change() {
        let mut cell = make_cell_with_inner_margin(false, 0);
        let mut report = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut report);
        assert_eq!(cell.text_direction, 0);
        assert_eq!(report.cells_list_attr_bit16_set, 0);
    }

    #[test]
    fn stage3_list_attr_byte_layout_has_bit16_after_adapter() {
        // serializer/control.rs:429 의 list_attr 합성식과 동일:
        //   list_attr = (text_direction << 16) | (v_align << 21)
        // 어댑터가 text_direction=1 으로 만든 후 출력 list_attr 의 bit 16 이 1 인지 확인.
        let mut cell = make_cell_with_inner_margin(true, 0);
        let mut report = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut report);

        let v_align_code: u32 = 0; // VerticalAlign::Top
        let list_attr: u32 = ((cell.text_direction as u32) << 16) | (v_align_code << 21);
        assert_eq!(list_attr & (1 << 16), 1 << 16, "list_attr 의 bit 16 = 1");

        // 한컴 파서 해석 (parser/control.rs:371) 와 일치:
        let recovered_apply_inner_margin = (list_attr >> 16) & 0x01 != 0;
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
        // 1차 호출 후 text_direction=1, apply_inner_margin=true
        assert_eq!(cell.text_direction, 1);

        let mut r2 = AdapterReport::new();
        adapt_cell_list_attr(&mut cell, &mut r2);
        // 2차 호출은 text_direction == 1 이므로 변경 없음 (가드에 막힘)
        assert_eq!(cell.text_direction, 1);
        assert_eq!(r2.cells_list_attr_bit16_set, 0);
    }
}
