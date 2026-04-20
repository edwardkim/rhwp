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

use crate::model::control::Control;
use crate::model::document::{Document, Section};
use crate::model::paragraph::Paragraph;
use crate::model::table::{Cell, Table};
use crate::parser::FileFormat;

use super::common_obj_attr_writer::serialize_common_obj_attr;

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
    /// `cell.list_attr bit 16` 보강 횟수 (Stage 3)
    pub cells_list_attr_bit16_set: u32,
    /// `Control::SectionDef` 컨트롤 삽입 횟수 (Stage 4 — 섹션 개수)
    pub section_def_controls_inserted: u32,
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
                + self.cells_list_attr_bit16_set
                + self.section_def_controls_inserted)
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

    // Stage 4: SectionDef 컨트롤 삽입 (HWPX 파서가 만들지 않으므로 직렬화기가 PAGE_DEF 출력 못 함)
    for section in &mut doc.sections {
        insert_section_def_control(section, &mut report);
    }

    // Stage 2/3: 표 ctrl_data + 셀 list_attr (raw_ctrl_data 합성)
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            adapt_paragraph(para, &mut report);
        }
    }

    report
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
    first_para
        .controls
        .insert(0, Control::SectionDef(Box::new(section.section_def.clone())));
    report.section_def_controls_inserted += 1;
}

fn adapt_paragraph(para: &mut Paragraph, report: &mut AdapterReport) {
    for ctrl in &mut para.controls {
        if let Control::Table(table) = ctrl {
            adapt_table(table, report);
        }
    }
}

fn adapt_table(table: &mut Table, report: &mut AdapterReport) {
    // 1. raw_ctrl_data 합성 (HWPX 출처는 비어있음)
    if table.raw_ctrl_data.is_empty() {
        table.raw_ctrl_data = serialize_common_obj_attr(&table.common);
        report.tables_ctrl_data_synthesized += 1;
    }

    // 2. attr 동기화: serializer/control.rs:349 가 raw_ctrl_data 를 그대로 쓰므로
    //    raw_ctrl_data 안의 attr 비트가 진실. table.attr 자체는 직렬화기 경로에서
    //    raw_ctrl_data 가 비어있을 때만 사용되므로 추가 작업 불필요.
    //    다만 IR 의 일관성을 위해 (다른 코드가 table.attr 을 읽을 가능성) 동기화.
    if table.attr == 0 && !table.raw_ctrl_data.is_empty() && table.raw_ctrl_data.len() >= 4 {
        let attr = u32::from_le_bytes([
            table.raw_ctrl_data[0],
            table.raw_ctrl_data[1],
            table.raw_ctrl_data[2],
            table.raw_ctrl_data[3],
        ]);
        if attr != 0 {
            table.attr = attr;
            report.tables_attr_packed += 1;
        }
    }

    // 셀별 보강 + 내부 문단 재귀 (중첩 표 대응)
    for cell in &mut table.cells {
        adapt_cell_list_attr(cell, report);
        for cpara in &mut cell.paragraphs {
            adapt_paragraph(cpara, report);
        }
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
    if source_format != FileFormat::Hwpx {
        return AdapterReport::new().no_op("source_format != Hwpx");
    }
    convert_hwpx_to_hwp_ir(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_doc_no_change() {
        let mut doc = Document::default();
        let report = convert_hwpx_to_hwp_ir(&mut doc);
        assert!(!report.changed_anything());
        assert!(report.skipped_reason.is_none());
    }

    #[test]
    fn hwp_source_no_op_via_filter() {
        let mut doc = Document::default();
        let report = convert_if_hwpx_source(&mut doc, FileFormat::Hwp);
        assert_eq!(report.skipped_reason.as_deref(), Some("source_format != Hwpx"));
    }

    #[test]
    fn idempotent_when_called_twice() {
        let mut doc = Document::default();
        let r1 = convert_hwpx_to_hwp_ir(&mut doc);
        let r2 = convert_hwpx_to_hwp_ir(&mut doc);
        // 두 번째 호출은 변경 없음 (이미 정규화됨).
        assert_eq!(r2.tables_ctrl_data_synthesized, 0);
        assert_eq!(r1, r2);
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
        assert!(recovered_apply_inner_margin, "재파싱 시 apply_inner_margin 회복");
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
