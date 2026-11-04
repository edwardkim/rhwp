//! 고정형: 독립 한컴 Print 5쪽 기관명 두 줄의 원점·폭을 ±2px로 보호한다.
//! 주석 갱신: 2026-11-04 03:14:58 KST; 신규 검사.
//! Native 98.91953%, fresh WASM 99.09245%에서 review·overlay 직접 판독 완료.
//! 측정 코드: 3710501d878bde7432a923c104f23d76bc7dd61a (96 DPI, Print 2024).
//! 원본: samples/issue6551/113424_evaluation_guideline.hwpx.
//! 독립 기준: pdf/113424_evaluation_guideline-2024.pdf 5쪽 ExpoM-HM span.
//! PDF SHA-256: f2db782bb42cabd2b96cd8985415554803ad7d4322904177a9e1bc903115bf6d.
//! 원본 전진 보존을 지정했으므로 표시용 Noto의 0.92em으로 상자를 수축하면 실패한다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::renderer::font_environment::FontEnvironment;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::DocumentCore;

#[test]
fn display_program_keeps_the_print_source_nameplate_origin_and_width() {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6551/113424_evaluation_guideline.hwpx"
    ))
    .unwrap();
    core.set_dpi(96.0);
    core.set_hangul2024_compat(true);
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"source-frame-8533","preserveSourceMetrics":true,"substitutions":{"휴먼엑스포":"NotoSansKR-Regular"}}"#,
    ).unwrap())).unwrap();
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[
            include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf").to_vec(),
        ])
        .unwrap();
    // 실제 Native 출력이 하는 프로그램 준비를 생략하면 폭 오염을 놓친다.
    renderer.prepare_native_otf_fonts(&mut core).unwrap();
    let tree = core.build_page_render_tree(4).unwrap();
    fn collect(node: &RenderNode, text: &str, out: &mut Vec<(f64, f64)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.display_or_text() == text {
                out.push((node.bbox.x, node.bbox.width));
            }
        }
        for child in &node.children {
            collect(child, text, out);
        }
    }
    // PDF pt 좌표에 96/72를 적용했다. 현재 렌더러의 계산값을 기대값으로 복사하지 않는다.
    for (text, x, width) in [
        ("국민권익위원회", 323.51999918619794, 233.2794392903646),
        ("운영지원과", 366.7200113932292, 146.72208658854166),
    ] {
        let mut boxes = Vec::new();
        collect(&tree.root, text, &mut boxes);
        assert_eq!(boxes.len(), 1, "같은 쪽의 기관명 누락·중복: {text}");
        let (actual_x, actual_width) = boxes[0];
        assert!(
            (actual_x - x).abs() <= 2.0,
            "독립 Print 원점: {text}: {actual_x} vs {x}"
        );
        assert!(
            (actual_width - width).abs() <= 2.0,
            "독립 Print 폭: {text}: {actual_width} vs {width}"
        );
    }
}
