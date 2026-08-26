//! [Issue #5882] 그릴 내용이 없는 OLE 의 진단용 자리표시는 사용자 산출물에 그려지지 않는다.
//!
//! 미리보기·차트·이미지 폴백이 전부 실패한 OLE(3067979 문서의 ole23 — 4바이트
//! 길이 접두어 뒤에 디렉터리 엔트리 0개인 빈 CFB)에 대해 rhwp 는 회색 점선 상자와
//! `OLE 개체 (BinData #N)` 라벨을 export-svg/pdf 산출물에 그렸다. 한글 2022 정본은
//! 같은 자리에 아무것도 그리지 않는다 — 빈 자리다.
//!
//! - 내용 없는 OLE: 렌더 트리에도 자리표시 노드가 남지 않는다(빈 자리 유지).
//! - 미리보기가 있는 정상 OLE(`samples/한셀OLE.hwpx`): 종전대로 렌더된다.
//! - 사유 라벨이 있는 [#5582] 차트 폴백은 이 변경 대상이 아니다.
//!
//! 진단이 필요하면 `RHWP_DIAG_OLE_PLACEHOLDER` 환경 변수로 종전 표시를 복원한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::process::Command;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const HEALTHY_SAMPLE: &str = "samples/한셀OLE.hwpx";
const BROKEN_OLE_ENTRY: &str = "BinData/ole1.ole";

/// 정상 샘플의 OLE payload 를 CFB 매직이 아닌 바이트로 바꾼 HWPX — 모든
/// 미리보기 폴백이 실패하는 개체를 흉내 낸다 (3067979 ole23 변형).
fn broken_ole_hwpx_bytes() -> Vec<u8> {
    let src = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(HEALTHY_SAMPLE))
        .expect("read healthy sample");
    let mut archive = zip::ZipArchive::new(Cursor::new(&src)).expect("open zip");
    let mut out = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut out);
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).expect("zip entry");
            let name = entry.name().to_string();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).expect("read zip entry");
            if name == BROKEN_OLE_ENTRY {
                data = b"NOT-A-CFB-PAYLOAD".to_vec();
            }
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            writer.start_file(name, opts).expect("start_file");
            writer.write_all(&data).expect("write zip entry");
        }
        writer.finish().expect("finish zip");
    }
    out.into_inner()
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn placeholder_labels(root: &RenderNode) -> Vec<String> {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Placeholder(p) => Some(p.label.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn contentless_ole_leaves_an_empty_slot_in_the_render_tree() {
    let bytes = broken_ole_hwpx_bytes();
    let core = DocumentCore::from_bytes(&bytes).expect("open broken-ole variant");
    let page = core.build_page_render_tree(0).expect("page 1");
    let labels = placeholder_labels(&page.root);
    assert!(
        labels.is_empty(),
        "내용 없는 OLE 자리에 자리표시 노드가 남아 있다 — 한글은 빈 자리로 둔다: {labels:?}"
    );
}

fn write_temp_doc(bytes: &[u8], tag: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("rhwp_5882_{}_{}.hwpx", tag, std::process::id()));
    std::fs::write(&path, bytes).expect("write temp hwpx");
    path
}

fn export_svg(sample: &Path, tag: &str) -> String {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = std::env::temp_dir().join(format!(
        "rhwp_5882_svg_{tag}_{}_{}",
        std::process::id(),
        nth
    ));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("출력 디렉토리 생성");
    let rhwp = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    let done = Command::new(rhwp)
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")))
        .args([
            "export-svg",
            sample.to_str().expect("경로"),
            "-p",
            "0",
            "-o",
            out.to_str().expect("출력 경로"),
        ])
        .output()
        .expect("rhwp export-svg 실행");
    assert!(
        done.status.success(),
        "export-svg 실패: {}",
        String::from_utf8_lossy(&done.stderr)
    );
    let svg = std::fs::read_dir(&out)
        .expect("출력 디렉토리")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "svg"))
        .expect("SVG 산출물");
    std::fs::read_to_string(svg).expect("SVG 읽기")
}

#[test]
fn contentless_ole_export_svg_has_no_placeholder_box_or_label() {
    let doc = write_temp_doc(&broken_ole_hwpx_bytes(), "cli");
    let svg = export_svg(&doc, "broken");
    assert!(
        !svg.contains("OLE 개체"),
        "export-svg 에 'OLE 개체' 라벨이 그려진다 — 사용자 산출물의 이물이다"
    );
    assert!(
        !svg.contains("f0f0f0"),
        "회색 점선 상자(#F0F0F0)가 export-svg 에 남아 있다"
    );
}

#[test]
fn healthy_ole_still_renders_its_preview() {
    // 회귀 가드: 자리표시 억제가 정상 OLE 렌더를 건드리지 않는다.
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join(HEALTHY_SAMPLE);
    let svg = export_svg(&sample, "healthy");
    assert!(
        svg.contains("data:image/svg+xml"),
        "정상 OLE 의 미리보기(SVG data URI)가 사라졌다"
    );
    assert!(!svg.contains("OLE 개체"), "정상 OLE 에 자리표시가 그려진다");
}
