//! 독립 Print 46쪽의 저장 줄말 구분 공백은 가시 글자의 분배 예산을 차지하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    model::control::Control,
    renderer::{hwpunit_to_px, DEFAULT_DPI},
    DocumentCore,
};
use serde_json::Value;
#[test]
fn saved_justified_visible_tail_reaches_its_source_frame_without_losing_the_separator() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6031/3249937_asset_management_rules.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    let table = match &core.document().sections[0].paragraphs[569].controls[0] {
        Control::Table(t) => t,
        _ => panic!("감면 및 징수유예 절차도"),
    };
    let layout: Value =
        serde_json::from_str(&core.get_page_text_layout_native(45).unwrap()).unwrap();
    for cell_index in [16usize, 24] {
        let para = &table.cells[cell_index].paragraphs[1];
        assert!(para.text.starts_with("농업피해확인대장 발급"));
        let first_line = &para.line_segs[0];
        let next = &para.line_segs[1];
        assert_eq!(
            next.text_start, 9,
            "8가시 글자 뒤 구분 공백도 다음 줄 소속으로 바꾸지 않는다"
        );
        let runs: Vec<_> = layout["runs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                r["parentParaIdx"].as_u64() == Some(569)
                    && r["cellIdx"].as_u64() == Some(cell_index as u64)
                    && r["cellParaIdx"].as_u64() == Some(1)
            })
            .collect();
        let painted: String = runs.iter().map(|r| r["text"].as_str().unwrap()).collect();
        assert_eq!(painted, para.text, "원문 순서·구분 공백·후속 줄 내용 보존");
        let head = runs
            .iter()
            .find(|r| r["charStart"].as_u64() == Some(0))
            .unwrap();
        let tail = runs
            .iter()
            .find(|r| r["charStart"].as_u64() == Some(7))
            .unwrap();
        assert_eq!(tail["text"].as_str(), Some("장 "));
        let source_style = &core.document().doc_info.char_shapes[para
            .char_shapes
            .iter()
            .rev()
            .find(|s| s.start_pos <= 7)
            .unwrap()
            .char_shape_id
            as usize];
        assert_eq!(source_style.spacings[0], 0, "마지막 전각 글자는 원본 자간0");
        let last_advance = hwpunit_to_px(source_style.base_size as i32, DEFAULT_DPI)
            * f64::from(source_style.relative_sizes[0])
            / 100.0
            * f64::from(source_style.ratios[0])
            / 100.0;
        let frame_width = hwpunit_to_px(first_line.segment_width, DEFAULT_DPI);
        let visible_width =
            tail["x"].as_f64().unwrap() + last_advance - head["x"].as_f64().unwrap();
        assert!((visible_width/frame_width-1.0).abs()<0.01,"가시 마지막 전각 글자의 전진 끝은 원본 줄 프레임을 채운다: visible={visible_width}, source={frame_width}");
    }
}
