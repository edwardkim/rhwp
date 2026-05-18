//! HWPX → HWP IR 어댑터 통합 테스트 (#178)
//!
//! Stage 1: 베이스라인 측정 (페이지 폭주 + 영역별 차이 인벤토리).
//!         아직 어댑터 본체가 동작하지 않으므로 회복 검증 없음 — 측정만.

use rhwp::document_core::converters::diagnostics::diff_hwpx_vs_serializer_assumptions;
use rhwp::document_core::converters::hwpx_to_hwp::{
    convert_hwpx_to_hwp_ir, convert_if_hwpx_source,
};
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{CommonObjAttr, ShapeObject};
use rhwp::model::style::FillType;
use rhwp::model::table::Table;
use rhwp::parser::cfb_reader::CfbReader;

fn load_sample(name: &str) -> Vec<u8> {
    let path = format!("samples/hwpx/{}", name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("샘플 로드 실패 {}: {}", path, e))
}

fn oracle_hwp_page_count_for_hwpx(name: &str) -> Option<u32> {
    let stem = name.strip_suffix(".hwpx").unwrap_or(name);
    let path = format!("samples/hwpx/hancom-hwp/{}.hwp", stem);
    let bytes = std::fs::read(&path).ok()?;
    let core = DocumentCore::from_bytes(&bytes).ok()?;
    Some(core.page_count())
}

fn expected_hwp_page_count(name: &str, fallback_hwpx_pages: u32) -> u32 {
    oracle_hwp_page_count_for_hwpx(name).unwrap_or(fallback_hwpx_pages)
}

fn page_count_after_hwp_export(hwpx_bytes: &[u8]) -> (u32, u32) {
    let core = DocumentCore::from_bytes(hwpx_bytes).expect("HWPX 로드 실패");
    let original_pages = core.page_count();

    let hwp_bytes = core.export_hwp_native().expect("HWP 직렬화 실패");

    let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("HWP 재로드 실패");
    let reloaded_pages = reloaded.page_count();

    (original_pages, reloaded_pages)
}

/// 베이스라인 측정: 현 단계는 페이지 폭주 (reloaded > orig) 가 발생하는 것이 "정상".
/// 어댑터 영역별 매핑이 누적되면서 폭주 비율이 줄고, Stage 5 완료 시점에는
/// reloaded == orig 가 되도록 게이트가 강화된다.
fn assert_explosion_baseline(name: &str, bytes: &[u8]) {
    let (orig, reloaded) = page_count_after_hwp_export(bytes);
    eprintln!(
        "[#178 baseline] {}: orig={}, reloaded={}",
        name, orig, reloaded
    );
    assert!(orig >= 1, "{}: 원본 페이지 수 측정 실패", name);
    assert!(
        reloaded > orig,
        "{}: 현 단계는 폭주가 발생해야 정상 (어댑터 미적용). orig={}, reloaded={}",
        name,
        orig,
        reloaded
    );
}

#[test]
fn baseline_page_count_explosion_hwpx_h_01() {
    assert_explosion_baseline("hwpx-h-01", &load_sample("hwpx-h-01.hwpx"));
}

#[test]
fn baseline_page_count_explosion_hwpx_h_02() {
    assert_explosion_baseline("hwpx-h-02", &load_sample("hwpx-h-02.hwpx"));
}

#[test]
fn baseline_page_count_explosion_hwpx_h_03() {
    let bytes = load_sample("hwpx-h-03.hwpx");
    let (orig, reloaded) = page_count_after_hwp_export(&bytes);
    eprintln!(
        "[#178 baseline] hwpx-h-03: orig={}, reloaded={}",
        orig, reloaded
    );
    // hwpx-h-03 은 폭주 여부 자체가 미확정 — 측정만 기록.
    assert!(orig >= 1);
    assert!(reloaded >= 1);
}

#[test]
fn baseline_diff_inventory_hwpx_h_01() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let summary = diff_hwpx_vs_serializer_assumptions(core.document());
    eprintln!("[#178 inventory] hwpx-h-01:\n{}", summary.human_report());
    // 영역별 카운트는 측정만. assert 는 의미있는 영역이 1개 이상 검출됐는지.
    let counts = summary.counts_by_area();
    let interesting = counts.iter().any(|(a, c)| {
        *c > 0
            && (*a == "table.raw_ctrl_data"
                || *a == "paragraph.line_seg.vertical_pos"
                || *a == "cell.list_attr.bit16")
    });
    assert!(
        interesting,
        "hwpx-h-01 에서 위반 영역이 검출돼야 함 (페이지 폭주가 발생하므로). counts={:?}",
        counts
    );
}

#[test]
fn adapter_deterministic_across_clones() {
    // 두 개의 동일 클론에 어댑터를 적용하면 결과가 같다 (결정론적 동작).
    let bytes = load_sample("hwpx-h-01.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");

    let mut doc1 = core.document().clone();
    let mut doc2 = core.document().clone();

    let r1 = convert_hwpx_to_hwp_ir(&mut doc1);
    let r2 = convert_hwpx_to_hwp_ir(&mut doc2);
    assert_eq!(r1, r2);
}

#[test]
fn adapter_skips_hwp_source() {
    let mut doc = rhwp::model::document::Document::default();
    let report = convert_if_hwpx_source(&mut doc, rhwp::parser::FileFormat::Hwp);
    assert_eq!(
        report.skipped_reason.as_deref(),
        Some("source_format != Hwpx/Hwp3")
    );
}

// ============================================================
// Stage 2 — table.raw_ctrl_data 합성 검증
// ============================================================

#[test]
fn stage2_raw_ctrl_data_synthesized_for_hwpx_h_01() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");

    // 어댑터 적용 전: raw_ctrl_data 가 모두 비어있어야 함 (HWPX 출처 특성)
    let mut empty_count_before = 0;
    for section in &core.document().sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Table(t) = ctrl {
                    if t.raw_ctrl_data.is_empty() {
                        empty_count_before += 1;
                    }
                }
            }
        }
    }
    assert!(
        empty_count_before > 0,
        "HWPX 출처에는 빈 raw_ctrl_data 가 있어야 함"
    );

    // 어댑터 적용
    let mut doc = core.document().clone();
    let report = convert_hwpx_to_hwp_ir(&mut doc);
    assert!(
        report.tables_ctrl_data_synthesized > 0,
        "어댑터가 ctrl_data 를 합성해야 함. report={:?}",
        report
    );

    // 어댑터 적용 후: 모든 표의 raw_ctrl_data 가 채워져 있어야 함
    let mut empty_count_after = 0;
    for section in &doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Table(t) = ctrl {
                    if t.raw_ctrl_data.is_empty() {
                        empty_count_after += 1;
                    }
                }
            }
        }
    }
    assert_eq!(
        empty_count_after, 0,
        "어댑터 적용 후 모든 표는 raw_ctrl_data 가 채워져야 함"
    );
}

#[test]
fn stage2_diagnostics_no_longer_flag_table_ctrl_data() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let mut doc = core.document().clone();
    convert_hwpx_to_hwp_ir(&mut doc);

    let summary = diff_hwpx_vs_serializer_assumptions(&doc);
    let counts = summary.counts_by_area();
    let ctrl_data_count = counts
        .iter()
        .find(|(a, _)| *a == "table.raw_ctrl_data")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    assert_eq!(
        ctrl_data_count, 0,
        "어댑터 적용 후 진단 도구가 table.raw_ctrl_data 위반을 보고하지 않아야 함. counts={:?}",
        counts
    );
}

#[test]
fn stage2_idempotent_does_not_double_synthesize() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let mut doc = core.document().clone();

    let r1 = convert_hwpx_to_hwp_ir(&mut doc);
    let r2 = convert_hwpx_to_hwp_ir(&mut doc);

    assert!(r1.tables_ctrl_data_synthesized > 0, "1차 호출 시 합성 발생");
    assert_eq!(
        r2.tables_ctrl_data_synthesized, 0,
        "2차 호출 시 합성 0 (idempotent)"
    );
}

#[test]
fn stage2_hwp_source_unchanged() {
    // HWP 원본 로드 → 어댑터 적용 → 표 raw_ctrl_data 가 변경되지 않아야 함
    // (HWP 출처는 raw_ctrl_data 가 이미 비어있지 않으므로 어댑터 가드에 막힘)
    let path = "samples/hwp_table_test.hwp";
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("[skip] {} 없음", path);
            return;
        }
    };
    let core = DocumentCore::from_bytes(&bytes).expect("HWP 로드 실패");
    let mut doc = core.document().clone();

    // 어댑터 적용 전 raw_ctrl_data 스냅샷
    let snapshot_before: Vec<Vec<u8>> = doc
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::Table(t) => Some(t.raw_ctrl_data.clone()),
            _ => None,
        })
        .collect();

    convert_hwpx_to_hwp_ir(&mut doc);

    let snapshot_after: Vec<Vec<u8>> = doc
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::Table(t) => Some(t.raw_ctrl_data.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(
        snapshot_before, snapshot_after,
        "HWP 출처 raw_ctrl_data 는 어댑터에 의해 변경되지 않아야 함"
    );
}

/// Stage 2 베이스라인 측정: 어댑터 적용 후 페이지 폭주 비율이 줄어야 함.
/// (완전 회복은 Stage 4 lineseg vpos 사전계산 후, 단계 회귀 측정 목적)
fn page_count_with_adapter(hwpx_bytes: &[u8]) -> (u32, u32) {
    let core = DocumentCore::from_bytes(hwpx_bytes).expect("HWPX 로드 실패");
    let original_pages = core.page_count();

    let mut doc = core.document().clone();
    convert_hwpx_to_hwp_ir(&mut doc);

    // 어댑터 적용된 doc 으로 직렬화 — DocumentCore 우회
    let hwp_bytes = rhwp::serializer::serialize_hwp(&doc).expect("직렬화 실패");

    let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("HWP 재로드 실패");
    let reloaded_pages = reloaded.page_count();

    (original_pages, reloaded_pages)
}

#[test]
fn stage2_page_count_after_adapter_hwpx_h_01() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let (orig, after) = page_count_with_adapter(&bytes);
    let (_, before) = page_count_after_hwp_export(&bytes);
    eprintln!(
        "[#178 Stage 2] hwpx-h-01: orig={}, before_adapter={}, after_adapter={}",
        orig, before, after
    );
    // 회복 단계 — Stage 5 까지는 부분 개선만 기대.
    // 어댑터로 인해 폭주가 더 심해지면 Stage 2 가 잘못된 합성을 한 것이므로 실패.
    assert!(
        after <= before,
        "어댑터 적용 후 페이지 수가 더 늘면 회귀: before={} after={}",
        before,
        after
    );
}

#[test]
fn task888_basic_table_materializes_hancom_table_attrs() {
    let bytes = load_sample("basic-table-01.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let mut doc = core.document().clone();

    let report = convert_hwpx_to_hwp_ir(&mut doc);
    let table = doc
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .find_map(|ctrl| match ctrl {
            Control::Table(t) => Some(t),
            _ => None,
        })
        .expect("basic-table-01 표 없음");

    assert_eq!(report.table_ctrl_header_attr_materialized, 1);
    assert_eq!(report.table_record_attr_materialized, 1);
    assert_eq!(
        table.raw_table_record_attr, 0x0000_0006,
        "HWPX table record attr는 pageBreak/repeatHeader/noAdjust 계약 필드로 재구성한다"
    );
    assert_eq!(report.table_record_row_sizes_materialized, 1);
    assert_eq!(table.row_sizes, vec![4, 4, 4]);
    assert!(table.raw_ctrl_data.len() >= 4);
    assert_eq!(
        u32::from_le_bytes([
            table.raw_ctrl_data[0],
            table.raw_ctrl_data[1],
            table.raw_ctrl_data[2],
            table.raw_ctrl_data[3],
        ]),
        0x082a_2210
    );
}

#[test]
fn task888_expense_report_materializes_tac_table_ctrl_attrs() {
    let bytes = load_sample("expense_report.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let mut doc = core.document().clone();

    let report = convert_hwpx_to_hwp_ir(&mut doc);
    let mut tac_attrs = Vec::new();
    let mut tac_row_sizes = Vec::new();

    for section in &doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Table(t) = ctrl {
                    if t.common.treat_as_char {
                        assert!(t.raw_ctrl_data.len() >= 4, "TAC table raw_ctrl_data");
                        let packed = u32::from_le_bytes([
                            t.raw_ctrl_data[0],
                            t.raw_ctrl_data[1],
                            t.raw_ctrl_data[2],
                            t.raw_ctrl_data[3],
                        ]);
                        assert_ne!(packed, 0, "TAC table CTRL_HEADER attr must be materialized");
                        assert_eq!(t.attr, packed);
                        tac_attrs.push(packed);
                        tac_row_sizes.push(t.row_sizes.clone());
                    }
                }
            }
        }
    }

    assert_eq!(tac_attrs.len(), 2, "expense_report TAC table count");
    assert_eq!(
        tac_row_sizes,
        vec![vec![5, 3, 3], vec![4, 1, 4, 3, 6, 1, 3, 1, 2]],
        "TAC table row_sizes must be row cell counts, not row heights"
    );
    assert_eq!(report.table_ctrl_header_attr_materialized, 2);
    assert_eq!(report.table_record_row_sizes_materialized, 2);
}

#[test]
fn task888_expense_report_normalizes_transparent_paragraph_border_fill() {
    let bytes = load_sample("expense_report.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let mut doc = core.document().clone();

    let report = convert_hwpx_to_hwp_ir(&mut doc);
    assert_eq!(report.border_fills_no_fill_normalized, 1);

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

    assert!(
        !refs.is_empty(),
        "paragraph/char BorderFill refs must exist"
    );
    for id in refs {
        let border_fill = doc
            .doc_info
            .border_fills
            .get(id.saturating_sub(1) as usize)
            .expect("valid BorderFill ref");
        assert!(
            matches!(border_fill.fill.fill_type, FillType::None),
            "paragraph/char BorderFill #{} must be normalized to no-fill",
            id
        );
    }
}

#[test]
fn task888_expense_report_parses_page_border_fills() {
    let bytes = load_sample("expense_report.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let section_def = &core.document().sections[0].section_def;

    assert_eq!(section_def.page_border_fill.attr, 0x0000_0041);
    assert_eq!(section_def.page_border_fill.border_fill_id, 3);
    assert_eq!(section_def.page_border_fill.spacing_left, 4252);
    assert_eq!(section_def.page_border_fill.spacing_right, 4252);
    assert_eq!(section_def.page_border_fill.spacing_top, 4252);
    assert_eq!(section_def.page_border_fill.spacing_bottom, 4252);

    assert_eq!(section_def.extra_page_border_fills.len(), 2);
    assert_eq!(section_def.extra_page_border_fills[0].attr, 0x0000_0041);
    assert_eq!(section_def.extra_page_border_fills[0].border_fill_id, 3);
    assert_eq!(section_def.extra_page_border_fills[1].attr, 0x0000_0001);
    assert_eq!(section_def.extra_page_border_fills[1].border_fill_id, 3);
}

#[test]
fn task888_expense_report_page_border_fills_survive_hwp_save_reload() {
    let bytes = load_sample("expense_report.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");

    let hwp_bytes = core.export_hwp_with_adapter().expect("HWP 직렬화 실패");
    let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("HWP 재로드 실패");
    let section_def = &reloaded.document().sections[0].section_def;

    assert_eq!(section_def.page_border_fill.attr, 0x0000_0041);
    assert_eq!(section_def.page_border_fill.border_fill_id, 3);
    assert_eq!(section_def.page_border_fill.spacing_left, 4252);
    assert_eq!(section_def.page_border_fill.spacing_right, 4252);
    assert_eq!(section_def.page_border_fill.spacing_top, 4252);
    assert_eq!(section_def.page_border_fill.spacing_bottom, 4252);

    assert_eq!(section_def.extra_page_border_fills.len(), 2);
    assert_eq!(section_def.extra_page_border_fills[0].attr, 0x0000_0041);
    assert_eq!(section_def.extra_page_border_fills[0].border_fill_id, 3);
    assert_eq!(section_def.extra_page_border_fills[1].attr, 0x0000_0001);
    assert_eq!(section_def.extra_page_border_fills[1].border_fill_id, 3);
}

#[test]
fn task899_business_overview_cell_backgrounds_use_no_pattern() {
    let bytes = load_sample("business_overview.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let doc = core.document();

    for border_fill_id in [5_u16, 6, 7] {
        let border_fill = doc
            .doc_info
            .border_fills
            .get((border_fill_id - 1) as usize)
            .unwrap_or_else(|| panic!("BorderFill #{} 없음", border_fill_id));
        assert!(
            matches!(border_fill.fill.fill_type, FillType::Solid),
            "BorderFill #{} must be solid fill",
            border_fill_id
        );
        let solid = border_fill
            .fill
            .solid
            .as_ref()
            .unwrap_or_else(|| panic!("BorderFill #{} solid fill 없음", border_fill_id));
        assert!(
            solid.background_color != 0xffff_ffff,
            "BorderFill #{} must preserve faceColor",
            border_fill_id
        );
        assert_eq!(
            solid.pattern_type, -1,
            "BorderFill #{} has faceColor but no hatchStyle; HWP save must encode no-pattern as -1",
            border_fill_id
        );
    }
}

// ============================================================
// Stage 4 — lineseg lh/vpos 사전계산 + SectionDef 컨트롤 삽입 검증
// ============================================================

#[test]
fn stage4_section_def_control_inserted() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");

    // 어댑터 적용 전: 첫 문단에 SectionDef 컨트롤이 없어야 함 (HWPX 출처 특성)
    let first_para_orig = &core.document().sections[0].paragraphs[0];
    assert!(
        !first_para_orig
            .controls
            .iter()
            .any(|c| matches!(c, Control::SectionDef(_))),
        "HWPX 출처 첫 문단에 SectionDef 가 이미 있다면 가정 위반"
    );

    let mut doc = core.document().clone();
    let report = convert_hwpx_to_hwp_ir(&mut doc);
    assert!(
        report.section_def_controls_inserted > 0,
        "SectionDef 삽입이 발생해야 함"
    );

    // 어댑터 적용 후: 모든 섹션의 첫 문단에 SectionDef 가 있어야 함
    for (s_idx, section) in doc.sections.iter().enumerate() {
        let first_para = &section.paragraphs[0];
        assert!(
            first_para
                .controls
                .iter()
                .any(|c| matches!(c, Control::SectionDef(_))),
            "섹션 {} 의 첫 문단에 SectionDef 컨트롤 없음",
            s_idx
        );
    }
}

#[test]
fn stage4_section_def_idempotent() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let mut doc = core.document().clone();

    let r1 = convert_hwpx_to_hwp_ir(&mut doc);
    let r2 = convert_hwpx_to_hwp_ir(&mut doc);
    assert!(r1.section_def_controls_inserted > 0);
    assert_eq!(
        r2.section_def_controls_inserted, 0,
        "2차 호출 시 삽입 0 (idempotent)"
    );
}

#[test]
fn stage4_page_def_preserved_after_roundtrip() {
    // 어댑터 적용 후 직렬화 → 재로드 시 PageDef (width, height, margins) 가 보존돼야 함.
    let bytes = load_sample("hwpx-h-01.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let orig_pd = core.document().sections[0].section_def.page_def.clone();

    let mut doc = core.document().clone();
    convert_hwpx_to_hwp_ir(&mut doc);
    let hwp_bytes = rhwp::serializer::serialize_hwp(&doc).expect("직렬화 실패");
    let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("재로드 실패");
    let reload_pd = &reloaded.document().sections[0].section_def.page_def;

    assert_eq!(orig_pd.width, reload_pd.width, "width 보존");
    assert_eq!(orig_pd.height, reload_pd.height, "height 보존");
    assert_eq!(
        orig_pd.margin_left, reload_pd.margin_left,
        "margin_left 보존"
    );
    assert_eq!(
        orig_pd.margin_right, reload_pd.margin_right,
        "margin_right 보존"
    );
    assert_eq!(orig_pd.margin_top, reload_pd.margin_top, "margin_top 보존");
    assert_eq!(
        orig_pd.margin_bottom, reload_pd.margin_bottom,
        "margin_bottom 보존"
    );
}

/// Stage 4 핵심 게이트: 어댑터 적용 → 직렬화 → 재로드 시 페이지 수가 HWP 저장 기준과 일치.
fn assert_page_count_recovered(name: &str, bytes: &[u8]) {
    let (orig, after) = page_count_with_adapter(bytes);
    let expected = expected_hwp_page_count(name, orig);
    eprintln!(
        "[#178 Stage 4] {}: orig={}, expected_hwp={}, after_adapter={}",
        name, orig, expected, after
    );
    assert_eq!(
        after, expected,
        "{}: 어댑터 적용 후 페이지 수 {} != HWP 저장 기준 {} (HWPX 원본 {})",
        name, after, expected, orig
    );
}

#[test]
fn stage4_page_count_recovered_hwpx_h_01() {
    assert_page_count_recovered("hwpx-h-01", &load_sample("hwpx-h-01.hwpx"));
}

#[test]
fn stage4_page_count_recovered_hwpx_h_02() {
    assert_page_count_recovered("hwpx-h-02", &load_sample("hwpx-h-02.hwpx"));
}

#[test]
fn stage4_page_count_recovered_hwpx_h_03() {
    assert_page_count_recovered("hwpx-h-03", &load_sample("hwpx-h-03.hwpx"));
}

// ============================================================
// Stage 5 — 통합 진입점 export_hwp_with_adapter() 검증
// ============================================================

#[test]
fn stage5_export_hwp_with_adapter_hwpx_source_recovers_pages() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");
    let orig = core.page_count();

    let hwp_bytes = core.export_hwp_with_adapter().expect("HWP 직렬화");
    let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("HWP 재로드");

    assert_eq!(
        reloaded.page_count(),
        orig,
        "어댑터 통합 진입점: 페이지 수 보존 (orig={}, reloaded={})",
        orig,
        reloaded.page_count()
    );
}

#[test]
fn stage5_export_hwp_with_adapter_hwp_source_unchanged() {
    // HWP 원본 — 어댑터는 no-op (source_format != Hwpx)
    let path = "samples/hwp_table_test.hwp";
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("[skip] {} 없음", path);
            return;
        }
    };
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWP 로드");

    let bytes_native = core.export_hwp_native().expect("native 직렬화");
    let bytes_adapter = core.export_hwp_with_adapter().expect("adapter 직렬화");

    assert_eq!(
        bytes_native, bytes_adapter,
        "HWP 출처는 어댑터 호출이 native 와 동일 결과여야 함"
    );
}

#[test]
fn stage5_export_hwp_with_adapter_idempotent_on_repeated_calls() {
    // 같은 DocumentCore 에 export_hwp_with_adapter() 를 두 번 호출해도 저장 결과가 같다.
    // Stage #854부터 어댑터는 저장용 clone에만 적용되어 live IR을 변경하지 않는다.
    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");

    let first = core.export_hwp_with_adapter().expect("1차");
    let second = core.export_hwp_with_adapter().expect("2차");

    assert_eq!(
        first, second,
        "동일 DocumentCore 에 어댑터 통합 진입점 2회 호출 시 같은 bytes"
    );
}

#[test]
fn stage5_all_three_samples_recover_via_unified_entry_point() {
    for name in ["hwpx-h-01.hwpx", "hwpx-h-02.hwpx", "hwpx-h-03.hwpx"] {
        let bytes = load_sample(name);
        let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");
        let orig = core.page_count();
        let expected = expected_hwp_page_count(name, orig);

        let hwp_bytes = core.export_hwp_with_adapter().expect("HWP 직렬화");
        let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("HWP 재로드");

        assert_eq!(
            reloaded.page_count(),
            expected,
            "{}: HWP 저장 기준 페이지 수 일치 (orig={}, expected_hwp={}, reloaded={})",
            name,
            orig,
            expected,
            reloaded.page_count()
        );
    }
}

// ============================================================
// Stage 6 — serialize_hwp_with_verify 명시 검증 함수
// ============================================================

#[test]
fn stage6_verify_recovered_for_hwpx_h_01() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");
    let v = core.serialize_hwp_with_verify().expect("verify");
    eprintln!(
        "[#178 Stage 6] verify hwpx-h-01: before={}, after={}, recovered={}, bytes={}",
        v.page_count_before, v.page_count_after, v.recovered, v.bytes_len
    );
    assert!(
        v.recovered,
        "페이지 회복 실패: before={} after={}",
        v.page_count_before, v.page_count_after
    );
    assert_eq!(v.page_count_before, v.page_count_after);
    assert!(v.bytes_len > 0);
}

#[test]
fn stage6_verify_recovered_for_all_three_samples() {
    for name in ["hwpx-h-01.hwpx", "hwpx-h-02.hwpx", "hwpx-h-03.hwpx"] {
        let bytes = load_sample(name);
        let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");
        let v = core.serialize_hwp_with_verify().expect("verify");
        let expected = expected_hwp_page_count(name, v.page_count_before);
        assert_eq!(
            v.page_count_after, expected,
            "{}: HWP 저장 기준 페이지 수 불일치 before={} expected_hwp={} after={}",
            name, v.page_count_before, expected, v.page_count_after
        );
    }
}

#[test]
fn stage6_verify_for_hwp_source_also_recovered() {
    // HWP 출처 — 어댑터는 no-op, 그래도 verify 는 동작해야 함 (recovered=true)
    let path = "samples/hwp_table_test.hwp";
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("[skip] {} 없음", path);
            return;
        }
    };
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWP 로드");
    let v = core.serialize_hwp_with_verify().expect("verify");
    assert!(v.recovered, "HWP 출처 자기 재로드 페이지 수 일치");
}

fn first_line_vpos(core: &DocumentCore, section_idx: usize, para_idx: usize) -> i32 {
    core.document().sections[section_idx].paragraphs[para_idx].line_segs[0].vertical_pos
}

#[test]
fn task949_stage33_hwpx_h03_explicit_lineseg_vpos_preserved_on_load() {
    let bytes = load_sample("hwpx-h-03.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");

    assert_eq!(
        first_line_vpos(&core, 0, 18),
        68258,
        "HWPX source lineSegArray의 명시 vertpos를 자동 reflow가 덮어쓰면 안 됨"
    );
}

#[test]
fn task949_stage33_hwpx_h03_explicit_lineseg_vpos_survives_adapter_export_reload() {
    let bytes = load_sample("hwpx-h-03.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");

    let hwp_bytes = core.export_hwp_with_adapter().expect("HWP 직렬화");
    let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("HWP 재로드");

    assert_eq!(
        first_line_vpos(&reloaded, 0, 18),
        68258,
        "HWPX -> HWP 저장/재로드 후에도 paragraph 18의 lineSeg vpos가 보존되어야 함"
    );
}

#[test]
fn stage5_wasm_api_export_hwp_uses_adapter() {
    // wasm_api 의 export_hwp (네이티브 래퍼: export_hwp_native_wrapper 가 아니라
    // HwpDocument 자체가 DerefMut<DocumentCore>) 가 어댑터를 자동 적용하는지 확인.
    // 본 테스트는 네이티브 환경에서 wasm_api 진입점 동작을 검증.
    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("HWPX 로드");
    let orig = doc.page_count();

    // export_hwp 는 wasm_bindgen 메서드라 직접 호출 불가 → 동등한 export_hwp_with_adapter 호출
    let hwp_bytes = doc.export_hwp_with_adapter().expect("어댑터 직렬화");
    let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("HWP 재로드");

    assert_eq!(
        reloaded.page_count(),
        orig as u32,
        "wasm_api 경로: 페이지 수 보존 (orig={}, reloaded={})",
        orig,
        reloaded.page_count()
    );
}

#[test]
fn task903_hwpx_h_01_section_count_is_materialized_by_adapter() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let expected_section_count = core.document().sections.len() as u16;

    assert_eq!(
        expected_section_count, 2,
        "fixture는 2개 section으로 구성되어야 함"
    );
    assert_ne!(
        core.document().doc_properties.section_count,
        expected_section_count,
        "HWPX parser baseline은 section_count를 아직 보정하지 않아야 함"
    );

    let report = convert_hwpx_to_hwp_ir(core.document_mut());

    assert_eq!(
        report.file_header_compression_normalized, 1,
        "어댑터가 HWPX 출처 FileHeader를 compressed HWP5 저장 관례로 보정해야 함"
    );
    assert!(
        core.document().header.compressed,
        "HWP 저장 전 FileHeader.compressed=true여야 함"
    );
    assert_eq!(
        core.document().header.flags & 0x01,
        0x01,
        "HWP 저장 전 FileHeader flags에 compressed bit가 켜져야 함"
    );
    assert_eq!(
        report.doc_properties_section_count_normalized, 1,
        "어댑터가 DocProperties.section_count를 한 번 보정해야 함"
    );
    assert_eq!(
        core.document().doc_properties.section_count,
        expected_section_count,
        "HWP 저장 전 DocProperties.section_count는 실제 section 수와 같아야 함"
    );
    assert!(
        core.document().doc_properties.raw_data.is_none(),
        "DOCUMENT_PROPERTIES raw_data가 남으면 보정값이 직렬화되지 않음"
    );
    assert!(
        core.document().doc_info.raw_stream_dirty,
        "DocInfo raw stream을 재직렬화해야 section_count 보정이 HWP에 반영됨"
    );
}

#[test]
fn task903_hwpx_h_01_para_shape_margin_children_are_parsed() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let reference_bytes =
        std::fs::read("samples/hwpx/hancom-hwp/hwpx-h-01.hwp").expect("한컴 정답 HWP 필요");
    let reference = DocumentCore::from_bytes(&reference_bytes).expect("한컴 정답 HWP 파싱 실패");

    let source_shapes = &core.document().doc_info.para_shapes;
    let reference_shapes = &reference.document().doc_info.para_shapes;
    assert!(
        source_shapes.len() > 20 && reference_shapes.len() > 20,
        "fixture는 ParaShape 20개 이상을 가져야 함"
    );

    for idx in [10usize, 17, 20] {
        let actual = &source_shapes[idx];
        let expected = &reference_shapes[idx];
        assert_eq!(actual.indent, expected.indent, "ParaShape[{}].indent", idx);
        assert_eq!(
            actual.margin_left, expected.margin_left,
            "ParaShape[{}].margin_left",
            idx
        );
        assert_eq!(
            actual.margin_right, expected.margin_right,
            "ParaShape[{}].margin_right",
            idx
        );
        assert_eq!(
            actual.spacing_before, expected.spacing_before,
            "ParaShape[{}].spacing_before",
            idx
        );
        assert_eq!(
            actual.spacing_after, expected.spacing_after,
            "ParaShape[{}].spacing_after",
            idx
        );
    }

    assert_eq!(source_shapes[10].indent, -2800);
    assert_eq!(source_shapes[17].margin_left, 3000);
    assert_eq!(source_shapes[20].spacing_after, 1000);
}

#[test]
fn task903_stage31_restart_generate_impl_verify() {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let hwp_bytes = core.export_hwp_with_adapter().expect("HWP 직렬화 실패");

    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage31_restart");
    std::fs::create_dir_all(out_dir).expect("Stage31 restart output dir 생성 실패");
    let out_path = out_dir.join("hwpx-h-01.hwp");
    std::fs::write(&out_path, &hwp_bytes).expect("Stage31 restart HWP 저장 실패");

    let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("HWP 재로드 실패");
    assert!(
        hwp_bytes.len() < 600_000,
        "Stage31 restart 산출물은 compressed 저장으로 600KB 미만이어야 함 (actual={})",
        hwp_bytes.len()
    );
    assert!(
        reloaded.document().header.compressed,
        "Stage31 restart 산출물은 FileHeader.compressed=true여야 함"
    );
    assert_eq!(
        reloaded.document().header.flags & 0x01,
        0x01,
        "Stage31 restart 산출물은 FileHeader compressed bit가 켜져야 함"
    );
    assert_eq!(
        reloaded.document().doc_properties.section_count,
        2,
        "Stage31 restart 산출물은 DOCUMENT_PROPERTIES section_count=2를 가져야 함"
    );
    assert_eq!(
        reloaded.page_count(),
        9,
        "Stage31 restart 산출물은 rhwp-studio 기준 9페이지를 유지해야 함"
    );

    eprintln!(
        "[#903 Stage31 restart] generated {} bytes={} pages={}",
        out_path.display(),
        hwp_bytes.len(),
        reloaded.page_count()
    );
}

#[derive(Clone, Copy)]
enum Stage32Axis {
    Vpos,
    TablePlacement,
    CharShapeRefs,
    Text,
    ShapePlacement,
}

fn task903_load_hwp_document(path: &str) -> Option<Document> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("[skip] {} 읽기 실패: {}", path, err);
            return None;
        }
    };
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|err| {
        panic!("{} HWP 파싱 실패: {:?}", path, err);
    });
    Some(core.document().clone())
}

fn task903_stage31_restart_document() -> Document {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    convert_hwpx_to_hwp_ir(core.document_mut());
    core.document().clone()
}

fn task903_prepare_variant_for_reserialize(doc: &mut Document) {
    doc.header.compressed = true;
    doc.header.flags |= 0x01;
    doc.header.raw_data = None;
    for section in &mut doc.sections {
        section.raw_stream = None;
    }
}

fn task903_apply_stage32_axes(target: &mut Document, positive: &Document, axes: &[Stage32Axis]) {
    for (target_section, positive_section) in target.sections.iter_mut().zip(&positive.sections) {
        for (target_para, positive_para) in target_section
            .paragraphs
            .iter_mut()
            .zip(&positive_section.paragraphs)
        {
            task903_apply_axes_to_paragraph(target_para, positive_para, axes);
        }
    }
    task903_prepare_variant_for_reserialize(target);
}

fn task903_apply_axes_to_paragraph(
    target: &mut Paragraph,
    positive: &Paragraph,
    axes: &[Stage32Axis],
) {
    if task903_has_axis(axes, Stage32Axis::Vpos) {
        for (target_line, positive_line) in target.line_segs.iter_mut().zip(&positive.line_segs) {
            target_line.vertical_pos = positive_line.vertical_pos;
        }
    }

    if task903_has_axis(axes, Stage32Axis::Text) {
        target.char_count = positive.char_count;
        target.text = positive.text.clone();
        target.char_offsets = positive.char_offsets.clone();
        target.has_para_text = positive.has_para_text;
        target.tab_extended = positive.tab_extended.clone();
    }

    if task903_has_axis(axes, Stage32Axis::CharShapeRefs) {
        target.char_shapes = positive.char_shapes.clone();
    }

    for (target_ctrl, positive_ctrl) in target.controls.iter_mut().zip(&positive.controls) {
        match (target_ctrl, positive_ctrl) {
            (Control::Table(target_table), Control::Table(positive_table)) => {
                if task903_has_axis(axes, Stage32Axis::TablePlacement) {
                    task903_graft_table_placement(target_table, positive_table);
                }
                for (target_cell, positive_cell) in
                    target_table.cells.iter_mut().zip(&positive_table.cells)
                {
                    for (target_para, positive_para) in target_cell
                        .paragraphs
                        .iter_mut()
                        .zip(&positive_cell.paragraphs)
                    {
                        task903_apply_axes_to_paragraph(target_para, positive_para, axes);
                    }
                }
            }
            (Control::Shape(target_shape), Control::Shape(positive_shape)) => {
                if task903_has_axis(axes, Stage32Axis::ShapePlacement) {
                    task903_graft_shape_placement(target_shape, positive_shape);
                }
            }
            _ => {}
        }
    }
}

fn task903_has_axis(axes: &[Stage32Axis], needle: Stage32Axis) -> bool {
    axes.iter()
        .any(|axis| std::mem::discriminant(axis) == std::mem::discriminant(&needle))
}

fn task903_graft_table_placement(target: &mut Table, positive: &Table) {
    target.outer_margin_left = positive.outer_margin_left;
    target.outer_margin_right = positive.outer_margin_right;
    target.outer_margin_top = positive.outer_margin_top;
    target.outer_margin_bottom = positive.outer_margin_bottom;
    target.page_break = positive.page_break;
    target.raw_table_record_attr = positive.raw_table_record_attr;

    task903_graft_common_placement(&mut target.common, &positive.common);
    target.raw_ctrl_data = positive.raw_ctrl_data.clone();
}

fn task903_graft_shape_placement(target: &mut ShapeObject, positive: &ShapeObject) {
    task903_graft_common_placement(target.common_mut(), positive.common());
    if let (ShapeObject::Group(target_group), ShapeObject::Group(positive_group)) =
        (target, positive)
    {
        for (target_child, positive_child) in target_group
            .children
            .iter_mut()
            .zip(&positive_group.children)
        {
            task903_graft_shape_placement(target_child, positive_child);
        }
    }
}

fn task903_graft_common_placement(target: &mut CommonObjAttr, positive: &CommonObjAttr) {
    target.vert_rel_to = positive.vert_rel_to;
    target.horz_rel_to = positive.horz_rel_to;
    target.text_wrap = positive.text_wrap;
    target.raw_extra = positive.raw_extra.clone();
}

#[test]
fn task903_stage32_generate_residual_axis_probe_variants() {
    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let baseline = task903_stage31_restart_document();
    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage32_residual_axis_probe");
    std::fs::create_dir_all(out_dir).expect("Stage32 output dir 생성 실패");

    let variants: &[(&str, &[Stage32Axis])] = &[
        ("01_vpos_only", &[Stage32Axis::Vpos]),
        ("02_table_placement_tuple", &[Stage32Axis::TablePlacement]),
        ("03_char_shape_ref_tuple", &[Stage32Axis::CharShapeRefs]),
        ("04_text_tuple", &[Stage32Axis::Text]),
        ("05_shape_placement_tuple", &[Stage32Axis::ShapePlacement]),
        (
            "06_vpos_plus_table_placement",
            &[Stage32Axis::Vpos, Stage32Axis::TablePlacement],
        ),
        (
            "07_table_placement_plus_char_shape_ref",
            &[Stage32Axis::TablePlacement, Stage32Axis::CharShapeRefs],
        ),
        (
            "08_all_residual_axes",
            &[
                Stage32Axis::Vpos,
                Stage32Axis::TablePlacement,
                Stage32Axis::CharShapeRefs,
                Stage32Axis::Text,
                Stage32Axis::ShapePlacement,
            ],
        ),
    ];

    for (name, axes) in variants {
        let mut doc = baseline.clone();
        task903_apply_stage32_axes(&mut doc, &positive, axes);
        let hwp_bytes = rhwp::serializer::serialize_hwp(&doc).expect("Stage32 HWP 직렬화 실패");
        assert!(
            hwp_bytes.len() > 300_000,
            "{} 산출물이 너무 작음: {} bytes",
            name,
            hwp_bytes.len()
        );
        let out_path = out_dir.join(format!("{}.hwp", name));
        std::fs::write(&out_path, &hwp_bytes).expect("Stage32 HWP 저장 실패");

        let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("Stage32 HWP rhwp 재로드 실패");
        assert_eq!(
            reloaded.page_count(),
            9,
            "{}는 rhwp 재로드 기준 9페이지여야 함",
            name
        );
        eprintln!(
            "[#903 Stage32] generated {} bytes={} pages={}",
            out_path.display(),
            hwp_bytes.len(),
            reloaded.page_count()
        );
    }
}

fn task903_stage32_all_residual_axes_document(positive: &Document) -> Document {
    let mut doc = task903_stage31_restart_document();
    task903_apply_stage32_axes(
        &mut doc,
        positive,
        &[
            Stage32Axis::Vpos,
            Stage32Axis::TablePlacement,
            Stage32Axis::CharShapeRefs,
            Stage32Axis::Text,
            Stage32Axis::ShapePlacement,
        ],
    );
    doc
}

fn task903_para29_shape_pair_mut<'a>(
    target: &'a mut Document,
    positive: &'a Document,
) -> (&'a mut Paragraph, &'a ShapeObject) {
    let target_para = target
        .sections
        .get_mut(0)
        .and_then(|section| section.paragraphs.get_mut(29))
        .expect("target section0 para29 필요");
    let positive_shape = positive
        .sections
        .get(0)
        .and_then(|section| section.paragraphs.get(29))
        .and_then(|para| para.controls.get(0))
        .and_then(|ctrl| match ctrl {
            Control::Shape(shape) => Some(shape.as_ref()),
            _ => None,
        })
        .expect("positive section0 para29 ctrl0 shape 필요");

    let is_target_shape = matches!(target_para.controls.get(0), Some(Control::Shape(_)));
    assert!(is_target_shape, "target section0 para29 ctrl0 shape 필요");
    (target_para, positive_shape)
}

fn task903_target_shape_mut(para: &mut Paragraph) -> &mut ShapeObject {
    match para.controls.get_mut(0) {
        Some(Control::Shape(shape)) => shape.as_mut(),
        _ => panic!("target para ctrl0 shape 필요"),
    }
}

fn task903_copy_para29_ctrl_data_record(target_para: &mut Paragraph, positive: &Document) {
    let positive_para = positive
        .sections
        .get(0)
        .and_then(|section| section.paragraphs.get(29))
        .expect("positive section0 para29 필요");
    let positive_ctrl_data = positive_para
        .ctrl_data_records
        .get(0)
        .cloned()
        .unwrap_or(None);

    if target_para.ctrl_data_records.is_empty() {
        target_para.ctrl_data_records.resize(1, None);
    }
    target_para.ctrl_data_records[0] = positive_ctrl_data;
}

fn task903_copy_para29_shape_control(target_para: &mut Paragraph, positive: &Document) {
    let positive_para = positive
        .sections
        .get(0)
        .and_then(|section| section.paragraphs.get(29))
        .expect("positive section0 para29 필요");
    target_para.controls[0] = positive_para
        .controls
        .get(0)
        .expect("positive para29 ctrl0 필요")
        .clone();
    task903_copy_para29_ctrl_data_record(target_para, positive);
}

#[test]
fn task903_stage33_generate_shape_attr_probe_variants() {
    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let baseline = task903_stage32_all_residual_axes_document(&positive);
    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage33_shape_attr_probe");
    std::fs::create_dir_all(out_dir).expect("Stage33 output dir 생성 실패");

    type PatchFn = fn(&mut Document, &Document);
    let variants: &[(&str, PatchFn)] = &[
        ("01_shape_common_attr_only", |doc, positive| {
            let (target_para, positive_shape) = task903_para29_shape_pair_mut(doc, positive);
            task903_target_shape_mut(target_para).common_mut().attr = positive_shape.common().attr;
        }),
        ("02_shape_common_attr_plus_enum", |doc, positive| {
            let (target_para, positive_shape) = task903_para29_shape_pair_mut(doc, positive);
            let target_common = task903_target_shape_mut(target_para).common_mut();
            target_common.attr = positive_shape.common().attr;
            target_common.vert_rel_to = positive_shape.common().vert_rel_to;
            target_common.horz_rel_to = positive_shape.common().horz_rel_to;
            target_common.text_wrap = positive_shape.common().text_wrap;
        }),
        ("03_shape_common_full", |doc, positive| {
            let (target_para, positive_shape) = task903_para29_shape_pair_mut(doc, positive);
            *task903_target_shape_mut(target_para).common_mut() = positive_shape.common().clone();
        }),
        ("04_shape_common_full_plus_ctrl_data", |doc, positive| {
            let (target_para, positive_shape) = task903_para29_shape_pair_mut(doc, positive);
            *task903_target_shape_mut(target_para).common_mut() = positive_shape.common().clone();
            task903_copy_para29_ctrl_data_record(target_para, positive);
        }),
        ("05_stage30_para29_shape_control_full", |doc, positive| {
            let (target_para, _) = task903_para29_shape_pair_mut(doc, positive);
            task903_copy_para29_shape_control(target_para, positive);
        }),
    ];

    for (name, patch) in variants {
        let mut doc = baseline.clone();
        patch(&mut doc, &positive);
        task903_prepare_variant_for_reserialize(&mut doc);
        let hwp_bytes = rhwp::serializer::serialize_hwp(&doc).expect("Stage33 HWP 직렬화 실패");
        assert!(
            hwp_bytes.len() > 300_000,
            "{} 산출물이 너무 작음: {} bytes",
            name,
            hwp_bytes.len()
        );

        let out_path = out_dir.join(format!("{}.hwp", name));
        std::fs::write(&out_path, &hwp_bytes).expect("Stage33 HWP 저장 실패");

        let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("Stage33 HWP rhwp 재로드 실패");
        assert_eq!(
            reloaded.page_count(),
            9,
            "{}는 rhwp 재로드 기준 9페이지여야 함",
            name
        );
        eprintln!(
            "[#903 Stage33] generated {} bytes={} pages={}",
            out_path.display(),
            hwp_bytes.len(),
            reloaded.page_count()
        );
    }
}

fn task903_hash_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

fn task903_short_hash(data: &[u8]) -> String {
    task903_hash_hex(data).chars().take(16).collect()
}

fn task903_first_diff(a: &[u8], b: &[u8]) -> String {
    let max_len = a.len().max(b.len());
    for i in 0..max_len {
        if a.get(i) != b.get(i) {
            return format!(
                "{}: {:?} vs {:?}",
                i,
                a.get(i).map(|v| format!("0x{:02x}", v)),
                b.get(i).map(|v| format!("0x{:02x}", v))
            );
        }
    }
    "-".to_string()
}

fn task903_file_header_flags(data: &[u8]) -> u32 {
    let mut cfb = CfbReader::open(data).expect("CFB 열기 실패");
    let header = cfb.read_file_header().expect("FileHeader 읽기 실패");
    assert!(header.len() >= 40, "FileHeader must be at least 40 bytes");
    u32::from_le_bytes([header[36], header[37], header[38], header[39]])
}

fn task903_read_raw_stream(data: &[u8], path: &str) -> Option<Vec<u8>> {
    let mut cfb = CfbReader::open(data).ok()?;
    cfb.read_stream_raw(path).ok()
}

fn task903_read_decompressed_docinfo(data: &[u8], compressed: bool) -> Option<Vec<u8>> {
    let mut cfb = CfbReader::open(data).ok()?;
    cfb.read_doc_info(compressed).ok()
}

fn task903_read_decompressed_section(data: &[u8], index: u32, compressed: bool) -> Option<Vec<u8>> {
    let mut cfb = CfbReader::open(data).ok()?;
    cfb.read_body_text_section(index, compressed, false).ok()
}

fn task903_record_count(data: &[u8]) -> usize {
    task903_parse_records(data).len()
}

#[derive(Debug, Clone)]
struct Task903RecordMeta {
    tag: u32,
    level: u32,
    size: usize,
    offset: usize,
    data_offset: usize,
}

fn task903_parse_records(data: &[u8]) -> Vec<Task903RecordMeta> {
    let mut pos = 0usize;
    let mut records = Vec::new();
    while pos + 4 <= data.len() {
        let header = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let tag = header & 0x3ff;
        let level = (header >> 10) & 0x3ff;
        let size_field = (header >> 20) & 0xFFF;
        let mut size = size_field as usize;
        let mut data_offset = pos + 4;
        if size_field == 0xFFF {
            if pos + 8 > data.len() {
                break;
            }
            size = u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                as usize;
            data_offset = pos + 8;
        }
        if data_offset + size > data.len() {
            break;
        }
        records.push(Task903RecordMeta {
            tag,
            level,
            size,
            offset: pos,
            data_offset,
        });
        pos = data_offset + size;
    }
    records
}

fn task903_record_end(record: &Task903RecordMeta) -> usize {
    record.data_offset + record.size
}

fn task903_graft_record_tags_by_index(
    target_raw: &[u8],
    positive_raw: &[u8],
    tags: &[u32],
) -> Vec<u8> {
    let target_records = task903_parse_records(target_raw);
    let positive_records = task903_parse_records(positive_raw);
    assert_eq!(
        target_records.len(),
        positive_records.len(),
        "BodyText record count mismatch"
    );

    let mut out = Vec::with_capacity(target_raw.len().max(positive_raw.len()));
    let mut cursor = 0usize;
    for (idx, target_record) in target_records.iter().enumerate() {
        let positive_record = &positive_records[idx];
        assert_eq!(
            target_record.tag, positive_record.tag,
            "record tag mismatch at idx {}: target={} positive={}",
            idx, target_record.tag, positive_record.tag
        );
        assert_eq!(
            target_record.level, positive_record.level,
            "record level mismatch at idx {} tag {}",
            idx, target_record.tag
        );

        if cursor < target_record.offset {
            out.extend_from_slice(&target_raw[cursor..target_record.offset]);
        }

        if tags.contains(&target_record.tag) {
            out.extend_from_slice(
                &positive_raw[positive_record.offset..task903_record_end(positive_record)],
            );
        } else {
            out.extend_from_slice(
                &target_raw[target_record.offset..task903_record_end(target_record)],
            );
        }
        cursor = task903_record_end(target_record);
    }
    if cursor < target_raw.len() {
        out.extend_from_slice(&target_raw[cursor..]);
    }
    out
}

fn task903_graft_record_indices(
    target_raw: &[u8],
    positive_raw: &[u8],
    indices: &[usize],
) -> Vec<u8> {
    let target_records = task903_parse_records(target_raw);
    let positive_records = task903_parse_records(positive_raw);
    assert_eq!(
        target_records.len(),
        positive_records.len(),
        "BodyText record count mismatch"
    );

    let mut out = Vec::with_capacity(target_raw.len().max(positive_raw.len()));
    let mut cursor = 0usize;
    for (idx, target_record) in target_records.iter().enumerate() {
        let positive_record = &positive_records[idx];
        assert_eq!(
            target_record.tag, positive_record.tag,
            "record tag mismatch at idx {}: target={} positive={}",
            idx, target_record.tag, positive_record.tag
        );
        assert_eq!(
            target_record.level, positive_record.level,
            "record level mismatch at idx {} tag {}",
            idx, target_record.tag
        );

        if cursor < target_record.offset {
            out.extend_from_slice(&target_raw[cursor..target_record.offset]);
        }

        if indices.contains(&idx) {
            out.extend_from_slice(
                &positive_raw[positive_record.offset..task903_record_end(positive_record)],
            );
        } else {
            out.extend_from_slice(
                &target_raw[target_record.offset..task903_record_end(target_record)],
            );
        }
        cursor = task903_record_end(target_record);
    }
    if cursor < target_raw.len() {
        out.extend_from_slice(&target_raw[cursor..]);
    }
    out
}

fn task903_record_tag_name(tag: u32) -> &'static str {
    match tag {
        16 => "DOCUMENT_PROPERTIES",
        17 => "ID_MAPPINGS",
        18 => "BIN_DATA",
        19 => "FACE_NAME",
        20 => "BORDER_FILL",
        21 => "CHAR_SHAPE",
        22 => "TAB_DEF",
        23 => "NUMBERING",
        24 => "BULLET",
        25 => "PARA_SHAPE",
        26 => "STYLE",
        27 => "DOC_DATA",
        30 => "COMPATIBLE_DOCUMENT",
        31 => "LAYOUT_COMPATIBILITY",
        66 => "PARA_HEADER",
        67 => "PARA_TEXT",
        68 => "PARA_CHAR_SHAPE",
        69 => "PARA_LINE_SEG",
        70 => "PARA_RANGE_TAG",
        71 => "CTRL_HEADER",
        72 => "LIST_HEADER",
        73 => "PAGE_DEF",
        74 => "FOOTNOTE_SHAPE",
        75 => "PAGE_BORDER_FILL",
        76 => "SHAPE_COMPONENT",
        77 => "TABLE",
        78 => "SHAPE_LINE",
        79 => "SHAPE_RECT",
        80 => "SHAPE_ELLIPSE",
        81 => "SHAPE_ARC",
        82 => "SHAPE_POLYGON",
        83 => "SHAPE_CURVE",
        84 => "SHAPE_OLE",
        85 => "SHAPE_PICTURE",
        86 => "SHAPE_CONTAINER",
        87 => "CTRL_DATA",
        88 => "EQEDIT",
        95 => "CHART_DATA",
        _ => "UNKNOWN",
    }
}

#[test]
#[ignore]
fn task903_stage34_compare_streams_for_ir_equal_files() {
    use std::collections::BTreeSet;
    use std::fmt::Write as _;

    let positive_path =
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp";
    let failing_path =
        "output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/01_shape_common_attr_only.hwp";

    let positive = std::fs::read(positive_path).expect("Stage30 positive 파일 읽기 실패");
    let failing = std::fs::read(failing_path).expect("Stage33 failing 파일 읽기 실패");

    let positive_cfb = CfbReader::open(&positive).expect("positive CFB 열기 실패");
    let failing_cfb = CfbReader::open(&failing).expect("failing CFB 열기 실패");

    let positive_streams: BTreeSet<String> = positive_cfb.list_streams().into_iter().collect();
    let failing_streams: BTreeSet<String> = failing_cfb.list_streams().into_iter().collect();
    let all_streams: BTreeSet<String> = positive_streams.union(&failing_streams).cloned().collect();

    let positive_flags = task903_file_header_flags(&positive);
    let failing_flags = task903_file_header_flags(&failing);
    let positive_compressed = positive_flags & 0x01 != 0;
    let failing_compressed = failing_flags & 0x01 != 0;

    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage34_stream_compare");
    std::fs::create_dir_all(out_dir).expect("Stage34 output dir 생성 실패");
    let report_path = out_dir.join("stage34_stream_compare.md");

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 34 Stream Compare").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. Files").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| role | path | bytes | blake3 | flags | compressed |"
    )
    .unwrap();
    writeln!(report, "|---|---|---:|---|---:|---|").unwrap();
    writeln!(
        report,
        "| positive | `{}` | {} | `{}` | `{:#010x}` | {} |",
        positive_path,
        positive.len(),
        task903_hash_hex(&positive),
        positive_flags,
        positive_compressed
    )
    .unwrap();
    writeln!(
        report,
        "| failing | `{}` | {} | `{}` | `{:#010x}` | {} |",
        failing_path,
        failing.len(),
        task903_hash_hex(&failing),
        failing_flags,
        failing_compressed
    )
    .unwrap();

    writeln!(report).unwrap();
    writeln!(report, "## 2. Raw CFB Stream Inventory").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| stream | positive bytes | failing bytes | same bytes | positive hash | failing hash | first diff |"
    )
    .unwrap();
    writeln!(report, "|---|---:|---:|---|---|---|---|").unwrap();

    let mut differing_streams = Vec::new();
    for stream in &all_streams {
        let p_raw = task903_read_raw_stream(&positive, stream);
        let f_raw = task903_read_raw_stream(&failing, stream);
        let same = p_raw == f_raw;
        if !same {
            differing_streams.push(stream.clone());
        }
        let first_diff = match (&p_raw, &f_raw) {
            (Some(p), Some(f)) => task903_first_diff(p, f),
            (None, Some(_)) => "missing in positive".to_string(),
            (Some(_), None) => "missing in failing".to_string(),
            (None, None) => "-".to_string(),
        };
        writeln!(
            report,
            "| `{}` | {} | {} | {} | `{}` | `{}` | {} |",
            stream,
            p_raw.as_ref().map(|v| v.len()).unwrap_or(0),
            f_raw.as_ref().map(|v| v.len()).unwrap_or(0),
            same,
            p_raw
                .as_ref()
                .map(|v| task903_short_hash(v))
                .unwrap_or_else(|| "-".to_string()),
            f_raw
                .as_ref()
                .map(|v| task903_short_hash(v))
                .unwrap_or_else(|| "-".to_string()),
            first_diff
        )
        .unwrap();
    }

    writeln!(report).unwrap();
    writeln!(report, "## 3. Decompressed Record Streams").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| stream | positive bytes | failing bytes | same bytes | positive records | failing records | positive hash | failing hash | first diff |"
    )
    .unwrap();
    writeln!(report, "|---|---:|---:|---|---:|---:|---|---|---|").unwrap();

    let mut decompressed_items: Vec<(String, Option<Vec<u8>>, Option<Vec<u8>>)> = Vec::new();
    decompressed_items.push((
        "/DocInfo".to_string(),
        task903_read_decompressed_docinfo(&positive, positive_compressed),
        task903_read_decompressed_docinfo(&failing, failing_compressed),
    ));

    let positive_section_count = positive_cfb.section_count();
    let failing_section_count = failing_cfb.section_count();
    let max_sections = positive_section_count.max(failing_section_count);
    for section_idx in 0..max_sections {
        decompressed_items.push((
            format!("/BodyText/Section{}", section_idx),
            task903_read_decompressed_section(&positive, section_idx, positive_compressed),
            task903_read_decompressed_section(&failing, section_idx, failing_compressed),
        ));
    }

    let mut decompressed_diffs = Vec::new();
    let mut record_diff_report = String::new();
    for (name, p_data, f_data) in decompressed_items {
        let same = p_data == f_data;
        if !same {
            decompressed_diffs.push(name.clone());
        }
        let first_diff = match (&p_data, &f_data) {
            (Some(p), Some(f)) => task903_first_diff(p, f),
            (None, Some(_)) => "missing in positive".to_string(),
            (Some(_), None) => "missing in failing".to_string(),
            (None, None) => "-".to_string(),
        };
        writeln!(
            report,
            "| `{}` | {} | {} | {} | {} | {} | `{}` | `{}` | {} |",
            name,
            p_data.as_ref().map(|v| v.len()).unwrap_or(0),
            f_data.as_ref().map(|v| v.len()).unwrap_or(0),
            same,
            p_data
                .as_ref()
                .map(|v| task903_record_count(v))
                .unwrap_or(0),
            f_data
                .as_ref()
                .map(|v| task903_record_count(v))
                .unwrap_or(0),
            p_data
                .as_ref()
                .map(|v| task903_short_hash(v))
                .unwrap_or_else(|| "-".to_string()),
            f_data
                .as_ref()
                .map(|v| task903_short_hash(v))
                .unwrap_or_else(|| "-".to_string()),
            first_diff
        )
        .unwrap();

        if let (Some(p_bytes), Some(f_bytes)) = (&p_data, &f_data) {
            let p_records = task903_parse_records(p_bytes);
            let f_records = task903_parse_records(f_bytes);
            writeln!(record_diff_report).unwrap();
            writeln!(record_diff_report, "### `{}`", name).unwrap();
            writeln!(record_diff_report).unwrap();
            writeln!(
                record_diff_report,
                "positive records={}, failing records={}",
                p_records.len(),
                f_records.len()
            )
            .unwrap();
            writeln!(record_diff_report).unwrap();
            writeln!(
                record_diff_report,
                "| idx | status | positive tag | p lvl | p size | failing tag | f lvl | f size | p offset | f offset | first data diff |"
            )
            .unwrap();
            writeln!(
                record_diff_report,
                "|---:|---|---|---:|---:|---|---:|---:|---:|---:|---|"
            )
            .unwrap();

            let max_records = p_records.len().max(f_records.len());
            let mut shown = 0usize;
            let mut diff_count = 0usize;
            for idx in 0..max_records {
                let p_rec = p_records.get(idx);
                let f_rec = f_records.get(idx);
                let status = match (p_rec, f_rec) {
                    (Some(p), Some(f))
                        if p.tag == f.tag && p.level == f.level && p.size == f.size =>
                    {
                        let p_slice = &p_bytes[p.data_offset..p.data_offset + p.size];
                        let f_slice = &f_bytes[f.data_offset..f.data_offset + f.size];
                        if p_slice == f_slice {
                            "OK"
                        } else {
                            "DATA_DIFF"
                        }
                    }
                    (Some(p), Some(f)) if p.tag == f.tag && p.level == f.level => "SIZE_DIFF",
                    (Some(_), Some(_)) => "TAG_OR_LEVEL_DIFF",
                    (Some(_), None) => "MISSING_IN_FAILING",
                    (None, Some(_)) => "MISSING_IN_POSITIVE",
                    (None, None) => "OK",
                };

                if status == "OK" {
                    continue;
                }
                diff_count += 1;
                if shown >= 40 {
                    continue;
                }
                shown += 1;

                let first_data_diff = match (p_rec, f_rec) {
                    (Some(p), Some(f)) if p.size == f.size => {
                        let p_slice = &p_bytes[p.data_offset..p.data_offset + p.size];
                        let f_slice = &f_bytes[f.data_offset..f.data_offset + f.size];
                        task903_first_diff(p_slice, f_slice)
                    }
                    (Some(_), Some(_)) => "size differs".to_string(),
                    (Some(_), None) => "missing in failing".to_string(),
                    (None, Some(_)) => "missing in positive".to_string(),
                    (None, None) => "-".to_string(),
                };

                writeln!(
                    record_diff_report,
                    "| {} | {} | {}({}) | {} | {} | {}({}) | {} | {} | {} | {} | {} |",
                    idx,
                    status,
                    p_rec.map(|r| task903_record_tag_name(r.tag)).unwrap_or("-"),
                    p_rec
                        .map(|r| r.tag.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    p_rec
                        .map(|r| r.level.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    p_rec
                        .map(|r| r.size.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    f_rec.map(|r| task903_record_tag_name(r.tag)).unwrap_or("-"),
                    f_rec
                        .map(|r| r.tag.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    f_rec
                        .map(|r| r.level.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    f_rec
                        .map(|r| r.size.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    p_rec
                        .map(|r| r.offset.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    f_rec
                        .map(|r| r.offset.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    first_data_diff
                )
                .unwrap();
            }
            if diff_count > shown {
                writeln!(
                    record_diff_report,
                    "\n{} diff records total; first {} shown.",
                    diff_count, shown
                )
                .unwrap();
            } else {
                writeln!(record_diff_report, "\n{} diff records total.", diff_count).unwrap();
            }
        }
    }

    writeln!(report).unwrap();
    writeln!(report, "## 4. Record-level Diffs").unwrap();
    report.push_str(&record_diff_report);

    writeln!(report).unwrap();
    writeln!(report, "## 5. BinData Streams").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| stream | positive bytes | failing bytes | same bytes | positive hash | failing hash |"
    )
    .unwrap();
    writeln!(report, "|---|---:|---:|---|---|---|").unwrap();
    for stream in all_streams.iter().filter(|s| s.starts_with("/BinData/")) {
        let p_raw = task903_read_raw_stream(&positive, stream);
        let f_raw = task903_read_raw_stream(&failing, stream);
        writeln!(
            report,
            "| `{}` | {} | {} | {} | `{}` | `{}` |",
            stream,
            p_raw.as_ref().map(|v| v.len()).unwrap_or(0),
            f_raw.as_ref().map(|v| v.len()).unwrap_or(0),
            p_raw == f_raw,
            p_raw
                .as_ref()
                .map(|v| task903_short_hash(v))
                .unwrap_or_else(|| "-".to_string()),
            f_raw
                .as_ref()
                .map(|v| task903_short_hash(v))
                .unwrap_or_else(|| "-".to_string())
        )
        .unwrap();
    }

    writeln!(report).unwrap();
    writeln!(report, "## 6. Summary").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- positive streams: {}, failing streams: {}, union: {}",
        positive_streams.len(),
        failing_streams.len(),
        all_streams.len()
    )
    .unwrap();
    writeln!(
        report,
        "- raw differing streams: {}",
        if differing_streams.is_empty() {
            "(none)".to_string()
        } else {
            differing_streams.join(", ")
        }
    )
    .unwrap();
    writeln!(
        report,
        "- decompressed differing streams: {}",
        if decompressed_diffs.is_empty() {
            "(none)".to_string()
        } else {
            decompressed_diffs.join(", ")
        }
    )
    .unwrap();

    std::fs::write(&report_path, report).expect("Stage34 report 저장 실패");
    eprintln!("[#903 Stage34] wrote {}", report_path.display());
}

fn task903_stage35_failing_baseline_document() -> Document {
    task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/01_shape_common_attr_only.hwp",
    )
    .expect("Stage35 failing baseline 필요")
}

fn task903_stage35_mark_docinfo_dirty(doc: &mut Document) {
    doc.header.compressed = true;
    doc.header.flags |= 0x01;
    doc.header.raw_data = None;
    doc.doc_info.raw_stream_dirty = true;
}

fn task903_copy_bin_data_model_fields(doc: &mut Document, positive: &Document) {
    doc.doc_info.bin_data_list = positive.doc_info.bin_data_list.clone();
    for bin in &mut doc.doc_info.bin_data_list {
        bin.raw_data = None;
    }
    task903_stage35_mark_docinfo_dirty(doc);
}

fn task903_copy_bin_data_raw_records(doc: &mut Document, positive: &Document) {
    for (target, source) in doc
        .doc_info
        .bin_data_list
        .iter_mut()
        .zip(positive.doc_info.bin_data_list.iter())
    {
        target.raw_data = source.raw_data.clone();
    }
    task903_stage35_mark_docinfo_dirty(doc);
}

fn task903_copy_para_shape_model_fields(doc: &mut Document, positive: &Document) {
    doc.doc_info.para_shapes = positive.doc_info.para_shapes.clone();
    for para_shape in &mut doc.doc_info.para_shapes {
        para_shape.raw_data = None;
    }
    task903_stage35_mark_docinfo_dirty(doc);
}

fn task903_copy_para_shape_raw_records(doc: &mut Document, positive: &Document) {
    for (target, source) in doc
        .doc_info
        .para_shapes
        .iter_mut()
        .zip(positive.doc_info.para_shapes.iter())
    {
        target.raw_data = source.raw_data.clone();
    }
    task903_stage35_mark_docinfo_dirty(doc);
}

#[test]
fn task903_stage35_generate_docinfo_payload_probe_variants() {
    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let baseline = task903_stage35_failing_baseline_document();
    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe");
    std::fs::create_dir_all(out_dir).expect("Stage35 output dir 생성 실패");

    type PatchFn = fn(&mut Document, &Document);
    let variants: &[(&str, PatchFn)] = &[
        ("01_bin_data_model_fields_only", |doc, positive| {
            task903_copy_bin_data_model_fields(doc, positive);
        }),
        ("02_bin_data_raw_records_only", |doc, positive| {
            task903_copy_bin_data_raw_records(doc, positive);
        }),
        ("03_para_shape_model_fields_no_raw", |doc, positive| {
            task903_copy_para_shape_model_fields(doc, positive);
        }),
        ("04_para_shape_raw_records_only", |doc, positive| {
            task903_copy_para_shape_raw_records(doc, positive);
        }),
        (
            "05_bin_data_model_plus_para_shape_model",
            |doc, positive| {
                task903_copy_bin_data_model_fields(doc, positive);
                task903_copy_para_shape_model_fields(doc, positive);
            },
        ),
        ("06_bin_data_raw_plus_para_shape_raw", |doc, positive| {
            task903_copy_bin_data_raw_records(doc, positive);
            task903_copy_para_shape_raw_records(doc, positive);
        }),
    ];

    for (name, patch) in variants {
        let mut doc = baseline.clone();
        // Stage33 failing 파일은 CFB에는 BinData stream이 있지만, 잘못된 BIN_DATA metadata 때문에
        // rhwp 파싱 시 bin_data_content가 비어 있을 수 있다. Stage34에서 실제 BinData stream
        // 바이트는 positive와 동일함을 확인했으므로 판정 가능한 HWP 생성을 위해 content를 보강한다.
        doc.bin_data_content = positive.bin_data_content.clone();
        patch(&mut doc, &positive);
        let hwp_bytes = rhwp::serializer::serialize_hwp(&doc).expect("Stage35 HWP 직렬화 실패");
        assert!(
            hwp_bytes.len() > 300_000,
            "{} 산출물이 너무 작음: {} bytes",
            name,
            hwp_bytes.len()
        );

        let out_path = out_dir.join(format!("{}.hwp", name));
        std::fs::write(&out_path, &hwp_bytes).expect("Stage35 HWP 저장 실패");

        let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("Stage35 HWP rhwp 재로드 실패");
        assert_eq!(
            reloaded.page_count(),
            9,
            "{}는 rhwp 재로드 기준 9페이지여야 함",
            name
        );
        eprintln!(
            "[#903 Stage35] generated {} bytes={} pages={}",
            out_path.display(),
            hwp_bytes.len(),
            reloaded.page_count()
        );
    }
}

fn task903_stage36_baseline_document() -> Document {
    task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage35_docinfo_payload_probe/05_bin_data_model_plus_para_shape_model.hwp",
    )
    .expect("Stage36 baseline 필요")
}

fn task903_stage36_prepare_for_raw_bodytext(doc: &mut Document) {
    doc.header.compressed = true;
    doc.header.flags |= 0x01;
    doc.header.raw_data = None;
}

fn task903_patch_bodytext_tags(doc: &mut Document, positive: &Document, tags: &[u32]) {
    for section_idx in 0..doc.sections.len().min(positive.sections.len()) {
        let target_raw = doc.sections[section_idx]
            .raw_stream
            .as_ref()
            .unwrap_or_else(|| panic!("target section{} raw_stream 필요", section_idx));
        let positive_raw = positive.sections[section_idx]
            .raw_stream
            .as_ref()
            .unwrap_or_else(|| panic!("positive section{} raw_stream 필요", section_idx));
        let patched = task903_graft_record_tags_by_index(target_raw, positive_raw, tags);
        doc.sections[section_idx].raw_stream = Some(patched);
    }
    task903_stage36_prepare_for_raw_bodytext(doc);
}

fn task903_copy_section_raw(doc: &mut Document, positive: &Document, section_idx: usize) {
    let raw = positive.sections[section_idx]
        .raw_stream
        .as_ref()
        .unwrap_or_else(|| panic!("positive section{} raw_stream 필요", section_idx))
        .clone();
    doc.sections[section_idx].raw_stream = Some(raw);
    task903_stage36_prepare_for_raw_bodytext(doc);
}

#[test]
fn task903_stage36_generate_bodytext_payload_probe_variants() {
    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let baseline = task903_stage36_baseline_document();
    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe");
    std::fs::create_dir_all(out_dir).expect("Stage36 output dir 생성 실패");

    type PatchFn = fn(&mut Document, &Document);
    let variants: &[(&str, PatchFn)] = &[
        ("01_ctrl_header_records", |doc, positive| {
            task903_patch_bodytext_tags(doc, positive, &[71]);
        }),
        ("02_list_header_records", |doc, positive| {
            task903_patch_bodytext_tags(doc, positive, &[72]);
        }),
        ("03_para_header_records", |doc, positive| {
            task903_patch_bodytext_tags(doc, positive, &[66]);
        }),
        ("04_ctrl_plus_list_headers", |doc, positive| {
            task903_patch_bodytext_tags(doc, positive, &[71, 72]);
        }),
        ("05_ctrl_list_para_headers", |doc, positive| {
            task903_patch_bodytext_tags(doc, positive, &[71, 72, 66]);
        }),
        ("06_section0_raw_only", |doc, positive| {
            task903_copy_section_raw(doc, positive, 0);
        }),
        ("07_section0_section1_raw", |doc, positive| {
            task903_copy_section_raw(doc, positive, 0);
            task903_copy_section_raw(doc, positive, 1);
        }),
    ];

    for (name, patch) in variants {
        let mut doc = baseline.clone();
        patch(&mut doc, &positive);
        let hwp_bytes = rhwp::serializer::serialize_hwp(&doc).expect("Stage36 HWP 직렬화 실패");
        assert!(
            hwp_bytes.len() > 300_000,
            "{} 산출물이 너무 작음: {} bytes",
            name,
            hwp_bytes.len()
        );

        let out_path = out_dir.join(format!("{}.hwp", name));
        std::fs::write(&out_path, &hwp_bytes).expect("Stage36 HWP 저장 실패");

        let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("Stage36 HWP rhwp 재로드 실패");
        assert_eq!(
            reloaded.page_count(),
            9,
            "{}는 rhwp 재로드 기준 9페이지여야 함",
            name
        );
        eprintln!(
            "[#903 Stage36] generated {} bytes={} pages={}",
            out_path.display(),
            hwp_bytes.len(),
            reloaded.page_count()
        );
    }
}

fn task903_stage37_failing_document() -> Document {
    task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/05_ctrl_list_para_headers.hwp",
    )
    .expect("Stage37 failing baseline 필요")
}

fn task903_stage37_success_document() -> Document {
    task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage36_bodytext_payload_probe/06_section0_raw_only.hwp",
    )
    .expect("Stage37 success baseline 필요")
}

fn task903_patch_section0_tags(doc: &mut Document, positive: &Document, tags: &[u32]) {
    let target_raw = doc.sections[0]
        .raw_stream
        .as_ref()
        .expect("target section0 raw_stream 필요");
    let positive_raw = positive.sections[0]
        .raw_stream
        .as_ref()
        .expect("positive section0 raw_stream 필요");
    let patched = task903_graft_record_tags_by_index(target_raw, positive_raw, tags);
    doc.sections[0].raw_stream = Some(patched);
    task903_stage36_prepare_for_raw_bodytext(doc);
}

fn task903_stage37_write_section0_diff_report(failing: &Document, success: &Document) {
    use std::collections::BTreeMap;
    use std::fmt::Write as _;

    let failing_raw = failing.sections[0]
        .raw_stream
        .as_ref()
        .expect("failing section0 raw_stream 필요");
    let success_raw = success.sections[0]
        .raw_stream
        .as_ref()
        .expect("success section0 raw_stream 필요");
    let failing_records = task903_parse_records(failing_raw);
    let success_records = task903_parse_records(success_raw);
    assert_eq!(failing_records.len(), success_records.len());

    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe");
    std::fs::create_dir_all(out_dir).expect("Stage37 output dir 생성 실패");
    let report_path = out_dir.join("stage37_section0_diff.md");

    let mut counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut examples = String::new();
    let mut shown = 0usize;
    let mut total = 0usize;
    for (idx, (f_rec, s_rec)) in failing_records
        .iter()
        .zip(success_records.iter())
        .enumerate()
    {
        assert_eq!(f_rec.tag, s_rec.tag, "tag mismatch at record {}", idx);
        assert_eq!(f_rec.level, s_rec.level, "level mismatch at record {}", idx);

        let f_record_bytes = &failing_raw[f_rec.offset..task903_record_end(f_rec)];
        let s_record_bytes = &success_raw[s_rec.offset..task903_record_end(s_rec)];
        if f_record_bytes == s_record_bytes {
            continue;
        }
        total += 1;
        *counts.entry(f_rec.tag).or_insert(0) += 1;
        if shown < 80 {
            shown += 1;
            let status = if f_rec.size == s_rec.size {
                "DATA_DIFF"
            } else {
                "SIZE_DIFF"
            };
            let first_diff = if f_rec.size == s_rec.size {
                task903_first_diff(
                    &failing_raw[f_rec.data_offset..f_rec.data_offset + f_rec.size],
                    &success_raw[s_rec.data_offset..s_rec.data_offset + s_rec.size],
                )
            } else {
                "size differs".to_string()
            };
            writeln!(
                examples,
                "| {} | {} | {}({}) | {} | {} | {} | {} | {} |",
                idx,
                status,
                task903_record_tag_name(f_rec.tag),
                f_rec.tag,
                f_rec.level,
                f_rec.size,
                s_rec.size,
                f_rec.offset,
                first_diff
            )
            .unwrap();
        }
    }

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 37 Section0 Diff").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. Summary").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "- failing section0 bytes: {}", failing_raw.len()).unwrap();
    writeln!(report, "- success section0 bytes: {}", success_raw.len()).unwrap();
    writeln!(report, "- record count: {}", failing_records.len()).unwrap();
    writeln!(report, "- differing records: {}", total).unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 2. Diff Count By Tag").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| tag | name | diff records |").unwrap();
    writeln!(report, "|---:|---|---:|").unwrap();
    for (tag, count) in counts {
        writeln!(
            report,
            "| {} | {} | {} |",
            tag,
            task903_record_tag_name(tag),
            count
        )
        .unwrap();
    }
    writeln!(report).unwrap();
    writeln!(report, "## 3. First Diff Records").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | status | tag | level | failing size | success size | failing offset | first data diff |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---|---:|---:|---:|---:|---|").unwrap();
    report.push_str(&examples);
    if total > shown {
        writeln!(report, "\n{} total; first {} shown.", total, shown).unwrap();
    }

    std::fs::write(&report_path, report).expect("Stage37 diff report 저장 실패");
    eprintln!("[#903 Stage37] wrote {}", report_path.display());
}

#[test]
fn task903_stage37_generate_section0_record_type_probe_variants() {
    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let baseline = task903_stage37_failing_document();
    let success = task903_stage37_success_document();
    task903_stage37_write_section0_diff_report(&baseline, &success);

    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage37_section0_record_type_probe");
    std::fs::create_dir_all(out_dir).expect("Stage37 output dir 생성 실패");

    type PatchFn = fn(&mut Document, &Document);
    let variants: &[(&str, PatchFn)] = &[
        ("01_shape_component_records", |doc, positive| {
            task903_patch_section0_tags(doc, positive, &[76]);
        }),
        ("02_shape_picture_records", |doc, positive| {
            task903_patch_section0_tags(doc, positive, &[85]);
        }),
        ("03_table_records", |doc, positive| {
            task903_patch_section0_tags(doc, positive, &[77]);
        }),
        ("04_para_line_seg_records", |doc, positive| {
            task903_patch_section0_tags(doc, positive, &[69]);
        }),
        ("05_page_records", |doc, positive| {
            task903_patch_section0_tags(doc, positive, &[73, 75, 74]);
        }),
        ("06_shape_component_picture", |doc, positive| {
            task903_patch_section0_tags(doc, positive, &[76, 85]);
        }),
        ("07_shape_picture_table", |doc, positive| {
            task903_patch_section0_tags(doc, positive, &[76, 85, 77]);
        }),
        ("08_all_remaining_candidate_types", |doc, positive| {
            task903_patch_section0_tags(doc, positive, &[76, 85, 77, 69, 73, 75, 74]);
        }),
    ];

    for (name, patch) in variants {
        let mut doc = baseline.clone();
        patch(&mut doc, &positive);
        let hwp_bytes = rhwp::serializer::serialize_hwp(&doc).expect("Stage37 HWP 직렬화 실패");
        assert!(
            hwp_bytes.len() > 300_000,
            "{} 산출물이 너무 작음: {} bytes",
            name,
            hwp_bytes.len()
        );

        let out_path = out_dir.join(format!("{}.hwp", name));
        std::fs::write(&out_path, &hwp_bytes).expect("Stage37 HWP 저장 실패");

        let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("Stage37 HWP rhwp 재로드 실패");
        assert_eq!(
            reloaded.page_count(),
            9,
            "{}는 rhwp 재로드 기준 9페이지여야 함",
            name
        );
        eprintln!(
            "[#903 Stage37] generated {} bytes={} pages={}",
            out_path.display(),
            hwp_bytes.len(),
            reloaded.page_count()
        );
    }
}

fn task903_patch_section0_indices(doc: &mut Document, positive: &Document, indices: &[usize]) {
    let target_raw = doc.sections[0]
        .raw_stream
        .as_ref()
        .expect("target section0 raw_stream 필요");
    let positive_raw = positive.sections[0]
        .raw_stream
        .as_ref()
        .expect("positive section0 raw_stream 필요");
    let patched = task903_graft_record_indices(target_raw, positive_raw, indices);
    doc.sections[0].raw_stream = Some(patched);
    task903_stage36_prepare_for_raw_bodytext(doc);
}

fn task903_stage38_write_index_map_report(baseline: &Document, positive: &Document) {
    use std::fmt::Write as _;

    let target_raw = baseline.sections[0]
        .raw_stream
        .as_ref()
        .expect("baseline section0 raw_stream 필요");
    let positive_raw = positive.sections[0]
        .raw_stream
        .as_ref()
        .expect("positive section0 raw_stream 필요");
    let target_records = task903_parse_records(target_raw);
    let positive_records = task903_parse_records(positive_raw);
    assert_eq!(target_records.len(), positive_records.len());

    let interesting_indices: &[usize] = &[
        21, 22, 35, 36, 48, 103, 286, 433, 563, 742, 807, 808, 809, 810, 811, 812, 813, 819, 1619,
        2944, 6466, 6596, 6986, 7376,
    ];

    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage38_record_index_probe");
    std::fs::create_dir_all(out_dir).expect("Stage38 output dir 생성 실패");
    let report_path = out_dir.join("stage38_record_index_map.md");

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 38 Record Index Map").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | tag | level | target size | positive size | target offset | first data diff |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---:|---:|---:|---:|---|").unwrap();

    for &idx in interesting_indices {
        let target_record = &target_records[idx];
        let positive_record = &positive_records[idx];
        assert_eq!(target_record.tag, positive_record.tag);
        assert_eq!(target_record.level, positive_record.level);
        let first_diff = if target_record.size == positive_record.size {
            task903_first_diff(
                &target_raw
                    [target_record.data_offset..target_record.data_offset + target_record.size],
                &positive_raw[positive_record.data_offset
                    ..positive_record.data_offset + positive_record.size],
            )
        } else {
            "size differs".to_string()
        };
        writeln!(
            report,
            "| {} | {}({}) | {} | {} | {} | {} | {} |",
            idx,
            task903_record_tag_name(target_record.tag),
            target_record.tag,
            target_record.level,
            target_record.size,
            positive_record.size,
            target_record.offset,
            first_diff
        )
        .unwrap();
    }

    std::fs::write(&report_path, report).expect("Stage38 index map 저장 실패");
    eprintln!("[#903 Stage38] wrote {}", report_path.display());
}

#[test]
fn task903_stage38_generate_record_index_probe_variants() {
    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let baseline = task903_stage37_failing_document();
    task903_stage38_write_index_map_report(&baseline, &positive);

    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage38_record_index_probe");
    std::fs::create_dir_all(out_dir).expect("Stage38 output dir 생성 실패");

    const TABLE_FIRST: &[usize] = &[48];
    const TABLE_EARLY: &[usize] = &[48, 103, 286, 433, 563, 742, 819];
    const TABLE_LATE: &[usize] = &[1619, 2944, 6466, 6596, 6986, 7376];
    const TABLE_ALL: &[usize] = &[
        48, 103, 286, 433, 563, 742, 819, 1619, 2944, 6466, 6596, 6986, 7376,
    ];
    const FIRST_PAGE_PICTURES: &[usize] = &[21, 22, 35, 36];
    const GROUP_PICTURES: &[usize] = &[807, 808, 809, 810, 811, 812, 813];
    const SHAPE_ALL: &[usize] = &[21, 22, 35, 36, 807, 808, 809, 810, 811, 812, 813];

    let mut first_picture_table_all = Vec::new();
    first_picture_table_all.extend_from_slice(FIRST_PAGE_PICTURES);
    first_picture_table_all.extend_from_slice(TABLE_ALL);

    let mut group_picture_table_all = Vec::new();
    group_picture_table_all.extend_from_slice(GROUP_PICTURES);
    group_picture_table_all.extend_from_slice(TABLE_ALL);

    let mut shape_all_table_early = Vec::new();
    shape_all_table_early.extend_from_slice(SHAPE_ALL);
    shape_all_table_early.extend_from_slice(TABLE_EARLY);

    let mut shape_all_table_late = Vec::new();
    shape_all_table_late.extend_from_slice(SHAPE_ALL);
    shape_all_table_late.extend_from_slice(TABLE_LATE);

    let mut shape_all_table_all = Vec::new();
    shape_all_table_all.extend_from_slice(SHAPE_ALL);
    shape_all_table_all.extend_from_slice(TABLE_ALL);

    let variants: Vec<(&str, Vec<usize>)> = vec![
        ("01_table_first_record_only", TABLE_FIRST.to_vec()),
        ("02_table_early_size_diffs", TABLE_EARLY.to_vec()),
        ("03_table_late_size_diffs", TABLE_LATE.to_vec()),
        ("04_table_all_indices", TABLE_ALL.to_vec()),
        ("05_first_page_picture_pair", FIRST_PAGE_PICTURES.to_vec()),
        ("06_group_picture_cluster", GROUP_PICTURES.to_vec()),
        ("07_shape_all_indices", SHAPE_ALL.to_vec()),
        ("08_first_picture_plus_table_all", first_picture_table_all),
        ("09_group_picture_plus_table_all", group_picture_table_all),
        ("10_shape_all_plus_table_early", shape_all_table_early),
        ("11_shape_all_plus_table_late", shape_all_table_late),
        ("12_shape_all_plus_table_all", shape_all_table_all),
    ];

    for (name, indices) in variants {
        let mut doc = baseline.clone();
        task903_patch_section0_indices(&mut doc, &positive, &indices);
        let hwp_bytes = rhwp::serializer::serialize_hwp(&doc).expect("Stage38 HWP 직렬화 실패");
        assert!(
            hwp_bytes.len() > 300_000,
            "{} 산출물이 너무 작음: {} bytes",
            name,
            hwp_bytes.len()
        );

        let out_path = out_dir.join(format!("{}.hwp", name));
        std::fs::write(&out_path, &hwp_bytes).expect("Stage38 HWP 저장 실패");

        let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("Stage38 HWP rhwp 재로드 실패");
        assert_eq!(
            reloaded.page_count(),
            9,
            "{}는 rhwp 재로드 기준 9페이지여야 함",
            name
        );
        eprintln!(
            "[#903 Stage38] generated {} bytes={} pages={} indices={:?}",
            out_path.display(),
            hwp_bytes.len(),
            reloaded.page_count(),
            indices
        );
    }
}

fn task903_stage39_indices_with_shape(table_indices: &[usize]) -> Vec<usize> {
    let mut indices = vec![21, 22, 35, 36, 807, 808, 809, 810, 811, 812, 813];
    indices.extend_from_slice(table_indices);
    indices
}

fn task903_stage39_table_all_except(excluded: &[usize]) -> Vec<usize> {
    [
        48usize, 103, 286, 433, 563, 742, 819, 1619, 2944, 6466, 6596, 6986, 7376,
    ]
    .into_iter()
    .filter(|idx| !excluded.contains(idx))
    .collect()
}

#[test]
fn task903_stage39_generate_table_index_min_probe_variants() {
    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let baseline = task903_stage37_failing_document();
    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage39_table_index_min_probe");
    std::fs::create_dir_all(out_dir).expect("Stage39 output dir 생성 실패");

    const TABLE_1: &[usize] = &[48];
    const TABLE_3: &[usize] = &[48, 103, 286];
    const TABLE_5: &[usize] = &[48, 103, 286, 433, 563];
    const TABLE_7: &[usize] = &[48, 103, 286, 433, 563, 742, 819];
    const TABLE_9: &[usize] = &[48, 103, 286, 433, 563, 742, 819, 1619, 2944];
    const TABLE_11: &[usize] = &[48, 103, 286, 433, 563, 742, 819, 1619, 2944, 6466, 6596];
    const TABLE_12: &[usize] = &[
        48, 103, 286, 433, 563, 742, 819, 1619, 2944, 6466, 6596, 6986,
    ];
    const TABLE_13: &[usize] = &[
        48, 103, 286, 433, 563, 742, 819, 1619, 2944, 6466, 6596, 6986, 7376,
    ];
    const TABLE_EARLY: &[usize] = &[48, 103, 286, 433, 563, 742, 819];
    const TABLE_LATE: &[usize] = &[1619, 2944, 6466, 6596, 6986, 7376];

    let variants: Vec<(&str, Vec<usize>)> = vec![
        (
            "01_shape_all_table_1",
            task903_stage39_indices_with_shape(TABLE_1),
        ),
        (
            "02_shape_all_table_3",
            task903_stage39_indices_with_shape(TABLE_3),
        ),
        (
            "03_shape_all_table_5",
            task903_stage39_indices_with_shape(TABLE_5),
        ),
        (
            "04_shape_all_table_7",
            task903_stage39_indices_with_shape(TABLE_7),
        ),
        (
            "05_shape_all_table_9",
            task903_stage39_indices_with_shape(TABLE_9),
        ),
        (
            "06_shape_all_table_11",
            task903_stage39_indices_with_shape(TABLE_11),
        ),
        (
            "07_shape_all_table_12",
            task903_stage39_indices_with_shape(TABLE_12),
        ),
        (
            "08_shape_all_table_13",
            task903_stage39_indices_with_shape(TABLE_13),
        ),
        (
            "09_all_except_48",
            task903_stage39_indices_with_shape(&task903_stage39_table_all_except(&[48])),
        ),
        (
            "10_all_except_819",
            task903_stage39_indices_with_shape(&task903_stage39_table_all_except(&[819])),
        ),
        (
            "11_all_except_1619",
            task903_stage39_indices_with_shape(&task903_stage39_table_all_except(&[1619])),
        ),
        (
            "12_all_except_7376",
            task903_stage39_indices_with_shape(&task903_stage39_table_all_except(&[7376])),
        ),
        (
            "13_all_except_late",
            task903_stage39_indices_with_shape(&task903_stage39_table_all_except(TABLE_LATE)),
        ),
        (
            "14_all_except_early",
            task903_stage39_indices_with_shape(&task903_stage39_table_all_except(TABLE_EARLY)),
        ),
    ];

    for (name, indices) in variants {
        let mut doc = baseline.clone();
        task903_patch_section0_indices(&mut doc, &positive, &indices);
        let hwp_bytes = rhwp::serializer::serialize_hwp(&doc).expect("Stage39 HWP 직렬화 실패");
        assert!(
            hwp_bytes.len() > 300_000,
            "{} 산출물이 너무 작음: {} bytes",
            name,
            hwp_bytes.len()
        );

        let out_path = out_dir.join(format!("{}.hwp", name));
        std::fs::write(&out_path, &hwp_bytes).expect("Stage39 HWP 저장 실패");

        let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("Stage39 HWP rhwp 재로드 실패");
        assert_eq!(
            reloaded.page_count(),
            9,
            "{}는 rhwp 재로드 기준 9페이지여야 함",
            name
        );
        eprintln!(
            "[#903 Stage39] generated {} bytes={} pages={} indices={:?}",
            out_path.display(),
            hwp_bytes.len(),
            reloaded.page_count(),
            indices
        );
    }
}

fn task903_stage40_without(base: &[usize], excluded: usize) -> Vec<usize> {
    base.iter()
        .copied()
        .filter(|idx| *idx != excluded)
        .collect()
}

#[test]
fn task903_stage40_generate_table_min_leave_one_out_variants() {
    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let baseline = task903_stage37_failing_document();
    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out");
    std::fs::create_dir_all(out_dir).expect("Stage40 output dir 생성 실패");

    const BASE_TABLE_11: &[usize] = &[48, 103, 286, 433, 563, 742, 819, 1619, 2944, 6466, 6596];
    const BASE_WITHOUT_819: &[usize] = &[48, 103, 286, 433, 563, 742, 1619, 2944, 6466, 6596];
    const BASE_WITHOUT_819_PLUS_6986: &[usize] =
        &[48, 103, 286, 433, 563, 742, 1619, 2944, 6466, 6596, 6986];
    const BASE_WITHOUT_819_PLUS_7376: &[usize] =
        &[48, 103, 286, 433, 563, 742, 1619, 2944, 6466, 6596, 7376];
    const BASE_WITHOUT_819_PLUS_6986_7376: &[usize] = &[
        48, 103, 286, 433, 563, 742, 1619, 2944, 6466, 6596, 6986, 7376,
    ];

    let variants: Vec<(&str, Vec<usize>)> = vec![
        (
            "01_base_table_11",
            task903_stage39_indices_with_shape(BASE_TABLE_11),
        ),
        (
            "02_base_without_819",
            task903_stage39_indices_with_shape(BASE_WITHOUT_819),
        ),
        (
            "03_without_48",
            task903_stage39_indices_with_shape(&task903_stage40_without(BASE_WITHOUT_819, 48)),
        ),
        (
            "04_without_103",
            task903_stage39_indices_with_shape(&task903_stage40_without(BASE_WITHOUT_819, 103)),
        ),
        (
            "05_without_286",
            task903_stage39_indices_with_shape(&task903_stage40_without(BASE_WITHOUT_819, 286)),
        ),
        (
            "06_without_433",
            task903_stage39_indices_with_shape(&task903_stage40_without(BASE_WITHOUT_819, 433)),
        ),
        (
            "07_without_563",
            task903_stage39_indices_with_shape(&task903_stage40_without(BASE_WITHOUT_819, 563)),
        ),
        (
            "08_without_742",
            task903_stage39_indices_with_shape(&task903_stage40_without(BASE_WITHOUT_819, 742)),
        ),
        (
            "09_without_1619",
            task903_stage39_indices_with_shape(&task903_stage40_without(BASE_WITHOUT_819, 1619)),
        ),
        (
            "10_without_2944",
            task903_stage39_indices_with_shape(&task903_stage40_without(BASE_WITHOUT_819, 2944)),
        ),
        (
            "11_without_6466",
            task903_stage39_indices_with_shape(&task903_stage40_without(BASE_WITHOUT_819, 6466)),
        ),
        (
            "12_without_6596",
            task903_stage39_indices_with_shape(&task903_stage40_without(BASE_WITHOUT_819, 6596)),
        ),
        (
            "13_base_without_819_plus_6986",
            task903_stage39_indices_with_shape(BASE_WITHOUT_819_PLUS_6986),
        ),
        (
            "14_base_without_819_plus_7376",
            task903_stage39_indices_with_shape(BASE_WITHOUT_819_PLUS_7376),
        ),
        (
            "15_base_without_819_plus_6986_7376",
            task903_stage39_indices_with_shape(BASE_WITHOUT_819_PLUS_6986_7376),
        ),
    ];

    for (name, indices) in variants {
        let mut doc = baseline.clone();
        task903_patch_section0_indices(&mut doc, &positive, &indices);
        let hwp_bytes = rhwp::serializer::serialize_hwp(&doc).expect("Stage40 HWP 직렬화 실패");
        assert!(
            hwp_bytes.len() > 300_000,
            "{} 산출물이 너무 작음: {} bytes",
            name,
            hwp_bytes.len()
        );

        let out_path = out_dir.join(format!("{}.hwp", name));
        std::fs::write(&out_path, &hwp_bytes).expect("Stage40 HWP 저장 실패");

        let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("Stage40 HWP rhwp 재로드 실패");
        assert_eq!(
            reloaded.page_count(),
            9,
            "{}는 rhwp 재로드 기준 9페이지여야 함",
            name
        );
        eprintln!(
            "[#903 Stage40] generated {} bytes={} pages={} indices={:?}",
            out_path.display(),
            hwp_bytes.len(),
            reloaded.page_count(),
            indices
        );
    }
}

fn task903_hex_bytes(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn task903_read_u16_at(data: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes([
        *data.get(offset)?,
        *data.get(offset + 1)?,
    ]))
}

fn task903_read_i16_at(data: &[u8], offset: usize) -> Option<i16> {
    Some(i16::from_le_bytes([
        *data.get(offset)?,
        *data.get(offset + 1)?,
    ]))
}

fn task903_read_u32_at(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *data.get(offset)?,
        *data.get(offset + 1)?,
        *data.get(offset + 2)?,
        *data.get(offset + 3)?,
    ]))
}

fn task903_decode_table_payload(data: &[u8]) -> String {
    let attr = task903_read_u32_at(data, 0).unwrap_or(0);
    let row_count = task903_read_u16_at(data, 4).unwrap_or(0);
    let col_count = task903_read_u16_at(data, 6).unwrap_or(0);
    let cell_spacing = task903_read_i16_at(data, 8).unwrap_or(0);
    let padding = [
        task903_read_i16_at(data, 10).unwrap_or(0),
        task903_read_i16_at(data, 12).unwrap_or(0),
        task903_read_i16_at(data, 14).unwrap_or(0),
        task903_read_i16_at(data, 16).unwrap_or(0),
    ];
    let mut pos = 18usize;
    let mut row_sizes = Vec::new();
    for _ in 0..row_count {
        if let Some(value) = task903_read_i16_at(data, pos) {
            row_sizes.push(value);
        }
        pos += 2;
    }
    let border_fill = task903_read_u16_at(data, pos).unwrap_or(0);
    pos += 2;
    let n_zones = task903_read_u16_at(data, pos).unwrap_or(0);
    let zone_bytes = if data.len() >= pos + 2 {
        let expected = 2usize.saturating_add(n_zones as usize * 10);
        expected.min(data.len().saturating_sub(pos))
    } else {
        0
    };
    let tail_start = pos.saturating_add(zone_bytes);
    let tail = if tail_start < data.len() {
        task903_hex_bytes(&data[tail_start..])
    } else {
        String::new()
    };

    format!(
        "attr={:#010x}, rows={}, cols={}, spacing={}, padding={:?}, row_sizes={:?}, border_fill={}, n_zones={}, zone_bytes={}, raw_tail={}",
        attr,
        row_count,
        col_count,
        cell_spacing,
        padding,
        row_sizes,
        border_fill,
        n_zones,
        zone_bytes,
        if tail.is_empty() { "-".to_string() } else { tail }
    )
}

fn task903_stage41_write_table_payload_diff_report() {
    use std::fmt::Write as _;

    let failing = task903_stage37_failing_document();
    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let target_raw = failing.sections[0]
        .raw_stream
        .as_ref()
        .expect("failing section0 raw_stream 필요");
    let positive_raw = positive.sections[0]
        .raw_stream
        .as_ref()
        .expect("positive section0 raw_stream 필요");
    let target_records = task903_parse_records(target_raw);
    let positive_records = task903_parse_records(positive_raw);
    assert_eq!(target_records.len(), positive_records.len());

    let required: &[usize] = &[48, 103, 286, 433, 563, 742, 1619, 2944, 6466];
    let optional: &[usize] = &[819, 6596, 6986, 7376];
    let all_indices: Vec<(usize, &str)> = required
        .iter()
        .copied()
        .map(|idx| (idx, "required"))
        .chain(optional.iter().copied().map(|idx| (idx, "optional")))
        .collect();

    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage41_table_payload_diff");
    std::fs::create_dir_all(out_dir).expect("Stage41 output dir 생성 실패");
    let report_path = out_dir.join("table_payload_diff.md");

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 41 TABLE Payload Diff").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. Summary").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "- required TABLE indices: {:?}", required).unwrap();
    writeln!(report, "- optional TABLE indices: {:?}", optional).unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 2. Payload Diff Table").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | class | level | target size | positive size | target is positive prefix | positive extra tail | first diff | target decoded | positive decoded |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---:|---:|---:|---|---|---|---|---|").unwrap();

    for (idx, class) in all_indices {
        let target_record = &target_records[idx];
        let positive_record = &positive_records[idx];
        assert_eq!(target_record.tag, positive_record.tag);
        assert_eq!(target_record.tag, 77, "idx {}는 TABLE이어야 함", idx);
        assert_eq!(target_record.level, positive_record.level);

        let target_payload =
            &target_raw[target_record.data_offset..target_record.data_offset + target_record.size];
        let positive_payload = &positive_raw
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        let target_is_prefix = positive_payload.starts_with(target_payload);
        let positive_extra_tail =
            if target_is_prefix && positive_payload.len() > target_payload.len() {
                task903_hex_bytes(&positive_payload[target_payload.len()..])
            } else {
                "-".to_string()
            };
        let first_diff = task903_first_diff(target_payload, positive_payload);

        writeln!(
            report,
            "| {} | {} | {} | {} | {} | {} | `{}` | {} | `{}` | `{}` |",
            idx,
            class,
            target_record.level,
            target_record.size,
            positive_record.size,
            target_is_prefix,
            positive_extra_tail,
            first_diff,
            task903_decode_table_payload(target_payload),
            task903_decode_table_payload(positive_payload)
        )
        .unwrap();
    }

    writeln!(report).unwrap();
    writeln!(report, "## 3. Raw Payload Hex").unwrap();
    writeln!(report).unwrap();
    for (idx, class) in required
        .iter()
        .copied()
        .map(|idx| (idx, "required"))
        .chain(optional.iter().copied().map(|idx| (idx, "optional")))
    {
        let target_record = &target_records[idx];
        let positive_record = &positive_records[idx];
        let target_payload =
            &target_raw[target_record.data_offset..target_record.data_offset + target_record.size];
        let positive_payload = &positive_raw
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        writeln!(report, "### idx {} ({})", idx, class).unwrap();
        writeln!(report).unwrap();
        writeln!(report, "- target: `{}`", task903_hex_bytes(target_payload)).unwrap();
        writeln!(
            report,
            "- positive: `{}`",
            task903_hex_bytes(positive_payload)
        )
        .unwrap();
        writeln!(report).unwrap();
    }

    std::fs::write(&report_path, report).expect("Stage41 report 저장 실패");
    eprintln!("[#903 Stage41] wrote {}", report_path.display());
}

#[test]
#[ignore]
fn task903_stage41_generate_table_payload_diff_report() {
    task903_stage41_write_table_payload_diff_report();
}

fn task903_table_payload_row_sizes(data: &[u8]) -> Vec<i16> {
    let row_count = task903_read_u16_at(data, 4).unwrap_or(0);
    let mut pos = 18usize;
    let mut row_sizes = Vec::new();
    for _ in 0..row_count {
        if let Some(value) = task903_read_i16_at(data, pos) {
            row_sizes.push(value);
        }
        pos += 2;
    }
    row_sizes
}

#[test]
fn task903_stage42_generate_table_rowsizes_probe() {
    use std::fmt::Write as _;

    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let report = convert_hwpx_to_hwp_ir(core.document_mut());
    eprintln!("[#903 Stage42] adapter report: {:?}", report);

    let hwp_bytes =
        rhwp::serializer::serialize_hwp(core.document()).expect("Stage42 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "Stage42 산출물이 너무 작음: {} bytes",
        hwp_bytes.len()
    );

    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter");
    std::fs::create_dir_all(out_dir).expect("Stage42 output dir 생성 실패");
    let out_path = out_dir.join("hwpx-h-01.hwp");
    std::fs::write(&out_path, &hwp_bytes).expect("Stage42 HWP 저장 실패");

    let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("Stage42 HWP rhwp 재로드 실패");
    assert_eq!(
        reloaded.page_count(),
        9,
        "Stage42 산출물은 rhwp 기준 9페이지여야 함"
    );

    let compressed = task903_file_header_flags(&hwp_bytes) & 0x01 != 0;
    let output_section0 = task903_read_decompressed_section(&hwp_bytes, 0, compressed)
        .expect("Stage42 output section0 읽기 실패");
    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let positive_section0 = positive.sections[0]
        .raw_stream
        .as_ref()
        .expect("positive section0 raw_stream 필요");

    let output_records = task903_parse_records(&output_section0);
    let positive_records = task903_parse_records(positive_section0);
    assert_eq!(
        output_records.len(),
        positive_records.len(),
        "Stage42 record count mismatch"
    );

    let table_min: &[usize] = &[48, 103, 286, 433, 563, 742, 1619, 2944, 6466];
    let mut report_md = String::new();
    writeln!(report_md, "# Task m100 #903 Stage 42 TABLE RowSizes").unwrap();
    writeln!(report_md).unwrap();
    writeln!(report_md, "- output: `{}`", out_path.display()).unwrap();
    writeln!(report_md, "- bytes: {}", hwp_bytes.len()).unwrap();
    writeln!(report_md, "- rhwp reload pages: {}", reloaded.page_count()).unwrap();
    writeln!(
        report_md,
        "- adapter table_record_row_sizes_materialized: {}",
        report.table_record_row_sizes_materialized
    )
    .unwrap();
    writeln!(report_md).unwrap();
    writeln!(
        report_md,
        "| idx | output size | positive size | output row_sizes | positive row_sizes | row_sizes match | output decoded | positive decoded |"
    )
    .unwrap();
    writeln!(report_md, "|---:|---:|---:|---|---|---|---|---|").unwrap();

    for &idx in table_min {
        let output_record = &output_records[idx];
        let positive_record = &positive_records[idx];
        assert_eq!(output_record.tag, 77, "idx {}는 TABLE이어야 함", idx);
        assert_eq!(output_record.tag, positive_record.tag);
        assert_eq!(output_record.level, positive_record.level);

        let output_payload = &output_section0
            [output_record.data_offset..output_record.data_offset + output_record.size];
        let positive_payload = &positive_section0
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        let output_row_sizes = task903_table_payload_row_sizes(output_payload);
        let positive_row_sizes = task903_table_payload_row_sizes(positive_payload);
        let row_sizes_match = output_row_sizes == positive_row_sizes;
        assert!(
            row_sizes_match,
            "idx {} row_sizes mismatch: output={:?} positive={:?}",
            idx, output_row_sizes, positive_row_sizes
        );

        writeln!(
            report_md,
            "| {} | {} | {} | `{:?}` | `{:?}` | {} | `{}` | `{}` |",
            idx,
            output_record.size,
            positive_record.size,
            output_row_sizes,
            positive_row_sizes,
            row_sizes_match,
            task903_decode_table_payload(output_payload),
            task903_decode_table_payload(positive_payload)
        )
        .unwrap();
    }

    let report_path = out_dir.join("table_payload_after_rowsizes.md");
    std::fs::write(&report_path, report_md).expect("Stage42 report 저장 실패");
    eprintln!(
        "[#903 Stage42] generated {} bytes={} pages={} report={}",
        out_path.display(),
        hwp_bytes.len(),
        reloaded.page_count(),
        report_path.display()
    );
}

fn task903_shape_component_field_at(data: &[u8], offset: usize) -> String {
    let is_two_ctrl_id = data.len() >= 8 && data[0..4] == data[4..8];
    let id_offset = if is_two_ctrl_id { 8 } else { 4 };

    if offset < 4 {
        return "ctrl_id[0]".to_string();
    }
    if is_two_ctrl_id && offset < 8 {
        return "ctrl_id[1]".to_string();
    }
    if offset < id_offset {
        return "ctrl_id/prefix".to_string();
    }

    let rel = offset - id_offset;
    match rel {
        0..=3 => "offset_x".to_string(),
        4..=7 => "offset_y".to_string(),
        8..=9 => "group_level".to_string(),
        10..=11 => "local_file_version".to_string(),
        12..=15 => "original_width".to_string(),
        16..=19 => "original_height".to_string(),
        20..=23 => "current_width".to_string(),
        24..=27 => "current_height".to_string(),
        28..=31 => "flip".to_string(),
        32..=33 => "rotation_angle".to_string(),
        34..=37 => "rotation_center_x".to_string(),
        38..=41 => "rotation_center_y".to_string(),
        42..=43 => "rendering_count".to_string(),
        _ => {
            let cnt = task903_read_u16_at(data, id_offset + 42).unwrap_or(0) as usize;
            let rendering_end = id_offset + 44 + 48 + cnt * 96;
            if offset < rendering_end {
                "raw_rendering".to_string()
            } else {
                "drawing_extra_after_rendering".to_string()
            }
        }
    }
}

fn task903_decode_shape_component_payload(data: &[u8]) -> String {
    let is_two_ctrl_id = data.len() >= 8 && data[0..4] == data[4..8];
    let id_offset = if is_two_ctrl_id { 8 } else { 4 };
    let ctrl_id = task903_read_u32_at(data, 0).unwrap_or(0);

    let offset_x = task903_read_u32_at(data, id_offset).unwrap_or(0) as i32;
    let offset_y = task903_read_u32_at(data, id_offset + 4).unwrap_or(0) as i32;
    let group_level = task903_read_u16_at(data, id_offset + 8).unwrap_or(0);
    let local_file_version = task903_read_u16_at(data, id_offset + 10).unwrap_or(0);
    let original_width = task903_read_u32_at(data, id_offset + 12).unwrap_or(0);
    let original_height = task903_read_u32_at(data, id_offset + 16).unwrap_or(0);
    let current_width = task903_read_u32_at(data, id_offset + 20).unwrap_or(0);
    let current_height = task903_read_u32_at(data, id_offset + 24).unwrap_or(0);
    let flip = task903_read_u32_at(data, id_offset + 28).unwrap_or(0);
    let rotation_angle = task903_read_i16_at(data, id_offset + 32).unwrap_or(0);
    let rotation_center_x = task903_read_u32_at(data, id_offset + 34).unwrap_or(0) as i32;
    let rotation_center_y = task903_read_u32_at(data, id_offset + 38).unwrap_or(0) as i32;
    let rendering_count = task903_read_u16_at(data, id_offset + 42).unwrap_or(0);
    let rendering_len = 2usize + 48usize + rendering_count as usize * 96usize;
    let rendering_end = id_offset + 42 + rendering_len;
    let extra_len = data.len().saturating_sub(rendering_end);

    format!(
        "ctrl_id={:#010x}, two_ctrl_id={}, offset=({},{}), group_level={}, local_ver={}, original={}x{}, current={}x{}, flip={:#010x}, rotation={}, center=({},{}), rendering_count={}, rendering_len={}, extra_after_rendering={}",
        ctrl_id,
        is_two_ctrl_id,
        offset_x,
        offset_y,
        group_level,
        local_file_version,
        original_width,
        original_height,
        current_width,
        current_height,
        flip,
        rotation_angle,
        rotation_center_x,
        rotation_center_y,
        rendering_count,
        rendering_len,
        extra_len
    )
}

fn task903_shape_picture_field_at(offset: usize) -> &'static str {
    match offset {
        0..=3 => "border_color",
        4..=7 => "border_width",
        8..=11 => "border_attr",
        12..=27 => "border_x[4]",
        28..=43 => "border_y[4]",
        44..=47 => "crop_left",
        48..=51 => "crop_top",
        52..=55 => "crop_right",
        56..=59 => "crop_bottom",
        60..=61 => "padding_left",
        62..=63 => "padding_right",
        64..=65 => "padding_top",
        66..=67 => "padding_bottom",
        68 => "brightness",
        69 => "contrast",
        70 => "effect",
        71..=72 => "bin_data_id",
        _ => "raw_picture_extra",
    }
}

fn task903_decode_shape_picture_payload(data: &[u8]) -> String {
    let border_color = task903_read_u32_at(data, 0).unwrap_or(0);
    let border_width = task903_read_u32_at(data, 4).unwrap_or(0) as i32;
    let border_attr = task903_read_u32_at(data, 8).unwrap_or(0);
    let border_x = [
        task903_read_u32_at(data, 12).unwrap_or(0) as i32,
        task903_read_u32_at(data, 16).unwrap_or(0) as i32,
        task903_read_u32_at(data, 20).unwrap_or(0) as i32,
        task903_read_u32_at(data, 24).unwrap_or(0) as i32,
    ];
    let border_y = [
        task903_read_u32_at(data, 28).unwrap_or(0) as i32,
        task903_read_u32_at(data, 32).unwrap_or(0) as i32,
        task903_read_u32_at(data, 36).unwrap_or(0) as i32,
        task903_read_u32_at(data, 40).unwrap_or(0) as i32,
    ];
    let crop = [
        task903_read_u32_at(data, 44).unwrap_or(0) as i32,
        task903_read_u32_at(data, 48).unwrap_or(0) as i32,
        task903_read_u32_at(data, 52).unwrap_or(0) as i32,
        task903_read_u32_at(data, 56).unwrap_or(0) as i32,
    ];
    let padding = [
        task903_read_i16_at(data, 60).unwrap_or(0),
        task903_read_i16_at(data, 62).unwrap_or(0),
        task903_read_i16_at(data, 64).unwrap_or(0),
        task903_read_i16_at(data, 66).unwrap_or(0),
    ];
    let brightness = data.get(68).copied().unwrap_or(0) as i8;
    let contrast = data.get(69).copied().unwrap_or(0) as i8;
    let effect = data.get(70).copied().unwrap_or(0);
    let bin_data_id = task903_read_u16_at(data, 71).unwrap_or(0);
    let extra_len = data.len().saturating_sub(73);

    format!(
        "border_color={:#010x}, border_width={}, border_attr={:#010x}, border_x={:?}, border_y={:?}, crop={:?}, padding={:?}, brightness={}, contrast={}, effect={}, bin_data_id={}, raw_picture_extra_len={}",
        border_color,
        border_width,
        border_attr,
        border_x,
        border_y,
        crop,
        padding,
        brightness,
        contrast,
        effect,
        bin_data_id,
        extra_len
    )
}

fn task903_shape_picture_border_tuple(data: &[u8]) -> ([i32; 4], [i32; 4]) {
    (
        [
            task903_read_u32_at(data, 12).unwrap_or(0) as i32,
            task903_read_u32_at(data, 16).unwrap_or(0) as i32,
            task903_read_u32_at(data, 20).unwrap_or(0) as i32,
            task903_read_u32_at(data, 24).unwrap_or(0) as i32,
        ],
        [
            task903_read_u32_at(data, 28).unwrap_or(0) as i32,
            task903_read_u32_at(data, 32).unwrap_or(0) as i32,
            task903_read_u32_at(data, 36).unwrap_or(0) as i32,
            task903_read_u32_at(data, 40).unwrap_or(0) as i32,
        ],
    )
}

fn task903_payload_diff_field(tag: u32, data: &[u8], first_diff: &str) -> String {
    let offset = first_diff
        .split(':')
        .next()
        .and_then(|s| s.parse::<usize>().ok());
    match (tag, offset) {
        (76, Some(offset)) => task903_shape_component_field_at(data, offset),
        (85, Some(offset)) => task903_shape_picture_field_at(offset).to_string(),
        _ => "-".to_string(),
    }
}

fn task903_payload_window(data: &[u8], first_diff: &str) -> String {
    let Some(offset) = first_diff
        .split(':')
        .next()
        .and_then(|s| s.parse::<usize>().ok())
    else {
        return "-".to_string();
    };
    let start = offset.saturating_sub(8);
    let end = (offset + 24).min(data.len());
    task903_hex_bytes(&data[start..end])
}

#[test]
#[ignore]
fn task903_stage43_generate_shape_payload_diff_report() {
    use std::fmt::Write as _;

    let target_bytes =
        std::fs::read("output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/hwpx-h-01.hwp")
            .expect("Stage42 target 파일 필요");
    let target_compressed = task903_file_header_flags(&target_bytes) & 0x01 != 0;
    let target_section0 = task903_read_decompressed_section(&target_bytes, 0, target_compressed)
        .expect("Stage42 target section0 읽기 실패");

    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let positive_section0 = positive.sections[0]
        .raw_stream
        .as_ref()
        .expect("positive section0 raw_stream 필요");

    let target_records = task903_parse_records(&target_section0);
    let positive_records = task903_parse_records(positive_section0);
    assert_eq!(
        target_records.len(),
        positive_records.len(),
        "Stage43 record count mismatch"
    );

    let mut shape_component_diffs = Vec::new();
    let mut shape_picture_diffs = Vec::new();

    for (idx, (target_record, positive_record)) in target_records
        .iter()
        .zip(positive_records.iter())
        .enumerate()
    {
        assert_eq!(
            target_record.tag, positive_record.tag,
            "tag mismatch at {idx}"
        );
        assert_eq!(
            target_record.level, positive_record.level,
            "level mismatch at {idx}"
        );
        if target_record.tag != 76 && target_record.tag != 85 {
            continue;
        }

        let target_payload = &target_section0
            [target_record.data_offset..target_record.data_offset + target_record.size];
        let positive_payload = &positive_section0
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        if target_payload == positive_payload {
            continue;
        }

        if target_record.tag == 76 {
            shape_component_diffs.push(idx);
        } else {
            shape_picture_diffs.push(idx);
        }
    }

    assert_eq!(shape_component_diffs.len(), 6, "SHAPE_COMPONENT diff count");
    assert_eq!(shape_picture_diffs.len(), 5, "SHAPE_PICTURE diff count");

    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage43_shape_payload_diff");
    std::fs::create_dir_all(out_dir).expect("Stage43 output dir 생성 실패");
    let report_path = out_dir.join("shape_payload_diff.md");

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 43 SHAPE Payload Diff").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. Summary").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- target: `output/poc/hwpx2hwp/task903/stage42_table_rowsizes_adapter/hwpx-h-01.hwp`"
    )
    .unwrap();
    writeln!(
        report,
        "- positive: `output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp`"
    )
    .unwrap();
    writeln!(report, "- record count: {}", target_records.len()).unwrap();
    writeln!(
        report,
        "- SHAPE_COMPONENT diff records: {:?}",
        shape_component_diffs
    )
    .unwrap();
    writeln!(
        report,
        "- SHAPE_PICTURE diff records: {:?}",
        shape_picture_diffs
    )
    .unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 2. Payload Diff Table").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | tag | level | target size | positive size | first diff | first diff field | target hash | positive hash | target decoded | positive decoded |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---:|---:|---:|---|---|---|---|---|---|").unwrap();

    for idx in shape_component_diffs
        .iter()
        .copied()
        .chain(shape_picture_diffs.iter().copied())
    {
        let target_record = &target_records[idx];
        let positive_record = &positive_records[idx];
        let target_payload = &target_section0
            [target_record.data_offset..target_record.data_offset + target_record.size];
        let positive_payload = &positive_section0
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        let first_diff = task903_first_diff(target_payload, positive_payload);
        let field = task903_payload_diff_field(target_record.tag, target_payload, &first_diff);
        let target_decoded = match target_record.tag {
            76 => task903_decode_shape_component_payload(target_payload),
            85 => task903_decode_shape_picture_payload(target_payload),
            _ => "-".to_string(),
        };
        let positive_decoded = match positive_record.tag {
            76 => task903_decode_shape_component_payload(positive_payload),
            85 => task903_decode_shape_picture_payload(positive_payload),
            _ => "-".to_string(),
        };

        writeln!(
            report,
            "| {} | {}({}) | {} | {} | {} | {} | {} | `{}` | `{}` | `{}` | `{}` |",
            idx,
            task903_record_tag_name(target_record.tag),
            target_record.tag,
            target_record.level,
            target_record.size,
            positive_record.size,
            first_diff,
            field,
            task903_short_hash(target_payload),
            task903_short_hash(positive_payload),
            target_decoded,
            positive_decoded
        )
        .unwrap();
    }

    writeln!(report).unwrap();
    writeln!(report, "## 3. First Diff Windows").unwrap();
    writeln!(report).unwrap();
    for idx in shape_component_diffs
        .iter()
        .copied()
        .chain(shape_picture_diffs.iter().copied())
    {
        let target_record = &target_records[idx];
        let positive_record = &positive_records[idx];
        let target_payload = &target_section0
            [target_record.data_offset..target_record.data_offset + target_record.size];
        let positive_payload = &positive_section0
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        let first_diff = task903_first_diff(target_payload, positive_payload);
        writeln!(
            report,
            "### idx {} {}({})",
            idx,
            task903_record_tag_name(target_record.tag),
            target_record.tag
        )
        .unwrap();
        writeln!(report).unwrap();
        writeln!(report, "- first diff: {}", first_diff).unwrap();
        writeln!(
            report,
            "- target window: `{}`",
            task903_payload_window(target_payload, &first_diff)
        )
        .unwrap();
        writeln!(
            report,
            "- positive window: `{}`",
            task903_payload_window(positive_payload, &first_diff)
        )
        .unwrap();
        writeln!(report).unwrap();
    }

    std::fs::write(&report_path, report).expect("Stage43 report 저장 실패");
    eprintln!("[#903 Stage43] wrote {}", report_path.display());
}

#[test]
fn task903_stage44_generate_shape_materialized_candidate() {
    use std::fmt::Write as _;

    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    let adapter_report = convert_hwpx_to_hwp_ir(core.document_mut());
    eprintln!("[#903 Stage44] adapter report: {:?}", adapter_report);

    let hwp_bytes =
        rhwp::serializer::serialize_hwp(core.document()).expect("Stage44 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "Stage44 산출물이 너무 작음: {} bytes",
        hwp_bytes.len()
    );

    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage44_shape_materialized_candidate");
    std::fs::create_dir_all(out_dir).expect("Stage44 output dir 생성 실패");
    let out_path = out_dir.join("hwpx-h-01.hwp");
    std::fs::write(&out_path, &hwp_bytes).expect("Stage44 HWP 저장 실패");

    let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("Stage44 HWP rhwp 재로드 실패");
    assert_eq!(
        reloaded.page_count(),
        9,
        "Stage44 산출물은 rhwp 기준 9페이지여야 함"
    );

    let compressed = task903_file_header_flags(&hwp_bytes) & 0x01 != 0;
    let output_section0 = task903_read_decompressed_section(&hwp_bytes, 0, compressed)
        .expect("Stage44 output section0 읽기 실패");

    let Some(positive) = task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    ) else {
        return;
    };
    let positive_section0 = positive.sections[0]
        .raw_stream
        .as_ref()
        .expect("positive section0 raw_stream 필요");

    let output_records = task903_parse_records(&output_section0);
    let positive_records = task903_parse_records(positive_section0);
    assert_eq!(
        output_records.len(),
        positive_records.len(),
        "Stage44 record count mismatch"
    );

    let mut shape_component_diffs = Vec::new();
    let mut shape_picture_diffs = Vec::new();
    let mut picture_border_rows = Vec::new();

    for (idx, (output_record, positive_record)) in output_records
        .iter()
        .zip(positive_records.iter())
        .enumerate()
    {
        assert_eq!(
            output_record.tag, positive_record.tag,
            "tag mismatch at {idx}"
        );
        assert_eq!(
            output_record.level, positive_record.level,
            "level mismatch at {idx}"
        );
        if output_record.tag != 76 && output_record.tag != 85 {
            continue;
        }

        let output_payload = &output_section0
            [output_record.data_offset..output_record.data_offset + output_record.size];
        let positive_payload = &positive_section0
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];

        if output_record.tag == 85 {
            let output_border = task903_shape_picture_border_tuple(output_payload);
            let positive_border = task903_shape_picture_border_tuple(positive_payload);
            picture_border_rows.push((idx, output_border, positive_border));
        }

        if output_payload == positive_payload {
            continue;
        }

        if output_record.tag == 76 {
            shape_component_diffs.push(idx);
        } else {
            shape_picture_diffs.push(idx);
        }
    }

    let report_path = out_dir.join("shape_payload_after_materialize.md");
    let mut report = String::new();
    writeln!(
        report,
        "# Task m100 #903 Stage 44 Shape Materialized Candidate"
    )
    .unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. Summary").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "- output: `{}`", out_path.display()).unwrap();
    writeln!(
        report,
        "- positive: `output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp`"
    )
    .unwrap();
    writeln!(report, "- bytes: {}", hwp_bytes.len()).unwrap();
    writeln!(report, "- rhwp pages: {}", reloaded.page_count()).unwrap();
    writeln!(report, "- record count: {}", output_records.len()).unwrap();
    writeln!(
        report,
        "- SHAPE_COMPONENT remaining diff records: {:?}",
        shape_component_diffs
    )
    .unwrap();
    writeln!(
        report,
        "- SHAPE_PICTURE remaining diff records: {:?}",
        shape_picture_diffs
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. Picture Border Materialization").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | output border_x | output border_y | positive border_x | positive border_y | match |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---|---|---|---|").unwrap();
    for (idx, output_border, positive_border) in &picture_border_rows {
        writeln!(
            report,
            "| {} | `{:?}` | `{:?}` | `{:?}` | `{:?}` | {} |",
            idx,
            output_border.0,
            output_border.1,
            positive_border.0,
            positive_border.1,
            output_border == positive_border
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 3. Remaining Payload Diff Table").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | tag | level | output size | positive size | first diff | first diff field | output decoded | positive decoded |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---:|---:|---:|---|---|---|---|").unwrap();

    for idx in shape_component_diffs
        .iter()
        .copied()
        .chain(shape_picture_diffs.iter().copied())
    {
        let output_record = &output_records[idx];
        let positive_record = &positive_records[idx];
        let output_payload = &output_section0
            [output_record.data_offset..output_record.data_offset + output_record.size];
        let positive_payload = &positive_section0
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        let first_diff = task903_first_diff(output_payload, positive_payload);
        let field = task903_payload_diff_field(output_record.tag, output_payload, &first_diff);
        let output_decoded = match output_record.tag {
            76 => task903_decode_shape_component_payload(output_payload),
            85 => task903_decode_shape_picture_payload(output_payload),
            _ => "-".to_string(),
        };
        let positive_decoded = match positive_record.tag {
            76 => task903_decode_shape_component_payload(positive_payload),
            85 => task903_decode_shape_picture_payload(positive_payload),
            _ => "-".to_string(),
        };

        writeln!(
            report,
            "| {} | {}({}) | {} | {} | {} | {} | {} | `{}` | `{}` |",
            idx,
            task903_record_tag_name(output_record.tag),
            output_record.tag,
            output_record.level,
            output_record.size,
            positive_record.size,
            first_diff,
            field,
            output_decoded,
            positive_decoded
        )
        .unwrap();
    }

    std::fs::write(&report_path, report).expect("Stage44 report 저장 실패");
    eprintln!(
        "[#903 Stage44] generated {} bytes={} pages={} report={}",
        out_path.display(),
        hwp_bytes.len(),
        reloaded.page_count(),
        report_path.display()
    );

    assert!(
        picture_border_rows
            .iter()
            .all(|(_, output_border, _)| output_border.0 != [0; 4] || output_border.1 != [0; 4]),
        "Stage44 SHAPE_PICTURE imgRect border가 materialize되지 않음"
    );
}

const TASK903_STAGE45_SHAPE_COMPONENT_INDICES: &[usize] = &[21, 35, 807, 808, 810, 812];
const TASK903_STAGE45_SHAPE_PICTURE_INDICES: &[usize] = &[22, 36, 809, 811, 813];

fn task903_stage45_base_hwp_bytes() -> Vec<u8> {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드 실패");
    convert_hwpx_to_hwp_ir(core.document_mut());
    rhwp::serializer::serialize_hwp(core.document()).expect("Stage45 base HWP 직렬화 실패")
}

fn task903_stage45_base_hwp_document() -> Document {
    let hwpx_bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&hwpx_bytes).expect("HWPX 로드 실패");
    convert_hwpx_to_hwp_ir(core.document_mut());

    let hwp_bytes =
        rhwp::serializer::serialize_hwp(core.document()).expect("Stage45 base HWP 직렬화 실패");
    let compressed = task903_file_header_flags(&hwp_bytes) & 0x01 != 0;

    let mut doc = core.document().clone();
    for section_idx in 0..doc.sections.len() {
        let raw = task903_read_decompressed_section(&hwp_bytes, section_idx as u32, compressed)
            .unwrap_or_else(|| panic!("Stage45 base section{} raw stream 읽기 실패", section_idx));
        doc.sections[section_idx].raw_stream = Some(raw);
    }
    task903_stage36_prepare_for_raw_bodytext(&mut doc);
    doc
}

fn task903_stage45_positive_document() -> Option<Document> {
    task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp",
    )
}

fn task903_shape_component_id_offset(data: &[u8]) -> usize {
    if data.len() >= 8 && data[0..4] == data[4..8] {
        8
    } else {
        4
    }
}

fn task903_shape_component_rendering_range(data: &[u8]) -> std::ops::Range<usize> {
    let id_offset = task903_shape_component_id_offset(data);
    let count = task903_read_u16_at(data, id_offset + 42).unwrap_or(0) as usize;
    let start = id_offset + 42;
    let len = 2usize + 48usize + count * 96usize;
    start..(start + len).min(data.len())
}

fn task903_shape_component_rendering_count(data: &[u8]) -> u16 {
    let id_offset = task903_shape_component_id_offset(data);
    task903_read_u16_at(data, id_offset + 42).unwrap_or(0)
}

fn task903_shape_picture_extra(data: &[u8]) -> &[u8] {
    if data.len() > 73 {
        &data[73..]
    } else {
        &[]
    }
}

fn task903_decode_shape_picture_extra(data: &[u8]) -> String {
    let extra = task903_shape_picture_extra(data);
    let border_opacity = extra.get(0).copied().unwrap_or(0);
    let instance_id = task903_read_u32_at(data, 74).unwrap_or(0);
    let image_effect_extra = task903_read_u32_at(data, 78).unwrap_or(0);
    let original_width = task903_read_u32_at(data, 82).unwrap_or(0);
    let original_height = task903_read_u32_at(data, 86).unwrap_or(0);
    let flag = data.get(90).copied().unwrap_or(0);
    format!(
        "extra_len={}, border_opacity={}, instance_id={}, image_effect_extra={:#010x}, original={}x{}, flag={}",
        extra.len(),
        border_opacity,
        instance_id,
        image_effect_extra,
        original_width,
        original_height,
        flag
    )
}

fn task903_graft_picture_extra_by_index(
    target_raw: &[u8],
    positive_raw: &[u8],
    indices: &[usize],
) -> Vec<u8> {
    let target_records = task903_parse_records(target_raw);
    let positive_records = task903_parse_records(positive_raw);
    assert_eq!(
        target_records.len(),
        positive_records.len(),
        "BodyText record count mismatch"
    );

    let mut out = target_raw.to_vec();
    for &idx in indices {
        let target_record = &target_records[idx];
        let positive_record = &positive_records[idx];
        assert_eq!(target_record.tag, 85, "idx {idx} must be SHAPE_PICTURE");
        assert_eq!(
            positive_record.tag, 85,
            "positive idx {idx} must be SHAPE_PICTURE"
        );
        assert_eq!(
            target_record.size, positive_record.size,
            "SHAPE_PICTURE size mismatch at idx {idx}"
        );
        assert!(
            target_record.size >= 73,
            "SHAPE_PICTURE payload too short at idx {idx}"
        );

        let target_extra_start = target_record.data_offset + 73;
        let target_extra_end = task903_record_end(target_record);
        let positive_extra_start = positive_record.data_offset + 73;
        let positive_extra_end = task903_record_end(positive_record);
        out[target_extra_start..target_extra_end]
            .copy_from_slice(&positive_raw[positive_extra_start..positive_extra_end]);
    }
    out
}

fn task903_patch_section0_picture_extra(doc: &mut Document, positive: &Document) {
    let target_raw = doc.sections[0]
        .raw_stream
        .as_ref()
        .expect("Stage45 target section0 raw_stream 필요");
    let positive_raw = positive.sections[0]
        .raw_stream
        .as_ref()
        .expect("Stage45 positive section0 raw_stream 필요");
    let patched = task903_graft_picture_extra_by_index(
        target_raw,
        positive_raw,
        TASK903_STAGE45_SHAPE_PICTURE_INDICES,
    );
    doc.sections[0].raw_stream = Some(patched);
    task903_stage36_prepare_for_raw_bodytext(doc);
}

fn task903_patch_section0_shape_components(doc: &mut Document, positive: &Document) {
    task903_patch_section0_indices(doc, positive, TASK903_STAGE45_SHAPE_COMPONENT_INDICES);
}

fn task903_stage45_force_group_child_rendering_in_shape(shape: &mut ShapeObject) -> usize {
    match shape {
        ShapeObject::Picture(pic) => {
            if pic.shape_attr.group_level == 0 {
                return 0;
            }
            pic.shape_attr.raw_rendering.clear();
            pic.shape_attr.render_tx = 0.0;
            pic.shape_attr.render_ty = 0.0;
            pic.shape_attr.render_sx = 1.0;
            pic.shape_attr.render_sy = 1.0;
            pic.shape_attr.render_b = 0.0;
            pic.shape_attr.render_c = 0.0;
            1
        }
        ShapeObject::Group(group) => group
            .children
            .iter_mut()
            .map(task903_stage45_force_group_child_rendering_in_shape)
            .sum(),
        _ => 0,
    }
}

fn task903_stage45_force_group_child_rendering_count2(doc: &mut Document) -> usize {
    let mut changed = 0usize;
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            for ctrl in &mut para.controls {
                match ctrl {
                    Control::Shape(shape) => {
                        changed +=
                            task903_stage45_force_group_child_rendering_in_shape(shape.as_mut());
                    }
                    Control::Picture(pic) => {
                        if pic.shape_attr.group_level > 0 {
                            pic.shape_attr.raw_rendering.clear();
                            pic.shape_attr.render_tx = 0.0;
                            pic.shape_attr.render_ty = 0.0;
                            pic.shape_attr.render_sx = 1.0;
                            pic.shape_attr.render_sy = 1.0;
                            pic.shape_attr.render_b = 0.0;
                            pic.shape_attr.render_c = 0.0;
                            changed += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    changed
}

fn task903_stage45_write_probe(
    out_dir: &std::path::Path,
    name: &str,
    doc: &Document,
) -> (usize, Option<u32>) {
    let hwp_bytes = rhwp::serializer::serialize_hwp(doc).expect("Stage45 HWP 직렬화 실패");
    let out_path = out_dir.join(name);
    std::fs::write(&out_path, &hwp_bytes).expect("Stage45 HWP 저장 실패");
    let reloaded_pages = DocumentCore::from_bytes(&hwp_bytes)
        .map(|core| core.page_count())
        .ok();
    eprintln!(
        "[#903 Stage45] generated {} bytes={} rhwp_pages={:?}",
        out_path.display(),
        hwp_bytes.len(),
        reloaded_pages
    );
    (hwp_bytes.len(), reloaded_pages)
}

fn task903_stage45_write_detail_report(
    out_dir: &std::path::Path,
    base_raw: &[u8],
    positive_raw: &[u8],
) {
    use std::fmt::Write as _;

    let base_records = task903_parse_records(base_raw);
    let positive_records = task903_parse_records(positive_raw);
    assert_eq!(base_records.len(), positive_records.len());

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 45 Shape Remaining Detail").unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 1. SHAPE_COMPONENT raw_rendering").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | base count | positive count | base rendering bytes | positive rendering bytes | base decoded | positive decoded |"
    )
    .unwrap();
    writeln!(report, "|---:|---:|---:|---|---|---|---|").unwrap();
    for &idx in TASK903_STAGE45_SHAPE_COMPONENT_INDICES {
        let base_record = &base_records[idx];
        let positive_record = &positive_records[idx];
        let base_payload =
            &base_raw[base_record.data_offset..base_record.data_offset + base_record.size];
        let positive_payload = &positive_raw
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        let base_range = task903_shape_component_rendering_range(base_payload);
        let positive_range = task903_shape_component_rendering_range(positive_payload);
        writeln!(
            report,
            "| {} | {} | {} | `{}` | `{}` | `{}` | `{}` |",
            idx,
            task903_shape_component_rendering_count(base_payload),
            task903_shape_component_rendering_count(positive_payload),
            task903_hex_bytes(&base_payload[base_range]),
            task903_hex_bytes(&positive_payload[positive_range]),
            task903_decode_shape_component_payload(base_payload),
            task903_decode_shape_component_payload(positive_payload)
        )
        .unwrap();
    }

    writeln!(report).unwrap();
    writeln!(report, "## 2. SHAPE_PICTURE raw_picture_extra").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | base extra | positive extra | base decoded | positive decoded | full base decoded | full positive decoded |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---|---|---|---|---|").unwrap();
    for &idx in TASK903_STAGE45_SHAPE_PICTURE_INDICES {
        let base_record = &base_records[idx];
        let positive_record = &positive_records[idx];
        let base_payload =
            &base_raw[base_record.data_offset..base_record.data_offset + base_record.size];
        let positive_payload = &positive_raw
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        writeln!(
            report,
            "| {} | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
            idx,
            task903_hex_bytes(task903_shape_picture_extra(base_payload)),
            task903_hex_bytes(task903_shape_picture_extra(positive_payload)),
            task903_decode_shape_picture_extra(base_payload),
            task903_decode_shape_picture_extra(positive_payload),
            task903_decode_shape_picture_payload(base_payload),
            task903_decode_shape_picture_payload(positive_payload)
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("shape_remaining_detail.md"), report)
        .expect("Stage45 detail report 저장 실패");
}

#[test]
fn task903_stage45_generate_shape_remaining_probe() {
    let Some(positive) = task903_stage45_positive_document() else {
        return;
    };
    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage45_shape_remaining_probe");
    std::fs::create_dir_all(out_dir).expect("Stage45 output dir 생성 실패");

    let base = task903_stage45_base_hwp_document();
    let base_raw = base.sections[0]
        .raw_stream
        .as_ref()
        .expect("Stage45 base section0 raw_stream 필요")
        .clone();
    let positive_raw = positive.sections[0]
        .raw_stream
        .as_ref()
        .expect("Stage45 positive section0 raw_stream 필요")
        .clone();
    task903_stage45_write_detail_report(out_dir, &base_raw, &positive_raw);

    let mut probe1 = base.clone();
    task903_patch_section0_picture_extra(&mut probe1, &positive);
    task903_stage45_write_probe(out_dir, "01_picture_extra_from_positive.hwp", &probe1);

    let mut probe2 = base.clone();
    task903_patch_section0_shape_components(&mut probe2, &positive);
    task903_stage45_write_probe(out_dir, "02_shape_raw_rendering_from_positive.hwp", &probe2);

    let mut probe3 = base.clone();
    task903_patch_section0_shape_components(&mut probe3, &positive);
    task903_patch_section0_picture_extra(&mut probe3, &positive);
    task903_stage45_write_probe(
        out_dir,
        "03_picture_extra_plus_shape_raw_rendering.hwp",
        &probe3,
    );

    let mut clean_core =
        DocumentCore::from_bytes(&load_sample("hwpx-h-01.hwpx")).expect("HWPX 로드 실패");
    convert_hwpx_to_hwp_ir(clean_core.document_mut());
    let changed = task903_stage45_force_group_child_rendering_count2(clean_core.document_mut());
    assert!(
        changed > 0,
        "group child rendering 후보 적용 대상이 있어야 함"
    );
    task903_stage45_write_probe(
        out_dir,
        "04_group_child_rendering_count2_clean_candidate.hwp",
        clean_core.document(),
    );
}

const TASK903_STAGE46_TABLE_REQUIRED_INDICES: &[usize] =
    &[48, 103, 286, 433, 563, 742, 1619, 2944, 6466];
const TASK903_STAGE46_SHAPE_FULL_INDICES: &[usize] =
    &[21, 22, 35, 36, 807, 808, 809, 810, 811, 812, 813];

fn task903_stage46_patch_docinfo_bindata(doc: &mut Document, positive: &Document) {
    doc.doc_info.bin_data_list = positive.doc_info.bin_data_list.clone();
    task903_stage35_mark_docinfo_dirty(doc);
}

fn task903_stage46_patch_table_required(doc: &mut Document, positive: &Document) {
    task903_patch_section0_indices(doc, positive, TASK903_STAGE46_TABLE_REQUIRED_INDICES);
}

fn task903_stage46_patch_shape_full(doc: &mut Document, positive: &Document) {
    task903_patch_section0_indices(doc, positive, TASK903_STAGE46_SHAPE_FULL_INDICES);
}

fn task903_stage46_patch_section0_index_union(
    doc: &mut Document,
    positive: &Document,
    groups: &[&[usize]],
) {
    let mut indices = Vec::new();
    for group in groups {
        indices.extend_from_slice(group);
    }
    indices.sort_unstable();
    indices.dedup();
    task903_patch_section0_indices(doc, positive, &indices);
}

fn task903_stage46_write_probe(
    out_dir: &std::path::Path,
    name: &str,
    doc: &Document,
) -> (usize, Option<u32>) {
    let hwp_bytes = rhwp::serializer::serialize_hwp(doc).expect("Stage46 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "{} 산출물이 너무 작음: {} bytes",
        name,
        hwp_bytes.len()
    );
    let out_path = out_dir.join(name);
    std::fs::write(&out_path, &hwp_bytes).expect("Stage46 HWP 저장 실패");
    let reloaded_pages = DocumentCore::from_bytes(&hwp_bytes)
        .map(|core| core.page_count())
        .ok();
    eprintln!(
        "[#903 Stage46] generated {} bytes={} rhwp_pages={:?}",
        out_path.display(),
        hwp_bytes.len(),
        reloaded_pages
    );
    (hwp_bytes.len(), reloaded_pages)
}

fn task903_stage46_write_detail_report(
    out_dir: &std::path::Path,
    base: &Document,
    positive: &Document,
    rows: &[(&str, usize, Option<u32>)],
) {
    use std::fmt::Write as _;

    let mut report = String::new();
    writeln!(
        report,
        "# Task m100 #903 Stage 46 BinData/Table Integration Detail"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 1. 기준").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- base: Stage44/45 clean adapter output with materialized shape fields"
    )
    .unwrap();
    writeln!(
        report,
        "- positive: `output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp`"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. 생성 파일").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| variant | bytes | rhwp reload |").unwrap();
    writeln!(report, "|---|---:|---|").unwrap();
    for (name, len, pages) in rows {
        writeln!(
            report,
            "| `{}` | {} | {} |",
            name,
            len,
            pages
                .map(|v| format!("ok, pages={}", v))
                .unwrap_or_else(|| "failed".to_string())
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 3. DocInfo BIN_DATA").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | base raw | positive raw | base model | positive model |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---|---|---|").unwrap();
    for (idx, (base_bin, positive_bin)) in base
        .doc_info
        .bin_data_list
        .iter()
        .zip(&positive.doc_info.bin_data_list)
        .enumerate()
    {
        writeln!(
            report,
            "| {} | `{}` | `{}` | `attr={:#x}, storage_id={}, ext={:?}, type={:?}, compression={:?}, status={:?}` | `attr={:#x}, storage_id={}, ext={:?}, type={:?}, compression={:?}, status={:?}` |",
            idx + 1,
            base_bin
                .raw_data
                .as_deref()
                .map(task903_hex_bytes)
                .unwrap_or_else(|| "-".to_string()),
            positive_bin
                .raw_data
                .as_deref()
                .map(task903_hex_bytes)
                .unwrap_or_else(|| "-".to_string()),
            base_bin.attr,
            base_bin.storage_id,
            base_bin.extension,
            base_bin.data_type,
            base_bin.compression,
            base_bin.status,
            positive_bin.attr,
            positive_bin.storage_id,
            positive_bin.extension,
            positive_bin.data_type,
            positive_bin.compression,
            positive_bin.status,
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 4. Section0 적용 대상").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- shape full indices: {:?}",
        TASK903_STAGE46_SHAPE_FULL_INDICES
    )
    .unwrap();
    writeln!(
        report,
        "- table required indices: {:?}",
        TASK903_STAGE46_TABLE_REQUIRED_INDICES
    )
    .unwrap();

    std::fs::write(out_dir.join("integration_detail.md"), report)
        .expect("Stage46 detail report 저장 실패");
}

#[test]
fn task903_stage46_generate_bindata_table_integration_probe() {
    let Some(positive) = task903_stage45_positive_document() else {
        return;
    };
    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe");
    std::fs::create_dir_all(out_dir).expect("Stage46 output dir 생성 실패");

    let base = task903_stage45_base_hwp_document();
    let mut rows = Vec::new();

    let mut probe1 = base.clone();
    task903_stage46_patch_docinfo_bindata(&mut probe1, &positive);
    rows.push((
        "01_docinfo_bindata_only.hwp",
        task903_stage46_write_probe(out_dir, "01_docinfo_bindata_only.hwp", &probe1),
    ));

    let mut probe2 = base.clone();
    task903_stage46_patch_table_required(&mut probe2, &positive);
    rows.push((
        "02_table_required_payload_only.hwp",
        task903_stage46_write_probe(out_dir, "02_table_required_payload_only.hwp", &probe2),
    ));

    let mut probe3 = base.clone();
    task903_stage46_patch_docinfo_bindata(&mut probe3, &positive);
    task903_stage46_patch_table_required(&mut probe3, &positive);
    rows.push((
        "03_docinfo_bindata_plus_table_required.hwp",
        task903_stage46_write_probe(
            out_dir,
            "03_docinfo_bindata_plus_table_required.hwp",
            &probe3,
        ),
    ));

    let mut probe4 = base.clone();
    task903_stage46_patch_section0_index_union(
        &mut probe4,
        &positive,
        &[
            TASK903_STAGE46_SHAPE_FULL_INDICES,
            TASK903_STAGE46_TABLE_REQUIRED_INDICES,
        ],
    );
    rows.push((
        "04_shape_full_payload_plus_table_required.hwp",
        task903_stage46_write_probe(
            out_dir,
            "04_shape_full_payload_plus_table_required.hwp",
            &probe4,
        ),
    ));

    let mut probe5 = base.clone();
    task903_stage46_patch_docinfo_bindata(&mut probe5, &positive);
    task903_stage46_patch_shape_full(&mut probe5, &positive);
    rows.push((
        "05_docinfo_bindata_plus_shape_full_payload.hwp",
        task903_stage46_write_probe(
            out_dir,
            "05_docinfo_bindata_plus_shape_full_payload.hwp",
            &probe5,
        ),
    ));

    let mut probe6 = base.clone();
    task903_stage46_patch_docinfo_bindata(&mut probe6, &positive);
    task903_stage46_patch_section0_index_union(
        &mut probe6,
        &positive,
        &[
            TASK903_STAGE46_SHAPE_FULL_INDICES,
            TASK903_STAGE46_TABLE_REQUIRED_INDICES,
        ],
    );
    rows.push((
        "06_docinfo_bindata_plus_shape_full_payload_plus_table_required.hwp",
        task903_stage46_write_probe(
            out_dir,
            "06_docinfo_bindata_plus_shape_full_payload_plus_table_required.hwp",
            &probe6,
        ),
    ));

    let report_rows: Vec<(&str, usize, Option<u32>)> = rows
        .iter()
        .map(|(name, (len, pages))| (*name, *len, *pages))
        .collect();
    task903_stage46_write_detail_report(out_dir, &base, &positive, &report_rows);
}

fn task903_stage47_success_document() -> Document {
    task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp",
    )
    .expect("Stage47 success 기준 파일 필요")
}

fn task903_stage47_failing_document() -> Document {
    task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/06_docinfo_bindata_plus_shape_full_payload_plus_table_required.hwp",
    )
    .expect("Stage47 failing 기준 파일 필요")
}

fn task903_stage47_write_section0_diff_report(
    out_dir: &std::path::Path,
    success: &Document,
    failing: &Document,
) {
    use std::collections::BTreeMap;
    use std::fmt::Write as _;

    let success_raw = success.sections[0]
        .raw_stream
        .as_ref()
        .expect("Stage47 success section0 raw_stream 필요");
    let failing_raw = failing.sections[0]
        .raw_stream
        .as_ref()
        .expect("Stage47 failing section0 raw_stream 필요");
    let success_records = task903_parse_records(success_raw);
    let failing_records = task903_parse_records(failing_raw);

    let mut tag_diff_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut diff_rows = Vec::new();
    let record_len = success_records.len().min(failing_records.len());
    for idx in 0..record_len {
        let success_record = &success_records[idx];
        let failing_record = &failing_records[idx];
        let tag_level_same = success_record.tag == failing_record.tag
            && success_record.level == failing_record.level;
        let success_payload = &success_raw
            [success_record.data_offset..success_record.data_offset + success_record.size];
        let failing_payload = &failing_raw
            [failing_record.data_offset..failing_record.data_offset + failing_record.size];
        let payload_same = success_payload == failing_payload;
        if !tag_level_same || !payload_same {
            *tag_diff_counts.entry(success_record.tag).or_default() += 1;
            diff_rows.push((
                idx,
                success_record.tag,
                failing_record.tag,
                success_record.level,
                failing_record.level,
                success_record.size,
                failing_record.size,
                task903_first_diff(success_payload, failing_payload),
            ));
        }
    }

    let mut report = String::new();
    writeln!(
        report,
        "# Task m100 #903 Stage 47 Success vs Stage46 Section0 Diff"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 1. 기준").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- success: `output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp`"
    )
    .unwrap();
    writeln!(
        report,
        "- failing: `output/poc/hwpx2hwp/task903/stage46_bindata_table_integration_probe/06_docinfo_bindata_plus_shape_full_payload_plus_table_required.hwp`"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. Record summary").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| item | success | failing |").unwrap();
    writeln!(report, "|---|---:|---:|").unwrap();
    writeln!(
        report,
        "| section0 bytes | {} | {} |",
        success_raw.len(),
        failing_raw.len()
    )
    .unwrap();
    writeln!(
        report,
        "| record count | {} | {} |",
        success_records.len(),
        failing_records.len()
    )
    .unwrap();
    writeln!(
        report,
        "| differing comparable records | {} | {} |",
        diff_rows.len(),
        diff_rows.len()
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 3. Diff counts by tag").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| tag | name | count |").unwrap();
    writeln!(report, "|---:|---|---:|").unwrap();
    for (tag, count) in &tag_diff_counts {
        writeln!(
            report,
            "| {} | {} | {} |",
            tag,
            task903_record_tag_name(*tag),
            count
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 4. Header-class diff rows").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | tag | success size | failing size | first diff |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---:|---:|---|").unwrap();
    for (
        idx,
        success_tag,
        _failing_tag,
        _success_level,
        _failing_level,
        success_size,
        failing_size,
        first_diff,
    ) in diff_rows
        .iter()
        .filter(|(_, tag, _, _, _, _, _, _)| [66, 71, 72].contains(tag))
    {
        writeln!(
            report,
            "| {} | {}({}) | {} | {} | {} |",
            idx,
            task903_record_tag_name(*success_tag),
            success_tag,
            success_size,
            failing_size,
            first_diff
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 5. Shape/Table watch indices").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | tag | success size | failing size | first diff |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---:|---:|---|").unwrap();
    for &(
        idx,
        success_tag,
        _failing_tag,
        _success_level,
        _failing_level,
        success_size,
        failing_size,
        ref first_diff,
    ) in &diff_rows
    {
        if TASK903_STAGE46_SHAPE_FULL_INDICES.contains(&idx)
            || TASK903_STAGE46_TABLE_REQUIRED_INDICES.contains(&idx)
        {
            writeln!(
                report,
                "| {} | {}({}) | {} | {} | {} |",
                idx,
                task903_record_tag_name(success_tag),
                success_tag,
                success_size,
                failing_size,
                first_diff
            )
            .unwrap();
        }
    }
    writeln!(report).unwrap();

    writeln!(report, "## 6. First 300 diff rows").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| idx | success tag | failing tag | success level | failing level | success size | failing size | first diff |").unwrap();
    writeln!(report, "|---:|---|---|---:|---:|---:|---:|---|").unwrap();
    for (
        idx,
        success_tag,
        failing_tag,
        success_level,
        failing_level,
        success_size,
        failing_size,
        first_diff,
    ) in diff_rows.iter().take(300)
    {
        writeln!(
            report,
            "| {} | {}({}) | {}({}) | {} | {} | {} | {} | {} |",
            idx,
            task903_record_tag_name(*success_tag),
            success_tag,
            task903_record_tag_name(*failing_tag),
            failing_tag,
            success_level,
            failing_level,
            success_size,
            failing_size,
            first_diff
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("stage47_section0_diff.md"), report)
        .expect("Stage47 section0 diff report 저장 실패");
}

fn task903_stage47_write_probe(
    out_dir: &std::path::Path,
    name: &str,
    doc: &Document,
) -> (usize, Option<u32>) {
    let hwp_bytes = rhwp::serializer::serialize_hwp(doc).expect("Stage47 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "{} 산출물이 너무 작음: {} bytes",
        name,
        hwp_bytes.len()
    );
    let out_path = out_dir.join(name);
    std::fs::write(&out_path, &hwp_bytes).expect("Stage47 HWP 저장 실패");
    let reloaded_pages = DocumentCore::from_bytes(&hwp_bytes)
        .map(|core| core.page_count())
        .ok();
    eprintln!(
        "[#903 Stage47] generated {} bytes={} rhwp_pages={:?}",
        out_path.display(),
        hwp_bytes.len(),
        reloaded_pages
    );
    (hwp_bytes.len(), reloaded_pages)
}

fn task903_stage47_write_generation_report(
    out_dir: &std::path::Path,
    rows: &[(&str, usize, Option<u32>)],
) {
    use std::fmt::Write as _;

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 47 Header Prereq Probe").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 생성 파일").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| variant | bytes | rhwp reload |").unwrap();
    writeln!(report, "|---|---:|---|").unwrap();
    for (name, len, pages) in rows {
        writeln!(
            report,
            "| `{}` | {} | {} |",
            name,
            len,
            pages
                .map(|v| format!("ok, pages={}", v))
                .unwrap_or_else(|| "failed".to_string())
        )
        .unwrap();
    }
    writeln!(report).unwrap();
    writeln!(report, "## 2. 적용 tag").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "- CTRL_HEADER: tag 71").unwrap();
    writeln!(report, "- LIST_HEADER: tag 72").unwrap();
    writeln!(report, "- PARA_HEADER: tag 66").unwrap();

    std::fs::write(out_dir.join("header_prereq_detail.md"), report)
        .expect("Stage47 generation report 저장 실패");
}

#[test]
#[ignore]
fn task903_stage47_generate_header_prereq_probe() {
    let positive = task903_stage45_positive_document().expect("Stage47 positive 파일 필요");
    let success = task903_stage47_success_document();
    let baseline = task903_stage47_failing_document();

    let diff_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage47_success_vs_stage46_diff");
    std::fs::create_dir_all(diff_dir).expect("Stage47 diff output dir 생성 실패");
    task903_stage47_write_section0_diff_report(diff_dir, &success, &baseline);

    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage47_header_prereq_probe");
    std::fs::create_dir_all(out_dir).expect("Stage47 probe output dir 생성 실패");

    let variants: &[(&str, &[u32])] = &[
        ("01_stage46_06_plus_ctrl_header.hwp", &[71]),
        ("02_stage46_06_plus_list_header.hwp", &[72]),
        ("03_stage46_06_plus_para_header.hwp", &[66]),
        ("04_stage46_06_plus_ctrl_list_headers.hwp", &[71, 72]),
        ("05_stage46_06_plus_ctrl_para_headers.hwp", &[71, 66]),
        (
            "06_stage46_06_plus_ctrl_list_para_headers.hwp",
            &[71, 72, 66],
        ),
    ];

    let mut rows = Vec::new();
    for (name, tags) in variants {
        let mut doc = baseline.clone();
        task903_patch_section0_tags(&mut doc, &positive, tags);
        let (len, pages) = task903_stage47_write_probe(out_dir, name, &doc);
        rows.push((*name, len, pages));
    }
    task903_stage47_write_generation_report(out_dir, &rows);
}

fn task903_stage48_baseline_document() -> Document {
    task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/06_stage46_06_plus_ctrl_list_para_headers.hwp",
    )
    .expect("Stage48 baseline 파일 필요")
}

fn task903_stage48_write_residual_diff_report(
    out_dir: &std::path::Path,
    success: &Document,
    baseline: &Document,
) {
    use std::collections::BTreeMap;
    use std::fmt::Write as _;

    let success_raw = success.sections[0]
        .raw_stream
        .as_ref()
        .expect("Stage48 success section0 raw_stream 필요");
    let baseline_raw = baseline.sections[0]
        .raw_stream
        .as_ref()
        .expect("Stage48 baseline section0 raw_stream 필요");
    let success_records = task903_parse_records(success_raw);
    let baseline_records = task903_parse_records(baseline_raw);

    let mut tag_diff_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut diff_rows = Vec::new();
    let record_len = success_records.len().min(baseline_records.len());
    for idx in 0..record_len {
        let success_record = &success_records[idx];
        let baseline_record = &baseline_records[idx];
        let success_payload = &success_raw
            [success_record.data_offset..success_record.data_offset + success_record.size];
        let baseline_payload = &baseline_raw
            [baseline_record.data_offset..baseline_record.data_offset + baseline_record.size];
        if success_record.tag != baseline_record.tag
            || success_record.level != baseline_record.level
            || success_payload != baseline_payload
        {
            *tag_diff_counts.entry(success_record.tag).or_default() += 1;
            diff_rows.push((
                idx,
                success_record.tag,
                baseline_record.tag,
                success_record.level,
                baseline_record.level,
                success_record.size,
                baseline_record.size,
                task903_first_diff(success_payload, baseline_payload),
            ));
        }
    }

    let mut report = String::new();
    writeln!(
        report,
        "# Task m100 #903 Stage 48 Residual Text Layout Diff"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 1. 기준").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- success: `output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp`"
    )
    .unwrap();
    writeln!(
        report,
        "- baseline: `output/poc/hwpx2hwp/task903/stage47_header_prereq_probe/06_stage46_06_plus_ctrl_list_para_headers.hwp`"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. Record summary").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| item | success | baseline |").unwrap();
    writeln!(report, "|---|---:|---:|").unwrap();
    writeln!(
        report,
        "| section0 bytes | {} | {} |",
        success_raw.len(),
        baseline_raw.len()
    )
    .unwrap();
    writeln!(
        report,
        "| record count | {} | {} |",
        success_records.len(),
        baseline_records.len()
    )
    .unwrap();
    writeln!(
        report,
        "| differing comparable records | {} | {} |",
        diff_rows.len(),
        diff_rows.len()
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 3. Diff counts by tag").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| tag | name | count |").unwrap();
    writeln!(report, "|---:|---|---:|").unwrap();
    for (tag, count) in &tag_diff_counts {
        writeln!(
            report,
            "| {} | {} | {} |",
            tag,
            task903_record_tag_name(*tag),
            count
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 4. Residual diff rows").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| idx | success tag | baseline tag | success level | baseline level | success size | baseline size | first diff |").unwrap();
    writeln!(report, "|---:|---|---|---:|---:|---:|---:|---|").unwrap();
    for (
        idx,
        success_tag,
        baseline_tag,
        success_level,
        baseline_level,
        success_size,
        baseline_size,
        first_diff,
    ) in diff_rows.iter().take(500)
    {
        writeln!(
            report,
            "| {} | {}({}) | {}({}) | {} | {} | {} | {} | {} |",
            idx,
            task903_record_tag_name(*success_tag),
            success_tag,
            task903_record_tag_name(*baseline_tag),
            baseline_tag,
            success_level,
            baseline_level,
            success_size,
            baseline_size,
            first_diff
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("stage48_residual_diff.md"), report)
        .expect("Stage48 residual diff report 저장 실패");
}

fn task903_stage48_write_probe(
    out_dir: &std::path::Path,
    name: &str,
    doc: &Document,
) -> (usize, Option<u32>) {
    let hwp_bytes = rhwp::serializer::serialize_hwp(doc).expect("Stage48 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "{} 산출물이 너무 작음: {} bytes",
        name,
        hwp_bytes.len()
    );
    let out_path = out_dir.join(name);
    std::fs::write(&out_path, &hwp_bytes).expect("Stage48 HWP 저장 실패");
    let reloaded_pages = DocumentCore::from_bytes(&hwp_bytes)
        .map(|core| core.page_count())
        .ok();
    eprintln!(
        "[#903 Stage48] generated {} bytes={} rhwp_pages={:?}",
        out_path.display(),
        hwp_bytes.len(),
        reloaded_pages
    );
    (hwp_bytes.len(), reloaded_pages)
}

fn task903_stage48_write_generation_report(
    out_dir: &std::path::Path,
    rows: &[(&str, usize, Option<u32>)],
) {
    use std::fmt::Write as _;

    let mut report = String::new();
    writeln!(
        report,
        "# Task m100 #903 Stage 48 Residual Text Layout Probe"
    )
    .unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 생성 파일").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| variant | bytes | rhwp reload |").unwrap();
    writeln!(report, "|---|---:|---|").unwrap();
    for (name, len, pages) in rows {
        writeln!(
            report,
            "| `{}` | {} | {} |",
            name,
            len,
            pages
                .map(|v| format!("ok, pages={}", v))
                .unwrap_or_else(|| "failed".to_string())
        )
        .unwrap();
    }
    writeln!(report).unwrap();
    writeln!(report, "## 2. 적용 tag").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "- PARA_TEXT: tag 67").unwrap();
    writeln!(report, "- PARA_CHAR_SHAPE: tag 68").unwrap();
    writeln!(report, "- PARA_LINE_SEG: tag 69").unwrap();
    writeln!(report, "- TABLE: tag 77").unwrap();

    std::fs::write(out_dir.join("residual_text_layout_detail.md"), report)
        .expect("Stage48 generation report 저장 실패");
}

#[test]
#[ignore]
fn task903_stage48_generate_residual_text_layout_probe() {
    let success = task903_stage47_success_document();
    let baseline = task903_stage48_baseline_document();

    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe");
    std::fs::create_dir_all(out_dir).expect("Stage48 output dir 생성 실패");
    task903_stage48_write_residual_diff_report(out_dir, &success, &baseline);

    let variants: &[(&str, &[u32])] = &[
        ("01_plus_para_text.hwp", &[67]),
        ("02_plus_para_char_shape.hwp", &[68]),
        ("03_plus_para_line_seg.hwp", &[69]),
        ("04_plus_table_all.hwp", &[77]),
        ("05_plus_text_char_shape.hwp", &[67, 68]),
        ("06_plus_text_char_line_seg.hwp", &[67, 68, 69]),
        ("07_plus_line_seg_table.hwp", &[69, 77]),
        ("08_plus_text_char_line_seg_table.hwp", &[67, 68, 69, 77]),
    ];

    let mut rows = Vec::new();
    for (name, tags) in variants {
        let mut doc = baseline.clone();
        task903_patch_section0_tags(&mut doc, &success, tags);
        let (len, pages) = task903_stage48_write_probe(out_dir, name, &doc);
        rows.push((*name, len, pages));
    }
    task903_stage48_write_generation_report(out_dir, &rows);
}

fn task903_stage49_positive_document() -> Document {
    task903_stage47_success_document()
}

fn task903_stage49_baseline_document() -> Document {
    task903_load_hwp_document(
        "output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp",
    )
    .expect("Stage49 baseline 파일 필요")
}

fn task903_stage49_patch_docinfo_tags(doc: &mut Document, positive: &Document, tags: &[u32]) {
    let target_raw = doc
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage49 target DocInfo raw_stream 필요");
    let positive_raw = positive
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage49 positive DocInfo raw_stream 필요");
    let patched = task903_graft_record_tags_by_index(target_raw, positive_raw, tags);
    doc.doc_info.raw_stream = Some(patched);
    doc.doc_info.raw_stream_dirty = false;
    doc.header.compressed = true;
    doc.header.flags |= 0x01;
    doc.header.raw_data = None;
}

#[derive(Default)]
struct Task903Stage49RefStats {
    top_paragraphs: usize,
    cell_paragraphs: usize,
    para_shape_ids: std::collections::BTreeSet<u16>,
    style_ids: std::collections::BTreeSet<u8>,
    char_shape_ids: std::collections::BTreeSet<u32>,
}

fn task903_stage49_collect_para_refs(
    para: &Paragraph,
    in_cell: bool,
    stats: &mut Task903Stage49RefStats,
) {
    if in_cell {
        stats.cell_paragraphs += 1;
    } else {
        stats.top_paragraphs += 1;
    }
    stats.para_shape_ids.insert(para.para_shape_id);
    stats.style_ids.insert(para.style_id);
    for char_shape in &para.char_shapes {
        stats.char_shape_ids.insert(char_shape.char_shape_id);
    }

    for control in &para.controls {
        if let Control::Table(table) = control {
            for cell in &table.cells {
                for cell_para in &cell.paragraphs {
                    task903_stage49_collect_para_refs(cell_para, true, stats);
                }
            }
        }
    }
}

fn task903_stage49_ref_stats(doc: &Document) -> Task903Stage49RefStats {
    let mut stats = Task903Stage49RefStats::default();
    for section in &doc.sections {
        for para in &section.paragraphs {
            task903_stage49_collect_para_refs(para, false, &mut stats);
        }
    }
    stats
}

fn task903_stage49_format_set<T: std::fmt::Display + Ord>(
    set: &std::collections::BTreeSet<T>,
) -> String {
    if set.is_empty() {
        return "-".to_string();
    }
    set.iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn task903_stage49_docinfo_model_counts(doc: &Document) -> Vec<(&'static str, usize)> {
    vec![
        ("bin_data", doc.doc_info.bin_data_list.len()),
        (
            "font_faces",
            doc.doc_info
                .font_faces
                .iter()
                .map(|fonts| fonts.len())
                .sum(),
        ),
        ("border_fills", doc.doc_info.border_fills.len()),
        ("char_shapes", doc.doc_info.char_shapes.len()),
        ("tab_defs", doc.doc_info.tab_defs.len()),
        ("numberings", doc.doc_info.numberings.len()),
        ("bullets", doc.doc_info.bullets.len()),
        ("para_shapes", doc.doc_info.para_shapes.len()),
        ("styles", doc.doc_info.styles.len()),
        ("extra_records", doc.doc_info.extra_records.len()),
    ]
}

fn task903_stage49_count_records_by_tag(raw: &[u8]) -> std::collections::BTreeMap<u32, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for record in task903_parse_records(raw) {
        *counts.entry(record.tag).or_default() += 1;
    }
    counts
}

fn task903_stage49_write_docinfo_ref_diff_report(
    out_dir: &std::path::Path,
    positive: &Document,
    baseline: &Document,
) {
    use std::collections::BTreeSet;
    use std::fmt::Write as _;

    let positive_raw = positive
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage49 positive DocInfo raw_stream 필요");
    let baseline_raw = baseline
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage49 baseline DocInfo raw_stream 필요");
    let positive_records = task903_parse_records(positive_raw);
    let baseline_records = task903_parse_records(baseline_raw);
    let positive_counts = task903_stage49_count_records_by_tag(positive_raw);
    let baseline_counts = task903_stage49_count_records_by_tag(baseline_raw);
    let all_tags: BTreeSet<u32> = positive_counts
        .keys()
        .chain(baseline_counts.keys())
        .copied()
        .collect();

    let mut tag_diff_counts: std::collections::BTreeMap<u32, usize> =
        std::collections::BTreeMap::new();
    let mut diff_rows = Vec::new();
    let record_len = positive_records.len().min(baseline_records.len());
    for idx in 0..record_len {
        let positive_record = &positive_records[idx];
        let baseline_record = &baseline_records[idx];
        let positive_payload = &positive_raw
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        let baseline_payload = &baseline_raw
            [baseline_record.data_offset..baseline_record.data_offset + baseline_record.size];
        if positive_record.tag != baseline_record.tag
            || positive_record.level != baseline_record.level
            || positive_payload != baseline_payload
        {
            *tag_diff_counts.entry(positive_record.tag).or_default() += 1;
            diff_rows.push((
                idx,
                positive_record.tag,
                baseline_record.tag,
                positive_record.size,
                baseline_record.size,
                task903_first_diff(positive_payload, baseline_payload),
            ));
        }
    }

    let positive_refs = task903_stage49_ref_stats(positive);
    let baseline_refs = task903_stage49_ref_stats(baseline);

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 49 DocInfo Ref Diff").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 기준").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- positive: `output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp`"
    )
    .unwrap();
    writeln!(
        report,
        "- baseline: `output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp`"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. DocInfo raw summary").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| item | positive | baseline |").unwrap();
    writeln!(report, "|---|---:|---:|").unwrap();
    writeln!(
        report,
        "| raw bytes | {} | {} |",
        positive_raw.len(),
        baseline_raw.len()
    )
    .unwrap();
    writeln!(
        report,
        "| records | {} | {} |",
        positive_records.len(),
        baseline_records.len()
    )
    .unwrap();
    writeln!(
        report,
        "| differing comparable records | {} | {} |",
        diff_rows.len(),
        diff_rows.len()
    )
    .unwrap();
    writeln!(
        report,
        "| blake3 | `{}` | `{}` |",
        task903_short_hash(positive_raw),
        task903_short_hash(baseline_raw)
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 3. DocInfo record counts").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| tag | name | positive | baseline |").unwrap();
    writeln!(report, "|---:|---|---:|---:|").unwrap();
    for tag in all_tags {
        writeln!(
            report,
            "| {} | {} | {} | {} |",
            tag,
            task903_record_tag_name(tag),
            positive_counts.get(&tag).copied().unwrap_or(0),
            baseline_counts.get(&tag).copied().unwrap_or(0)
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 4. DocInfo model counts").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| item | positive | baseline |").unwrap();
    writeln!(report, "|---|---:|---:|").unwrap();
    let positive_model_counts = task903_stage49_docinfo_model_counts(positive);
    let baseline_model_counts = task903_stage49_docinfo_model_counts(baseline);
    for ((name, positive_count), (_, baseline_count)) in positive_model_counts
        .iter()
        .zip(baseline_model_counts.iter())
    {
        writeln!(
            report,
            "| {} | {} | {} |",
            name, positive_count, baseline_count
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 5. BodyText reference ids").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| role | top paragraphs | cell paragraphs | para_shape_ids | style_ids | char_shape_ids |"
    )
    .unwrap();
    writeln!(report, "|---|---:|---:|---|---|---|").unwrap();
    writeln!(
        report,
        "| positive | {} | {} | {} | {} | {} |",
        positive_refs.top_paragraphs,
        positive_refs.cell_paragraphs,
        task903_stage49_format_set(&positive_refs.para_shape_ids),
        task903_stage49_format_set(&positive_refs.style_ids),
        task903_stage49_format_set(&positive_refs.char_shape_ids)
    )
    .unwrap();
    writeln!(
        report,
        "| baseline | {} | {} | {} | {} | {} |",
        baseline_refs.top_paragraphs,
        baseline_refs.cell_paragraphs,
        task903_stage49_format_set(&baseline_refs.para_shape_ids),
        task903_stage49_format_set(&baseline_refs.style_ids),
        task903_stage49_format_set(&baseline_refs.char_shape_ids)
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 6. Diff counts by DocInfo tag").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| tag | name | differing records |").unwrap();
    writeln!(report, "|---:|---|---:|").unwrap();
    for (tag, count) in &tag_diff_counts {
        writeln!(
            report,
            "| {} | {} | {} |",
            tag,
            task903_record_tag_name(*tag),
            count
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 7. First DocInfo diff rows").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | positive tag | baseline tag | positive size | baseline size | first diff |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---|---:|---:|---|").unwrap();
    for (idx, positive_tag, baseline_tag, positive_size, baseline_size, first_diff) in
        diff_rows.iter().take(300)
    {
        writeln!(
            report,
            "| {} | {}({}) | {}({}) | {} | {} | {} |",
            idx,
            task903_record_tag_name(*positive_tag),
            positive_tag,
            task903_record_tag_name(*baseline_tag),
            baseline_tag,
            positive_size,
            baseline_size,
            first_diff
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 8. 해석 메모").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "Stage48에서 BodyText residual graft가 실패했으므로, 이 리포트는 cell paragraph가 참조하는 DocInfo 쪽 값을 우선 분리하기 위한 것이다."
    )
    .unwrap();

    std::fs::write(out_dir.join("stage49_docinfo_ref_diff.md"), report)
        .expect("Stage49 DocInfo ref diff report 저장 실패");
}

fn task903_stage49_write_probe(
    out_dir: &std::path::Path,
    name: &str,
    doc: &Document,
) -> (usize, Option<u32>) {
    let hwp_bytes = rhwp::serializer::serialize_hwp(doc).expect("Stage49 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "{} 산출물이 너무 작음: {} bytes",
        name,
        hwp_bytes.len()
    );
    let out_path = out_dir.join(name);
    std::fs::write(&out_path, &hwp_bytes).expect("Stage49 HWP 저장 실패");
    let reloaded_pages = DocumentCore::from_bytes(&hwp_bytes)
        .map(|core| core.page_count())
        .ok();
    eprintln!(
        "[#903 Stage49] generated {} bytes={} rhwp_pages={:?}",
        out_path.display(),
        hwp_bytes.len(),
        reloaded_pages
    );
    (hwp_bytes.len(), reloaded_pages)
}

fn task903_stage49_write_generation_report(
    out_dir: &std::path::Path,
    rows: &[(&str, usize, Option<u32>)],
) {
    use std::fmt::Write as _;

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 49 DocInfo Text Clip Probe").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 생성 파일").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| variant | bytes | rhwp reload |").unwrap();
    writeln!(report, "|---|---:|---|").unwrap();
    for (name, len, pages) in rows {
        writeln!(
            report,
            "| `{}` | {} | {} |",
            name,
            len,
            pages
                .map(|v| format!("ok, pages={}", v))
                .unwrap_or_else(|| "failed".to_string())
        )
        .unwrap();
    }
    writeln!(report).unwrap();
    writeln!(report, "## 2. 적용 DocInfo tag").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "- FACE_NAME: tag 19").unwrap();
    writeln!(report, "- CHAR_SHAPE: tag 21").unwrap();
    writeln!(report, "- PARA_SHAPE: tag 25").unwrap();
    writeln!(report, "- STYLE: tag 26").unwrap();
    writeln!(
        report,
        "- 08은 BIN_DATA(tag 18)를 제외한 주요 DocInfo tag 묶음"
    )
    .unwrap();

    std::fs::write(out_dir.join("docinfo_text_clip_detail.md"), report)
        .expect("Stage49 generation report 저장 실패");
}

#[test]
#[ignore]
fn task903_stage49_generate_docinfo_text_clip_probe() {
    let positive = task903_stage49_positive_document();
    let baseline = task903_stage49_baseline_document();

    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage49_docinfo_text_clip_probe");
    std::fs::create_dir_all(out_dir).expect("Stage49 output dir 생성 실패");
    task903_stage49_write_docinfo_ref_diff_report(out_dir, &positive, &baseline);

    let variants: &[(&str, &[u32])] = &[
        ("01_para_shapes_from_positive.hwp", &[25]),
        ("02_char_shapes_from_positive.hwp", &[21]),
        ("03_styles_from_positive.hwp", &[26]),
        ("04_font_faces_from_positive.hwp", &[19]),
        ("05_para_char_shapes_from_positive.hwp", &[25, 21]),
        ("06_para_char_styles_from_positive.hwp", &[25, 21, 26]),
        ("07_docinfo_text_metrics_bundle.hwp", &[19, 21, 25, 26]),
        (
            "08_docinfo_all_non_bindata_from_positive.hwp",
            &[16, 17, 19, 20, 21, 22, 23, 24, 25, 26, 27, 30, 31],
        ),
    ];

    let mut rows = Vec::new();
    for (name, tags) in variants {
        let mut doc = baseline.clone();
        task903_stage49_patch_docinfo_tags(&mut doc, &positive, tags);
        let (len, pages) = task903_stage49_write_probe(out_dir, name, &doc);
        rows.push((*name, len, pages));
    }
    task903_stage49_write_generation_report(out_dir, &rows);
}

fn task903_stage50_patch_parashape_ranges(
    doc: &mut Document,
    positive: &Document,
    ranges: &[(usize, usize)],
) {
    let target_raw = doc
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage50 target DocInfo raw_stream 필요");
    let positive_raw = positive
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage50 positive DocInfo raw_stream 필요");
    let target_records = task903_parse_records(target_raw);
    let positive_records = task903_parse_records(positive_raw);
    assert_eq!(
        target_records.len(),
        positive_records.len(),
        "Stage50 DocInfo record count mismatch"
    );

    let mut out = Vec::with_capacity(target_raw.len());
    let mut cursor = 0usize;
    for (idx, target_record) in target_records.iter().enumerate() {
        let positive_record = &positive_records[idx];
        assert_eq!(
            target_record.tag, positive_record.tag,
            "Stage50 DocInfo tag mismatch at idx {}",
            idx
        );
        assert_eq!(
            target_record.level, positive_record.level,
            "Stage50 DocInfo level mismatch at idx {}",
            idx
        );

        if cursor < target_record.offset {
            out.extend_from_slice(&target_raw[cursor..target_record.offset]);
        }

        if target_record.tag == 25 {
            assert_eq!(
                target_record.size, positive_record.size,
                "Stage50 ParaShape payload size mismatch at idx {}",
                idx
            );
            let mut record_bytes =
                target_raw[target_record.offset..task903_record_end(target_record)].to_vec();
            let target_payload_base = target_record.data_offset - target_record.offset;
            let positive_payload = &positive_raw
                [positive_record.data_offset..positive_record.data_offset + positive_record.size];
            for &(start, end) in ranges {
                assert!(
                    start <= end && end <= target_record.size,
                    "Stage50 invalid ParaShape range {}..{} for payload size {}",
                    start,
                    end,
                    target_record.size
                );
                record_bytes[target_payload_base + start..target_payload_base + end]
                    .copy_from_slice(&positive_payload[start..end]);
            }
            out.extend_from_slice(&record_bytes);
        } else {
            out.extend_from_slice(
                &target_raw[target_record.offset..task903_record_end(target_record)],
            );
        }
        cursor = task903_record_end(target_record);
    }
    if cursor < target_raw.len() {
        out.extend_from_slice(&target_raw[cursor..]);
    }

    doc.doc_info.raw_stream = Some(out);
    doc.doc_info.raw_stream_dirty = false;
    doc.header.compressed = true;
    doc.header.flags |= 0x01;
    doc.header.raw_data = None;
}

fn task903_stage50_field_ranges() -> Vec<(&'static str, usize, usize)> {
    vec![
        ("attr1", 0, 4),
        ("margin_fields", 4, 24),
        ("line_spacing", 24, 28),
        ("reference_fields", 28, 34),
        ("border_spacing", 34, 42),
        ("attr2_attr3_lsv2", 42, 54),
    ]
}

fn task903_stage50_write_parashape_field_diff_report(
    out_dir: &std::path::Path,
    positive: &Document,
    baseline: &Document,
) {
    use std::collections::BTreeMap;
    use std::fmt::Write as _;

    let positive_raw = positive
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage50 positive DocInfo raw_stream 필요");
    let baseline_raw = baseline
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage50 baseline DocInfo raw_stream 필요");
    let positive_records = task903_parse_records(positive_raw);
    let baseline_records = task903_parse_records(baseline_raw);
    let baseline_refs = task903_stage49_ref_stats(baseline);
    let fields = task903_stage50_field_ranges();

    let mut field_diff_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut rows = Vec::new();
    let mut para_shape_id = 0usize;
    for (record_idx, positive_record) in positive_records.iter().enumerate() {
        if positive_record.tag != 25 {
            continue;
        }
        let baseline_record = &baseline_records[record_idx];
        assert_eq!(baseline_record.tag, 25, "Stage50 ParaShape tag mismatch");
        let positive_payload = &positive_raw
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        let baseline_payload = &baseline_raw
            [baseline_record.data_offset..baseline_record.data_offset + baseline_record.size];

        let mut diff_fields = Vec::new();
        for &(name, start, end) in &fields {
            if positive_payload.get(start..end) != baseline_payload.get(start..end) {
                *field_diff_counts.entry(name).or_default() += 1;
                diff_fields.push(name);
            }
        }

        rows.push((
            para_shape_id,
            record_idx,
            baseline_refs
                .para_shape_ids
                .contains(&(para_shape_id as u16)),
            positive_record.size,
            task903_first_diff(positive_payload, baseline_payload),
            diff_fields.join(", "),
        ));
        para_shape_id += 1;
    }

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 50 ParaShape Field Diff").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 기준").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- positive: `output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp`"
    )
    .unwrap();
    writeln!(
        report,
        "- baseline: `output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp`"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. Field ranges").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| field | payload range |").unwrap();
    writeln!(report, "|---|---|").unwrap();
    for (name, start, end) in &fields {
        writeln!(report, "| {} | {}..{} |", name, start, end).unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 3. Diff counts by field").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| field | differing ParaShape records |").unwrap();
    writeln!(report, "|---|---:|").unwrap();
    for (name, _, _) in &fields {
        writeln!(
            report,
            "| {} | {} |",
            name,
            field_diff_counts.get(name).copied().unwrap_or(0)
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 4. Referenced ParaShape ids").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "{}",
        task903_stage49_format_set(&baseline_refs.para_shape_ids)
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 5. ParaShape row detail").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| para_shape_id | docinfo_record_idx | referenced | payload_size | first diff | diff fields |"
    )
    .unwrap();
    writeln!(report, "|---:|---:|---|---:|---|---|").unwrap();
    for (id, record_idx, referenced, size, first_diff, diff_fields) in &rows {
        writeln!(
            report,
            "| {} | {} | {} | {} | {} | {} |",
            id,
            record_idx,
            referenced,
            size,
            first_diff,
            if diff_fields.is_empty() {
                "-"
            } else {
                diff_fields
            }
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("stage50_parashape_field_diff.md"), report)
        .expect("Stage50 ParaShape field diff report 저장 실패");
}

fn task903_stage50_write_probe(
    out_dir: &std::path::Path,
    name: &str,
    doc: &Document,
) -> (usize, Option<u32>) {
    let hwp_bytes = rhwp::serializer::serialize_hwp(doc).expect("Stage50 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "{} 산출물이 너무 작음: {} bytes",
        name,
        hwp_bytes.len()
    );
    let out_path = out_dir.join(name);
    std::fs::write(&out_path, &hwp_bytes).expect("Stage50 HWP 저장 실패");
    let reloaded_pages = DocumentCore::from_bytes(&hwp_bytes)
        .map(|core| core.page_count())
        .ok();
    eprintln!(
        "[#903 Stage50] generated {} bytes={} rhwp_pages={:?}",
        out_path.display(),
        hwp_bytes.len(),
        reloaded_pages
    );
    (hwp_bytes.len(), reloaded_pages)
}

fn task903_stage50_write_generation_report(
    out_dir: &std::path::Path,
    rows: &[(&str, usize, Option<u32>)],
) {
    use std::fmt::Write as _;

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 50 ParaShape Field Probe").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 생성 파일").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| variant | bytes | rhwp reload |").unwrap();
    writeln!(report, "|---|---:|---|").unwrap();
    for (name, len, pages) in rows {
        writeln!(
            report,
            "| `{}` | {} | {} |",
            name,
            len,
            pages
                .map(|v| format!("ok, pages={}", v))
                .unwrap_or_else(|| "failed".to_string())
        )
        .unwrap();
    }
    writeln!(report).unwrap();
    writeln!(report, "## 2. 후보 의미").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| variant | ParaShape payload ranges |").unwrap();
    writeln!(report, "|---|---|").unwrap();
    writeln!(report, "| 01_attr1_only | 0..4 |").unwrap();
    writeln!(report, "| 02_margin_fields_only | 4..24 |").unwrap();
    writeln!(report, "| 03_line_spacing_only | 24..28 |").unwrap();
    writeln!(report, "| 04_reference_fields_only | 28..34 |").unwrap();
    writeln!(report, "| 05_border_spacing_only | 34..42 |").unwrap();
    writeln!(report, "| 06_attr2_attr3_lsv2_only | 42..54 |").unwrap();
    writeln!(report, "| 07_attr1_line_spacing | 0..4, 24..28 |").unwrap();
    writeln!(report, "| 08_attr1_attr2_attr3_lsv2 | 0..4, 42..54 |").unwrap();
    writeln!(report, "| 09_all_except_margins | 0..4, 24..54 |").unwrap();
    writeln!(report, "| 10_all_parashape_positive_control | 0..54 |").unwrap();

    std::fs::write(out_dir.join("parashape_field_probe_detail.md"), report)
        .expect("Stage50 generation report 저장 실패");
}

#[test]
#[ignore]
fn task903_stage50_generate_parashape_field_probe() {
    let positive = task903_stage49_positive_document();
    let baseline = task903_stage49_baseline_document();

    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage50_parashape_field_probe");
    std::fs::create_dir_all(out_dir).expect("Stage50 output dir 생성 실패");
    task903_stage50_write_parashape_field_diff_report(out_dir, &positive, &baseline);

    let variants: &[(&str, &[(usize, usize)])] = &[
        ("01_attr1_only.hwp", &[(0, 4)]),
        ("02_margin_fields_only.hwp", &[(4, 24)]),
        ("03_line_spacing_only.hwp", &[(24, 28)]),
        ("04_reference_fields_only.hwp", &[(28, 34)]),
        ("05_border_spacing_only.hwp", &[(34, 42)]),
        ("06_attr2_attr3_lsv2_only.hwp", &[(42, 54)]),
        ("07_attr1_line_spacing.hwp", &[(0, 4), (24, 28)]),
        ("08_attr1_attr2_attr3_lsv2.hwp", &[(0, 4), (42, 54)]),
        (
            "09_all_except_margins.hwp",
            &[(0, 4), (24, 28), (28, 34), (34, 42), (42, 54)],
        ),
        ("10_all_parashape_positive_control.hwp", &[(0, 54)]),
    ];

    let mut rows = Vec::new();
    for (name, ranges) in variants {
        let mut doc = baseline.clone();
        task903_stage50_patch_parashape_ranges(&mut doc, &positive, ranges);
        let (len, pages) = task903_stage50_write_probe(out_dir, name, &doc);
        rows.push((*name, len, pages));
    }
    task903_stage50_write_generation_report(out_dir, &rows);
}

fn task903_stage51_attr1_masks() -> Vec<(&'static str, u32, &'static str)> {
    vec![
        (
            "01_line_spacing_type_bits_0_1.hwp",
            0x0000_0003,
            "bits 0..1: line spacing type",
        ),
        (
            "02_alignment_bits_2_4.hwp",
            0x0000_001c,
            "bits 2..4: alignment",
        ),
        (
            "03_break_unit_bits_5_7.hwp",
            0x0000_00e0,
            "bits 5..7: English/Korean break unit",
        ),
        (
            "04_snap_to_grid_bit_8.hwp",
            0x0000_0100,
            "bit 8: snap to page line grid",
        ),
        (
            "05_minimum_space_bits_9_15.hwp",
            0x0000_fe00,
            "bits 9..15: minimum space",
        ),
        (
            "06_paragraph_flow_bits_16_19.hwp",
            0x000f_0000,
            "bits 16..19: widow/orphan, keep, page break",
        ),
        (
            "07_vertical_align_bits_20_21.hwp",
            0x0030_0000,
            "bits 20..21: vertical alignment",
        ),
        (
            "08_font_line_height_bit_22.hwp",
            0x0040_0000,
            "bit 22: font line height",
        ),
        (
            "09_heading_bits_23_27.hwp",
            0x0f80_0000,
            "bits 23..27: heading type and level",
        ),
        (
            "10_border_tail_bits_28_30.hwp",
            0x7000_0000,
            "bits 28..30: border connection, ignore margin, tail",
        ),
        (
            "11_break_unit_plus_snap_to_grid.hwp",
            0x0000_01e0,
            "bits 5..8: break unit + snap to grid",
        ),
        (
            "12_snap_to_grid_plus_font_line_height.hwp",
            0x0040_0100,
            "bits 8 and 22: snap to grid + font line height",
        ),
        (
            "13_vertical_align_plus_font_line_height.hwp",
            0x0070_0000,
            "bits 20..22: vertical alignment + font line height",
        ),
        (
            "14_attr1_positive_control.hwp",
            0xffff_ffff,
            "all attr1 bits",
        ),
    ]
}

fn task903_stage51_patch_parashape_attr1_mask(doc: &mut Document, positive: &Document, mask: u32) {
    let target_raw = doc
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage51 target DocInfo raw_stream 필요");
    let positive_raw = positive
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage51 positive DocInfo raw_stream 필요");
    let target_records = task903_parse_records(target_raw);
    let positive_records = task903_parse_records(positive_raw);
    assert_eq!(
        target_records.len(),
        positive_records.len(),
        "Stage51 DocInfo record count mismatch"
    );

    let mut out = Vec::with_capacity(target_raw.len());
    let mut cursor = 0usize;
    for (idx, target_record) in target_records.iter().enumerate() {
        let positive_record = &positive_records[idx];
        assert_eq!(
            target_record.tag, positive_record.tag,
            "Stage51 DocInfo tag mismatch at idx {}",
            idx
        );
        assert_eq!(
            target_record.level, positive_record.level,
            "Stage51 DocInfo level mismatch at idx {}",
            idx
        );

        if cursor < target_record.offset {
            out.extend_from_slice(&target_raw[cursor..target_record.offset]);
        }

        if target_record.tag == 25 {
            assert_eq!(
                target_record.size, positive_record.size,
                "Stage51 ParaShape payload size mismatch at idx {}",
                idx
            );
            assert!(
                target_record.size >= 4,
                "Stage51 ParaShape payload too small at idx {}: {}",
                idx,
                target_record.size
            );
            let mut record_bytes =
                target_raw[target_record.offset..task903_record_end(target_record)].to_vec();
            let target_payload_base = target_record.data_offset - target_record.offset;
            let target_payload = &target_raw
                [target_record.data_offset..target_record.data_offset + target_record.size];
            let positive_payload = &positive_raw
                [positive_record.data_offset..positive_record.data_offset + positive_record.size];
            let target_attr1 = u32::from_le_bytes(target_payload[0..4].try_into().unwrap());
            let positive_attr1 = u32::from_le_bytes(positive_payload[0..4].try_into().unwrap());
            let patched_attr1 = (target_attr1 & !mask) | (positive_attr1 & mask);
            record_bytes[target_payload_base..target_payload_base + 4]
                .copy_from_slice(&patched_attr1.to_le_bytes());
            out.extend_from_slice(&record_bytes);
        } else {
            out.extend_from_slice(
                &target_raw[target_record.offset..task903_record_end(target_record)],
            );
        }
        cursor = task903_record_end(target_record);
    }
    if cursor < target_raw.len() {
        out.extend_from_slice(&target_raw[cursor..]);
    }

    doc.doc_info.raw_stream = Some(out);
    doc.doc_info.raw_stream_dirty = false;
    doc.header.compressed = true;
    doc.header.flags |= 0x01;
    doc.header.raw_data = None;
}

fn task903_stage51_write_attr1_bit_diff_report(
    out_dir: &std::path::Path,
    positive: &Document,
    baseline: &Document,
) {
    use std::fmt::Write as _;

    let positive_raw = positive
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage51 positive DocInfo raw_stream 필요");
    let baseline_raw = baseline
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage51 baseline DocInfo raw_stream 필요");
    let positive_records = task903_parse_records(positive_raw);
    let baseline_records = task903_parse_records(baseline_raw);
    let baseline_refs = task903_stage49_ref_stats(baseline);
    let masks = task903_stage51_attr1_masks();

    let mut diff_counts = vec![0usize; masks.len()];
    let mut referenced_diff_counts = vec![0usize; masks.len()];
    let mut rows = Vec::new();
    let mut para_shape_id = 0usize;

    for (record_idx, positive_record) in positive_records.iter().enumerate() {
        if positive_record.tag != 25 {
            continue;
        }
        let baseline_record = &baseline_records[record_idx];
        assert_eq!(baseline_record.tag, 25, "Stage51 ParaShape tag mismatch");
        let positive_payload = &positive_raw
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        let baseline_payload = &baseline_raw
            [baseline_record.data_offset..baseline_record.data_offset + baseline_record.size];
        assert!(
            positive_payload.len() >= 4 && baseline_payload.len() >= 4,
            "Stage51 ParaShape attr1 payload too small"
        );
        let positive_attr1 = u32::from_le_bytes(positive_payload[0..4].try_into().unwrap());
        let baseline_attr1 = u32::from_le_bytes(baseline_payload[0..4].try_into().unwrap());
        let xor = positive_attr1 ^ baseline_attr1;
        let referenced = baseline_refs
            .para_shape_ids
            .contains(&(para_shape_id as u16));
        let mut changed_groups = Vec::new();
        for (mask_idx, (name, mask, desc)) in masks.iter().enumerate() {
            if xor & mask != 0 {
                diff_counts[mask_idx] += 1;
                if referenced {
                    referenced_diff_counts[mask_idx] += 1;
                }
                changed_groups.push(format!(
                    "{} `{:#010x}` {}",
                    name.trim_end_matches(".hwp"),
                    mask,
                    desc
                ));
            }
        }
        if xor != 0 {
            rows.push((
                para_shape_id,
                record_idx,
                referenced,
                baseline_attr1,
                positive_attr1,
                xor,
                changed_groups.join("<br>"),
            ));
        }
        para_shape_id += 1;
    }

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 51 ParaShape attr1 Bit Diff").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 기준").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- positive: `output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp`"
    )
    .unwrap();
    writeln!(
        report,
        "- baseline: `output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp`"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. Mask별 diff count").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| variant | mask | meaning | differing ParaShape records | referenced differing records |"
    )
    .unwrap();
    writeln!(report, "|---|---:|---|---:|---:|").unwrap();
    for (idx, (name, mask, desc)) in masks.iter().enumerate() {
        writeln!(
            report,
            "| `{}` | `{:#010x}` | {} | {} | {} |",
            name, mask, desc, diff_counts[idx], referenced_diff_counts[idx]
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 3. BodyText 참조 ParaShape ids").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "{}",
        task903_stage49_format_set(&baseline_refs.para_shape_ids)
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 4. attr1 diff detail").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| para_shape_id | docinfo_record_idx | referenced | baseline attr1 | positive attr1 | xor | changed groups |"
    )
    .unwrap();
    writeln!(report, "|---:|---:|---|---:|---:|---:|---|").unwrap();
    for (
        para_shape_id,
        record_idx,
        referenced,
        baseline_attr1,
        positive_attr1,
        xor,
        changed_groups,
    ) in rows
    {
        writeln!(
            report,
            "| {} | {} | {} | `{:#010x}` | `{:#010x}` | `{:#010x}` | {} |",
            para_shape_id,
            record_idx,
            referenced,
            baseline_attr1,
            positive_attr1,
            xor,
            if changed_groups.is_empty() {
                "-".to_string()
            } else {
                changed_groups
            }
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("stage51_attr1_bit_diff.md"), report)
        .expect("Stage51 attr1 bit diff report 저장 실패");
}

fn task903_stage51_write_probe(
    out_dir: &std::path::Path,
    name: &str,
    doc: &Document,
) -> (usize, Option<u32>, String) {
    let hwp_bytes = rhwp::serializer::serialize_hwp(doc).expect("Stage51 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "{} 산출물이 너무 작음: {} bytes",
        name,
        hwp_bytes.len()
    );
    let hash = task903_short_hash(&hwp_bytes);
    let out_path = out_dir.join(name);
    std::fs::write(&out_path, &hwp_bytes).expect("Stage51 HWP 저장 실패");
    let reloaded_pages = DocumentCore::from_bytes(&hwp_bytes)
        .map(|core| core.page_count())
        .ok();
    eprintln!(
        "[#903 Stage51] generated {} bytes={} hash={} rhwp_pages={:?}",
        out_path.display(),
        hwp_bytes.len(),
        hash,
        reloaded_pages
    );
    (hwp_bytes.len(), reloaded_pages, hash)
}

fn task903_stage51_write_generation_report(
    out_dir: &std::path::Path,
    rows: &[(&str, u32, &str, usize, Option<u32>, String)],
) {
    use std::fmt::Write as _;

    let mut report = String::new();
    writeln!(
        report,
        "# Task m100 #903 Stage 51 ParaShape attr1 Bit Probe"
    )
    .unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 생성 파일").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| variant | mask | meaning | bytes | hash | rhwp reload |"
    )
    .unwrap();
    writeln!(report, "|---|---:|---|---:|---|---|").unwrap();
    for (name, mask, desc, len, pages, hash) in rows {
        writeln!(
            report,
            "| `{}` | `{:#010x}` | {} | {} | `{}` | {} |",
            name,
            mask,
            desc,
            len,
            hash,
            pages
                .map(|v| format!("ok, pages={}", v))
                .unwrap_or_else(|| "failed".to_string())
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 2. 작업지시자 판정표").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | rhwp-studio 판정 | 비고 |"
    )
    .unwrap();
    writeln!(report, "|---|---|---|---|---|---|---|").unwrap();
    for (name, _, _, _, _, _) in rows {
        writeln!(
            report,
            "| {} |  |  |  |  |  |  |",
            name.trim_end_matches(".hwp")
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("parashape_attr1_bit_probe_detail.md"), report)
        .expect("Stage51 generation report 저장 실패");
}

#[test]
#[ignore]
fn task903_stage51_generate_parashape_attr1_bit_probe() {
    let positive = task903_stage49_positive_document();
    let baseline = task903_stage49_baseline_document();

    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage51_parashape_attr1_bit_probe");
    std::fs::create_dir_all(out_dir).expect("Stage51 output dir 생성 실패");
    task903_stage51_write_attr1_bit_diff_report(out_dir, &positive, &baseline);

    let mut rows = Vec::new();
    for (name, mask, desc) in task903_stage51_attr1_masks() {
        let mut doc = baseline.clone();
        task903_stage51_patch_parashape_attr1_mask(&mut doc, &positive, mask);
        let (len, pages, hash) = task903_stage51_write_probe(out_dir, name, &doc);
        rows.push((name, mask, desc, len, pages, hash));
    }
    task903_stage51_write_generation_report(out_dir, &rows);
}

#[derive(Debug, Clone, Default)]
struct Task903Stage52ParaPrInfo {
    id: usize,
    tab_pr_id_ref: String,
    condense: String,
    font_line_height: String,
    snap_to_grid: String,
    horizontal: String,
    vertical: String,
    line_spacing_type: String,
    line_spacing_value: String,
    break_setting: String,
    auto_spacing_e_asian_eng: String,
    auto_spacing_e_asian_num: String,
}

fn task903_stage52_local_name(name: &[u8]) -> &[u8] {
    if let Some(pos) = name.iter().position(|&b| b == b':') {
        &name[pos + 1..]
    } else {
        name
    }
}

fn task903_stage52_attr_value(attr: &quick_xml::events::attributes::Attribute) -> String {
    String::from_utf8_lossy(attr.value.as_ref()).to_string()
}

fn task903_stage52_read_header_xml() -> String {
    use std::io::Read as _;

    let bytes = load_sample("hwpx-h-01.hwpx");
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).expect("Stage52 HWPX zip 열기 실패");
    let mut header = archive
        .by_name("Contents/header.xml")
        .expect("Stage52 Contents/header.xml 찾기 실패");
    let mut xml = String::new();
    header
        .read_to_string(&mut xml)
        .expect("Stage52 header.xml 읽기 실패");
    xml
}

fn task903_stage52_vertical_bits(value: &str) -> u32 {
    match value {
        "TOP" => 1,
        "CENTER" => 2,
        "BOTTOM" => 3,
        "BASELINE" | "" => 0,
        _ => 0,
    }
}

fn task903_stage52_read_para_pr_attrs(
    e: &quick_xml::events::BytesStart,
) -> Task903Stage52ParaPrInfo {
    let mut info = Task903Stage52ParaPrInfo::default();
    for attr in e.attributes().flatten() {
        match task903_stage52_local_name(attr.key.as_ref()) {
            b"id" => info.id = task903_stage52_attr_value(&attr).parse().unwrap_or(0),
            b"tabPrIDRef" => info.tab_pr_id_ref = task903_stage52_attr_value(&attr),
            b"condense" => info.condense = task903_stage52_attr_value(&attr),
            b"fontLineHeight" => info.font_line_height = task903_stage52_attr_value(&attr),
            b"snapToGrid" => info.snap_to_grid = task903_stage52_attr_value(&attr),
            _ => {}
        }
    }
    info
}

fn task903_stage52_apply_para_pr_child(
    e: &quick_xml::events::BytesStart,
    info: &mut Task903Stage52ParaPrInfo,
) {
    let name = e.name();
    let local = task903_stage52_local_name(name.as_ref());
    match local {
        b"align" => {
            for attr in e.attributes().flatten() {
                match task903_stage52_local_name(attr.key.as_ref()) {
                    b"horizontal" => info.horizontal = task903_stage52_attr_value(&attr),
                    b"vertical" => info.vertical = task903_stage52_attr_value(&attr),
                    _ => {}
                }
            }
        }
        b"lineSpacing" => {
            for attr in e.attributes().flatten() {
                match task903_stage52_local_name(attr.key.as_ref()) {
                    b"type" => info.line_spacing_type = task903_stage52_attr_value(&attr),
                    b"value" => info.line_spacing_value = task903_stage52_attr_value(&attr),
                    _ => {}
                }
            }
        }
        b"breakSetting" => {
            let mut attrs = Vec::new();
            for attr in e.attributes().flatten() {
                attrs.push(format!(
                    "{}={}",
                    String::from_utf8_lossy(task903_stage52_local_name(attr.key.as_ref())),
                    task903_stage52_attr_value(&attr)
                ));
            }
            info.break_setting = attrs.join(", ");
        }
        b"autoSpacing" => {
            for attr in e.attributes().flatten() {
                match task903_stage52_local_name(attr.key.as_ref()) {
                    b"eAsianEng" => {
                        info.auto_spacing_e_asian_eng = task903_stage52_attr_value(&attr)
                    }
                    b"eAsianNum" => {
                        info.auto_spacing_e_asian_num = task903_stage52_attr_value(&attr)
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn task903_stage52_collect_para_pr_inventory() -> Vec<Task903Stage52ParaPrInfo> {
    let xml = task903_stage52_read_header_xml();
    let mut reader = quick_xml::Reader::from_str(&xml);
    let mut buf = Vec::new();
    let mut current: Option<Task903Stage52ParaPrInfo> = None;
    let mut rows = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                let name = e.name();
                let local = task903_stage52_local_name(name.as_ref());
                match local {
                    b"paraPr" => {
                        current = Some(task903_stage52_read_para_pr_attrs(e));
                    }
                    _ => {
                        if let Some(info) = current.as_mut() {
                            task903_stage52_apply_para_pr_child(e, info);
                        }
                    }
                }
            }
            Ok(quick_xml::events::Event::Empty(ref e)) => {
                let name = e.name();
                let local = task903_stage52_local_name(name.as_ref());
                if local == b"paraPr" {
                    rows.push(task903_stage52_read_para_pr_attrs(e));
                } else if let Some(info) = current.as_mut() {
                    task903_stage52_apply_para_pr_child(e, info);
                }
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                let name = e.name();
                if task903_stage52_local_name(name.as_ref()) == b"paraPr" {
                    if let Some(info) = current.take() {
                        rows.push(info);
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(err) => panic!("Stage52 header.xml 파싱 실패: {}", err),
            _ => {}
        }
        buf.clear();
    }

    rows
}

fn task903_stage52_docinfo_parashape_attr1(doc: &Document) -> Vec<(usize, u32)> {
    if let Some(raw) = doc.doc_info.raw_stream.as_ref() {
        let mut rows = Vec::new();
        let mut para_shape_id = 0usize;
        for record in task903_parse_records(raw) {
            if record.tag != 25 {
                continue;
            }
            let payload = &raw[record.data_offset..record.data_offset + record.size];
            assert!(
                payload.len() >= 4,
                "Stage52 ParaShape payload too small: id={} size={}",
                para_shape_id,
                payload.len()
            );
            let attr1 = u32::from_le_bytes(payload[0..4].try_into().unwrap());
            rows.push((para_shape_id, attr1));
            para_shape_id += 1;
        }
        rows
    } else {
        doc.doc_info
            .para_shapes
            .iter()
            .enumerate()
            .map(|(idx, ps)| (idx, ps.attr1))
            .collect()
    }
}

fn task903_stage52_write_para_pr_inventory(
    out_dir: &std::path::Path,
    inventory: &[Task903Stage52ParaPrInfo],
) {
    use std::collections::BTreeMap;
    use std::fmt::Write as _;

    let mut vertical_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut auto_counts: BTreeMap<String, usize> = BTreeMap::new();
    for info in inventory {
        *vertical_counts.entry(info.vertical.clone()).or_default() += 1;
        *auto_counts
            .entry(format!(
                "{}/{}",
                info.auto_spacing_e_asian_eng, info.auto_spacing_e_asian_num
            ))
            .or_default() += 1;
    }

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 52 HWPX paraPr Inventory").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. Summary").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "- paraPr count: {}", inventory.len()).unwrap();
    writeln!(report).unwrap();
    writeln!(report, "### align.vertical").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| value | count | HWP attr1 bits 20..21 candidate |"
    )
    .unwrap();
    writeln!(report, "|---|---:|---:|").unwrap();
    for (value, count) in &vertical_counts {
        writeln!(
            report,
            "| `{}` | {} | {} |",
            value,
            count,
            task903_stage52_vertical_bits(value)
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "### autoSpacing").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| eAsianEng/eAsianNum | count |").unwrap();
    writeln!(report, "|---|---:|").unwrap();
    for (value, count) in &auto_counts {
        writeln!(report, "| `{}` | {} |", value, count).unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 2. paraPr detail").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| id | horizontal | vertical | candidate bits | autoSpacing | snapToGrid | fontLineHeight | lineSpacing | breakSetting |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---|---:|---|---|---|---|---|").unwrap();
    for info in inventory {
        writeln!(
            report,
            "| {} | `{}` | `{}` | {} | `{}/{}` | `{}` | `{}` | `{}/{}` | `{}` |",
            info.id,
            info.horizontal,
            info.vertical,
            task903_stage52_vertical_bits(&info.vertical),
            info.auto_spacing_e_asian_eng,
            info.auto_spacing_e_asian_num,
            info.snap_to_grid,
            info.font_line_height,
            info.line_spacing_type,
            info.line_spacing_value,
            info.break_setting
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("para_pr_inventory.md"), report)
        .expect("Stage52 paraPr inventory 저장 실패");
}

fn task903_stage52_write_vertical_bits_report(
    out_dir: &std::path::Path,
    inventory: &[Task903Stage52ParaPrInfo],
    parsed_doc: &Document,
    reference: &Document,
    positive: &Document,
    baseline: &Document,
) {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fmt::Write as _;

    let parsed_rows: BTreeMap<usize, u32> = task903_stage52_docinfo_parashape_attr1(parsed_doc)
        .into_iter()
        .collect();
    let reference_rows: BTreeMap<usize, u32> = task903_stage52_docinfo_parashape_attr1(reference)
        .into_iter()
        .collect();
    let positive_rows: BTreeMap<usize, u32> = task903_stage52_docinfo_parashape_attr1(positive)
        .into_iter()
        .collect();
    let baseline_rows: BTreeMap<usize, u32> = task903_stage52_docinfo_parashape_attr1(baseline)
        .into_iter()
        .collect();
    let refs = task903_stage49_ref_stats(positive);

    let center_ids: BTreeSet<usize> = inventory
        .iter()
        .filter(|info| info.vertical == "CENTER")
        .map(|info| info.id)
        .collect();
    let ref_center_ids: BTreeSet<usize> = reference_rows
        .iter()
        .filter(|(_, attr1)| ((*attr1 >> 20) & 0x03) == 2)
        .map(|(id, _)| *id)
        .collect();

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 52 ParaShape Vertical Bits").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 기준").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "- HWPX: `samples/hwpx/hwpx-h-01.hwpx`").unwrap();
    writeln!(
        report,
        "- reference HWP: `samples/hwpx/hancom-hwp/hwpx-h-01.hwp`"
    )
    .unwrap();
    writeln!(
        report,
        "- positive HWP: `output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp`"
    )
    .unwrap();
    writeln!(
        report,
        "- baseline HWP: `output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp`"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. 결론 후보").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- HWPX `align.vertical=CENTER` paraPr id: `{}`",
        task903_stage49_format_set(&center_ids)
    )
    .unwrap();
    writeln!(
        report,
        "- reference HWP `attr1 bits 20..21 == 2` ParaShape id: `{}`",
        task903_stage49_format_set(&ref_center_ids)
    )
    .unwrap();
    writeln!(
        report,
        "- autoSpacing은 본 샘플에서 `{}` 분포이며, CENTER id와 무관하다.",
        inventory
            .iter()
            .map(|info| format!(
                "{}/{}",
                info.auto_spacing_e_asian_eng, info.auto_spacing_e_asian_num
            ))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 3. ParaShape detail").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| id | referenced | HWPX vertical | expected bits | parsed bits | reference bits | positive bits | baseline bits | parsed attr1 | reference attr1 | positive attr1 | baseline attr1 |"
    )
    .unwrap();
    writeln!(
        report,
        "|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    )
    .unwrap();
    for info in inventory {
        let id = info.id;
        let expected = task903_stage52_vertical_bits(&info.vertical);
        let parsed_attr1 = parsed_rows.get(&id).copied().unwrap_or(0);
        let reference_attr1 = reference_rows.get(&id).copied().unwrap_or(0);
        let positive_attr1 = positive_rows.get(&id).copied().unwrap_or(0);
        let baseline_attr1 = baseline_rows.get(&id).copied().unwrap_or(0);
        writeln!(
            report,
            "| {} | {} | `{}` | {} | {} | {} | {} | {} | `{:#010x}` | `{:#010x}` | `{:#010x}` | `{:#010x}` |",
            id,
            refs.para_shape_ids.contains(&(id as u16)),
            info.vertical,
            expected,
            (parsed_attr1 >> 20) & 0x03,
            (reference_attr1 >> 20) & 0x03,
            (positive_attr1 >> 20) & 0x03,
            (baseline_attr1 >> 20) & 0x03,
            parsed_attr1,
            reference_attr1,
            positive_attr1,
            baseline_attr1
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("parashape_vertical_bits.md"), report)
        .expect("Stage52 vertical bits report 저장 실패");
}

fn task903_stage52_write_candidate_hwp(out_dir: &std::path::Path) -> (usize, Option<u32>, String) {
    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("Stage52 HWPX 로드 실패");
    let report = convert_hwpx_to_hwp_ir(core.document_mut());
    eprintln!("[#903 Stage52] adapter report: {:?}", report);

    let hwp_bytes =
        rhwp::serializer::serialize_hwp(core.document()).expect("Stage52 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "Stage52 산출물이 너무 작음: {} bytes",
        hwp_bytes.len()
    );
    let hash = task903_short_hash(&hwp_bytes);
    let out_path = out_dir.join("hwpx-h-01.hwp");
    std::fs::write(&out_path, &hwp_bytes).expect("Stage52 HWP 저장 실패");
    let reloaded_pages = DocumentCore::from_bytes(&hwp_bytes)
        .map(|core| core.page_count())
        .ok();
    eprintln!(
        "[#903 Stage52] generated {} bytes={} hash={} rhwp_pages={:?}",
        out_path.display(),
        hwp_bytes.len(),
        hash,
        reloaded_pages
    );
    (hwp_bytes.len(), reloaded_pages, hash)
}

fn task903_stage52_write_generation_report(
    out_dir: &std::path::Path,
    bytes: usize,
    pages: Option<u32>,
    hash: &str,
) {
    use std::fmt::Write as _;

    let mut report = String::new();
    writeln!(
        report,
        "# Task m100 #903 Stage 52 Vertical Attribute Mapping"
    )
    .unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 생성 파일").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| file | bytes | hash | rhwp reload |").unwrap();
    writeln!(report, "|---|---:|---|---|").unwrap();
    writeln!(
        report,
        "| `hwpx-h-01.hwp` | {} | `{}` | {} |",
        bytes,
        hash,
        pages
            .map(|v| format!("ok, pages={}", v))
            .unwrap_or_else(|| "failed".to_string())
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. 작업지시자 판정표").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| 파일 | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |"
    )
    .unwrap();
    writeln!(report, "|---|---|---|---|---|---|---|---|").unwrap();
    writeln!(
        report,
        "| `output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/hwpx-h-01.hwp` |  |  |  |  |  |  |  |"
    )
    .unwrap();

    std::fs::write(out_dir.join("stage52_generation.md"), report)
        .expect("Stage52 generation report 저장 실패");
}

#[test]
#[ignore]
fn task903_stage52_generate_vertical_attr_mapping_probe() {
    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping");
    std::fs::create_dir_all(out_dir).expect("Stage52 output dir 생성 실패");

    let inventory = task903_stage52_collect_para_pr_inventory();
    assert_eq!(inventory.len(), 85, "Stage52 paraPr 개수 불일치");

    let parsed_core =
        DocumentCore::from_bytes(&load_sample("hwpx-h-01.hwpx")).expect("Stage52 HWPX 로드 실패");
    let reference = task903_load_hwp_document("samples/hwpx/hancom-hwp/hwpx-h-01.hwp")
        .expect("Stage52 reference HWP 필요");
    let positive = task903_stage49_positive_document();
    let baseline = task903_stage49_baseline_document();

    task903_stage52_write_para_pr_inventory(out_dir, &inventory);
    task903_stage52_write_vertical_bits_report(
        out_dir,
        &inventory,
        parsed_core.document(),
        &reference,
        &positive,
        &baseline,
    );

    let center_ids = inventory
        .iter()
        .filter(|info| info.vertical == "CENTER")
        .map(|info| info.id)
        .collect::<Vec<_>>();
    assert_eq!(
        center_ids,
        vec![1, 3, 7, 11, 41, 44, 56, 62, 63, 71, 73],
        "Stage52 CENTER paraPr id 집합이 예상과 다름"
    );

    for info in &inventory {
        let ps = parsed_core
            .document()
            .doc_info
            .para_shapes
            .get(info.id)
            .unwrap_or_else(|| panic!("Stage52 para shape id {} 누락", info.id));
        assert_eq!(
            (ps.attr1 >> 20) & 0x03,
            task903_stage52_vertical_bits(&info.vertical),
            "Stage52 parsed ParaShape vertical bits mismatch id={}",
            info.id
        );
    }
}

#[test]
fn task903_stage52_export_hwpx_h_01_after_parser_fix() {
    let out_dir = std::path::Path::new("output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping");
    std::fs::create_dir_all(out_dir).expect("Stage52 output dir 생성 실패");
    let (bytes, pages, hash) = task903_stage52_write_candidate_hwp(out_dir);
    task903_stage52_write_generation_report(out_dir, bytes, pages, &hash);
    assert_eq!(pages, Some(9), "Stage52 rhwp 재로드 기준 9페이지 필요");
}

fn task903_stage53_current_impl_document(out_dir: &std::path::Path) -> Document {
    let hwpx_bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&hwpx_bytes).expect("Stage53 HWPX 로드 실패");
    let report = convert_hwpx_to_hwp_ir(core.document_mut());
    eprintln!("[#903 Stage53] adapter report: {:?}", report);

    let hwp_bytes =
        rhwp::serializer::serialize_hwp(core.document()).expect("Stage53 current HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "Stage53 current 산출물이 너무 작음: {} bytes",
        hwp_bytes.len()
    );

    std::fs::write(out_dir.join("00_current_impl_baseline.hwp"), &hwp_bytes)
        .expect("Stage53 current baseline 저장 실패");

    let compressed = task903_file_header_flags(&hwp_bytes) & 0x01 != 0;
    let mut doc = core.document().clone();
    doc.doc_info.raw_stream = task903_read_decompressed_docinfo(&hwp_bytes, compressed);
    doc.doc_info.raw_stream_dirty = false;
    for section_idx in 0..doc.sections.len() {
        let raw = task903_read_decompressed_section(&hwp_bytes, section_idx as u32, compressed)
            .unwrap_or_else(|| {
                panic!(
                    "Stage53 current section{} raw stream 읽기 실패",
                    section_idx
                )
            });
        doc.sections[section_idx].raw_stream = Some(raw);
    }
    task903_stage36_prepare_for_raw_bodytext(&mut doc);
    doc.doc_info.raw_stream_dirty = false;
    doc
}

fn task903_stage53_positive_document() -> Document {
    task903_stage47_success_document()
}

fn task903_stage53_patch_docinfo_tags(doc: &mut Document, positive: &Document, tags: &[u32]) {
    let target_raw = doc
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage53 target DocInfo raw_stream 필요");
    let positive_raw = positive
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage53 positive DocInfo raw_stream 필요");
    let patched = task903_graft_record_tags_by_index(target_raw, positive_raw, tags);
    doc.doc_info.raw_stream = Some(patched);
    doc.doc_info.raw_stream_dirty = false;
    doc.header.compressed = true;
    doc.header.flags |= 0x01;
    doc.header.raw_data = None;
}

fn task903_stage53_patch_bindata(doc: &mut Document, positive: &Document) {
    doc.doc_info.bin_data_list = positive.doc_info.bin_data_list.clone();
    task903_stage53_patch_docinfo_tags(doc, positive, &[18]);
}

fn task903_stage53_patch_ctrl_header(doc: &mut Document, positive: &Document) {
    task903_patch_section0_tags(doc, positive, &[71]);
}

fn task903_stage53_patch_parashape_vertical_bits(doc: &mut Document, positive: &Document) {
    task903_stage51_patch_parashape_attr1_mask(doc, positive, 0x0030_0000);
}

fn task903_stage53_patch_parashape_full(doc: &mut Document, positive: &Document) {
    task903_copy_para_shape_model_fields(doc, positive);
    task903_stage53_patch_docinfo_tags(doc, positive, &[25]);
}

fn task903_stage53_write_probe(
    out_dir: &std::path::Path,
    name: &str,
    doc: &Document,
) -> (usize, Option<u32>, String) {
    let hwp_bytes = rhwp::serializer::serialize_hwp(doc).expect("Stage53 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "{} 산출물이 너무 작음: {} bytes",
        name,
        hwp_bytes.len()
    );
    let hash = task903_short_hash(&hwp_bytes);
    let out_path = out_dir.join(name);
    std::fs::write(&out_path, &hwp_bytes).expect("Stage53 HWP 저장 실패");
    let reloaded_pages = DocumentCore::from_bytes(&hwp_bytes)
        .map(|core| core.page_count())
        .ok();
    eprintln!(
        "[#903 Stage53] generated {} bytes={} hash={} rhwp_pages={:?}",
        out_path.display(),
        hwp_bytes.len(),
        hash,
        reloaded_pages
    );
    (hwp_bytes.len(), reloaded_pages, hash)
}

fn task903_stage53_docinfo_tag_counts(doc: &Document) -> std::collections::BTreeMap<u32, usize> {
    doc.doc_info
        .raw_stream
        .as_ref()
        .map(|raw| task903_stage49_count_records_by_tag(raw))
        .unwrap_or_default()
}

fn task903_stage53_write_docinfo_report(
    out_dir: &std::path::Path,
    current: &Document,
    positive: &Document,
) {
    use std::collections::BTreeSet;
    use std::fmt::Write as _;

    let current_raw = current
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage53 current DocInfo raw_stream 필요");
    let positive_raw = positive
        .doc_info
        .raw_stream
        .as_ref()
        .expect("Stage53 positive DocInfo raw_stream 필요");
    let current_records = task903_parse_records(current_raw);
    let positive_records = task903_parse_records(positive_raw);
    let current_counts = task903_stage53_docinfo_tag_counts(current);
    let positive_counts = task903_stage53_docinfo_tag_counts(positive);
    let all_tags: BTreeSet<u32> = current_counts
        .keys()
        .chain(positive_counts.keys())
        .copied()
        .collect();

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 53 DocInfo Gap Report").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 기준").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "- current: Stage52 parser/adapter implementation output"
    )
    .unwrap();
    writeln!(
        report,
        "- positive: `output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp`"
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. Summary").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| item | current | positive |").unwrap();
    writeln!(report, "|---|---:|---:|").unwrap();
    writeln!(
        report,
        "| section_count | {} | {} |",
        current.doc_properties.section_count, positive.doc_properties.section_count
    )
    .unwrap();
    writeln!(
        report,
        "| bin_data model count | {} | {} |",
        current.doc_info.bin_data_list.len(),
        positive.doc_info.bin_data_list.len()
    )
    .unwrap();
    writeln!(
        report,
        "| bin_data_content count | {} | {} |",
        current.bin_data_content.len(),
        positive.bin_data_content.len()
    )
    .unwrap();
    writeln!(
        report,
        "| para_shape model count | {} | {} |",
        current.doc_info.para_shapes.len(),
        positive.doc_info.para_shapes.len()
    )
    .unwrap();
    writeln!(
        report,
        "| DocInfo raw bytes | {} | {} |",
        current_raw.len(),
        positive_raw.len()
    )
    .unwrap();
    writeln!(
        report,
        "| DocInfo record count | {} | {} |",
        current_records.len(),
        positive_records.len()
    )
    .unwrap();
    writeln!(
        report,
        "| DocInfo hash | `{}` | `{}` |",
        task903_short_hash(current_raw),
        task903_short_hash(positive_raw)
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 3. DocInfo record counts").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| tag | name | current | positive |").unwrap();
    writeln!(report, "|---:|---|---:|---:|").unwrap();
    for tag in all_tags {
        writeln!(
            report,
            "| {} | {} | {} | {} |",
            tag,
            task903_record_tag_name(tag),
            current_counts.get(&tag).copied().unwrap_or(0),
            positive_counts.get(&tag).copied().unwrap_or(0)
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 4. BIN_DATA model detail").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | current raw | positive raw | current model | positive model |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---|---|---|").unwrap();
    for idx in 0..current
        .doc_info
        .bin_data_list
        .len()
        .max(positive.doc_info.bin_data_list.len())
    {
        let current_bin = current.doc_info.bin_data_list.get(idx);
        let positive_bin = positive.doc_info.bin_data_list.get(idx);
        writeln!(
            report,
            "| {} | `{}` | `{}` | `{}` | `{}` |",
            idx + 1,
            current_bin
                .and_then(|bin| bin.raw_data.as_deref())
                .map(task903_hex_bytes)
                .unwrap_or_else(|| "-".to_string()),
            positive_bin
                .and_then(|bin| bin.raw_data.as_deref())
                .map(task903_hex_bytes)
                .unwrap_or_else(|| "-".to_string()),
            current_bin
                .map(|bin| format!(
                    "attr={:#x}, storage_id={}, ext={:?}, type={:?}, compression={:?}, status={:?}",
                    bin.attr,
                    bin.storage_id,
                    bin.extension,
                    bin.data_type,
                    bin.compression,
                    bin.status
                ))
                .unwrap_or_else(|| "-".to_string()),
            positive_bin
                .map(|bin| format!(
                    "attr={:#x}, storage_id={}, ext={:?}, type={:?}, compression={:?}, status={:?}",
                    bin.attr,
                    bin.storage_id,
                    bin.extension,
                    bin.data_type,
                    bin.compression,
                    bin.status
                ))
                .unwrap_or_else(|| "-".to_string())
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("current_vs_positive_docinfo.md"), report)
        .expect("Stage53 DocInfo report 저장 실패");
}

fn task903_stage53_write_section0_report(
    out_dir: &std::path::Path,
    current: &Document,
    positive: &Document,
) {
    use std::collections::BTreeMap;
    use std::fmt::Write as _;

    let current_raw = current.sections[0]
        .raw_stream
        .as_ref()
        .expect("Stage53 current section0 raw_stream 필요");
    let positive_raw = positive.sections[0]
        .raw_stream
        .as_ref()
        .expect("Stage53 positive section0 raw_stream 필요");
    let current_records = task903_parse_records(current_raw);
    let positive_records = task903_parse_records(positive_raw);
    let mut diff_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut focus_rows = Vec::new();
    for idx in 0..current_records.len().min(positive_records.len()) {
        let current_record = &current_records[idx];
        let positive_record = &positive_records[idx];
        let current_payload = &current_raw
            [current_record.data_offset..current_record.data_offset + current_record.size];
        let positive_payload = &positive_raw
            [positive_record.data_offset..positive_record.data_offset + positive_record.size];
        if current_record.tag != positive_record.tag
            || current_record.level != positive_record.level
            || current_payload != positive_payload
        {
            *diff_counts.entry(positive_record.tag).or_default() += 1;
            if [66, 67, 68, 69, 71, 72, 77].contains(&positive_record.tag) {
                focus_rows.push((
                    idx,
                    positive_record.tag,
                    current_record.tag,
                    positive_record.size,
                    current_record.size,
                    task903_first_diff(positive_payload, current_payload),
                ));
            }
        }
    }

    let mut report = String::new();
    writeln!(report, "# Task m100 #903 Stage 53 Section0 Gap Report").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. Summary").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| item | current | positive |").unwrap();
    writeln!(report, "|---|---:|---:|").unwrap();
    writeln!(
        report,
        "| section0 bytes | {} | {} |",
        current_raw.len(),
        positive_raw.len()
    )
    .unwrap();
    writeln!(
        report,
        "| section0 records | {} | {} |",
        current_records.len(),
        positive_records.len()
    )
    .unwrap();
    writeln!(
        report,
        "| comparable differing records | {} | {} |",
        diff_counts.values().sum::<usize>(),
        diff_counts.values().sum::<usize>()
    )
    .unwrap();
    writeln!(
        report,
        "| section0 hash | `{}` | `{}` |",
        task903_short_hash(current_raw),
        task903_short_hash(positive_raw)
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. Diff counts by tag").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| tag | name | differing records |").unwrap();
    writeln!(report, "|---:|---|---:|").unwrap();
    for (tag, count) in &diff_counts {
        writeln!(
            report,
            "| {} | {} | {} |",
            tag,
            task903_record_tag_name(*tag),
            count
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 3. Focus rows").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| idx | positive tag | current tag | positive size | current size | first diff |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---|---:|---:|---|").unwrap();
    for (idx, positive_tag, current_tag, positive_size, current_size, first_diff) in focus_rows {
        writeln!(
            report,
            "| {} | {}({}) | {}({}) | {} | {} | {} |",
            idx,
            task903_record_tag_name(positive_tag),
            positive_tag,
            task903_record_tag_name(current_tag),
            current_tag,
            positive_size,
            current_size,
            first_diff
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("current_vs_positive_section0.md"), report)
        .expect("Stage53 section0 report 저장 실패");
}

fn task903_stage53_write_vertical_bits_report(
    out_dir: &std::path::Path,
    current: &Document,
    reference: &Document,
    positive: &Document,
) {
    use std::collections::BTreeMap;
    use std::fmt::Write as _;

    let inventory = task903_stage52_collect_para_pr_inventory();
    let current_rows: BTreeMap<usize, u32> = task903_stage52_docinfo_parashape_attr1(current)
        .into_iter()
        .collect();
    let reference_rows: BTreeMap<usize, u32> = task903_stage52_docinfo_parashape_attr1(reference)
        .into_iter()
        .collect();
    let positive_rows: BTreeMap<usize, u32> = task903_stage52_docinfo_parashape_attr1(positive)
        .into_iter()
        .collect();
    let refs = task903_stage49_ref_stats(current);

    let mut report = String::new();
    writeln!(
        report,
        "# Task m100 #903 Stage 53 Current ParaShape Vertical Bits"
    )
    .unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| id | referenced | HWPX vertical | expected bits | current bits | reference bits | positive bits | current attr1 | reference attr1 | positive attr1 |"
    )
    .unwrap();
    writeln!(report, "|---:|---|---|---:|---:|---:|---:|---:|---:|---:|").unwrap();
    for info in inventory {
        let id = info.id;
        let expected = task903_stage52_vertical_bits(&info.vertical);
        let current_attr1 = current_rows.get(&id).copied().unwrap_or(0);
        let reference_attr1 = reference_rows.get(&id).copied().unwrap_or(0);
        let positive_attr1 = positive_rows.get(&id).copied().unwrap_or(0);
        writeln!(
            report,
            "| {} | {} | `{}` | {} | {} | {} | {} | `{:#010x}` | `{:#010x}` | `{:#010x}` |",
            id,
            refs.para_shape_ids.contains(&(id as u16)),
            info.vertical,
            expected,
            (current_attr1 >> 20) & 0x03,
            (reference_attr1 >> 20) & 0x03,
            (positive_attr1 >> 20) & 0x03,
            current_attr1,
            reference_attr1,
            positive_attr1
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("current_parashape_vertical_bits.md"), report)
        .expect("Stage53 vertical bits report 저장 실패");
}

fn task903_stage53_write_generation_report(
    out_dir: &std::path::Path,
    rows: &[(&str, usize, Option<u32>, String)],
) {
    use std::fmt::Write as _;

    let mut report = String::new();
    writeln!(
        report,
        "# Task m100 #903 Stage 53 Current Implementation Gap Probe"
    )
    .unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 생성 파일").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| variant | bytes | hash | rhwp reload |").unwrap();
    writeln!(report, "|---|---:|---|---|").unwrap();
    for (name, bytes, pages, hash) in rows {
        writeln!(
            report,
            "| `{}` | {} | `{}` | {} |",
            name,
            bytes,
            hash,
            pages
                .map(|v| format!("ok, pages={}", v))
                .unwrap_or_else(|| "failed".to_string())
        )
        .unwrap();
    }
    writeln!(report).unwrap();

    writeln!(report, "## 2. 작업지시자 판정표").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |"
    )
    .unwrap();
    writeln!(report, "|---|---|---|---|---|---|---|---|").unwrap();
    for (name, _, _, _) in rows {
        writeln!(
            report,
            "| {} |  |  |  |  |  |  |  |",
            name.trim_end_matches(".hwp")
        )
        .unwrap();
    }

    std::fs::write(out_dir.join("stage53_generation.md"), report)
        .expect("Stage53 generation report 저장 실패");
}

#[test]
#[ignore]
fn task903_stage53_generate_current_impl_gap_probe() {
    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe");
    std::fs::create_dir_all(out_dir).expect("Stage53 output dir 생성 실패");

    let current = task903_stage53_current_impl_document(out_dir);
    let positive = task903_stage53_positive_document();
    let reference = task903_load_hwp_document("samples/hwpx/hancom-hwp/hwpx-h-01.hwp")
        .expect("Stage53 reference HWP 필요");

    task903_stage53_write_docinfo_report(out_dir, &current, &positive);
    task903_stage53_write_section0_report(out_dir, &current, &positive);
    task903_stage53_write_vertical_bits_report(out_dir, &current, &reference, &positive);

    type PatchFn = fn(&mut Document, &Document);
    let variants: &[(&str, PatchFn)] = &[
        ("01_current_plus_bindata.hwp", |doc, positive| {
            task903_stage53_patch_bindata(doc, positive);
        }),
        ("02_current_plus_ctrl_header.hwp", |doc, positive| {
            task903_stage53_patch_ctrl_header(doc, positive);
        }),
        (
            "03_current_plus_bindata_ctrl_header.hwp",
            |doc, positive| {
                task903_stage53_patch_bindata(doc, positive);
                task903_stage53_patch_ctrl_header(doc, positive);
            },
        ),
        (
            "04_current_plus_parashape_vertical_bits.hwp",
            |doc, positive| {
                task903_stage53_patch_parashape_vertical_bits(doc, positive);
            },
        ),
        (
            "05_current_plus_bindata_ctrl_header_vertical_bits.hwp",
            |doc, positive| {
                task903_stage53_patch_bindata(doc, positive);
                task903_stage53_patch_ctrl_header(doc, positive);
                task903_stage53_patch_parashape_vertical_bits(doc, positive);
            },
        ),
        ("06_current_plus_parashape_full.hwp", |doc, positive| {
            task903_stage53_patch_parashape_full(doc, positive);
        }),
        (
            "07_current_plus_bindata_ctrl_header_parashape_full.hwp",
            |doc, positive| {
                task903_stage53_patch_bindata(doc, positive);
                task903_stage53_patch_ctrl_header(doc, positive);
                task903_stage53_patch_parashape_full(doc, positive);
            },
        ),
    ];

    let mut rows = Vec::new();
    for (name, patch) in variants {
        let mut doc = current.clone();
        patch(&mut doc, &positive);
        let (bytes, pages, hash) = task903_stage53_write_probe(out_dir, name, &doc);
        rows.push((*name, bytes, pages, hash));
    }
    task903_stage53_write_generation_report(out_dir, &rows);
}

fn task903_stage54_candidate_document(
    out_dir: &std::path::Path,
) -> (Document, usize, Option<u32>, String) {
    let hwpx_bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&hwpx_bytes).expect("Stage54 HWPX 로드 실패");
    let report = convert_hwpx_to_hwp_ir(core.document_mut());
    eprintln!("[#903 Stage54] adapter report: {:?}", report);

    let hwp_bytes =
        rhwp::serializer::serialize_hwp(core.document()).expect("Stage54 HWP 직렬화 실패");
    assert!(
        hwp_bytes.len() > 300_000,
        "Stage54 산출물이 너무 작음: {} bytes",
        hwp_bytes.len()
    );

    let out_path = out_dir.join("hwpx-h-01.hwp");
    std::fs::write(&out_path, &hwp_bytes).expect("Stage54 HWP 저장 실패");

    let reloaded_pages = DocumentCore::from_bytes(&hwp_bytes)
        .map(|core| core.page_count())
        .ok();
    let hash = task903_short_hash(&hwp_bytes);
    eprintln!(
        "[#903 Stage54] generated {} bytes={} hash={} rhwp_pages={:?}",
        out_path.display(),
        hwp_bytes.len(),
        hash,
        reloaded_pages
    );

    let compressed = task903_file_header_flags(&hwp_bytes) & 0x01 != 0;
    let mut doc = core.document().clone();
    doc.doc_info.raw_stream = task903_read_decompressed_docinfo(&hwp_bytes, compressed);
    doc.doc_info.raw_stream_dirty = false;
    for section_idx in 0..doc.sections.len() {
        let raw = task903_read_decompressed_section(&hwp_bytes, section_idx as u32, compressed)
            .unwrap_or_else(|| {
                panic!(
                    "Stage54 candidate section{} raw stream 읽기 실패",
                    section_idx
                )
            });
        doc.sections[section_idx].raw_stream = Some(raw);
    }
    task903_stage36_prepare_for_raw_bodytext(&mut doc);
    doc.doc_info.raw_stream_dirty = false;

    (doc, hwp_bytes.len(), reloaded_pages, hash)
}

fn task903_stage54_write_generation_report(
    out_dir: &std::path::Path,
    bytes: usize,
    pages: Option<u32>,
    hash: &str,
) {
    use std::fmt::Write as _;

    let mut report = String::new();
    writeln!(
        report,
        "# Task m100 #903 Stage 54 Minimal Implementation Candidate"
    )
    .unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## 1. 생성 파일").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| file | bytes | hash | rhwp reload |").unwrap();
    writeln!(report, "|---|---:|---|---|").unwrap();
    writeln!(
        report,
        "| `hwpx-h-01.hwp` | {} | `{}` | {} |",
        bytes,
        hash,
        pages
            .map(|v| format!("ok, pages={}", v))
            .unwrap_or_else(|| "failed".to_string())
    )
    .unwrap();
    writeln!(report).unwrap();

    writeln!(report, "## 2. 작업지시자 판정표").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "| 파일 | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |"
    )
    .unwrap();
    writeln!(report, "|---|---|---|---|---|---|---|---|").unwrap();
    writeln!(
        report,
        "| `output/poc/hwpx2hwp/task903/stage54_minimal_impl_candidate/hwpx-h-01.hwp` |  |  |  |  |  |  |  |"
    )
    .unwrap();

    std::fs::write(out_dir.join("stage54_generation.md"), report)
        .expect("Stage54 generation report 저장 실패");
}

#[test]
#[ignore]
fn task903_stage54_generate_minimal_impl_candidate() {
    let out_dir =
        std::path::Path::new("output/poc/hwpx2hwp/task903/stage54_minimal_impl_candidate");
    std::fs::create_dir_all(out_dir).expect("Stage54 output dir 생성 실패");

    let (candidate, bytes, pages, hash) = task903_stage54_candidate_document(out_dir);
    let positive = task903_stage53_positive_document();

    task903_stage53_write_docinfo_report(out_dir, &candidate, &positive);
    task903_stage53_write_section0_report(out_dir, &candidate, &positive);
    task903_stage54_write_generation_report(out_dir, bytes, pages, &hash);

    assert_eq!(pages, Some(9), "Stage54 rhwp 재로드 기준 9페이지 필요");
}
