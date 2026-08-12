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
use crate::model::document::{
    Document, HwpVersion, Section, SectionDef, HWP3_ORIGIN_STREAM_PATH, HWPX_ORIGIN_STREAM_PATH,
};
use crate::model::image::Picture;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{common_obj_offsets, OleShape, ShapeObject, TextBox};
use crate::model::style::{BorderFill, BorderLineType, Fill, FillType};
use crate::model::table::{Cell, Table, TablePageBreak};
use crate::parser::FileFormat;

use super::common_obj_attr_writer::serialize_common_obj_attr;
use super::hwpx_master_page_slots::materialize_hwp5_master_page_slots;
// [#4400] bit packing 은 serializer 소유 — document_core::converters 는 더 이상 이 로직을
// 직접 갖지 않는다.
use crate::serializer::control::pack_common_attr_bits;

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
    /// `cell.list_attr bit 16` 보강 횟수 (Stage 3)
    pub cells_list_attr_bit16_set: u32,
    /// HWPX 출처 셀 LIST_HEADER width_ref/raw_list_extra materialize 횟수
    pub cells_list_header_contract_materialized: u32,
    /// paragraph/char shape 참조 BorderFill 무채움 정규화 횟수
    pub border_fills_no_fill_normalized: u32,
    /// HWPX 출처 FileHeader를 HWP5 compressed 저장 관례로 보정한 횟수
    pub file_header_compression_normalized: u32,
    /// [#3706] HWP3 출처 FileHeader 버전(major=3)을 HWP5 버전으로 실체화한 횟수
    pub file_header_version_materialized: u32,
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
    /// HWPX 희소 바탕쪽을 HWP5 `Both`/`Odd` 저장 슬롯으로 명시화한 구역 수 (#3930)
    pub master_page_apply_slots_materialized: u32,
    /// [#2767] 캡션이 있는 그림(gso `$pic`) CTRL_HEADER 의 한컴 캡션 비트(bit 29,
    /// 0x2000_0000) 보강 횟수. 표는 이미 `materialize_table_ctrl_header_attr` 로
    /// 보강되지만 그림은 빠져 있었다(전 코퍼스 실측 80/80 이 개체 종류와 무관하게
    /// 이 비트를 요구).
    pub picture_caption_common_attr_materialized: u32,
    /// [#4099] HWPX `<hp:chart>` OleShape 를 `<hp:default>` fallback OLE 로 접은 횟수.
    pub chart_ole_folded_to_fallback: u32,
    /// [#4099] fallback 이 없어 접지 못하고 참조만 비운 차트 OleShape 수.
    pub chart_ole_without_fallback: u32,
    /// [#4099] HWP5 산출에서 제거한 `ooxml_chart` BinDataContent 수.
    pub chart_bin_data_contents_removed: u32,
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
                + self.cells_list_attr_bit16_set
                + self.cells_list_header_contract_materialized
                + self.border_fills_no_fill_normalized
                + self.file_header_compression_normalized
                + self.file_header_version_materialized
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
                + self.master_page_line_rendering_size_ratio_materialized
                + self.master_page_apply_slots_materialized
                + self.picture_caption_common_attr_materialized
                + self.chart_ole_folded_to_fallback
                + self.chart_ole_without_fallback
                + self.chart_bin_data_contents_removed)
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
    let master_page_apply_slots_materialized = materialize_hwp5_master_page_slots(doc);
    let mut report = convert_to_hwp_ir(doc, true);
    report.master_page_apply_slots_materialized = master_page_apply_slots_materialized;
    report
}

/// HWPX/HWP3 출처 IR 을 HWP 직렬화기가 기대하는 형태로 정규화한다.
///
/// 한컴 HWP5 스트림은 출처와 관계없이 구역당 `PAGE_BORDER_FILL` 레코드 세 개를
/// 요구한다. HWPX 원본의 단일 BOTH XML 구조 보존은 이 변환을 생략하는 대신,
/// `DocumentCore` HWP export 경계에서 저장 뒤 PBF overlay를 되돌려 보장한다.
fn convert_to_hwp_ir(doc: &mut Document, source_is_hwpx: bool) -> AdapterReport {
    let mut report = AdapterReport::new();

    normalize_file_header_for_hwp(doc, &mut report);
    normalize_page_border_fills_for_hwp(doc);
    // [#4099] 도형을 건드리는 첫 패스여야 한다 — 뒤 패스가 바깥 차트 OleShape 에 가한
    // 변경은 fold 로 통째로 버려지고, BinData 순서 materialize 는 fold 가 올려놓은
    // 진짜 `bin_data_id` 를 봐야 remap 이 맞는다.
    fold_hwpx_chart_ole_for_hwp(doc, &mut report);
    normalize_picture_geometry_for_hwp(doc, source_is_hwpx);
    normalize_doc_properties_for_hwp(doc, &mut report);
    materialize_hwp5_bin_data_order(doc, &mut report);
    normalize_bin_data_for_hwp(doc, &mut report);

    // Stage 4: SectionDef 컨트롤 삽입 (HWPX 파서가 만들지 않으므로 직렬화기가 PAGE_DEF 출력 못 함)
    for (section_idx, section) in doc.sections.iter_mut().enumerate() {
        adapt_section_def(&mut section.section_def, &mut report);
        insert_section_def_control(section, &mut report);
        materialize_following_section_break_type(section_idx, section, &mut report);

        // HWPX -> HWP 어댑터가 SectionDef, 바탕쪽, 문단 제어를 물질화했으므로
        // 이전 BodyText raw stream이 있다면 재사용하면 안 된다. HWP5 저장본은
        // 반드시 동기화된 inline SectionDef와 master-page LIST_HEADER에서 다시 쓴다.
        section.raw_stream = None;
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

/// HWPX 파서가 `Chart/chartN.xml` 파트에 붙이는 확장자 표식
/// (`parser/hwpx/mod.rs`, Task #195 규약).
const OOXML_CHART_EXTENSION: &str = "ooxml_chart";

/// 모든 `OleShape` 를 가변으로 방문한다.
///
/// 순회 골격은 `normalize_picture_geometry_for_hwp` 에서 가져왔다 — 이 파일의 네 워커
/// 중 컨테이너 커버리지가 가장 넓은 쪽이다.
///
/// | 컨테이너 | bin order(`collect_bin_order_*`) | bin ref remap(`remap_bin_refs_*`) | 이 워커 |
/// |---|---|---|---|
/// | 표 셀 · 그룹 자식 · 머리말/꼬리말/각주/미주/숨은설명 | ✓ | ✓ | ✓ |
/// | `drawing.text_box` · `drawing.caption` | ✓ | ✓ | ✓ |
/// | `pic`/`group`/`chart`/`ole` own caption | 일부 | 일부 | ✓ |
/// | `Control::Field.memo_paragraphs` | ✗ | ✗ | ✓ |
/// | `Control::SectionDef.master_pages` | ✗ | ✗ | ✓ |
///
/// 네 워커를 하나의 visitor 로 통합하는 것은 커버리지가 서로 달라 동작이 바뀔 수 있는
/// 별개 리팩터다 — 여기서는 좁은 타입으로만 골격을 재사용한다.
///
/// `chart_switch_fallback` 안쪽은 방문하지 않는다. HWPX 파서는 fallback 안에 또 다른
/// 차트를 만들지 않고, `fold_hwpx_chart_ole_for_hwp` 가 그 상자를 곧 없앤다.
fn for_each_ole_mut(doc: &mut Document, f: &mut dyn FnMut(&mut OleShape)) {
    fn walk_paragraphs(paragraphs: &mut [Paragraph], f: &mut dyn FnMut(&mut OleShape)) {
        for para in paragraphs {
            walk_controls(&mut para.controls, f);
        }
    }

    fn walk_caption(caption: &mut crate::model::shape::Caption, f: &mut dyn FnMut(&mut OleShape)) {
        walk_paragraphs(&mut caption.paragraphs, f);
    }

    fn walk_master_pages(
        master_pages: &mut [crate::model::header_footer::MasterPage],
        f: &mut dyn FnMut(&mut OleShape),
    ) {
        for master_page in master_pages {
            walk_paragraphs(&mut master_page.paragraphs, f);
        }
    }

    fn walk_drawing(
        drawing: &mut crate::model::shape::DrawingObjAttr,
        f: &mut dyn FnMut(&mut OleShape),
    ) {
        if let Some(text_box) = &mut drawing.text_box {
            walk_paragraphs(&mut text_box.paragraphs, f);
        }
        if let Some(caption) = &mut drawing.caption {
            walk_caption(caption, f);
        }
    }

    fn walk_shape(shape: &mut ShapeObject, f: &mut dyn FnMut(&mut OleShape)) {
        match shape {
            ShapeObject::Picture(pic) => {
                if let Some(caption) = &mut pic.caption {
                    walk_caption(caption, f);
                }
            }
            ShapeObject::Group(group) => {
                for child in &mut group.children {
                    walk_shape(child, f);
                }
                if let Some(caption) = &mut group.caption {
                    walk_caption(caption, f);
                }
            }
            ShapeObject::Chart(chart) => {
                walk_drawing(&mut chart.drawing, f);
                if let Some(caption) = &mut chart.caption {
                    walk_caption(caption, f);
                }
            }
            ShapeObject::Ole(ole) => {
                // 캡션·글상자를 먼저 훑는다. `f` 가 fold 로 OleShape 를 통째로
                // 갈아끼우므로, 뒤에 방문하면 교체된 쪽을 다시 보게 된다.
                walk_drawing(&mut ole.drawing, f);
                if let Some(caption) = &mut ole.caption {
                    walk_caption(caption, f);
                }
                f(ole);
            }
            _ => {
                if let Some(drawing) = shape.drawing_mut() {
                    walk_drawing(drawing, f);
                }
            }
        }
    }

    fn walk_controls(controls: &mut [Control], f: &mut dyn FnMut(&mut OleShape)) {
        for control in controls {
            match control {
                Control::Picture(pic) => {
                    if let Some(caption) = &mut pic.caption {
                        walk_caption(caption, f);
                    }
                }
                Control::Shape(shape) => walk_shape(shape, f),
                Control::Table(table) => {
                    for cell in &mut table.cells {
                        walk_paragraphs(&mut cell.paragraphs, f);
                    }
                    if let Some(caption) = &mut table.caption {
                        walk_caption(caption, f);
                    }
                }
                Control::Header(header) => walk_paragraphs(&mut header.paragraphs, f),
                Control::Footer(footer) => walk_paragraphs(&mut footer.paragraphs, f),
                Control::Footnote(footnote) => walk_paragraphs(&mut footnote.paragraphs, f),
                Control::Endnote(endnote) => walk_paragraphs(&mut endnote.paragraphs, f),
                Control::HiddenComment(comment) => walk_paragraphs(&mut comment.paragraphs, f),
                Control::Field(field) => walk_paragraphs(&mut field.memo_paragraphs, f),
                Control::SectionDef(section_def) => {
                    walk_master_pages(&mut section_def.master_pages, f)
                }
                _ => {}
            }
        }
    }

    for section in doc.sections.iter_mut() {
        walk_paragraphs(&mut section.paragraphs, f);
        walk_master_pages(&mut section.section_def.master_pages, f);
    }
}

/// [#4099] HWPX 차트를 HWP5 가 참조할 수 있는 OLE 하나로 접는다.
///
/// HWPX 파서는 `<hp:switch>` 의 `<hp:case>` 브랜치를 채택해 **가상 id**
/// `bin_data_id = 60000+N` 을 세우고, `<hp:default>` 의 진짜 OLE 를
/// `chart_switch_fallback` 에 매달아 둔다(#3546 — HWPX 저장 시 원형 재방출의 재료).
/// 그 가상 id 는 zip 파트 `Chart/chartN.xml` 을 가리키므로 **HWP5 에는 대응물이 없다.**
/// 손대지 않으면 세 가지가 한꺼번에 깨진다.
///
/// - `serialize_ole_data` 가 `60001` 을 그대로 기록 → HWP5 DocInfo 에 없는 storage 참조
/// - `find_bin_data_info_with_compress` 폴백이 `/BinData/BINEA61.ooxml_chart` 라는
///   **DocInfo 미등록 정크 스트림**을 만든다
/// - 재파싱이 그 스트림을 읽지 않아 `--verify` 가 `bin_data_content count` 로 실패
///
/// ## 왜 fallback 을 통째로 채택하는가
///
/// 한컴이 저장한 같은 문서의 `.hwp` 를 대조하면 답이 나온다. GenShape CTRL_HEADER 의
/// `instance_id` 가 **0** 인데, 이는 `<hp:default><hp:ole instid="0">` 의 값이지
/// `<hp:chart id="1117817146">` 의 값이 아니다 — **한컴 자신의 HWPX→HWP5 변환도
/// fallback 브랜치를 쓴다.** 그 OLE 가 가리키는 `BinData/ole1.ole` 안에는 한컴이 실제로
/// 읽는 중첩 `OOXMLChartContents` 가 들어 있다(#4055 실측: 그 사본만 고쳐도 렌더에
/// 반영된다).
///
/// 두 브랜치는 `sz`/`pos`/`outMargin`/`zOrder`/`textWrap` 이 전부 같고, 모델에 남는
/// 차이는 `bin_data_id`·`instance_id`·`rotate_image`·`drawing_aspect`·`caption` 뿐이다
/// (`parse_common_shape_children` 이 `orgSz`/`curSz`/`flip`/`lineShape` 를 IR 에 싣지
/// 않는다). 그중 HWP5 로 나가는 것은 앞의 둘이고 둘 다 fallback 쪽이 정답이다.
///
/// ## fallback 이 없으면
///
/// `<hp:switch>` 없이 `<hp:chart>` 만 있거나 `<hp:default>` 가 빠진 case-only switch 는
/// 접을 대상이 없다(코퍼스 0건, 파서 주석도 "아직 보지 못한 변형"). 참조를 비워
/// 정크 스트림과 dangling 을 둘 다 막고 placeholder 로 남긴다.
///
/// 차트 XML 을 `mini_cfb` 로 OLE CFB 에 싸는 길도 있다 — #4097 이
/// `build_cfb_with_root_clsid` 를 넣어 도구는 갖춰졌다. 다만 참조할 원본 CLSID 가 없어
/// `{4C3DA137-DC90-47B9-9BED-59DAE352A280}` 를 하드코딩해야 하고, `OOXMLChartContents`
/// 하나만 든 CFB 를 한컴이 받아들이는지는 미검증이다(#4055 는 기존 CFB 를 수정했을 뿐
/// 새로 만들지 않았다). 실물 변종이 관측되면 그때 채운다.
fn fold_hwpx_chart_ole_for_hwp(doc: &mut Document, report: &mut AdapterReport) {
    let mut folded = 0u32;
    let mut orphaned = 0u32;

    for_each_ole_mut(doc, &mut |ole: &mut OleShape| {
        if ole.chart_id_ref.is_none() {
            return;
        }
        match ole.chart_switch_fallback.take() {
            Some(fallback) => {
                // [#4319] 파서가 `<hp:chart>` 와 `<hp:default><hp:ole>` 양쪽의
                // `<hp:caption>` 을 읽는다. 실물은 두 브랜치가 같은 캡션을 중복
                // 기록하지만, chart 쪽에만 있는 경우 fallback 채택으로 조용히
                // 사라지지 않게 이월한다.
                let chart_caption = ole.caption.take();
                *ole = *fallback;
                if ole.caption.is_none() {
                    ole.caption = chart_caption;
                }
                debug_assert!(
                    ole.chart_id_ref.is_none() && ole.chart_switch_fallback.is_none(),
                    "fallback 브랜치에는 HWPX 차트 표식이 없어야 한다 — 멱등성 계약"
                );
                debug_assert!(
                    ole.raw_tag_data.is_empty(),
                    "HWPX 출신 OleShape 는 raw_tag_data 가 비어 있어야 한다 \
                     (비면 serialize_ole_data 가 bin_data_id 필드를 무시한다)"
                );
                folded += 1;
            }
            None => {
                ole.bin_data_id = 0;
                ole.chart_id_ref = None;
                orphaned += 1;
            }
        }
    });

    // 차트 XML 은 HWP5 에 담을 자리가 없다. 남기면 `cfb_writer` 폴백이 DocInfo 미등록
    // 정크 스트림을 만들고 `--verify` 가 개수 불일치로 실패한다. 한컴이 읽는 표현은
    // 중첩 CFB 안의 `OOXMLChartContents` 사본이므로 내용 손실도 없다.
    let before = doc.bin_data_content.len();
    doc.bin_data_content
        .retain(|content| content.extension != OOXML_CHART_EXTENSION);
    let removed = (before - doc.bin_data_content.len()) as u32;

    if orphaned > 0 {
        eprintln!(
            "경고: fallback OLE 가 없는 HWPX 차트 {orphaned}개는 HWP5 로 옮기지 못해 \
             빈 개체로 남깁니다 (#4099)"
        );
    }

    report.chart_ole_folded_to_fallback += folded;
    report.chart_ole_without_fallback += orphaned;
    report.chart_bin_data_contents_removed += removed;
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
            // [#2736] 그림 캡션 문단도 순회한다. 표 캡션(아래 arm)·도형 캡션
            // (collect_bin_order_from_shape)은 이미 방문하는데 그림 캡션만 빠져 있어,
            // 같은 캡션 컨테이너인데 소유 개체 종류에 따라 순회 여부가 갈렸다.
            if let Some(caption) = &pic.caption {
                collect_bin_order_from_paragraphs(&caption.paragraphs, bin_count, order, seen);
            }
        }
        Control::Shape(shape) => collect_bin_order_from_shape(shape, bin_count, order, seen),
        Control::Table(table) => {
            for cell in &table.cells {
                collect_bin_order_from_paragraphs(&cell.paragraphs, bin_count, order, seen);
            }
            if let Some(caption) = &table.caption {
                collect_bin_order_from_paragraphs(&caption.paragraphs, bin_count, order, seen);
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
        Control::HiddenComment(comment) => {
            collect_bin_order_from_paragraphs(&comment.paragraphs, bin_count, order, seen);
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
        if let Some(caption) = &drawing.caption {
            collect_bin_order_from_paragraphs(&caption.paragraphs, bin_count, order, seen);
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
            // [#2736] 그림 캡션 문단 안의 그림도 리맵해야 한다. 미방문 시 캡션 안 그림의
            // bin_data_id 가 재정렬 이전 번호로 남아 엉뚱한 이미지로 해석된다 —
            // 표 캡션에 대한 동형 결함이 이미 table_caption_picture_bin_ref_is_remapped
            // 로 회귀 고정돼 있고, 그림 캡션이 그 미수정 형제였다.
            if let Some(caption) = &mut pic.caption {
                remap_bin_refs_in_paragraphs(&mut caption.paragraphs, remap);
            }
        }
        Control::Shape(shape) => remap_bin_refs_in_shape(shape, remap),
        Control::Table(table) => {
            for cell in &mut table.cells {
                remap_bin_refs_in_paragraphs(&mut cell.paragraphs, remap);
            }
            if let Some(caption) = &mut table.caption {
                remap_bin_refs_in_paragraphs(&mut caption.paragraphs, remap);
            }
        }
        Control::Header(header) => remap_bin_refs_in_paragraphs(&mut header.paragraphs, remap),
        Control::Footer(footer) => remap_bin_refs_in_paragraphs(&mut footer.paragraphs, remap),
        Control::Footnote(footnote) => {
            remap_bin_refs_in_paragraphs(&mut footnote.paragraphs, remap)
        }
        Control::Endnote(endnote) => remap_bin_refs_in_paragraphs(&mut endnote.paragraphs, remap),
        // [#2767] adapt 워크·border-fill 워크는 이미 HiddenComment 를 방문한다
        // (#2467 근거). remap 워크만 빠져 있어 숨은설명 안 그림이 재정렬 후
        // 엉뚱한 이미지를 가리키는 채로 남았다.
        Control::HiddenComment(comment) => {
            remap_bin_refs_in_paragraphs(&mut comment.paragraphs, remap)
        }
        _ => {}
    }
}

fn remap_bin_refs_in_shape(shape: &mut ShapeObject, remap: &[u16]) {
    match shape {
        ShapeObject::Picture(pic) => {
            pic.image_attr.bin_data_id = remap_bin_ref(pic.image_attr.bin_data_id, remap);
            // [#2767] 그룹 내부 그림(ShapeObject::Picture)은 drawing_mut()이 항상
            // None 이라(모델상 DrawingObjAttr 를 갖지 않음) 아래 공통 캡션 remap
            // 경로를 타지 않는다. Picture 자신의 caption 필드를 직접 재귀해야 한다.
            if let Some(caption) = &mut pic.caption {
                remap_bin_refs_in_paragraphs(&mut caption.paragraphs, remap);
            }
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
        if let Some(caption) = &mut drawing.caption {
            remap_bin_refs_in_paragraphs(&mut caption.paragraphs, remap);
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
/// 개체 요소 속성에 대한 가변 접근 — 도형 종류마다 보관 위치가 달라 한 곳에 모은다.
fn shape_attr_mut(
    obj: &mut crate::model::shape::ShapeObject,
) -> Option<&mut crate::model::shape::ShapeComponentAttr> {
    use crate::model::shape::ShapeObject as S;
    Some(match obj {
        S::Line(s) => &mut s.drawing.shape_attr,
        S::Rectangle(s) => &mut s.drawing.shape_attr,
        S::Ellipse(s) => &mut s.drawing.shape_attr,
        S::Arc(s) => &mut s.drawing.shape_attr,
        S::Polygon(s) => &mut s.drawing.shape_attr,
        S::Curve(s) => &mut s.drawing.shape_attr,
        S::Group(g) => &mut g.shape_attr,
        S::Picture(p) => &mut p.shape_attr,
        S::Chart(c) => &mut c.drawing.shape_attr,
        S::Ole(o) => &mut o.drawing.shape_attr,
    })
}

/// [#3676] 그림 개체의 사각형 4점과 자르기 정보가 비어 있으면 크기에서 채운다.
///
/// `HWPTAG_SHAPE_COMPONENT_PICTURE` 는 개체의 네 꼭짓점을 (x,y) 쌍 4개로 담는다.
/// 한컴이 저장한 문서는 `(0,0) (w,0) (w,h) (0,h)` 형태이고 자르기 우/하단에 원본
/// 크기가 들어간다. HWP3 파서는 이 값들을 채우지 않아 **전부 0** 으로 나갔고,
/// 크기 0 짜리 그림이 든 문서를 한컴이 거부했다(그림 1개짜리 34KB 문서도 거부 —
/// 문서 크기가 아니라 그림 유무가 판별자).
///
/// 크기는 `SHAPE_COMPONENT` 가 이미 갖고 있다(현재 폭/높이). 그것으로 사각형을
/// 만들고, 자르기는 원본 크기 기준 전체 영역으로 둔다. 이미 채워진 그림은
/// 건드리지 않으므로 HWPX·HWP5 경로는 무영향이다.
fn normalize_picture_geometry_for_hwp(doc: &mut Document, source_is_hwpx: bool) {
    fn fill(pic: &mut crate::model::image::Picture, source_is_hwpx: bool) {
        // HWPX `hp:imgDim`은 논리 원본 이미지 크기이며 IR에 그대로 보존한다. 다만
        // 한컴 2020의 HWPX -> HWP 저장본은 SC_PICTURE extra(18 byte) 속의 별도
        // original-width/height 칸을 0으로 쓴다. 이 칸에 imgDim을 복사하면 묶음
        // 그림을 인쇄할 때 한컴이 크기를 다시 해석해 표지가 크게 어긋난다.
        if source_is_hwpx && pic.raw_picture_extra.is_empty() {
            // HWPX hc:img는 bright, contrast 순서지만 한컴 2020이 HWP5
            // SC_PICTURE에 저장하는 두 i8 칸은 반대 순서다. HWP5 serializer는
            // 모델 순서대로 기록하므로 이 경계에서만 바꾼다. raw extra를 함께
            // 채워 두므로 adapter 재호출 시 다시 교환되지 않는다.
            std::mem::swap(&mut pic.image_attr.brightness, &mut pic.image_attr.contrast);
            pic.raw_picture_extra.reserve_exact(18);
            pic.raw_picture_extra.push(pic.border_opacity);
            pic.raw_picture_extra
                .extend_from_slice(&pic.instance_id.to_le_bytes());
            pic.raw_picture_extra
                .extend_from_slice(&0_u32.to_le_bytes());
            pic.raw_picture_extra
                .extend_from_slice(&0_u32.to_le_bytes());
            pic.raw_picture_extra
                .extend_from_slice(&0_u32.to_le_bytes());
            pic.raw_picture_extra
                .push(pic.image_attr.transparency_alpha_byte());
        }
        // `SHAPE_COMPONENT` 의 local file version. 한컴 저장본은 1, HWP3 변환본은 0 이다
        // (같은 그림의 바이트 대조로 확인). 기하와 무관하게 항상 맞춘다.
        if pic.shape_attr.local_file_version == 0 {
            pic.shape_attr.local_file_version = 1;
        }
        if pic.border_x.iter().any(|&v| v != 0) || pic.border_y.iter().any(|&v| v != 0) {
            return;
        }
        let w = if pic.shape_attr.current_width > 0 {
            pic.shape_attr.current_width as i32
        } else {
            pic.common.width as i32
        };
        let h = if pic.shape_attr.current_height > 0 {
            pic.shape_attr.current_height as i32
        } else {
            pic.common.height as i32
        };
        if w <= 0 || h <= 0 {
            return;
        }
        // `border_x`/`border_y` 는 이름과 달리 **디스크의 8개 스칼라를 앞뒤로 나눈
        // 것**이다. 실제 저장 순서는 (x,y) 쌍 4개다 — HWPX 파서가 같은 규약을
        // 문서화해 두었다(`parser/hwpx/section.rs`). 한컴 저장본 실측도 같다:
        // 원본 그림 하나가 `(0,0) (w,0) (w,h) (0,h)` 로 들어 있다.
        pic.border_x = [0, 0, w, 0];
        pic.border_y = [w, h, 0, h];
        if pic.crop.right == 0 && pic.crop.bottom == 0 {
            let ow = if pic.shape_attr.original_width > 0 {
                pic.shape_attr.original_width as i32
            } else {
                w
            };
            let oh = if pic.shape_attr.original_height > 0 {
                pic.shape_attr.original_height as i32
            } else {
                h
            };
            pic.crop.right = ow;
            pic.crop.bottom = oh;
        }
    }
    // HWP3 파서는 아래 컨테이너에 실제로 문단을 만든다: 표 셀/캡션, 그림·그리기
    // 개체 캡션, 글상자, 숨은설명, 머리말·꼬리말·각주·미주, 바탕쪽. 어느 하나라도
    // 빠뜨리면 그 안의 그림 또는 도형만 HWP5 계약(geometry/local-file-version)을
    // 잃고, 한컴은 문서 전체를 거부할 수 있다. 각 변환 단계가 독자 walker를 조금씩
    // 달리 두지 않도록 여기서는 모든 paragraph container를 하나의 재귀로 방문한다.
    fn walk_paragraphs(paragraphs: &mut [Paragraph], source_is_hwpx: bool) {
        for para in paragraphs {
            walk_controls(&mut para.controls, source_is_hwpx);
        }
    }

    fn walk_caption(caption: &mut crate::model::shape::Caption, source_is_hwpx: bool) {
        walk_paragraphs(&mut caption.paragraphs, source_is_hwpx);
    }

    fn walk_master_pages(
        master_pages: &mut [crate::model::header_footer::MasterPage],
        source_is_hwpx: bool,
    ) {
        for master_page in master_pages {
            walk_paragraphs(&mut master_page.paragraphs, source_is_hwpx);
        }
    }

    fn walk_drawing(drawing: &mut crate::model::shape::DrawingObjAttr, source_is_hwpx: bool) {
        if let Some(text_box) = &mut drawing.text_box {
            walk_paragraphs(&mut text_box.paragraphs, source_is_hwpx);
        }
        if let Some(caption) = &mut drawing.caption {
            walk_caption(caption, source_is_hwpx);
        }
    }

    fn walk_shape(shape: &mut ShapeObject, source_is_hwpx: bool) {
        // local file version 은 그림뿐 아니라 **모든 개체 요소**가 1 이어야 한다.
        // 한컴 저장본은 예외 없이 1 이고, HWP3 변환본은 도형(`$con`/`$rec` 등)만
        // 0 으로 남아 문서 전체가 거부됐다(그림만 고쳤을 때 20건 중 2건 잔존).
        if let Some(attr) = shape_attr_mut(shape) {
            if attr.local_file_version == 0 {
                attr.local_file_version = 1;
            }
        }

        match shape {
            ShapeObject::Picture(pic) => {
                fill(pic, source_is_hwpx);
                if let Some(caption) = &mut pic.caption {
                    walk_caption(caption, source_is_hwpx);
                }
            }
            ShapeObject::Group(group) => {
                for child in &mut group.children {
                    walk_shape(child, source_is_hwpx);
                }
                if let Some(caption) = &mut group.caption {
                    walk_caption(caption, source_is_hwpx);
                }
            }
            // Chart/OLE은 DrawingObjAttr의 caption과 별개로 HWP3 parser가 채우는
            // own caption을 가진다. 특히 HWP3 OLE fixup은 picture caption을
            // `ole.caption`으로 옮긴다. 둘 다 누락하면 0 geometry picture가 남는다.
            ShapeObject::Chart(chart) => {
                walk_drawing(&mut chart.drawing, source_is_hwpx);
                if let Some(caption) = &mut chart.caption {
                    walk_caption(caption, source_is_hwpx);
                }
            }
            ShapeObject::Ole(ole) => {
                walk_drawing(&mut ole.drawing, source_is_hwpx);
                if let Some(caption) = &mut ole.caption {
                    walk_caption(caption, source_is_hwpx);
                }
            }
            _ => {
                // Line/Rectangle/Ellipse/Arc/Polygon/Curve의 text box와 caption은
                // 모두 동일한 paragraph container이므로 같은 walker로 재귀한다.
                if let Some(drawing) = shape.drawing_mut() {
                    walk_drawing(drawing, source_is_hwpx);
                }
            }
        }
    }

    fn walk_controls(controls: &mut [Control], source_is_hwpx: bool) {
        for control in controls {
            match control {
                Control::Picture(pic) => {
                    fill(pic, source_is_hwpx);
                    if let Some(caption) = &mut pic.caption {
                        walk_caption(caption, source_is_hwpx);
                    }
                }
                Control::Shape(shape) => walk_shape(shape, source_is_hwpx),
                Control::Table(table) => {
                    for cell in &mut table.cells {
                        walk_paragraphs(&mut cell.paragraphs, source_is_hwpx);
                    }
                    if let Some(caption) = &mut table.caption {
                        walk_caption(caption, source_is_hwpx);
                    }
                }
                Control::Header(header) => walk_paragraphs(&mut header.paragraphs, source_is_hwpx),
                Control::Footer(footer) => walk_paragraphs(&mut footer.paragraphs, source_is_hwpx),
                Control::Footnote(footnote) => {
                    walk_paragraphs(&mut footnote.paragraphs, source_is_hwpx)
                }
                Control::Endnote(endnote) => {
                    walk_paragraphs(&mut endnote.paragraphs, source_is_hwpx)
                }
                Control::HiddenComment(comment) => {
                    walk_paragraphs(&mut comment.paragraphs, source_is_hwpx)
                }
                // HWPX memo field와 HWP3의 SectionDef control도 문단을 품을 수 있다.
                // 본문 SectionDef와는 별개 IR 인스턴스이므로 여기서도 안전하게 덮는다.
                Control::Field(field) => {
                    walk_paragraphs(&mut field.memo_paragraphs, source_is_hwpx)
                }
                Control::SectionDef(section_def) => {
                    walk_master_pages(&mut section_def.master_pages, source_is_hwpx)
                }
                _ => {}
            }
        }
    }

    for section in doc.sections.iter_mut() {
        walk_paragraphs(&mut section.paragraphs, source_is_hwpx);
        walk_master_pages(&mut section.section_def.master_pages, source_is_hwpx);
    }
}

/// [#3676] HWP 저장 구역마다 `HWPTAG_PAGE_BORDER_FILL` 을 **3개** 채운다
/// (양쪽/짝수쪽/홀수쪽).
///
/// 한컴이 저장한 문서와 rhwp 의 HWP5 왕복본은 예외 없이 3개다. HWP3 파서는
/// `extra_page_border_fills` 를 채우지 않아 **1개만** 나갔고, 그 스트림을 정상 파일에
/// 이식하면 한컴이 파일을 거부한다(스트림 단위 이분 탐색으로 확인 — DocInfo 이식은
/// 정상 열림, BodyText/Section0 이식만 거부).
///
/// 세 레코드는 한컴 원본에서도 내용이 완전히 동일하다(attr·간격 모두 같음) — 구분
/// 플래그가 아니라 **개수 자체가 규격**이므로 첫 레코드를 복제해 채운다.
/// HWPX 는 실제 문서에서 BOTH 하나만 갖는 것이 보통이지만, HWP 출력에도 같은 세 record가
/// 필요하다. HWPX live IR 원형은 호출 경계에서 해당 overlay를 복원한다.
fn normalize_page_border_fills_for_hwp(doc: &mut Document) {
    for section in doc.sections.iter_mut() {
        let sd = &mut section.section_def;
        while sd.extra_page_border_fills.len() < 2 {
            sd.extra_page_border_fills.push(sd.page_border_fill.clone());
        }
    }
}

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

    // [#3706, #3676 후속] HWP3 파서는 HWP5 컨테이너용 버전(5.0.3.0)을 `raw_data` 바이트
    // (32..36 = revision/build/minor/major)에만 기록하고, 필드 `version` 은
    // major=3 (메모리 전용 표시)으로 남긴다 — `serialize_file_header` 가
    // raw_data 를 우선 쓰는 것을 전제한 설계다 (`parser/hwp3/mod.rs`).
    // 그런데 본 함수가 아래에서 raw_data 를 버리므로 직렬화가 필드 경로로
    // 떨어져 HWP5 컨테이너에 버전 3 이 기록됐다(규격 위반 — 한컴 저장본은
    // 예외 없이 5.x). 버리기 전에 raw_data 의 5.x 버전을 필드로 회수하고,
    // 회수할 수 없으면 파서 기본값 5.0.3.0 으로 실체화한다.
    // 이미 5.x 인 경로(HWPX 파서는 5.1.0.0 을 필드에 직접 기록)는 무변경.
    if doc.header.version.major < 5 {
        let salvaged = doc
            .header
            .raw_data
            .as_deref()
            .filter(|raw| raw.len() >= 36 && raw[35] >= 5)
            .map(|raw| (raw[35], raw[34], raw[33], raw[32]));
        let (major, minor, build, revision) = salvaged.unwrap_or((5, 0, 3, 0));
        doc.header.version = HwpVersion {
            major,
            minor,
            build,
            revision,
        };
        report.file_header_version_materialized += 1;
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
        // 바탕쪽(master page) 문단 안 개체 참조도 수집한다. bin 워크
        // (materialize_hwp5_bin_data_order, remap_bin_refs_in_doc)와 adapt 워크가
        // 이미 바탕쪽을 순회하는데 이 collect 워크만 빠져 있었다.
        for master_page in &section.section_def.master_pages {
            for para in &master_page.paragraphs {
                collect_object_border_fill_refs_from_paragraph(para, &mut refs);
            }
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
            // [#2736] 그림 캡션 문단 안 개체 참조도 수집한다. `Control::Picture` arm 자체가
            // 없어 표 캡션(collect_table_border_fill_refs)·도형 캡션
            // (collect_object_border_fill_refs_from_shape)과 달리 그림 캡션만 빠져 있었고,
            // 미수집 시 normalize_paragraph_char_border_fills 가드가 실패해 캡션 안 개체
            // 채우기가 no-fill 로 정규화된다(#2467 과 동일 메커니즘).
            Control::Picture(pic) => {
                if let Some(caption) = &pic.caption {
                    for p in &caption.paragraphs {
                        collect_object_border_fill_refs_from_paragraph(p, refs);
                    }
                }
            }
            // 각주/미주/숨은설명 문단 안의 개체도 border_fill 을 참조할 수 있다.
            // 이 참조가 수집되지 않으면 normalize_paragraph_char_border_fills 의
            // 가드가 실패해, 문단 char-border 와 공유된 border_fill 이 no-fill 로
            // 정규화되어 컨테이너 안 개체의 채우기가 유실된다(#2467 adapt 워크와 동형).
            Control::Footnote(footnote) => {
                for p in &footnote.paragraphs {
                    collect_object_border_fill_refs_from_paragraph(p, refs);
                }
            }
            Control::Endnote(endnote) => {
                for p in &endnote.paragraphs {
                    collect_object_border_fill_refs_from_paragraph(p, refs);
                }
            }
            Control::HiddenComment(comment) => {
                for p in &comment.paragraphs {
                    collect_object_border_fill_refs_from_paragraph(p, refs);
                }
            }
            // 머리말/꼬리말 문단 안 개체도 다른 워크(bin order/remap, adapt)와 동일하게 순회.
            Control::Header(header) => {
                for p in &header.paragraphs {
                    collect_object_border_fill_refs_from_paragraph(p, refs);
                }
            }
            Control::Footer(footer) => {
                for p in &footer.paragraphs {
                    collect_object_border_fill_refs_from_paragraph(p, refs);
                }
            }
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
        // 도형 캡션 문단 안 개체 참조도 수집(bin order/remap 워크와 동형).
        // #2483 은 표 캡션을 다뤘고, 이 도형 drawing.caption 은 별개 경로다.
        if let Some(caption) = &drawing.caption {
            for para in &caption.paragraphs {
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
    // 표 캡션 문단 안의 개체 참조도 수집(#2467 과 동일 근거).
    if let Some(caption) = &table.caption {
        for para in &caption.paragraphs {
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
                adapt_picture_href_ctrl_data(pic, &mut ctrl_data_records[idx], report);
                // [#2736] 그림 캡션 문단도 보강한다. 표 캡션은 adapt_table_with_context 가
                // 이미 보강하고, bin order/remap·border_fill 수집 워크는 캡션을 순회하는데
                // adapt 워크만 그림 캡션을 건너뛰었다. 실측(samples/hwpx/aift.hwpx): 본문
                // 921/921·표 캡션 2/2 가 방문되는 동안 그림 캡션 9/9 전부 미방문.
                // 미방문 시 캡션 안 그림 href·표 raw_ctrl_data·수식 보정이 물질화되지 않고
                // 문단 header tail 도 10바이트로 남아 PARA_HEADER 가 22/24 로 섞인다.
                if let Some(caption) = &mut pic.caption {
                    adapt_paragraphs_with_context(&mut caption.paragraphs, report, context);
                }
                materialize_picture_caption_common_attr(pic, report);
            }
            Control::Shape(shape) => adapt_shape_with_context(shape, report, context),
            Control::Equation(eq) => adapt_equation(eq, report),
            // 각주/미주/숨은설명 문단도 body 문단과 동일하게 보강해야 한다.
            // bin 참조 수집·리맵 워크(collect_bin_order/remap_bin_refs)는 이미 이들을
            // 재귀하므로 adapt 워크만 빠져 있으면, 이 안의 그림 href·표 ctrl_data 등이
            // 물질화되지 않고 HWPX→HWP 변환 시 유실된다.
            Control::Footnote(footnote) => {
                adapt_paragraphs_with_context(&mut footnote.paragraphs, report, context)
            }
            Control::Endnote(endnote) => {
                adapt_paragraphs_with_context(&mut endnote.paragraphs, report, context)
            }
            Control::HiddenComment(comment) => {
                adapt_paragraphs_with_context(&mut comment.paragraphs, report, context)
            }
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
    const HANCOM_SINGLE_BOTH_MASTER_PAGE_FLAGS: u32 = 0x2000_0000;
    const HANCOM_SINGLE_ODD_MASTER_PAGE_FLAGS: u32 = 0x8000_0000;
    const MASTER_PAGE_FLAGS_MASK: u32 = 0xe000_0000;

    // 희소 HWPX 바탕쪽은 직전 슬롯 정규화에서 1→2개로 늘 수 있으므로 입력 flags가 아니라
    // 최종 슬롯 개수만 HWP5 SECTION_DEF 계약의 기준으로 쓴다.
    if section_def.master_pages.len() != 1 {
        return;
    }

    // HWP 2020 HWPX -> HWP 저장본의 단일 Odd LIST_HEADER는 0x80000000이다.
    // 이 비트는 이전 구역의 짝수 바탕쪽을 유지한 채 현재 구역의 홀수 바탕쪽만
    // 교체하는 저장 계약이다. 단일 Both(기존 한컴 저장 계약)는 0x20000000을 쓴다.
    let single_master = &section_def.master_pages[0];
    let master_page_flags = match single_master.apply_to {
        crate::model::header_footer::HeaderFooterApply::Odd => HANCOM_SINGLE_ODD_MASTER_PAGE_FLAGS,
        crate::model::header_footer::HeaderFooterApply::Both
        | crate::model::header_footer::HeaderFooterApply::Even => {
            HANCOM_SINGLE_BOTH_MASTER_PAGE_FLAGS
        }
    };
    let expected = (section_def.flags & !MASTER_PAGE_FLAGS_MASK) | master_page_flags;
    if section_def.flags != expected {
        section_def.flags = expected;
        report.section_def_single_master_page_flags_materialized += 1;
    }
}

fn materialize_multi_master_page_flags(section_def: &mut SectionDef, report: &mut AdapterReport) {
    const HANCOM_MULTI_MASTER_PAGE_FLAGS: u32 = 0xC000_0000;
    const MASTER_PAGE_FLAGS_MASK: u32 = 0xe000_0000;

    if section_def.master_pages.len() < 2 {
        return;
    }

    let expected = (section_def.flags & !MASTER_PAGE_FLAGS_MASK) | HANCOM_MULTI_MASTER_PAGE_FLAGS;
    if section_def.flags != expected {
        section_def.flags = expected;
        report.section_def_multi_master_page_flags_materialized += 1;
    }
}

fn materialize_section_def_master_page_tail(
    section_def: &mut SectionDef,
    report: &mut AdapterReport,
) {
    if section_def.master_pages.is_empty() {
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
    if section_def.raw_ctrl_extra != extra {
        section_def.raw_ctrl_extra = extra;
        report.section_def_master_page_tail_materialized += 1;
    }
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
        // [#2736] 도형 캡션 문단도 보강한다. `DrawingObjAttr` 를 공유하는 사각형·타원·선·호·
        // 다각형·곡선·글상자·묶음·차트·OLE 이 한 번에 덮인다. 형제 워크
        // (collect_bin_order_from_shape / remap_bin_refs_in_shape /
        // collect_object_border_fill_refs_from_shape)는 셋 다 이미 drawing.caption 을
        // 순회하는데 adapt 워크만 빠져 있었다.
        if let Some(caption) = &mut drawing.caption {
            adapt_paragraphs_with_context(&mut caption.paragraphs, report, context);
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

// #1064/#1099에서 같은 104바이트 payload가 관찰됐다. #4438: 이 11쌍은 하나의
// opaque table-layout payload에서 관찰된 정확한 계약이지, 개별 item의 의미 표가 아니다.
// 외부 소비자가 각 item을 해석하는지는 확인되지 않았으므로 호환 의미를 추정하지 않는다.
// 0x4000부터의 연속값을 일반 item-id 할당 범위로 해석하거나 다음 ID를 발명하지 않는다.
// 새 의미를 추가하려면 이 배열을 연장하지 말고 독립된 바이너리 근거와 소비자를 먼저 확정한다.
const TABLE_LAYOUT_CTRL_DATA_I4_ITEMS: [(u16, u32); 11] = [
    (0x4000, 3826),
    (0x4001, 1048),
    (0x4002, 28346),
    (0x4003, 8475),
    (0x4004, 708),
    (0x4005, 0),
    (0x4006, 2),
    (0x4007, 9),
    (0x4008, 0),
    (0x4009, 59528),
    (0x400a, 84188),
];

fn build_table_layout_ctrl_data() -> Vec<u8> {
    // HWPX→HWP adapter가 특정 3x2 선택지 표 뒤에 materialize하는 104바이트 raw 계약.
    // serializer는 이 payload를 해석하거나 재번호화하지 않고 ctrl_data_records에서 복사한다.
    let mut data = Vec::with_capacity(104);
    data.extend_from_slice(&0x021b_u16.to_le_bytes());
    data.extend_from_slice(&1_u16.to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());
    data.extend_from_slice(&0x0242_u16.to_le_bytes());
    data.extend_from_slice(&0x8000_u16.to_le_bytes());
    data.extend_from_slice(&0x0242_u16.to_le_bytes());
    data.extend_from_slice(&(TABLE_LAYOUT_CTRL_DATA_I4_ITEMS.len() as u16).to_le_bytes());
    data.extend_from_slice(&0_u16.to_le_bytes());
    for &(item_id, value) in &TABLE_LAYOUT_CTRL_DATA_I4_ITEMS {
        data.extend_from_slice(&item_id.to_le_bytes());
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

    // 표 캡션 문단도 보강한다(#2443 에서 캡션이 bin 리맵 대상임이 확인됨).
    // 누락 시 캡션 안의 그림 href·중첩 표 ctrl_data 등이 물질화되지 않는다.
    if let Some(caption) = &mut table.caption {
        adapt_paragraphs_with_context(&mut caption.paragraphs, report, context);
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

fn materialize_table_ctrl_header_attr(table: &mut Table, report: &mut AdapterReport) {
    const HWPX_TABLE_NUMBERING_BIT: u32 = 0x0800_0000;
    const HWP5_TABLE_CAPTION_COMMON_ATTR_BIT: u32 = 0x2000_0000;

    let before = table.common.attr;
    // [#3834] 자리차지 비트(bit 13)는 `pack_common_attr_bits` 가 IR 에서 만든다. 종전
    // 무조건 OR 은 원본 `flowWithText="0"` 를 파괴했다 — HWPX 는 이 속성을 항상 명시
    // 하므로(코퍼스 119문서 표 560개, 누락 0) 파서 기본값을 메울 필요가 없다. 한글
    // 2022 도 같은 변환에서 `0` 을 그대로 둔다 — HWPX 를 열어 HWP 로 저장한 실측에서
    // 13문서 표 84개 전부 보존, 정규화 0건(`tools/hangul_flowwithtext_oracle.py`).
    // #2697 의 표 번호 비트 무조건 OR 과 같은 계열이다.
    let mut attr = pack_common_attr_bits(&table.common);
    // [#2697] 파서(materialize_hwpx_table_attrs)와 동일 게이트: "표 번호" 비트는
    // numberingType 이 실제로 TABLE 일 때만 세운다. 종전 무조건 OR 은 numberingType=
    // "PICTURE"/"NONE" 표를 HWP5 저장 시 표 번호 범주로 되돌려, 파서가 #2697 에서
    // 제거한 IR 모순(numbering_type=Picture ↔ attr=TABLE)을 변환 계층이 재도입했다.
    if table.common.numbering_type == crate::model::shape::ObjectNumberingType::Table {
        attr |= HWPX_TABLE_NUMBERING_BIT;
    }
    if table.caption.is_some() {
        attr |= HWP5_TABLE_CAPTION_COMMON_ATTR_BIT;
    }
    table.common.attr = attr;

    if table.common.attr != before {
        report.table_ctrl_header_attr_materialized += 1;
    }
}

/// [#2767] 캡션이 있는 그림(gso `$pic`) CTRL_HEADER 의 한컴 캡션 비트(bit 29) 보강.
///
/// 전 코퍼스 실측(`samples/**/*.hwp`, 한컴 저작 원본): 캡션 동반 gso 80개(그림 42,
/// 사각형 33, 연결선 3, OLE 2) 전부 bit 29=1, 캡션 없는 gso 2,674개 전부 bit 29=0.
/// 개체 종류와 무관하게 캡션 유무만으로 결정된다. HWPX 출처 그림의
/// `common.attr` 는 파서(`pack_hwpx_common_obj_attr`)가 이미 non-zero로 채우고
/// 직렬화기가 verbatim 기록하므로, 표처럼 `pack_common_attr_bits(...)`로
/// **recompute** 하지 않고 비트만 **OR** 한다 — recompute 하면 표 전용 비트를
/// 그림에 강제로 얹는 회귀가 된다. 멱등: 비트가 이미 켜져 있으면 카운트하지 않음.
///
/// 도형(`$rec`/`$con`)·OLE 캡션은 범위 밖이다 — 직렬화기(`serializer/control.rs`)가
/// 도형 캡션 레코드 자체를 아직 출력하지 않아, 레코드 없이 비트만 켜면 자기모순
/// 레코드가 된다(별도 후속 과제).
fn materialize_picture_caption_common_attr(pic: &mut Picture, report: &mut AdapterReport) {
    const HWP5_GSO_CAPTION_COMMON_ATTR_BIT: u32 = 0x2000_0000;
    if pic.caption.is_some() && pic.common.attr & HWP5_GSO_CAPTION_COMMON_ATTR_BIT == 0 {
        pic.common.attr |= HWP5_GSO_CAPTION_COMMON_ATTR_BIT;
        report.picture_caption_common_attr_materialized += 1;
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
    // [Issue #1770] HWPX 출처만 마커 부여 (HWP3 은 자체 variant 시멘틱 유지).
    // idempotent — 이미 있으면 추가하지 않는다.
    let master_page_apply_slots_materialized = if matches!(source_format, FileFormat::Hwpx) {
        materialize_hwp5_master_page_slots(doc)
    } else {
        0
    };
    if matches!(source_format, FileFormat::Hwpx)
        && !doc
            .extra_streams
            .iter()
            .any(|(p, _)| p == HWPX_ORIGIN_STREAM_PATH)
    {
        doc.extra_streams
            .push((HWPX_ORIGIN_STREAM_PATH.to_string(), b"1".to_vec()));
    }
    // [#3707] HWP3 출처 마커 — 재파싱이 쪽나눔 허용치를 되돌릴 수 있게 한다.
    // HWP3 파서가 세우는 `pagination_bottom_tolerance`(1600 HU)는 렌더러 내부 값이라
    // 저장 파일에 남지 않는다. 마커로 출처만 기록해 재파싱이 결정론적으로 복원한다.
    if matches!(source_format, FileFormat::Hwp3)
        && !doc
            .extra_streams
            .iter()
            .any(|(p, _)| p == HWP3_ORIGIN_STREAM_PATH)
    {
        doc.extra_streams
            .push((HWP3_ORIGIN_STREAM_PATH.to_string(), b"1".to_vec()));
    }
    let mut report = convert_to_hwp_ir(doc, matches!(source_format, FileFormat::Hwpx));
    report.master_page_apply_slots_materialized = master_page_apply_slots_materialized;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::CharShapeRef;

    /// [#3676] HWP3 parser가 실제로 만드는 그림/도형/표 캡션과 HiddenComment, 그리고
    /// 공통 adapter가 다루는 바탕쪽 글상자를 모두 건너 문단 직속 그림만 보정하면,
    /// 한컴은 그 하나의 0 geometry를 이유로 문서 전체를 거부할 수 있다. 중첩 그림도
    /// geometry·crop·local file version 계약을 정확히 받아야 한다.
    #[test]
    fn hwp3_nested_caption_hidden_comment_and_master_page_pictures_are_normalized() {
        use crate::model::control::HiddenComment;
        use crate::model::header_footer::MasterPage;
        use crate::model::shape::{
            Caption, ChartShape, GroupShape, OleShape, RectangleShape, TextBox,
        };

        fn hwp3_picture(
            current_width: u32,
            current_height: u32,
            original_width: u32,
            original_height: u32,
        ) -> Picture {
            Picture {
                common: crate::model::shape::CommonObjAttr {
                    width: current_width,
                    height: current_height,
                    ..Default::default()
                },
                shape_attr: crate::model::shape::ShapeComponentAttr {
                    current_width,
                    current_height,
                    original_width,
                    original_height,
                    // HWP3 파서는 0으로 남긴다. adapter가 한컴 HWP5 contract인
                    // 정확한 값 1로 보정해야 한다.
                    local_file_version: 0,
                    ..Default::default()
                },
                ..Default::default()
            }
        }

        fn assert_hancom_picture_contract(
            picture: &Picture,
            width: i32,
            height: i32,
            original_width: i32,
            original_height: i32,
        ) {
            assert_eq!(picture.border_x, [0, 0, width, 0]);
            assert_eq!(picture.border_y, [width, height, 0, height]);
            assert_eq!(picture.crop.left, 0);
            assert_eq!(picture.crop.top, 0);
            assert_eq!(picture.crop.right, original_width);
            assert_eq!(picture.crop.bottom, original_height);
            assert_eq!(picture.shape_attr.local_file_version, 1);
        }

        // HWP3 picture caption 안의 또 다른 picture. 기존 walker는 이 문단을
        // 방문하지 않아 inner picture가 모두 0으로 남았다.
        let mut picture_caption_para = Paragraph::default();
        picture_caption_para
            .controls
            .push(Control::Picture(Box::new(hwp3_picture(120, 80, 240, 160))));
        let outer_picture = Picture {
            caption: Some(Caption {
                paragraphs: vec![picture_caption_para],
                ..Default::default()
            }),
            ..Default::default()
        };

        // HWP3 drawing caption → group caption → table caption → HiddenComment
        // 경로. 이들 모두 parse_paragraph_list가 반환하는 실제 paragraph container다.
        let mut hidden_comment_para = Paragraph::default();
        hidden_comment_para
            .controls
            .push(Control::Picture(Box::new(hwp3_picture(101, 202, 303, 404))));
        let mut table_caption_para = Paragraph::default();
        table_caption_para
            .controls
            .push(Control::HiddenComment(Box::new(HiddenComment {
                paragraphs: vec![hidden_comment_para],
            })));
        let mut group_caption_para = Paragraph::default();
        group_caption_para
            .controls
            .push(Control::Table(Box::new(Table {
                caption: Some(Caption {
                    paragraphs: vec![table_caption_para],
                    ..Default::default()
                }),
                ..Default::default()
            })));
        let mut drawing_caption_para = Paragraph::default();
        drawing_caption_para
            .controls
            .push(Control::Shape(Box::new(ShapeObject::Group(GroupShape {
                caption: Some(Caption {
                    paragraphs: vec![group_caption_para],
                    ..Default::default()
                }),
                ..Default::default()
            }))));
        let mut rectangle = RectangleShape::default();
        rectangle.drawing.caption = Some(Caption {
            paragraphs: vec![drawing_caption_para],
            ..Default::default()
        });

        // Chart는 DrawingObjAttr과 별도로 own caption을 보존한다.
        let mut chart_caption_para = Paragraph::default();
        chart_caption_para
            .controls
            .push(Control::Picture(Box::new(hwp3_picture(55, 44, 110, 88))));
        let chart = ChartShape {
            caption: Some(Caption {
                paragraphs: vec![chart_caption_para],
                ..Default::default()
            }),
            ..Default::default()
        };

        // 공통 adapter의 바탕쪽 OLE 글상자는 DrawingObjAttr 공통 text_box 경로를 쓴다.
        let mut master_page_text_box_para = Paragraph::default();
        master_page_text_box_para
            .controls
            .push(Control::Picture(Box::new(hwp3_picture(70, 60, 140, 120))));
        let mut ole = OleShape::default();
        ole.drawing.text_box = Some(TextBox {
            paragraphs: vec![master_page_text_box_para],
            ..Default::default()
        });
        // HWP3 OLE fixup은 picture caption을 이 own caption 필드에 옮긴다.
        let mut ole_caption_para = Paragraph::default();
        ole_caption_para
            .controls
            .push(Control::Picture(Box::new(hwp3_picture(45, 35, 90, 70))));
        ole.caption = Some(Caption {
            paragraphs: vec![ole_caption_para],
            ..Default::default()
        });
        let master_page = MasterPage {
            paragraphs: vec![Paragraph {
                controls: vec![Control::Shape(Box::new(ShapeObject::Ole(Box::new(ole))))],
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut doc = Document {
            sections: vec![Section {
                paragraphs: vec![Paragraph {
                    controls: vec![
                        Control::Picture(Box::new(outer_picture)),
                        Control::Shape(Box::new(ShapeObject::Rectangle(rectangle))),
                        Control::Shape(Box::new(ShapeObject::Chart(Box::new(chart)))),
                    ],
                    ..Default::default()
                }],
                section_def: SectionDef {
                    master_pages: vec![master_page],
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        };

        convert_if_hwpx_source(&mut doc, FileFormat::Hwp3);

        // adapter는 본문 첫 문단에 SectionDef control을 앞에 삽입하므로, 고정
        // control index 대신 대상 타입을 찾는다.
        let outer_picture = doc.sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Picture(picture) => Some(picture.as_ref()),
                _ => None,
            })
            .expect("outer picture expected");
        let Control::Picture(caption_picture) =
            &outer_picture.caption.as_ref().unwrap().paragraphs[0].controls[0]
        else {
            panic!("picture expected inside HWP3 picture caption");
        };
        assert_hancom_picture_contract(caption_picture, 120, 80, 240, 160);

        let rectangle = doc.sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Shape(shape) => match shape.as_ref() {
                    ShapeObject::Rectangle(rectangle) => Some(rectangle),
                    _ => None,
                },
                _ => None,
            })
            .expect("rectangle expected");
        let Control::Shape(group) =
            &rectangle.drawing.caption.as_ref().unwrap().paragraphs[0].controls[0]
        else {
            panic!("group expected in drawing caption");
        };
        let ShapeObject::Group(group) = group.as_ref() else {
            panic!("group shape expected");
        };
        assert_eq!(group.shape_attr.local_file_version, 1);
        let Control::Table(table) = &group.caption.as_ref().unwrap().paragraphs[0].controls[0]
        else {
            panic!("table expected in group caption");
        };
        let Control::HiddenComment(comment) =
            &table.caption.as_ref().unwrap().paragraphs[0].controls[0]
        else {
            panic!("HiddenComment expected in table caption");
        };
        let Control::Picture(hidden_comment_picture) = &comment.paragraphs[0].controls[0] else {
            panic!("picture expected inside HiddenComment");
        };
        assert_hancom_picture_contract(hidden_comment_picture, 101, 202, 303, 404);

        let chart = doc.sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Shape(shape) => match shape.as_ref() {
                    ShapeObject::Chart(chart) => Some(chart.as_ref()),
                    _ => None,
                },
                _ => None,
            })
            .expect("chart expected");
        let Control::Picture(chart_caption_picture) =
            &chart.caption.as_ref().unwrap().paragraphs[0].controls[0]
        else {
            panic!("picture expected inside chart caption");
        };
        assert_hancom_picture_contract(chart_caption_picture, 55, 44, 110, 88);

        let Control::Shape(ole) =
            &doc.sections[0].section_def.master_pages[0].paragraphs[0].controls[0]
        else {
            panic!("OLE expected in master page");
        };
        let ShapeObject::Ole(ole) = ole.as_ref() else {
            panic!("OLE shape expected");
        };
        let Control::Picture(master_page_picture) =
            &ole.drawing.text_box.as_ref().unwrap().paragraphs[0].controls[0]
        else {
            panic!("picture expected inside master-page OLE text box");
        };
        assert_hancom_picture_contract(master_page_picture, 70, 60, 140, 120);
        let Control::Picture(ole_caption_picture) =
            &ole.caption.as_ref().unwrap().paragraphs[0].controls[0]
        else {
            panic!("picture expected inside HWP3 OLE caption");
        };
        assert_hancom_picture_contract(ole_caption_picture, 45, 35, 90, 70);
    }

    #[test]
    fn border_fill_refs_collected_inside_footnote_and_caption() {
        use crate::model::footnote::Footnote;
        use crate::model::shape::Caption;

        // 각주 문단 안 표의 border_fill 이 수집돼야 한다(종전엔 각주 미재귀로 누락).
        let mut fn_para = Paragraph::default();
        fn_para.controls.push(Control::Table(Box::new(Table {
            border_fill_id: 7,
            ..Default::default()
        })));
        let mut footnote = Footnote::default();
        footnote.paragraphs.push(fn_para);
        let mut para = Paragraph::default();
        para.controls.push(Control::Footnote(Box::new(footnote)));
        let mut doc = Document::default();
        doc.sections.push(Section {
            paragraphs: vec![para],
            ..Default::default()
        });
        let refs = collect_object_border_fill_refs(&doc);
        assert!(refs.contains(&7), "각주 안 표의 border_fill 이 수집돼야 함");

        // 표 캡션 문단 안 표의 border_fill 도 수집돼야 한다.
        let mut cap_para = Paragraph::default();
        cap_para.controls.push(Control::Table(Box::new(Table {
            border_fill_id: 9,
            ..Default::default()
        })));
        let mut caption = Caption::default();
        caption.paragraphs.push(cap_para);
        let mut tpara = Paragraph::default();
        tpara.controls.push(Control::Table(Box::new(Table {
            border_fill_id: 1,
            caption: Some(caption),
            ..Default::default()
        })));
        let mut doc2 = Document::default();
        doc2.sections.push(Section {
            paragraphs: vec![tpara],
            ..Default::default()
        });
        let refs2 = collect_object_border_fill_refs(&doc2);
        assert!(
            refs2.contains(&9),
            "표 캡션 안 표의 border_fill 이 수집돼야 함"
        );
    }

    #[test]
    fn border_fill_refs_collected_inside_header_masterpage_and_shape_caption() {
        use crate::model::header_footer::{Header, MasterPage};
        use crate::model::shape::{Caption, RectangleShape, ShapeObject};

        let table_para = |bf: u16| {
            let mut p = Paragraph::default();
            p.controls.push(Control::Table(Box::new(Table {
                border_fill_id: bf,
                ..Default::default()
            })));
            p
        };

        // 머리말 안 표
        let mut header = Header::default();
        header.paragraphs.push(table_para(11));
        let mut hpara = Paragraph::default();
        hpara.controls.push(Control::Header(Box::new(header)));
        let mut doc = Document::default();
        doc.sections.push(Section {
            paragraphs: vec![hpara],
            ..Default::default()
        });
        assert!(
            collect_object_border_fill_refs(&doc).contains(&11),
            "머리말 안 표의 border_fill 이 수집돼야 함"
        );

        // 바탕쪽(master page) 안 표
        let mut mp = MasterPage::default();
        mp.paragraphs.push(table_para(13));
        let mut sec = Section::default();
        sec.section_def.master_pages.push(mp);
        let mut doc2 = Document::default();
        doc2.sections.push(sec);
        assert!(
            collect_object_border_fill_refs(&doc2).contains(&13),
            "바탕쪽 안 표의 border_fill 이 수집돼야 함"
        );

        // 도형 캡션 안 표
        let mut caption = Caption::default();
        caption.paragraphs.push(table_para(15));
        let mut rect = RectangleShape::default();
        rect.drawing.caption = Some(caption);
        let mut spara = Paragraph::default();
        spara
            .controls
            .push(Control::Shape(Box::new(ShapeObject::Rectangle(rect))));
        let mut doc3 = Document::default();
        doc3.sections.push(Section {
            paragraphs: vec![spara],
            ..Default::default()
        });
        assert!(
            collect_object_border_fill_refs(&doc3).contains(&15),
            "도형 캡션 안 표의 border_fill 이 수집돼야 함"
        );
    }

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
                // [#3834] 계약값 bit 13 은 IR 의 이 필드에서 나온다. 원본 한컴 표가 자리차지
                // 이므로 픽스처도 켠다 — 종전에는 어댑터가 무조건 OR 해 IR 과 무관했다.
                flow_with_text: true,
                // HWPX 파서는 표 기본 numberingType 을 TABLE 로 채운다 (section.rs).
                numbering_type: crate::model::shape::ObjectNumberingType::Table,
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
        // [#3570] 한컴은 zone 개수까지만 쓰고 끝낸다 — 여분 2바이트를 넣지 않는다.
        assert!(table.raw_table_record_extra.is_empty());
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
                // [#3834] 계약값 bit 13 의 출처 — 위 표와 같다.
                flow_with_text: true,
                // HWPX 파서는 표 기본 numberingType 을 TABLE 로 채운다 (section.rs).
                numbering_type: crate::model::shape::ObjectNumberingType::Table,
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
    fn picture_numbering_table_keeps_category_on_hwp_save() {
        // [#2697] HWPX 파서는 numberingType="PICTURE" 표에 TABLE 번호 비트(0x0800_0000)를
        // 세우지 않는다. HWP5 저장 어댑터도 같은 게이트를 따라야 한다 — 종전 무조건 OR 은
        // 그림 번호 캡션 표를 표 번호 범주로 되돌려 파서가 만든 IR 계약과 모순됐다.
        use crate::model::shape::ObjectNumberingType;

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
            ..Default::default()
        };
        table.common.numbering_type = ObjectNumberingType::Picture;

        let mut report = AdapterReport::new();
        adapt_table(&mut table, &mut report);

        assert_eq!(
            table.common.attr & 0x0800_0000,
            0,
            "PICTURE 번호 표에 TABLE 번호 비트가 서면 IR 모순: {:#010x}",
            table.common.attr
        );
        let flags = u32::from_le_bytes(
            table.raw_ctrl_data[common_obj_offsets::FLAGS]
                .try_into()
                .unwrap(),
        );
        assert_eq!(
            flags & 0x0800_0000,
            0,
            "raw_ctrl_data 에도 TABLE 번호 비트가 없어야 함: {flags:#010x}"
        );
    }

    #[test]
    fn none_numbering_table_keeps_category_on_hwp_save() {
        // numberingType="NONE"(번호 매김 제외) 표도 저장 시 표 번호 범주로 승격되면 안 된다.
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
            ..Default::default()
        };
        // Table::default() 의 common.numbering_type 은 None.

        let mut report = AdapterReport::new();
        adapt_table(&mut table, &mut report);

        assert_eq!(
            table.common.attr & 0x0800_0000,
            0,
            "NONE 번호 표에 TABLE 번호 비트가 서면 안 됨: {:#010x}",
            table.common.attr
        );
    }

    #[test]
    fn table_numbering_table_still_sets_numbering_bit() {
        // 회귀 가드: 기본 표(numberingType="TABLE")는 종전대로 표 번호 비트를 유지한다.
        use crate::model::shape::ObjectNumberingType;

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
            ..Default::default()
        };
        table.common.numbering_type = ObjectNumberingType::Table;

        let mut report = AdapterReport::new();
        adapt_table(&mut table, &mut report);

        assert_eq!(
            table.common.attr & 0x0800_0000,
            0x0800_0000,
            "TABLE 번호 표는 표 번호 비트를 유지해야 함: {:#010x}",
            table.common.attr
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
    fn picture_href_materializes_inside_footnote_and_caption() {
        use crate::model::footnote::Footnote;
        use crate::model::shape::Caption;

        // 각주 문단 안의 그림 href 가 물질화되어야 한다(종전엔 adapt 워크가
        // 각주를 재귀하지 않아 유실).
        let mut fn_inner = Paragraph::default();
        fn_inner.controls.push(Control::Picture(Box::new(Picture {
            href: Some("http://www.korea.kr;1;0;0;".to_string()),
            ..Default::default()
        })));
        let mut footnote = Footnote::default();
        footnote.paragraphs.push(fn_inner);
        let mut para = Paragraph::default();
        para.controls.push(Control::Footnote(Box::new(footnote)));

        let mut report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut report);
        assert_eq!(report.picture_href_ctrl_data_materialized, 1);
        let Control::Footnote(fnote) = &para.controls[0] else {
            panic!("expected footnote control");
        };
        assert_eq!(
            fnote.paragraphs[0].ctrl_data_records[0]
                .as_ref()
                .unwrap()
                .len(),
            76
        );

        // 표 캡션 문단 안의 그림 href 도 물질화되어야 한다.
        let mut cap_inner = Paragraph::default();
        cap_inner.controls.push(Control::Picture(Box::new(Picture {
            href: Some("http://www.korea.kr;1;0;0;".to_string()),
            ..Default::default()
        })));
        let mut caption = Caption::default();
        caption.paragraphs.push(cap_inner);
        let mut tpara = Paragraph::default();
        tpara.controls.push(Control::Table(Box::new(Table {
            caption: Some(caption),
            ..Default::default()
        })));

        let mut treport = AdapterReport::new();
        adapt_paragraph(&mut tpara, &mut treport);
        assert_eq!(treport.picture_href_ctrl_data_materialized, 1);
        let Control::Table(tbl) = &tpara.controls[0] else {
            panic!("expected table control");
        };
        let cap = tbl.caption.as_ref().unwrap();
        assert_eq!(
            cap.paragraphs[0].ctrl_data_records[0]
                .as_ref()
                .unwrap()
                .len(),
            76
        );
    }

    /// [#2736] adapt 워크가 그림 캡션·도형 캡션 문단을 방문하는지.
    ///
    /// 표 캡션은 `adapt_table_with_context` 가 이미 보강하고, bin order/remap·
    /// border_fill 수집 워크도 캡션을 순회하는데 adapt 워크만 그림/도형 캡션을
    /// 건너뛰어 캡션 안 그림의 href 가 HWP 저장 시 유실됐다.
    #[test]
    fn picture_href_materializes_inside_picture_and_shape_caption() {
        use crate::model::shape::{Caption, RectangleShape, ShapeObject};

        let href_caption = || {
            let mut inner = Paragraph::default();
            inner.controls.push(Control::Picture(Box::new(Picture {
                href: Some("http://www.korea.kr;1;0;0;".to_string()),
                ..Default::default()
            })));
            Caption {
                paragraphs: vec![inner],
                ..Default::default()
            }
        };

        // 그림 캡션 문단 안의 그림 href
        let mut ppara = Paragraph::default();
        ppara.controls.push(Control::Picture(Box::new(Picture {
            caption: Some(href_caption()),
            ..Default::default()
        })));

        let mut preport = AdapterReport::new();
        adapt_paragraph(&mut ppara, &mut preport);
        assert_eq!(
            preport.picture_href_ctrl_data_materialized, 1,
            "그림 캡션 문단 안 그림의 href 가 물질화돼야 함(캡션 문단 미방문)"
        );
        let Control::Picture(pic) = &ppara.controls[0] else {
            panic!("expected picture control");
        };
        let pcap = pic.caption.as_ref().unwrap();
        assert_eq!(
            pcap.paragraphs[0].ctrl_data_records[0]
                .as_ref()
                .unwrap()
                .len(),
            76
        );

        // 도형 캡션 문단 안의 그림 href (DrawingObjAttr 공유 → 묶음·차트·OLE 동시 적용)
        let mut rect = RectangleShape::default();
        rect.drawing.caption = Some(href_caption());
        let mut spara = Paragraph::default();
        spara
            .controls
            .push(Control::Shape(Box::new(ShapeObject::Rectangle(rect))));

        let mut sreport = AdapterReport::new();
        adapt_paragraph(&mut spara, &mut sreport);
        assert_eq!(
            sreport.picture_href_ctrl_data_materialized, 1,
            "도형 캡션 문단 안 그림의 href 가 물질화돼야 함(캡션 문단 미방문)"
        );
        let Control::Shape(shape) = &spara.controls[0] else {
            panic!("expected shape control");
        };
        let scap = shape.drawing().unwrap().caption.as_ref().unwrap();
        assert_eq!(
            scap.paragraphs[0].ctrl_data_records[0]
                .as_ref()
                .unwrap()
                .len(),
            76
        );
    }

    /// [#2736] 그림 캡션 문단 안 그림의 `bin_data_id` 리맵 — 표 캡션에 대한 동형
    /// 회귀(`table_caption_picture_bin_ref_is_remapped`)의 미수정 형제였다.
    #[test]
    fn picture_caption_picture_bin_ref_is_remapped() {
        use crate::model::image::Picture;
        use crate::model::shape::Caption;

        let mut inner_pic = Picture::default();
        inner_pic.image_attr.bin_data_id = 1;
        let mut cap_para = Paragraph::default();
        cap_para
            .controls
            .push(Control::Picture(Box::new(inner_pic)));
        let mut outer = Picture::default();
        outer.image_attr.bin_data_id = 2;
        outer.caption = Some(Caption {
            paragraphs: vec![cap_para],
            ..Default::default()
        });
        let mut ctrl = Control::Picture(Box::new(outer));

        // remap: bin id 1 → 2, 2 → 1
        let remap = vec![0u16, 2, 1];
        remap_bin_refs_in_control(&mut ctrl, &remap);

        let Control::Picture(outer) = &ctrl else {
            panic!("expected picture");
        };
        assert_eq!(outer.image_attr.bin_data_id, 1);
        let caption = outer.caption.as_ref().unwrap();
        let Control::Picture(inner) = &caption.paragraphs[0].controls[0] else {
            panic!("expected caption picture");
        };
        assert_eq!(
            inner.image_attr.bin_data_id, 2,
            "그림 캡션 안 그림의 bin_data_id 가 remap 되지 않음(캡션 문단 미방문)"
        );
    }

    /// [#2736] 그림 캡션 문단 안 개체의 `border_fill` 참조 수집 —
    /// `collect_object_border_fill_refs_from_paragraph` 에 `Control::Picture` arm 이
    /// 아예 없어 표 캡션·도형 캡션과 달리 그림 캡션만 빠져 있었다.
    #[test]
    fn border_fill_refs_collected_inside_picture_caption() {
        use crate::model::image::Picture;
        use crate::model::shape::Caption;

        let mut cap_para = Paragraph::default();
        cap_para.controls.push(Control::Table(Box::new(Table {
            border_fill_id: 17,
            ..Default::default()
        })));
        let mut pic = Picture::default();
        pic.caption = Some(Caption {
            paragraphs: vec![cap_para],
            ..Default::default()
        });
        let mut para = Paragraph::default();
        para.controls.push(Control::Picture(Box::new(pic)));

        let mut doc = Document::default();
        doc.sections.push(Section {
            paragraphs: vec![para],
            ..Default::default()
        });

        assert!(
            collect_object_border_fill_refs(&doc).contains(&17),
            "그림 캡션 안 표의 border_fill 이 수집돼야 함"
        );
    }

    /// [#2736] 실파일 회귀 — 한컴 산출 `aift.hwpx` 의 그림 캡션 문단 9개가 adapt 워크의
    /// 방문 표식(`materialize_para_header_tail` 의 12바이트 header tail)을 받는지.
    ///
    /// 수정 전 실측: 본문 921/921·표 캡션 2/2 는 12바이트, 그림 캡션 9/9 는 파서가 넣은
    /// 10바이트 그대로 — 직렬화 시 캡션 문단만 PARA_HEADER 22바이트로 갈렸다.
    #[test]
    fn aift_picture_caption_paragraphs_are_adapted() {
        fn count_picture_caption_paragraphs(
            para: &Paragraph,
            total: &mut usize,
            tail12: &mut usize,
        ) {
            for ctrl in &para.controls {
                match ctrl {
                    Control::Picture(pic) => {
                        if let Some(caption) = &pic.caption {
                            for p in &caption.paragraphs {
                                *total += 1;
                                if p.raw_header_extra.len() >= 12 {
                                    *tail12 += 1;
                                }
                            }
                        }
                    }
                    Control::Table(table) => {
                        for cell in &table.cells {
                            for p in &cell.paragraphs {
                                count_picture_caption_paragraphs(p, total, tail12);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let data = std::fs::read("samples/hwpx/aift.hwpx").expect("sample exists");
        let mut core = crate::document_core::DocumentCore::from_bytes(&data).expect("parse hwpx");
        convert_hwpx_to_hwp_ir(core.document_mut());

        let mut total = 0usize;
        let mut tail12 = 0usize;
        for section in &core.document().sections {
            for para in &section.paragraphs {
                count_picture_caption_paragraphs(para, &mut total, &mut tail12);
            }
        }

        assert_eq!(total, 9, "aift.hwpx 의 그림 캡션 문단 수");
        assert_eq!(
            tail12, total,
            "그림 캡션 문단이 adapt 워크의 방문 표식(header tail 12바이트)을 받아야 함"
        );
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
    fn table_layout_ctrl_data_item_ids_are_an_explicit_observed_contract() {
        assert_eq!(
            TABLE_LAYOUT_CTRL_DATA_I4_ITEMS,
            [
                (0x4000, 3826),
                (0x4001, 1048),
                (0x4002, 28346),
                (0x4003, 8475),
                (0x4004, 708),
                (0x4005, 0),
                (0x4006, 2),
                (0x4007, 9),
                (0x4008, 0),
                (0x4009, 59528),
                (0x400a, 84188),
            ]
        );

        let payload = build_table_layout_ctrl_data();
        assert_eq!(payload.len(), 104);
        for (index, &(item_id, value)) in TABLE_LAYOUT_CTRL_DATA_I4_ITEMS.iter().enumerate() {
            let offset = 16 + index * 8;
            assert_eq!(
                &payload[offset..offset + 2],
                &item_id.to_le_bytes(),
                "item[{index}] id"
            );
            assert_eq!(
                &payload[offset + 2..offset + 4],
                &0x0004_u16.to_le_bytes(),
                "item[{index}] type"
            );
            assert_eq!(
                &payload[offset + 4..offset + 8],
                &value.to_le_bytes(),
                "item[{index}] value"
            );
        }
    }

    #[test]
    fn table_layout_ctrl_data_materializes_for_nested_cell_owner() {
        let mut nested_paragraph = Paragraph::default();
        nested_paragraph
            .controls
            .push(Control::Table(Box::new(Table {
                row_count: 3,
                col_count: 2,
                page_break: TablePageBreak::RowBreak,
                repeat_header: true,
                ..Default::default()
            })));

        let mut paragraph = Paragraph::default();
        paragraph.controls.push(Control::Table(Box::new(Table {
            row_count: 1,
            col_count: 1,
            cells: vec![Cell {
                col_span: 1,
                row_span: 1,
                paragraphs: vec![nested_paragraph],
                ..Default::default()
            }],
            ..Default::default()
        })));

        let mut report = AdapterReport::new();
        adapt_paragraph(&mut paragraph, &mut report);

        assert_eq!(report.table_layout_ctrl_data_materialized, 1);
        assert!(paragraph.ctrl_data_records[0].is_none());
        let Control::Table(outer) = &paragraph.controls[0] else {
            panic!("expected outer table");
        };
        let nested_owner = &outer.cells[0].paragraphs[0];
        assert_eq!(nested_owner.ctrl_data_records.len(), 1);
        assert_eq!(
            nested_owner.ctrl_data_records[0].as_deref(),
            Some(build_table_layout_ctrl_data().as_slice())
        );
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
    fn single_odd_master_page_flags_preserve_hancom_inherited_even_contract() {
        let mut section_def = SectionDef {
            flags: 0x2000_0000,
            master_pages: vec![crate::model::header_footer::MasterPage {
                apply_to: crate::model::header_footer::HeaderFooterApply::Odd,
                ..Default::default()
            }],
            ..Default::default()
        };

        let mut report = AdapterReport::new();
        adapt_section_def(&mut section_def, &mut report);

        assert_eq!(section_def.flags & 0xe000_0000, 0x8000_0000);
        assert_eq!(report.section_def_single_master_page_flags_materialized, 1);
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
    fn materialized_second_master_page_updates_stale_single_master_flag() {
        let mut section_def = SectionDef {
            flags: 0x4000_0000,
            master_pages: vec![Default::default(), Default::default()],
            ..Default::default()
        };

        let mut report = AdapterReport::new();
        adapt_section_def(&mut section_def, &mut report);

        assert_eq!(section_def.flags & 0xe000_0000, 0xc000_0000);
        assert_eq!(report.section_def_multi_master_page_flags_materialized, 1);
        assert_eq!(report.section_def_single_master_page_flags_materialized, 0);
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
            // [#3570] 한컴은 zone 개수까지만 쓰고 끝낸다 — 여분 2바이트 없음.
            assert!(table.raw_table_record_extra.is_empty());
            assert_eq!(table.cells[0].raw_list_extra.len(), 13);
        }

        assert_eq!(report.tables_ctrl_data_synthesized, 2);
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

    /// [bin remap 회귀] HWPX→HWP 변환의 BinData 재정렬 시 표 캡션 문단 안의 그림
    /// bin_data_id 가 remap 되지 않아 캡션 그림이 엉뚱한 이미지로 해석되던 결함.
    #[test]
    fn table_caption_picture_bin_ref_is_remapped() {
        use crate::model::image::Picture;
        use crate::model::shape::Caption;
        use crate::model::table::Table;

        let mut pic = Picture::default();
        pic.image_attr.bin_data_id = 1;
        let mut cap_para = Paragraph::default();
        cap_para.controls.push(Control::Picture(Box::new(pic)));
        let mut table = Table::default();
        table.caption = Some(Caption {
            paragraphs: vec![cap_para],
            ..Default::default()
        });
        let mut ctrl = Control::Table(Box::new(table));

        // remap: bin id 1 → 2
        let remap = vec![0u16, 2, 1];
        remap_bin_refs_in_control(&mut ctrl, &remap);

        let Control::Table(table) = &ctrl else {
            panic!("expected table");
        };
        let caption = table.caption.as_ref().unwrap();
        let Control::Picture(pic) = &caption.paragraphs[0].controls[0] else {
            panic!("expected caption picture");
        };
        assert_eq!(
            pic.image_attr.bin_data_id, 2,
            "표 캡션 그림의 bin_data_id 가 remap 되지 않음(캡션 문단 미방문)"
        );
    }

    // ---------- #2767 결함 A — 그림 캡션 gso CTRL_HEADER 캡션 비트 ----------

    #[test]
    fn picture_caption_common_attr_bit_is_or_ed_in_when_caption_present() {
        let mut para = Paragraph::default();
        let pic = Picture {
            common: crate::model::shape::CommonObjAttr {
                // 실측(이슈 §A-2)에서 확인한 파서 값 — 캡션 비트 이전에 이미 non-zero.
                attr: 0x042A_2211,
                ..Default::default()
            },
            caption: Some(crate::model::shape::Caption::default()),
            ..Default::default()
        };
        para.controls.push(Control::Picture(Box::new(pic)));

        let mut report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut report);

        let Control::Picture(pic) = &para.controls[0] else {
            panic!("Picture control expected");
        };
        assert_eq!(
            pic.common.attr, 0x242A_2211,
            "캡션이 있으면 bit 29 가 OR 되어야 함(recompute 아님)"
        );
        assert_eq!(report.picture_caption_common_attr_materialized, 1);

        // 멱등: 다시 적용해도 카운트가 늘지 않아야 함.
        let mut second = AdapterReport::new();
        adapt_paragraph(&mut para, &mut second);
        assert_eq!(second.picture_caption_common_attr_materialized, 0);
    }

    #[test]
    fn picture_caption_common_attr_bit_untouched_without_caption() {
        let mut para = Paragraph::default();
        let pic = Picture {
            common: crate::model::shape::CommonObjAttr {
                attr: 0x042A_2211,
                ..Default::default()
            },
            caption: None,
            ..Default::default()
        };
        para.controls.push(Control::Picture(Box::new(pic)));

        let mut report = AdapterReport::new();
        adapt_paragraph(&mut para, &mut report);

        let Control::Picture(pic) = &para.controls[0] else {
            panic!("Picture control expected");
        };
        assert_eq!(
            pic.common.attr, 0x042A_2211,
            "캡션이 없으면 bit 29 를 켜면 안 됨(거짓양성 방지)"
        );
        assert_eq!(report.picture_caption_common_attr_materialized, 0);
    }

    // ---------- #2767 결함 B(잔여) — HiddenComment · 그룹 내부 그림 캡션 remap ----------

    #[test]
    fn hidden_comment_picture_bin_ref_is_remapped() {
        let inner_pic = Picture {
            image_attr: crate::model::image::ImageAttr {
                bin_data_id: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut inner_para = Paragraph::default();
        inner_para
            .controls
            .push(Control::Picture(Box::new(inner_pic)));
        let comment = crate::model::control::HiddenComment {
            paragraphs: vec![inner_para],
        };
        let mut ctrl = Control::HiddenComment(Box::new(comment));

        // remap[1] = 2 : bin_data_id 1 을 2 로 재배치.
        let remap = vec![0u16, 2, 1];
        remap_bin_refs_in_control(&mut ctrl, &remap);

        let Control::HiddenComment(comment) = &ctrl else {
            panic!("HiddenComment control expected");
        };
        let Control::Picture(pic) = &comment.paragraphs[0].controls[0] else {
            panic!("Picture control expected inside HiddenComment");
        };
        assert_eq!(
            pic.image_attr.bin_data_id, 2,
            "숨은설명 안 그림의 bin_data_id 도 재정렬 remap 을 반영해야 함"
        );
    }

    #[test]
    fn grouped_picture_caption_bin_ref_is_remapped() {
        // ShapeObject::Picture(그룹 내부 그림)는 drawing_mut()이 None 이라 공통
        // caption remap 경로를 타지 않는다 — Picture 자신의 caption 을 직접 재귀해야 함.
        let inner_pic = Picture {
            image_attr: crate::model::image::ImageAttr {
                bin_data_id: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut caption_para = Paragraph::default();
        caption_para
            .controls
            .push(Control::Picture(Box::new(inner_pic)));
        let grouped_pic = Picture {
            caption: Some(crate::model::shape::Caption {
                paragraphs: vec![caption_para],
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut shape = ShapeObject::Picture(Box::new(grouped_pic));

        let remap = vec![0u16, 2, 1];
        remap_bin_refs_in_shape(&mut shape, &remap);

        let ShapeObject::Picture(pic) = &shape else {
            panic!("Picture shape expected");
        };
        let caption = pic.caption.as_ref().expect("caption preserved");
        let Control::Picture(inner) = &caption.paragraphs[0].controls[0] else {
            panic!("Picture control expected inside caption");
        };
        assert_eq!(
            inner.image_attr.bin_data_id, 2,
            "그룹 내부 그림 캡션 안 그림의 bin_data_id 도 재정렬 remap 을 반영해야 함"
        );
    }

    #[test]
    fn hidden_comment_picture_is_collected_into_bin_order() {
        use crate::model::control::HiddenComment;
        use crate::model::image::Picture;
        use std::collections::BTreeSet;

        let mut pic = Picture::default();
        pic.image_attr.bin_data_id = 2;
        let mut comment_para = Paragraph::default();
        comment_para.controls.push(Control::Picture(Box::new(pic)));
        let ctrl = Control::HiddenComment(Box::new(HiddenComment {
            paragraphs: vec![comment_para],
        }));

        let mut order = Vec::new();
        let mut seen = BTreeSet::new();
        collect_bin_order_from_control(&ctrl, 2, &mut order, &mut seen);

        assert_eq!(
            order,
            vec![2],
            "숨은설명 안 그림의 bin_data_id 가 순서 수집에서 누락됨(숨은설명 문단 미방문)"
        );
    }
}
