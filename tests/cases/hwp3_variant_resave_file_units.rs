//! HWP3→HWP5 변환본에서 편집한 구역을 다시 저장해도 파일 단위 값이 유지된다.
//!
//! 파서는 변환본(`Document::is_hwp3_variant`)을 열 때 문단 모양의 문단 위/아래 간격을 반으로
//! 줄이고, 본문 LINE_SEG vertical_pos 에서 누적 문단 위 간격을 뺀다(`parser/mod.rs`).
//! 편집 없는 저장은 원본 바이트를 그대로 쓰므로 무사하다. 그러나 다시 쓰는 구역·문단 모양에는
//! 보정된 값이 그대로 실려, 다시 열 때 같은 보정이 한 번 더 걸렸다. 그래서 저장할 때마다
//! vpos 가 줄었고(`HWP5-nopassword-123456.hwp`: 문단 70 은 저장마다 −852HU), 고친 문단 모양은
//! 간격이 반씩 줄었다(284/424 → 142/212). 한컴은 파일의 vpos·간격을 그대로 읽는다.

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/HWP5-nopassword-123456.hwp";

fn open(bytes: &[u8]) -> HwpDocument {
    HwpDocument::from_bytes(bytes).unwrap()
}

fn sample() -> HwpDocument {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = open(&std::fs::read(path).unwrap());
    assert!(doc.document().is_hwp3_variant, "변환본 판정이 전제다");
    doc
}

fn body_vpos(d: &HwpDocument) -> Vec<i32> {
    d.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.line_segs.iter().map(|l| l.vertical_pos))
        .collect()
}

/// 모든 구역을 원본 스트림 없이 모델에서 다시 쓰게 한다 — 편집한 구역이 타는 저장 경로다.
/// (내용이 그대로인 편집은 봉인이 맞아 원본 스트림을 재사용하므로 이 경로를 타지 않는다.)
fn resave_all_sections(d: &mut HwpDocument) -> HwpDocument {
    for s in &mut d.document_mut().sections {
        s.raw_stream = None;
    }
    open(&d.export_hwp_native().unwrap())
}

#[test]
fn rewritten_section_vpos_survives_repeated_saves() {
    let mut doc = sample();
    let before = body_vpos(&doc);
    let mut once = resave_all_sections(&mut doc);
    assert_eq!(body_vpos(&once), before, "1회 저장");
    let twice = resave_all_sections(&mut once);
    assert_eq!(body_vpos(&twice), before, "2회 저장");
}

#[test]
fn edited_para_shape_keeps_spacing() {
    let mut doc = sample();
    let pi = doc.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| {
            let ps = &doc.document().doc_info.para_shapes[p.para_shape_id as usize];
            ps.spacing_before > 0 && ps.spacing_after > 0
        })
        .expect("문단 위/아래 간격이 있는 문단");
    doc.apply_para_format_native(0, pi, r#"{"alignment":"center"}"#)
        .unwrap();
    let id = doc.document().sections[0].paragraphs[pi].para_shape_id as usize;
    let want = {
        let ps = &doc.document().doc_info.para_shapes[id];
        (ps.spacing_before, ps.spacing_after)
    };
    let back = open(&doc.export_hwp_native().unwrap());
    let ps = &back.document().doc_info.para_shapes[id];
    assert_eq!(
        (ps.spacing_before, ps.spacing_after),
        want,
        "새 문단 모양의 간격이 저장·다시 열기로 반이 되면 안 된다"
    );
}
