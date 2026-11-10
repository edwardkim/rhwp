//! 주석·쪽번호 경계: 변경 대상 endnote-forced-break-field-90 전3쪽의 최소 글꼴 Native/fresh WASM raw가 각각100%다.
//! 주석 갱신: 2026-11-10T10:41:54+09:00.
//!
//! 원문/논리 주소, 쪽·줄 소속과 내용 보존을 검사한다. 완료된 3쪽의 중요 위치만
//! 독립 PDF 원점으로 ±2px 고정한다. 출처·TSV·PNG는 아래 README에 연결한다.
//! mydocs/pr/assets/semanticist21_20261022/inline_note_page_fields/README.md
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::DocumentCore,
    model::control::{AutoNumberType, Control},
    renderer::render_tree::{RenderNode, RenderNodeType},
};

const FIXTURES: &str = "tests/fixtures/semanticist21_20261022/inline_note_page_fields";

fn load(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES)
        .join(format!("{name}.hwpx"));
    DocumentCore::from_bytes(&std::fs::read(path).expect("검증한 원본")).unwrap()
}

fn body_lines<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(
        node.node_type,
        RenderNodeType::Header
            | RenderNodeType::Footer
            | RenderNodeType::FootnoteArea
            | RenderNodeType::Table(_)
            | RenderNodeType::TextBox
    ) {
        return;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(1)) {
        out.push(node);
        return;
    }
    for child in &node.children {
        body_lines(child, out);
    }
}

fn near(label: &str, actual: f64, print: f64) {
    assert!(
        (actual - print).abs() <= 2.0,
        "{label}: 실제 {actual}, 독립 Print {print}, 허용 ±2px"
    );
}

/// 검사 유형: 관계형. 원문 주소·소속·본문 순서와 누락·중복을 검사한다.
#[test]
fn partial_page_fields_and_note_slots_keep_source_order_across_pages() {
    for (name, repeats, pages, field_page, note_page, adjacent) in [
        ("endnote-forced-break-field-40", 40, 2, 2, 1, true),
        ("endnote-forced-break-field-90", 90, 3, 3, 3, true),
        ("footnote-forced-break-field-90", 90, 3, 3, 3, true),
        ("early-endnote-and-field-90", 90, 3, 1, 1, false),
        ("early-endnote-late-field-90", 90, 3, 3, 1, false),
    ] {
        let core = load(name);
        assert_eq!(core.page_count(), pages, "{name}: 독립 Print 전체 쪽수");
        let para = &core.document().sections[0].paragraphs[1];
        let field = para
            .controls
            .iter()
            .position(|control| matches!(control, Control::AutoNumber(n) if n.number_type == AutoNumberType::Page))
            .expect("원본 본문 쪽번호");
        let source_position = para.control_text_positions()[field];
        let logical_position = para.logical_control_positions()[field];
        assert_eq!(logical_position, source_position + 1, "{name}: 주석 한 칸");
        assert_eq!(para.text.matches("연결검사").count(), repeats);
        let trees: Vec<_> = (0..pages)
            .map(|page| {
                core.build_page_render_tree(page)
                    .expect("모든 부분 문단의 실제 소유")
            })
            .collect();
        let mut field_owners = Vec::new();
        let mut note_owners = Vec::new();
        let mut text = String::new();
        let mut global_line = 0;
        for (page, tree) in trees.iter().enumerate() {
            let mut lines = Vec::new();
            body_lines(&tree.root, &mut lines);
            for line in &lines {
                for child in &line.children {
                    match &child.node_type {
                        RenderNodeType::TextRun(run) => {
                            text.push_str(run.display_text.as_deref().unwrap_or(&run.text));
                            if run.char_start == Some(logical_position) && run.text == " " {
                                assert_eq!(
                                    run.display_text.as_deref(),
                                    Some(field_page.to_string().as_str())
                                );
                                assert!(child.bbox.x >= line.bbox.x);
                                assert!(
                                    child.bbox.x + child.bbox.width
                                        <= line.bbox.x + line.bbox.width
                                );
                                field_owners.push((page + 1, global_line));
                            }
                        }
                        RenderNodeType::FootnoteMarker(marker) => {
                            assert_eq!(marker.control_index, 0, "{name}: 원문 주석 소유");
                            assert_eq!(marker.text, "1)");
                            note_owners.push((page + 1, global_line));
                        }
                        _ => {}
                    }
                }
                global_line += 1;
            }
        }
        assert_eq!(field_owners.len(), 1, "{name}: 쪽번호 누락·중복");
        assert_eq!(note_owners.len(), 1, "{name}: 주석 참조 누락·중복");
        assert_eq!(
            field_owners[0].0, field_page as usize,
            "{name}: Print 필드 쪽"
        );
        assert_eq!(note_owners[0].0, note_page, "{name}: Print 참조 쪽");
        if adjacent {
            assert!(note_owners[0] < field_owners[0]);
            assert_eq!(
                note_owners[0].1 + 1,
                field_owners[0].1,
                "{name}: 참조 다음 줄의 필드"
            );
        } else if note_page == 1 && field_page == 3 {
            assert!(
                note_owners[0] < field_owners[0],
                "앞쪽 주석의 논리 칸도 이어받는다"
            );
        } else {
            assert_eq!(note_owners[0], field_owners[0], "원문 앞쪽의 주석과 번호");
        }
        let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
        let expected = if field_page == 1 {
            format!("앞1{}뒤", "연결검사".repeat(repeats))
        } else {
            format!("앞{}{field_page}뒤", "연결검사".repeat(repeats))
        };
        assert_eq!(compact, expected, "{name}: 전쪽 본문 순서·누락·중복");
    }
}

/// 검사 유형: 고정형. 완료된3쪽의 독립 Print 원점만 ±2CSSpx로 보호한다.
#[test]
fn completed_note_boundary_keeps_independent_print_origin_in_actual_cli_svg() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = root
        .join(FIXTURES)
        .join("endnote-forced-break-field-90.hwpx");
    let dir = root
        .join("output/pr-review/test-inline-note-field")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    std::fs::create_dir_all(&dir).unwrap();
    let _cleanup = Cleanup(dir.clone());
    let binary =
        std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
    let result = std::process::Command::new(binary)
        .args([
            "export-svg",
            input.to_str().unwrap(),
            "-p",
            "2",
            "--compat",
            "2022",
            "-o",
            dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "실제 출력: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "svg"))
        .collect();
    assert_eq!(files.len(), 1, "지정한 물리 3쪽만 출력");
    let svg = std::fs::read_to_string(&files[0]).unwrap();
    let doc = roxmltree::Document::parse(&svg).unwrap();
    // 본문 참조는 주석 본문보다 먼저 paint된다. px 영역으로 글자를 선택하지 않는다.
    let texts: Vec<_> = doc
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .map(|node| {
            let content = node
                .descendants()
                .filter(|child| child.is_text())
                .filter_map(|child| child.text())
                .collect::<String>();
            (node, content)
        })
        .collect();
    // 같은 번호가 단일 text 또는 확정 전진을 가진 연속 text로 출력될 수 있다.
    // 내용·paint 순서와 같은 줄로 식별하고 독립 Print의 원점 검사는 유지한다.
    let (note_index, note) = texts
        .iter()
        .enumerate()
        .find_map(|(index, (node, content))| {
            if content == "1)" {
                return Some((index, *node));
            }
            if content != "1" {
                return None;
            }
            let (next, suffix) = texts.get(index + 1)?;
            (suffix == ")"
                && node.parent() == next.parent()
                && node.attribute("y") == next.attribute("y"))
            .then_some((index, *node))
        })
        .expect("본문 미주 참조의 전체 번호·같은 줄·paint 순서");
    let (field_index, field) = texts
        .iter()
        .enumerate()
        .find_map(|(index, (node, content))| (content == "3").then_some((index, *node)))
        .expect("물리3쪽의 쪽번호");
    assert!(
        note_index < field_index,
        "본문 참조 다음에 PAGE 필드를 그린다"
    );
    let attribute =
        |node: roxmltree::Node<'_, '_>, name| node.attribute(name).unwrap().parse::<f64>().unwrap();
    // 독립 Print의 device CTM 0.119935/-0.119869와 텍스트 원점을 72→96dpi로 환산.
    near(
        "본문 참조 x",
        attribute(note, "x"),
        4198.0 * 0.119935 * 96.0 / 72.0,
    );
    near(
        "본문 참조 baseline",
        attribute(note, "y"),
        1947.0 * 0.119869 * 96.0 / 72.0,
    );
    near(
        "쪽번호 x",
        attribute(field, "x"),
        709.0 * 0.119935 * 96.0 / 72.0,
    );
    near(
        "쪽번호 baseline",
        attribute(field, "y"),
        2098.0 * 0.119869 * 96.0 / 72.0,
    );
}
