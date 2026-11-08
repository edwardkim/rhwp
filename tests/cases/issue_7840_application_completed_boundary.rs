//! #7840: 저장 프레임 이어받기와 마지막 표 조각의 원문 보존.
//! 원본 SHA-256: 0ca950ef7424e197a432df115aabb8a8c4169f9f4a4f0ccce7816c8931bb6744.
//! 독립 한컴 2020 Print PDF 4쪽: 416bc2b227b506a3dfdd172902099dde2f834861b505de6a7b846b9bcd0052c4.
//! c96354d4e6·96dpi·Print·compat2022 전쪽 Native/fresh WASM 최저 99.39534/99.36649%.
//! 4쪽 양쪽 raw 99.93592/99.91154%를 직접 판독했다. 공동 최솟값이 가장 높은 완료 쪽이다.
//! 원문 소유·순서·보존 관계와 독립 PDF 괘선을 ±2px로 함께 보호한다.
//! TSV·대표 PNG·생산자와 입력 해시는 mydocs/pr/assets/issue_7840/application_completed_boundary/README.md.
//! 실제 고정 devel 80968497c에서 5쪽이라 FAIL, 현재 4쪽·마지막 칸 보존으로 PASS를 확인했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn compact(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn text(node: &RenderNode) -> String {
    let mut value = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        value.push_str(&text(child));
    }
    value
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print {expected:.6}px, 실제 {actual:.6}px, 허용 ±2px"
    );
}

#[test]
fn application_final_fragment_preserves_source_rows_and_print_boundary() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7840/1150200_youth_vision2030_application.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        core.page_count(),
        4,
        "독립 Print의 전쪽 수와 불필요한 끝 쪽 방지"
    );
    let source = core.document().sections[0].paragraphs[3]
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some(table.as_ref())
            } else {
                None
            }
        })
        .expect("원본 7행 신청서 표");
    // 앞의 긴 행도 마지막 조각의 쪽수와 함께 보호한다. 원본에서 이미 별도 표인
    // 첫 쪽의 과제 개요는 이 7행 표의 내용으로 합치지 않는다.
    let mut carried = [String::new(), String::new()];
    for page_index in 0..core.page_count() {
        let tree = core.build_page_render_tree(page_index as u32).unwrap();
        let mut descendants = Vec::new();
        walk(&tree.root, &mut descendants);
        for frame in descendants.into_iter().filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(3))
        }) {
            for node in &frame.children {
                if let RenderNodeType::TableCell(cell) = &node.node_type {
                    if cell.row == 0 {
                        assert!(cell.col < 2);
                        carried[cell.col as usize].push_str(&text(node));
                    }
                }
            }
        }
    }
    for col in 0..=1 {
        let original = source
            .cells
            .iter()
            .find(|cell| cell.row == 0 && cell.col == col)
            .unwrap();
        let expected: String = original
            .paragraphs
            .iter()
            .map(|paragraph| {
                // 독립 PDF 2/3쪽의 자동 표제 번호는 IR 본문 문자열에 포함되지 않는다.
                // 본문 원문과 별개인 다섯 번호를 정답지에서 보완하며, 지우고 비교하지 않는다.
                let prefix = match compact(&paragraph.text).as_str() {
                    "교육분과" => "②",
                    "가족분과" => "③",
                    "직업/진로분과" => "④",
                    "문화/여가분과" => "⑤",
                    "환경분과" => "⑥",
                    _ => "",
                };
                format!("{prefix}{}", paragraph.text)
            })
            .collect();
        assert_eq!(
            compact(&carried[col as usize]),
            compact(&expected),
            "앞 조각의 원문 순서·누락·중복도 마지막 쪽수와 함께 보호"
        );
    }
    let page = core.build_page_render_tree(3).unwrap();
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let frames: Vec<_> = nodes.into_iter().filter(|node| {
        matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(3))
    }).collect();
    assert_eq!(frames.len(), 1, "4쪽에 원본 신청서 표 조각은 하나");
    let frame = frames[0];
    // 독립 PDF 괘선의 72pt → 96dpi 좌표다. 선택은 원본 소유로 수행한다.
    near(frame.bbox.x, 77.398666);
    near(frame.bbox.y, 115.233317);
    near(frame.bbox.x + frame.bbox.width, 712.895996);
    near(frame.bbox.y + frame.bbox.height, 1023.840007);
    let cells: Vec<_> = frame
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert_eq!(cells.len(), 12, "4쪽의 여섯 행·두 열, 누락·중복 방지");
    for row in 1..=6 {
        for col in 0..=1 {
            let owned: Vec<_> = cells.iter().copied().filter(|node| {
                matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == row && cell.col == col)
            }).collect();
            assert_eq!(owned.len(), 1, "원본 행 {row}·열 {col} 소유는 유일");
            let original = source
                .cells
                .iter()
                .find(|cell| cell.row == row && cell.col == col)
                .unwrap();
            let expected: String = original
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect();
            assert_eq!(
                compact(&text(owned[0])),
                compact(&expected),
                "행 {row}·열 {col} 원문 순서·누락·중복"
            );
        }
    }
    let last = cells.into_iter().find(|node| {
        matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 6 && cell.col == 1)
    }).unwrap();
    assert!(
        compact(&text(last)).ends_with("연구진선정"),
        "마지막 문구는 마지막 칸 안에 보존"
    );
    let mut descendants = Vec::new();
    walk(last, &mut descendants);
    for node in descendants {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.display_or_text().trim().is_empty() {
                continue;
            }
            let baseline = node.bbox.y + run.baseline;
            assert!(
                baseline >= last.bbox.y && baseline <= last.bbox.y + last.bbox.height + 2.0,
                "마지막 칸의 내용 기준선은 원본 칸 안에 있어야 함"
            );
        }
    }
}
