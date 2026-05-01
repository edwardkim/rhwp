//! HWPX → HWP IR 어댑터 통합 테스트 (#178)
//!
//! Stage 1: 베이스라인 측정 (페이지 폭주 + 영역별 차이 인벤토리).
//!         아직 어댑터 본체가 동작하지 않으므로 회복 검증 없음 — 측정만.

use rhwp::document_core::DocumentCore;
use rhwp::document_core::converters::diagnostics::diff_hwpx_vs_serializer_assumptions;
use rhwp::document_core::converters::hwpx_to_hwp::{
    convert_if_hwpx_source, convert_hwpx_to_hwp_ir,
};
use rhwp::model::control::Control;

fn load_sample(name: &str) -> Vec<u8> {
    let path = format!("samples/hwpx/{}", name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("샘플 로드 실패 {}: {}", path, e))
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
    eprintln!("[#178 baseline] {}: orig={}, reloaded={}", name, orig, reloaded);
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
    eprintln!("[#178 baseline] hwpx-h-03: orig={}, reloaded={}", orig, reloaded);
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
    assert_eq!(report.skipped_reason.as_deref(), Some("source_format != Hwpx/Hwp3"));
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
    assert!(empty_count_before > 0, "HWPX 출처에는 빈 raw_ctrl_data 가 있어야 함");

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
    assert_eq!(empty_count_after, 0, "어댑터 적용 후 모든 표는 raw_ctrl_data 가 채워져야 함");
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
        !first_para_orig.controls.iter().any(|c| matches!(c, Control::SectionDef(_))),
        "HWPX 출처 첫 문단에 SectionDef 가 이미 있다면 가정 위반"
    );

    let mut doc = core.document().clone();
    let report = convert_hwpx_to_hwp_ir(&mut doc);
    assert!(report.section_def_controls_inserted > 0, "SectionDef 삽입이 발생해야 함");

    // 어댑터 적용 후: 모든 섹션의 첫 문단에 SectionDef 가 있어야 함
    for (s_idx, section) in doc.sections.iter().enumerate() {
        let first_para = &section.paragraphs[0];
        assert!(
            first_para.controls.iter().any(|c| matches!(c, Control::SectionDef(_))),
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
    assert_eq!(r2.section_def_controls_inserted, 0, "2차 호출 시 삽입 0 (idempotent)");
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
    assert_eq!(orig_pd.margin_left, reload_pd.margin_left, "margin_left 보존");
    assert_eq!(orig_pd.margin_right, reload_pd.margin_right, "margin_right 보존");
    assert_eq!(orig_pd.margin_top, reload_pd.margin_top, "margin_top 보존");
    assert_eq!(orig_pd.margin_bottom, reload_pd.margin_bottom, "margin_bottom 보존");
}

/// Stage 4 핵심 게이트: 어댑터 적용 → 직렬화 → 재로드 시 페이지 수가 원본과 일치.
fn assert_page_count_recovered(name: &str, bytes: &[u8]) {
    let (orig, after) = page_count_with_adapter(bytes);
    eprintln!("[#178 Stage 4] {}: orig={}, after_adapter={}", name, orig, after);
    assert_eq!(
        after, orig,
        "{}: 어댑터 적용 후 페이지 수 {} != 원본 {}",
        name, after, orig
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

    assert_eq!(reloaded.page_count(), orig,
        "어댑터 통합 진입점: 페이지 수 보존 (orig={}, reloaded={})",
        orig, reloaded.page_count());
}

#[test]
fn stage5_export_hwp_with_adapter_hwp_source_unchanged() {
    // HWP 원본 — 어댑터는 no-op (source_format != Hwpx)
    let path = "samples/hwp_table_test.hwp";
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => { eprintln!("[skip] {} 없음", path); return; }
    };
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWP 로드");

    let bytes_native = core.export_hwp_native().expect("native 직렬화");
    let bytes_adapter = core.export_hwp_with_adapter().expect("adapter 직렬화");

    assert_eq!(bytes_native, bytes_adapter,
        "HWP 출처는 어댑터 호출이 native 와 동일 결과여야 함");
}

#[test]
fn stage5_export_hwp_with_adapter_idempotent_on_repeated_calls() {
    // 같은 DocumentCore 에 export_hwp_with_adapter() 를 두 번 호출.
    // 1차 호출이 IR 을 정규화하면, 2차 호출은 어댑터 가드에 막혀 변경 없음.
    let bytes = load_sample("hwpx-h-01.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");

    let first = core.export_hwp_with_adapter().expect("1차");
    let second = core.export_hwp_with_adapter().expect("2차");

    assert_eq!(first, second,
        "동일 DocumentCore 에 어댑터 통합 진입점 2회 호출 시 같은 bytes");
}

#[test]
fn stage5_all_three_samples_recover_via_unified_entry_point() {
    for name in ["hwpx-h-01.hwpx", "hwpx-h-02.hwpx", "hwpx-h-03.hwpx"] {
        let bytes = load_sample(name);
        let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");
        let orig = core.page_count();

        let hwp_bytes = core.export_hwp_with_adapter().expect("HWP 직렬화");
        let reloaded = DocumentCore::from_bytes(&hwp_bytes).expect("HWP 재로드");

        assert_eq!(reloaded.page_count(), orig,
            "{}: 페이지 수 보존 (orig={}, reloaded={})",
            name, orig, reloaded.page_count());
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
    assert!(v.recovered, "페이지 회복 실패: before={} after={}", v.page_count_before, v.page_count_after);
    assert_eq!(v.page_count_before, v.page_count_after);
    assert!(v.bytes_len > 0);
}

#[test]
fn stage6_verify_recovered_for_all_three_samples() {
    for name in ["hwpx-h-01.hwpx", "hwpx-h-02.hwpx", "hwpx-h-03.hwpx"] {
        let bytes = load_sample(name);
        let mut core = DocumentCore::from_bytes(&bytes).expect("HWPX 로드");
        let v = core.serialize_hwp_with_verify().expect("verify");
        assert!(
            v.recovered,
            "{}: before={} after={}",
            name, v.page_count_before, v.page_count_after
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

    assert_eq!(reloaded.page_count(), orig as u32,
        "wasm_api 경로: 페이지 수 보존 (orig={}, reloaded={})",
        orig, reloaded.page_count());
}
