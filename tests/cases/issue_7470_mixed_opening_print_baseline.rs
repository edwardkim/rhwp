//! 원본 첫 혼합 조각의 정렬과 다음 쪽 내용 소유를 독립 Print로 보호한다.
//! 전 18쪽 Native/fresh WASM TSV·7쪽 확대 판독과 좌표 근거:
//! mydocs/pr/assets/semanticist21_20261019/mixed_opening_frame/README.md
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::{control::Control, table::VerticalAlign},
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

fn source_cell(root: &RenderNode) -> &RenderNode {
    let mut nodes = Vec::new();
    walk(root, &mut nodes);
    let parents: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
            if t.section_index == Some(0) && t.para_index == Some(21)
                && t.control_index == Some(0) && t.row_count == 5 && t.col_count == 2)
        })
        .collect();
    assert_eq!(parents.len(), 1, "원본 표의 단일 물리 조각");
    nodes.clear();
    walk(parents[0], &mut nodes);
    let owners: Vec<_> = nodes
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(c)
            if c.row == 3 && c.col == 1)
        })
        .collect();
    assert_eq!(owners.len(), 1, "같은 원본 행·열의 소유 셀");
    owners[0]
}

fn body(node: &RenderNode) -> String {
    let text = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text(),
        _ => "",
    };
    let mut result: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    for child in &node.children {
        result.push_str(&body(child));
    }
    result
}

#[test]
fn original_mixed_opening_frame_keeps_print_baseline_and_continuation_owner() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rowbreak-problem-pages.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 18, "같은 원본의 독립 Print 전쪽 수");
    let Control::Table(table) = &core.document().sections[0].paragraphs[21].controls[0] else {
        panic!("원본 표");
    };
    let original = table
        .cells
        .iter()
        .find(|c| c.row == 3 && c.col == 1)
        .unwrap();
    assert_eq!(original.vertical_align, VerticalAlign::Center);

    let first = core.build_page_render_tree(6).unwrap();
    let next = core.build_page_render_tree(7).unwrap();
    let owner = source_cell(&first.root);
    let successor = source_cell(&next.root);
    let mut nodes = Vec::new();
    walk(owner, &mut nodes);
    let markers: Vec<_> = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.para_index == Some(0) && run.text.starts_with('①') =>
            {
                Some((*node, run))
            }
            _ => None,
        })
        .collect();
    assert_eq!(markers.len(), 1, "첫 본문 번호의 단일 소유");
    let (marker, run) = markers[0];
    // PDF 7쪽의 ①이용자는... baseline 494.28pt를 96dpi로 환산했다.
    // 같은 페이지의 양쪽 raw가 98% 이상이며 다른 쪽은 관계 검사만 사용한다.
    let print_baseline = 494.28 * 96.0 / 72.0;
    assert!(
        (marker.bbox.y + run.baseline - print_baseline).abs() <= 2.0,
        "원본 Center 정렬: 실제 {}, 독립 Print {print_baseline}, 허용 ±2px",
        marker.bbox.y + run.baseline
    );
    assert!(marker.bbox.y >= owner.bbox.y, "첫 줄은 소유 셀 내부");
    let children: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(1) && t.control_index == Some(0)
                && t.row_count == 1 && t.col_count == 1)
        })
        .collect();
    assert_eq!(children.len(), 1, "호스트 문단의 자식 첫 조각");
    let child = children[0].bbox;
    let parent = owner.bbox;
    assert!(
        marker.bbox.y + marker.bbox.height < child.y,
        "본문 뒤의 자식 순서"
    );
    assert!(
        child.x >= parent.x
            && child.y >= parent.y
            && child.x + child.width <= parent.x + parent.width + 0.05
            && child.y + child.height <= parent.y + parent.height + 0.05,
        "자식 첫 조각은 원래 셀 안에 포함된다"
    );
    let opening = body(owner);
    let closing = body(successor);
    let head = "이용자는클라우드컴퓨팅서비스제공자에게이용자정보가";
    let tail = "하여정보를제공하거나정보의제공을매개하는자를말한다.";
    assert_eq!(opening.matches(head).count(), 1, "첫 쪽의 본문 보존");
    assert!(
        !closing.contains(head),
        "이미 소비한 첫 본문을 반복하지 않는다"
    );
    assert!(
        !opening.contains(tail),
        "다음 쪽의 꼬리를 앞 컷에 넣지 않는다"
    );
    assert_eq!(
        closing.matches(tail).count(),
        1,
        "마지막 자식 줄의 누락·중복 없음"
    );
}

#[test]
fn original_mixed_opening_cli_baseline_matches_independent_print() {
    struct Output(std::path::PathBuf);
    impl Drop for Output {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = Output(root.join(format!(
        "output/pr-review/rowbreak-mixed-opening-{}-{stamp}",
        std::process::id()
    )));
    std::fs::create_dir_all(&out.0).unwrap();
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = std::process::Command::new(binary)
        .arg("export-svg")
        .arg(root.join("samples/rowbreak-problem-pages.hwp"))
        .args(["--compat", "2024", "-p", "6", "-o"])
        .arg(&out.0)
        .output()
        .expect("실제 CLI의 7쪽 출력");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let files: Vec<_> = std::fs::read_dir(&out.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "svg"))
        .collect();
    assert_eq!(files.len(), 1, "선정한 물리 7쪽만 출력");
    let svg = std::fs::read_to_string(&files[0]).unwrap();
    let document = roxmltree::Document::parse(&svg).unwrap();
    let pieces: Vec<_> = document
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .map(|node| {
            let text: String = node
                .text()
                .unwrap_or("")
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .collect();
            (text, node.attribute("y").map(|y| y.parse::<f64>().unwrap()))
        })
        .collect();
    let text: String = pieces.iter().map(|(text, _)| text.as_str()).collect();
    let needle = "이용자는클라우드컴퓨팅서비스제공자에게이용자정보가";
    assert_eq!(
        text.matches(needle).count(),
        1,
        "내용으로 고른 원본 문장의 단일 소유"
    );
    let at = text.find(needle).unwrap();
    let mut offset = 0;
    let mut baseline = None;
    for (text, y) in pieces {
        if offset <= at && at < offset + text.len() {
            baseline = y;
            break;
        }
        offset += text.len();
    }
    let actual = baseline.expect("첫 원문 glyph의 실제 baseline");
    let print = 494.28 * 96.0 / 72.0;
    assert!(
        (actual - print).abs() <= 2.0,
        "실제 CLI baseline {actual}, 독립 Print {print}, 허용 ±2px"
    );
}
