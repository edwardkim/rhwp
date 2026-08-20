//! [Issue #5725/#5724] `Contents` 만 가진 OLE 의 페이로드 구제.
//!
//! OLE 폴백 사다리는 `raw_contents` 를 레거시 차트 전용 입력으로만 해석했다.
//! 차트가 아니면 EMF/WMF **미리보기** 폴백으로 내려가는데(#5582), 이 두 갈래는
//! 미리보기가 아예 없어(수식 OLE 의 OlePres000 은 28바이트 스텁) 자리표시로
//! 끝났다. 10k 코퍼스 실측 — 수식 8문서/39개체, WMF 8문서/11개체, BMP 1문서.
//!
//! - #5725: `Hwp 5.0 Equation Editor(HwpEq5x)` 봉투 → 기존 수식 렌더러로 배선.
//! - #5724: placeable/표준 WMF·EMF·비트맵 `CONTENTS` → 기존 재생기로 배선.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::parser::ole_container::{
    parse_equation_contents_script, parse_ole_container, raw_contents_is_emf, raw_contents_is_wmf,
};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const EQ_SAMPLE: &str = "samples/issue5725/2921145_equation_ole.hwpx";
const WMF_SAMPLE: &str = "samples/issue5724/2689441_wmf_contents_ole.hwp";

fn sample_bytes(rel: &str) -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(rel))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn ole_placeholder_labels(root: &RenderNode) -> Vec<String> {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Placeholder(p) if p.label.contains("OLE") => Some(p.label.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn issue_5725_equation_envelope_parses_script() {
    // 실물 봉투 (2921145 BinData/ole1.ole)
    let file = std::fs::File::open(Path::new(env!("CARGO_MANIFEST_DIR")).join(EQ_SAMPLE))
        .expect("open eq sample");
    let mut zip = zip::ZipArchive::new(file).expect("zip");
    let mut ole_bytes = Vec::new();
    zip.by_name("BinData/ole1.ole")
        .expect("ole1")
        .read_to_end(&mut ole_bytes)
        .expect("read");
    let container = parse_ole_container(&ole_bytes).expect("container");
    let raw = container.raw_contents.as_deref().expect("Contents");
    let script = parse_equation_contents_script(raw).expect("수식 스크립트 봉투");
    assert!(
        script.contains("\\OVER") && script.contains("0.75"),
        "정원산출식 스크립트가 나와야 한다: {script:?}"
    );

    // 수식 봉투가 아닌 바이트는 None — 차트/메타파일 경로를 침범하지 않는다.
    assert!(parse_equation_contents_script(b"not an equation").is_none());
    assert!(parse_equation_contents_script(&[]).is_none());
}

#[test]
fn issue_5724_metafile_magic_detection() {
    // placeable WMF (2689441 CONTENTS 선두 실측)
    let mut placeable = vec![0xD7, 0xCD, 0xC6, 0x9A];
    placeable.resize(32, 0);
    assert!(raw_contents_is_wmf(&placeable));
    // 표준 WMF
    let mut standard = Vec::new();
    standard.extend_from_slice(&1u16.to_le_bytes());
    standard.extend_from_slice(&9u16.to_le_bytes());
    standard.extend_from_slice(&0x0300u16.to_le_bytes());
    standard.resize(32, 0);
    assert!(raw_contents_is_wmf(&standard));
    // EMF: type=1 + offset 40 " EMF"
    let mut emf = 1u32.to_le_bytes().to_vec();
    emf.resize(40, 0);
    emf.extend_from_slice(b" EMF");
    assert!(raw_contents_is_emf(&emf));
    // 수식 봉투/기타는 어느 쪽도 아니다
    assert!(!raw_contents_is_wmf(b"Hwp 5.0 Equation Editor(HwpEq5x)"));
    assert!(!raw_contents_is_emf(b"Hwp 5.0 Equation Editor(HwpEq5x)"));
}

#[test]
fn issue_5725_equation_ole_renders_equation_node() {
    let core = DocumentCore::from_bytes(&sample_bytes(EQ_SAMPLE)).expect("open eq sample");
    let page = core.build_page_render_tree(0).expect("page 1");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let eq = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Equation(eq) => Some(eq),
            _ => None,
        })
        .expect("수식 OLE 가 Equation 노드로 그려져야 한다 (#5725)");
    assert!(
        eq.script.contains("\\OVER"),
        "봉투에서 꺼낸 스크립트가 실려야 한다: {:?}",
        eq.script
    );
    assert!(
        ole_placeholder_labels(&page.root).is_empty(),
        "자리표시가 남아 있으면 안 된다: {:?}",
        ole_placeholder_labels(&page.root)
    );
}

#[test]
fn issue_5724_wmf_contents_ole_renders_vector_image() {
    let core = DocumentCore::from_bytes(&sample_bytes(WMF_SAMPLE)).expect("open wmf sample");
    let page = core.build_page_render_tree(1).expect("page 2");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let has_wmf_svg = nodes.iter().any(|n| match &n.node_type {
        RenderNodeType::RawSvg(raw) => raw.svg.contains("data:image/svg+xml"),
        _ => false,
    });
    assert!(
        has_wmf_svg,
        "CONTENTS WMF 가 SVG data URI 로 그려져야 한다 (#5724)"
    );
    assert!(
        ole_placeholder_labels(&page.root).is_empty(),
        "자리표시가 남아 있으면 안 된다: {:?}",
        ole_placeholder_labels(&page.root)
    );
}
