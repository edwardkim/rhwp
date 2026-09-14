//! PR #7143 검토용 메모리 축소 대조. production/CI test에는 등록하지 않는다.
//! 원본 fixture의 한 section/표 문단을 보존하고 나머지 section/문단만 제거한다.
//! 독립 한컴 출력이 있는 문서가 아닌, 같은 IR을 base/PR에 주는 내부 계약 실험이다.
//! 저장소 루트에서 각 검토 head를 빌드한 뒤 실행한다:
//! rustc --edition=2021 mydocs/pr/assets/pr7143_repro_probe.rs \
//!   --extern rhwp=target/pr7143-review-20260914/release-test/deps/librhwp.rlib \
//!   -L dependency=target/pr7143-review-20260914/release-test/deps -o /tmp/pr7143-probe
//! mkdir -p /tmp/pr7143-probe-output
//! RHWP_DIAG_6981=1 /tmp/pr7143-probe /tmp/pr7143-probe-output
//! stdout에 0.5px 초과 Table/Body bbox만 보고하며, 모든 parse/render 실패는 중단한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
fn text(n: &RenderNode) -> String {
    let mut s = String::new();
    if let RenderNodeType::TextRun(r) = &n.node_type {
        s.push_str(&r.text);
    }
    for c in &n.children {
        s.push_str(&text(c));
    }
    s
}
fn scan(n: &RenderNode, body: Option<f64>, out: &mut Vec<String>) {
    if !n.visible || n.editor_only {
        return;
    }
    let b = if matches!(n.node_type, RenderNodeType::Body { .. }) {
        Some(n.bbox.y + n.bbox.height)
    } else {
        body
    };
    if let Some(bottom) = b {
        if matches!(n.node_type, RenderNodeType::Table(_))
            && n.bbox.y + n.bbox.height > bottom + 0.5
        {
            out.push(format!(
                "table_bottom={:.8} body_bottom={:.8}",
                n.bbox.y + n.bbox.height,
                bottom
            ));
        }
    }
    for c in &n.children {
        scan(c, b, out);
    }
}
fn main() {
    let bytes = std::fs::read("samples/task2287/1342000_edu_curriculum_map.hwp").unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let source = core.document();
    let mut target = None;
    for (si, s) in source.sections.iter().enumerate() {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            for c in &p.controls {
                if let Control::Table(t) = c {
                    if t.cells.iter().any(|c| {
                        c.row == 62
                            && c.col == 8
                            && c.paragraphs.iter().any(|p| p.text.contains("선언문"))
                    }) {
                        eprintln!("target section={si} para={pi} rows={}", t.row_count);
                        target = Some((si, pi));
                    }
                }
            }
        }
    }
    let (si, pi) = target.unwrap();
    for height in [source.sections[si].section_def.page_def.height] {
        let mut d = source.clone();
        d.sections = vec![source.sections[si].clone()];
        d.sections[0].paragraphs = vec![source.sections[si].paragraphs[pi].clone()];
        d.sections[0].section_def.page_def.height = height;
        let mut c = DocumentCore::new_empty();
        c.set_document(d);
        let mut out = Vec::new();
        for p in 0..c.page_count() {
            let t = c.build_page_render_tree(p).unwrap();
            let mut xs = Vec::new();
            scan(&t.root, None, &mut xs);
            for x in xs {
                out.push(format!("p{} {}", p + 1, x));
            }
            if p == 2 {
                std::fs::write(
                    format!("{}/probe-page3.svg", std::env::args().nth(1).unwrap()),
                    c.render_page_svg_native(p).unwrap(),
                )
                .unwrap();
                std::fs::write(
                    format!("{}/probe-page3.json", std::env::args().nth(1).unwrap()),
                    t.root.to_json(),
                )
                .unwrap();
            }
            let tx = text(&t.root);
            if tx.contains("선언문") {
                eprintln!("height={height} needlepage={}", p + 1);
            }
        }
        println!("{}\t{}\t{}", height, c.page_count(), out.join(";"));
    }
}
