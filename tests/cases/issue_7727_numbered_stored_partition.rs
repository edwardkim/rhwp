//! 검사 유형: 관계형 — 같은 번호 프레임의 저장 분할과 현재 채움의 문자 소속을 대조한다.
//! 주석 갱신: 2026-11-14T23:23:18+09:00.
//! 2026-11-14 정본PNG Print / compat2022 / 96DPI / raw2px로 원본204쪽을 양쪽 모두 측정했다.
//! 보정 후 대상96쪽 Native/fresh WASM99.72/99.74%, 전쪽 최저98.11/98.08%(2쪽)다.
//! 정식 증거: mydocs/pr/assets/issues7727_7732_20261114/marker-partition/cause.json.
//! 독립 Print96쪽은 `교육훈련` 뒤에서 나뉘지만 저장 캐시는 `교육훈` 뒤에서 나뉜다.
//! 이 검사는 실물의 절대 px를 고정하지 않는다. 원본 제목·번호·상자를 분리하고,
//! 설치 글꼴에 의존하지 않는 공통 메트릭 글꼴로 바꾼 두 경로의 줄 소속을 비교한다.
//! 같은 줄 수에서 경계만 이동하는 경우와 번호 없는 저장 분할 보존을 구분한다.
//! 근거: mydocs/pr/assets/issues7727_7732_20261114/cbta-heading-investigation.json.

use rhwp::{
    model::{paragraph::NumberingMarker, style::HeadType},
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    DocumentCore,
};

fn core(numbered: bool) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    let mut doc = core.document().clone();
    let mut section = doc.sections[0].clone();
    let mut para = section.paragraphs[919].clone();
    assert_eq!(para.line_segs.len(), 2);
    assert_eq!(para.line_segs[1].text_start, 34);
    assert!(para.controls.is_empty());
    let first = para.line_segs[0].vertical_pos;
    for line in &mut para.line_segs {
        line.vertical_pos -= first;
    }
    para.numbering_marker = NumberingMarker::Unresolved;
    let style = &mut doc.doc_info.para_shapes[para.para_shape_id as usize];
    assert_eq!(style.head_type, HeadType::Number);
    if !numbered {
        style.head_type = HeadType::None;
    }
    for family in &mut doc.doc_info.font_faces {
        for font in family {
            font.name = "Malgun Gothic".to_owned();
            font.alt_type = 1;
            font.native_program = None;
            font.subst_font = None;
        }
    }
    section.paragraphs = vec![para];
    doc.sections = vec![section];
    core.set_document(doc);
    core
}

fn lines(core: &DocumentCore) -> Vec<String> {
    let tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    fn collect(node: &LayerNode, out: &mut Vec<(f64, f64, String)>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    collect(child, out);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                collect(child, out);
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::TextRun { run, bbox, .. } = op {
                        if run.char_shape_id == Some(14) {
                            out.push((bbox.y, bbox.x, run.text.clone()));
                        }
                    }
                }
            }
        }
    }
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    runs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut lines: Vec<(f64, String)> = Vec::new();
    for (y, _, text) in runs {
        if let Some((_, line)) = lines.last_mut().filter(|(last, _)| *last == y) {
            line.push_str(&text);
        } else {
            lines.push((y, text));
        }
    }
    lines
        .into_iter()
        .map(|(_, text)| text.chars().filter(|ch| !ch.is_whitespace()).collect())
        .collect()
}

#[test]
fn numbered_same_count_rows_use_the_current_frame_partition() {
    let mut core = core(true);
    let cached = lines(&core);
    core.reflow_body_paragraph_range_native(0, 0..1).unwrap();
    let fresh = lines(&core);
    assert_eq!(cached.len(), 2);
    assert_eq!(fresh.len(), 2);
    assert_eq!(
        cached, fresh,
        "같은 줄 수가 낡은 문자 경계를 인증하지 않는다"
    );
    assert_eq!(
        cached.concat(),
        core.document().sections[0].paragraphs[0]
            .text
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>(),
        "원문 누락·중복 없이 같은 소속을 발행한다"
    );
}

#[test]
fn unnumbered_rows_keep_their_declared_partition() {
    let core = core(false);
    let observed = lines(&core);
    assert_eq!(observed.len(), 2);
    assert!(observed[0].ends_with("교육훈"), "{observed:?}");
    assert!(observed[1].starts_with("련단계"), "{observed:?}");
}
