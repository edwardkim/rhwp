//! 개체 번호는 캡션 필드 수가 아니라 소유 개체의 범주·순번에서 결정한다.
//! 독립 Hancom2020 Print의 원본/저장본/범주 대조군 전16쪽을 비교했다.
//! 전쪽 Native/fresh WASM raw2px 최저90% 이상. 본문 필드1쪽은
//! Native99.03275/WASM98.92675%이며 독립 PDF x(CSS96dpi)를 ±2px로 보호한다.
//! 입력·PDF SHA와 선행 검증은 fixtures/issue265_object_number_ownership/README.md.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    model::{
        control::{AutoNumberType, Control},
        paragraph::Paragraph,
        shape::ObjectNumberingType,
    },
    DocumentCore,
};
const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/issue265_object_number_ownership"
);
fn load(path: &str) -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}
fn input(name: &str) -> DocumentCore {
    load(&format!("{FIXTURES}/{name}.hwpx"))
}
fn collect(
    ps: &[Paragraph],
    caption: bool,
    caps: &mut Vec<u16>,
    body: &mut Vec<u16>,
    categories: &mut Vec<ObjectNumberingType>,
    cached: &mut Vec<u16>,
) {
    for p in ps {
        for c in &p.controls {
            match c {
                Control::AutoNumber(a)
                    if matches!(
                        a.number_type,
                        AutoNumberType::Picture | AutoNumberType::Table | AutoNumberType::Equation
                    ) =>
                {
                    if caption {
                        caps.push(a.assigned_number);
                        cached.push(a.number)
                    } else {
                        body.push(a.assigned_number)
                    }
                }
                Control::Picture(p) => {
                    categories.push(p.common.numbering_type);
                    if let Some(cap) = &p.caption {
                        collect(&cap.paragraphs, true, caps, body, categories, cached)
                    }
                }
                Control::Table(t) => {
                    for cell in &t.cells {
                        collect(&cell.paragraphs, false, caps, body, categories, cached)
                    }
                    if let Some(cap) = &t.caption {
                        collect(&cap.paragraphs, true, caps, body, categories, cached)
                    }
                }
                Control::Header(h) => collect(&h.paragraphs, false, caps, body, categories, cached),
                Control::Footer(h) => collect(&h.paragraphs, false, caps, body, categories, cached),
                _ => {}
            }
        }
    }
}
fn painted_caption_numbers(core: &DocumentCore) -> Vec<u16> {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk(n: &RenderNode, owner: Option<usize>, out: &mut Vec<u16>) {
        let owner = if let RenderNodeType::TextLine(line) = &n.node_type {
            line.caption_owner.as_ref().map(|o| o.para_idx).or(owner)
        } else {
            owner
        };
        if owner.is_some() {
            if let RenderNodeType::TextRun(run) = &n.node_type {
                if let Some(number) = run
                    .display_text
                    .as_deref()
                    .and_then(|s| s.parse::<u16>().ok())
                {
                    out.push(number)
                }
            }
        }
        for child in &n.children {
            walk(child, owner, out)
        }
    }
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        walk(
            &core.build_page_render_tree(page).unwrap().root,
            None,
            &mut out,
        )
    }
    out
}
fn painted_body_fields(core: &DocumentCore) -> Vec<String> {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk(n: &RenderNode, out: &mut Vec<String>) {
        if let RenderNodeType::TextLine(line) = &n.node_type {
            if line.caption_owner.is_none() {
                let text = n
                    .children
                    .iter()
                    .filter_map(|c| {
                        if let RenderNodeType::TextRun(run) = &c.node_type {
                            Some(run.display_text.as_deref().unwrap_or(&run.text))
                        } else {
                            None
                        }
                    })
                    .collect::<String>();
                if text.starts_with("Before:") || text.starts_with("After:") {
                    out.push(text.chars().filter(|c| !c.is_whitespace()).collect())
                }
            }
        }
        for child in &n.children {
            walk(child, out)
        }
    }
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        walk(&core.build_page_render_tree(page).unwrap().root, &mut out)
    }
    out
}
fn values(d: &DocumentCore) -> (Vec<u16>, Vec<u16>, Vec<ObjectNumberingType>, Vec<u16>) {
    let (mut c, mut b, mut n, mut raw) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for s in &d.document().sections {
        collect(&s.paragraphs, false, &mut c, &mut b, &mut n, &mut raw)
    }
    assert_eq!(
        painted_caption_numbers(d),
        c,
        "actual displayed caption fields retain owner order without omission or duplication"
    );
    (c, b, n, raw)
}
#[test]
fn none_objects_do_not_inherit_another_object_or_the_start_number() {
    for path in [
        format!("{FIXTURES}/old-none-hwp.hwp"),
        format!("{FIXTURES}/old-none-hwpx.hwpx"),
        format!("{FIXTURES}/none-start7.hwpx"),
    ] {
        let core = load(&path);
        let (c, _, _, cached) = values(&core);
        assert_eq!(
            c,
            [1, 1, 1, 1],
            "independent Print NONE object captions: {path}"
        );
        assert_eq!(
            cached,
            [1, 2, 3, 4],
            "stored source cache is preserved: {path}"
        );
    }
}
#[test]
fn caption_fields_repeat_the_object_number_and_captionless_objects_count() {
    for (key, expected) in [
        ("all-picture", vec![2, 3, 4, 5]),
        ("mixed-picture-none", vec![1, 2, 1, 3]),
        ("duplicate-field-owner", vec![2, 2, 3, 4, 5]),
        ("cross-caption-category", vec![2, 2, 3, 4, 5]),
        ("picture-start7", vec![8, 9, 10, 11]),
    ] {
        let core = input(key);
        assert_eq!(
            values(&core).0,
            expected,
            "independent Print owner numbers: {key}"
        );
    }
}
#[test]
fn unbound_fields_read_the_current_object_without_advancing_it() {
    let core = input("inline-unbound");
    let (c, b, _, _) = values(&core);
    assert_eq!(
        painted_body_fields(&core),
        ["Before:1/1", "After:5/5"],
        "actual body fields retain their own source position and current object number"
    );
    assert_eq!(
        b,
        [1, 1, 5, 5],
        "independent Print leading/trailing current number"
    );
    assert_eq!(
        c,
        [2, 3, 4, 5],
        "ordinary fields do not steal a picture ordinal"
    );
}
#[test]
fn implicit_source_picture_scope_is_preserved_when_saved() {
    let core = load(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue_265.hwp"
    ));
    let (c, _, categories, _) = values(&core);
    assert_eq!(c, [2, 3, 4, 5], "original independent Print");
    assert_eq!(
        categories,
        vec![ObjectNumberingType::Picture; 5],
        "the implicit source picture category is made explicit"
    );
    for bytes in [
        core.export_hwp_with_adapter_snapshot().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            values(&reopened).0,
            c,
            "save/reopen must preserve the displayed source numbers"
        );
    }
}

#[test]
fn retained_number_slots_do_not_leave_an_extra_blank_in_the_final_paint() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn walk(n: &RenderNode, found: &mut Vec<(f64, String, Option<String>)>) {
        if let RenderNodeType::TextLine(line) = &n.node_type {
            if line.caption_owner.is_none()&&n.children.iter().any(|c|matches!(&c.node_type,RenderNodeType::TextRun(r)if r.text.starts_with("Before:"))){
   for c in &n.children{if let RenderNodeType::TextRun(r)=&c.node_type{if r.display_text.as_deref().unwrap_or(&r.text)=="1"{found.push((c.bbox.x,r.text.clone(),r.display_text.clone()))}}}
  }
        }
        for c in &n.children {
            walk(c, found)
        }
    }
    let core = input("inline-unbound");
    let mut fields = Vec::new();
    walk(&core.build_page_render_tree(0).unwrap().root, &mut fields);
    assert_eq!(fields.len(), 2);
    for ((x, _, _), pdf_x) in fields.iter().zip([162.9516805013021, 188.2179972330729]) {
        assert!(
            (x - pdf_x).abs() <= 2.0,
            "independent Print p1 field x: actual={x} expected={pdf_x}"
        );
    }
    for (_, source, display) in &fields {
        assert_eq!(
            source, " ",
            "retained source placeholder is consumed, not drawn in addition to an inserted label"
        );
        assert_eq!(display.as_deref(), Some("1"));
    }
}
